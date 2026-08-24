# A-SRF-01: TUI integration

> Status: complete
> Reviewer: Codex primary reviewer (delegated evidence independently sampled)
> Executor: Codex review subagent
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: both source repositories clean before report creation; only
> Codex A-SRF-01 reports were added

## Question

Does the TUI expose and correctly render the complete EKO Agent feature set
rather than a reduced execution path?

## Scope

- Default TUI binary entry, composition arguments, `TuiApp`, command catalog,
  event loop, `TuiChatSink`, tool/task/Subagent/HITL reducers, sidebar, and
  conversation recovery under `echo-agent-cli/src/tui`.
- TUI-facing app-core services needed to prove real reachability: shared chat
  resources/driver, TaskRuntime file authority, HITL provider, attachments,
  pool, browser, scheduler, plugin, memory/review, and surface contract tests.
- Peer GUI code only where required for a concrete parity comparison: canonical
  task revision commands, durable TaskRuntime hydration, and full Chart event.

## Out Of Scope

- Shared chat terminal semantics, pre-stream failure delivery, committed
  FinalAnswer handling, queued-turn settlement, and sink backpressure are owned
  by [A-CHAT-01](A-CHAT-01.md). This report does not duplicate those findings.
- HITL provider concurrency, pending-request identity, timeout/cancel/default,
  permission parity, and MCP over-gating are owned by
  [A-HITL-01](A-HITL-01.md).
- Prepared-turn attachment identity/cleanup and live steer normalization are
  owned by [A-INP-01](A-INP-01.md); startup/root consistency is A-BOOT-01.
- `RuntimeDagExecutor` ownership, scheduling, retry/cancel/stall semantics belong
  to A-TSK-03. Its required Codex report does not yet exist, so this review does
  not infer or duplicate executor conclusions.
- Source fixes, roadmap design, Cargo/rustc/tests/builds/dynamic fixtures, and
  network activity.

## Inputs

- Root `AGENTS.md`; shared review `README.md`, `REPORTING.md`, exact A-SRF-01
  card in `TASKS.md`; Codex isolation protocol and report templates.
- Codex dependency reports [A-CHAT-01](A-CHAT-01.md) and
  [A-HITL-01](A-HITL-01.md). A-TSK-03 was searched by exact ID but is not yet
  available. [A-TSK-02](A-TSK-02.md) was used only to locate the already
  accepted canonical task-authoring adapter; current source was independently
  traced for all A-SRF conclusions.
- Current clean source. No other reviewer directory was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Agent events, Subagent event identities, Task revision/DAG primitives, and durable framework/application stores stay in their existing owners. TUI defects do not justify adding product UI policy to `echo-agent`. |
| EKO product policy | TUI command UX, textual renderers, restart hydration, direct attended task controls, and parity with GUI/CLI/channel belong in `echo-agent-cli`. |
| Adapter boundary | `TuiChatSink` and Task/Subagent projections should convert shared facts thinly and losslessly. They must not truncate canonical payloads, invent a second graph API, or use an ephemeral feed when a durable EKO authority exists. |
| Duplicate search | Searched both repositories for TUI/GUI task commands, revision updates, TaskRuntime events, Subagent buses, chart mapping, HITL, attachment, browser, MCP, cron, plugin, memory, artifact, and test contracts. The core capabilities exist; three TUI adapter gaps remain. |
| Migration deletion | Reuse `apply_eko_task_update`, TaskRuntime `list_events`/artifact APIs, and the full Chart payload. Delete the TUI-only chart preview downgrade, ephemeral-only TaskRuntime/Subagent projection assumptions, and prose-only parity assertions after cutover. |

## Current Path

```text
default echo-agent-cli
  -> AgentRuntime::bootstrap
  -> file TaskRuntimeStore + task tools + shared AgentPool/task_execute
  -> TuiHumanLoopProvider + shared headless services
  -> run_tui(all service handles)
     -> TuiApp
        -> normal/queued input -> PreparedUserTurn -> ChatResources -> drive_chat
        -> TuiChatSink -> local AgentEvent -> reducer/widgets
        -> SubagentEventBus live subscription -> SubagentRuntimeView
        -> 250 ms TaskRuntime latest-run/get-plan projection
        -> slash commands -> live conversation/MCP/browser/plugin/cron/services
```

