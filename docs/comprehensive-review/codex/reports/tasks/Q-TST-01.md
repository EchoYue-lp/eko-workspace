# Q-TST-01: Test suite credibility and coverage map

> Status: complete
> Reviewer: Codex review subagent
> Executor: Codex review subagent
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: framework HEAD matched with 67 external dirty paths; CLI
> HEAD matched with external dirty `Cargo.lock`; all source evidence came from
> the two committed snapshots and all live dirty bodies/diffs were excluded

## Question

Which production invariants have meaningful tests, which tests only restate
implementations, and where do mocks hide integration failures?

## Scope

- Static production-module-to-test topology for all framework Rust packages,
  EKO Rust/app-core, and the React/TypeScript frontend.
- Assertion/fixture quality sampling of every external Rust integration-test
  file plus the named EKO surface contract.
- CI topology, ignored/live/platform-gated tests, and source-level negative
  control of selected critical claims.
- Accepted framework mock limitations from F-TST-01 and explicit test-gap
  metadata from completed Codex subsystem reports.
- Test credibility and coverage ownership only. Runtime defect findings remain
  with their subsystem reports.

## Out Of Scope

- Any Cargo, rustc, Clippy, test, build, frontend command, mutation execution,
  dynamic fixture, network/provider call, benchmark or coverage tool.
- Source fixes, workflow edits, new tests, generated code, index/README changes,
  and other reviewer directories.
- Re-proving production defects already owned by F-RCT, F-TSK, A-SRF, A-FE,
  X-EVT and other subsystem tasks.
- Treating raw test/file counts as statement, branch or invariant coverage.

## Inputs

- Root `AGENTS.md`; shared `README.md`, `REPORTING.md`, exact Q-TST-01 card in
  `TASKS.md`; Codex `README.md`; report templates.
- Dependency [F-TST-01](F-TST-01.md), at the same reviewed commits, for accepted
  mock-contract boundaries.
- Completed Codex subsystem reports [B-BASE-01](B-BASE-01.md),
  [B-PATH-01](B-PATH-01.md), [A-SRF-01](A-SRF-01.md) and
  [X-EVT-01](X-EVT-01.md), read to preserve canonical ownership of CI, prose
  parity and adapter/reducer contract-test findings. Other completed reports
  were used only through mechanically scanned status/Validation Matrix metadata
  for the aggregate gap map.
- Current source at the two fixed committed snapshots. No other reviewer report
  was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Neutral Agent/LLM/Tool scripts, deterministic clocks/cancellation, strict expectation consumption, provider protocol fixtures and framework invariant tests belong to `echo-agent`. |
| EKO product policy | GUI/TUI/CLI/channel parity fixtures, TaskRuntime crash/restart, local files/worktrees and frontend mounted interaction tests belong to `echo-agent-cli`. |
| Adapter boundary | Recorded fixtures must cross the real framework event/result -> EKO adapter -> wire -> reducer boundary without inventing a second schema or lifecycle authority. |
| Duplicate search | Searched committed tests, cfg/ignore gates, integration directories, CI workflows, frontend test dependencies/configs and critical production transports. Existing canonical test findings were reconciled by ID/title before adding Q-TST findings. |
| Migration deletion | Replace prose-only/overclaimed tests with production-connected assertions; delete or rename misleading claims after the real fixture becomes authoritative. Do not delete useful pure unit tests or independent framework public APIs. |

## Current Path

```text
framework production
  -> dense inline unit tests (pure value/algorithm/selected orchestration)
  -> two external test files / eight tests
     -> calibrated tokenizer + shell classifier (honest narrow integration)
     -> cache DTO/layout arithmetic (named more broadly than actual reach)
  -> shared public mocks
     -> valid deterministic happy-path evidence
     -> no timed pending cancellation / strict exhaustion / full Agent/Tool contract

EKO production
  -> dense app-core inline tests
  -> one external runtime_state_e2e file / five tests
     -> real app constructor -> framework Agent state/prompt (meaningful positive seam)
  -> React frontend: 26 test files / 101 test calls
     -> pure stores/reducers + static server markup
     -> no mounted DOM/Tauri/EventSource/WebSocket lifecycle fixture

CI
  -> Ubuntu Rust lanes only
  -> frontend absent; sibling framework floating (canonical B-BASE findings)
  -> deterministic known-red ReAct terminal test excluded by #[ignore]
```

### Production-invariant credibility map

