# A-SRF-03: GUI chat and frontend state integration

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: echo-agent clean; echo-agent-cli has 79 pre-existing
> modified files under `web-frontend/src/generated/` (ts-rs formatting drift
> only — fields identical to HEAD; md5-verified unchanged by this review's
> vitest runs).

## Question

Does the React chat surface consume backend facts without inventing lifecycle
state or dropping late/duplicate events?

Answer: **no, with two P1 and one P2 reducer/hook defects.** The store is a
faithful single-path projection of `chat://event` + `execution://event` facts
(message-scoped gate, race-safe listeners, monotone subagent-run ingest,
identity-deduped reload hydration), and late events from finished turns are
correctly dropped. But (a) the interrupt prompt path strands the frontend
turn state — after any `interrupt_prompt` (a TaskRuntime run is in progress
when a message is sent) no terminal chat event ever arrives for the ghost
message_key, so all subsequent messages queue forever and the chat input is
dead until reload (P1-01); (b) the error/cancel terminal handling ends with
`runStatus: 'completed'` and replaces the streamed partial answer with the
error text, so failed and user-cancelled turns are reported as "就绪" and the
partial output is lost from both the UI and the persisted conversation
(P1-02 — the frontend half of A-CHAT-01-P1-01 + F-RCT-03-P1-02, now shown to
be caused by the frontend reducer itself, not only the backend TurnStatus);
(c) the live tool-event ingest keys rows by per-producer `detail_ref` id
instead of logical (owner, call_id) identity, so the duplicate subagent tool
projection (A-SRF-02-P2-01) renders duplicated tool cards and inflated counts
(P2-01).

## Scope

Primary source paths inspected (full or behavior-slice reads):

- `web-frontend/src/stores/chatStore.ts` (full, 528), `toolExecutionStore.ts`
  (full, 255), `subagentRunStore.ts` (full, 536), `conversationStore.ts`
  (full, 481), `taskRuntimeStore.ts` (interrupt/resume/refresh slices),
  `queuedChat.ts` (full).
- `web-frontend/src/hooks/useTauriChat.ts` (full, 390),
  `chatEventHandler.ts` (full, 222), `useBrowserEvents.ts` (contract only).
- `web-frontend/src/types/api.ts` (ChatEvent union, ChatRunStatus,
  ToolExecution, ExecutionRound/Step), `src/api/endpoints.ts` (tool detail
  endpoints).
- `web-frontend/src/components/chat/` — `ChatPanel.tsx` (full),
  `MessageBubble.tsx` (full), `ToolExecutionGroup.tsx`, `InlineToolCall.tsx`,
  `SubagentStreamBlock.tsx`, `ExecutionProcessGroup.tsx`,
  `ParallelExecutionBlock.tsx` (contract), `FailureToast.tsx`; plus the test
  files in that directory and `stores/*.test.ts`.
- Backend event producers (contract anchors): `src/tauri/commands/chat.rs`
  (ChatEvent enum, `emit_chat_event`, `send_chat_message`,
  `steer_chat_message`, `cancel_chat`, `TauriChatSink`,
  `agent_event_to_chat_event`, `TauriExecutionProjector`),
  `echo-agent-app-core/src/tool_execution.rs` (ToolExecutionSummary/start),
  `echo-agent-app-core/src/chat_driver.rs` (drive_chat Result contract via
  A-CHAT-01).

## Out Of Scope

- Tauri command lifecycle/setup composition → A-SRF-02 (dependency report;
  cross-verified findings below).
- `drive_chat`/sink semantics and envelope terminal normalization →
  A-CHAT-01, F-RCT-02/03 (dependency facts consumed, not re-filed).
- TaskRuntime executor, claims, recovery → A-TSK-01..06.
- Dynamic GUI smoke (webview console, real event interleaving) →
  Q-GUI-01, Q-E2E-01.
- Full frontend submission gate (prettier/tsc/build) → Q-WEB-01.
- Frontend architecture/perf/a11y → A-FE-03 (depends on this report).

## Inputs

- Root `AGENTS.md` (UTF-8/panic safety, surface parity, layering gate,
  "严禁平行实现同一语义", no invented authority), shared `README.md`,
  `REPORTING.md`, `TASKS.md` (A-SRF-03 card), `zcode-ds/README.md`, report
  templates.
