# A-SRF-02: Tauri command and desktop integration

> Status: complete
> Reviewer: Codex review subagent
> Primary acceptance: Codex primary reviewer
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: both source repositories clean at entry; `echo-agent-cli`
> remained clean; unrelated concurrent `echo-agent` changes were excluded and
> are recorded in V00

## Question

Are Tauri commands thin, lifecycle-safe adapters with consistent state and no
duplicate business authority?

## Scope

- Production desktop construction and Tauri command registration under
  `echo-agent-cli/src/tauri` and `src-tauri`.
- `TauriState`, command-to-service ownership, lock/await regions, browser and
  Subagent event bridges, window/application teardown, and PTY lifecycle.
- Frontend callers/consumers only where required to prove registration,
  reachability, typed contract, and impact.
- Static test and scoped history inventory.

## Out Of Scope

- Startup error swallowing, browser prewarm rollback, primary Agent/pool
  shutdown: [A-BOOT-01](A-BOOT-01.md).
- Chat outcome/sink terminal rules: [A-CHAT-01](A-CHAT-01.md).
- HITL provider/pending/deadline/permission policy: [A-HITL-01](A-HITL-01.md).
- Prepared input and attachment identity/lifecycle: [A-INP-01](A-INP-01.md).
- Conversation frontend persistence/projection: [A-STATE-01](A-STATE-01.md).
- TaskRuntime scheduling/terminal settlement: [A-TSK-03](A-TSK-03.md).
- Frontend reducer replay/reconnect beyond proving adapter impact: A-SRF-03.
- Source fixes, shared/Codex index edits, Cargo/rustc/tests/builds/dynamic
  fixtures/network, all prohibited for this task.

## Inputs

- Root `AGENTS.md`; shared `README.md`, `REPORTING.md`, exact `TASKS.md` card;
  Codex `README.md` and report templates.
- Authorized Codex dependency reports [A-BOOT-01](A-BOOT-01.md),
  [A-CHAT-01](A-CHAT-01.md), [A-HITL-01](A-HITL-01.md),
  [A-INP-01](A-INP-01.md), [A-STATE-01](A-STATE-01.md), and
  [A-TSK-03](A-TSK-03.md).
- Current clean `echo-agent-cli` source, scoped Git history, and installed local
  source for the exact Tauri 2.11.2 and portable-pty 0.8.1 dependencies. No
  other reviewer directory was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Framework BrowserRuntime, Subagent event bus, and Graph workflow remain reusable framework capabilities. Their public APIs are not deletion candidates because of EKO reachability. |
| EKO product policy | Desktop window/plugin setup, IPC DTOs, terminal tabs/sessions, local workflow library, and surface parity are EKO application concerns. |
| Adapter boundary | Tauri should register commands, translate typed DTOs, emit canonical events, and own GUI-local resource handles. It must not own product workflow CRUD/execution or silently replace setup/lifecycle hooks. |
| Duplicate search | Searched command definitions/registration, AppState/BrowserRuntime/TerminalManager construction, workflow/skill/worktree operations, setup hooks, event names, frontend invoke/listen calls, and TUI/CLI equivalents. One AppState is shared; one workflow implementation exists but is trapped in the GUI adapter. |
| Migration deletion | Keep framework Graph and app-core services. Consolidate two setup hooks; delete terminal payload previews and the current non-atomic/unawaited cleanup path; move workflow business logic out of `panels.rs`, then delete its private store/executor helpers after all surfaces use the service. |

No EKO window, workflow-library, or terminal-tab policy should move into
`echo-agent`. The right convergence target is an app-core EKO service with thin
Tauri/TUI/CLI/channel adapters.

## Current Path

```text
src-tauri/main -> run_desktop_entry -> run_desktop
  -> construct runtime + one Arc<AppState>
  -> start TaskService/Scheduler/background services
  -> build_tauri_app(AppState, BrowserRuntime)
     -> manage TauriState { AppState, BrowserRuntime, TerminalManager }
     -> setup #1: DevTools + BrowserRuntime -> browser://event
     -> generate_handler![218 unique commands]
     -> setup #2: global shortcut + SubagentEventBus -> execution://event
  -> .run(...)
  -> cancel common token + task hook shutdown + BrowserRuntime shutdown
```

Tauri 2.11.2 `Builder::setup` is a setter, not an accumulator: the second call
replaces setup #1. Therefore only shortcut/Subagent bridging starts. Browser
commands remain registered and execute `BrowserRuntime`, but the React store's
only fact source, `browser://event`, is absent.

The state map is mostly well layered:

| Resource/capability | State owner | Tauri role | Result |
|---|---|---|---|
| Core Agent/workspace/config/tasks/memory/plugins | one `Arc<AppState>` created by desktop | command delegation/projection | shared, no duplicate construction found |
| Browser | one desktop `Arc<BrowserRuntime>` | commands + intended event bridge | commands reachable; bridge overwritten |
| Formal TaskRuntime | app-core services/store | thin queries/actions plus event projection | deeper lifecycle belongs A-TSK-03 |
| Chat/HITL/input/conversation | app-core plus scoped Tauri provider/sink | IPC/event adaptation | deeper findings remain dependency-owned |
| Interactive PTY | Tauri `TerminalManager` | create/write/resize/close/events | appropriate local owner, defective admission/cleanup/logging |
| Saved workflows | raw `AppState.history.workflows` map | adapter performs CRUD, identity, parse, execute | GUI-only product authority, not thin |

## Findings

### A-SRF-02-P0-01: Every interactive terminal write persists a plaintext content preview

- Priority: P0
- Confidence: high
- Layer: adapter
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/web-frontend/src/components/terminal/Terminal.tsx:127`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/terminal.rs:323`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/terminal.rs:344`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/infra.rs:1592`
- Reachability: active Tauri terminal `onData` base64-encodes every keystroke or
  paste -> registered `write_terminal` -> decode -> info log `preview` -> GUI
  tracing appends the record to `~/.eko/logs/app.log`.
- Expected invariant: interactive terminal diagnostics record identity, size,
  and outcome without recording commands, passwords, access tokens, or other
  content.
- Observed behavior: the first 80 decoded bytes of every write are logged
  verbatim after lossy UTF-8 conversion. Pasted secrets are directly stored;
  ordered one-character records can reconstruct longer typed secrets.
- Impact: credentials and confidential commands entered in the local terminal
  persist in a long-lived diagnostic file and stderr, creating direct secret
  exposure on the user's machine.
- Root cause: an online-XSS-oriented “audit every write” mitigation treats
  sensitive payload contents as observability data. The size bound and UTF-8
  conversion address abuse/panic, not confidentiality.
- Direction: delete the content preview and its reconstruction entirely; log
  only session ID, byte count, operation outcome, and a non-content correlation
  ID. Preserve direct interactive terminal access without Agent permission-mode
  gating.
- Regression validation: paste a canary token and type another character by
  character; capture all tracing sinks and assert the canaries/substrings never
  occur while metadata and error outcomes remain observable.
- Validation reports: [V06](../validations/A-SRF-02/V06-01.md),
  [V08](../validations/A-SRF-02/V08-01.md)

### A-SRF-02-P1-02: A second Tauri `setup` replaces the only browser event bridge

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/mod.rs:40`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/mod.rs:311`;
  `/Users/ls/.cargo/registry/src/rsproxy.cn-e3de039b2554c837/tauri-2.11.2/src/app.rs:1765`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/web-frontend/src/hooks/useBrowserEvents.ts:5`
- Reachability: production desktop calls `build_tauri_app`; builder setup #1
  would subscribe to BrowserRuntime and emit `browser://event`, while setup #2
  installs shortcut/Subagent bridging. Tauri assigns the second closure over
  `self.setup`. BrowserPanel listens only to `browser://event`.
- Expected invariant: every registered runtime-to-GUI bridge is installed once;
  adding an unrelated setup concern cannot disable an existing capability.
- Observed behavior: setup #1 never runs. Debug DevTools also do not auto-open,
  but the material defect is absence of all BrowserRuntime events.
- Impact: GUI browser commands may execute, yet the panel receives no session,
  navigation, tab, screenshot, confirmation, action-failure, or close facts;
  browser state stays empty/stale and a major GUI capability is unusable.
- Root cause: `Builder::setup` was assumed additive when it is a single hook
  setter; no builder-composition regression test exists.
- Direction: consolidate DevTools, browser subscription, shortcut registration,
  and Subagent bridge startup into one fallible setup owner, with explicit
  handles/cancellation for spawned bridges. Delete the second competing setter.
- Regression validation: build the real Tauri app and assert both event topics
  are emitted/received exactly once, plus debug setup and shortcut registration;
  fail the test if a later setup replaces any earlier concern.
- Validation reports: [V03](../validations/A-SRF-02/V03-01.md),
  [V08](../validations/A-SRF-02/V08-01.md),
  [V10](../validations/A-SRF-02/V10-01.md)

### A-SRF-02-P1-03: Terminal shutdown and EOF have no authoritative awaited cleanup path

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/terminal.rs:136`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/terminal.rs:256`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/desktop.rs:255`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/web-frontend/src/components/terminal/TerminalDrawer.tsx:60`
- Reachability: every GUI terminal is registered in `TerminalManager`. Explicit
  tab close removes then awaits kill. EOF/error only emits an event. Tauri `.run`
  return never receives/calls the manager. `close_all` has zero production
  callers and spawns unjoined kills even if called.
