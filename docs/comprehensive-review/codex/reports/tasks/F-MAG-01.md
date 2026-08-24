# F-MAG-01: Handoff, topology, and multi-agent coordination

> Status: complete
> Reviewer: Codex review subagent
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d35c9a432370b923acabfbb5f32d63`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: source repositories clean at final source inspection; previously disclosed externally owned changes at `echo-agent-cli/web-frontend/src/generated/ApiError.ts` and `StreamingEvent.ts` were not read, modified, or reverted; reports live outside both source repositories

## Question

Are the public Handoff and topology APIs coherent with the Subagent-only model,
or do they create overlapping identity, routing, ownership, context, result,
cancellation, and lifecycle authorities?

## Scope

- `echo-agent/src/handoff/{mod,tool}.rs`: target/context/result, registry,
  routing, chain, Tool entry, Agent invocation, locking, errors, and cancellation.
- `echo-agent/src/topology.rs`: node/edge identity, callback integration,
  counting/outcome semantics, concurrency, and JSON/Mermaid/DOT exports.
- Root feature/module/prelude exports, manifests, dedicated and enterprise
  examples, React Tool callback pipeline, Agent/Tool/Message contracts.
- EKO manifests and exact symbol search only to classify reachability; the two
  externally owned generated files were excluded.
- Static UTF-8/panic/overflow and test inventory.

## Out Of Scope

- Source fixes or executable validation.
- Subagent registry/catalog/prompt/result defects already in F-SUB-01.
- Sync/Fork/Teammate/Team/background/checkpoint/isolation defects already in
  F-SUB-02.
- A2A transport, workflow DSL, Task DAG, EKO product role routing, and UI graph
  rendering.
- Generic Tool pipeline conclusions beyond the callback/context facts needed for
  HandoffTool/TopologyCallback reachability.

## Inputs

- Root `AGENTS.md`; shared `README.md`, `REPORTING.md`, `TASKS.md`; Codex protocol
  and templates.
- Accepted [F-SUB-01](F-SUB-01.md): canonical definition/registry/context/result
  boundary and its known defects.
- [F-SUB-02](F-SUB-02.md), consumed as needs-evidence temporary input: canonical
  execution lifecycle target and Team/background defects. Findings here do not
  repeat those mode-specific defects.
- Current source and scoped Git history. No other reviewer report was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Typed transfer/delegation, nested routing policy, invocation identity/context, cancellation/deadline, child outcomes, and runtime topology are reusable framework mechanisms. |
| EKO product policy | Role selection, when a conversation should transfer, UI graph projection, topology retention, and return-to-source interaction remain application policy. |
| Adapter boundary | A Handoff-facing API may express transfer intent and a Topology view may project canonical events, but neither should own an Agent registry/scheduler/result classifier or invent uncorrelated identities. |
| Duplicate search | Searched Handoff/transfer/delegate/route, topology/node/edge/callback, registry/register, target/source/parent identities, ToolContext/AgentInvocationContext, cancellation/deadline, result/status/usage/artifact, and all feature/export/caller/test sites across both repositories. |
| Migration deletion | Preserve reasonable public intent APIs, adapt them to one Subagent registry/executor/outcome/event authority, and delete Handoff's raw Agent registry/spawn/prompt/result loop. Rebuild automatic topology as an event projection; delete name-only callback inference and inert return fields. |

## Current Path

```text
programmatic Handoff
  -> HandoffManager private HashMap<String, Arc<dyn Agent>>
  -> serialize HandoffContext into one text prompt
  -> tokio::spawn(raw Agent::execute)
  -> oneshot String -> HandoffResult

model HandoffTool
  -> legacy Tool::execute (ToolContext ignored)
  -> lock Arc<Mutex<HandoffManager>> for complete target execution
  -> same raw handoff path
  -> unstructured ToolResult text -> source Agent continues