- Dependency task reports read (zcode-ds): `A-SRF-02` (complete),
  `A-CHAT-01` (complete).
- Historical documents treated as hypotheses: `docs/MASTER-PLAN.md`,
  `echo-agent-cli/docs/2026-07-16-agent-lifecycle-audit.md`,
  `echo-agent-cli/docs/2026-07-11-running-input-interrupt-design.md`,
  `echo-agent-cli/docs/gui-status.md`.

## Layering Decision

- Generic mechanism (framework, reused as-is): the `AgentEvent`/`EventEnvelope`
  stream and `AgentHandle` — framework-owned; the frontend consumes only the
  Tauri-projected chat events. No framework movement recommended.
- EKO product policy (application, correct placement): `ChatRunStatus` +
  `runStatus` lifecycle, `ChatMessage`/`ExecutionRound`/`ExecutionStep`
  projections, `toolExecutionStore`/`subagentRunStore`, queue/steer policy,
  `interruptPrompt` dialog — all application-layer projections of backend
  facts. The three findings below are application-layer (frontend reducer/
  hook) defects, not misplacement.
- Adapter boundary: `useTauriChat`/`chatEventHandler` are the thin
  transport→store bridge; the interrupt ghost (P1-01) is a defect at this
  boundary (backend early-return without a lifecycle + frontend with no
  handling for the invoke-response shape).
- Duplicate search (terms + results, V01-01): `TurnStatus`/`turn_status`
  (zero frontend producers/consumers besides the relay),
  `ChatRunStatus` (single definition, api.ts:38), `run_status` (single
  producer chat.rs:1365-1368, single consumer chatEventHandler.ts:159-180),
  `ChatEvent::Done` (single producer chat.rs:1383), `useWebSocket`
  (removed; only PTY WebSocket remains), `tool_batch_start/end` (single
  backend path), `setStreaming` (dead, zero callers), `worker` (none in
  touched files), `chat://event` emitters (two — chat.rs sink + panels.rs
  manual-compress notice; the latter is message-less by design and harmless),
  `execution://event` producers (three; the two subagent-tool overlap is
  A-SRF-02-P2-01, frontend consequence is P2-01 here).

## Current Path

Verified call graph (V02-01, V03-01, V03-02):

1. Send: `ChatPanel` → `useTauriChat.sendMessage` (useTauriChat.ts:251-264;
   queues if a turn is in flight) → `dispatchMessage` (:169-247):
   `addUserMessage` → `startAssistantMessage(message_key)` (:189) →
   ensure conversation → `apiInvoke('send_chat_message')` (:207-223).
2. Backend `send_chat_message` (chat.rs:443-733): interrupt detection
   (:511-535) returns `{kind:'interrupt_prompt'}` BEFORE any turn
   registration; normal path registers turn/cancel-token/HITL (:536-589),
   builds `TauriChatSink`, spawns `drive_chat`; after the driver returns,
   emits `TurnStatus{completed|failed|cancelled}` + `Done` (:690-711).
3. Streaming: `drive_chat` → envelope → sink → `chat://event` payloads
   (message_key injected, chat.rs:137-149); `handle_tool_event`
   (chat.rs:1218-1308) persists tool rows + emits `execution://event`
   summaries; `agent_event_to_chat_event` (chat.rs:1449-1545) maps
   Token/ThinkStart/LlmUsage/ContextCompressed/ToolBatch*/Chart/FinalAnswer/
   Cancelled/Error/Notices.
4. Frontend listener (useTauriChat.ts:84-167): `chat://event` gated by
   `isCurrentRunEvent` (message_key equality, :50-58) → `handleChatEvent`
   → chatStore; `execution://event` (:109-150) → toolExecutionStore
   (kind=tool) / subagentRunStore (kind=subagent) / taskRuntimeStore
   (kind=run).
5. Terminals: `final_answer` → `finalizeAssistantMessage` (runStatus
   'completed'); `error` → `setRunStatus('failed')` then
   `finalizeAssistantMessage(id, '[Error] …')` (chatEventHandler.ts:140-150)
   — the latter overwrites both the status and the content (P1-02); `done`
   → queue drain (:69-71); `cancelled` → runStatus 'cancelled'.
