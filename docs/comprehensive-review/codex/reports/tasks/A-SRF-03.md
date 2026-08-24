# A-SRF-03: GUI chat and frontend state integration

> Status: complete
> Reviewer: Codex review subagent
> Executor: Codex review subagent
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: CLI clean; framework concurrent dirty paths excluded; only Codex A-SRF-03 reports written
> Accepted by: Codex primary reviewer after independent source-anchor,
> reachability, finding-count, link, executor, commit, and isolation sampling.

## Question

Does the React GUI consume canonical backend chat, Tool, TaskRuntime, Subagent,
attachment, artifact, and terminal facts without inventing lifecycle state,
dropping late/duplicate events, or switching the user's active conversation?

## Scope

- `chat://event` definition/emission/listening and the shared chat reducer.
- `execution://event` Tool/Subagent/run dispatch into frontend stores.
- Chat message, Tool, TaskRuntime and Subagent identity/terminal projections.
- Conversation navigation, history reload, active TaskRun focus, polling and
  remount/reconnect behavior.
- Attachment, Tool detail, Subagent result/artifact and execution-card rendering
  sufficient to verify fact preservation.
- Existing static tests and scoped repository-document drift.

## Out Of Scope

- Shared `drive_chat` outcome and GUI sink producer defects already owned by
  `A-CHAT-01`, including Agent Error being followed by completed and persistence
  failures suppressing Tool events.
- Tauri command registration/setup/terminal/workflow defects owned by
  `A-SRF-02`.
- Conversation backend persistence authority owned by `A-STATE-01`, prepared
  attachment lifecycle owned by `A-INP-01`, and TaskRuntime execution/claim
  semantics owned by application Task tasks.
- Rust/TypeScript DTO generation and field-level contract drift, owned by
  `A-FE-01`.
- CLI/TUI/channel surface parity (`A-SRF-01`/`A-SRF-04`), frontend visual design,
  fixes, source/shared-index mutation, Cargo, rustc, tests, builds, dynamic
  fixtures, WebView launch, and network.

## Inputs

- Root `AGENTS.md`; shared `README.md`, `REPORTING.md`, exact `TASKS.md` card;
  Codex reviewer protocol and templates.
- Authorized complete Codex dependencies `A-SRF-02` and `A-CHAT-01` only.
- Current clean CLI source at the revision above. Concurrent framework dirty
  contents/diffs and all other reviewer directories were not read.
- V09-01 preserves one incorrect historical-search path and uses none of its
  partial output; corrected V09-02 uses explicit existing repository documents.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Canonical event identities, sequence/terminal semantics, persistent replay cursors, and typed Tool/Subagent/Task artifacts must originate in the reusable runtime/framework or application core producer contract. |
| EKO product policy | Which conversation is focused, queue behavior, GUI store lifetimes, right-rail selection, message association, and render composition belong to `echo-agent-cli`. |
| Adapter boundary | The React/Tauri boundary should translate one typed event/snapshot and reduce it by canonical identity; it must not synthesize a second terminal owner or globally replace focused state from background events. |
| Duplicate search | Searched ChatEvent/ExecEvent definitions, all emit/listen sites, conversation/message/run/task/execution/tool/artifact fields, Zustand actions, load/hydrate/poll paths, component imports/callers and tests across both repositories. |
| Migration deletion | Once one conversation-keyed snapshot/reducer exists, delete hook-local lifecycle refs, direct `run_started -> global activeRun` replacement, and live Tool overwrite logic. Do not add another event channel or parallel Task/Subagent store. |

## Current Path

```text
send_chat_message(conversation_id, message_key)
  -> detached drive_chat + TauriChatSink
  -> chat://event {type, message_key, conversation_id}
       -> one ChatPanel useTauriChat listener
       -> hook-local currentMessageKey/currentConversation refs
       -> handleChatEvent -> global chatStore/messages/HITL/runStatus
       -> done callback -> local queued input dispatch

ChatDriver execution facts
  -> execution://event {kind=run|task|subagent|tool, identities...}
       -> kind=subagent -> subagentRunStore[(run_id, execution_id)]
       -> kind=tool -> toolExecutionStore[id] + chat message Tool step
       -> kind=run/run_started -> loadByConversation(event conversation_id)
                                    -> one global TaskRuntime activeRun

conversation navigation / app restoration
  -> conversationStore generation -> messages + attachments + Tool summaries
  -> App activeId effect -> latest TaskRuntime run snapshot
       -> plan/todos/events/artifacts/blockers + polling
       -> durable events rebuild Subagent attempts and Tool boundaries
```