This is not a deliberately reduced product. The production entry constructs and
passes the same classes of core service required by GUI: TaskRuntime, Subagents,
HITL, memory/review, attachments, MCP/plugins, browser, scheduler, persistence,
webhook, and diagnostics ([V01](../validations/A-SRF-01/V01-01.md),
[V02](../validations/A-SRF-01/V02-01.md),
[V06](../validations/A-SRF-01/V06-01.md)). The gaps are specifically in surface
controls and lossless/recoverable projection.

### Capability and reducer matrix

| Capability | TUI path | Static result |
|---|---|---|
| Chat/Auto/Task | `/mode` -> PreparedUserTurn -> shared `drive_chat` | live; shared lifecycle defects deferred to A-CHAT |
| Plan/TaskRun | `/tasks`, cancel/pause/resume/recovery/retry/skip; store projection | execution/control live; attended graph edit missing |
| Subagent | framework event-bus subscription + sidebar/result summary | live-only; no durable hydration/lag repair |
| Tools | structured tool call/stream/result reducer + artifact opener | live; existing bounded Unicode handling is safe |
| HITL | TUI provider + priority approval/input/selection card | live; provider defects deferred to A-HITL |
| Resume | file conversation store + framework message restore | chat live; task evidence incomplete after resume |
| Attachments | `/attach` -> PreparedUserTurn -> ChatResources | live; lifecycle issues deferred to A-INP |
| Browser/MCP/plugins/skills | slash commands -> live shared services | live |
| Cron/background | SchedulerRunner commands and shared pool | live |
| Chart | sink -> 500-character Notice | lossy versus GUI full typed spec |

## Findings

### A-SRF-01-P1-01: TUI cannot perform the attended Task graph edits available in GUI

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tui/commands.rs:89`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tui/events.rs:4216`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tui/events.rs:4305`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/web-frontend/src/components/task/TaskRuntimePanel.tsx:745`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/web-frontend/src/stores/taskRuntimeStore.ts:306`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/task_runtime.rs:399`
- Reachability: every TUI TaskRun reaches the read projection and run controls,
  while GUI pending/blocked tasks reach `update_tasks` -> the canonical EKO
  `TaskRevisionService`. Repository search finds no TUI call to that adapter.
- Expected invariant: TUI and GUI are complete Agent surfaces. Both let a user
  inspect and revise the same pending/blocked revisioned graph, differing only
  in interaction/rendering.
- Observed behavior: TUI can list a plan and cancel/pause/resume/recover/retry/
  skip a run, but it cannot directly edit/insert/reorder a PlanTask. GUI exposes
  those attended operations. Asking the Agent to invoke task tools is not
  equivalent during execution because ordinary TUI input is rejected while the
  active turn is busy.
- Impact: a TUI user cannot correct a generated plan title, dependency/order,
  role/tool choice, or insert follow-up work before execution with the control
  available to GUI users, violating mandatory surface parity.
- Root cause: TaskRuntime read/run controls were added to TUI, but the already
  canonical revision mutation adapter was exposed only through Tauri/frontend.
- Direction: add compact TUI edit/insert/reorder/skip commands that call the
  same EKO revision service with current revision/CAS and typed errors. Do not
  create TUI-local Plan CRUD or another state machine. Delete no framework API;
  delete any duplicate command parsing if a shared application request parser
  is introduced.
- Regression validation: apply the same edit/insert/reorder sequence from TUI
  and GUI and assert identical next revision, events, validation failures, and
  stale-revision handling.
- Validation reports: [V04](../validations/A-SRF-01/V04-01.md)