- Expected invariant: explicit close, natural child exit, window/application
  exit, and partial startup all converge on one idempotent cleanup that removes
  state, terminates/reaps the child, joins the reader, and reports failures.
- Observed behavior: natural exits remain listed indefinitely; app exit does not
  invoke terminal cleanup; the only bulk helper discards kill results and cannot
  be awaited. Reader thread handles and the child handle itself are not retained
  for joining/reaping.
- Impact: the long-running GUI accumulates stale sessions; teardown may leave
  shell/process cleanup to platform-dependent PTY/process behavior, and failures
  are unobservable. Reopen/list/close can target already exited sessions.
- Root cause: ownership stores only master/writer/killer handles and treats event
  emission as terminal cleanup; lifecycle hooks and join handles are absent.
- Direction: make TerminalManager an explicit desktop lifecycle participant;
  retain child/reader ownership and implement one async, idempotent close path
  used by explicit close, EOF/error, startup rollback, and Tauri exit. Delete
  fire-and-forget `close_all`.
- Regression validation: real PTY fixtures for EOF, read/kill failure, explicit
  tab close, window close, and application exit; assert empty manager, reaped
  child, joined reader, exactly one exit event, and surfaced cleanup errors.
- Validation reports: [V05](../validations/A-SRF-02/V05-01.md),
  [V08](../validations/A-SRF-02/V08-01.md)

### A-SRF-02-P1-04: Concurrent duplicate terminal creation can orphan a live shell

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/terminal.rs:216`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/terminal.rs:224`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/terminal.rs:227`
- Reachability: `create_terminal` is a registered async Tauri command and calls
  synchronous `TerminalManager::create`; concurrent invokes using the same ID
  can both pass `contains_key`, spawn independent shells/read threads, then
  insert under the same key.
- Expected invariant: a terminal ID is admitted atomically before any process
  side effect; duplicate creation returns conflict without spawning.
- Observed behavior: check and insert are separated by the entire fallible PTY/
  process/thread spawn. The later insert replaces the first `Arc<PtySession>`;
  dropping its `std::process::Child` handle does not kill the process, and no
  manager key remains through which to close it.
- Impact: one invoke can report a pid/session that is immediately untracked;
  shell and reader resources can live without any addressable cleanup path.
- Root cause: DashMap is used as separate lookup/insert operations instead of
  atomic admission/reservation, and process side effects precede ownership
  commit.
- Direction: reserve the ID atomically using the entry API or a lifecycle slot,
  then spawn and commit; roll back the reservation and kill/reap on every failure.
  Never replace an occupied live session.
- Regression validation: synchronize two same-ID create calls at a barrier for
  many iterations; exactly one may spawn/succeed and the loser must create no
  process/thread. Inject writer/thread-spawn failures and assert rollback.
- Validation reports: [V05](../validations/A-SRF-02/V05-01.md),
  [V08](../validations/A-SRF-02/V08-01.md)

### A-SRF-02-P2-05: Workflow CRUD and execution are GUI-only business authority inside a Tauri command module

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/panels.rs:676`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/panels.rs:709`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/panels.rs:746`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/web-frontend/src/components/workflow/WorkflowPanel.tsx:12`
- Reachability: WorkflowPanel invokes registered list/create/delete/execute
  commands. Those commands directly allocate IDs/timestamps, mutate the
  `StoredWorkflow` map, parse YAML/JSON, create shared state, and await Graph
  execution. Repository search found no app-core workflow service or TUI/CLI
  workflow caller.
- Expected invariant: Tauri is a thin transport for one EKO product service;
  TUI, GUI, CLI, and channel may render differently but expose the same workflow
  capability and authority.
- Observed behavior: complete workflow lifecycle authority lives inside
  `panels.rs`, making the capability GUI-only. Its ad-hoc JSON also disagrees
  with the declared TypeScript `WorkflowInfo` shape.
- Impact: non-GUI surfaces cannot list/create/execute workflows, violating
  feature parity; future callers must duplicate IDs, storage, validation,
  execution, and DTO semantics or couple to Tauri.
- Root cause: product behavior was implemented at the UI adapter boundary
  because no reusable EKO workflow service was established.
- Direction: create one app-core workflow-library service that owns typed
  records, validation, persistence, cancellation, and framework Graph calls;
  route all surfaces through it. Keep Graph generic in the framework and delete
  the private Tauri CRUD/executor logic after cutover.
- Regression validation: field-level service/adapter round trips and identical
  create/list/get/execute/delete/error/cancel scenarios through GUI, TUI, CLI,
  and channel; assert stable identity and generated/shared DTO parity.
- Validation reports: [V01](../validations/A-SRF-02/V01-01.md),
  [V07](../validations/A-SRF-02/V07-01.md),
  [V08](../validations/A-SRF-02/V08-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Commit, clean-state, and concurrent-edit provenance | yes | passed | [V00](../validations/A-SRF-02/V00-01.md) |
| V01 | Definition, ownership, and duplicate search | yes | passed | [V01](../validations/A-SRF-02/V01-01.md) |
| V02 | Command registration and production reachability | yes | passed | [V02](../validations/A-SRF-02/V02-01.md) |
| V03 | Setup composition and event emission contract | yes | failed | [V03](../validations/A-SRF-02/V03-01.md) |
| V04 | State/lock/await inspection | yes | passed | [V04](../validations/A-SRF-02/V04-01.md) |
| V05 | Window/PTY cleanup and atomic identity scenarios | yes | failed | [V05](../validations/A-SRF-02/V05-01.md) |
| V06 | Terminal input confidentiality | yes | failed | [V06](../validations/A-SRF-02/V06-01.md) |
| V07 | Thin adapter, surface parity, and typed DTO boundary | yes | failed | [V07](../validations/A-SRF-02/V07-01.md) |
| V08 | Existing test coverage inventory | yes | failed | [V08](../validations/A-SRF-02/V08-01.md) |
| V09 | Dependency/finding deduplication | yes | passed | [V09](../validations/A-SRF-02/V09-01.md) |
| V10 | Scoped historical drift | yes | passed | [V10](../validations/A-SRF-02/V10-01.md) |
| V11 | Executable desktop/PTY/frontend regressions | future | not_run | [V11](../validations/A-SRF-02/V11-01.md) |
| V12 | Exact ID/header/link/path/source-clean integrity gate | yes | passed | [V12](../validations/A-SRF-02/V12-01.md) |
| V30 | Primary dependency-source and production-path sample | yes | passed | [V30](../validations/A-SRF-02/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `code-review-2026-07-03.md`: terminal audit preview is defense-in-depth | regressed/incorrect under current local threat model | Preview is persisted plaintext secret exposure; [V06](../validations/A-SRF-02/V06-01.md). |
| `docs/MASTER-PLAN.md`: direct interactive terminal is outside Agent automation permission mode | current | No permission-mode check is in create/write; first real keystroke performs session confirmation. |
| `docs/MASTER-PLAN.md`: Tauri/TUI/CLI/channel capabilities are converged | incomplete | Saved workflow lifecycle is reachable only through Tauri/React; [V07](../validations/A-SRF-02/V07-01.md). |
| Browser workspace panel commit intended a live `browser://event` bridge | regressed at introduction | A later-positioned setup already existed, so the new first hook is overwritten; [V03](../validations/A-SRF-02/V03-01.md). |
| Initial desktop terminal integration includes `close_all` | stale as lifecycle evidence | It has no production caller and cannot be awaited; [V05](../validations/A-SRF-02/V05-01.md). |