Positive conclusions:

- Chat events carry both message and conversation identity; the backend admits
  only one foreground turn per conversation and emits `Done` only after releasing
  its active-turn ownership.
- Conversation and TaskRuntime loads use generations to prevent stale async
  responses from overwriting newer navigation. Stable persisted message IDs
  reconnect execution steps, Tools and Task root messages; attachments restore.
- Subagent records use `(run_id, subagent_run_id)`, preserve revision/attempt
  identity, reject terminal reopen, and rebuild result/artifact/verification
  facts from durable TaskRuntime events.
- Tool history hydration merges by `(owner, run_id, call_id)` with terminal rank
  and timestamp; large Tool output stays behind an opaque detail reference and
  paginated loader.
- The defined `SubagentPanel`/detail selection writer has no production
  import/render caller and was not falsely treated as reachable.

## Findings

### A-SRF-03-P1-01: Agent terminal payloads release frontend turn identity before backend completion

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/src/tauri/commands/chat.rs:681`, `:700`, `:707`,
  `:1341`, `:1365`; `echo-agent-cli/web-frontend/src/hooks/useTauriChat.ts:40`,
  `:50`, `:60`, `:69`; `echo-agent-cli/web-frontend/src/hooks/chatEventHandler.ts:97`,
  `:140`, `:151`, `:206`.
- Reachability: every GUI chat turn receives Agent FinalAnswer/Error/Cancelled
  through the shared handler, followed later by the Tauri caller's postlude
  `RunStatus` and `Done` for the same message key.
- Expected invariant: Agent payload rendering does not release the turn. One
  backend-complete `Done` commits terminal status, clears identity, and advances
  exactly one queued input after active-turn/HITL cleanup.
- Observed behavior: each Agent terminal branch clears `currentMessageKeyRef`.
  The next same-key `run_status` and `done` are rejected by `isCurrentRunEvent`,
  so the queued dispatcher is never called and backend terminal authority never
  reaches the reducer. Locally, a new input is no longer queued even though the
  backend may still own the conversation.
- Impact: queued prompts remain stuck after every ordinary terminal. A user send
  during the postlude can create transient messages then fail with
  `chat_turn_busy`; status/queue behavior depends on the earlier Agent event
  rather than the application's completed lifecycle.
- Root cause: hook-local message identity doubles as both event correlation and
  admission/queue ownership, and three reducer branches release it before the
  canonical completion boundary.
- Direction: introduce one conversation-scoped frontend turn state keyed by
  message key; render Agent terminal payloads without releasing it, then consume
  a typed application outcome/Done once. Delete terminal-specific ref clearing
  and advance the queue only from that reducer transition. Coordinate with
  `A-CHAT-01-P1-01` rather than compensating for its wrong producer status.
- Regression validation: FinalAnswer, Error, Cancelled, setup failure and raw
  EOF with queued input/HITL/Tool cleanup; assert one terminal, one queue advance,
  no busy retry and no post-terminal state mutation.
- Validation reports: [V02](../validations/A-SRF-03/V02-01.md).

### A-SRF-03-P1-02: WebView reload or ChatPanel remount permanently detaches an active ordinary chat

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/web-frontend/src/hooks/useTauriChat.ts:23`, `:50`,
  `:74`, `:169`; `echo-agent-cli/src/tauri/commands/chat.rs:536`, `:556`, `:681`;
  `echo-agent-cli/web-frontend/src/stores/taskRuntimeStore.ts:219`;
  `echo-agent-cli/web-frontend/src/App.tsx:51`.
- Reachability: the Rust turn runs in a detached Tokio task independently of the
  WebView listener. Reload, ErrorBoundary recovery, or a real component remount
  recreates hook refs while the backend process/turn can continue.
- Expected invariant: listener registration obtains an active-turn snapshot and
  cursor, replays missed facts, then subscribes; otherwise it cancels the orphan
  and publishes an explicit terminal state.
- Observed behavior: every mount initializes message/conversation refs to null.
  All continued events carry a message key and are therefore rejected. No IPC
  exposes the backend `active_chat_turns`, event cursor, partial snapshot, or
  rebind. Persisted TaskRuntime recovery cannot restore ordinary chat token,
  HITL, final-answer, queue, or terminal state.