### A-SRF-01-P1-02: TUI Task/Subagent/tool projection cannot reconstruct durable runtime evidence after restart or lag

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tui/events.rs:562`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tui/events.rs:570`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tui/events.rs:4818`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tui/events.rs:4920`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/src/agent/subagent/events.rs:10`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:1790`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/web-frontend/src/stores/taskRuntimeStore.ts:219`
- Reachability: production TUI subscribes once to the framework's 128-entry
  broadcast and refreshes TaskRuntime every 250 ms. `/resume` and startup reload
  conversations, while the file store already exposes durable events and
  artifacts for the selected run.
- Expected invariant: selecting/resuming a conversation rebuilds its latest
  run, PlanTasks, Subagent attempts/outcomes, tool terminal details, and artifact
  references from durable authority before live updates; lag triggers explicit
  resynchronization.
- Observed behavior: TUI only reloads latest run + plan and clears
  `subagent_runs`; stored tool messages lose tool identity. It never consumes
  TaskRuntime `list_events`/artifacts to hydrate Task/Subagent/tool evidence.
  `while let Ok` silently stops draining on `Lagged`, leaving no visible warning
  or replay. GUI already performs durable sequence-zero event/artifact/tool
  hydration.
- Impact: after restart, conversation resume, or a burst exceeding broadcast
  capacity, TUI can show a final task status while omitting or retaining stale
  Subagent/tool outcomes and evidence. The user cannot reliably audit what ran,
  failed, or produced an artifact.
- Root cause: TUI combines a durable high-level plan poller with an ephemeral
  framework event feed, while GUI treats the TaskRuntime event log as the
  application projection authority.
- Direction: build the TUI projection from TaskRuntime run/plan/todo/event/
  artifact APIs, track the durable sequence, then overlay live events
  idempotently. On broadcast lag, emit a typed notice and resync. Remove
  ephemeral-only `subagent_runs` authority and generic restored tool labels
  after cutover; retain the framework bus for non-Task inline activity.
- Regression validation: restart and `/resume` during/after parallel Subagents,
  tool output, failure, cancel, and retry; inject broadcast lag; assert parity
  with the durable log and GUI without duplicate attempts.
- Validation reports: [V05](../validations/A-SRF-01/V05-01.md)

### A-SRF-01-P1-03: TUI irreversibly truncates canonical Chart payloads

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tui/events.rs:2184`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/chat.rs:1540`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/web-frontend/src/hooks/chatEventHandler.ts:92`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/web-frontend/src/components/chat/ChartCard.tsx:89`
- Reachability: every framework Chart event in a TUI chat passes through
  `TuiChatSink`; the peer GUI path passes the same event to a ChartCard.
- Expected invariant: a text surface may show a summary, JSON pager, or artifact
  link, but the user must be able to inspect/recover the complete chart spec.
- Observed behavior: TUI serializes the JSON, keeps only the first 500 Unicode
  characters, and stores it as a generic Notice. It creates neither a full
  message payload nor an artifact reference. GUI preserves and renders the full
  typed spec.
- Impact: valid larger charts lose series, labels, provenance, or options and
  cannot be reproduced from the TUI transcript. The capability is materially
  different across TUI and GUI.
- Root cause: the sink treats Chart as a cosmetic notification instead of a
  canonical output artifact/event.
- Direction: preserve the typed full spec in TUI state and render a concise
  preview with an expandable pager or durable JSON artifact. Delete the
  500-character Notice downgrade. Apply the same full-artifact rule to CLI/
  channel under A-SRF-04/X-SRF-01 rather than lowering TUI to their current
  truncation.
- Regression validation: small/large/Unicode chart specs, repeated charts, and
  transcript restore; assert exact payload equality and an inspectable terminal
  representation on every surface.
- Validation reports: [V03](../validations/A-SRF-01/V03-01.md)

### A-SRF-01-P2-04: The product capability-matrix test validates prose, not reachability or reducer behavior

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/surface_contract.rs:23`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/surface_contract.rs:28`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/surface_contract.rs:141`
- Reachability: the module is compiled only for tests from app-core `lib.rs`; its
  single matrix assertion runs over 11 rows of manually written string evidence.
- Expected invariant: a parity contract fails when a capability handler,
  service binding, semantic field, or terminal reducer is missing.
- Observed behavior: the test checks only row count, fixed array length, and
  non-empty strings. It still passes if `/attach`, TaskRuntime wiring, Chart
  payload preservation, or an entire reducer branch is removed.
- Impact: maintainers receive a green named “capability matrix” while direct
  attended task editing, durable projection, and Chart parity are absent. This
  gives false regression confidence across the highest-risk product invariant.
- Root cause: architecture notes were encoded as data without executable links
  to command definitions, service constructors, or reducer fixtures.
- Direction: replace or supplement prose rows with shared request/event fixture
  tables invoked by each adapter, plus compile-visible command/service mapping.
  Keep explicit N/A rows only where product semantics genuinely make them
  inapplicable. Delete the prose-only assertion once executable coverage owns
  the contract.