6. Reload: `loadConversation` (conversationStore.ts:296-388) rebuilds
   messages (deterministic ids), hydrates tool rows identity-deduped
   (`mergeHydratedToolExecutions`), and TaskRuntime refresh re-projects
   subagent cards from durable events (subagentRunStore.ts:330-405,
   taskRuntimeStore.ts:196-197/:246).

## Findings

### A-SRF-03-P1-01: The interrupt prompt strands the frontend turn state — after any `interrupt_prompt`, all subsequent messages queue forever and the chat input is dead until reload

- Priority: P1
- Confidence: high (fully static-verified chain; deterministic given the
  backend early-return)
- Layer: adapter (frontend hook × backend command contract)
- Evidence:
  - Backend early-return: `echo-agent-cli/src/tauri/commands/chat.rs:511-535`
    — `send_chat_message` emits `InterruptPrompt` and returns
    `{kind:"interrupt_prompt", run_id}` before registering
    `active_chat_turns`/`cancel_token`/sink (:536+) — no chat event of any
    kind will ever be emitted for this message_key.
  - Frontend turn creation: `web-frontend/src/hooks/useTauriChat.ts:188-190`
    sets `currentMessageKeyRef` and creates the assistant message BEFORE the
    invoke; the invoke response is handled at :227-233 by reading only
    `run_id` — the `{kind:'interrupt_prompt'}` shape is never acted on.
  - Queue gate: `useTauriChat.ts:253-259` — `sendMessage` queues whenever
    `currentMessageKeyRef.current` is set; the queue drains only on `done`
    (:40-48, :69-71) which never fires; the `interrupt_prompt` chat event
    handler (chatEventHandler.ts:196-204) only opens the TaskRuntime dialog.
  - Dialog choices: `web-frontend/src/components/task/TaskRuntimePanel.tsx`
    (InterruptPromptDialog) — resume/dismiss/abandon all touch only
    taskRuntimeStore (resumeTaskRun, dismissInterruptPrompt,
    cancel_task_run); none clear the chat turn refs; `steer_chat_message`
    returns "no active chat turn" (chat.rs:750) because no turn was
    registered, so the visible queue cannot be steered either.
  - Ref-clear inventory (grep): `currentMessageKeyRef` is cleared only at
    useTauriChat.ts:243 (catch) and chatEventHandler.ts:106/147/154/211
    (final_answer/error/cancelled/done) — none fire for the ghost key;
    `cancel()` (:314-327) also does not clear it.
- Reachability: user sends a message while a TaskRuntime run is
  Running/Paused in the same conversation (interaction mode). Deterministic;
  every occurrence strands the input.
- Expected invariant: every turn the frontend starts is terminated by a
  backend terminal event; after a rejected/interrupt turn the chat input
  remains usable and the visible FIFO keeps the "当前 turn 终态后逐条启动"
  contract (MASTER-PLAN:124/:342 "一条权威生命周期"; interrupt-design
  doc:16-20).
- Observed behavior: an empty assistant message with a blinking cursor
  remains in the list, `isStreaming`/`runStatus 'running'` stay set (stop
  button stuck), and every subsequent `sendMessage` lands in the queue that
  never dispatches. Only a webview reload (which resets the refs) recovers.
- Impact: the flagship chat surface is unusable after a documented,
  user-reachable flow — queued messages silently never send; the user sees no
  error explaining why (P1: core path unusable).
- Root cause: the backend interrupt branch returns without a turn lifecycle
  (no terminal event will ever come), while the frontend keyed its whole
  in-flight/turn state and its queue-drain trigger on a terminal chat event
  — with no handling for the invoke-response `kind:'interrupt_prompt'` and no
  rollback of the optimistic assistant message.
- Direction: on `kind === 'interrupt_prompt'` in `dispatchMessage`, roll back
  the optimistic turn (remove the ghost message, clear refs, restore
  `runStatus`/`isStreaming`, drain the queue) or have the backend emit a
  terminal `TurnStatus`/`Done` for the interrupted message_key; alternatively
  move the interrupt detection before the frontend creates the turn (e.g.
  pre-flight invoke) — pick one authority.
