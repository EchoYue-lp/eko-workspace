# F-RCT-02: Non-streaming ReAct loop

> Status: complete
> Reviewer: Codex primary reviewer, with isolated subagent evidence
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: `echo-agent` source clean; `echo-agent-cli` had concurrent untracked `web-frontend/src/generated/*.ts` files that were not read, modified, or reverted; this task changed only Codex reports

## Question

Does one non-streaming turn transition correctly through thinking, tool batches,
stopping, errors, limits, and final response?

## Scope

- Framework non-stream entry and collection in
  `echo-agent/src/agent/react/run/react_loop.rs`.
- Canonical shared loop in `run/stream_channel.rs`, including preparation,
  compact, think, verify, tool, and finalization phases.
- Terminal trace/node/checkpoint/transcript/hook/event ownership.
- Finite and unlimited max-step semantics, loop-detector reachability, malformed
  tool-call recovery, cancellation, error propagation, and final response.
- Definition, call, duplicate-authority, current tests, history, panic, UTF-8,
  and overflow inspection.

## Out Of Scope

- Streaming channel ordering/backpressure equivalence: `F-RCT-03`.
- Per-tool concurrency, timeout, cancellation side effects, and result ordering:
  `F-RCT-04`; this report uses only the cancellation-to-terminal boundary.
- Snapshot/resume idempotency: `F-RCT-05`.
- Provider request/response-format defects and provider/cache panics:
  `F-LLM-01`.
- Construction-time option wiring, including the disconnected loop detector:
  F-RCT-01-P2-05. This report records its runtime consequence without creating
  a duplicate finding.
- Application lifecycle/rendering and EKO generated frontend files.
- Cargo, rustc, tests, builds, and dynamic fixtures, prohibited for this task.

## Inputs

- `AGENTS.md`.
- `docs/comprehensive-review/{README.md,REPORTING.md,TASKS.md}` and
  `docs/comprehensive-review/codex/README.md`.
- Dependency report `F-RCT-01` (`needs_evidence` at read time): canonical
  construction path and F-RCT-01-P2-05 option-wiring result.
- Dependency report `F-LLM-01` (`complete`): F-LLM-01-P1-07 owns configured
  response-format omission; F-LLM-01-P1-08 owns provider/cache panic evidence.
- Current source and Git history only; no other reviewer's report was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | ReAct iteration, cancellation, limits, typed terminal outcomes, hook ordering, trace closure, checkpointing, and tool-result insertion are reusable framework mechanisms and belong in `echo-agent`. |
| EKO product policy | None is required to correct these findings. UI text and product persistence projections remain outside this task. |
| Adapter boundary | The non-stream wrapper may adapt a typed core outcome to `Result<String>`, but must not infer success from channel closure or own a second lifecycle. |
| Duplicate search | Searches covered loop/phase definitions and calls, `process_steps`, `run_core_loop`, `AgentTurn`, phase events, finalizers, LoopDetector construction, cancellation events, and current tests. One live loop and one dead competing step processor were found. |
| Migration deletion | Retain the phase-based `run_core_loop`; delete dead `process_steps` and stale comments/tests that imply it is an execution authority. Replace branch finalizers with one terminal commit, deleting branch-specific duplicated terminal side effects after migration. |

No application-to-framework movement is recommended.

## Current Path

The verified non-stream path is:

```text
run_direct / run_chat_direct
  -> run_react_loop (holds execution mutex and active-turn lease)
     -> prepare_react_context
        -> input guard -> start trace -> Recall -> memory/context user message
     -> optional direct-answer shortcut
     -> spawn run_core_loop and collect AgentEvent channel
        -> prepare_turn -> LoopState
        -> for iteration in 0..max_iterations
           -> compact -> think -> budget update
           -> tool batch | verify text | no response
           -> branch-specific finalizer or continue
        -> max-iteration finalizer
     -> break collector on FinalAnswer/Cancelled/Error
     -> drop lease -> Result<String>
```

Source anchors:

- Entrypoint/collector: `src/agent/react/run/react_loop.rs:598-750`.
- Core driver: `src/agent/react/run/stream_channel.rs:494-755`.
- Tool cancellation and completed-batch checkpoint:
  `src/agent/react/run/phases/tools.rs:194-205`, `:295-312`, `:418-442`.
- Terminal branches: `src/agent/react/run/phases/finalize.rs:23-270`.
- Formal but disconnected turn projection: `src/agent/turn.rs:1-80` and
  `src/agent/react/run/react_loop.rs:511-555`.

Finite positive `max_iterations` reaches a typed max-iteration error with
failure hooks, checkpoint, transcript, node, and trace updates. `0` maps to
`usize::MAX`, intentionally making the core practically unbounded. The loop
detector configured by the builder has no production construction/call, as
owned by F-RCT-01-P2-05; therefore unlimited execution currently relies on
external cancellation and contains no intrinsic repeated-call/no-progress
break. Budget arithmetic in the inspected driver is saturating.