TopologyCallback on ReactAgent
  -> on_tool_start(agent name, tool name, args)
  -> ignore args and record Agent -> Tool-name edge
  -> on_tool_end no-op; on_tool_error has no current pipeline caller
  -> name-keyed aggregate HashMaps -> JSON/Mermaid/DOT
```

Both capabilities are legitimate optional framework surfaces even though EKO
does not enable their features. The defect is not application non-use; it is
that their live contracts predate and bypass the canonical Subagent runtime.

## Findings

### F-MAG-01-P1-01: Handoff is a second Agent registry and execution lifecycle outside Subagent authority

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/handoff/mod.rs:143`,
  `echo-agent/src/handoff/mod.rs:156`, `echo-agent/src/handoff/mod.rs:190`,
  `echo-agent/src/handoff/mod.rs:261`, `echo-agent/src/handoff/mod.rs:281`
- Reachability: direct public manager calls are demonstrated by dedicated and
  enterprise examples; HandoffTool delegates into the same manager.
- Expected invariant: Agent-to-Agent delegation/transfer resolves one canonical
  executable identity and uses one invocation, lifecycle, outcome, and event
  authority shared with Subagents.
- Observed behavior: Handoff owns a separate name->Arc Agent registry, raw
  `execute` spawn, text context compiler, String result, and no Subagent events/
  hooks/delegation-depth policy. Same names can identify different executables.
- Impact: registration/removal/readiness, nested limits, fresh instance policy,
  cancellation, evidence, usage, and terminal behavior diverge depending on
  whether consumers select Handoff or Subagent APIs.
- Root cause: a pre-Subagent collaboration feature remained an independent
  orchestration stack after the canonical runtime was introduced.
- Direction: make transfer intent resolve through SubagentRegistry and dispatch a
  typed child request/outcome; delete HandoffManager's Agent HashMap, raw spawn,
  and String-only execution loop after adapters migrate.
- Regression validation: same-name revision, missing/not-ready target, nested
  depth, typed result/events, registration removal, and fresh/serialized instance.
- Validation reports: [V01](../validations/F-MAG-01/V01-01.md),
  [V02](../validations/F-MAG-01/V02-01.md)

### F-MAG-01-P1-02: HandoffTool and prompt conversion discard invocation and structured context/result

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/handoff/tool.rs:27`,
  `echo-agent/src/handoff/tool.rs:90`, `echo-agent/src/handoff/mod.rs:90`,
  `echo-agent/src/handoff/mod.rs:230`,
  `echo-agent/echo-core/src/llm/types.rs:239`,
  `echo-agent/src/handoff/mod.rs:128`
- Reachability: any public HandoffTool registered on an Agent enters legacy
  execute with no ToolContext; direct callers can supply history but use the same
  lossy prompt conversion.
- Expected invariant: run/execution/call identity, trace, cancel, delegation
  policy, working directory, eligible messages/attachments/tool linkage, and
  typed terminal evidence survive transfer.
- Observed behavior: ToolContext is ignored; model Tool creates no message
  history. Programmatic transfer keeps only plain Text content and flattens it
  with metadata into a prompt, dropping multimodal Parts, tool calls/IDs, name,
  reasoning and structural boundaries. Result is target/source/output/unused bool
  then unstructured Tool success text.
- Impact: a transferred task can lose the very file/image/tool evidence it needs,
  disappear from correlated trace/UI records, and convert target failure/partial
  work/evidence into an opaque parent-facing string.
- Root cause: context and result are parallel legacy DTOs rather than lossless
  adapters over Message, AgentInvocationContext, and SubagentOutcome.
- Direction: accept ToolContext/current Message and preserve structured messages;
  route through the canonical invocation/outcome. Delete prompt serialization and
  duplicate result DTO fields once a thin intent/result adapter exists.
- Regression validation: every Message field/variant, metadata delimiter text,
  Unicode, active attachment, ToolContext fields, usage/artifact/remaining-work,
  typed cancel/timeout/failure round-trip.
- Validation reports: [V03](../validations/F-MAG-01/V03-01.md),
  [V09](../validations/F-MAG-01/V09-01.md)

### F-MAG-01-P1-03: Handoff cancellation or outer timeout detaches target execution

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/handoff/mod.rs:261`,
  `echo-agent/src/handoff/mod.rs:265`, `echo-agent/src/handoff/mod.rs:271`,
  `echo-agent/src/handoff/tool.rs:63`,
  `echo-agent/echo-core/src/agent/mod.rs:549`