- Regression validation: hook-level fixture — `dispatchMessage` resolving
  with `{kind:'interrupt_prompt'}` leaves no ghost message, clears the refs,
  and the immediately following `sendMessage` dispatches (not queues); a
  queue fixture asserting the FIFO drains after interrupt.
- Validation reports: [V02-01](../validations/A-SRF-03/V02-01.md), [V03-02](../validations/A-SRF-03/V03-02.md), [V05-01](../validations/A-SRF-03/V05-01.md)

### A-SRF-03-P1-02: Error/cancel terminal handling ends with `runStatus: 'completed'` and wipes the streamed partial answer — `finalizeAssistantMessage` unconditionally sets 'completed' and replaces content

- Priority: P1
- Confidence: high (reducer-level deterministic; ordering of backend error
  vs TurnStatus verified in A-CHAT-01/F-RCT-03)
- Layer: application (frontend reducer)
- Evidence:
  - `web-frontend/src/hooks/chatEventHandler.ts:140-150` — `error` case:
    `store.setRunStatus('failed')` then, if an assistant message exists,
    `store.finalizeAssistantMessage(id, '[Error] ' + event.message)`.
  - `web-frontend/src/stores/chatStore.ts:354-362` — `finalizeAssistantMessage`
    unconditionally sets `runStatus: 'completed'` and REPLACES
    `messages[..].content` with the passed string, then `scheduleAutoSave`
    persists the wiped content.
  - Cancel path: `useTauriChat.ts:314-327` (`markCancelled` → 'cancelled'),
    then the backend's envelope-fabricated Error event (event_envelope.rs
    normalizes every terminal-less/err end; F-RCT-03-P1-02: cancel never
    emits `Cancelled`) arrives → error handler → 'failed' then 'completed'.
  - Rendering: `ChatPanel.tsx:432-451` maps 'completed' to the default
    "就绪" label; `isCancelled` stays true on cancel so the "已停止响应"
    divider and the `[Error]` body render simultaneously.
  - The backend's own `TurnStatus('completed')` (chat.rs:709-711, from
    `outcome.is_ok()` — always Ok after envelope normalization,
    A-CHAT-01-P1-01) is suppressed on the frontend by `isCancelledRef`
    (chatEventHandler.ts:161), so the reducer's own 'completed' is the final
    state — the frontend does not merely relay the backend lie, it
    re-produces it.
- Reachability: every chat turn whose envelope yields an Error payload
  (provider error, no-response, max-iterations, tool-batch timeout) and every
  user-cancelled turn.
- Expected invariant: terminal states are truthful and monotone — an Error
  terminal ends 'failed', a user cancel ends 'cancelled', and the streamed
  partial answer remains visible and persisted (Claude Code/Codex keep
  partial output; MASTER-PLAN:124 one truthful lifecycle).
- Observed behavior: failed turns end with `runStatus 'completed'` (header
  "就绪"), and the message body is replaced by `[Error] …`, losing all
  streamed tokens; on cancel the same wipe occurs with a fabricated error;
  the auto-saved conversation permanently stores the wiped content.
- Impact: misleading success on failure + user-visible loss of partial
  output on the flagship surface (P1); the persisted conversation is
  corrupted relative to what the user actually saw stream.
- Root cause: `finalizeAssistantMessage` conflates three concerns — mark
  message non-streaming, set the terminal status, replace the content — and
  the error handler calls it after setting 'failed', so the last write wins
  with 'completed' + error text.
- Direction: split content finalization from status: on Error, keep the
  partial content and append the error as a notice/annotation (or separate
  field), set `runStatus` per the last agent event (FinalAnswer→completed,
  Error→failed, cancel-token-fired→cancelled); make the terminal status
  transition monotone (no terminal → different terminal); persist partial
  content before terminalization.
- Regression validation: store-level fixtures — (a) streamed tokens then an
  Error event → runStatus 'failed', content retains the partial text;
  (b) cancel flow → 'cancelled' with no '[Error]' body; (c) reload of such a
  conversation restores the partial content; (d) `finalizeAssistantMessage`
  no longer changes a status it was not asked to set.