Tool calls are reconstructed in stable index order, the assistant tool-call
message is inserted before execution, every completed call receives an ID-bound
tool result/error, and a full-batch checkpoint is saved before the next model
call. Malformed JSON tool arguments are omitted and a retry note is inserted;
this consumes an iteration and becomes especially important under unlimited
mode, but is not independently classified without dynamic provider cases.

## Findings

### F-RCT-02-P1-01: Non-stream channel closure can become successful empty output

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/react/run/react_loop.rs:711-750`,
  `run/stream_channel.rs:592-609`, `:748-750`,
  `run/phases/tools.rs:194-205`, `:295-312`, `:418-422`
- Reachability: `run_direct`/`run_chat_direct` -> `run_react_loop` -> spawned
  `run_core_loop`; tool cancellation returns `IterOutcome::Abandoned`; the
  driver returns `Ok(())`; all senders close; the collector returns its initial
  empty `String`.
- Expected invariant: a successful `Result<String>` requires an explicit
  successful terminal. Cancellation, intervention failure, phase failure, and
  EOF-before-terminal must return typed non-success.
- Observed behavior: the spawn wrapper logs `run_core_loop` errors but does not
  join or forward its terminal result. The collector starts with `String::new()`
  and returns it after channel close. React tool-cancellation branches do not
  produce the `AgentEvent::Cancelled` handled by the collector. Several `?`
  exits depend on a phase having separately sent an error, which is not a
  core-loop contract.
- Impact: cancelled or internally failed turns can be presented as successful
  empty answers. Callers cannot distinguish a valid empty answer from lost
  terminal state, undermining retries, recovery, UI status, and automation.
- Root cause: `Result<String>` is inferred from an intermediate event channel
  while the spawned core's typed completion is discarded.
- Direction: make the spawned core return one typed terminal outcome and await
  its join handle. Publish events from that outcome, reject EOF before terminal,
  and delete collector fallback success. Do not create a second application
  terminal state machine.
- Regression validation: cancel during serial and concurrent tools; final-answer
  intervention cancel/block; forced phase error; receiver close; and ordinary
  final answer. Assert one typed result and one matching durable terminal.
- Validation reports: [V02](../validations/F-RCT-02/V02-01.md),
  [V06](../validations/F-RCT-02/V06-01.md)

### F-RCT-02-P1-02: Stop continuation runs after the turn is already published successful

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/react/run/phases/finalize.rs:141-207`,
  `:87-110`, `echo-agent/src/agent/react/run/react_loop.rs:729-750`
- Reachability: verified final text -> `emit_final_text` -> success persistence
  and `FinalAnswer` -> Stop hook -> `ControlFlow::Continue`; concurrently the
  non-stream collector breaks on `FinalAnswer`, returns, and drops its active
  turn lease.
- Expected invariant: a terminal event is last. Stop continuation is decided
  before trace/node success and terminal publication; a continued run remains
  exclusively owned until its actual terminal.
- Observed behavior: text finalization marks node/trace successful and sends
  `FinalAnswer` before Stop. A continuation then launches another loop
  iteration in the detached core after the non-stream caller has returned. The
  tool final-answer branch also calls Stop after `FinalAnswer`, but merely
  records the reason and terminates, so text and tool semantics disagree.
- Impact: consumers can accept a partial answer as terminal while hidden work
  continues. The execution lease can be released early, allowing a new turn to
  overlap mutation of the same agent/context. Traces, callbacks, and transcripts
  may record multiple or contradictory completions.
- Root cause: Stop is modeled as post-terminal notification in persistence/event
  order but as a pre-terminal continuation decision in control flow.
- Direction: resolve Stop before the terminal commit. If it continues, inject
  context without publishing or persisting success; otherwise use the one common
  terminal authority. Unify text and tool-final-answer behavior and delete the
  branch-specific post-terminal Stop implementations.
- Regression validation: a one-shot continuing Stop hook followed immediately
  by a second caller; assert no early terminal, no lease release, one final
  trace/node completion, and identical text/tool semantics.
- Validation reports: [V03-01](../validations/F-RCT-02/V03-01.md),
  [V06](../validations/F-RCT-02/V06-01.md)

