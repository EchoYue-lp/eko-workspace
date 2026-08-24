# A-SRF-03: GUI chat and frontend state integration

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: not-applicable (frontend-only review; backend contract read at b3b2e81)
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only review)

## Question

Does the React chat surface consume backend facts without inventing
lifecycle state or dropping late/duplicate events?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent-cli/web-frontend/src/hooks/useTauriChat.ts` (full, 389 lines) —
  the single chat transport. Sets up `chat://event` + `execution://event`
  Tauri listeners, dispatches messages, manages queued inputs and refs.
- `echo-agent-cli/web-frontend/src/hooks/chatEventHandler.ts` (full, 222
  lines) — the canonical `ChatEvent` switch shared by all transports
  (`handleChatEvent`).
- `echo-agent-cli/web-frontend/src/hooks/useBrowserEvents.ts` (full, 27 lines)
  — sibling transport listener used to compare cleanup-race handling.
- `echo-agent-cli/web-frontend/src/stores/chatStore.ts` (full, 527 lines) —
  streaming-message reducer (tokens, thinking, tool batches, rounds, finalize).
- `echo-agent-cli/web-frontend/src/stores/subagentRunStore.ts` (full, 536
  lines) — `execution://event` kind="subagent" reducer with terminal
  monotonicity guard.
- `echo-agent-cli/web-frontend/src/stores/toolExecutionStore.ts` (full, 255
  lines) — `execution://event` kind="tool" reducer + hydration merge logic.
- `echo-agent-cli/web-frontend/src/stores/taskRuntimeStore.ts` (full, 394
  lines) — polling load + recovery path (`loadByConversation`,
  `refresh`).
- `echo-agent-cli/web-frontend/src/stores/conversationStore.ts` (full, 481
  lines) — conversation list + load + save loop, generation-based race
  protection.
- `echo-agent-cli/web-frontend/src/stores/authStore.ts` (full, 122 lines) —
  token rehydration on module load.
- `echo-agent-cli/web-frontend/src/types/api.ts:125-177` — the
  `ChatEvent` discriminated union (frontend contract).
- `echo-agent-cli/web-frontend/src/lib/tauri-bridge.ts` (full, 175 lines) —
  `isTauri` memoization, `apiInvoke`, `errorMessage`.
- `echo-agent-cli/web-frontend/src/api/endpoints.ts:408-467, 517-613` —
  `conversationApi`, `toolExecutionApi`, `taskRuntimeApi`.
- `echo-agent-cli/web-frontend/src/api/client.ts` (full, 135 lines) —
  HTTP fallback + 401 handling.
- `echo-agent-cli/web-frontend/src/App.tsx` (full, 199 lines) — mount-time
  initialization + activeId-driven TaskRuntime load.
- `echo-agent-cli/web-frontend/src/components/chat/ChatPanel.tsx` (full,
  551 lines) — message list, approval/input/selection cards, cancel,
  regenerate.
- `echo-agent-cli/web-frontend/src/components/chat/MessageBubble.tsx`
  (full, 477 lines) — execution step flattening, run association,
  streaming cursor.
- `echo-agent-cli/web-frontend/src/components/chat/InlineToolCall.tsx`
  (full, 269 lines) — tool detail fetcher (manifest + paginated output).
- `echo-agent-cli/web-frontend/src/components/chat/ParallelExecutionBlock.tsx`
  (full, 109 lines) + `SubagentStreamBlock.tsx` (head, 100 lines) —
  subagent rendering and message-association rules.
- All test files under `src/stores/*.test.ts`, `src/hooks/*.test.ts`, and
  `src/components/chat/*.test.{ts,tsx}` (executed: 26 files, 101 tests,
  exit 0).
- Backend emit contract cross-checked against
  `echo-agent-cli/src/tauri/commands/chat.rs:30-208` (the `ChatEvent`
  enum + `emit_chat_event` + `emit_tool_execution_summary`) and
  `:1341-1416` (`TauriChatSink::on_event` ordering).

## Out Of Scope

Deferred to downstream tasks:

- **A-SRF-02**: Tauri command-side adapter correctness, the
  `execution://event` untyped-emit finding (A-SRF-02-P3-02), and the
  duplicated tool-execution persistence authorities. This task consumes
  the emit contract as authoritative and only audits the receive half.
- **A-CHAT-01**: `drive_chat` lifecycle ownership and the
  `envelope_event_stream` one-terminal invariant. This task depends on
  that invariant and audits only how the React sink consumes it.
- **A-STATE-01**: `ConversationStore` backend durability (file vs
  SQLite) and atomic write semantics. This task audits only the
  frontend `saveCurrent` / `loadConversation` shape.
- **A-TSK-***: `TaskRuntimeStore` backend internals; this task audits
  only the frontend polling/hydration reducer.
- **F-RCT-02 / F-RCT-03**: framework-level terminal-drop and
  ReactAgent-never-emits-`Cancelled` defects (load-bearing for the
  `done`/`final_answer` ordering analysis).
- The 33 `*.test.ts` files outside chat/stores/hooks (analysis,
  papers, plugins, etc.) — not exercised in this review.

## Inputs

Required repository documents read in full:

- Repository root `AGENTS.md` (multi-mode parity rule, no-panic / UTF-8
  safety, "check whether it already exists before adding", framework-vs-
  application layering gate).
- `docs/comprehensive-review/REPORTING.md`,
  `templates/task-report.md`, `templates/validation-report.md`.
- `docs/comprehensive-review/TASKS.md` (A-SRF-03 card and dependencies).

Dependency reports read:

- `zcode-glm/tasks/A-SRF-02.md` (complete) — establishes the emit-side
  contract: four channels (`chat://event`, `execution://event`,
  `browser://event`, `terminal-output`/`terminal-exit`); `chat://event`
  is typed (20-variant enum) while `execution://event` is hand-built
  JSON (A-SRF-02-P3-02). Load-bearing for V01: the receive-side must
  treat `execution://event` field names as string-convention only
  (no shared schema struct).