- Validation reports: [V03-01](../validations/A-SRF-03/V03-01.md), [V05-01](../validations/A-SRF-03/V05-01.md)

### A-SRF-03-P2-01: Live tool-event ingest is keyed by per-producer `detail_ref` id, not logical (owner, call_id) identity — the duplicate subagent tool projection renders duplicated cards and inflated counts

- Priority: P2
- Confidence: high
- Layer: application (frontend store) — consequence of the A-SRF-02-P2-01
  double producer
- Evidence:
  - Live path: `web-frontend/src/stores/toolExecutionStore.ts:206-217` —
    `ingest` stores by `tool.id` and appends that id to `idsByOwner`.
  - Producer ids: `echo-agent-cli/echo-agent-app-core/src/tool_execution.rs:191-200`
    — `start()` allocates a fresh `detail_ref` UUID per call, and
    `ToolExecutionSummary.id = detail_ref`; both live producers
    (subagent-event bus bridge `src/tauri/mod.rs:347-768` and
    `TauriExecutionProjector` `src/tauri/commands/chat.rs:957-1114`) call
    `start()` for the same subagent tool (A-SRF-02-P2-01) → two summaries,
    two different ids, same (owner, call_id).
  - Rendering: `web-frontend/src/components/chat/SubagentStreamBlock.tsx:46-49`
    renders every id in `idsByOwner[ownerKey]` → "N 工具" inflated and
    duplicated `InlineToolCall` rows.
  - Dedupe exists but is hydration-only: `executionIdentity`
    (toolExecutionStore.ts:46-48) and `mergeToolExecution` (:58-86) are used
    by `mergeHydratedToolExecutions` (:106-117, reload path) and the
    TaskRuntime boundary merge — not by `ingest`.
  - Test gap: `stores/chatStore.toolExecution.test.ts` "ignores duplicate
    start events for one execution ID" covers only same-id duplicates —
    certifies a weaker invariant than the two-producer scenario.
- Reachability: any GUI chat turn in which a TaskRuntime/foreground run's
  subagent calls a tool (both producers live and verified in A-SRF-02-P2-01);
  chat-owner tools have a single producer and are unaffected.
- Expected invariant: one projection row per logical tool execution
  (owner+call_id); duplicate summaries are idempotent (AGENTS.md "严禁平行
  实现同一语义"; A-SRF-02-P2-01's direction assumes the frontend tolerates
  duplicates until the backend is fixed).
- Observed behavior: two rows per logical call; the subagent card shows a
  doubled tool count and duplicated tool rows with separate detail files.
- Impact: misleading tool accounting on the flagship surface; the duplicate
  backend persist (A-SRF-02-P2-01) becomes user-visible instead of being
  absorbed.
- Root cause: the store keys by producer-assigned id while the natural key —
  (owner, call_id) — is already available and already used by the hydration
  path; the two producers were wired without idempotency.
- Direction: make `ingest` upsert by `executionIdentity` (merge incoming with
  `mergeToolExecution`, preferring the row that has a `detail_ref` and the
  newer activity timestamp) while keeping `id` for detail loading; the
  backend-side fix (single producer, per A-SRF-02-P2-01) remains the
  canonical delete target.
- Regression validation: fixture ingesting two started/finished summary pairs
  with identical (owner, call_id) but different ids → exactly one row in
  `tools` and one id in `idsByOwner`; SubagentStreamBlock shows count 1.
- Validation reports: [V01-01](../validations/A-SRF-03/V01-01.md), [V03-01](../validations/A-SRF-03/V03-01.md)

### A-SRF-03-P3-01: `setStreaming` is dead code and the `ChatRunStatus` doc header mentions a nonexistent 'connecting' state

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `web-frontend/src/stores/chatStore.ts:390` — `setStreaming`
  defined, zero callers (grep across `src/`, V01-01);
  `web-frontend/src/types/api.ts:27-31` — comment "ChatRunStatus (enum with
  frontend-only states like 'connecting')"; the actual union (:38-47) has no
  `connecting` member.