### F-RCT-02-P1-03: Terminal persistence and lifecycle ownership are fragmented by branch

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/react/run/phases/finalize.rs:23-270`,
  `run/phases/prepare.rs:57-92`, `run/stream_channel.rs:509-516`,
  `:704-755`, `run/react_loop.rs:598-750`
- Reachability: all cited finalizers are selected directly by the canonical
  loop; the direct-answer shortcut is selected by the live non-stream wrapper.
- Expected invariant: each success, failure, cancellation, and block passes
  through one terminal commit that consistently closes trace/node state and
  writes the applicable checkpoint/transcript/hooks before publishing terminal.
- Observed behavior: text success finalizes trace, but successful
  `final_answer` tool does not. Max-iteration failure has full fan-out, but
  no-response omits checkpoint/transcript/failure hooks. UserPromptSubmit block
  fires SessionEnd and a FinalAnswer without closing the already-started trace.
  Think/tool abandonment returns without the common terminal projection. The
  direct-answer shortcut bypasses `prepare_turn` and the common finalizers.
- Impact: completed tool answers and blocked/cancelled turns can remain `Running`
  in trace storage; task node, recovery checkpoint, transcript, hooks, and
  emitted result disagree. Resume/diagnostic consumers cannot infer a reliable
  terminal state.
- Root cause: each phase/shortcut manually owns a subset of terminal side
  effects, and there is no typed terminal outcome plus idempotent commit point.
- Direction: introduce one framework terminal outcome and one idempotent commit
  function with an explicit per-outcome policy table. Route direct answer,
  preparation block, think/tool cancellation, no response, max iterations, and
  both success branches through it; delete the duplicated branch persistence.
- Regression validation: table-driven terminal matrix asserting trace, node,
  checkpoint, transcript, SessionEnd/StopFailure, and event cardinality for each
  outcome, including persistence failures.
- Validation reports: [V03-02](../validations/F-RCT-02/V03-02.md),
  [V06](../validations/F-RCT-02/V06-01.md)

### F-RCT-02-P2-04: A dead compiled step executor remains beside the canonical phase loop

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/react/run/react_loop.rs:174-502`,
  `run/stream_channel.rs:494-755`
- Reachability: repository search finds no caller of `process_steps`; it is
  retained with `#[allow(dead_code)]`. All live non-stream and stream execution
  calls `run_core_loop`.
- Expected invariant: one semantic authority owns approval, concurrency, tool
  errors, final-answer interventions, and terminal decisions.
- Observed behavior: the old several-hundred-line `process_steps` independently
  implements those decisions with different data flow and error/terminal
  behavior, while being unreachable.
- Impact: maintainers can fix or extend the wrong implementation, and future
  refactors can accidentally revive stale semantics. It also obscures proof that
  the phase loop is canonical.
- Root cause: loop migration suppressed dead-code warnings instead of deleting
  the replaced internal mechanism.
- Direction: delete `process_steps` and its stale explanatory comments/imports.
  Retain `call_llm_with_retry` only while the separate direct-answer shortcut
  requires it; converge that shortcut's terminal lifecycle under P1-03.
- Regression validation: static zero-reference gate plus existing canonical
  text/tool loop tests.
- Validation reports: [V01](../validations/F-RCT-02/V01-01.md),
  [V05](../validations/F-RCT-02/V05-01.md)