| Invariant family | Current meaningful evidence | Evidence not sufficient for claim | Static disposition |
|---|---|---|---|
| Pure values, serialization, parsers, graph/layout algorithms | Many focused inline tests; `react_smoke` pure paths | Attribute/file counts alone | Creditable per assertion |
| EKO Agent construction, state store and prompt assembly | `runtime_state_e2e` calls real application constructor and inspects framework Agent | Does not execute stream/restart | Creditable positive construction seam |
| ReAct stream terminal/error/cancel | Specialized local fixtures in selected modules | Shared mock cannot hold pending chunks; deterministic red terminal test ignored | Not a mandatory trustworthy gate |
| Tool/Subagent lifecycle, retries and unexpected extra calls | Dedicated stubs are valid where explicitly used | Permissive shared MockAgent/MockTool can fabricate exhausted success | Discount generic mock happy paths for these cells |
| Cache user identity propagation | Local DTO/config/hash/layout tests | `cache_user_id_test` never calls either production assignment | Not covered despite module claim |
| Cross-surface parity and event identity/order | Local producer serialization/reducer assertions | Prose matrix and producer-only contract stop before composition/wire/reducer | Canonical gaps: B-PATH/A-SRF/X-EVT |
| Frontend mount/unmount, listener cleanup, IPC/SSE/WebSocket behavior | Pure reducers/stores and server markup | No DOM/transport harness or critical-hook test | Not covered |
| macOS/Windows process/path/sandbox/desktop adapters | Linux-common compilation/tests | Both CI workflows are Ubuntu-only | Not compiled on target by mandatory CI |
| Live providers/LSP/Zotero | Six opt-in ignored smoke tests, five appropriately environment-backed | No scheduled/latest external compatibility evidence | Optional/manual only |
| Fault/restart/replay/ownership invariants across subsystems | Static atomic reviews define exact scenarios | 120 explicit gap/not-run rows across 66 completed task matrices at sampling time | Highest-value future fixture backlog; not a coverage percentage |

## Findings

### Q-TST-01-P1-01: The frontend suite cannot exercise the mounted application or its transport lifecycle

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/web-frontend/package.json:31`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/web-frontend/src/hooks/useTauriChat.ts:74`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/web-frontend/src/components/tasks/TasksPanel.tsx:130`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/web-frontend/src/components/terminal/Terminal.tsx:112`
- Reachability: GUI chat always mounts `useTauriChat`; task and terminal views
  construct EventSource/WebSocket or Tauri listeners and own asynchronous
  cleanup. These are shipped interactive paths, not dormant utilities.
- Expected invariant: a mounted-component harness drives listener registration,
  unmount during pending import/listen, remount, events, invoke failures,
  reconnect and cleanup through injectable Tauri/browser transports.
- Observed behavior: 26 test files contain 101 test calls, but component tests
  only use server-side static markup. Test bodies contain zero DOM render,
  fireEvent/userEvent, Tauri/invoke, EventSource or WebSocket references; no DOM
  or browser test environment dependency/config exists. The critical transport
  hooks/components have no direct test.
- Impact: listener leaks/ghost events, duplicate subscriptions, cleanup races,
  cancel/command regressions and browser transport reconnection can change the
  shipped GUI while all 101 frontend tests remain green. This is a major
  surface-behavior verification failure, independently of B-BASE-01's separate
  finding that frontend commands are absent from CI.
- Root cause: frontend tests were built around pure reducers/stores and static
  rendering without a mounted adapter boundary or injectable transport.
- Direction: introduce one DOM-capable Vitest environment and a single typed
  in-memory Tauri/browser transport adapter; mount real hooks/components and
  delete direct dynamic-import/global transport coupling after callers migrate.
  Keep pure tests as the fast inner tier.
- Regression validation: separate fixtures for unmount during each await,
  remount, duplicate/out-of-order/late terminal events, invoke rejection,
  EventSource reconnect and WebSocket/listener closure; add them to the
  frontend CI lane owned by B-BASE-01-P2-01.
- Validation reports: [V03](../validations/Q-TST-01/V03-01.md),
  [V10](../validations/Q-TST-01/V10-01.md)

### Q-TST-01-P1-02: A deterministic known-red ReAct terminal regression is excluded from the mandatory gate

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/src/agent/react/run/stream_channel.rs:2313`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/.github/workflows/rust-ci.yml:60`
- Reachability: every streaming ReAct turn consumes the same stream-channel
  loop; the fixture needs no provider or environment and asserts that partial
  output followed by error cannot become FinalAnswer.
- Expected invariant: deterministic regressions for one-terminal/error
  semantics run in the mandatory suite and block a green gate until fixed.
- Observed behavior: `truncated_stream_is_not_accepted_as_complete` is marked
  red and `#[ignore]` until a later milestone. CI runs normal tests only, so the
  known failing core invariant is deliberately invisible to the green result.
- Impact: mandatory CI can report success while a documented major ReAct
  terminal invariant is known to fail. Downstream fixes can neither prove the
  defect closed nor prevent recurrence until someone manually opts into the
  ignored test.