- Reachability: none (dead code) / n/a (comment).
- Expected invariant: no dead store API; comments describe the real type.
- Observed behavior: dead method retained; stale comment.
- Impact: minor — misleading readers of the reducer contract.
- Root cause: `isStreaming` is derived inside `setRunStatus`, leaving the
  standalone setter unused; comment not updated when 'connecting' was
  dropped.
- Direction: delete `setStreaming` (and its interface line chatStore.ts:79)
  or wire it to the derived logic; fix the api.ts comment.
- Regression validation: grep `setStreaming` returns nothing; `npx tsc -b`
  passes.
- Validation reports: [V01-01](../validations/A-SRF-03/V01-01.md)

## Cross-Verified Dependency Findings (canonical IDs elsewhere; independently confirmed here)

| Canonical ID | Claim | Independent confirmation |
|---|---|---|
| A-CHAT-01-P1-01 | GUI error turns labeled "completed"; cancel renders fabricated error | Confirmed and strengthened: the frontend reducer ITSELF ends error/cancel turns at 'completed' via `finalizeAssistantMessage` (chatStore.ts:358), independently of the backend TurnStatus (A-SRF-03-P1-02); the message body shows `[Error] …` while the header shows "就绪" (ChatPanel.tsx:432-451). |
| A-SRF-02-P2-01 | Two live producers persist the same subagent tool events | Confirmed at the frontend: two summaries with distinct ids per logical call land in `toolExecutionStore` (P2-01) — duplicated cards, inflated counts; the hydration path's identity dedupe does not cover the live path. |
| A-SRF-02-P1-01 | `browser://event` bridge dead (double `.setup()` overwrite) | Confirmed frontend side is healthy: `useBrowserEvents.ts` listener + `browserStore.ingest` exist and are correctly registered; the deadness is purely backend (overwritten setup closure) — no frontend defect filed here. |
| A-CHAT-01-P2-01 | `ChatDriverEvent::Interrupt` dead; GUI emits InterruptPrompt directly | Confirmed: frontend `interrupt_prompt` handling works (dialog opens) but leaves the ghost turn state (P1-01) — the surface parity gap now has a concrete GUI defect attached. |
| A-CHAT-01-P2-02 | TauriChatSink owns durable tool projection; cancel-on-any-terminal | Confirmed the frontend consequence: tool cards can show 'cancelled' for completed tools (sink-driven summary status), which the frontend renders as fact — no separate frontend finding; fix belongs to the sink (A-CHAT-01). |

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search (frontend lifecycle concepts vs backend facts, both repos) | yes | passed | [V01-01](../validations/A-SRF-03/V01-01.md) |
| V02 | Registration and runtime reachability (backend chat://event + execution://event → listeners → stores; interrupt early-return trace) | yes | passed | [V02-01](../validations/A-SRF-03/V02-01.md) |
| V03 | Invariant/edge cases — reducer monotonicity and terminal semantics; reconnect/reload; late/duplicate events; streaming/tool/result fixtures | yes | passed (2 reports) | [V03-01](../validations/A-SRF-03/V03-01.md), [V03-02](../validations/A-SRF-03/V03-02.md) |
| V04 | Targeted executable check — full `vitest run`; targeted chat/execution suite | yes | passed (exit 0 / exit 0; 101 + 40 tests) | [V04-01](../validations/A-SRF-03/V04-01.md), [V04-02](../validations/A-SRF-03/V04-02.md) |
| V05 | Historical-document drift (MASTER-PLAN lifecycle/interrupt claims; lifecycle audit; interrupt design; gui-status; api.ts header) | yes | passed | [V05-01](../validations/A-SRF-03/V05-01.md) |

All required validations executed; every reported command has a known exit
code; no validation is pending.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN:124/:342 "一条权威生命周期…前端只改变策略或渲染,不改变状态语义";"GUI 也不得维护只在前端存在的权威运行状态" | regressed | A-SRF-03-P1-01 (ghost turn with no backend lifecycle) + P1-02 (reducer writes 'completed' on error) |
| MASTER-PLAN:411 "投影:Tauri events…前端 Zustand store" | current | V02-01/V03-02 |
| MASTER-PLAN:453 "前端 chat status 不再覆写 TaskRuntime" | current (fixed vs audit) | chatEventHandler.ts:159-180 touches only chatStore; regression test exists (V01-01) |
| MASTER-PLAN:770 "…turn status…interrupt 不再通过默认 no-op 静默丢失" | regressed | interrupt path strands frontend state (P1-01); `ChatDriverEvent::Interrupt` dead (A-CHAT-01-P2-01) |
| lifecycle-audit:19/:142/:187 "chat run_status 直接覆写 taskRuntimeStore.activeRun.status" | fixed | handler writes only chatStore; test "does not project a chat terminal status onto the active TaskRun" (V01-01) |
| running-input-interrupt-design:16-20 queued inputs dispatch after turn terminal | regressed (interrupt case) | no terminal ever arrives for the ghost key → FIFO never drains (P1-01) |
| running-input-interrupt-design:36-40 `turn/start`/`turn/steer`/`turn/interrupt` endpoints | stale | implemented as Tauri commands; steer carries expected-turn-id precondition (chat.rs:745-751) |
| gui-status.md "Chat and streaming: Connected" | current | surface streams end-to-end (V02-01), with P1-01/P1-02 caveats |
| api.ts:27-31 "ChatRunStatus (…like 'connecting')" | stale | no `connecting` member (P3-01) |

## Coverage And Uncertainty

- All conclusions are static except two vitest runs (V04); no GUI process was
  launched — the interrupt ghost, error/cancel mislabeling, and duplicate
  cards are proven by code traces with deterministic event sequences, not
  observed at runtime (Q-GUI-01/Q-E2E-01 own dynamic confirmation; the
  ordering of backend Error → TurnStatus → Done is inherited from the
  verified A-CHAT-01/F-RCT-03 envelope contract).
- The interrupt finding assumes the backend returns `{kind:'interrupt_prompt'}`
  without any later chat event for that message_key — verified statically
  (chat.rs:511-535); a future backend change that emits a terminal for the
  interrupted key would fix the symptom, and the finding's direction covers
  both sides.
- `ParallelExecutionBlock`/`contextUsage`/`CompressPanel` were read at
  contract level only; their internal behavior is A-FE-03/context-ui scope.
- The panels.rs manual-compress `chat://event` emitter (no message_key) is
  benign today because `context_compressed` needs no message binding; if more
  event kinds ever use that channel, the missing message_key would widen the
  gate (isCurrentRunEvent falls through to `return true`).
- The browser panel path (A-SRF-02-P1-01) was not re-verified beyond the
  frontend listener existence; dynamic GUI behavior is out of scope here.

## Handoff

- Downstream tasks may rely on: single-path backend-to-store flow with
  message-key gating (V02-01); late/duplicate chat events dropped at the
  boundary and subagent-run terminal monotonicity (V03-01/V03-02);
  identity-deduped reload hydration (V03-02); green fixtures at the reviewed
  commits (V04); the ghost-turn defect (P1-01), error/cancel mislabeling +
  partial-content wipe (P1-02), and non-idempotent live tool ingest (P2-01).
- Reports to read: this report + V01-01..V05-01; A-SRF-02 (P1-01, P2-01),
  A-CHAT-01 (P1-01, P2-01, P2-02), F-RCT-03 (P1-01, P1-02), F-RCT-02 (P1-01).
- Stale triggers: any change to `useTauriChat.ts` (dispatch/queue/refs/
  listeners), `chatEventHandler.ts`, `chatStore.ts` (finalize/setRunStatus/
  markCancelled), `toolExecutionStore.ts` (ingest/merge), the interrupt
  branch of `send_chat_message` (chat.rs:511-535), the Tauri sink
  TurnStatus/Done emission (chat.rs:1365-1399), or the two subagent-tool
  producers (mod.rs bridge, chat.rs projector) invalidates the corresponding
  claims.
- Follow-up task IDs (fixes are not implemented in this review): A-FE-01/02
  (tool-card identity contract), A-FE-03 (this report is a dependency),
  X-EVT-01 (frontend terminal conformance: error→completed, ghost turn,
  duplicate summaries), X-SRF-01 (interrupt parity row), Q-GUI-01, Q-E2E-01
  (scenarios: interrupt prompt then send; cancel a GUI turn; provider-error
  turn; plan-task subagent with one tool call → assert one card), Q-WEB-01.