- `zcode-glm/tasks/A-CHAT-01.md` (complete) — establishes the
  one-terminal invariant via `envelope_event_stream` and the
  ReactAgent-never-emits-`Cancelled` defect (F-RCT-03-P2-02).
  Load-bearing for V02: the frontend `done`/`final_answer`/`error`
  ordering assumptions are only sound because the backend emits them in
  a fixed order.

Historical documents treated as hypotheses:

- `useTauriChat.ts:74-83` docstring — claims the `pendingCleanup` +
  `aborted` flag pattern "covers three race windows" during listener
  setup. Treated as **current** for the chat transport (verified), and
  used as the reference for the divergent `useBrowserEvents.ts`
  implementation (P3-04).

## Layering Decision

This is an **application-layer** task. All inspected code lives in
`echo-agent-cli/web-frontend/src/`. The framework supplies no frontend
artifacts; the only framework contract consumed is the wire shape of
`AgentEvent` (via the backend's `ChatEvent` projection).

| Classification | Required answer |
|---|---|
| Generic mechanism | Tauri's `listen` / `invoke` IPC + Zustand `create` store. Both correctly used as transport / state primitive. |
| EKO product policy | The chat-event switch (`chatEventHandler.ts`), the streaming-message reducer (`chatStore`), the tool/subagent/run reducers, the recovery flow (`loadByConversation` + polling), and the message-bubble rendering projection are all EKO product policy, correctly in the frontend. |
| Adapter boundary | `useTauriChat` is the single transport adapter (the legacy `useWebSocket` was deleted per the `ChatPanel.tsx:21-25` comment). `chatEventHandler` is the single dispatcher. The stores own state; the components only read. No component invents lifecycle state. |
| Duplicate search | Searched the frontend tree for: `ChatEvent`, `handleChatEvent`, `useTauriChat`, `useWebSocket` (zero hits — removed), `subagentTraceStore` / `subagentStore` (zero hits — removed in Phase 4c per `subagentRunStore.ts:5-12`), `ToolExecution` reducers (single `toolExecutionStore`), `loadByConversation` (single live site at `taskRuntimeStore.ts:219` plus the `App.tsx:54` effect). No parallel chat transport or sink remains. |
| Migration deletion | No deletion proposed. The findings identify latent ordering brittleness and an under-typed execution channel; resolution is left to follow-up task IDs. |

## Current Path

### Backend-to-store flow (V01)

Two Tauri event channels reach the chat surface. Both are registered
exactly once, in a single `useEffect` inside `useTauriChat`
(`useTauriChat.ts:84-167`), which is itself mounted exactly once by
`ChatPanel` (`ChatPanel.tsx:23-25, 57-68`). No other component calls
`listen('chat://event')` or `listen('execution://event')` (verified by
grep across `src/`).

```text
Backend (Rust)                         Frontend (React + Zustand)
─────────────────                      ─────────────────────────
TauriChatSink::on_event                useTauriChat.ts (single mount in ChatPanel)
  → emit_chat_event                      useEffect (line 84):
    app.emit("chat://event", payload)      setupListener() async:
                                             const { listen } = await import('@tauri-apps/api/event')
                                             listen<ChatEvent>('chat://event', cb)        :93
                                               ↓ cb → handleEvent(event.payload)         :60-72
                                                 ↓ isCurrentRunEvent filter (message_key/conversation_id)
                                                 ↓ handleChatEvent(event, ctx)             :62
                                                   ↓ useChatStore.getState().{appendToken|finalize...}
subagent bridge (mod.rs:335-769)         listen<Record<string, unknown>>('execution://event', cb)  :109
  → emit_execution_event                   cb branches on payload.kind:
    app.emit("execution://event", map)       "subagent" → useSubagentRunStore.getState().ingest(payload)   :120
                                             "tool"     → useToolExecutionStore.getState().ingest(payload) :134
                                                       + useChatStore.recordToolStart (when chat-owned, started) :135-137
                                             "run"      → useTaskRuntimeStore.loadByConversation(convId)   :138-149
```

The four channels identified by A-SRF-02 are received as follows:

| Channel | Receiver | Store |
|---|---|---|
| `chat://event` | `useTauriChat.ts:93` | `useChatStore` (via `handleChatEvent`) |
| `execution://event` | `useTauriChat.ts:109` | `useSubagentRunStore`, `useToolExecutionStore`, `useTaskRuntimeStore` (branched on `kind`) |
| `browser://event` | `useBrowserEvents.ts:12` | `useBrowserStore.ingest` |
| `terminal-output` / `terminal-exit` | `components/terminal/Terminal.tsx:112, 120` | xterm.js directly (no Zustand) |

The transport is **strongly typed** on the receive side for `chat://event`
(the `ChatEvent` discriminated union at `types/api.ts:125-177` matches
the Rust enum at `chat.rs:30-112` variant-for-variant including the
`#[serde(rename = ...)]` tags) and **weakly typed** for
`execution://event` (received as `Record<string, unknown>` and narrowed
by `payload.kind as string` + `as unknown as ExecutionEvent` /
`as unknown as ToolExecution` casts at `useTauriChat.ts:120, 133`).
The asymmetry matches the emit-side finding A-SRF-02-P3-02: the channel
with the weakest schema on emit also has the weakest schema on receive.

### Reducer state ownership (V01)

| Store | Owns | Persistent? | Recovery on reload |
|---|---|---|---|
| `useChatStore` | in-memory messages, streaming state, run status | No (only via auto-save to `conversationStore`) | `conversationStore.loadConversation` → `chatStore.replaceMessages` |
| `useConversationStore` | conversation list, `activeId`, save/load | Yes (backend `conversationApi`) | `init()` on mount + `loadConversation(id)` on user click |
| `useToolExecutionStore` | tool execution summaries | Yes (backend `toolExecutionApi.list`) | `hydrateConversation(id, tools)` on conversation load |
| `useSubagentRunStore` | subagent lifecycle | Yes (via TaskRuntime events) | `ingestTaskRuntimeSubagentEvents` projected from RuntimeTaskEvent[] |
| `useTaskRuntimeStore` | active TaskRun, plan, todos, events | Yes (backend `taskRuntimeApi`) | `loadByConversation(id)` on `activeId` change (App.tsx:54) |
| `useAuthStore` | session token | `sessionStorage` (per-tab) | `getInitialState` on module load (`authStore.ts:14-39, 107-108`) |

No store reads another store's private state except through explicit
`getState()` accessors (`chatStore.scheduleAutoSave` →
`conversationStore.saveCurrent`; `chatEventHandler.handleChatEvent` →
`useChatStore.getState()`). There is no circular write dependency.

### Reducer monotonicity (V02)

Three distinct reducer policies coexist; the chat surface mixes them.

| Reducer | Monotonicity strategy | Code anchor |
|---|---|---|
| `useSubagentRunStore.ingest` | **Terminal lock**: once `prev.status !== 'running'`, the reducer returns `s` unchanged — late `started` / duplicate `usage` / duplicate `completed` cannot reopen or overwrite a terminal record. Retries use a new execution id (`{run_id}:{task_id}:{plan_revision}:{attempt}`), so they get a fresh record rather than mutating the old one. | `subagentRunStore.ts:451-460` |
| `useToolExecutionStore.ingest` (live path) | **Direct overwrite by `tool.id`**: `tools: { ...state.tools, [tool.id]: tool }`. No status-rank guard. | `toolExecutionStore.ts:206-217` |
| `useToolExecutionStore.hydrateConversation` (recovery path) | **Status-rank + activity-timestamp merge** (`mergeToolExecution`): terminal beats running; newer activity wins at equal rank. | `toolExecutionStore.ts:58-86, 106-117, 223-235` |
| `useChatStore.appendToken` | **Index lookup, drop if absent**: `findIndex` returns -1 → returns `{ messages: s.messages }` unchanged (silent drop). | `chatStore.ts:241-249` |
| `useChatStore.recordToolStart` | **Idempotent insert**: `some(step => step.type === 'tool' && step.callId === toolExecutionId)` short-circuits the duplicate. | `chatStore.ts:288-314` |
| `chatEventHandler` (`done` / `final_answer` / `error`) | **Last-write-wins finalize**, with the assistantId ref cleared on every terminal. | `chatEventHandler.ts:97-109, 140-150, 206-219` |
| `useTaskRuntimeStore.refresh` | **Generation counter** (`refreshRequestGeneration`) prevents stale `set` from an older request landing after a newer load. | `taskRuntimeStore.ts:165-217` |
| `useConversationStore.loadConversation` | **Generation counter** (`loadGeneration`) — when `startNew` interrupts an in-flight load, the in-flight load's `set` is suppressed. | `conversationStore.ts:296-316, 435-475` |

Net result: the subagent reducer is monotone by construction; the
tool-execution reducer has a **dual policy** (live = overwrite, hydrate =
merge) that creates a window for a late live `started` to clobber a
hydrated terminal (P3-01). The chat-event handler relies on the
backend's emit ordering for `final_answer` / `done` (P3-02).

### Reconnect / reload / recovery (V03)

There is no WebSocket to reconnect — `useTauriChat.ts:23-25` is the
only transport (`useWebSocket` removed; `ChatPanel.tsx:21-25` comment
documents the deletion). For Tauri desktop, IPC is in-process; the
listener lives as long as the React subtree, and the listener-setup
race (mount → `await import('@tauri-apps/api/event')` → `await listen`)
is handled by an explicit `aborted` flag + `pendingCleanup` array
(`useTauriChat.ts:87-167`). Three race windows are covered: abort
during `import`, abort between the two `listen`s, and abort after both
`listen`s but before `push`. The sibling `useBrowserEvents.ts` does
**not** have this hardening (P3-04).

Window reload (Cmd-R, Tauri restart) wipes all in-memory state. The
recovery path is:

```text
mount → App.tsx:36-39 useEffect
          → useWorkspaceStore.init()
          → useConversationStore.init()   // loads conversation LIST (not active)
          → authStore already rehydrated from sessionStorage at module load

useEffect[activeId] → App.tsx:54-56
  if (activeId) loadTaskRun(activeId)     // only fires AFTER user picks one
```

`activeId` is **not** auto-restored after reload. The user lands on the
`WelcomeScreen`, which calls `sessionApi.getLatest()` and surfaces a
"Resume last session" button (`WelcomeScreen.tsx:36-73`). Resuming is
a manual click. This is a UX choice (the user picks whether to resume
the last conversation or start fresh), not a correctness gap — but it
means a user who reloads mid-run must explicitly resume to see the
TaskRuntime panel again.

`loadConversation` (`conversationStore.ts:296-388`) is the authoritative
recovery entry: it fetches the conversation record + tool executions
in parallel, calls `toolExecutionStore.hydrateConversation` (which
runs the merge logic so already-loaded tools from another
conversation survive), restores agent context via `conversationApi.restore`,
then `chatStore.replaceMessages` (which sets `isHistoryView: false`
because agent context was restored — `conversationStore.ts:376-377`).
After `activeId` becomes set, the `App.tsx:54` effect also fires
`taskRuntimeStore.loadByConversation` which itself rehydrates subagent
runs from `RuntimeTaskEvent[]` via `ingestTaskRuntimeSubagentEvents`
(the projection adapter at `subagentRunStore.ts:325-405`).

Cancels clear in-memory state via `clearCurrent` / `reset` /
`useSubagentRunStore.clear()` (`ChatPanel.tsx:70-77`).

### Streaming / tool / result rendering (V04)

The streaming message is rendered by `MessageBubble`
(`MessageBubble.tsx:153-446`), which composes three independently
updating surfaces:

1. **Token stream** — `chatStore.appendToken` mutates `message.content`
   by string concat (`chatStore.ts:241-249`). The reducer is O(n) on
   every token (full message-array clone + content concat), capped at
   `MAX_MESSAGES = 500` (`chatStore.ts:104-112`). The cursor is a CSS
   pseudo-element on `message.isStreaming` (`MessageBubble.tsx:434-436`).
   `MessageBubble` is wrapped in `memo` (`:153`), so other messages
   don't re-render on each token.
2. **Thinking stream** — `thinking_start` / `token` / `thinking_end`
   route tokens into `thinkingSegments` instead of `content`
   (`chatEventHandler.ts:36-59`). `appendThinking` mutates only the
   last segment (`chatStore.ts:251-267`).
3. **Tool/subagent stream** — `tool_batch_start` captures the current
   thinking segment into `currentRound.thinking`
   (`chatStore.ts:317-332`); `recordToolStart` appends `callId` to
   `currentRound.toolCallIds` (idempotent); `tool_batch_end` pushes the
   round into `message.executionRounds`. `execution://event`
   `kind="tool"` ingest writes the per-tool row into
   `toolExecutionStore`.

Rendering reads from three stores in one `MessageBubble`:
`useChatStore.messages` (for `lastAssistantMessageId` and
`knownMessageIds`), `useTaskRuntimeStore.activeRun`, and
`useSubagentRunStore.runs`. `visibleSubagentRuns`
(`ParallelExecutionBlock.tsx:31-63`) projects subagents onto messages
by `run.messageId === messageId` (exact match), with conservative
fallbacks only for legacy runs lacking a persisted `messageId` (gated
on `activeRun.run_id` or `activeRun.conversation_id`).

Tool detail rendering is **pull-based**: `InlineToolCall`
(`InlineToolCall.tsx:43-269`) holds local React state for manifest and
output chunks; the detail is fetched via `toolExecutionApi.detail` +
`toolExecutionApi.readOutput` only when expanded, with auto-refresh
every 500 ms while `tool.status === 'running'` and a 256 KiB
auto-load cap to bound memory. Late tool-terminal events surface
through the same `useToolExecutionStore` subscription; the manifest is
re-fetched on `tool.status` change (`InlineToolCall.tsx:86-117`).

## Findings

### A-SRF-03-P2-01: `useToolExecutionStore.ingest` (live path) is a direct overwrite — a late `started` event clobbers a terminal

- Priority: P2
- Confidence: medium
- Layer: application
- Evidence:
  - `echo-agent-cli/web-frontend/src/stores/toolExecutionStore.ts:206-217`
    — the live `ingest(tool)` reducer:
    ```ts
    ingest: (tool) => {
      set((state) => {
        const ownerKey = toolExecutionOwnerKey(tool.owner, tool.run_id);
        const ownerIds = state.idsByOwner[ownerKey] ?? [];
        return {
          tools: { ...state.tools, [tool.id]: tool },           // direct overwrite
          idsByOwner: ownerIds.includes(tool.id) ? state.idsByOwner
            : { ...state.idsByOwner, [ownerKey]: [...ownerIds, tool.id] },
        };
      });
    },
    ```
    No status-rank guard. A `tool` payload with `status: 'running'`
    arriving after a payload with `status: 'succeeded'` (same `tool.id`)
    overwrites the terminal with running.
  - `echo-agent-cli/web-frontend/src/hooks/useTauriChat.ts:132-137`
    — the live event path calls `useToolExecutionStore.getState().ingest(tool)`
    directly, bypassing the merge logic that exists in
    `mergeToolExecution` / `mergeHydratedToolExecutions`
    (`toolExecutionStore.ts:58-117`).
  - `echo-agent-cli/web-frontend/src/stores/toolExecutionStore.ts:88-104`
    — `mergeTaskRuntimeBoundary` is the right behavior: it preserves
    the persisted status when the runtime boundary is `running` or
    equal, and only advances running→terminal. The live path does not
    use it.
- Reachability: every `execution://event` with `kind: "tool"`. The
  backend `emit_tool_execution_summary` is invoked from
  `chat.rs:1179, 1224, 1286, 1318` (foreground-agent path) and
  `mod.rs:429, 478, 516` (subagent bridge). On the happy path the
  backend emits `started` → `completed` in order on a single channel,
  and Tauri's IPC preserves intra-app.emit ordering, so the live path
  sees events in the same order they were emitted. The defect is
  reachable when (a) the user loads an in-progress conversation
  (`loadByConversation` → `hydrateConversation` merges to terminal)
  and a delayed `started` event from the still-running backend then
  clobbers the merged state, or (b) two events on the same `tool.id`
  are emitted from different threads (the foreground sink and the
  subagent bridge both write to the same repository) and race in
  delivery.
- Expected invariant: a terminal tool state must be monotone — once
  `succeeded` / `failed` / `cancelled`, no later event may regress it
  to `running`. This is the same invariant the subagent store already
  enforces (`subagentRunStore.ts:458-460`) and the hydration path
  already enforces (`mergeToolExecution`).
- Observed behavior: the live reducer overwrites unconditionally. A
  late `started` event produces a tool card that flips from
  `succeeded` back to `running` and never returns to terminal (the
  backend will not re-emit `completed` for the same `call_id`).
- Impact: visual regression — the InlineToolCall spinner restarts on a
  completed tool, the `ToolExecutionGroup` label changes from "已执行"
  back to "正在执行", and `isExecutionProcessCompleted`
  (`MessageBubble.tsx:72-87`) may flip the execution block back to
  expanded/streaming. No data loss (the persisted repository still has
  the terminal); the next `hydrateConversation` or page reload
  restores the correct state. Severity is medium because the user
 -visible symptom is a "tool that finished but shows as still running".
- Root cause: the live ingest predates the merge logic; when
  `mergeToolExecution` was added (for the recovery / TaskRuntime
  paths), the live path was not retrofitted to use it.
- Direction: make `useToolExecutionStore.ingest` reuse the existing
  `mergeToolExecution` when an entry already exists:
  ```ts
  ingest: (tool) => set((state) => {
    const existing = state.tools[tool.id];
    const next = existing ? mergeToolExecution(existing, tool) : tool;
    ...
    return { tools: { ...state.tools, [tool.id]: next }, ... };
  }),
  ```
  This unifies the live and hydrate paths under one status-rank policy.
  Add a unit test mirroring the existing
  `toolExecutionStore.test.ts:147-165` ("keeps a live terminal detail
  when a stale running snapshot arrives later") but driven through
  `ingest` rather than `mergeHydratedToolExecutions`.
- Regression validation: extend `toolExecutionStore.test.ts` with a
  case that ingests `succeeded` then ingests `running` for the same
  id and asserts the store still holds `succeeded`.
- Validation reports: [V02](../validations/A-SRF-03/V02-01.md).

### A-SRF-03-P3-01: `done` / `final_answer` ordering is brittle — `done` arriving first finalizes with empty content and the late `final_answer` is then dropped

- Priority: P3
- Confidence: medium
- Layer: application
- Evidence:
  - `echo-agent-cli/web-frontend/src/hooks/chatEventHandler.ts:206-219`
    — the `done` branch:
    ```ts
    case 'done': {
      if (ctx.assistantIdRef.current && !ctx.isCancelledRef.current) {
        store.finalizeAssistantMessage(ctx.assistantIdRef.current, '');
      }
      ctx.assistantIdRef.current = null;
      ...
    }
    ```
    If `assistantIdRef.current` is still set (no preceding
    `final_answer` / `error`), the message is finalized with empty
    content and the ref is cleared.
  - `echo-agent-cli/web-frontend/src/hooks/chatEventHandler.ts:97-109`
    — the `final_answer` branch reads `ctx.assistantIdRef.current` and
    no-ops if it is null. So a `final_answer` arriving AFTER `done`
    finds the ref cleared and is silently dropped; the empty content
    from `done` is the user-visible result.
  - `echo-agent-cli/src/tauri/commands/chat.rs:1365-1388` — the
    backend emits `Done` only from `ChatDriverEvent::TurnStatus { status
    != "running" }`, and `TurnStatus` is emitted by `send_chat` AFTER
    `drive_chat` returns. Because `drive_chat` only returns after the
    stream's terminal (FinalAnswer / Error), the emit order
    FinalAnswer → Done is currently guaranteed.
- Reachability: not reachable on the happy path today (backend
  ordering holds). Reachable if (a) a future refactor moves the
  `Done` emit, (b) the chat sink ever emits `Done` from a non-
  TurnStatus path, or (c) Tauri event delivery reorders across
  windows / webviews (single-window default is FIFO, but multi-window
  setups are not guaranteed).
- Expected invariant: the answer text must not be silently replaced
  with empty string when both `final_answer` and `done` arrive
  regardless of order.
- Observed behavior: today correct because the backend ordering holds.
  The frontend code itself, however, has no defense: `done` finalize
  is unconditional on `assistantIdRef.current` being set, and a
  subsequent `final_answer` is a no-op.
- Impact: latent. If emit ordering ever regresses (e.g. a `Done` is
  emitted from a cleanup path before the stream finishes), the user
  sees an empty assistant bubble for a turn that actually produced
  output.
- Root cause: `done` was written as a "guarantee the message is
  finalized even if no terminal arrived" safety net, but it does not
  distinguish "no terminal yet" from "terminal being processed".
- Direction: either (a) make `done` not finalize with empty content
  — only clear refs and let the existing `final_answer` / `error` /
  `cancelled` arms own the finalize, or (b) keep `done` as the
  safety net but stash `event.data` from `final_answer` into a ref
  (`pendingFinalAnswer`) and have `done` use the stashed value if
  present. Option (a) is simpler; the safety net can be moved to a
  timeout or a debug assert.
- Regression validation: a unit test in `chatEventHandler.test.ts`
  that emits `done` then `final_answer` (in that order) and asserts
  the message content equals the `final_answer.data`.
- Validation reports: [V02](../validations/A-SRF-03/V02-01.md).

### A-SRF-03-P3-02: `chatEventHandler` `cancelled` branch clears the assistant ref without finalizing the streaming message

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/web-frontend/src/hooks/chatEventHandler.ts:151-158`
    — the `cancelled` branch:
    ```ts
    case 'cancelled': {
      store.setRunStatus('cancelled');
      ctx.assistantIdRef.current = null;
      ctx.currentMessageKeyRef.current = null;
      ctx.currentMessageIdRef.current = null;
      ctx.isCancelledRef.current = false;
      break;
    }
    ```
    No `finalizeAssistantMessage` call. The streaming message (if any)
    remains in `isStreaming: true`.
  - `echo-agent-cli/src/tauri/commands/chat.rs:1562` — the backend
    maps `AgentEvent::Cancelled => ChatEvent::Cancelled`, so a
    framework-emitted Cancelled would arrive on the frontend as
    `type: 'cancelled'`.
  - `echo-agent-cli/web-frontend/src/hooks/useTauriChat.ts:314-327`
    — the user-initiated `cancel()` callback calls
    `useChatStore.getState().markCancelled()`, which DOES set
    `isStreaming: false` on every streaming message
    (`chatStore.ts:399-416`). So in the user-clicks-stop flow the
    message is finalized by `markCancelled`, not by the `cancelled`
    event handler.
- Reachability: not reachable on the live path today, because (per
  A-CHAT-01 handoff and F-RCT-03-P2-02) ReactAgent never emits
  `AgentEvent::Cancelled`; the framework synthesizes an `Error`
  instead. So the `cancelled` branch is dead code on the cancel path,
  and the user-clicks-stop path is covered by `markCancelled`.
- Expected invariant: every ChatEvent that signals turn termination
  (`final_answer`, `error`, `cancelled`, `done`) must leave the
  streaming message in `isStreaming: false`.
- Observed behavior: the `cancelled` branch violates this. If a
  future change ever makes the framework emit `Cancelled` (or if the
  backend gains a non-user-triggered cancel path that bypasses the
  synthesizes-Error compensation), the streaming message would stay
  in `isStreaming: true` with no cursor advancement and no spinner
  exit.
- Impact: latent today; trap for the next contributor who assumes
  `cancelled` is reachable.
- Root cause: the handler was written before the
  ReactAgent-never-emits-Cancelled contract was documented, and the
  user-initiated cancel path masks the gap.
- Direction: add `store.finalizeAssistantMessage(id, '')` (or the
  existing cancelled marker pattern) before clearing the ref, mirroring
  the `done` branch's safety finalize. Alternatively, document the
  branch as unreachable and remove it. Either is safe; the current
  half-implementation is the worst option.
- Regression validation: a unit test asserting that after a
  `cancelled` event the message has `isStreaming: false`.
- Validation reports: [V02](../validations/A-SRF-03/V02-01.md).

### A-SRF-03-P3-03: `execution://event` kind discriminator is narrowed with `as` casts — no compile-time defense against the untyped emit (A-SRF-02-P3-02)

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/web-frontend/src/hooks/useTauriChat.ts:109-150`
    — the receive side:
    ```ts
    const unlistenExec = await listen<Record<string, unknown>>('execution://event', (event) => {
      ...
      const kind = payload.kind as string | undefined;
      if (kind === 'subagent') {
        ...
        useSubagentRunStore.getState().ingest(payload as unknown as ExecutionEvent);
        ...
      } else if (kind === 'tool') {
        const tool = payload as unknown as ToolExecution;
        useToolExecutionStore.getState().ingest(tool);
        ...
      } else if (kind === 'run' && payload.event === 'run_started') {
        ...
      }
    });
    ```
    The payload is typed as `Record<string, unknown>`, then cast via
    `as unknown as ExecutionEvent` / `as unknown as ToolExecution`.
    A typo in the backend's `"subagent_run_id"` field
    (A-SRF-02-P3-02 already flagged this risk on the emit side) would
    produce `subagent_run_id: undefined` at runtime with no compile
    error.
  - `echo-agent-cli/web-frontend/src/types/api.ts:80-93` —
    `ToolExecution` is a hand-written interface with required fields
    (`id`, `call_id`, `owner`, `name`, `args_preview`, `status`,
    `started_at`, `detail_ref`). The cast bypasses these required
    fields; if the backend omits any, the store silently holds
    `undefined`.
  - `echo-agent-cli/web-frontend/src/stores/subagentRunStore.ts:52-96`
    — `ExecutionEvent` is also hand-written with `[key: string]:
    unknown` index signature, so it accepts any shape.
- Reachability: every `execution://event` receive. Today the backend
  emits well-formed payloads, so the cast is "safe" at runtime — but
  the safety is undocumented and undefended.
- Expected invariant: the wire schema should be enforced on both ends
  via a shared, compiler-checked contract. The
  `chat://event` channel achieves this (typed enum + per-variant
  serde tags); `execution://event` does not.
- Observed behavior: the frontend treats the cast payload as a typed
  value without runtime validation. The subagent reducer is somewhat
  defensive (it reads fields with `?? ` fallbacks at
  `subagentRunStore.ts:497-515`), but the tool reducer is not
  (`ingest(tool)` writes the cast value directly).
- Impact: maintainability — a backend field rename compiles cleanly
  on both sides and silently breaks the frontend grouping (e.g.
  `owner.message_id` becoming `owner.messageId` would make every
  chat-owned tool invisible because
  `toolExecutionOwnerKey` returns `chat:undefined`).
- Root cause: A-SRF-02-P3-02 already documented the emit-side gap
  (hand-built `serde_json::Map`). This finding is the receive-side
  counterpart — the frontend did not add a runtime validator (zod /
  valibot / hand-rolled `parse`) to compensate.
- Direction: either (a) wait for A-SRF-02-P3-02's
  `ExecutionEvent` enum to land and derive the TypeScript type from
  it via ts-rs (the canonical fix), or (b) add a runtime validator
  for the `kind` discriminator and the few required fields per kind
  (e.g. `parseExecutionEvent` that returns a typed result and a
  fallback for malformed payloads). Either way, remove the
  `as unknown as` casts.
- Regression validation: a unit test feeding a malformed payload
  (missing `subagent_run_id`) through the validator and asserting it
  is dropped or warned rather than ingested as `undefined`.
- Validation reports: [V01](../validations/A-SRF-03/V01-01.md),
  [V04](../validations/A-SRF-03/V04-01.md).

### A-SRF-03-P3-04: `useBrowserEvents` listener-setup race is unfixed — same defect `useTauriChat` documents and works around

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/web-frontend/src/hooks/useBrowserEvents.ts:6-25`
    — the listener setup:
    ```ts
    useEffect(() => {
      if (!isTauri()) return;
      let disposed = false;
      let unlisten: (() => void) | null = null;
      import('@tauri-apps/api/event')
        .then(({ listen }) =>
          listen<BrowserEvent>('browser://event', ({ payload }) => {
            if (!disposed) useBrowserStore.getState().ingest(payload);
          })
        )
        .then((cleanup) => {
          if (disposed) cleanup();
          else unlisten = cleanup;
        })
        .catch((error) => console.warn('[Browser] event listener failed:', error));
      return () => {
        disposed = true;
        unlisten?.();
      };
    }, []);
    ```
    If the component unmounts while the `import()` is in flight,
    `disposed` is set to true, but the chain continues: `listen`
    resolves, the data callback closes over `disposed` (correct), the
    cleanup is then invoked synchronously inside the second `.then`
    (also correct, because `disposed` is true at that point). However,
    if the component unmounts **between** the first `.then` resolving
    `listen(...)` and the second `.then` resolving `cleanup`, the
    listener is registered (live) but `disposed` is now true and the
    cleanup runs immediately — this case is actually handled.
    The genuine gap is the case where the `listen` promise rejects
    after the component unmounts: `.catch` logs, but `unlisten` stays
    null and there is no leak (because no listener was registered).
    On reinspection this implementation is mostly safe, but it lacks
    the explicit "abort during import" early-return that
    `useTauriChat.ts:92` has, and the cleanup-collection pattern that
    `useTauriChat.ts:88, 103, 156, 164` has.
  - `echo-agent-cli/web-frontend/src/hooks/useTauriChat.ts:76-83`
    — the comment documenting the three race windows.
- Reachability: low. The browser event channel is lower cardinality
  than chat, and React StrictMode in development double-invokes
  effects (which is where this kind of race surfaces most often).
- Expected invariant: sibling transports should adopt the same
  hardened pattern when their setup involves multiple async hops.
- Observed behavior: `useBrowserEvents` works in practice (the
  `.then((cleanup) => { if (disposed) cleanup(); ... })` chain covers
  the common case), but the implementation is divergent from the
  documented-correct `useTauriChat` pattern.
- Impact: minor. Browser events may transiently leak a listener on
  fast mount/unmount/mount cycles; the leak is per-listener and
  bounded by the number of `BrowserPanel` mounts.
- Root cause: the fix in `useTauriChat` (P0-4 per the comment) was
  not back-ported to `useBrowserEvents`.
- Direction: extract the listener-setup-with-cleanup-race-protection
  into a shared helper (e.g. `useTauriListener(channel, handler)` in
  `src/hooks/`), and have both `useTauriChat` and `useBrowserEvents`
  use it. This eliminates the divergence and the
  pattern-not-applied-consistently trap.
- Regression validation: a unit test (or integration test with a
  mocked `listen`) that unmounts during the `import()` phase and
  asserts the listener is never registered (or is immediately
  unregistered).
- Validation reports: [V03](../validations/A-SRF-03/V03-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Backend-to-store flow: single transport, typed `chat://event`, untyped `execution://event`, one ingestion point per kind | yes | passed (with finding) | [V01-01](../validations/A-SRF-03/V01-01.md) |
| V02 | Reducer monotonicity: terminal lock, status-rank merge, idempotent inserts, generation counters — and where they are missing | yes | passed (with findings) | [V02-01](../validations/A-SRF-03/V02-01.md) |
| V03 | Reconnect / reload / state recovery: no WS reconnect path, generation-protected loads, `hydrateConversation` merge, manual resume UX | yes | passed (with finding) | [V03-01](../validations/A-SRF-03/V03-01.md) |
| V04 | Streaming / tool / result rendering: token/thinking/tool-batch reducers, pull-based tool detail, message-bubble memoization | yes | passed (with finding) | [V04-01](../validations/A-SRF-03/V04-01.md) |
| V05 | Historical-document drift | conditional | n/a | No prior A-SRF-03 report under `zcode-glm/`; the one historical claim (the `useTauriChat.ts:74-83` pendingCleanup docstring) is verified current and used as the reference for P3-04. |

Executed command (exit 0):

```text
cd echo-agent-cli/web-frontend
npx vitest run --reporter=dot
  Test Files  26 passed (26)
  Tests       101 passed (101)
```

No `cargo` command was required: this is a frontend-only review. The
backend emit contract was read statically (no Rust code changed).

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `useTauriChat.ts:21-25` (ChatPanel) — "Tauri IPC is the only live transport. The WebSocket transport was removed" | current | `grep "useWebSocket" web-frontend/src` returns zero hits; `useChatTransport` returns `useTauriChat()` (`ChatPanel.tsx:23-25`). |
| `useTauriChat.ts:74-83` — pendingCleanup + aborted flag "covers three race windows" | current (the pattern is correct) | Verified by reading the three windows: import-phase abort (`:92`), between-listens abort (`:99-102`), post-listen-pre-push abort (`:152-155`). P3-04 notes that `useBrowserEvents` lacks this hardening. |
| `subagentRunStore.ts:5-12` — "legacy `subagent://trace` / `subagent://event` channels and their stores were deleted in Phase 4c; this is now the single source of truth" | current | `grep "subagentTraceStore\|subagentStore\b" web-frontend/src` returns zero hits; only `useSubagentRunStore` exists. |
| A-SRF-02-P3-02 (hand-built `execution://event` JSON) | current (load-bearing) | Receive-side confirms the asymmetry: `chat://event` is typed via `ChatEvent` union; `execution://event` is `Record<string, unknown>` cast via `as unknown as` (P3-03). |
| A-CHAT-01 one-terminal invariant + F-RCT-03-P2-02 (ReactAgent never emits `Cancelled`) | current (load-bearing) | The frontend's `done` / `final_answer` / `cancelled` ordering analysis depends on this; P3-01 and P3-02 are the latent traps if the invariant ever regresses. |

## Coverage And Uncertainty

Inspected in full: `useTauriChat.ts`, `chatEventHandler.ts`,
`useBrowserEvents.ts`, all eight chat-related stores
(`chatStore`, `conversationStore`, `subagentRunStore`,
`toolExecutionStore`, `taskRuntimeStore`, `authStore`, plus the
`rightWorkspaceStore` / `workspaceStore` heads for the recovery flow),
`App.tsx`, `main.tsx`, `tauri-bridge.ts`, the chat rendering
components (`ChatPanel`, `MessageBubble`, `InlineToolCall`,
`ParallelExecutionBlock`, `SubagentStreamBlock` head,
`ToolExecutionGroup`), the `ChatEvent` type contract, the
`api/client.ts` HTTP fallback, and the relevant
`api/endpoints.ts` sections. All 26 frontend test files executed (101
tests, exit 0).

Not inspected (out of scope or deferred):

- The 19 `panels.rs` Tauri commands and the 20 `research.rs` / 14
  `plugins.rs` commands beyond confirming their frontend endpoints
  exist (`endpoints.ts:1278-1837`). They follow the same
  `isTauri() ? apiInvoke(...) : http(...)` shape; the chat-surface
  audit does not depend on them.
- The notebook / papers / systematic-review frontend panels — they
  are independent of the chat surface and have their own stores
  (which were not inspected). The chat-surface findings do not
  generalize to those panels without further audit.
- The terminal renderer beyond the listener setup
  (`Terminal.tsx:112-125`) — terminal events flow into xterm.js
  directly (no Zustand), so the chat-surface reducer analysis does
  not apply.
- The `MarkdownContent` renderer internals — treated as a black box
  that consumes `message.content`. No chat-event ordering dependency.
- E2E / Playwright tests (if any) — only vitest unit/integration
  tests were executed.

Environmental constraints:

- Read-only static review against `echo-agent-cli` commit `b3b2e81`.
  No code was modified. The vitest run used the existing incremental
  cache; no rebuild was triggered.

Uncertain claims:

- Whether Tauri event delivery is strictly FIFO across multiple
  webviews. The single-window case is FIFO per the documented
  `app.emit` semantics; multi-window (e.g. a detached dev tools
  window) was not exercised. P3-01's reachability hinges on this.
- Whether the `execution://event` live-overwrite (P2-01) is
  observable in practice today. The happy path is fine; the defect
  requires either a cross-thread emit race or a hydrate-then-live
  interleaving. A targeted E2E test would confirm or refute
  reachability; this static review cannot.
- Whether the `cancelled` ChatEvent (P3-02) is genuinely unreachable.
  The A-CHAT-01 handoff says ReactAgent never emits Cancelled, but
  other agent implementations (subagents, future agent kinds) might.
  The store mapping (`chat.rs:1562`) keeps the variant alive, so the
  handler should too.

## Handoff

Conclusions downstream tasks may rely on:

1. **There is exactly one chat transport.** `useTauriChat` is the
   single `chat://event` and `execution://event` receiver; `ChatPanel`
   mounts it exactly once. Downstream tasks auditing a specific
   chat feature can rely on this single ingestion point and do not
   need to look for parallel transports.
2. **`chat://event` is typed end-to-end** (`ChatEvent` enum on both
   sides). `execution://event` is not (A-SRF-02-P3-02 emit-side,
   A-SRF-03-P3-03 receive-side). Any task touching the execution
   channel contract must update both sides and remove the
   `as unknown as` casts.
3. **The subagent reducer is monotone** (terminal lock at
   `subagentRunStore.ts:458-460`); the tool-execution reducer is
   monotone only on the hydrate path. The live-ingest overwrite
   (P2-01) is the single correctness gap in the reducer layer.
4. **Recovery is well-designed** (generation counters on both loads,
   `hydrateConversation` preserves other-conversation tools, persisted
   id round-trip via `restoredMessageId`). The only recovery-related
   trap is the lack of auto-resume (the user must click "Resume" in
   `WelcomeScreen`); this is intentional UX, not a defect.
5. **The chat-event handler depends on backend emit ordering** for
   `final_answer` / `done` / `cancelled`. The backend currently
   guarantees this; the frontend does not defend against reorder
   (P3-01, P3-02). Any task that changes `TauriChatSink::on_event`
   ordering must update the frontend handler in lockstep.

Reports downstream tasks must read:

- This report (A-SRF-03) for the receive-side contract, the reducer
  policy matrix, and the recovery flow.
- `tasks/A-SRF-02.md` for the emit-side contract (especially
  P3-02 untyped execution channel — the cause of A-SRF-03-P3-03).
- `tasks/A-CHAT-01.md` for the one-terminal invariant and the
  ReactAgent-never-emits-`Cancelled` contract that A-SRF-03-P3-02
  depends on.

Conditions that make this report stale:

- Reintroducing a second transport (`useWebSocket` or similar)
  invalidates V01's "single transport" claim.
- Reintroducing the legacy `subagent://trace` / `subagent://event`
  channels and stores invalidates the duplicate-search conclusion.
- Adding zod / valibot validation to `execution://event` receive
  (resolving P3-03) invalidates V01's "untyped" claim.
- Making `useToolExecutionStore.ingest` use `mergeToolExecution`
  (resolving P2-01) invalidates V02's central finding.
- Adding a `final_answer` stash or removing the `done` finalize
  (resolving P3-01) invalidates V02's ordering finding.
- Auto-restoring `activeId` after reload invalidates the V03
  "manual resume" conclusion.

Follow-up task IDs (no fixes implemented in this review):

- A **tool-execution reducer unification** task — resolve
  A-SRF-03-P2-01 by routing live `ingest` through
  `mergeToolExecution`. Pair with A-SRF-02-P2-03 (the backend's
  parallel recorder authority) so both emit and receive sides agree
  on status-rank.
- A **chat-event terminal hardening** task — resolve A-SRF-03-P3-01
  and P3-02 by making `done` / `cancelled` either delegate to the
  existing terminal arms or document them as unreachable.
- A **listener-setup helper extraction** task — resolve
  A-SRF-03-P3-04 by lifting the `useTauriChat.ts:84-167` race-safe
  pattern into a shared `useTauriListener` hook used by both
  `useTauriChat` and `useBrowserEvents`.
- A **runtime validator for `execution://event`** task (paired with
  A-SRF-02-P3-02's `ExecutionEvent` enum) — resolve A-SRF-03-P3-03.
- A **TUI parity** task — the chat surface's reducer policies
  (terminal lock, generation counters, status-rank merge) are
  reusable patterns the TUI should adopt for its own state
  management (cross-reference AGENTS.md multi-mode parity rule).
