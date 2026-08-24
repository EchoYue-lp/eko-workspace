# A-FE-03: Frontend architecture, performance, and accessibility

> Status: complete
> Reviewer: Codex primary reviewer
> Executor: Codex primary reviewer
> Review date: 2026-08-13
> `echo-agent` commit: 3aa7929928442aab91e4dce9c426d909a5f0a1ab
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: CLI clean; framework live worktree excluded except fixed committed anchors

## Question

Are frontend components and stores organized around stable domain facts, with
bounded rendering, correct listener/timer ownership, accessible interactions,
and no accidental monolithic state owner?

## Scope

- Root/layout assembly and frontend files over 500 lines.
- Chat, TaskRuntime, Subagent, and tool subscription/render paths.
- Module/component timers and browser-event cleanup.
- Command palette, settings, task creation, and interrupt modal semantics,
  keyboard/focus behavior, labels, and narrow-screen constraints.
- Adjacent static tests and dependency inventory.

## Out Of Scope

- Rust/TypeScript DTO correctness owned by `A-FE-01`.
- Task/Subagent/tool lifecycle and artifact semantics owned by `A-FE-02` and
  `A-SRF-03`.
- Backend task limits, execution correctness, and output completeness.
- Source fixes, browser automation, performance measurements, builds, tests,
  fixtures, or network access.

## Inputs

- Root `AGENTS.md`, review protocol, exact `A-FE-03` task card, and Codex rules.
- Accepted Codex dependencies `A-SRF-03`, `A-FE-01`, and `A-FE-02`.
- Clean CLI source at the fixed commit and the framework's committed plan limit.

## Layering Decision

Frontend stores remain EKO projections, not new authorities. Framework event and
Task contracts should be indexed once at the frontend adapter boundary; EKO
selectors may derive view models but must not rescan the complete graph inside
every rendered row. Rendering window policy, modal focus management, responsive
layout, and browser lifecycle ownership belong to the application. Fixes should
extend the current stores/hooks and shared overlay primitive rather than add a
second chat, Task, Subagent, or authentication store.

## Positive Conclusions

- Chat memory is capped at 500 messages and conversation auto-save is debounced,
  so the inspected hot path is bounded even though its bound is still expensive.
- ChatPanel uses individual Zustand selectors, canonical TaskRuntime polling
  prevents overlapping refreshes, and most inspected component intervals,
  EventSources, and browser listeners have paired cleanup.
- Settings already has a full-screen small-device layout; tool detail output is
  paged and only auto-loads while explicitly expanded.
- The frontend has one TaskRuntime store, one Subagent run store, and one tool
  execution store; this task found inefficient projections, not a second domain
  authority.

## Findings

### A-FE-03-P1-01: Every chat update causes all message rows to rescan global chat and Subagent state

- Priority: P1
- Confidence: high
- Layer: frontend rendering
- Evidence: `echo-agent-cli/web-frontend/src/components/chat/ChatPanel.tsx:177`; `echo-agent-cli/web-frontend/src/components/chat/MessageBubble.tsx:185`; `echo-agent-cli/web-frontend/src/components/chat/MessageBubble.tsx:188`; `echo-agent-cli/web-frontend/src/components/chat/MessageBubble.tsx:195`; `echo-agent-cli/web-frontend/src/components/chat/MessageBubble.tsx:198`; `echo-agent-cli/web-frontend/src/components/chat/ParallelExecutionBlock.tsx:68`; `echo-agent-cli/web-frontend/src/stores/chatStore.ts:241`
- Reachability: every streamed token clones the messages array -> each mounted MessageBubble subscribes to that whole array and the whole Subagent map -> its nested ParallelExecutionBlock subscribes and scans them again -> ChatPanel retains and maps all messages.
- Expected invariant: one hot token update touches the active message and a bounded visible window; conversation-wide message IDs and Subagent associations are indexed once per store update.
- Observed behavior: up to 500 rendered messages each rebuild a message-ID set, search for the last assistant, convert/filter/sort all Subagent runs, and repeat much of that work in the nested block. There is no virtualization dependency or windowing path. Memoization cannot suppress store-triggered rerenders.
- Impact: streaming latency and CPU scale approximately as `messages * (messages + subagent_runs)`; long coding sessions can visibly stall before reaching the explicit 500-message cap, especially with Markdown and 4Hz running-tool clocks mounted in history.
- Root cause: global association facts are derived independently inside every row instead of being normalized/indexed at the store or list boundary.
- Direction: build stable `messageId -> current runs` and latest-assistant selectors once, subscribe rows to their own IDs, virtualize/window the transcript while preserving scroll anchoring and accessible log semantics, and use one shared elapsed-time clock for visible running rows.
- Regression validation: 500-message/100-Subagent streaming fixture with render-count and frame-time budgets; update one token and assert only the active/affected visible rows render.
- Validation reports: [V02](../validations/A-FE-03/V02-01.md), [V06](../validations/A-FE-03/V06-01.md)