- Impact: after a frontend reload/remount the visible response can remain
  permanently partial/running while the backend continues Tools or waits for an
  invisible HITL request. Cancel without the lost key falls back to cancelling
  all active turns, affecting unrelated conversations.
- Root cause: listener lifetime is treated as turn lifetime; correlation state
  exists only in React refs while execution ownership exists only in Rust maps.
- Direction: add one application-owned conversation turn snapshot/replay API
  with `{conversation_id, message_key, phase, terminal, cursor, pending_hitl}`;
  bind before live subscription and use scoped cancel. Delete null-ref filtering
  as the sole recovery/admission contract.
- Regression validation: WebView reload/remount during token, Tool, approval,
  final payload and postlude for two concurrent conversations; assert scoped
  rebind/cancel, replay ordering and exactly one terminal.
- Validation reports: [V03](../validations/A-SRF-03/V03-01.md).

### A-SRF-03-P1-03: A background conversation's RunStarted event replaces the focused TaskRuntime projection

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:350`;
  `echo-agent-cli/src/tauri/commands/chat.rs:1419`;
  `echo-agent-cli/web-frontend/src/hooks/useTauriChat.ts:138`;
  `echo-agent-cli/web-frontend/src/stores/taskRuntimeStore.ts:219`;
  `echo-agent-cli/web-frontend/src/App.tsx:51`;
  `echo-agent-cli/web-frontend/src/components/layout/RightRail.tsx:5`;
  `echo-agent-cli/web-frontend/src/components/chat/MessageBubble.tsx:185`.
- Reachability: pooled GUI conversations can execute concurrently and every
  live TaskRuntime sink emits a global RunStarted. The one ChatPanel listener is
  not scoped to the currently displayed conversation.
- Expected invariant: background events update conversation-keyed snapshots;
  only `conversationStore.activeId` chooses the right-rail/message projection.
- Observed behavior: any RunStarted invokes `loadByConversation` with the event's
  conversation ID, without comparing activeId. That replaces the single global
  activeRun/plan/todos/artifacts and starts polling it. App corrects focus only
  when activeId itself changes.
- Impact: while viewing conversation A, a later event from B can show B's task
  goal/progress/recovery controls beside A's messages. Pause, cancel, retry or
  plan edits can then target B from the wrong conversation context.
- Root cause: the global event consumer owns focus selection in addition to
  ingestion; TaskRuntime state is modeled as one active record rather than
  conversation-keyed snapshots plus a selector.
- Direction: cache TaskRuntime snapshots by conversation/run and update them by
  event identity; derive the focused view only from activeId. Delete direct
  RunStarted-to-global-load replacement after keyed ingestion exists.
- Regression validation: active A plus background B with controlled RunStarted,
  terminal and navigation order; assert panel/messages/actions remain A until
  explicit navigation and B is current immediately after selection.
- Validation reports: [V04](../validations/A-SRF-03/V04-01.md).

### A-SRF-03-P2-04: Live Tool ingestion can reopen a terminal execution

- Priority: P2
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent-cli/echo-agent-app-core/src/tool_execution.rs:191`;
  `echo-agent-cli/src/tauri/commands/chat.rs:185`;
  `echo-agent-cli/web-frontend/src/hooks/useTauriChat.ts:132`;
  `echo-agent-cli/web-frontend/src/stores/toolExecutionStore.ts:46`, `:106`,
  `:202`, `:223`, `:240`.
- Reachability: main and Subagent Tool summaries arrive live through the global
  execution listener while conversation/TaskRuntime hydration can concurrently
  merge persisted facts for the same repository detail ID.
- Expected invariant: one Tool execution has a monotonic lifecycle; terminal
  beats running and, between terminal facts, the newest authoritative timestamp
  wins regardless of live, persisted or TaskRuntime source.
- Observed behavior: history hydration and TaskRuntime reconstruction use
  identity-aware terminal merges, but live `ingest` directly overwrites
  `tools[tool.id]`. A queued/duplicate started summary after a terminal summary
  changes the card back to running and discards terminal fields.
- Impact: completed/failed/cancelled Tool cards can spin indefinitely or lose
  their final duration/status after reload/event interleaving; UI history and
  live behavior disagree for the same facts.
- Root cause: three ingestion paths implement two lifecycle reducers; only the
  snapshot paths use the canonical merge.
- Direction: route every Tool fact through one identity-aware monotonic merge
  and retain a source/sequence or authoritative timestamp. Delete direct live
  map overwrite after migration.