- Reachability: every direct or Tool handoff spawns the raw target call and drops
  its JoinHandle.
- Expected invariant: caller cancellation/deadline propagates to the target and
  target tool processes; no detached work survives its owner.
- Observed behavior: no token or deadline enters `Agent::execute`. Cancelling or
  timing out the waiter drops only the oneshot receiver; spawned execution keeps
  running. HandoffTool declares no appropriate internal timeout.
- Impact: cancelled work can continue making model/tool/filesystem changes after
  the source run considers the call gone, with no retained handle or terminal
  event.
- Root cause: spawn/oneshot was used to release an imagined lock rather than
  model an owned cancellable child task.
- Direction: remove the detached spawn and dispatch through the canonical child
  token/deadline/handle lifecycle; if a background transfer is desired, use the
  durable background authority identified in F-SUB-02.
- Regression validation: ToolContext cancel, caller drop, outer timeout, target
  stream/tool subprocess, exactly one terminal outcome, and no late side effect.
- Validation reports: [V04](../validations/F-MAG-01/V04-01.md),
  [V09](../validations/F-MAG-01/V09-01.md)

### F-MAG-01-P1-04: HandoffTool can deadlock nested transfer and HandoffManager concurrently executes shared Agents

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/handoff/tool.rs:12`,
  `echo-agent/src/handoff/tool.rs:104`, `echo-agent/src/handoff/mod.rs:145`,
  `echo-agent/src/handoff/mod.rs:262`,
  `echo-agent/echo-core/src/agent/mod.rs:464`
- Reachability: public consumers can share one HandoffTool manager across source
  and targets; direct manager calls need only `&self` and may run concurrently.
- Expected invariant: nested routing makes progress and shared Agent mutable state
  is fresh or serialized.
- Observed behavior: HandoffTool retains its manager mutex guard while awaiting
  the target. A target's transfer through the same manager waits on that guard
  while the outer call waits on the target. Separate direct handoffs clone the
  same Arc Agent and execute it concurrently despite Agent's explicit prohibition.
- Impact: nested collaboration can hang indefinitely; concurrent transfer can
  race conversation/context/tool caches and cross-contaminate outputs.
- Root cause: mutable registry protection is coupled to full execution while
  target instances have no per-instance lifecycle guard/factory.
- Direction: resolve a canonical immutable registration record under a short
  lock, release it, and let SubagentExecutor create/serialize the target. Delete
  the external whole-manager mutex execution scope.
- Regression validation: A->B->C, B->A cycle/depth rejection, simultaneous same-
  target calls, factory generation, cancellation while waiting, and state leak.
- Validation reports: [V05](../validations/F-MAG-01/V05-01.md),
  [V09](../validations/F-MAG-01/V09-01.md)

### F-MAG-01-P1-05: Automatic topology records Tool names, not Subagent routing or outcomes

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/topology.rs:483`,
  `echo-agent/src/topology.rs:490`, `echo-agent/src/topology.rs:499`,
  `echo-agent/src/topology.rs:508`,
  `echo-agent/src/agent/react/run/pipeline.rs:792`,
  `echo-agent/src/agent/react/run/pipeline.rs:798`
- Reachability: TopologyCallback registered through ReactAgentBuilder is called by
  the default live Tool pipeline.
- Expected invariant: automatic multi-Agent topology derives target Subagent and
  correlated execution/result from authoritative dispatch lifecycle events.
