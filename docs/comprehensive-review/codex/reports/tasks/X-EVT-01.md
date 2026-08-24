# X-EVT-01: Event lifecycle conformance

> Status: complete
> Reviewer: Codex primary reviewer
> Executor: Codex primary reviewer
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: framework inspected only through committed HEAD; external CLI `Cargo.lock` excluded

## Question

Do framework events, EKO persistence, Rust surfaces, and TypeScript reducers
agree on identity, ordering, terminal status, cancellation and timeout?

## Scope

- Framework `AgentEvent`, `EventIdentity`, `EventEnvelope`, sequence/parent and
  terminal validation at committed HEAD.
- EKO `ChatDriverEvent`, Tauri chat/execution wire adapters, ordinary chat
  persistence reachability, TaskRuntime `ExecEvent`/`RuntimeTaskEvent`, and
  frontend chat/Subagent/tool reducers.
- Producer-to-consumer fields, terminal table, variant exhaustiveness, schema
  ownership, live/durable merge, replay reachability, and existing test bounds.

## Out Of Scope

- Tool payload/error/artifact field fidelity owned by X-TOL-01.
- General persistence crash recovery and deletion retention owned by X-STA-01.
- Surface capability availability owned by X-SRF-01.
- Source changes, builds, tests, dynamic fixtures, UI launch and network access.

## Inputs

- Root AGENTS.md and exact shared X-EVT-01 task card.
- Complete Codex dependencies F-CORE-01, F-RCT-03, A-CHAT-01, A-FE-01 and
  A-FE-02. No other reviewer directory was read.
- Current clean application source. Framework evidence was reconstructed from
  `git show`/`git grep` at the pinned commit because its worktree is externally dirty.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Versioned Agent envelopes, stable invocation/event identity, sequence, parent correlation and terminal validation belong in `echo-agent`; they are valid public framework capabilities independent of EKO use. |
| EKO product policy | Durable ordinary-turn ownership, surface replay cursor, frontend merge rules and terminal UI state belong in `echo-agent-cli`. |
| Adapter boundary | EKO must carry the framework envelope losslessly into one versioned application event, then add product-specific Task/Tool/Subagent facts without discarding identity or inventing a second terminal. |
| Duplicate search | Searched both repositories for every envelope/validator consumer plus ChatEvent, StreamingEvent, ServerMessage, ExecEvent, RuntimeTaskEvent, Tauri emit and frontend ingest/replay paths. |
| Migration deletion | Keep the framework envelope and durable TaskRuntime stream. Replace handwritten live event unions and payload-only adapters with one application envelope; delete dormant StreamingEvent/ServerMessage authorities and heuristic duplicate projection after cutover. |

## Producer-To-Consumer Matrix

| Boundary | Identity retained | Ordering retained | Terminal retained | Durable/replayable | Result |
|---|---|---|---|---|---|
| Framework raw stream -> EventEnvelope | conversation/run/turn/execution/parent; no message id | sequence + event id | one FinalAnswer/Cancelled/Error | reusable API, not storage | mostly sound; F-CORE owns remaining defects |
| EventEnvelope -> ChatDriverEvent | full envelope | full | full payload kind | no persistence | positive |
| ChatDriverEvent -> Tauri ChatEvent | only message key + conversation | none | payload mapped, then separate string status/done | no | failed |
| Tauri ChatEvent -> TS reducer | message key/conversation refs | arrival order only | reducer-local status | message autosave only | failed |
| TaskRuntime state mutation -> RuntimeTaskEvent | run/task/step | durable i64 seq + timestamp | typed event kind | JSONL + replay | positive authority |
| TaskRuntime -> live ExecEvent -> execution://event | run/task/Subagent id | seq/timestamp omitted | event string | no cursor | failed |
| RuntimeTaskEvent -> frontend projection | reconstructs execution id from payload/step | seq discarded | first terminal wins | replay adapter | non-confluent |

## Findings