- Regression validation: every permutation/duplicate of start, success,
  failure, cancel, persisted snapshot and TaskRuntime boundary across two runs;
  assert identity isolation and terminal monotonicity.
- Validation reports: [V05](../validations/A-SRF-03/V05-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Commit and concurrent-dirty isolation | yes | passed | [report](../validations/A-SRF-03/V00-01.md) |
| V01 | Definition, duplicate and production reachability map | yes | passed | [report](../validations/A-SRF-03/V01-01.md) |
| V02 | Turn terminal identity and queued dispatch | yes | failed -> finding | [report](../validations/A-SRF-03/V02-01.md) |
| V03 | Remount/reload active-chat recovery | yes | failed -> finding | [report](../validations/A-SRF-03/V03-01.md) |
| V04 | Multi-conversation RunStarted focus | yes | failed -> finding | [report](../validations/A-SRF-03/V04-01.md) |
| V05 | Live Tool duplicate/late monotonicity | yes | failed -> finding | [report](../validations/A-SRF-03/V05-01.md) |
| V06 | Subagent attempt/result/artifact identity | yes | passed | [report](../validations/A-SRF-03/V06-01.md) |
| V07 | Conversation/TaskRuntime reload matrix | yes | passed with V03/V05 limitations | [report](../validations/A-SRF-03/V07-01.md) |
| V08 | Existing static test inventory | yes | passed with gaps | [report](../validations/A-SRF-03/V08-01.md) |
| V09-01 | Historical search with invalid assumed path | retained failure | inconclusive | [report](../validations/A-SRF-03/V09-01.md) |
| V09-02 | Corrected scoped historical drift | yes | passed | [report](../validations/A-SRF-03/V09-02.md) |
| V10 | Executable replay/remount/concurrency fixtures | future | not_run by direction | [report](../validations/A-SRF-03/V10-01.md) |
| V99-01 | Static integrity preflight | retained failure | failed: self-report absent | [report](../validations/A-SRF-03/V99-01.md) |
| V99-02 | Static integrity final gate | yes | passed | [report](../validations/A-SRF-03/V99-02.md) |
| V30 | Primary acceptance sampling | yes | passed | [report](../validations/A-SRF-03/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `A-CHAT-01-P1-01`: producer collapses Agent Error/cancel into transport success/GUI completed | current and canonical; not duplicated | V02 isolates frontend early release/queue behavior |
| `A-CHAT-01-P1-05`: GUI persistence failure suppresses Tool events | current and canonical; not duplicated | V05 begins only after a Tool summary is delivered |
| `A-SRF-02`: Tauri command/bridge remains the GUI adapter boundary | current dependency conclusion | V01, V03 |
| Subagent unification plan: one execution channel and stable execution identity are complete | structurally current | V01, V06 |
| MASTER-PLAN: GUI Tool history/reload and typed TaskRuntime projection are complete | current with live-ingest/remount limitations | V05, V06, V07 |
| README: React GUI uses WebSocket real-time chat transport | stale | production ChatPanel explicitly selects Tauri and only listens to Tauri events; V01/V09-02 |

## Coverage And Uncertainty

- No Cargo, rustc, frontend test/build, dynamic fixture, WebView/application
  launch, or network command was run. V10 records future executable evidence.
- Static evidence is conclusive for ref clearing/filtering order, absent recovery
  API, global RunStarted focus mutation, and direct Tool overwrite. Exact user
  timing/frequency remains dynamically unmeasured.
- Conversation data durability and backend restore correctness remain delegated
  to A-STATE-01. This report only verifies frontend reconstruction/control flow.
- ChatEvent remains hand-written across Rust/TypeScript. A-FE-01 must perform the
  field/variant contract review instead of copying a new finding from here.
- Browser events are absent because of A-SRF-02-P1-02; Browser reducer behavior
  is not duplicated here.

## Handoff

- First fix the canonical application outcome under A-CHAT-01, then make one
  message-keyed frontend reducer release turn/queue only at backend Done.
- Add a conversation-keyed active-turn snapshot/replay and TaskRuntime cache;
  focus must derive solely from activeId. Preserve the existing TaskRuntime load
  generations and Subagent `(run_id, execution_id)` terminal guard.
- Route live/persisted/runtime Tool summaries through one monotonic merge and
  delete direct overwrite, not the durable detail repository.
- Primary must independently sample V02-V05 and run V99 before changing status
  from `needs_evidence` to `complete`.
