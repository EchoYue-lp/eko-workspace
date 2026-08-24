# A-FE-03: Frontend architecture, performance, and accessibility

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63 (baseline 9b0e0fa)
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5 (baseline b3b2e81)
> Worktree state: dirty — 79 modified files, all `web-frontend/src/generated/*.ts`; formatting-only, pre-existing (A-FE-01-P3-02). No other dirty paths in either repository. Generated-dir content verified unchanged by this review (aggregate md5 of `src/generated/*.ts`, web-frontend-relative: `ece079794ea6a9652859b2716229bcba`, before and after every executable validation; no ts-rs regeneration command was executed).

## Question

Are frontend components/stores organized around stable domain facts, with bounded rendering, cleanup of listeners/timers, accessible interactions, and no monolithic accidental state owners?

**Answer: mostly yes on the state-facts and cleanup axes, with two P2 and six P3 findings on the rendering/a11y/maintainability axes.** The 13 stores each own one domain concept (chat per-message streaming, task-runtime run, subagent-run projection, tool executions, browser, file, ui, toast, auth, workspace, right-workspace, subagent-detail) with no duplicate store authority; the 500-message in-memory cap (`chatStore.ts:104`) bounds memory on every growth path; listener/timer cleanup is exemplary (19/23 sites clean, the two race-window hooks explicitly guard unmount races); the task-runtime polling lifecycle is race-guarded and self-terminating. However: (P2) streaming is not render-bounded — every token re-renders all message bubbles via whole-chat store subscriptions and memo-defeating unstable callbacks, with no windowing (O(n) renders and O(n²) Set construction per token at the 500 cap); (P2) the turn lifecycle is split across two mirrors (hook refs + reducer state) with four `runStatus` writers and no monotonicity guard — the systematic "state-owner dispersion" that roots A-SRF-03-P1-01/P1-02; plus six P3 items (icon-only buttons without accessible names; modals without dialog semantics/focus management; a dead legacy SSE panel with a latent reconnect-churn defect; untested chatStore core reducer; an unremovable module-level auth timer/interval duplicating RequireAuth's check; and the 1103-line ChatInput monolith with hand-rolled menu exclusivity).

## Scope

Primary source paths inspected (full or behavior-slice reads):

- Root assembly/layout: `src/main.tsx`, `src/App.tsx`, `components/layout/AppLayout.tsx`, `LeftSidebar.tsx`, `RightRail.tsx`, `RightWorkspace.tsx`, `SettingsDialog.tsx` (dialog section), `components/common/CommandPalette.tsx`, `MarkdownContent.tsx`, `Toggle.tsx`, `components/Auth/RequireAuth.tsx`.
- Stores (all 13): `stores/chatStore.ts` (full), `subagentRunStore.ts` (contract), `taskRuntimeStore.ts` (full), `toolExecutionStore.ts` (contract; details A-FE-02), `conversationStore.ts`, `browserStore.ts`, `fileStore.ts`, `uiStore.ts`, `toastStore.ts`, `authStore.ts` (full), `workspaceStore.ts`, `rightWorkspaceStore.ts`, `subagentDetailStore.ts` (contract).
- Hooks: `useTauriChat.ts` (full, listener/queue slices), `chatEventHandler.ts` (contract), `useKeyboardShortcuts.ts` (full), `useBrowserEvents.ts` (full), `queuedChat.ts` (contract).
- Chat rendering: `components/chat/ChatPanel.tsx` (full), `MessageBubble.tsx` (full), `ParallelExecutionBlock.tsx` (full), `ChatInput.tsx` (full, 1103 lines), `InlineToolCall.tsx` (timers/lazy slice), `ThinkingSegment.tsx`/`ToolExecutionGroup.tsx`/`ExecutionProcessGroup.tsx`/`SubagentStreamBlock.tsx` (contract), `FailureToast.tsx` (full).
- Timers/listeners components: `components/tasks/TasksPanel.tsx` (full), `TerminalDrawer.tsx`, `Terminal.tsx` (contract), `FileBrowser.tsx` (top), `BrowserPanel.tsx`, `BrowserViewport.tsx`, `ChromeSetupDialog.tsx` (slices), `SubagentCard.tsx` (full), `SubagentDetailView.tsx` (contract), `task/TaskRuntimePanel.tsx` (InterruptPromptDialog :850-910).
- Large panels (characterization): `papers/ReviewWorkbench.tsx` (head + hook profile), `evolution/EvolutionPanel.tsx`, `plugins/PluginPanel.tsx`, `analysis/AnalysisPanel.tsx`, `observability/ObservabilityPanel.tsx`, `providers/ProviderPanel.tsx`, `scheduler/SchedulerPanel.tsx`, `workspace/NewTaskDialog.tsx` (hook profile + dialog semantics grep).
- Tests: all 26 test files inventoried; `chatStore` coverage grep.
- `package.json` (dependencies — virtualization check).

## Out Of Scope

- Chat-store reducer semantics, terminal mislabeling, interrupt ghost turn → A-SRF-03 (P1-01, P1-02, P2-01, P3-01) — consumed as dependencies, cross-referenced, not re-filed.
- Tool/subagent/task projection correctness, lazy-output identity → A-FE-02 (P2-01..P2-03, P3-01, P3-02).
- Type contracts, generated-vs-handwritten drift, dormant HTTP surface → A-FE-01.
- Backend producers, Tauri command lifecycle → A-SRF-02; chat driver → A-CHAT-01.
- Full submission gate (prettier/build as gates) → Q-WEB-01; GUI dynamic verification → Q-GUI-01/Q-E2E-01; runtime performance measurement → Q-PERF-01.
- TUI/CLI surfaces → A-SRF-01/A-SRF-04.

## Inputs

- Root `AGENTS.md` (full: UTF-8/panic safety, 防重复造轮子, framework-vs-app layering gate, "动手前先查是不是已经有了", read-only review, multi-surface feature parity).
- Shared `README.md`, `REPORTING.md`, `TASKS.md` (A-FE-03 card), `zcode-ds/README.md`, report templates.
- Dependency task reports read (zcode-ds): `A-SRF-03` (complete), `A-FE-01` (complete), `A-FE-02` (complete).
- Historical documents treated as hypotheses (classified in V05-01): `docs/MASTER-PLAN.md` (:124, :312, :789), `echo-agent-cli/docs/2026-07-25-gui-tool-execution-lazy-loading.md` (:32, :101), `echo-agent-cli/docs/gui-status.md`, `echo-agent-cli/docs/2026-07-16-agent-lifecycle-audit.md` (:161 P0-4).

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism (framework) | None in scope — the frontend consumes only Tauri-projected facts; no framework code is affected by this task's findings. |
| EKO product policy (application, correct placement) | Stores (chat/task/subagent/tool/browser/file/ui/toast/auth/workspace/right-workspace/subagent-detail), the 500-message cap, auto-save debounce, polling policy, queue policy, runStatus derivation, lazy tool output, layout/responsive policy, a11y surfaces — all application-layer. The P2/P3 findings below are application-layer defects in placement/behavior, not misplacement into the framework. |
| Adapter boundary | `useTauriChat`/`chatEventHandler`/`useBrowserEvents` are thin transport→store bridges; the turn-lifecycle split (P2-02) is a defect at this boundary (hook refs mirror store state instead of delegating). |
| Duplicate search (terms + results, V01-01) | `MessageBubble`, `ParallelExecutionBlock`, `visibleSubagentRuns`, `lastAssistantMessageId`, `knownMessageIds`, `runStatus` writers, `MAX_MESSAGES`, `setInterval`/`setTimeout`/`addEventListener` (23 sites inventoried), `EventSource`, `role="dialog"`, `aria-modal`, `worker` (zero frontend hits), `react-window`/`virtuoso` (zero), `TasksPanel`/`McpManagerPanel`/`PlanEditor`/`ResultFullView` (dead, zero importers), `setStreaming` (dead, zero callers — A-SRF-03-P3-01 cross-check). Result: no duplicate store concept; one authoritative chat-store; 3 whole-chat subscribers; no virtualization; 4 dead components. |
| Migration deletion | P2-01: no deletion — refactor subscriptions/props (direction below). P2-02: no deletion — converge the two mirrors onto one authority. P3-03: delete `TasksPanel.tsx` (and `McpManagerPanel.tsx` per A-FE-01-P3-01) or wire them only after fixing the churn. P3-05: remove the module-level authStore interval/focus listener and keep RequireAuth's interval. P3-06: extract sub-components from ChatInput. |

## Current Path

Verified call graph (V01-01/V02-01/V03-01):

1. **Mount**: `main.tsx` → `App.tsx` (:177-196) mounts `AppLayout(left=LeftSidebar, center=ChatPanel, right=RightWorkspace)` + `SettingsDialog` + `NewTaskDialog` + `CommandPalette` + `ToastContainer` + `InterruptPromptDialog` once each, inside `ErrorBoundary`/`RequireAuth`.
2. **Streaming render path**: backend `chat://event` → `useTauriChat` listener (:84-167, aborted+pendingCleanup race-safe) → `chatEventHandler` → `chatStore` actions; per token `appendToken`/`appendThinking` (chatStore.ts:241-267) replace the whole `messages` array; all subscribers of `(s) => s.messages` re-render — `ChatPanel.tsx:28`, and per bubble `MessageBubble.tsx:187` + `ParallelExecutionBlock.tsx:71` (rendered inside each bubble at MessageBubble.tsx:371/375). ChatPanel's unmemoized `handleRegenerate`/`handleEditAndResend` (ChatPanel.tsx:79-95) defeat `React.memo` (MessageBubble.tsx:153) on the props path as well. No virtualization library (package.json). Memory is capped: `trimToMax`/`MAX_MESSAGES = 500` (chatStore.ts:104-112) on all growth paths.
3. **Polling path**: `taskRuntimeStore.startPolling` (2 s, :133-155) idempotent, self-stopping on terminal status (:144-148), cleared by `stopPolling`/`reset`/`loadByConversation` (:157-163, :228, :375-392); `refresh` double generation-guarded + `refreshInFlight` (:165-217).
4. **Listeners/timers**: 23 sites inventoried (V02-01); 19 cleaned in effect cleanup, 2 intentional app-lifetime singletons (`chatStore.autoSaveTimer` :128-132 debounce; `authStore` 5-min interval + focus listener :111-122 — the only unremovable listener), 2 benign one-shot timers.
5. **A11y/labels**: role=main/log (ChatPanel :126/:168), role=tooltip (ChatInput :334), role=tab (BrowserTabs :23), role=switch (Toggle :11), role=dialog+aria-modal+aria-labelledby only in ChromeSetupDialog (:78); SettingsDialog/InterruptPromptDialog/NewTaskDialog/CommandPalette have no dialog semantics; icon-only buttons without accessible names at ChatInput.tsx:834-839/:883-889, SettingsDialog.tsx:369-374, TaskRuntimePanel.tsx:870-872, TasksPanel.tsx:251-267; `label htmlFor` used exactly once in the app.
6. **Responsive**: max-md overlays (AppLayout :26-39, RightWorkspace :52-54), viewport-adaptive right-workspace width (RightWorkspace.tsx:48), max-w-980 chat column (ChatPanel.tsx:175), FileBrowser mobile tree (FileBrowser.tsx:80).

## Findings

### A-FE-03-P2-01: Streaming is not render-bounded — every token re-renders all message bubbles via whole-chat store subscriptions and memo-defeating unstable callbacks; no windowing, O(n²) per-token Set construction at the 500-message cap

- Priority: P2
- Confidence: high (mechanism proven statically; magnitude extrapolated — no GUI process run)
- Layer: application (frontend rendering architecture)
- Evidence:
  - Per-token array identity: `chatStore.appendToken` (:241-249) / `appendThinking` (:251-267) produce a new `messages` array on every token; zustand v5 re-renders every subscriber whose selector value changed.
  - Whole-chat subscribers (V01-01): `ChatPanel.tsx:28`, `MessageBubble.tsx:187`, `ParallelExecutionBlock.tsx:71` — the latter two exist once per rendered bubble (ParallelExecutionBlock is mounted inside every MessageBubble at MessageBubble.tsx:371/375).
  - Per-bubble O(n) work: `ParallelExecutionBlock.tsx:72-84` rebuilds `lastAssistantMessageId` + `knownMessageIds = new Set(messages.map(m => m.id))` on every render; `MessageBubble.tsx:188-195` also builds `lastAssistantMessageId` + `new Set(chatMessages.map(...))` per render.
  - Memo defeat on the props path: ChatPanel's `handleRegenerate`/`handleEditAndResend` (ChatPanel.tsx:79-95) are plain functions re-created each render and passed to every bubble; `React.memo` (MessageBubble.tsx:153) cannot short-circuit prop changes, and it cannot short-circuit store-subscription re-renders at all.
  - No windowing: `package.json` has no react-window/virtuoso; the message list is a flat map (ChatPanel.tsx:177-222).
  - Memory is bounded but render cost is not: `MAX_MESSAGES = 500` + `trimToMax` (chatStore.ts:104-112) applied on every growth path (V03-01) — the cap bounds the DOM, not the per-token re-render fan-out.
- Reachability: every streaming GUI turn — user sends a message, backend streams tokens, `chatEventHandler` → `appendToken` per token; with n messages in the store (up to 500), each token triggers n bubble re-renders and n × O(n) Set constructions.
- Expected invariant: per-token updates re-render only the streaming message (mature chat products — Claude Code/Codex/Cursor desktop chats stay responsive in long threads; per-message subscription or windowing is the converged pattern); the memory cap is a backstop, not the rendering budget.
- Observed behavior: each streamed token re-renders every message bubble and re-runs per-bubble useMemo chains over the entire message list; at the 500 cap this is ~500 component renders + ~250k element ops per token on the main thread, plus full DOM diffing of all bubbles.
- Impact: main-thread jank and dropped frames during streaming in long conversations — a material performance defect on the flagship chat surface; also elevated CPU during every re-render for the whole session (each auto-save/finalize also re-renders everything).
- Root cause: bubbles derive per-message display facts (`lastAssistantMessageId`, `knownMessageIds`, `visibleSubagentRuns`) from whole-store subscriptions instead of receiving derived props; the parent list re-creates callback props without useCallback; and no windowing exists for the unbounded-by-design list (only capped).
- Direction: (1) compute `lastAssistantMessageId`/`knownMessageIds` once (ChatPanel memo or a derived store) and pass down; (2) subscribe MessageBubble per-message by id — `useChatStore((s) => s.messages.find(m => m.id === id))` returns a stable object reference for unchanged messages, so only the streaming message re-renders (same pattern already used by `InlineToolCall` for `tools[toolId]`, toolExecutionStore.ts:44); (3) wrap `handleRegenerate`/`handleEditAndResend` in `useCallback`; (4) for very long chats, window the list (e.g. react-virtuoso) or keep the cap as-is — windowing is the eventual bounded-rendering fix.
- Regression validation: a store+render fixture with 100 messages where appending a token to message k causes exactly one MessageBubble re-render (spy render counts); a prop-stability assertion that `handleRegenerate` identity is stable across ChatPanel re-renders; Q-PERF-01 large-chat measurement (500 messages, token stream) as acceptance.
- Validation reports: [V01-01](../validations/A-FE-03/V01-01.md), [V03-01](../validations/A-FE-03/V03-01.md)

### A-FE-03-P2-02: The chat turn lifecycle is owned by two mirrors — hook refs and reducer state — with four `runStatus` writers and no monotonicity guard; the systematic state-owner dispersion that roots A-SRF-03-P1-01/P1-02

- Priority: P2
- Confidence: high (static; the two P1 consequences are independently proven in A-SRF-03)
- Layer: application (frontend state architecture)
- Evidence:
  - Mirror 1 (hook refs): `useTauriChat.ts:24-33` — `currentMessageKeyRef`, `isCancelledRef`, `thinkingIdRef`, `queuedInputsRef` + `useState queuedInputs` (:33) mirror the store lifecycle; `isCurrentRunEvent` (:50-58) gates events on `currentMessageKeyRef`; the queue drains only on the `done` event (:69-71); `cancel()` (:314-327) clears neither ref.
  - Mirror 2 (reducer state): `chatStore` `runStatus`/`isStreaming`/`isCancelled` with four direct writers — `setRunStatus` (:391-397), `finalizeAssistantMessage` (:354-362, hardcodes `'completed'`), `markCancelled` (:399-416), `handoffToTaskRuntime` (:364-372) — plus indirect writers (`setApprovalRequest`/`setInputRequest`/`setSelectionRequest`/`addUserMessage`/`startAssistantMessage`), all last-write-wins with no monotone terminal guard.
  - Two-mirror drift consequences (cross-referenced, canonical in A-SRF-03): P1-01 the interrupt-prompt turn leaves `currentMessageKeyRef` set forever (no backend terminal; ref-clear inventory at A-SRF-03 V02-01 shows clears only at final_answer/error/cancelled/done), and P1-02 the error path ends at `'completed'` because `finalizeAssistantMessage` overwrites the `'failed'` set moments earlier (chatEventHandler.ts:140-150 → chatStore.ts:354-362).
  - Queue duplication: `queuedInputsRef` + `queuedInputs` state must be kept in sync by every mutation (`replaceQueue` :35-38) — a third mirror pair.
- Reachability: every chat turn (definition → registration → live path: ChatPanel → useTauriChat.sendMessage → dispatchMessage → chatStore + backend events; V02-01 of A-SRF-03).
- Expected invariant: one authority per lifecycle fact (MASTER-PLAN:124 "一条权威生命周期…前端只改变策略或渲染"; the task card's "no monolithic accidental state owners" and AGENTS.md "严禁平行实现同一语义") — the hook should delegate turn-terminal state to the store, or the store to the hook, not both.
- Observed behavior: the same turn is represented by refs (message key, cancel flag, thinking id, queue) AND store fields (runStatus, isStreaming, isCancelled, queue in state), with the two mirrors synchronized only by convention; any backend shape that produces neither a terminal event nor an expected invoke response (the interrupt prompt) strands the refs while the store shows 'running' forever; any error path produces contradictory store writes ('failed' then 'completed').
- Impact: the two P1 defects of A-SRF-03 are symptoms of this split, and any future terminal flow (steer, edit-and-resend, abandon) must remember to clear refs in yet another place — the defect class will keep reappearing (already three mirrors exist).
- Root cause: the turn lifecycle was built incrementally in the hook (refs) while the store gained its own status fields later, without a declared single owner; terminal-status derivation was left as last-write-wins across four actions instead of a monotone state machine.
- Direction: converge on one owner — keep per-turn identity in the hook but route all terminal/status writes through store actions with a monotonicity guard (no terminal → different terminal; error never becomes 'completed'), or move the message-key/queue into the store with the hook as a thin adapter; make the queue a single source (drop the ref mirror or the state mirror); add the interrupt-prompt terminal per A-SRF-03-P1-01 direction (rollback or backend terminal) so both mirrors always converge.
- Regression validation: hook/store fixtures — (a) interrupt-prompt invoke response leaves refs cleared and queue draining; (b) error→finalize sequence ends 'failed' with partial content; (c) a terminal-state transition table test asserting monotonicity for all four writers; (d) queue add/remove/reorder through one API only.
- Validation reports: [V01-01](../validations/A-FE-03/V01-01.md), [V03-01](../validations/A-FE-03/V03-01.md) (cross-refs: A-SRF-03 V01-01/V02-01/V03-01)

### A-FE-03-P3-01: Icon-only buttons without accessible names, and placeholder-as-label chat textarea — screen-reader users get unnamed controls on the flagship surface

- Priority: P3
- Confidence: high (static inventory)
- Layer: application (a11y)
- Evidence:
  - No text, aria-label, or title: ChatInput send button (ChatInput.tsx:883-889), ChatInput remove-file button (:834-839), SettingsDialog close button (SettingsDialog.tsx:369-374), InterruptPromptDialog dismiss (TaskRuntimePanel.tsx:870-872), TasksPanel refresh/plus (TasksPanel.tsx:251-267).
  - Title-only names (works for most AT but weaker): MessageBubble ActionButton (:473), ChatInput attachment/stop (:851/:878), AppLayout sidebar toggle (:48).
  - Textarea label: ChatInput textarea uses only `placeholder="Send follow-up"` (ChatInput.tsx:871) — no label/aria-label, and the placeholder is English inside a Chinese UI.
  - `label htmlFor` used exactly once in the whole app (V03-01); role=log/messages list is correctly `aria-live="polite"` (ChatPanel.tsx:168-170) — the main list is fine.
- Reachability: every GUI session — send button, file-attachment remove, settings close, interrupt dialog dismiss are all on core flows.
- Expected invariant: every interactive control has an accessible name (WAI-ARIA; desktop parity with TUI/CLI where no screen-reader dependence exists but standards still apply).
- Observed behavior: five icon-only controls announce as unnamed buttons; the input announces as a textarea whose only hint disappears once text is entered.
- Impact: screen-reader users cannot identify the primary send/stop/remove actions; keyboard users still can operate (buttons are native), so impact is limited to AT users — hence P3.
- Root cause: icon-only button pattern with `title`-as-label used inconsistently; no lint rule (jsx-a11y) enforces accessible names.
- Direction: add `aria-label` to the five unnamed buttons (and `aria-labels` to the title-only ones where the title text is already the right name); add `aria-label="消息输入"` (or a visually-hidden `<label>`) to the textarea; enable a jsx-a11y lint rule (`button-has-type`/`control-has-associated-label`) in CI to stop regressions.
- Regression validation: an axe/aria snapshot test over ChatPanel+ChatInput (or a simple DOM query asserting aria-label presence on the send button); `npx tsc -b` + vitest still green after adding labels.
- Validation reports: [V03-01](../validations/A-FE-03/V03-01.md)

### A-FE-03-P3-02: Modals lack dialog semantics and focus management — InterruptPromptDialog, SettingsDialog, NewTaskDialog, CommandPalette have no role/aria-modal/focus trap; only ChromeSetupDialog declares the pattern

- Priority: P3
- Confidence: high (static)
- Layer: application (a11y)
- Evidence:
  - `InterruptPromptDialog` (TaskRuntimePanel.tsx:850-910): fixed overlay, no `role="dialog"`, no `aria-modal`/`aria-labelledby`, no Escape handler, no initial focus, no focus trap — appears over the chat input on every mid-run message (live path).
  - `SettingsDialog` (SettingsDialog.tsx:344-355): backdrop + fixed dialog, no role/aria-modal/aria-labelledby, no focus management (Escape works, :333-340).
  - `NewTaskDialog`: no role/aria-modal (grep; only an inner onKeyDown at NewTaskDialog.tsx:334).
  - `CommandPalette`: overlay with Escape + input autofocus (:34, :60-66) but no role/aria-modal/aria-labelledby.
  - Only `ChromeSetupDialog` implements the pattern (`role="dialog" aria-modal="true" aria-labelledby="chrome-setup-title"`, ChromeSetupDialog.tsx:78-79).
- Reachability: interrupt dialog on every interrupted turn; settings on every open; palette on Cmd/Ctrl+K.
- Expected invariant: modal overlays expose dialog semantics and keep keyboard focus inside while open (desktop conventions; the interrupt dialog sits over an aria-live region, so AT users are additionally confused about which surface is active).
- Observed behavior: AT users get no dialog landmark; keyboard focus stays on the page behind the overlay (Tab can reach background controls); the interrupt dialog is unannounced.
- Impact: keyboard/AT users on core flows (interrupt choice, settings) get degraded modality; not a functional blocker for mouse users — P3.
- Root cause: dialog components were written before an a11y pattern was established; only the newest dialog (ChromeSetupDialog) follows it.
- Direction: add `role="dialog" aria-modal="true"` + `aria-labelledby` to the four dialogs, add Escape handling to InterruptPromptDialog (and NewTaskDialog), and implement a tiny shared `useFocusTrap`/`useModal` hook (initial focus, focus containment, restore on close) reused by all five; keep ChromeSetupDialog as the reference.
- Regression validation: vitest render fixtures asserting role/aria-modal/aria-labelledby present and initial focus lands inside; a focus-trap fixture (Tab from last control wraps inside).
- Validation reports: [V03-01](../validations/A-FE-03/V03-01.md)

### A-FE-03-P3-03: Legacy SSE `TasksPanel` is dead code (zero importers) and carries a latent defect — its 5 s fetch poll closes and re-creates every EventSource, and it targets a dormant HTTP surface

- Priority: P3
- Confidence: high (static; component never mounted)
- Layer: application (dead component with latent defect)
- Evidence:
  - Dead: `TasksPanel` has zero importers in `src/` (V01-01/V02-01); nothing mounts it; `McpManagerPanel` is likewise dead (A-FE-01-P3-01 canonical).
  - Latent churn: `fetchTasks` runs on a 5 s interval (TasksPanel.tsx:98-104) and calls `setTasks(data)` with a fresh array; the SSE effect depends on `[tasks, fetchTasks]` (TasksPanel.tsx:107-187), so every fetch re-runs its cleanup (closing ALL EventSources, :183-186) and re-opens them (:124-181) — SSE connections churn every 5 s while active tasks exist.
  - Dormant transport: the EventSource URL is `${protocol}//${window.location.host}/api/tasks/${task.id}/events` (:127-128) — the HTTP surface that A-FE-01 verified has no server in this workspace (dormant).
- Reachability: none today (dead). If a future task pane is wired from this component, the churn defect and the dormant URL become live.
- Expected invariant: no dead UI code under live-sounding names (AGENTS.md "删死代码"); SSE is long-lived — the connection lifecycle must not be tied to the poll cycle.
- Observed behavior: nothing renders; the component's design would reconnect every 5 s if mounted.
- Impact: maintenance burden + a trap for the next developer wiring a "background tasks" pane (reconnect storm against a server that also doesn't exist yet).
- Root cause: the SSE effect's dependency on the freshly-created `tasks` array couples connection lifetime to the poll cadence; the panel was superseded by RightRail/TaskRuntimePanel but never deleted.
- Direction: delete `TasksPanel.tsx` (and `McpManagerPanel.tsx`, per A-FE-01-P3-01) — or, if the background-task pane is re-planned, rewrite with (a) SSE opened once per task id (dependency on task ids, not the array) and (b) the Tauri command surface instead of the dormant HTTP URL.
- Regression validation: after deletion, grep `TasksPanel` returns only git history; if rewired, a fixture asserting one EventSource per task id across poll cycles.
- Validation reports: [V01-01](../validations/A-FE-03/V01-01.md), [V02-01](../validations/A-FE-03/V02-01.md)

### A-FE-03-P3-04: chatStore's core reducer — including the MAX_MESSAGES=500 cap that bounds large-chat memory — has zero direct unit tests

- Priority: P3
- Confidence: high (test-file inventory)
- Layer: application (test coverage)
- Evidence:
  - Test inventory (V04-01): the only chatStore test file is `stores/chatStore.toolExecution.test.ts` (2 tests, tool-execution slice); `hooks/chatEventHandler.test.ts` (4) covers the handler. Grep for `trimToMax|finalizeAssistantMessage|appendToken|MAX_MESSAGES` in all test files: zero hits (V03-01).
  - Untested invariants: `appendToken`/`appendThinking` (streaming accumulation, chatStore.ts:241-267), `finalizeAssistantMessage` (the unconditional 'completed' — A-SRF-03-P1-02), `markCancelled`, `handoffToTaskRuntime`, `setRunStatus` derived flags (:391-397), `trimToMax`/`MAX_MESSAGES` eviction (:104-112), `prepareRegenerate`/`prepareEditAndResend` (:478-526).
- Reachability: the cap is the memory backstop for large chats; the finalize/markCancelled actions are on every turn terminal.
- Expected invariant: the reducer that owns the flagship surface's lifecycle facts has fixture-level tests for its invariants (cap eviction, token accumulation, terminal statuses).
- Observed behavior: eviction of the oldest message at 500, streaming accumulation, and terminal-status transitions are all untested; the A-SRF-03-P1-02 defect (finalize overwriting 'failed') shipped with no test catching it.
- Impact: the exact class of defect that P1-02 is (reducer writes wrong terminal) can regress silently; the memory cap can be broken by a future growth path with no test alarm.
- Root cause: chatStore grew organically; the 6-10 test-suite expansion covered slices (tool execution, subagent, task runtime) but never the core chat reducer.
- Direction: add `stores/chatStore.test.ts` with fixtures: (a) 501 adds → 500 retained, oldest evicted, streaming message survives; (b) token/thinking accumulation merges into the target message only; (c) terminal transition table (idle→running→completed/failed/cancelled) asserting monotonicity per the P2-02 direction; (d) `prepareRegenerate`/`prepareEditAndResend` slices.
- Regression validation: the new fixtures pass; a mutation test (e.g., make finalize not clear isStreaming) is caught.
- Validation reports: [V03-01](../validations/A-FE-03/V03-01.md), [V04-01](../validations/A-FE-03/V04-01.md)

### A-FE-03-P3-05: Module-level authStore interval + window focus listener are never cleaned and duplicate RequireAuth's periodic auth check — two auth-check authorities

- Priority: P3
- Confidence: high (static)
- Layer: application (lifecycle hygiene)
- Evidence:
  - Module-level, no cleanup handle, no teardown: `authStore.ts:107-122` — `setInterval(checkAuth, 5 min)` + `window.addEventListener('focus', checkAuth)` at module scope.
  - Duplicate authority: `RequireAuth` runs its own 60 s `setInterval(checkAuth)` with proper cleanup (RequireAuth.tsx:21-26). Both timers call the same `checkAuth`; the module-level one is unreachable for tests and cannot be removed on logout/app teardown.
  - The auth surface itself is mostly dormant (RequireAuth comment "认证默认禁用"; A-FE-01 verified the HTTP auth surface is dormant), so the intervals mostly no-op — low impact.
- Reachability: every app session (module load); the listeners fire on focus/interval regardless of whether auth is used.
- Expected invariant: one auth-check cadence owned by one component/lifecycle; no unremovable global listeners (testability: vitest jsdom runs these intervals during tests).
- Observed behavior: two cadences (60 s + 5 min + focus) doing the same check; the module-level pair persists for the app lifetime and cannot be cleaned.
- Impact: redundant wake-ups (minor), test isolation hazard (intervals running in the test env), and a pattern other stores might copy — P3.
- Root cause: the periodic check predates RequireAuth's interval and was left in place when the component gained its own.
- Direction: delete the module-level interval + focus listener from authStore.ts:111-122 (keep `initFromStorage`/`checkAuth`); let RequireAuth's 60 s interval own the cadence; add focus-refresh inside RequireAuth if the immediate-on-focus behavior is desired.
- Regression validation: grep authStore for setInterval/addEventListener → zero; existing auth-related flows (none mounted in GUI default path) unchanged; vitest suite still green (V04-01).
- Validation reports: [V02-01](../validations/A-FE-03/V02-01.md)

### A-FE-03-P3-06: ChatInput is a 1103-line monolith with hand-rolled mutual exclusion of three dropdowns; four other panels exceed 1000 lines — maintainability debt with a concrete drift risk

- Priority: P3
- Confidence: high (static)
- Layer: application (maintainability)
- Evidence:
  - `ChatInput.tsx` (1103 lines) mixes: textarea + slash palette, file upload/paste/drag-drop, model switcher, permission-mode switcher, thinking-level switcher, interaction-mode switcher, context ring, draft-token estimate — 17 useState.
  - Hand-rolled menu exclusivity: each dropdown toggle manually closes the other two — permission button closes model+thinking (ChatInput.tsx:898-902), model button closes permission+thinking (:941-944), thinking button closes permission+model (:1016-1020) — the mutual-exclusion invariant is duplicated inline three times and will drift when a fourth menu is added.
  - Other >1000-line panels: `ReviewWorkbench.tsx` 1339 (15 useState, self-contained review-document editing — organized around the domain fact, just large), `EvolutionPanel.tsx` 1133 (22 useState), `PluginPanel.tsx` 818 (22 useState). Hook profile (V01-01): 7 panels with 10-22 useState each.
- Reachability: every chat session (ChatInput always mounted); the panels on their tabs.
- Expected invariant: components stay within a reviewable size with one shared pattern per repeated interaction (AGENTS.md layered, maintainable code; the app's own MenuOverlay refactor shows the team's intent to consolidate).
- Observed behavior: menu exclusivity is a three-copy invariant; large panels accumulate local editing state with no shared loading/error pattern (each panel hand-rolls fetch/loading/error useState).
- Impact: drift risk in the menu exclusivity logic; slow navigation through 1000+ line files; no functional defect today — P3.
- Root cause: feature accretion into the composer without extraction; panels were written as standalone screens before shared patterns existed.
- Direction: extract a `ComposerMenu`/dropdown model (one `openMenu: 'permission'|'model'|'thinking'|null` state instead of three booleans); extract file-paste and draft-token logic into hooks (`useFileAttachments`, already partially present); do not treat the panels' size as a defect by itself, but route them through shared `useAsyncData`-style helpers when touched (no forced refactor).
- Regression validation: a ChatInput render fixture cycling the three menus asserts exactly one menu open at a time; line-count guard (optional, no CI gate).
- Validation reports: [V01-01](../validations/A-FE-03/V01-01.md)

## Cross-Verified Dependency Findings (canonical IDs elsewhere; independently confirmed here)

| Canonical ID | Claim | Independent confirmation |
|---|---|---|
| A-SRF-03-P1-01 | Interrupt prompt strands the frontend turn state | Confirmed as the two-mirror architecture defect: refs (`currentMessageKeyRef` cleared only at terminal events) + store ('running' forever) — the ref-clear inventory and queue-drain trigger are the mechanism; A-FE-03-P2-02 is the architectural root-cause companion. |
| A-SRF-03-P1-02 | Error/cancel ends 'completed' with wiped content via `finalizeAssistantMessage` | Confirmed at the reducer (chatStore.ts:354-362) and additionally that the reducer has no test coverage for it (P3-04) and that `runStatus` has four writers with no monotone guard (P2-02). |
| A-SRF-03-P2-01 / A-FE-02-P2-01 | Live tool ingest keyed by `detail_ref`, not (owner, call_id) | Confirmed as a store-architecture note: `toolExecutionStore.ingest` (toolExecutionStore.ts:206-217) is the only path not using `executionIdentity`; not re-filed here. |
| A-FE-01-P3-01 | `McpManagerPanel` dead, connect flow dormant | Confirmed dead (zero importers) — grouped with the P3-03 dead-component cleanup. |
| A-FE-01-P3-02 | 79 generated files dirty (formatting-only) | Baseline confirmed before/after every executable validation; md5 of file contents stable (path-prefix variance explained in V01-01). |
| A-FE-02-P3-01 | `PlanEditor`/`ResultFullView` dead | Confirmed (zero importers); deletion target repeated in P3-03's direction. |
| MASTER-PLAN:789 | "56 个 Vitest" gate claim | Stale count — suite is 26 files/101 tests at the reviewed commit (V05-01). |

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search (store/component dependency map; >500-line inventory; whole-store subscriptions; runStatus writers; timers; dead components; virtualization) | yes | passed (P2-01/P2-02/P3-03/P3-06 evidence) | [V01-01](../validations/A-FE-03/V01-01.md) |
| V02 | Registration and runtime reachability (root assembly single-mount; 23 listener/timer sites cleanup audit; polling lifecycle; dead-surface imports) | yes | passed (P3-03/P3-05 evidence) | [V02-01](../validations/A-FE-03/V02-01.md) |
| V03 | Invariant/edge inspection (subscription cleanup; large-chat render behavior; keyboard/focus/label + responsive smoke; modal semantics; chatStore test coverage) | yes | passed (P2-01/P3-01/P3-02/P3-04 evidence) | [V03-01](../validations/A-FE-03/V03-01.md) |
| V04 | Targeted executable check — `npx vitest run` (full suite) | yes | passed (exit 0, 26 files / 101 tests) | [V04-01](../validations/A-FE-03/V04-01.md) |
| V04 | Targeted executable check — `npx tsc -b` | yes | passed (exit 0) | [V04-01](../validations/A-FE-03/V04-01.md) |
| V05 | Historical-document drift (MASTER-PLAN :124/:312/:789; lazy-loading doc; gui-status; lifecycle-audit P0-4) | yes | passed (1 regressed, 1 stale-count, rest current/n-a) | [V05-01](../validations/A-FE-03/V05-01.md) |

All required validations executed; every reported command has a known exit code; no validation is pending. No command that regenerates `web-frontend/src/generated/*.ts` was executed; the pre-existing 79-file dirty state and the generated-dir content were verified unchanged before and after every run.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN:124 "前端只改变策略或渲染，不改变状态语义；一条权威生命周期" | regressed | A-FE-03-P2-02 (two-mirror turn lifecycle; four runStatus writers) + A-SRF-03-P1-01/P1-02 (V05-01) |
| MASTER-PLAN:312 attachment data-URL is a frontend-owned persistence contract | current | conversationStore writes `data:` URLs, MessageBubble.tsx:241 renders `img.url` (V05-01) |
| MASTER-PLAN:789 frontend gate "56 个 Vitest" | stale (count) | suite now 26 files / 101 tests, still green (V04-01, V05-01) |
| 2026-07-25 lazy-loading doc:32/:101 tool payloads lazy, collapsed single-line | current | InlineToolCall.tsx:23/:91-105, 256 KiB cap (V05-01; single-producer assumption regressed per A-FE-02) |
| gui-status.md connected-surface matrix | current (no a11y/perf claims to drift) | V05-01 |
| chatStore.ts:104 "P0-4" comment (message cap) | current (un-documented detail; audit doc's P0-4 is a different item) | no historical claim contradicts the cap (V05-01) |

## Coverage And Uncertainty

- All conclusions are static traces plus the V04 suite/type-check runs; no GUI process was launched (Q-GUI-01/Q-E2E-01 own dynamic confirmation; Q-PERF-01 owns measured jank). P2-01's magnitude (frame drops at n messages) is extrapolated from the re-render mechanism, not measured.
- A11y claims are static inventories (accessible-name presence, role/aria attributes); no screen-reader or axe run was performed (Q-E2E-01 candidate).
- `Terminal.tsx`, `SubagentDetailView.tsx`, `conversationStore`, `subagentDetailStore`, `ThinkingSegment`/`ToolExecutionGroup`/`ExecutionProcessGroup`/`SubagentStreamBlock` internals were read at contract level only (their behavior slices belong to A-FE-02/A-SRF-03/Q-E2E-01); no finding here depends on their internals.
- The `authStore` interval runs in the vitest jsdom environment; no flake was observed across the V04 run, so no test-environment finding was filed beyond P3-05.
- P2-02's monotonicity table is proposed as the fix's regression fixture; the current behavior is last-write-wins, which the A-SRF-03 fixtures already demonstrate for the error path.

## Handoff

- Downstream tasks may rely on: one store per domain concept with no duplicate authority (V01-01); listener/timer cleanup is complete except two intentional singletons (V02-01); polling is race-guarded and self-terminating (V02-01); memory is capped at 500 messages on all growth paths while render cost is not (P2-01); the turn lifecycle is a two-mirror split with four runStatus writers (P2-02); a11y gaps are enumerable (P3-01, P3-02); dead components include TasksPanel/McpManagerPanel/PlanEditor/ResultFullView (P3-03 + A-FE-02-P3-01 + A-FE-01-P3-01); chatStore core reducer is untested (P3-04); suite and tsc green at the reviewed commits (V04-01).
- Findings for the roadmap: P2-01 (bounded rendering: per-message subscriptions + stable callbacks + optional windowing), P2-02 (single turn-lifecycle authority + monotone terminal transitions — the architectural companion to A-SRF-03-P1-01/P1-02), P3-01..P3-06 (a11y names, modal semantics, dead SSE panel deletion, chatStore tests, authStore cleanup, ChatInput extraction).
- Reports to read: this report + V01-01..V05-01; dependency reports A-SRF-03 (P1-01, P1-02, P2-01), A-FE-01 (P2-01/P2-02/P3-01..03), A-FE-02 (P2-01..P2-03, P3-01).
- Stale triggers: any change to `MessageBubble.tsx`/`ParallelExecutionBlock.tsx`/`ChatPanel.tsx` (subscriptions, callbacks, map), `chatStore.ts` (append/finalize/markCancelled/setRunStatus/trimToMax), `useTauriChat.ts` (refs/queue/listeners), `authStore.ts:107-122`, `TasksPanel.tsx`, `ChatInput.tsx` (menus/buttons), the dialogs (SettingsDialog/InterruptPromptDialog/NewTaskDialog/CommandPalette/ChromeSetupDialog), or package.json dependency additions invalidate the corresponding claims.
- Follow-up task IDs (fixes are not implemented in this review): A-SRF-03 (P1-01/P1-02 fixes consume P2-02's single-authority direction), X-EVT-01 (frontend terminal conformance), Q-PERF-01 (measured large-chat streaming cost; acceptance for P2-01), Q-GUI-01/Q-E2E-01 (axe/screen-reader smoke, modal focus behavior, dead-panel absence on screen), Q-WEB-01 (frontend gate incl. P3-02 of A-FE-01), S-RDM-01 (roadmap items for P2-01, P2-02, P3-01..P3-06).