### F-RCT-02-P2-05: The formal AgentTurn lifecycle never advances past Recall

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/turn.rs:1-80`,
  `src/agent/react/run/react_loop.rs:511-555`,
  `src/eval/replay.rs:103-109`
- Reachability: `prepare_react_context` stores a live `AgentTurn` and emits one
  Recall transition. Global search finds no production Think/Act/Finalize
  advance and no `AgentTurn::record_iteration` caller; replay consumes phase
  events.
- Expected invariant: a formal lifecycle projection follows the canonical
  driver's real phase and iteration, or it is not exposed as an authority.
- Observed behavior: the stored turn remains Recall with iteration count zero
  throughout thinking, tools, and finalization; traces likewise receive only
  the Recall transition.
- Impact: diagnostics, replay/eval, checkpoint decisions, and future consumers
  can observe a plausible but false state. The duplicate lifecycle model creates
  maintenance ambiguity with `LoopState` and the phase driver.
- Root cause: `AgentTurn` was initialized by the wrapper but never integrated
  into the later phase-loop extraction.
- Direction: either emit/update phase and iteration centrally at driver
  boundaries using one state owner, or delete `AgentTurn`/transition projection
  if events and `LoopState` are authoritative. Do not manually update it from
  every phase.
- Regression validation: one text and one multi-tool turn asserting monotonic
  ReceiveInput/Recall/Think/Act/Finalize and accurate iteration count, or a
  compile/static gate proving the obsolete projection is gone.
- Validation reports: [V03-03](../validations/F-RCT-02/V03-03.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition, duplicate authority, loop-detector reachability | yes | passed with deviations | [V01](../validations/F-RCT-02/V01-01.md) |
| V02 | Non-stream registration, call graph, and terminal propagation | yes | failed invariant | [V02](../validations/F-RCT-02/V02-01.md) |
| V03 | Stop/terminal ordering | yes | failed invariant | [V03-01](../validations/F-RCT-02/V03-01.md) |
| V03 | Terminal side-effect matrix | yes | failed invariant | [V03-02](../validations/F-RCT-02/V03-02.md) |
| V03 | Formal turn state transitions | yes | failed invariant | [V03-03](../validations/F-RCT-02/V03-03.md) |
| V03 | Panic/UTF-8/overflow bounded inspection | yes | passed | [V03-04](../validations/F-RCT-02/V03-04.md) |
| V04 | Mocked executable turns | future | not run by instruction | [V04](../validations/F-RCT-02/V04-01.md) |
| V05 | History/dependency drift and de-dup | yes | passed | [V05](../validations/F-RCT-02/V05-01.md) |
| V06 | Existing test coverage inventory | yes | passed static inventory | [V06](../validations/F-RCT-02/V06-01.md) |
| V07 | Initial obsolete-path history attempt | no | inconclusive, not adopted | [V07-01](../validations/F-RCT-02/V07-01.md) |
| V07 | Initial obsolete-path test inventory | no | inconclusive, not adopted | [V07-02](../validations/F-RCT-02/V07-02.md) |
| V08 | Initial obsolete-path panic scan | no | inconclusive, not adopted | [V08](../validations/F-RCT-02/V08-01.md) |
| V09 | Initial report integrity gate | no | inconclusive, not adopted | [V09-01](../validations/F-RCT-02/V09-01.md) |
| V09 | Corrected report/source integrity gate | yes | passed | [V09-02](../validations/F-RCT-02/V09-02.md) |
| V30 | Primary source-anchor acceptance | yes | passed | [V30](../validations/F-RCT-02/V30-01.md) |
| V31 | Primary acceptance integrity and source isolation | yes | passed | [V31](../validations/F-RCT-02/V31-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| F-RCT-01: builder reaches the shared phase-based ReAct loop | current | [V01](../validations/F-RCT-02/V01-01.md) |
| F-RCT-01-P2-05: configured loop detector is disconnected | current | `src/agent/config.rs:1025-1032`, `src/agent/react/loop_detector.rs:41-135`, [V01](../validations/F-RCT-02/V01-01.md) |
| F-LLM-01-P1-07: canonical streaming request omits response format | current dependency, not duplicated | [V05](../validations/F-RCT-02/V05-01.md) |
| F-LLM-01-P1-08: provider/cache code can panic | current dependency, outside inspected loop ownership | [V05](../validations/F-RCT-02/V05-01.md) |
| Old `process_steps` comments imply a live execution core | stale | zero caller search and live `run_core_loop`, [V01](../validations/F-RCT-02/V01-01.md) |

## Coverage And Uncertainty

- No executable validation was run by explicit instruction. This does not block
  source-conclusive findings, but runtime timing/cardinality must be validated
  during fixes.
- Future regression cases: ordinary text; ordinary/mixed tool batch; no
  response; finite max; repeated malformed JSON; `max_iterations=0` plus
  repeated tool calls and cancellation; cancel during serial/concurrent tool;
  final-answer intervention cancel/block; prompt-hook block; Stop continuation;
  channel close and persistence error.
- Tool-specific partial-effect, timeout, stream ordering, and backpressure
  semantics remain with F-RCT-03/F-RCT-04.
- The process-list command could not query macOS `sysmond`; no Cargo/rustc/test
  command was started by this task, and all executed static commands exited.
- Initial obsolete-path executions are preserved as inconclusive reports and
  explicitly not used. Corrected current-path reports are the evidence basis.
- Primary acceptance must independently sample anchors and recompute the terminal
  matrix; status therefore remains `needs_evidence`.

## Handoff

- `F-RCT-03` may rely on the canonical producer path and must inspect whether
  stream consumers also accept premature/missing terminals; keep P1-01/P1-02
  owned here unless a distinct stream-only impact is shown.
- `F-RCT-04` should use P1-01's cancellation boundary but independently review
  tool side effects, pairing, timeout, and result ordering.
- `F-RCT-05` must not assume trace/node/checkpoint terminal agreement until
  P1-03 is fixed.
- `A-CHAT-01` should treat framework terminal events as unreliable on these
  branches rather than inventing an application-side lifecycle workaround.
- Remediation order: typed core outcome and common terminal commit (P1-01/P1-03),
  pre-terminal Stop decision (P1-02), then delete dead/duplicate lifecycle
  authorities (P2-04/P2-05).
- Primary review independently sampled the non-stream collector, detached core
  result, Stop/finalization order, dead `process_steps`, and `AgentTurn`
  production callers. The five findings and priorities were accepted; see V30.
- This report becomes stale when `run_react_loop`, `run_core_loop`, phase
  finalizers, `AgentTurn`, loop-detector integration, or terminal event/result
  types change.