### A-FE-03-P2-02: TaskRuntime repeatedly filters and sorts all traces for every plan node

- Priority: P2
- Confidence: high
- Layer: frontend projection
- Evidence: `echo-agent-cli/web-frontend/src/components/task/TaskRuntimePanel.tsx:429`; `echo-agent-cli/web-frontend/src/components/task/TaskRuntimePanel.tsx:518`; `echo-agent-cli/web-frontend/src/components/task/TaskRuntimePanel.tsx:549`; `echo-agent-cli/web-frontend/src/components/task/TaskRuntimePanel.tsx:561`; `echo-agent-cli/web-frontend/src/components/task/TaskRuntimePanel.tsx:701`; `echo-agent/echo-orchestration/src/planning/validator.rs:34`
- Reachability: any Subagent event replaces the subscribed run map -> the panel derives all active-run traces -> completion counters and every one of up to 100 validated plan tasks call `traceRunForTodo` repeatedly.
- Expected invariant: one store update indexes the latest trace by task in linear time, and each row performs constant-time lookup; large lists render a bounded window.
- Observed behavior: `traceRunForTodo` filters and sorts the whole run list. It is called in the completion pass and multiple times per rendered todo, while each row also linearly searches plan tasks. No task-list windowing exists.
- Impact: active large DAGs amplify every trace event into repeated `O(tasks * traces log traces)` work and mount all rows, competing with chat streaming in the same WebView.
- Root cause: a row helper performs collection-wide derivation and is composed multiple times without a shared keyed projection.
- Direction: derive `taskId -> latestTrace` and `taskId -> PlanTask` maps once with deterministic identity/attempt precedence, compute counts in the same pass, and window long task lists without changing canonical Task status authority.
- Regression validation: maximum-size 100-task plan with multiple attempts; ingest one trace event and enforce bounded selector/render counts while preserving status results.
- Validation reports: [V03](../validations/A-FE-03/V03-01.md), [V06](../validations/A-FE-03/V06-01.md)

### A-FE-03-P2-03: Authentication polling has two owners and one unremovable module lifetime

- Priority: P2
- Confidence: high
- Layer: frontend lifecycle
- Evidence: `echo-agent-cli/web-frontend/src/stores/authStore.ts:106`; `echo-agent-cli/web-frontend/src/stores/authStore.ts:111`; `echo-agent-cli/web-frontend/src/stores/authStore.ts:119`; `echo-agent-cli/web-frontend/src/components/Auth/RequireAuth.tsx:17`; `echo-agent-cli/web-frontend/src/components/Auth/RequireAuth.tsx:21`
- Reachability: importing the store immediately installs a five-minute interval and anonymous focus listener; mounted RequireAuth independently polls every minute.
- Expected invariant: one application lifecycle owner installs authentication refresh, coalesces focus/periodic checks, and disposes or survives hot reload intentionally through an idempotent singleton contract.
- Observed behavior: the module-level timer/listener have no handles or removal path, while RequireAuth installs a second timer. Module re-evaluation can accumulate anonymous listeners, and normal runtime performs redundant checks even though authentication is disabled by default.
- Impact: duplicate backend calls, non-deterministic test/HMR behavior, and leaked browser callbacks obscure which owner controls login state.
- Root cause: store initialization and component lifecycle both own the same side effect.
- Direction: expose one idempotent `start/stop` authentication monitor owned by root composition, retain listener/timer handles, coalesce in-flight checks, and delete the second polling path.
- Regression validation: mount/unmount and simulated module reload/focus tests; assert one listener, one timer, at most one in-flight request, and complete disposal.
- Validation reports: [V04](../validations/A-FE-03/V04-01.md), [V06](../validations/A-FE-03/V06-01.md)

### A-FE-03-P1-04: Core overlays are visually modal but not semantic or focus-modal, and task creation overflows narrow screens

