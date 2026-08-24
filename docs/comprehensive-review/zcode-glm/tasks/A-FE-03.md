# A-FE-03: Frontend architecture, performance, and accessibility

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: not-applicable (frontend-only review)
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only review)

## Question

Are frontend components/stores organized around stable domain facts,
with bounded rendering, cleanup of listeners/timers, accessible
interactions, and no monolithic accidental state owners?

## Scope

Primary source paths and behaviors inspected:

- **Store layer (every non-test store, read in full)**:
  - `echo-agent-cli/web-frontend/src/stores/chatStore.ts` (527 lines) —
    chat messages + streaming + context-window accumulator +
    auto-save debouncer.
  - `echo-agent-cli/web-frontend/src/stores/conversationStore.ts`
    (481 lines) — conversation list + load/save loop +
    generation-counter stale-drop.
  - `echo-agent-cli/web-frontend/src/stores/workspaceStore.ts` (110
    lines) — workspace list + switchTo cross-store orchestration.
  - `echo-agent-cli/web-frontend/src/stores/taskRuntimeStore.ts`
    (394 lines) — single active run, generation counter, polling,
    `MAX_EVENTS=500` cap.
  - `echo-agent-cli/web-frontend/src/stores/subagentRunStore.ts`
    (536 lines) — per-attempt execution-id aggregation key + terminal
    lock + `MAX_EVENTS_PER_RUN=300` cap; imports
    `isCanonicalUsageEvent` from `../components/compress/subagentUsage`.
  - `echo-agent-cli/web-frontend/src/stores/toolExecutionStore.ts`
    (255 lines), `browserStore.ts` (351 lines), `fileStore.ts`
    (302 lines), `uiStore.ts` (131 lines), `authStore.ts` (122 lines),
    `toastStore.ts` (28 lines), `rightWorkspaceStore.ts` (60 lines),
    `subagentDetailStore.ts` (15 lines).
- **Root assembly & routing**:
  - `App.tsx` (199 lines) — mount-time init effects, keyboard
    shortcuts wiring, command palette items, dialog composition.
  - `main.tsx` (10 lines) — `StrictMode` root.
  - `components/layout/AppLayout.tsx` (full) — 3-pane layout, mobile
    sidebar overlay, terminal drawer slot.
  - `components/layout/RightWorkspace.tsx` (head + listeners) —
    resize drag handlers.
- **Chat rendering hot path**:
  - `components/chat/ChatPanel.tsx` (551 lines) — message list
    `.map`, scroll handler, role="log" + aria-live region.
  - `components/chat/MessageBubble.tsx` (477 lines) — `memo`'d bubble
    subscribing to 3 stores (`messages`, `runs`, `activeRun`); the
    `lastAssistantMessageId` / `messageIds` `useMemo` derivation.
  - `components/chat/InlineToolCall.tsx` (269 lines) — pull-based
    tool detail + 500 ms live refresh + 256 KiB live cap.
  - `components/common/MarkdownContent.tsx` (123 lines) — `memo`'d
    `react-markdown` + `remark-gfm` renderer.
- **Listener/timer inventory (V02)** — every `setInterval`,
  `setTimeout`, `addEventListener`, and Tauri `listen(` call in
  `src/` (excluding `generated/`, `*.test.*`). 16 in-component
  `useEffect` sites + 3 module-scope sites audited.
- **Accessibility surfaces (V04)**:
  - `components/task/TaskRuntimePanel.tsx:850-922` —
    `InterruptPromptDialog` (modal).
  - `components/common/CommandPalette.tsx` (full, 132 lines) —
    keyboard navigation.
  - `components/layout/SettingsDialog.tsx:332-340` — Escape handler.
  - `components/terminal/TerminalDrawer.tsx:88-105` — mousemove/up
    drag handlers.
  - `components/chat/ChatInput.tsx:411-437, 864-873` — listener
    cleanup + unlabeled textarea.
  - `components/common/Toggle.tsx` (full) — exemplary switch.
  - `components/common/ErrorBoundary.tsx` (full).
  - `hooks/useKeyboardShortcuts.ts` (full).
- **Tooling baseline**:
  - `web-frontend/package.json` (scripts, devDeps, deps).
  - `web-frontend/tsconfig.json`, `tsconfig.app.json`,
    `tsconfig.node.json`.
  - `web-frontend/eslint.config.js` (present but dead — see V04).
- Whole-frontend greps for `setInterval`, `setTimeout`,
  `addEventListener`, `listen<`, `react-window|react-virtual`,
  `aria-|role=`, `<button`, `tabIndex`, `alt=`, `eslint`.

Whole test suite executed: `npx vitest run` → 26 files, 101 tests,
exit 0 (5.10 s).

## Out Of Scope

Deferred to downstream tasks (already audited by prior reports and
not re-litigated here):

- **A-SRF-03**: Tauri chat-transport receive-side contract, listener
  race-window hardening (`useTauriChat` abort/pendingCleanup pattern),
  and the `useBrowserEvents` listener-race divergence
  (A-SRF-03-P3-04). This task cites the pattern but does not re-audit.
- **A-FE-01**: Rust↔TypeScript IPC type contract.
- **A-FE-02**: Per-attempt identity, terminal monotonicity,
  acceptance/check projection (the reducer correctness angle).
- **A-STATE-01**: `ConversationStore` backend durability and atomic
  write semantics.
- The 17 panel components outside the chat path
  (`ReviewWorkbench`, `EvolutionPanel`, `PluginPanel`,
  `AnalysisPanel`, `ObservabilityPanel`, `ProviderPanel`,
  `SchedulerPanel`, `NewTaskDialog`, `SkillsPanel`, `TasksPanel`
  [except the SSE-thrash hot spot in V02], `PaperPanel`,
  `MemoryPanel`, `ConfigPanel`, `McpManagerPanel`, `PaperDetail`,
  `WorkflowDebugger`, `SandboxPanel`). These are independent panels
  with their own local state; their internal a11y / perf is not
  re-audited beyond the V04 cross-cutting smoke.
- The 1339-line `ReviewWorkbench.tsx` and 1133-line
  `EvolutionPanel.tsx` — they exceed 500 lines but live outside the
  primary chat/task path. Their size is noted as a smell (Coverage
  And Uncertainty), not audited as a finding.

## Inputs

Required repository documents read in full:

- Repository root `AGENTS.md` (multi-mode parity rule; framework-vs-
  application layering gate; "first prove no duplicate exists"
  implementation gate; no-panic / UTF-8 safety; the cleanup rule).
- `docs/comprehensive-review/REPORTING.md`,
  `templates/task-report.md`, `templates/validation-report.md`.
- `docs/comprehensive-review/TASKS.md` (A-FE-03 card + dependency
  list).

Dependency reports read:

- **A-SRF-03** (complete) — established the single chat transport
  (`useTauriChat` is the only `chat://event`/`execution://event`
  receiver), the receive-side reducer policy matrix, the
  `useBrowserEvents.ts` listener-race divergence (A-SRF-03-P3-04),
  and the `useTauriChat.ts:74-167` abort/pendingCleanup race-safe
  pattern. Load-bearing for V01/V02: the chat surface already has
  one well-hardened transport; this task looks for sibling patterns
  in other panels.
- **A-FE-01** (complete) — established the IPC type-contract matrix
  and the absence of a contract test. Load-bearing for V04: the
  absence of any enforced lint/test discipline for the frontend
  means a11y issues are not caught at CI time either.
- **A-FE-02** (complete) — established the per-attempt identity
  model, the bounded event logs (`MAX_EVENTS_PER_RUN=300`,
  `MAX_EVENTS=500`), and the 256 KiB live-load cap on
  `InlineToolCall`. Load-bearing for V03: those bounds are reused
  as the rendering-budget assumptions.