### X-EVT-01-P1-01: The GUI adapter discards the canonical event envelope before its first durable or typed consumer

- Priority: P1; confidence: high; layer: adapter.
- Evidence: committed `echo-agent/echo-core/src/agent/event_envelope.rs:9-83`;
  `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:27-55,483-544`;
  `echo-agent-cli/src/tauri/commands/chat.rs:114-142,1341-1359,1448-1570`;
  `echo-agent-cli/web-frontend/src/types/api.ts:125-172` and
  `web-frontend/src/hooks/useTauriChat.ts:50-64`.
- Reachability: every GUI turn reaches `TauriChatSink`; that sink passes only
  `event.payload` to `agent_event_to_chat_event` and emits a newly serialized
  ChatEvent with only `message_key` and `conversation_id` added.
- Expected invariant: event id, sequence, schema version, turn/run/execution,
  parent and timestamp remain available until a durable reducer has rejected
  duplicates/out-of-order events.
- Observed behavior: all those fields are removed. The TypeScript union cannot
  deduplicate, validate parents, order replay, correlate nested execution, or
  distinguish a reused message key from the original envelope.
- Impact: duplicate or reordered delivery is accepted by arrival order and the
  framework's trajectory validator cannot protect the production GUI boundary.
- Direction: define one EKO surface envelope that embeds/reuses EventEnvelope
  identity and sequence; make the Tauri bridge and TS reducer consume it before
  projection. Do not re-create identity from component refs.
- Validation reports: [V01](../validations/X-EVT-01/V01-01.md),
  [V02](../validations/X-EVT-01/V02-01.md), [V03](../validations/X-EVT-01/V03-01.md).

### X-EVT-01-P1-02: One Agent error or cancellation can become a completed GUI turn

- Priority: P1; confidence: high; layer: cross-layer lifecycle.
- Evidence: committed `echo-agent/echo-core/src/agent/mod.rs:305-335`;
  `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:538-565`;
  `echo-agent-cli/src/tauri/commands/chat.rs:681-711,1365-1387`;
  `echo-agent-cli/web-frontend/src/hooks/chatEventHandler.ts:93-102,140-177,206-216`;
  `web-frontend/src/stores/chatStore.ts:354-361`.
- Reachability: envelope consumption returns `Ok(())` after any terminal
  AgentEvent. Tauri derives terminal status from transport `Result` and the
  external token, not the terminal payload. The frontend error branch sets
  failed and immediately calls a finalizer that overwrites status to completed.
- Expected invariant: FinalAnswer -> completed, Error -> failed, Cancelled ->
  cancelled, timeout -> a typed timed-out/failed outcome; later transport
  markers cannot alter that terminal.
- Observed behavior: Error -> `Ok` -> Tauri completed; frontend Error itself
  becomes completed. AgentEvent::Cancelled without the outer token set is also
  overwritten by the later completed status. Typed non-tool timeout is already
  flattened to Error text by F-CORE-01.
- Impact: UI, logs, webhooks and queue control can claim success for a failed or
  cancelled invocation and disagree about whether a next turn is admissible.
- Root ownership: A-CHAT-01 owns `drive_chat` result collapse; this finding owns
  the end-to-end terminal contradiction including the frontend overwrite.
- Direction: return a typed `TurnOutcome` derived from the one terminal
  envelope, remove independent terminal inference and make reducers monotonic.
- Validation reports: [V04](../validations/X-EVT-01/V04-01.md).

### X-EVT-01-P1-03: Live event variants have no exhaustive, versioned application contract

- Priority: P1; confidence: high; layer: adapter.
- Evidence: committed `echo-agent/echo-core/src/agent/mod.rs:138-143,329-335`;
  `echo-agent-cli/src/tauri/commands/chat.rs:30-112,1448-1570`;
  `echo-agent-cli/web-frontend/src/types/api.ts:119-172`;
  `echo-agent-cli/echo-agent-app-core/src/error.rs:30-76` and
  `echo-agent-app-core/src/types/response.rs:213-300`.