- Priority: P1
- Confidence: high
- Layer: frontend interaction/accessibility
- Evidence: `echo-agent-cli/web-frontend/src/components/common/CommandPalette.tsx:30`; `echo-agent-cli/web-frontend/src/components/common/CommandPalette.tsx:43`; `echo-agent-cli/web-frontend/src/components/common/CommandPalette.tsx:76`; `echo-agent-cli/web-frontend/src/components/layout/SettingsDialog.tsx:332`; `echo-agent-cli/web-frontend/src/components/layout/SettingsDialog.tsx:345`; `echo-agent-cli/web-frontend/src/components/workspace/NewTaskDialog.tsx:149`; `echo-agent-cli/web-frontend/src/components/workspace/NewTaskDialog.tsx:156`; `echo-agent-cli/web-frontend/src/components/task/TaskRuntimePanel.tsx:857`
- Reachability: command palette, settings, workspace/task creation, and interrupted Task handling are ordinary top-level GUI workflows.
- Expected invariant: each overlay exposes a named `dialog`/`aria-modal`, moves focus inside, traps Tab, restores the opener, labels icon-only controls, and remains inside every supported viewport.
- Observed behavior: none of the four inspected overlays declares dialog semantics or implements a focus trap/restore. Command keyboard handling is attached only to its input; settings only closes globally on Escape; task creation and interrupt do neither. Several close buttons are unlabeled. NewTaskDialog fixes width at 540px with no viewport maximum, so it overflows narrow screens, while background controls remain keyboard reachable.
- Impact: screen-reader and keyboard users can lose context or interact behind a destructive/decision modal; narrow desktop/mobile WebViews can hide task-creation content and controls.
- Root cause: each feature hand-rolls a visual overlay instead of sharing an accessible application dialog primitive.
- Direction: introduce one tested dialog/focus-scope primitive with semantic title/description, inert background, Escape policy, initial/return focus and icon labels; use responsive width constraints and migrate all four overlays.
- Regression validation: keyboard-only and accessibility-tree scenarios for open/Tab/Shift-Tab/Escape/close/return focus plus 320px and zoomed layouts.
- Validation reports: [V05](../validations/A-FE-03/V05-01.md), [V06](../validations/A-FE-03/V06-01.md)

## Validation Matrix

| ID | Claim | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Inputs, commits, isolation, and exact scope | yes | passed | [V00](../validations/A-FE-03/V00-01.md) |
| V01 | Store/component dependency and oversized-file map | yes | passed/inventory | [V01](../validations/A-FE-03/V01-01.md) |
| V02 | Long-chat subscription and render-cost trace | yes | failed/finding | [V02](../validations/A-FE-03/V02-01.md) |
| V03 | Large Task graph selector and render-cost trace | yes | failed/finding | [V03](../validations/A-FE-03/V03-01.md) |
| V04 | Listener, timer, polling, and cleanup inventory | yes | failed/finding | [V04](../validations/A-FE-03/V04-01.md) |
| V05 | Keyboard, focus, label, and responsive inspection | yes | failed/finding | [V05](../validations/A-FE-03/V05-01.md) |
| V06 | Existing test and performance/a11y gap inventory | yes | failed/gaps | [V06](../validations/A-FE-03/V06-01.md) |
| V07 | Dependency ownership and semantic de-duplication | yes | passed | [V07](../validations/A-FE-03/V07-01.md) |
| V08 | Dynamic render/a11y/responsive scenarios | future | not_run | [V08](../validations/A-FE-03/V08-01.md) |
| V09 | Exact-link/header/source-isolation integrity | yes | V09-01/02 failed harnesses; V09-03 passed | [V09](../validations/A-FE-03/V09-03.md) |

## Coverage And Uncertainty

- The review is source-conclusive and intentionally static. No Vitest, browser,
  accessibility scanner, profiler, build, fixture, or network command ran.
- Complexity statements follow directly from collection-wide operations nested
  in per-row render paths; actual device frame times remain future evidence.
- The explicit 500-message and 100-task limits bound worst-case sizes but do not
  make repeated quadratic work acceptable on a streaming hot path.
- Large domain-specific panels were inventoried by size, but line count alone
  was not treated as a defect. No duplicate domain store was inferred merely
  from a large component.
- A-FE-02 remains the owner of Subagent result retention and Task acceptance;
  A-SRF-03 owns terminal merge semantics. This task owns rendering/lifecycle/UI
  mechanics only.

## Handoff

Fix hot-path indexing and row subscriptions first, then establish the shared
dialog/focus primitive and single auth monitor. Keep the current domain stores
and typed identities; optimize their projections rather than introducing new
frontend authorities. Add measurable render-count/frame-time and accessibility
regressions during implementation.