- Observed behavior: start ignores args and records Agent -> Tool name; end is a
  no-op; error attempts another count but current pipeline never calls
  `on_tool_error`. Call ID/run ID/status/duration are unavailable. Failed ToolResult
  is treated by the generic callback path as end, not failure.
- Impact: users see `parent -> agent_tool/handoff`, not actual agent relationships,
  and cannot distinguish success, failure, retry, cancellation, or concurrent runs.
- Root cause: a generic Tool callback was treated as a semantic Subagent topology
  event source.
- Direction: project canonical Subagent start/terminal events keyed by execution
  identity into topology; retain generic Tool edges separately if useful. Delete
  target inference from Tool names/args.
- Regression validation: direct/nested/team success/failure/cancel/retry with
  duplicate names and execution IDs; exact edge/status/count/duration.
- Validation reports: [V07](../validations/F-MAG-01/V07-01.md),
  [V09](../validations/F-MAG-01/V09-01.md)

### F-MAG-01-P2-01: Handoff promises control transfer but implements blocking delegation

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/handoff/mod.rs:1`,
  `echo-agent/src/handoff/mod.rs:128`, `echo-agent/src/handoff/mod.rs:183`,
  `echo-agent/src/handoff/mod.rs:281`, `echo-agent/src/handoff/tool.rs:106`
- Reachability: all manager and Tool handoff calls await target output then return
  it to the source; chain repeats this sequentially.
- Expected invariant: API terminology and result fields state whether source
  ownership ends/suspends or whether this is an ordinary child delegation.
- Observed behavior: source always resumes after a Tool result; there is no owner
  state transition. `return_to_source` is always false and never read/configured.
- Impact: public consumers can make incorrect lifecycle assumptions and build a
  second transfer/return protocol around an API that does not implement it.
- Root cause: aspirational control-transfer terminology survived while only an
  RPC-like execute-and-return primitive was built.
- Direction: either rename/fold into Subagent delegation, or add explicit typed
  ownership transfer/return semantics through the canonical runtime. Delete the
  inert bool and duplicate chain when not selected.
- Regression validation: source continuation/suspension, target owner identity,
  return requested/not requested, chain failure/cancel, and terminal events.
- Validation reports: [V06](../validations/F-MAG-01/V06-01.md)

### F-MAG-01-P2-02: Topology is a lossy name aggregate with fragile counters and exports

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/topology.rs:61`,
  `echo-agent/src/topology.rs:141`, `echo-agent/src/topology.rs:163`,
  `echo-agent/src/topology.rs:198`, `echo-agent/src/topology.rs:231`,
  `echo-agent/src/topology.rs:283`, `echo-agent/src/topology.rs:334`
- Reachability: public manual tracker and automatic callback both write this
  state; examples/docs export it.
- Expected invariant: topology identities distinguish executions/Agents, counters
  cannot overflow, state errors are visible, and arbitrary labels produce valid,
  deterministic output.
- Observed behavior: free-form names collapse runs/instances; HashMap order is
  nondeterministic; locks silently degrade on poison; additions/sum are unchecked;
  Mermaid/DOT interpolate unescaped IDs/labels/edge text. UTF-8 truncation is safe.
- Impact: graph lineage/counts can be wrong or disappear, long-running debug
  builds can panic on overflow, and normal quotes/newlines can produce malformed
  diagrams.
- Root cause: display graph storage doubles as runtime lineage without typed
  identity, error, bounds, or structured exporters.
- Direction: consume typed event identities into bounded/saturating records,
  separate display label from escaped renderer ID, sort output deterministically,
  and surface lock/state errors. Delete name-only aggregation as lineage authority.
- Regression validation: same names across runs, quote/newline/emoji labels,
  max counters, poison behavior, deterministic JSON/Mermaid/DOT parse snapshots.