- Reachability: Rust ChatEvent and TypeScript ChatEvent are independently
  handwritten. AgentEvent is non-exhaustive, so the live adapter's wildcard
  converts future variants, including a future terminal, into an informational
  Notice. The exported StreamingEvent and ServerMessage are definition-only
  partial event authorities with different fields/variants.
- Expected invariant: adding or changing a material/terminal variant forces all
  application adapters and reducers to handle it or fail a contract gate.
- Observed behavior: the build can remain green while semantics silently
  degrade. Application chat/execution wires carry no schema version, and the
  framework's v1 number cannot describe these separate unions.
- Impact: provider/framework evolution can remove terminal, identity or typed
  failure semantics without a compiler or replay rejection.
- Direction: one generated, versioned application envelope with an explicit
  forward-compatibility policy; delete dormant parallel enums after consumers
  move. Unknown terminal/material variants must fail closed, not become prose.
- Validation reports: [V05](../validations/X-EVT-01/V05-01.md).

### X-EVT-01-P1-04: TaskRuntime live and durable events cannot be merged by a common identity or order

- Priority: P1; confidence: high; layer: application adapter.
- Evidence: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:85-97`;
  `echo-agent-app-core/src/tasks/task_runtime/types.rs:1339-1360`;
  `echo-agent-cli/src/tauri/commands/chat.rs:145-182,1419-1445`;
  `echo-agent-cli/web-frontend/src/stores/subagentRunStore.ts:325-404,443-520`;
  `web-frontend/src/stores/toolExecutionStore.ts:134-199`.
- Reachability: live TaskRuntime events are emitted as ExecEvent; refresh/reload
  fetches RuntimeTaskEvent and projects it into the same frontend stores.
- Expected invariant: live and replay representations share a stable event id,
  sequence/timestamp, execution/attempt identity and deterministic merge.
- Observed behavior: durable events have seq/timestamp/run/task/step, while live
  ExecEvent omits seq/timestamp/event id and the Tauri wire flattens payload.
  Replay reconstructs identities from optional payload fields and then drops
  durable sequence. Subagent ingestion permanently rejects all events after the
  first terminal, including richer durable facts; Tool merge uses separate
  status/timestamp heuristics.
- Impact: refresh timing changes the final frontend state; late durable results
  can be lost and two stores can disagree about the same execution terminal.
- Root ownership: A-FE-02 owns the first-terminal enrichment symptom; this
  finding owns the missing cross-representation identity/order contract.
- Direction: emit the persisted RuntimeTaskEvent (or a lossless projection with
  its event id/seq) live after append, and use one idempotent merge function.
- Validation reports: [V06](../validations/X-EVT-01/V06-01.md).

### X-EVT-01-P1-05: Ordinary chat has no durable envelope log or replay cursor

- Priority: P1; confidence: high; layer: application.
- Evidence: committed `echo-agent/echo-core/src/agent/event_envelope.rs:107-197`;
  committed repository search finds no production persistence consumer;
  application search finds EventEnvelope only in chat driver, channel tests and
  surface-contract tests. `echo-agent-cli/web-frontend/src/stores/chatStore.ts`
  persists projected messages, not canonical lifecycle events.
- Reachability: every ordinary chat turn is enveloped, then immediately reduced
  by ephemeral sinks; TaskRuntime alone has an append-only sequenced event path.
- Expected invariant: active ordinary turns can rebind/replay from a stable
  cursor and terminal/error facts survive WebView/process restart.
- Observed behavior: `envelope_event_stream_after` and trajectory validation are
  public but unused by EKO persistence. Message autosave cannot restore event
  identity, in-flight status, Tool/phase ordering or one terminal.
- Impact: ordinary chat is the one major execution mode that cannot reconstruct
  canonical state after disconnect/restart; duplicate continuation cannot be
  distinguished from replay.
- Direction: application-owned append-only ordinary-turn event storage keyed by
  conversation/turn, with retention and a cursor shared by all surfaces. Reuse
  the framework envelope; do not add another Agent event type.
- Validation reports: [V07](../validations/X-EVT-01/V07-01.md).

### X-EVT-01-P2-06: Existing contract tests stop before the adapters and reducers that violate the contract

- Priority: P2; confidence: high; layer: validation.
- Evidence: committed framework `echo-core/src/agent/event_envelope.rs` tests;
  `echo-agent-cli/echo-agent-app-core/src/surface_contract.rs:152-293`;
  `echo-agent-cli/web-frontend/src/hooks/chatEventHandler.test.ts:7-113`.
- Observed behavior: framework tests validate local envelopes. The EKO wire test
  serializes ChatDriverEvent before Tauri strips the envelope. Frontend tests do
  not cover Error/Cancelled/timeout, duplicate/out-of-order events, unknown
  variants, live+durable replay or terminal monotonicity.
- Impact: the most important cross-repository lifecycle contract can regress
  while all named unit contracts remain green.
- Direction: one recorded fixture must traverse envelope -> driver -> actual
  adapter wire -> reducer -> durable replay, with mutation controls for every
  identity field, ordering and terminal kind.
- Validation reports: [V08](../validations/X-EVT-01/V08-01.md),
  [V10](../validations/X-EVT-01/V10-01.md).

## Validation Matrix

| ID | Claim or execution | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Exact scope, commits and dirty-source isolation | yes | passed | [V00-01](../validations/X-EVT-01/V00-01.md) |
| V01 | Framework envelope identity/order/terminal authority | yes | passed | [V01-01](../validations/X-EVT-01/V01-01.md) |
| V02 | Producer-to-all-consumer field matrix | yes | failed | [V02-01](../validations/X-EVT-01/V02-01.md) |
| V03 | GUI identity and ordering preservation | yes | failed | [V03-01](../validations/X-EVT-01/V03-01.md) |
| V04 | Final/error/cancel/timeout terminal table | yes | failed | [V04-01](../validations/X-EVT-01/V04-01.md) |
| V05 | Variant exhaustiveness and schema authority | yes | failed | [V05-01](../validations/X-EVT-01/V05-01.md) |
| V06 | TaskRuntime live/durable event merge | yes | failed | [V06-01](../validations/X-EVT-01/V06-01.md) |
| V07 | Ordinary-chat persistence and replay reachability | yes | failed | [V07-01](../validations/X-EVT-01/V07-01.md) |
| V08 | Existing contract/reducer test coverage | yes | failed | [V08-01](../validations/X-EVT-01/V08-01.md) |
| V09 | Dependency finding reconciliation and ownership | yes | passed | [V09-01](../validations/X-EVT-01/V09-01.md) |
| V10 | Recorded cross-layer replay and mutation suite | future | not_run | [V10-01](../validations/X-EVT-01/V10-01.md) |
| V99 | Report/link/source-boundary integrity | yes | passed | [V99-01](../validations/X-EVT-01/V99-01.md) |

## Coverage And Uncertainty

- Static evidence is conclusive for field removal, terminal assignments,
  missing persistence consumers, schema duplication and test boundaries.
- Timing/frequency and executable replay remain future fix-stage evidence in
  V10 under the user's explicit no-build/no-test instruction.
- Framework atomic reports used an older commit; every framework anchor adopted
  here was rebuilt from committed current HEAD. No dirty framework body/diff or
  external CLI lock change was read.

## Handoff

- Preserve EventEnvelope and RuntimeTaskEvent as the two valid generic/durable
  authorities; join them through one versioned EKO surface envelope rather than
  adding more enums.
- Implement typed `TurnOutcome` and monotonic terminal reduction before surface
  feature work, because false completion corrupts every later projection.
- Carry identity/order first, then add ordinary-turn persistence/replay, then
  replace live/durable TaskRuntime heuristics and add the V10 fixture suite.