Historical documents treated as hypotheses:

- `chatStore.ts:102-112` comment — claims `MAX_MESSAGES=500`
  prevents OOM on very long conversations ("P0-4") and is applied
  to every grow path. Verified current by V03; the cap holds but
  is not paired with virtualization.
- `MessageBubble.tsx:153` `memo` wrapper — assumed to bound
  re-renders. Falsified for the streaming-token hot path by V03:
  `memo` does not stop Zustand-initiated re-renders.

## Layering Decision

This is an **application-layer** task. All inspected code lives in
`echo-agent-cli/web-frontend/src/`. The framework supplies no
frontend artifacts; the only framework contract consumed is the
Tauri IPC wire shape already audited by A-SRF-01/02/03 + A-FE-01/02.

| Classification | Required answer |
|---|---|
| Generic mechanism | Zustand `create` store + React `useEffect` + Tauri `listen`/`invoke` IPC. All used as transport/state primitives. |
| EKO product policy | Every store and component audited is EKO product policy, correctly in the frontend. The findings are about *how* they are wired (circular import, wide subscription, missing a11y), not *where* they live. |
| Adapter boundary | The transport adapters (`useTauriChat`, `useBrowserEvents`, `Terminal.tsx`'s terminal-output listener) are correctly single-mount. The `taskRuntimeSubagentExecutionEvents` / `taskRuntimeToolExecutions` projections (audited in A-FE-02) are thin. No new adapter-boundary violation found. |
| Duplicate search | Searched `web-frontend/src` for: a second chat transport (zero — A-SRF-03 confirmed `useWebSocket` removed); a second virtualization helper (zero — no virtualization library installed); a second modal-focus helper (zero — each modal rolls its own Escape / focus handling); a second auto-save debouncer (zero — `chatStore`'s `scheduleAutoSave` is the only one); any `react-window\|react-virtual\|virtuoso` import (zero hits). No parallel implementation remains. |
| Migration deletion | No deletion proposed. The findings identify cross-store coupling, a wide-subscription hot path, dev-only leaks, and a11y gaps; resolution is left to follow-up task IDs. |

## Current Path

### Store inventory and dependency graph (V01)

13 non-test Zustand stores, organized around stable domain facts:

```text
Domain fact                                Store                     Lines
─────────────                              ──────                    ─────
chat messages + streaming state            chatStore                 527
conversation list + load/save loop         conversationStore         481
subagent per-attempt lifecycle             subagentRunStore          536
tool execution summaries (live + hydrate)  toolExecutionStore        255
active TaskRun + plan + todos + events     taskRuntimeStore          394
workspace list + switch orchestration      workspaceStore            110
in-browser preview sessions                browserStore              351
file tree + open documents + dirty state   fileStore                 302
UI: sidebar / theme / terminal / settings  uiStore                   131
auth: session token + 401 re-auth          authStore                 122
toasts                                     toastStore                 28
right-rail tab + drag resize               rightWorkspaceStore        60
selected subagent detail                   subagentDetailStore        15
```

Dependency edges (arrow = "imports and calls `.getState()` at
runtime"):

```text
workspaceStore ──┐
                 ├─→ chatStore.clearMessages()
                 ├─→ conversationStore.setState / init()
                 │
chatStore ───────┼─→ conversationStore.saveCurrent() (via autoSave)
conversationStore ─→ chatStore.getState() (clearMessages, messages)
                 │
taskRuntimeStore ─→ subagentRunStore.ingestTaskRuntimeSubagentEvents
                 ├─→ toolExecutionStore.taskRuntimeToolExecutions
                 │
subagentRunStore ─→ ../components/compress/subagentUsage.isCanonicalUsageEvent
```

### Cross-store coupling (V01)

- **`chatStore ↔ conversationStore` form a runtime cycle**
  (`chatStore.ts:3` imports `useConversationStore`;
  `conversationStore.ts:2` imports `useChatStore`). Each calls
  `getState()` on the other:
  - `chatStore.scheduleAutoSave` → `conversationStore.saveCurrent(msgs)`
    (`chatStore.ts:118-123`).
  - `conversationStore.loadConversation` / `startNew` / `clearCurrent`
    → `chatStore.getState().clearMessages()` and
    `chatStore.getState().messages` reads
    (`conversationStore.ts:375,415,440,457,472`).
  The cycle is structurally safe (both `create()` calls run at
  module load before either is read), and `getState()` is resolved
  at call time, so there is no TDZ. The cost is testability and
  modularity: the two stores cannot be loaded in isolation and any
  future split (e.g. extracting auto-save into its own module) must
  break the cycle.
- **`workspaceStore` → `chatStore + conversationStore`** is one-way
  (switchTo clears chat and re-inits conversations). Not a cycle.
- **`taskRuntimeStore` → `subagentRunStore + toolExecutionStore`** is
  one-way (the projection adapters; audited in A-FE-02).

### Layering smell (V01)

`subagentRunStore.ts:17` imports
`isCanonicalUsageEvent` from `../components/compress/subagentUsage.ts`.
That file is pure TypeScript (no JSX, no React import) but lives
under `components/`. The dependency direction (store → component
tree) is the wrong way around: a store reaching into the component
directory for shared logic creates a layering inversion and makes
`components/compress/` un-deletable as long as the store needs it.
`subagentUsage.ts` should live under `lib/` or `utils/`.

### Listener / timer cleanup (V02)

Audited every `setInterval`, `setTimeout`, `addEventListener`, and
Tauri `listen(` call in `src/` (excluding `generated/`, `*.test.*`).

**16 in-component `useEffect` sites with listeners/timers — all
correctly clean up:**

| Site | Pattern | Cleanup |
|---|---|---|
| `useTauriChat.ts:84-167` | Tauri `listen` × 2 + `aborted` flag + `pendingCleanup` array | yes (covers 3 race windows; A-SRF-03) |
| `useBrowserEvents.ts:6-36` | Tauri `listen` + `disposed` flag | yes (less hardened than useTauriChat; A-SRF-03-P3-04) |
| `InlineToolCall.tsx:55-58` | `setInterval(setNow, 250)` while running | yes |
| `InlineToolCall.tsx:91-105` | `setInterval(loadPage, 500)` while expanded+running | yes |
| `TasksPanel.tsx:98-104` | `setInterval(fetchTasks, 5000)` | yes |
| `TasksPanel.tsx:107-187` | `EventSource` per active task | yes (closes all on unmount) **— but re-subscribes on every tasks update; see V02** |
| `BrowserPanel.tsx:25-31` | `setInterval(refreshFrame, 1500)` | yes |
| `ChromeSetupDialog.tsx:38-49` | `setInterval(refresh, 2000)` while open | yes |
| `SubagentCard.tsx:25-31` | `setInterval(elapsed, 1000)` | yes |
| `FileBrowser.tsx:38-44` | `setInterval(loadChanges, 2500)` | yes |
| `FileBrowser.tsx:46-55` | `addEventListener('keydown', save)` (Cmd+S) | yes |
| `RequireAuth.tsx:18-25` | `setInterval(checkAuth, 60s)` | yes |
| `ChatInput.tsx:415-437` | 4× `addEventListener` (focus + custom events) | yes |
| `SettingsDialog.tsx:333-340` | `addEventListener('keydown', Esc)` | yes |
| `TerminalDrawer.tsx:88-105` | `addEventListener('mousemove'/'mouseup', drag)` | yes |
| `RightWorkspace.tsx:30-46` | `addEventListener('pointermove'/'pointerup'/'resize', drag)` | yes |
| `Terminal.tsx:182-188` | container click handler + Tauri `listen` × 2 | yes |
| `useKeyboardShortcuts.ts:18-43` | `addEventListener('keydown', shortcuts)` | yes |

**3 module-scope sites — never cleaned up (dev-only leak):**

- `authStore.ts:107-122` — `setInterval(checkAuth, 5 min)` +
  `addEventListener('focus', checkAuth)` at module scope. Both
  intentionally page-lifetime. In production this is fine (the page
  is the application). In dev with HMR, every module re-evaluation
  adds a new interval+listener without removing the previous one
  (Vite HMR re-runs module top-level).
- `chatStore.ts:128-132` — module-scope `autoSaveTimer` debouncer.
  Same dev-only leak pattern. Also: a pending `autoSave` callback
  captures `useChatStore.getState()` at fire time, which is correct
  (resolved lazily) but means a pending timer always sees the latest
  state — including, in a test, after the store is replaced.

**One render-thrash hot spot (V02):**

`TasksPanel.tsx:107-187` — the SSE subscription `useEffect` depends
on `[tasks, fetchTasks]`. `tasks` is local `useState` updated via
`setTasks` after every poll (5 s) and every SSE-done refresh. Each
update creates a new array reference → the effect re-runs → cleanup
closes ALL `EventSource`s → next effect re-opens them. For an active
task receiving progress events, every progress callback can cascade
into a poll → setTasks → effect re-run → close+reopen every active
SSE. Not a memory leak (cleanups run), but a connection thrash and
a lost-progress-events risk (events arriving between close and
reopen are dropped).

### Rendering budget for large chats/tasks (V03)

- **No virtualization library installed**
  (`grep react-window\|react-virtual\|@tanstack/virtual\|virtuoso`
  in `package.json` and `src/` returns zero hits).
- **`MAX_MESSAGES = 500` cap** (`chatStore.ts:104-112`) is applied
  to every grow path (`addUserMessage`, `startAssistantMessage`,
  `continueAfterSteer`, `replaceMessages` via `trimToMax`). The
  in-memory list is bounded but **every one of those 500 bubbles
  renders in the DOM** (`ChatPanel.tsx:177` `.map`).
- **`MessageBubble` is `memo`'d** (`MessageBubble.tsx:153`) — this
  blocks re-renders caused by *parent* re-renders, but does NOT
  block re-renders caused by *Zustand subscriptions inside the
  child*.
- **Hot-path wide subscription**: `MessageBubble` subscribes to
  three stores (`MessageBubble.tsx:185-187`):
  ```ts
  const activeRun = useTaskRuntimeStore((state) => state.activeRun);
  const subagentRuns = useSubagentRunStore((state) => state.runs);
  const chatMessages = useChatStore((state) => state.messages);
  ```
  The `chatMessages` subscription is used only to derive
  `lastAssistantMessageId` and `messageIds` (`useMemo` at
  `:188-195`). On every `appendToken` reducer
  (`chatStore.ts:241-249`), the `messages` array reference changes
  → Zustand notifies every `MessageBubble` subscriber → each
  bubble's `useMemo` cache invalidates → every bubble re-renders
  (its MarkdownContent is `memo`'d against the unchanged
  `message.content` prop, so the heavy render is skipped for
  non-streaming bubbles, but the bubble's own function body runs
  and re-evaluates `flattenSteps`/`groupExecutionSteps`/etc.). With
  N=500 bubbles and T streaming tokens, this is O(N·T) work per
  turn.
- **Bounded event logs** (audited in A-FE-02 V03):
  `MAX_EVENTS_PER_RUN=300`, `MAX_EVENTS=500`. Not rendered
  (only plan/todos/artifacts are).
- **Bounded tool output** (A-FE-02 V03): `LIVE_DETAIL_AUTOLOAD_CHARS
  = 256 * 1024` live cap, cursor pagination, manual "load more".

### Accessibility smoke (V04)

Cross-cutting inventory (whole-tree greps):

| Primitive | Count |
|---|---|
| `<button>` elements | 292 |
| `aria-*` / `role=` occurrences | 46 (≈16% of buttons) |
| `tabIndex` | 3 |
| `alt=` (on `<img>`) | 4 |
| ESLint config enforcing react-hooks / jsx-a11y | **none installed** (see below) |

**Modal accessibility — three modals, three different qualities:**

| Modal | role="dialog" | aria-modal | Escape | Autofocus | Focus trap | Backdrop click |
|---|---|---|---|---|---|---|
| `SettingsDialog.tsx:332-340` | no | no | yes | no | no | yes |
| `CommandPalette.tsx:25-50` | no | no | yes | yes (50 ms) | no | yes |
| `InterruptPromptDialog.tsx:850-922` | no | no | **no** | **no** | **no** | **no** |

The `InterruptPromptDialog` is the worst: it renders a full-screen
overlay with `style={{ background: 'rgba(0,0,0,0.4)' }}` that blocks
pointer interaction with the rest of the UI, but has no keyboard
way to dismiss, no focus management, no semantic role, and the
backdrop itself is not clickable to close. A screen-reader user
cannot tell a modal opened; a keyboard user who was focused
elsewhere when it opened has no obvious path to the buttons.

**Other a11y issues:**

- `ChatInput.tsx:864-873` — the primary chat `<textarea>` has no
  `aria-label`, no `<label>`, no `title`. The placeholder ("Send
  follow-up") is the only identifier. Screen readers announce it
  as an unnamed text field.
- `AppLayout.tsx:26-33` — the mobile sidebar backdrop has only
  `onClick={toggleSidebar}`. No keyboard way to close the sidebar
  (no Escape, no close button on the overlay).
- `MessageBubble.tsx:298-301, 382-397` — hover-only action buttons
  (Copy / Edit / Regenerate) rely on `group-hover/msg:opacity-100`
  to become visible. They remain in the DOM and are technically
  focusable, but a sighted keyboard user sees no focus indicator
  until they happen to tab into the invisible cluster.
- `InlineToolCall.tsx:156-175` — the expand toggle `<button>` uses
  `title={summary}` only; the visible label is the tool name +
  args preview (acceptable, but the status/duration are read as
  raw text without `aria-label`).

**Positive a11y examples:**

- `common/Toggle.tsx` — exemplary: `role="switch"` +
  `aria-checked={checked}` + `sr-only` label span.
- `MessageBubble.tsx:305-318, 399-413` — edit `<textarea>`s have
  `autoFocus` + Enter-to-submit + Escape-to-cancel.
- `ChatPanel.tsx:168-170` — the message log has `role="log"` +
  `aria-live="polite"` + `aria-label="消息列表"` — a streaming
  region screen readers can follow.

**Tooling gap (V04 supporting evidence):**

`web-frontend/eslint.config.js` exists and configures
`eslint-plugin-react-hooks` (which would enforce
`exhaustive-deps`) + `typescript-eslint`. But:

- ESLint is **not** in `devDependencies`
  (`npm ls eslint` returns `(empty)`).
- ESLint is **not** installed in `node_modules`.
- `npm test` and `npm run build` (`tsc -b && vite build`) do NOT
  invoke ESLint.
- `eslint-plugin-jsx-a11y` is **not** configured.

So the lint config is dead code. No automated check enforces
react-hooks rules (exhaustive-deps, rules-of-hooks) or any a11y
rule. A future contributor editing the file would not see lint
errors locally or in CI.

## Findings

### A-FE-03-P2-01: `MessageBubble` subscribes to `chatStore.messages` for every bubble — every streaming token re-renders all 500 bubbles

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/web-frontend/src/components/chat/MessageBubble.tsx:185-187`
    — three subscriptions:
    ```ts
    const activeRun = useTaskRuntimeStore((state) => state.activeRun);
    const subagentRuns = useSubagentRunStore((state) => state.runs);
    const chatMessages = useChatStore((state) => state.messages);
    ```
    The third subscription is used only to derive
    `lastAssistantMessageId` (`:188-194`) and `messageIds`
    (`:195`) — both `useMemo`'d over the entire messages array.
  - `echo-agent-cli/web-frontend/src/stores/chatStore.ts:241-249`
    — `appendToken` returns a brand-new `messages` array on every
    token (`updated[idx] = { ...updated[idx], content: ... }`).
    Zustand's default `Object.is` equality check on the slice
    `state.messages` therefore reports a change on every token →
    every subscriber re-renders.
  - `echo-agent-cli/web-frontend/src/components/chat/MessageBubble.tsx:153`
    — `export const MessageBubble = memo(function MessageBubble(...)`.
    `React.memo` only short-circuits re-renders driven by *parent*
    prop changes. Zustand hooks bypass React's prop pipeline and
    re-render the component directly on store change.
  - `echo-agent-cli/web-frontend/src/components/chat/ChatPanel.tsx:177`
    — every message renders via `messages.map((msg, idx) => <div
    key={msg.id}><MessageBubble .../></div>)`. With `MAX_MESSAGES =
    500` (`chatStore.ts:104`), up to 500 `MessageBubble` instances
    are mounted simultaneously, each subscribed to `messages`.
- Reachability: every chat turn. Streaming produces T tokens; each
  token triggers N `MessageBubble` re-renders (where N is the
  number of currently mounted bubbles, up to 500). For a 100-token
  turn after a 200-message history, that is 100 × 200 = 20 000
  bubble-function invocations, each of which re-runs
  `flattenSteps(message)` (`:183`) and `groupExecutionSteps(steps)`
  (`:184`) on its own message.
- Expected invariant: a token append should re-render only the
  streaming bubble, not all bubbles.
- Observed behavior: every bubble re-renders on every token. The
  heavy `MarkdownContent` is `memo`'d against `content` so it does
  not re-parse, but the bubble's own function body (and the
  `useMemo` cache invalidation for `lastAssistantMessageId` /
  `messageIds`) runs N times per token.
- Impact: visible lag on long conversations during streaming. The
  500-bubble cap (`MAX_MESSAGES`) prevents unbounded growth, so
  the worst case is bounded — but it is still O(N·T) per turn. On
  a 500-bubble conversation, a 1000-token answer triggers ~500 000
  bubble-function invocations. Profiling was not performed in
  this static review; the impact is a perf smell, not a measured
  regression.
- Root cause: `MessageBubble` reaches across the data graph
  (subscribing to all messages) to derive a "is this the last
  assistant bubble?" flag and a "does this message id still
  exist?" set. The cross-bubble fact is needed by
  `visibleSubagentRuns` (`:196-206`) to associate subagent runs
  with the last assistant message.
- Direction: lift the two derived values out of `MessageBubble`
  and into `ChatPanel`. Compute `lastAssistantMessageId` once in
  `ChatPanel` (which already subscribes to `messages`) and pass it
  as a prop. Compute `messageIds` similarly, or replace the Set
  lookup with a stable "is this id still mounted" check that does
  not require the full array. Optionally switch
  `useChatStore((state) => state.messages)` to a narrower selector
  that returns only the streaming bubble's id. Either change
  decouples `MessageBubble` from the `messages` array reference.
- Regression validation: a vitest render test that mounts N
  bubbles, dispatches `appendToken` on one, and asserts the other
  N-1 bubble's render count stays at 0 (use the React Profiler
  or a `useEffect`-counter fixture). No such test exists today.
- Validation reports: [V03-01](../validations/A-FE-03/V03-01.md).

### A-FE-03-P2-02: `InterruptPromptDialog` is a non-interactive modal — no role, no focus, no Escape, no backdrop click

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/web-frontend/src/components/task/TaskRuntimePanel.tsx:850-922`
    — the full component:
    ```tsx
    return (
      <div className="fixed inset-0 z-50 flex items-center justify-center"
        style={{ background: 'rgba(0,0,0,0.4)' }}>
        <div className="mx-4 w-full max-w-sm rounded-lg p-4 shadow-[var(--shadow-lg)]"
          style={{ background: 'var(--bg-primary)', border: '1px solid var(--border-primary)' }}>
          ...
          <button onClick={dismiss}>...<X size={14}/>...</button>
          ...
          <button onClick={() => { resume(); }}>继续执行旧计划</button>
          <button onClick={async () => { dismiss(); }}>编辑计划后继续</button>
          <button onClick={async () => { ... cancel_task_run ...; dismiss(); }}>
            废弃旧计划，开始新任务
          </button>
        </div>
      </div>
    );
    ```
    No `role="dialog"`, no `aria-modal="true"`, no `aria-labelledby`,
    no `onKeyDown` Escape handler, no `autoFocus`, no focus trap,
    and the backdrop `<div>` has no `onClick` (only the inner card
    handles clicks).
  - `echo-agent-cli/web-frontend/src/components/common/SettingsDialog.tsx:332-340`
    — by contrast, `SettingsDialog` has an Escape handler; the
    backdrop is clickable (`:347-351`). It still lacks
    `role="dialog"` and focus trap, but is closer to the baseline.
  - `echo-agent-cli/web-frontend/src/components/common/CommandPalette.tsx:25-50`
    — by contrast, `CommandPalette` has ArrowUp/Down/Enter/Escape
    + autofocus on the input. Still lacks `role="dialog"` and
    focus trap, but is the most keyboard-friendly of the three.
- Reachability: the dialog renders whenever
  `useTaskRuntimeStore.interruptPrompt` is non-null — i.e. every
  time a user issues a new task while another task run is still
  active (a core HITL gate for the complex-task flow per
  A-CHAT-01 / A-TSK-06). This is a primary user touchpoint for
  task supervision, not an edge case.
- Expected invariant: a modal that blocks the underlying UI must
  (a) announce itself to assistive technology (`role="dialog"` +
  `aria-modal="true"`), (b) move focus into the dialog on open,
  (c) trap focus while open, (d) close on Escape, and (e) close
  on backdrop click. WAI-ARIA Authoring Practices, "Dialog
  Pattern".
- Observed behavior: none of the five invariants hold. A
  screen-reader user receives no announcement that a modal
  opened; a keyboard user who was typing in the chat textarea
  when the dialog appeared has focus stuck on a now-hidden
  element; pressing Escape does nothing; clicking the dark
  backdrop does nothing. The user can only interact with the
  dialog by tabbing blindly or switching to mouse.
- Impact: medium-high for the task-supervision UX (per AGENTS.md,
  EKO is a "local personal super-intelligence assistant" that the
  user supervises; the interrupt prompt is the primary "what do
  you want to do with the running task?" gate). Severity is P2
  (not P1) because the dialog does render and mouse users can
  click; the gap is accessibility, not functionality.
- Root cause: the dialog was written as a quick inline component
  inside `TaskRuntimePanel.tsx` (it's a named export at line 850
  of the same file) without going through any a11y pattern
  library. The Settings and CommandPalette modals evolved
  incrementally and partially; InterruptPromptDialog did not.
- Direction: extract a shared `Modal` / `Dialog` primitive under
  `components/common/` that handles role, aria-modal, focus
  trap (e.g. via `focus-trap-react` or a small custom
  implementation), Escape, autofocus to the first focusable, and
  backdrop click. Migrate all three modals (SettingsDialog,
  CommandPalette, InterruptPromptDialog) to use it. Minimum
  interim fix for InterruptPromptDialog: add
  `role="dialog" aria-modal="true" aria-label="任务中断提示"`,
  an Escape handler calling `dismiss`, an `autoFocus` on the
  first action button, and `onClick={dismiss}` on the backdrop.
- Regression validation: a vitest + `@testing-library/user-event`
  test that renders `InterruptPromptDialog`, tabs through focus,
  presses Escape, clicks the backdrop, and asserts the dialog
  dismissed. Pair with the same test for the other two modals.
- Validation reports: [V04-01](../validations/A-FE-03/V04-01.md).

### A-FE-03-P2-03: `chatStore` ↔ `conversationStore` form a runtime circular import

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/web-frontend/src/stores/chatStore.ts:3` —
    `import { useConversationStore } from './conversationStore';`
  - `echo-agent-cli/web-frontend/src/stores/conversationStore.ts:2`
    — `import { useChatStore } from './chatStore';`
  - `echo-agent-cli/web-frontend/src/stores/chatStore.ts:118-123`
    — runtime call:
    ```ts
    function autoSave() {
      const msgs = useChatStore.getState().messages;
      if (msgs.length > 0) {
        void useConversationStore.getState().saveCurrent(msgs);
      }
    }
    ```
  - `echo-agent-cli/web-frontend/src/stores/conversationStore.ts:375,415,440,457,472`
    — runtime calls: `useChatStore.getState().clearMessages()`
    (4 sites) and `useChatStore.getState().messages` read (1 site).
- Reachability: every chat turn (auto-save) and every
  conversation switch (clearMessages). The cycle is on the hot
  path.
- Expected invariant: stores should depend on each other in a
  DAG. Two stores importing each other and calling `getState()`
  is structurally safe in Zustand (the imports are resolved at
  module load, the calls are deferred to runtime) but creates a
  tight coupling that (a) prevents independent unit testing,
  (b) makes future extraction (e.g. pulling auto-save into its
  own module) require breaking the cycle, and (c) makes the
  dependency graph harder to reason about.
- Observed behavior: the cycle works today. There is no TDZ, no
  infinite loop, no observable bug. The cost is architectural.
- Impact: medium. No runtime defect; future maintenance is
  harder. The same auto-save concern is owned by both stores:
  `chatStore` knows *when* to save (debounced token reducer),
  `conversationStore` knows *how* to save (network call). The
  coupling makes the responsibility boundary unclear.
- Root cause: the auto-save wiring was added inside `chatStore`
  (which knows when messages change) but the save implementation
  lives in `conversationStore` (which owns the network). The
  natural way to bridge them was a direct import; the cleaner
  alternative (an event/observer or an explicit controller) was
  not adopted.
- Direction: pick one of
  1. **Invert the dependency** — have `conversationStore`
     subscribe to `chatStore` changes (e.g. via a
     `subscribeWithSelector` middleware on `chatStore` that
     emits to `conversationStore.scheduleSave`). Then remove
     the `chatStore → conversationStore` import.
  2. **Extract a coordinator** — lift the auto-save orchestration
     into a small module (e.g. `lib/autoSave.ts` or a React
     effect in `ChatPanel`) that imports both stores in one
     direction only. Both stores then import the coordinator,
     not each other.
  3. **Leave the cycle, document it** — add a comment at both
     import sites explaining why the cycle is safe and what would
     break if either store moved.
  Option 1 is the cleanest; option 3 is the smallest.
- Regression validation: no test exists today that exercises the
  cycle in isolation. After the change, a vitest test that
  imports `chatStore` without importing `conversationStore` (or
  vice versa) should succeed.
- Validation reports: [V01-01](../validations/A-FE-03/V01-01.md).

### A-FE-03-P3-01: `TasksPanel` SSE `useEffect` re-subscribes on every `tasks` update — progress events can thrash connections

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/web-frontend/src/components/tasks/TasksPanel.tsx:51`
    — `const [tasks, setTasks] = useState<BackgroundTask[]>([]);`
  - `echo-agent-cli/web-frontend/src/components/tasks/TasksPanel.tsx:107-187`
    — the SSE effect:
    ```ts
    useEffect(() => {
      const activeTasks = tasks.filter((t) => !['completed', ...].includes(t.status));
      ...
      // Close SSE for completed tasks
      existingIds.forEach((id) => {
        if (!currentIds.has(id)) { eventSourcesRef.current[id]?.close(); ... }
      });
      // Open SSE for new active tasks
      activeTasks.forEach((task) => {
        if (eventSourcesRef.current[task.id]) return;
        const es = new EventSource(url);
        eventSourcesRef.current[task.id] = es;
        es.addEventListener('progress', ...);
        ...
      });
      return () => {
        Object.values(eventSourcesRef.current).forEach((es) => es.close());
      };
    }, [tasks, fetchTasks]);
    ```
    The dependency array includes `tasks` (a local `useState`).
    Every `setTasks` creates a new array reference → effect re-runs
    → cleanup closes ALL `EventSource`s → next effect iterates
    `activeTasks` and... `if (eventSourcesRef.current[task.id])
    return` skips re-open. So the close step is real (every
    EventSource gets `.close()`'d on every tasks update), but the
    re-open step is conditional. After a close+reopen cycle,
    `eventSourcesRef.current[task.id]` was deleted by the cleanup?
    No — the cleanup only calls `.close()`, it does not delete the
    entry. So the guard `if (eventSourcesRef.current[task.id])
    return` evaluates the *closed* EventSource as truthy and
    **skips re-opening**. Result: after the first `setTasks`,
    every active task's SSE is closed and never re-opened. Progress
    events stop arriving until the task completes and a new task
    starts.
- Reachability: every active task. The polling fallback (5 s,
  `:98-104`) re-fetches the task list and calls `setTasks`, which
  triggers the close-and-skip cycle. So the SSE is closed on the
  very first poll after submission. The SSE does not work in
  practice — the polling carries the load. (The SSE may never
  have worked; the close-skip pattern means a closed `EventSource`
  object remains in the ref, blocking re-subscription for the
  task's lifetime.)
- Expected invariant: an `EventSource` should remain open for the
  lifetime of the active task; closing it should happen only when
  the task transitions out of active state.
- Observed behavior: every `setTasks` closes every active
  `EventSource`. The re-open guard (`if
  (eventSourcesRef.current[task.id]) return`) treats the closed
  object as truthy and skips re-opening. Progress events are
  silently dropped after the first poll. The polling fallback
  masks the defect.
- Impact: low (the polling carries the load), but the SSE code is
  effectively dead — it opens a connection, closes it on the next
  render, and never reopens. Net cost: a transient EventSource
  allocation per poll, plus a false sense that SSE progress
  updates work.
- Root cause: the effect was written to reconcile the
  EventSource set against the active-task list, but the cleanup
  step closes-and-leaves rather than closes-and-deletes, so the
  re-open guard incorrectly short-circuits. The dependency on the
  whole `tasks` array is also too broad — the effect should depend
  only on the set of active task IDs.
- Direction: (a) make the cleanup delete from `eventSourcesRef` as
  well as close; (b) narrow the dependency to a stable key like
  `activeTaskIds.join('|')` so the effect only re-runs when the
  active set changes, not when fields like `progress` change;
  (c) optionally gate the whole SSE path behind
  `if (!isTauri())` so it only runs in the HTTP-server mode
  (Tauri desktop uses IPC, not SSE — `tasksApi` may be
  HTTP-only).
- Regression validation: a vitest test that mocks `EventSource`,
  fires two `setTasks` calls, and asserts the EventSource for an
  active task is NOT closed between calls.
- Validation reports: [V02-01](../validations/A-FE-03/V02-01.md).

### A-FE-03-P3-02: Module-scope timers/listeners in `chatStore` and `authStore` are never cleaned up (dev-only leak under HMR)

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/web-frontend/src/stores/chatStore.ts:128-132`
    — module-scope debouncer:
    ```ts
    let autoSaveTimer: ReturnType<typeof setTimeout> | undefined;
    function scheduleAutoSave() {
      clearTimeout(autoSaveTimer);
      autoSaveTimer = setTimeout(autoSave, 300);
    }
    ```
    No corresponding clear on module unload / HMR dispose.
  - `echo-agent-cli/web-frontend/src/stores/authStore.ts:107-122`
    — module-scope singleton setup:
    ```ts
    if (typeof window !== 'undefined') {
      useAuthStore.getState().initFromStorage();
      setInterval(() => { useAuthStore.getState().checkAuth(); }, 5 * 60 * 1000);
      window.addEventListener('focus', () => {
        useAuthStore.getState().checkAuth();
      });
    }
    ```
    No `removeEventListener` / `clearInterval` anywhere.
- Reachability: dev only. In production (Tauri desktop or static
  HTTP), the page is the application lifetime and these are
  intentionally page-lifetime singletons. In dev with Vite HMR,
  editing any file that re-evaluates `chatStore.ts` or
  `authStore.ts` re-runs the module top-level, registering a new
  interval/listener/timer-handle without disposing the previous
  one. The accumulators compound across HMR cycles.
- Expected invariant: module-scope side effects should either be
  idempotent (guarded by a "did-init" flag) or disposed via
  `import.meta.hot?.dispose(...)` so HMR does not accumulate
  them.
- Observed behavior: no guard, no HMR dispose. Each HMR cycle of
  `authStore.ts` adds a new 5-min `setInterval` and a new `focus`
  listener; each cycle of `chatStore.ts` leaves the previous
  `autoSaveTimer` handle (which will fire `autoSave` on the new
  store reference — usually the same module-exported singleton,
  but in some test scenarios a replaced store).
- Impact: low (dev-only). The most visible symptom is duplicate
  network calls in dev tools after editing auth-related code.
  Production users are unaffected.
- Root cause: the singleton pattern was written without
  considering HMR. The same code in a non-HMR bundler would be
  fine; Vite's HMR re-runs module top-level on dependency-graph
  changes.
- Direction: wrap the module-scope setup in an idempotency guard
  (`if (!window.__echoAuthInit) { window.__echoAuthInit = true;
  ... }`) or register Vite HMR dispose hooks
  (`if (import.meta.hot) { import.meta.hot.accept();
  import.meta.hot.dispose(() => { clearInterval(...);
  window.removeEventListener(...); }); }`). Apply to both
  `authStore` and `chatStore`.
- Regression validation: dev-only manual check (open the app in
  `npm run dev`, edit a store file, observe dev tools — the
  interval count should not grow).
- Validation reports: [V02-01](../validations/A-FE-03/V02-01.md).

### A-FE-03-P3-03: `subagentRunStore` reaches into `components/compress/` for a pure utility — layering inversion

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/web-frontend/src/stores/subagentRunStore.ts:17`
    — `import { isCanonicalUsageEvent } from
    '../components/compress/subagentUsage';`
  - `echo-agent-cli/web-frontend/src/components/compress/subagentUsage.ts`
    — the file is pure TypeScript (no JSX, no React import). It
    exports `UsageEventLike`, `SubagentUsageRun`,
    `SubagentUsageSummary`, and `isCanonicalUsageEvent` — a pure
    predicate.
  - `echo-agent-cli/web-frontend/src/components/compress/subagentUsage.test.ts`
    — co-located test file.
- Reachability: every `usage` event ingested into the subagent
  store. The import is on the live path; the file is not dead.
- Expected invariant: a Zustand store should depend only on
  other stores, types, and pure utilities — not on the component
  tree. The component tree depends on stores (one-way). A store
  importing from `components/` inverts that direction.
- Observed behavior: the import works at runtime (ES modules
  resolve cyclically and the file has no React dependency).
  Structurally, the dependency direction is wrong: a refactor
  that deletes or moves `components/compress/` must now also
  update the store, and a contributor searching for store
  dependencies will not think to look under `components/`.
- Impact: low. No runtime defect; the file is pure and the
  import is one statement. The cost is discoverability and
  modularity.
- Root cause: `isCanonicalUsageEvent` was originally written
  alongside the Compress panel (its first consumer) and placed
  in `components/compress/`. When `subagentRunStore` later needed
  the same predicate, the import was added without relocating
  the utility.
- Direction: move `subagentUsage.ts` (and its test) from
  `components/compress/` to `utils/` (or `lib/`). Update the
  two import sites (`subagentRunStore.ts:17` and
  `components/compress/CompressPanel.tsx`). The move is
  mechanical; no behavior change.
- Regression validation: `npx vitest run` (the co-located test
  should still pass after the move) and `npx tsc -b` (no broken
  imports).
- Validation reports: [V01-01](../validations/A-FE-03/V01-01.md).

### A-FE-03-P3-04: No ESLint actually runs in dev or CI — `eslint.config.js` is dead config; `eslint-plugin-jsx-a11y` not configured

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/web-frontend/eslint.config.js` — present in
    the repo, configures `@eslint/js`, `typescript-eslint`,
    `eslint-plugin-react-hooks`, `eslint-plugin-react-refresh`.
    Does NOT configure `eslint-plugin-jsx-a11y`.
  - `echo-agent-cli/web-frontend/package.json` — `devDependencies`:
    `@tailwindcss/vite`, `@types/dompurify`, `@types/node`,
    `@types/react`, `@types/react-dom`, `@vitejs/plugin-react`,
    `prettier`, `tailwindcss`, `typescript`, `vite`, `vitest`. No
    `eslint`, no `eslint-plugin-react-hooks`, no
    `eslint-plugin-react-refresh`, no `eslint-plugin-jsx-a11y`.
  - `npm ls eslint` (run in `web-frontend/`) returns
    `(empty)`. ESLint is not installed transitively either.
  - `package.json` `scripts`: `dev`, `dev:tauri`, `test`
    (`vitest run`), `build` (`tsc -b && vite build`), `build:tauri`,
    `preview`. No `lint` script. Neither `test` nor `build`
    invokes ESLint.
- Reachability: every frontend change. A contributor editing
  `.tsx`/`.ts` files gets type errors (via `tsc -b` in `build`)
  and Prettier formatting (via `npx prettier --check`), but no
  ESLint feedback. The `eslint.config.js` file misleads
  contributors into thinking lint runs — it does not.
- Expected invariant: a frontend project that ships an
  `eslint.config.js` should install ESLint as a dev dependency
  and run it in `npm test` (or a dedicated `lint` script).
  `eslint-plugin-react-hooks` would enforce `exhaustive-deps`
  (catching missing effect dependencies that cause stale
  closures) and `rules-of-hooks` (catching conditional hooks).
  `eslint-plugin-jsx-a11y` would catch the unlabeled textarea
  (A-FE-03-P2-02 supporting evidence), missing `alt`, missing
  `role` on modals, etc.
- Observed behavior: no automated check enforces any ESLint rule.
  `exhaustive-deps` violations (e.g. an effect with a stale
  closure over a value not in its dep array) compile and test
  fine. A11y violations (missing labels, missing roles) compile
  and test fine.
- Impact: medium. The missing enforcement is what allowed the
  a11y gaps in A-FE-03-P2-02 to persist (a configured
  `jsx-a11y` rule would have flagged the unlabeled textarea and
  the roleless modal). It also means future stale-closure bugs
  in effects won't be caught.
- Root cause: the project was scaffolded with `npm create vite`
  (which emits `eslint.config.js` referencing the plugins as
  peer scaffolding), but the `npm install eslint ...` step was
  never run, and no `lint` script was added. The config file
  was committed anyway.
- Direction: either (a) install ESLint + the configured plugins
  + `eslint-plugin-jsx-a11y`, add a `lint` script, and wire it
  into `npm test` and CI; or (b) delete `eslint.config.js` so
  the repo doesn't carry dead config. Option (a) is the right
  move for a project of this size; option (b) is the honest
  minimal fix.
- Regression validation: after (a), `npx eslint .` should exit
  0 on the current codebase (after fixing whatever violations
  the new rules surface — likely the unlabeled textarea and
  some `exhaustive-deps` warnings).
- Validation reports: [V04-01](../validations/A-FE-03/V04-01.md).

### A-FE-03-P3-05: Primary chat `<textarea>` has no accessibility label

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/web-frontend/src/components/chat/ChatInput.tsx:864-873`
    — the primary chat input:
    ```tsx
    <textarea
      ref={textareaRef}
      value={text}
      onChange={(e) => setText(e.target.value)}
      onKeyDown={handleKeyDown}
      onPaste={handlePaste}
      rows={1}
      placeholder="Send follow-up"
      className="..."
    />
    ```
    No `aria-label`, no `aria-labelledby`, no surrounding
    `<label>`, no `title`.
  - `echo-agent-cli/web-frontend/src/components/chat/ChatInput.tsx:855-862`
    — the adjacent hidden file `<input type="file">` also lacks
    an accessible name (it is triggered by a separate
    `<button>` with `title="Upload attachment"` — acceptable
    because the button labels the action).
- Reachability: every chat turn. The textarea is the primary
  way the user talks to the assistant.
- Expected invariant: a visible interactive element should have
  a programmatic name. WCAG 2.1 SC 4.1.2 (Name, Role, Value).
  Placeholder text is not an accessible name.
- Observed behavior: screen readers announce the field as
  "edit text, blank" (or similar) with no name. A sighted user
  sees the placeholder; a non-sighted user does not.
- Impact: low for sighted users (the placeholder is visible).
  Medium for screen-reader users (the primary interaction
  surface is unnamed). The textarea is otherwise accessible
  (it's a native element, keyboard-operable, focusable).
- Root cause: the input was built without an a11y review; the
  placeholder was mistakenly treated as a label.
- Direction: add `aria-label="发送消息"` (or wrap in a `<label>`
  with `sr-only` text). One-line change.
- Regression validation: a vitest + `@testing-library/jest-dom`
  assertion `toBeAccessibleName()` (or manual screen-reader
  smoke).
- Validation reports: [V04-01](../validations/A-FE-03/V04-01.md).

### A-FE-03-P3-06: `AppLayout` mobile sidebar overlay is not keyboard-dismissable

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/web-frontend/src/components/layout/AppLayout.tsx:26-33`
    — the overlay:
    ```tsx
    {leftSidebarOpen && (
      <div
        className="fixed inset-0 z-40 md:hidden"
        style={{ background: 'var(--bg-overlay)' }}
        onClick={toggleLeftSidebar}
      />
    )}
    ```
    Only `onClick`. No `onKeyDown` Escape handler, no close
    button on the overlay, no `role="button"`, no `tabIndex`.
- Reachability: every mobile-width session (viewport < 768 px)
  that opens the sidebar. The sidebar auto-opens on first load
  (`uiStore.ts:96` — `leftSidebarOpen: typeof window !==
  'undefined' && window.innerWidth >= 768`; defaults closed on
  mobile, but the user opens it from the toggle button).
- Expected invariant: a dismissible overlay should be
  dismissible by keyboard (Escape) and by click outside.
- Observed behavior: clicking the dark backdrop closes the
  sidebar; pressing Escape does nothing. The user must find the
  toggle button (top-left) or click the backdrop to dismiss.
- Impact: low. The sidebar does not trap focus and does not
  block the rest of the UI (it's a slide-over, not a modal).
  Keyboard users can tab past it. The gap is convenience, not
  correctness.
- Root cause: the overlay was written as a click-only scrim
  without considering keyboard dismissal.
- Direction: add an Escape handler on the overlay div, or
  delegate to the existing `useKeyboardShortcuts` hook (which
  currently handles Cmd+Shift+S to toggle the sidebar).
- Regression validation: manual keyboard smoke on a mobile-width
  viewport.
- Validation reports: [V04-01](../validations/A-FE-03/V04-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Store/component dependency map: no monolithic owner, the chatStore↔conversationStore cycle, the store→component import inversion | yes | passed (with findings) | [V01-01](../validations/A-FE-03/V01-01.md) |
| V02 | Subscription cleanup: 16 in-component sites clean up correctly; 3 module-scope sites do not; TasksPanel SSE close-skip defect | yes | passed (with findings) | [V02-01](../validations/A-FE-03/V02-01.md) |
| V03 | Render behavior for large chats: no virtualization library; MAX_MESSAGES=500 cap; MessageBubble wide-subscription hot path | yes | passed (with finding) | [V03-01](../validations/A-FE-03/V03-01.md) |
| V04 | Keyboard/focus/label + responsive smoke: InterruptPromptDialog a11y gap, CommandPalette partial, unlabeled textarea, mobile sidebar no Escape, ESLint dead config | yes | passed (with findings) | [V04-01](../validations/A-FE-03/V04-01.md) |
| V05 | Historical-document drift | conditional (applicable — three code comments treated as hypotheses; classifications inline) | passed | classified inline in Historical Claim Status |

Executed command (exit 0):

```text
cd echo-agent-cli/web-frontend
npx vitest run --reporter=dot
  Test Files  26 passed (26)
  Tests       101 passed (101)
  Duration    5.10s
npx prettier --check "src/**/*.{ts,tsx}"
  All matched files use Prettier code style!
```

No `cargo` command was required: this is a frontend-only review.
No code was modified; the vitest run used the existing incremental
cache.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `chatStore.ts:102-112` — `MAX_MESSAGES=500` "prevents OOM on very long conversations (P0-4)" | partially overstated | The cap holds (`trimToMax` applied to every grow path, V03). But the cap is in-memory only — every one of the 500 bubbles still renders in the DOM (no virtualization, V03). The OOM claim is true for the array; the rendering-budget claim is understated. |
| `MessageBubble.tsx:153` `memo` wrapper — assumed to bound re-renders | falsified for streaming | `memo` blocks parent-driven re-renders only. `MessageBubble`'s internal Zustand subscriptions (`messages`, `runs`, `activeRun`) bypass `memo` and re-render every bubble on every token (A-FE-03-P2-01). |
| `A-SRF-03-P3-04` (`useBrowserEvents` listener race) | current (load-bearing) | This task's V02 confirms the in-component listener cleanup inventory is otherwise clean; the one divergent sibling pattern A-SRF-03 flagged is still the only listener-race finding. |
| `A-FE-02 V03` (bounded event logs + 256 KiB live cap) | current (load-bearing) | This task's V03 reuses those bounds as the rendering-budget assumptions. |
| `eslint.config.js` (presence implies enforcement) | stale | The config exists but ESLint is not installed, no `lint` script exists, and neither `npm test` nor `npm run build` invokes it (A-FE-03-P3-04). |

## Coverage And Uncertainty

Inspected in full: every non-test Zustand store (13), the root
assembly (`App.tsx`, `main.tsx`, `AppLayout.tsx`), the chat
rendering hot path (`ChatPanel`, `MessageBubble`, `InlineToolCall`,
`MarkdownContent`), the three modals (`SettingsDialog`,
`CommandPalette`, `InterruptPromptDialog`), `useKeyboardShortcuts`,
`useTauriChat` (head — already audited in A-SRF-03),
`useBrowserEvents` (head — A-SRF-03), the listener/timer inventory
across the whole tree, `Toggle`, `ErrorBoundary`, `TerminalDrawer`,
`RightWorkspace` (head), `ChatInput` (listener cleanup + textarea),
and the tooling baseline (`package.json`, tsconfigs,
`eslint.config.js`). Whole test suite executed (26 files, 101
tests, exit 0). Prettier check clean.

Not inspected (out of scope or deferred):

- The 17 panel components outside the chat/task path
  (`ReviewWorkbench`, `EvolutionPanel`, `PluginPanel`,
  `AnalysisPanel`, `ObservabilityPanel`, `ProviderPanel`,
  `SchedulerPanel`, `NewTaskDialog`, `SkillsPanel`, `PaperPanel`,
  `MemoryPanel`, `ConfigPanel`, `McpManagerPanel`, `PaperDetail`,
  `WorkflowDebugger`, `SandboxPanel`, `CompressPanel`). They have
  their own local state and component-local effects; their internal
  a11y / perf is not audited. **Two are over 500 lines**
  (`ReviewWorkbench.tsx` 1339 lines, `EvolutionPanel.tsx` 1133
  lines) — they may be monolithic, but the task scope is the chat /
  task hot path; a separate panel-architecture audit would cover
  them.
- The terminal renderer internals (`Terminal.tsx` beyond the
  listener setup at `:112-125`) — terminal events flow into
  xterm.js directly (no Zustand), so the chat-surface analysis
  does not apply.
- E2E / Playwright coverage (none found in `package.json`).
- Profiling of the streaming-token hot path (A-FE-03-P2-01). The
  O(N·T) claim is from static reasoning; a React Profiler trace
  on a 500-message conversation would confirm or refute the
  severity.

Environmental constraints:

- Read-only static review against `echo-agent-cli` commit
  `b3b2e81`. No code was modified. The vitest run used the
  existing incremental cache.
- Browser-based a11y testing (axe-core, Lighthouse, screen-reader
  smoke) was not performed; the a11y findings are from static
  inspection of the JSX.

Uncertain claims:

- Whether A-FE-03-P2-01's perf impact is user-visible. The static
  reasoning says O(N·T); a profile would say how many ms per
  token at N=500. Without that, the P2 rating is conservative.
- Whether the TasksPanel SSE was ever working
  (A-FE-03-P3-01). The close-skip pattern suggests it has been
  silently broken since it was added; `git log` / `git blame`
  would confirm. The polling fallback masks the defect either
  way.

## Handoff

Conclusions downstream tasks may rely on:

1. **There is no monolithic accidental state owner.** The 13 stores
   are organized around stable domain facts; the largest
   (`subagentRunStore` 536 lines, `chatStore` 527 lines) owns a
   single coherent concern. Downstream tasks auditing a specific
   feature can trust the store boundaries.
2. **There is one cycle: `chatStore ↔ conversationStore`.** It is
   structurally safe but architecturally smelly. Any task touching
   auto-save or conversation switching should be aware of it
   (A-FE-03-P2-03).
3. **In-component listener/timer cleanup is uniformly correct.**
   The 16 audited sites all return proper cleanup functions. The
   race-safe pattern lives in `useTauriChat` (A-SRF-03-P3-04
   notes `useBrowserEvents` should adopt it). The only cleanup-
   related defects are: the module-scope singletons
   (A-FE-03-P3-02, dev-only) and the TasksPanel SSE close-skip
   (A-FE-03-P3-01).
4. **Large-chat rendering is bounded by `MAX_MESSAGES=500` but
   not virtualized.** The hot path is the `MessageBubble` wide
   subscription to `chatStore.messages`
   (A-FE-03-P2-01). Downstream perf work should start there.
5. **Accessibility is partial.** The chat log has good
   `aria-live` coverage; the primary modal gap is
   `InterruptPromptDialog` (A-FE-03-P2-02). No a11y or
   react-hooks lint is enforced (A-FE-03-P3-04), so future a11y
   regressions won't be caught automatically.
6. **The frontend has no virtualization library and no ESLint
   running.** Both are tooling gaps that future architecture
   work should address.

Reports downstream tasks must read:

- This report (A-FE-03) for the store dependency map, the
  cleanup inventory, the rendering hot path, and the a11y smoke.
- `tasks/A-SRF-03.md` for the chat-transport receive-side
  contract and the `useBrowserEvents` listener race
  (A-SRF-03-P3-04 — the one in-component listener hardening
  gap).
- `tasks/A-FE-02.md` for the bounded event logs and the lazy
  tool-output rendering (the rendering-budget assumptions
  reused in V03).
- `tasks/A-FE-01.md` for the IPC type contract (relevant to V04:
  no contract test means no a11y-or-typesafety regression
  protection either).

Conditions that make this report stale:

- Lifting `lastAssistantMessageId` / `messageIds` out of
  `MessageBubble` (resolving A-FE-03-P2-01) invalidates the
  wide-subscription finding.
- Adding a shared `Modal` primitive and migrating the three
  modals (resolving A-FE-03-P2-02) invalidates the modal a11y
  findings.
- Breaking the `chatStore ↔ conversationStore` cycle (resolving
  A-FE-03-P2-03) invalidates the cycle finding.
- Fixing the TasksPanel SSE cleanup (resolving A-FE-03-P3-01)
  invalidates the close-skip finding.
- Adding idempotency guards / HMR dispose hooks to `chatStore`
  and `authStore` (resolving A-FE-03-P3-02) invalidates the
  dev-only leak finding.
- Moving `subagentUsage.ts` to `utils/` (resolving
  A-FE-03-P3-03) invalidates the layering inversion finding.
- Installing ESLint + plugins and wiring into `npm test`
  (resolving A-FE-03-P3-04) invalidates the dead-config finding.
- Adding `aria-label` to the chat textarea (resolving
  A-FE-03-P3-05) invalidates the unlabeled-textarea finding.
- Adding Escape handling to the mobile sidebar overlay
  (resolving A-FE-03-P3-06) invalidates the keyboard-dismiss
  finding.
- Installing a virtualization library (e.g. `@tanstack/react-
  virtual`) changes V03's "no virtualization" claim.

Follow-up task IDs (no fixes implemented in this review):

- A **`MessageBubble` subscription narrowing** task — resolve
  A-FE-03-P2-01 by lifting `lastAssistantMessageId` /
  `messageIds` into `ChatPanel` and passing as props. The
  single highest-impact perf fix in the frontend.
- A **shared modal primitive** task — resolve A-FE-03-P2-02 by
  extracting a `Dialog` component with role / focus-trap /
  Escape / autofocus and migrating all three modals. Pair with
  a vitest + `@testing-library/user-event` regression suite.
- A **store cycle refactor** task — resolve A-FE-03-P2-03 by
  inverting the auto-save dependency (event/observer from
  chatStore → conversationStore) or extracting a coordinator.
- A **TasksPanel SSE fix** task — resolve A-FE-03-P3-01 by
  making cleanup delete from `eventSourcesRef` and narrowing
  the effect dependency to a stable active-task-id key.
- A **dev-HMR hygiene** task — resolve A-FE-03-P3-02 by adding
  idempotency guards or `import.meta.hot.dispose` hooks to
  `chatStore` and `authStore` module-scope setup.
- A **utility relocation** task — resolve A-FE-03-P3-03 by
  moving `components/compress/subagentUsage.ts` (and its test)
  to `utils/`. Mechanical.
- A **lint wiring** task — resolve A-FE-03-P3-04 by installing
  ESLint + the configured plugins + `eslint-plugin-jsx-a11y`,
  adding a `lint` script, and wiring into `npm test` and CI.
- A **textarea label + sidebar Escape** task — resolve
  A-FE-03-P3-05 and A-FE-03-P3-06. Small, one-line-per-site.