- Root cause: the repository uses ignore as a backlog mechanism for a
  deterministic core regression instead of keeping the defect visible in the
  owning quality gate.
- Direction: fix the runtime defect under its canonical owner, remove the
  ignore immediately, and keep opt-in ignore only for genuine external-service
  smoke tests. Split the comment's distinct clean-EOF claim into a true missing-
  terminal fixture rather than overloading the error fixture.
- Regression validation: active partial-delta+Err and partial-delta+EOF/
  missing-terminal tests, each requiring one Error/Cancelled terminal and no
  FinalAnswer; prove the ordinary mandatory test command executes them.
- Validation reports: [V04](../validations/Q-TST-01/V04-01.md),
  [V10](../validations/Q-TST-01/V10-01.md)

### Q-TST-01-P2-03: Mandatory CI never compiles target-specific macOS and Windows branches

- Priority: P2
- Confidence: high
- Layer: adapter
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/.github/workflows/rust-ci.yml:12`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/.github/workflows/rust-ci.yml:12`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-execution/src/sandbox/local.rs:93`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/ipc.rs:135`
- Reachability: framework sandbox/script execution and EKO desktop/browser/
  IPC/TUI adapters choose OS-specific production branches at compile time.
- Expected invariant: supported desktop targets at least compile their own cfg
  bodies in mandatory CI, with focused behavior fixtures for native adapters.
- Observed behavior: both repositories have only Ubuntu runners. Static search
  finds 42 framework and 14 EKO macOS/Windows cfg occurrences; Linux all-
  features cannot compile target-specific bodies excluded by cfg.
- Impact: syntax/type/API drift in macOS/Windows production code can merge
  green and surface only on a developer or release machine. Native process,
  sandbox, IPC and desktop integration remain materially unverified.
- Root cause: all-features was treated as if it covered target cfgs, while the
  workflow has no target/runner matrix.
- Direction: add pinned macOS and Windows compile lanes for the supported
  binaries/packages plus narrow native adapter fixtures. Keep expensive GUI
  automation as a separate scheduled/release tier.
- Regression validation: source-level negative control in one target-only body
  must fail its target lane while Linux remains independent; execute one native
  process/path/IPC fixture per supported OS.
- Validation reports: [V05](../validations/Q-TST-01/V05-01.md),
  [V10](../validations/Q-TST-01/V10-01.md)

### Q-TST-01-P2-04: The cache propagation integration test never executes the propagation paths it claims to verify

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/tests/cache_user_id_test.rs:1`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/tests/cache_user_id_test.rs:41`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/src/agent/react/run/react_loop.rs:61`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/src/agent/react/run/phases/think.rs:322`
- Reachability: non-streaming and streaming ReAct request construction assigns
  the configured cache identity on live LLM calls. The integration-test module
  explicitly says it verifies every LLM call path.
- Expected invariant: removing either production assignment makes an injected
  request-capturing LLM assertion fail.
- Observed behavior: the tests exercise AgentConfig accessors, ChatRequest
  defaults/manual mutation, tracker arithmetic, hash determinism and cache
  layout. None invokes either ReAct request builder or captures an emitted
  request. A static negative control removing both assignments leaves all
  assertions independent and unchanged.
- Impact: a cache identity propagation regression can collapse provider cache
  hit rates while the prominently named integration suite remains green,
  misleading reviewers about the exact high-cost invariant it claims to guard.
- Root cause: comments/manual source-table inspection were encoded as proof,
  while assertions restate local DTO/config behavior.
- Direction: inject one strict request-recording LLM through streaming and non-
  streaming live ReAct entry paths, assert exact stable identity, and rename or
  split current pure tests to describe their narrower contracts. Delete the
  manual verified-path table once executable reachability owns the claim.
- Regression validation: remove each assignment separately and prove its
  corresponding path test fails; include absent/Unicode/stable-across-turn and
  Subagent inheritance cases without external provider calls.
- Validation reports: [V01](../validations/Q-TST-01/V01-01.md),
  [V06](../validations/Q-TST-01/V06-01.md),
  [V09](../validations/Q-TST-01/V09-01.md),
  [V10](../validations/Q-TST-01/V10-01.md)

## Canonical Existing Findings

These materially affect the coverage map but are not duplicated as Q-TST
findings:

| Existing owner | Current test-credibility consequence |
|---|---|
| B-BASE-01-P2-01 | Frontend test/build commands are absent from CI. |
| B-BASE-01-P2-02 | CLI CI tests a floating framework revision, so results are not reproducible for one repository pair. |
| B-BASE-01-P2-03 | Framework CI does not execute example/bench target-local tests promised by the documented all-target gate. |
| B-PATH-01-P2-02 / A-SRF-01-P2-04 | Surface capability parity is prose, not production composition/reducer evidence. |
| X-EVT-01-P2-06 | Event contract tests stop before adapters/reducers and miss ordering/terminal/replay mutations. |
| F-TST-01-P2-01..04 | Shared mocks cannot establish timed cancellation, full Agent/Tool lifecycle or strict unexpected-call behavior. |
| F-MAC-01-P2-06 | The procedural-macro package's green test command executes no macro contracts. |

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Exact commits and dirty-source isolation | yes | passed | [V00](../validations/Q-TST-01/V00-01.md) |
| V01 | Framework production-module/test topology | yes | passed | [V01](../validations/Q-TST-01/V01-01.md) |
| V02 | EKO Rust/application test topology | yes | passed | [V02](../validations/Q-TST-01/V02-01.md) |
| V03 | Frontend test and transport-lifecycle topology | yes | failed | [V03](../validations/Q-TST-01/V03-01.md) |
| V04 | Ignored/live deterministic-test inventory | yes | failed | [V04](../validations/Q-TST-01/V04-01.md) |
| V05 | Platform-gated source versus CI target matrix | yes | failed | [V05](../validations/Q-TST-01/V05-01.md) |
| V06 | Assertion/fixture quality sampling | yes | failed | [V06](../validations/Q-TST-01/V06-01.md) |
| V07 | Completed subsystem test-gap metadata map | yes | passed with counting caveat | [V07](../validations/Q-TST-01/V07-01.md) |
| V08 | Accepted mock credibility boundary | yes | failed/inherited | [V08](../validations/Q-TST-01/V08-01.md) |
| V09 | Static source-level negative-control sampling | yes | failed | [V09](../validations/Q-TST-01/V09-01.md) |
| V10 | Executable mutation/fault fixture matrix | future | not_run by explicit rule | [V10](../validations/Q-TST-01/V10-01.md) |
| V99 | Report/link/executor/source-boundary integrity | yes | passed | [V99](../validations/Q-TST-01/V99-01.md) |
| V30 | Primary source sampling and acceptance | yes | passed | [V30](../validations/Q-TST-01/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| B-BASE-01: frontend absent and framework checkout floating in CLI CI | current at assigned CLI commit | [V03](../validations/Q-TST-01/V03-01.md), committed workflow in V00 |
| B-PATH/A-SRF: capability matrix validates prose rather than reachability | current | [V06](../validations/Q-TST-01/V06-01.md), [V09](../validations/Q-TST-01/V09-01.md) |
| X-EVT-01: producer contract tests stop before wire/reducer | current | [V06](../validations/Q-TST-01/V06-01.md), [V09](../validations/Q-TST-01/V09-01.md) |
| F-TST-01: shared mocks are valid for narrow deterministic happy paths but not full lifecycle/fault contracts | current at identical framework commit | [V08](../validations/Q-TST-01/V08-01.md) |
| `cache_user_id_test`: every LLM call path is verified | stale/overclaimed | [V06](../validations/Q-TST-01/V06-01.md), [V09](../validations/Q-TST-01/V09-01.md) |

## Coverage And Uncertainty

- No executable test, mutation, build, coverage tool or platform probe was run.
  V10 is future implementation evidence and does not block the source-conclusive
  credibility review after primary acceptance.
- Static evidence is conclusive for test dependency topology, ignored flags,
  CI runner/command omission, fixture reach and absence of frontend harness
  dependencies/calls. It cannot measure timing flake frequency or prove an
  untested branch is currently wrong.
- Attribute and completed-report counts are deliberately not coverage
  percentages. They identify concentration and backlog only.
- The frontend contains meaningful pure reducer/store and static-render tests;
  the framework/application contain many meaningful focused unit tests. This
  review rejects overbroad claims, not the entire suite.
- Live provider/LSP/Zotero tests remain legitimately opt-in under the local
  product threat model. A scheduled compatibility lane is a quality option,
  not a mandatory per-PR network gate.
- Framework live dirty source and CLI Cargo.lock were neither read nor changed.

## Handoff

- Primary sampled V03-V06 anchors and retained existing canonical IDs for
  CI/prose parity/event/mock findings in V30.
- First iteration priority: activate the known-red deterministic ReAct test;
  add the mounted frontend transport fixture; turn cache propagation into a
  production-connected test; establish target compile lanes.
- Use the invariant matrix rather than a numeric coverage target. Every fix
  needs at least one negative control that removes/drops/reorders the exact fact
  and proves the test fails.
- This report becomes stale if either reviewed SHA, either workflow, ignored
  test list, frontend test environment/dependencies, ReAct request builders,
  `surface_contract.rs`, or framework testing mocks change.

## Primary Acceptance

V30 independently confirms all four new test-credibility gaps and their
separation from existing B/F/X owners. Dynamic mutation remains a required
property of replacement tests, not a prerequisite to this static review.