## Coverage And Uncertainty

- No Cargo, rustc, test, build, dynamic fixture, real WebView, PTY process, or
  network validation ran, by explicit review instruction. V11 is future work,
  not a blocker to the source-conclusive findings.
- Exact child survival after parent application exit is OS/shell dependent. The
  report does not claim universal survival; it claims absence of an owned,
  awaited cleanup/reap contract and definite stale manager entries after EOF.
- The large `panels.rs` was boundary-mapped, with workflow/skill/worktree/
  permission/sandbox samples, not behavior-reviewed feature by feature. Those
  features remain owned by their atomic reviews.
- Tauri `emit` failure handling and frontend reconnect/reducer behavior deserve
  A-SRF-03 coverage. This task only uses frontend code to prove adapter impact.
- Terminal TypeScript DTO mismatch is documented but not a separate finding
  because current Tauri drawer ignores the mismatched fields/return body.
- Concurrent framework modifications listed in V00 were excluded. No finding
  depends on them.

## Handoff

- Primary should first independently confirm Tauri 2.11.2 setup replacement,
  then browser listener exclusivity; this alone proves P1-02 without launching.
- Fix order: remove terminal payload logging immediately; compose one setup;
  make PTY admission/cleanup transactional; then extract workflow product
  authority and add all-surface adapters.
- A-SRF-03 should consume the fact that browser events are never emitted, but
  retain ownership of reducer/reconnect behavior after transport is repaired.
- A-BOOT-01 retains generic desktop/service shutdown findings; A-SRF-02-P1-03
  owns only the separately managed PTY resource.
- Q-E2E-01 should execute V11 scenarios after fixes. Changes to `build_tauri_app`,
  Tauri version/setup semantics, terminal manager/commands, workflow service,
  tracing sinks, or frontend terminal/browser APIs make this report stale.