- Validation reports: [V01](../validations/F-MAG-01/V01-01.md),
  [V08](../validations/F-MAG-01/V08-01.md),
  [V09](../validations/F-MAG-01/V09-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Concept/identity/authority duplicate search | yes | failed invariant | [V01](../validations/F-MAG-01/V01-01.md) |
| V02 | Feature/export/registration/real reachability trace | yes | passed | [V02](../validations/F-MAG-01/V02-01.md) |
| V03 | Handoff context/result preservation | yes | failed invariant | [V03](../validations/F-MAG-01/V03-01.md) |
| V04 | Handoff cancellation/timeout ownership | yes | failed invariant | [V04](../validations/F-MAG-01/V04-01.md) |
| V05 | Nested/concurrent routing and Agent ownership | yes | failed invariant | [V05](../validations/F-MAG-01/V05-01.md) |
| V06 | Control-transfer semantics | yes | failed invariant | [V06](../validations/F-MAG-01/V06-01.md) |
| V07 | Automatic topology target/outcome trace | yes | failed invariant | [V07](../validations/F-MAG-01/V07-01.md) |
| V08 | Topology identity/export/UTF-8/panic/overflow | yes | failed invariant | [V08](../validations/F-MAG-01/V08-01.md) |
| V09 | Existing tests and future executable matrix | yes | inconclusive | [V09](../validations/F-MAG-01/V09-01.md) |
| V10 | Report/link/anchor/executor/dirty integrity | yes | passed | [V10](../validations/F-MAG-01/V10-01.md) |
| V30 | Primary source-anchor sampling and acceptance | yes | passed | [V30](../validations/F-MAG-01/V30-01.md) |

Targeted executable fixtures were explicitly prohibited. V09 records future
scenarios without pretending they ran.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| F-SUB-01 canonical definition/context/result boundary | current | [F-SUB-01](F-SUB-01.md); Handoff bypass is a distinct overlap finding |
| F-SUB-02 canonical execution/cancellation target and Team duplicate authorities | current temporary input | [F-SUB-02](F-SUB-02.md); no mode/Team finding was repeated |
| Handoff transfers execution control between Agents with context | regressed | [V03](../validations/F-MAG-01/V03-01.md), [V06](../validations/F-MAG-01/V06-01.md): it is lossy blocking delegation |
| TopologyCallback automatically records Agent call relationships | stale | [V07](../validations/F-MAG-01/V07-01.md): it records Agent-to-Tool names without target/outcome identity |
| spawn avoids holding the Handoff lock during target execution | regressed | [V05](../validations/F-MAG-01/V05-01.md): HandoffTool retains its external manager mutex guard |

## Coverage And Uncertainty

- No compilation or tests ran. V09 is intentionally inconclusive; primary
  static acceptance is recorded in V30 and the task is `complete`.
- HandoffTool has no internal repository construction caller, but it is a public,
  documented, prelude-exported reasonable framework option. Impact is framed for
  consumers that register it; direct HandoffManager paths are example-reachable.
- EKO does not enable either feature. No recommendation deletes framework APIs
  merely for that reason; convergence is based on overlapping framework authority.
- A2A, workflow topology, Task DAG and application projection were excluded.
- Topology callback error observations are scoped to the current default Tool
  pipeline; generic AgentCallback implementers may call on_tool_error elsewhere.

## Handoff

- After primary acceptance, downstream synthesis should treat Handoff intent as
  a candidate adapter over canonical Subagent dispatch, not as another registry/
  scheduler. Topology should project typed Subagent lifecycle events and keep
  generic Tool-use graphs separately.
- Fix order: eliminate detached/raw Handoff execution and mutex scope; preserve
  invocation/message/outcome; define delegation-vs-transfer semantics; then
  rebuild correlated topology and remove legacy authorities.
- This report becomes stale if handoff registry/Tool/context/result, Agent callback
  dispatch, Subagent events/identity, or topology key/export behavior changes.
- Primary reviewer sampled the registry/spawn/context/mutex/callback/export
  anchors and recomputed all seven findings in V30. V10 confirms the delegated
  handoff is mechanically complete.