- Regression validation: deliberately remove a handler/field in a mutation test
  or compile fixture and prove the parity gate fails.
- Validation reports: [V07](../validations/A-SRF-01/V07-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | TUI capability definition/duplicate inventory | yes | passed | [V01](../validations/A-SRF-01/V01-01.md) |
| V02 | Production entry/composition/runtime reachability | yes | passed | [V02](../validations/A-SRF-01/V02-01.md) |
| V03 | Canonical Agent event field/reducer matrix | yes | failed: Chart payload loss | [V03](../validations/A-SRF-01/V03-01.md) |
| V04 | Task/Subagent/tool/HITL surface flow and attended parity | yes | failed: task edit gap | [V04](../validations/A-SRF-01/V04-01.md) |
| V05 | Resume/restart/lag durable projection | yes | failed | [V05](../validations/A-SRF-01/V05-01.md) |
| V06 | Resume/attachment/browser/MCP/plugin/cron reachability | yes | passed with dependency backlinks | [V06](../validations/A-SRF-01/V06-01.md) |
| V07 | Existing test and capability-contract coverage | yes | passed with gaps | [V07](../validations/A-SRF-01/V07-01.md) |
| V08 | Executable terminal/restart/parity fixtures | future | not run by rule | [V08](../validations/A-SRF-01/V08-01.md) |
| V09 | Dependency/deduplication gate | yes | passed with dependency gap | [V09](../validations/A-SRF-01/V09-01.md) |
| V10 | Task/Subagent/tool/HITL definition-to-reducer flows | yes | passed | [V10](../validations/A-SRF-01/V10-01.md) |
| V11 | Exact-ID/header/link/isolation/source-clean gate | yes | attempt 1 inconclusive; attempt 2 passed | [A1](../validations/A-SRF-01/V11-01.md), [A2](../validations/A-SRF-01/V11-02.md) |
| V30 | Primary source sampling and dependency acceptance | yes | passed | [V30](../validations/A-SRF-01/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `src/main.rs:172-174` every headless surface is a full Agent surface | current as composition intent, incomplete as product behavior | [V02](../validations/A-SRF-01/V02-01.md), findings P1-01 through P1-03 |
| `src/tui/events.rs:2019-2020` TuiChatSink is equivalent to GUI event mapping | regressed/incomplete | [V03](../validations/A-SRF-01/V03-01.md) |
| `app-core/src/surface_contract.rs` claims evidence for every surface | incomplete/misleading validation | [V07](../validations/A-SRF-01/V07-01.md) |
| Historical “TUI does not need TaskRuntime” interpretation | fixed at construction/reachability level | [V01](../validations/A-SRF-01/V01-01.md), [V02](../validations/A-SRF-01/V02-01.md) |

## Coverage And Uncertainty

This was a pure static review: no Cargo, rustc, test, build, dynamic fixture, or
network operation ran. The direct call paths make the three P1 findings and the
test-contract P2 source-conclusive, while exact terminal rendering under a live
PTY remains future evidence. A-TSK-03 is a catalog dependency but has no Codex
report; when it lands, only ownership/deduplication needs rechecking. A-CHAT-01,
A-HITL-01, A-INP-01, and A-BOOT-01 remain canonical for their overlapping
defects.

## Handoff

- Preserve the current full TUI composition and shared `drive_chat`; do not
  reinterpret existing gaps as a lightweight TUI product decision.
- Reuse the canonical EKO task revision service for attended TUI edits. Do not
  create TUI Plan CRUD, Todo state, or another executor.
- Make durable TaskRuntime events/artifacts the TUI replay source and use the
  framework Subagent bus as a live supplement, with explicit lag resync.
- Preserve full Chart payloads; textual presentation is sufficient only when
  the full typed data/artifact remains inspectable.
- A-TSK-03 should own executor-loop defects; A-SRF-04/X-SRF-01 should compare
  the same chart/full-artifact and replay invariants across CLI/channel/cron.
  Q-E2E-01 should execute the future fixture matrix.
- This report becomes stale if TUI composition, commands, `TuiChatSink`,
  TaskRuntime hydration, Subagent event consumption, or the shared surface
  contract changes.
