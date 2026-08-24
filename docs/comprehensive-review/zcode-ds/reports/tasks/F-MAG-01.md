# F-MAG-01: Handoff, topology, and multi-agent coordination

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63 (baseline 9b0e0fa, clean)
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5 (baseline b3b2e81, clean)
> Worktree state: both source repositories clean (no worktrees in use)

## Question

Are handoff and topology APIs coherent with the Subagent-only model, or do they
create overlapping identity, routing, ownership, or lifecycle authorities?

Answer: they are not coherent. `HandoffManager` is a second agent registry with a
second LLM-facing routing tool (`handoff` vs `agent_tool`) and a detached,
uncancellable, timeout-less execution lifecycle that duplicates — without the
guarantees of — the unified subagent dispatch. Topology is a passive recorder
that cannot see agent-to-agent calls and auto-classifies node identity by guess.
Both are framework-only (EKO does not enable either feature) and exercised only
by examples and happy-path unit tests.

## Scope

Primary source paths inspected (deep read):

- `echo-agent/src/handoff/mod.rs` (391 lines, full) — `HandoffTarget`,
  `HandoffContext`, `HandoffResult`, `HandoffManager` (registry, `handoff`,
  `handoff_chain`), tests.
- `echo-agent/src/handoff/tool.rs` (114 lines, full) — `HandoffTool` LLM tool.
- `echo-agent/src/topology.rs` (611 lines, full) — `NodeType`, `TopologyNode`,
  `TopologyEdge`, `TopologyTracker`, `TopologyCallback`, exports, tests.
- `echo-agent/examples/demo21_handoff.rs`, `demo24_topology.rs` (full) and
  `demo47_enterprise.rs` (handoff/topology sections) — exercised behavior.
- `echo-agent/src/lib.rs:74-79, 102-104, 296-323` and the `advanced` module
  (`:279-326`) — feature gates and re-exports.
- `echo-agent/Cargo.toml:64-75` — feature topology; `echo-agent-cli/Cargo.toml:50`,
  `echo-agent-cli/echo-agent-app-core/Cargo.toml:10-15` — EKO feature enablement.
- `echo-core/src/agent/mod.rs` — `Agent` trait execute contract and
  `AgentCallback` surface (no dispatch callbacks).
- `echo-agent/src/agent/react/run/pipeline.rs:758-795` — callback invocation.
- `echo-agent/src/tools/builtin/agent_dispatch.rs:355-385` — `agent_tool`
  contract (comparison authority).
- Multi-agent documentation: `echo-agent/README.md:70, 127-128, 207, 260-262,
  639-643`; `echo-agent/docs/en|zh/26-multi-agent.md`; `docs/zh/README.md:206-209`;
  `docs/MASTER-PLAN.md:106, 385`; `echo-agent-cli/docs/2026-07-17-ownership-dependency-scheduling.md:10-14`.
- `echo-agent/src/workflow/dsl.rs:88` and
  `echo-agent-cli/echo-agent-app-core/src/config_watcher.rs:8, 251` — "topology"
  term collision inventory.

## Out Of Scope

- Subagent definition/registry/prompt/result semantics and execution-mode
  lifecycles → F-SUB-01, F-SUB-02 (their findings cross-checked only at the
  authority-overlap surface).
- EKO multi-agent product policy (agent pool, subagent loader) → A-SUB-01.
- Workflow graph/DSL semantics (`StateGraph`, `GraphBuilder`) → F-WFL-01
  (only the `StateGraph` "topology builder" doc line is noted as a term
  collision).
- A2A cross-framework protocol → F-INT-02 (external fixed wire, allowed
  exception; `NodeType::External` naming only).
- Intent routing and HITL permission gates → F-INTENT-01, F-HITL-01.

## Inputs

- Root `AGENTS.md` (Subagent-only model; one authority per concept; UTF-8/panic
  safety; layering gates), shared `README.md`, `REPORTING.md`, `TASKS.md`
  (F-MAG-01 card), `zcode-ds/README.md`, report templates.
- Dependency task reports read: zcode-ds `F-SUB-01` (complete — registry/
  dispatch/result authority) and `F-SUB-02` (complete — mode lifecycles, team
  cancel/timeout, dead mailbox machinery).
- Historical documents treated as hypotheses: `echo-agent/README.md`,
  `echo-agent/docs/en|zh/26-multi-agent.md`, root `docs/MASTER-PLAN.md`,
  `echo-agent-cli/docs/2026-07-17-ownership-dependency-scheduling.md` —
  classified in the Historical Claim Status section and V05-01.

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism | Subagent dispatch (`SubagentRegistry` → `agent_tool`/programmatic helpers → `SubagentExecutor`) is the canonical multi-agent mechanism and correctly placed in the framework. Handoff and topology are also framework APIs (correct repository), but handoff internally duplicates the identity/routing/lifecycle of dispatch with weaker semantics — a framework-internal duplicate-authority defect, not a layering error. No repository movement is recommended; the fix is either deletion or re-implementation over the canonical dispatch. |
| EKO product policy | None: EKO does not enable the `handoff` or `topology` features (`echo-agent-cli/Cargo.toml:50`; `echo-agent-app-core/Cargo.toml:10-15`), and no EKO code references any handoff/topology type (V01-01). |
| Adapter boundary | None exists; there is no adapter to move between repos. |
| Duplicate search | Terms searched across both repos: `worker`, `HandoffManager`, `HandoffTool`, `handoff`, `TopologyTracker`, `TopologyCallback`, `NodeType`, `SubAgentMap`, `SubagentRegistry`, `agent_tool`, `delegate_to_agent`, `transfer_control`, `escrow`, `hand_off`, `CancellationToken` (in handoff/topology), `record_call_with_duration`. Results: zero `worker` terms; zero `CancellationToken` in `src/handoff`/`src/topology.rs`; `SubAgentMap` alias consumed only by the registry; `HandoffManager.agents: HashMap<String, Arc<dyn Agent>>` is a second name→agent map; `"handoff"` tool is a second LLM-facing routing tool (never registered by default); `record_call_with_duration` has zero producers outside `topology.rs`; topology node ids are free-form strings decoupled from subagent identity. |
| Migration deletion | If the P2-01 direction (deletion) is taken: delete `src/handoff/` (module + `handoff` feature + re-exports in `lib.rs:77-79, 300-303` and `advanced`) and demos `demo21_handoff.rs`/demo47 handoff section; if re-implementation is chosen, delete the spawn pattern (`mod.rs:262-273`), `return_to_source` (`:285`) and the tool's `transfer_history` semantics. Topology: if kept, add dispatch-event hooks or delete the auto-typing (`topology.rs:199-201`); delete the mailbox doc-fiction in README:642-643 and `26-multi-agent.md` with the F-SUB-02-P2-03 fix. |

## Current Path

Handoff (verified, V02-01): `HandoffManager::new()` → `register*`
(`handoff/mod.rs:157-171`, plain `HashMap`) → `handoff(target, context)`
(`:190-287`): name lookup (`:195`) → prompt assembled by string joining source/
metadata/history/task (`:212-253`, history flattened via `as_text_ref`
`filter_map`, non-text content dropped) → `tokio::spawn` +
oneshot (`:262-273`) around `agent.execute(&full_prompt)` (no cancellation, no
handoff-layer timeout, no events/hooks, no structured result) →
`HandoffResult { return_to_source: false }` (`:285`). `handoff_chain`
(`:293-339`) forces `transfer_history = true` per hop (`:303-304`) and
fabricates user/assistant messages (`:317-323`). The LLM `handoff` tool
(`tool.rs:63-113`) always builds an empty `messages` context (`:91`), so
`transfer_history` is a no-op there, and holds the manager mutex across the
whole target execution (`:104-105`). `HandoffTool::new` has zero callers
outside its module (V01-01): the tool is never registered by any default or
`enable_subagent` path.

Topology (verified, V02-01): `TopologyTracker` (`topology.rs:163-166`,
RwLock-guarded maps) → `add_node`/`record_call[_with_duration]` (`:186-219`,
auto-ensuring `from=Subagent`, `to=Tool`, `:199-201`) → exports
(`:283-403`). `TopologyCallback` (`:460-518`) implements `on_tool_start`
(`:483-497`), fired per tool call (`react/run/pipeline.rs:795`); `AgentCallback`
has no subagent-dispatch hook, so the only auto-recorded edges are agent→tool,
and `agent_tool` dispatches appear as `parent → agent_tool` (Tool node) with the
real subagent name absent.

Comparison authority: `agent_tool` (`agent_dispatch.rs:359-360`) routes through
`SubagentExecutor::dispatch` with child cancel token, timeout, events, hooks,
and structured `SubagentOutcome` (F-SUB-01/F-SUB-02 verified).

## Findings

### F-MAG-01-P1-01: Handoff executes target agents as detached, uncancellable, timeout-less tasks — a dropped or cancelled caller leaves the target running with no lifecycle events

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/handoff/mod.rs:262-273` — `tokio::spawn` around
  `agent.execute(&full_prompt)` with a oneshot; dropping/cancelling the caller
  future detaches the spawned task (standard Tokio semantics); zero
  `CancellationToken` occurrences in `src/handoff` (V01-01); no timeout at the
  handoff layer — a hung target blocks `rx.await` (`:271-273`) indefinitely;
  `agent.execute` (echo-core `Agent` trait) has no cancellation on its plain
  path and resets the target's history each call; `HandoffTool::execute`
  additionally holds the manager mutex across the execution
  (`tool.rs:104-105`).
- Reachability: public `HandoffManager::handoff`/`handoff_chain` — any framework
  consumer; exercised only by examples (`demo21_handoff.rs`, `demo47_enterprise.rs`)
  today; not reachable from EKO production (features disabled, V01-01).
- Expected invariant: MASTER-PLAN:101 — a stopped/cancelled caller must not
  leave agent execution running detached; every dispatch ends in exactly one
  terminal event (F-SUB-01 event contract).
- Observed behavior: the spawned execution is invisible to cancellation; no
  `DispatchStarted`/terminal events, no hooks, no timeout; caller cancellation
  silently orphans the target; a hung target hangs the handoff forever.
- Impact: concurrency/recovery error in a public framework API — detached
  execution and unbounded hangs, the exact "detached execution" class the task
  question asks about; the framework ships two agent-execution lifecycles with
  opposite guarantees, and the documented "control transfer" API is the unsafe
  one.
- Root cause: handoff predates the unified dispatch lifecycle and was built as
  spawn-and-wait without threading the cancellation contract that the subagent
  executor established.
- Direction: reimplement `handoff` over `SubagentExecutor::dispatch` (Sync mode)
  with the caller's child cancel token and the executor timeout/event/hook
  path, or thread `CancellationToken` + timeout into the spawned task and emit
  lifecycle events before the caller's select returns; delete the raw spawn
  pattern. Fix `HandoffTool` to release the manager lock before executing.
- Regression validation: unit tests with a blocking mock agent: (a) cancel the
  parent token mid-handoff → target `execute` future dropped and a typed
  cancelled result returned; (b) hung target + timeout → bounded-time timeout
  error; (c) exactly one terminal event per handoff. Add to Q-FLT-02 fixtures
  (must fail before the fix).
- Validation reports: [V01-01](../validations/F-MAG-01/V01-01.md),
  [V02-01](../validations/F-MAG-01/V02-01.md), [V03-01](../validations/F-MAG-01/V03-01.md)

### F-MAG-01-P2-01: `HandoffManager` is a second agent registry and `handoff` is a second LLM-facing routing tool — parallel identity/routing authority to `SubagentRegistry`/`agent_tool` with divergent semantics

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `HandoffManager.agents: HashMap<String, Arc<dyn Agent>>`
  (`handoff/mod.rs:144-146`) with `register/register_boxed/register_shared`
  (`:157-171`) vs `SubagentRegistry` (`agents`/`definitions`/`factories`,
  `registry.rs:72-78`) with `register*` (F-SUB-01 V02: single registry
  authority for the subagent model); `HandoffTool` name `"handoff"`
  (`tool.rs:28-29`) vs `AgentDispatchTool` name `"agent_tool"`
  (`agent_dispatch.rs:359-360`); the two tools describe overlapping behavior
  ("transfer control to another Agent" vs "dispatch tasks to specialized
  SubAgents") with completely different contracts (no modes, no catalog
  validation, no structured result, no cancellation in handoff); zero
  registration sites for `HandoffTool` (V01-01).
- Reachability: `HandoffManager::new()` programmatic (examples demo21/demo47);
  `handoff` tool requires manual consumer registration (zero production
  constructors, V01-01).
- Expected invariant: one identity/routing authority per concept (AGENTS.md
  "严禁平行实现同一语义"; "任务关系只有一个权威 API"); a name means one agent.
- Observed behavior: the same name can be registered in `HandoffManager` and in
  `SubagentRegistry` as two different agents; the LLM-facing surface offers two
  routing tools with contradictory guarantees; README:70 presents "SubAgent +
  Handoff" as parallel multi-agent patterns.
- Impact: duplicate identity/routing authority — a consumer using handoff gets
  a second, weaker dispatch story; misleading public API and docs; maintenance
  burden when the multi-agent model evolves.
- Root cause: handoff was implemented as an independent mechanism before the
  subagent dispatch became canonical, and was never unified or deleted.
- Direction: either (a) delete `src/handoff/` and the `handoff` feature
  (re-exports in `lib.rs:77-79, 300-303` and `advanced`; demos
  `demo21_handoff.rs` and demo47's handoff section) and document
  `agent_tool`/`SubagentExecutor` as the sole control-transfer mechanism, or
  (b) re-implement handoff as a thin wrapper over `dispatch` (per P1-01) so it
  shares one registry, one identity, and one lifecycle; prefer (a) unless a
  consumer needs the handoff shape.
- Regression validation: after deletion, `cargo check -p echo_agent
  --no-default-features --features handoff` must fail (feature removed) and
  grep for `HandoffManager`/`HandoffTool` returns nothing outside git history;
  if (b), a name registered once must dispatch through the executor with the
  unified result contract.
- Validation reports: [V01-01](../validations/F-MAG-01/V01-01.md),
  [V02-01](../validations/F-MAG-01/V02-01.md), [V05-01](../validations/F-MAG-01/V05-01.md)

### F-MAG-01-P2-02: Handoff context/result preservation is lossy or non-functional — the documented "context-aware transfer" (README:207) and `transfer_history` are not delivered

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: LLM tool path — `HandoffContext::new().with_source(...)` with
  `messages` always empty (`tool.rs:91`), so `transfer_history=true`
  (`tool.rs:79-88`) transfers nothing; programmatic path — history flattened to
  text lines via `msg.content.as_text_ref()` `filter_map`
  (`mod.rs:231-245`): non-text content (tool results, thinking, media) silently
  dropped; `handoff_chain` fabricates `Message::user(task)` +
  `Message::assistant(output)` (`mod.rs:317-323`) and re-renders them as text;
  result is a raw `String` with no structured status/artifacts/evidence
  (`:281-287`); `HandoffResult.return_to_source` is a dead field always `false`
  (`:285`); each `agent.execute` call resets the target's history (echo-core
  `Agent` trait doc).
- Reachability: every `handoff`/`handoff_chain`/`HandoffTool` invocation that
  relies on documented history or context transfer.
- Expected invariant: `transfer_history` transfers conversation; context is
  preserved without semantic loss; the result reflects target status
  (README:207 "Context-aware transfer between agents").
- Observed behavior: nothing is transferred on the LLM path; text-only lossy
  transfer on the programmatic path; no failure/cancel status on the result;
  chain history is a reconstruction, not the real conversation.
- Impact: consumers get unexpected context loss or silent no-ops; the
  documented contract misleads; result shape cannot drive UI or downstream
  logic the way `SubagentOutcome` can.
- Root cause: handoff predates the `SubagentContext`/`ContextInheritance` and
  result-contract work (F-SUB-01); context was modeled as prompt strings.
- Direction: with P2-01 direction (b), build the target invocation through the
  executor's context path so `ContextInheritance` governs; delete
  `return_to_source` and the tool's `transfer_history` parameter (or wire real
  message transfer); if direction (a), the doc claims are deleted with the
  module.
- Regression validation: executor-level test asserting the three documented
  history values on the handoff path (mirroring F-SUB-01-P2-02's test), and a
  test that non-text message content survives or is explicitly rejected.
- Validation reports: [V01-01](../validations/F-MAG-01/V01-01.md),
  [V03-01](../validations/F-MAG-01/V03-01.md), [V05-01](../validations/F-MAG-01/V05-01.md)

### F-MAG-01-P2-03: TopologyTracker auto-classifies node identity by guess — `record_call` hardcodes source=Subagent, target=Tool, and agent-to-agent calls are unrecordable through the callback path; README:128 "Multi-agent topology tracking" is unfulfilled

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `record_call_with_duration` ensures `from` as `NodeType::Subagent`
  and `to` as `NodeType::Tool` unconditionally (`topology.rs:199-201`) —
  correctness depends on callers pre-registering nodes (demo24 does this
  manually, masking the defect); `TopologyCallback::on_tool_start`
  (`:483-497`) records `record_call(agent, tool, "call")` from the only hook
  `AgentCallback` provides (no dispatch callback exists, echo-core `Agent`
  trait surface; invocation at `react/run/pipeline.rs:795`), so `agent_tool`
  dispatches appear as `parent → agent_tool` (Tool node) and the real subagent
  name never enters the graph; node ids are free-form strings with no linkage
  to subagent identity (`agent_tool-{uuid}` execution ids or registry names);
  `record_call_with_duration` has zero producers (V01-01), so `total_duration_ms`
  is always 0; no failure/cancel fixtures exist (V03-01: RwLock poison silently
  no-ops at `:186-190, 203-219, 246-261`; 7 tests cover happy paths only,
  V04-04).
- Reachability: `TopologyTracker`/`TopologyCallback` public API — exercised by
  demo24/demo47; zero EKO usage (V01-01).
- Expected invariant: recorded node types reflect real identities; the tracker
  records agent-to-agent calls as claimed ("Records call relationships between
  Agents at runtime", `topology.rs:1-3`); failure paths are tested.
- Observed behavior: orchestrator/planner calls are auto-labeled Subagent unless
  pre-registered; agent-to-agent edges cannot be produced by the callback path;
  the "call relationship between Agents" promise holds only for agent→tool
  edges; lock failures silently lose data.
- Impact: misleading observability API — a consumer trusting auto-recorded
  graphs gets wrong node types and no multi-agent edges; the documented feature
  ("Multi-agent topology tracking") does not track multi-agent calls.
- Root cause: the tracker was built on the tool-callback surface only; the
  auto-typing is a heuristic, and the subagent dispatch path never got a
  topology hook.
- Direction: either (a) wire topology recording into the subagent event path
  (e.g., a hook on `DispatchStarted`/`DispatchCompleted` carrying registry
  names and execution ids) and drop the `from=Subagent`/`to=Tool` defaults
  (require explicit `NodeType` or derive from a dispatch catalog), or (b) narrow
  the doc claim to "tool-call tracking" and fix the auto-typing; add tests for
  auto-classification, poison, concurrent writers, and long/Unicode labels.
- Regression validation: unit test asserting that a `record_call` without
  pre-registration does not fabricate wrong node types; a fixture recording a
  dispatch event produces `parent → <subagent name>` edge; poison test
  (pre-lock the RwLock) asserting no silent data loss.
- Validation reports: [V01-01](../validations/F-MAG-01/V01-01.md),
  [V02-01](../validations/F-MAG-01/V02-01.md), [V03-01](../validations/F-MAG-01/V03-01.md),
  [V04-04](../validations/F-MAG-01/V04-04.md)

### F-MAG-01-P3-01: "topology" is overloaded across three unrelated meanings, and the multi-agent docs present mailbox/member lifecycle fiction (README:642-643, 26-multi-agent.md) plus a non-conforming "SubAgent" casing

- Priority: P3
- Confidence: high
- Layer: framework (documentation/naming)
- Evidence: (a) term collision — runtime call-graph tracker
  (`src/topology.rs`), workflow `StateGraph` documented as "Declarative Agent
  topology builder" (`workflow/dsl.rs:88`), EKO "MCP server topology"
  (`config_watcher.rs:8, 251`), and "topological-order" task queries
  (`docs/MASTER-PLAN.md:385`); (b) mailbox fiction — README:642-643 "Teammate —
  collaborative mode with shared Mailbox" and `docs/en/26-multi-agent.md:43-49,
  253-272` (per-member `Mailbox` with capacity 64) document machinery that has
  zero production callers (F-SUB-02-P2-03; V05-01); (c) casing — README:70 uses
  "SubAgent" contrary to the Subagent-only terminology rule; (d) README:70
  "SubAgent + Handoff" presents handoff as a peer multi-agent pattern it is not
  (P2-01).
- Reachability: all docs are user-facing (README feature tables, feature-parity
  table, multi-agent guide).
- Expected invariant: one documented model per concept; uniform `Subagent`
  terminology; docs describe code that exists (AGENTS.md; F-SUB-02-P2-03).
- Observed behavior: three "topology" meanings; a documented mailbox lifecycle
  that does not exist; mixed casing in the flagship README.
- Impact: reader confusion; the mailbox claim can mislead consumers into
  wiring dead APIs; terminology drift.
- Root cause: features accumulated without doc normalization; the mailbox
  rewrite (Sprint 11 orchestrator) never updated the docs.
- Direction: rename/qualify the three "topology" concepts in docs (e.g.,
  "runtime call graph", "workflow graph", "MCP topology"); delete the mailbox
  section from `26-multi-agent.md` and fix README:642-643 together with the
  F-SUB-02-P2-03 code deletion; normalize "SubAgent" → "Subagent" in README.
- Regression validation: grep for "Mailbox" in docs returns only code-accurate
  references; grep for `SubAgent\b` in `echo-agent` returns nothing; Q-DOC-01
  sample check.
- Validation reports: [V05-01](../validations/F-MAG-01/V05-01.md)

### F-MAG-01-P3-02: `HandoffTool::execute` holds the manager mutex across the entire target execution, serializing all handoffs and registrations for the duration

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `handoff/tool.rs:104-105` — `let manager = self.manager.lock().await;`
  then `manager.handoff(...).await` inside the guard; the manager lock is
  otherwise only used for short map operations (`mod.rs:157-171, 195`).
- Reachability: any consumer registering `HandoffTool` (zero today, V01-01).
- Expected invariant: no lock is held across a long-running agent execution.
- Observed behavior: a slow target blocks every other handoff and every
  registration through the same manager.
- Impact: low today (no production users); concurrency hazard and a footgun for
  future consumers; also makes the tool the serialization point of the
  duplicate authority (P2-01).
- Root cause: the tool was written before the manager's `handoff` was made
  `&self`/async; the guard's scope was never narrowed.
- Direction: clone the `Arc` and drop the guard before awaiting `handoff`
  (`let manager = self.manager.lock().await.clone(); drop(manager);` shape), or
  move the lock inside `handoff`.
- Regression validation: unit test with a slow mock target asserting a second
  concurrent handoff through the same manager completes without waiting.
- Validation reports: [V03-01](../validations/F-MAG-01/V03-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Cross-repo concept/identity/duplicate search (handoff/topology vs subagent identity, routing, ownership, lifecycle; worker terms; producer inventories) | yes | passed | [V01-01](../validations/F-MAG-01/V01-01.md) |
| V02 | Registration and routing trace (handoff/topology chains vs `agent_tool` dispatch chain; EKO enablement) | yes | passed | [V02-01](../validations/F-MAG-01/V02-01.md) |
| V03 | Invariant/edge-case inspection (handoff result/context preservation; topology failure/cancel fixtures; panic/UTF-8 check) | yes | passed | [V03-01](../validations/F-MAG-01/V03-01.md) |
| V04 | `cargo check -p echo_agent --no-default-features --features handoff --locked` | yes | passed (exit 0) | [V04-01](../validations/F-MAG-01/V04-01.md) |
| V04 | `cargo check -p echo_agent --no-default-features --features topology --locked` | yes | passed (exit 0) | [V04-02](../validations/F-MAG-01/V04-02.md) |
| V04 | `cargo test -p echo_agent --lib --features "handoff,topology" --locked handoff` | yes | passed (exit 0; 4 passed) | [V04-03](../validations/F-MAG-01/V04-03.md) |
| V04 | `cargo test -p echo_agent --lib --features "handoff,topology" --locked topology` | yes | passed (exit 0; 7 passed) | [V04-04](../validations/F-MAG-01/V04-04.md) |
| V05 | Historical-document drift (README, 26-multi-agent, MASTER-PLAN term uses) | yes | passed | [V05-01](../validations/F-MAG-01/V05-01.md) |

All required validations executed with known exit codes; no validation is
pending.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| README:70 — "Multi-agent: SubAgent + Handoff" (parity with LangGraph/CrewAI/AutoGen) | stale/misleading | handoff is example-only, detached, second registry (`handoff/mod.rs:144-146, 262-273`; V01-01, V02-01) |
| README:207 — "Agent Handoff — Context-aware transfer between agents" | stale | history transfer no-op on tool path, lossy text flattening otherwise (P2-02; V03-01) |
| README:127-128, 260-262 — "handoff: Agent handoff/collaboration", "topology: Multi-agent topology tracking" | stale in part | topology cannot record agent-to-agent calls (P2-03; V02-01) |
| README:642-643 — "Teammate — collaborative mode with shared Mailbox"; docs/en/26-multi-agent.md:43-49, 253-272 — per-member Mailbox capacity 64 | stale (fiction) | mailbox machinery dead (F-SUB-02-P2-03; V05-01) |
| docs/zh/26-multi-agent.md, docs/zh/README.md:206-209 — same claims in Chinese | stale (same as above) | V05-01 |
| docs/MASTER-PLAN.md:106 — "TaskRuntime handoff" (GUI projection), :385 — "TaskManager 拓扑查询" | not applicable (different term meaning) | unrelated to HandoffManager/TopologyTracker; V05-01 |
| echo-agent-cli/docs/2026-07-17-ownership-dependency-scheduling.md:10-14 — "Handoff" (worktree code return) | not applicable (different term meaning) | unrelated to framework handoff; V05-01 |
| `workflow/dsl.rs:88` — StateGraph "Declarative Agent topology builder" | current (code) / term collision (docs) | third "topology" meaning; P3-01 |

## Coverage And Uncertainty

- All conclusions are static except the V04 test runs; no live LLM handoff or
  multi-agent run was executed (read-only review). P1-01 rests on the standard
  Tokio semantics of `tokio::spawn` + oneshot with zero cancellation in the
  module (V01-01) — the detach behavior itself is not dynamically reproduced
  here; Q-FLT-02 should carry the fixture.
- `demo47_enterprise.rs` was read only at its handoff/topology sections
  (acceptance checks around the two trackers); its other capabilities were not
  reviewed.
- The `workflow`/`StateGraph` and `a2a` modules were read only at the term/
  boundary level; their semantics belong to F-WFL-01 and F-INT-02.
- Whether handoff is deleted vs re-implemented (P2-01) is a product decision;
  this report documents the divergence, not the choice.
- F-SUB-02 cross-check (mandated): F-SUB-02-P1-01/P1-02 (Team cancel/timeout
  detach) are independent of handoff — `src/handoff` contains no team code and
  zero cancellation tokens; the finding here is the same defect class on a
  different path (all handoff executions, not just Team). F-SUB-02-P2-03 (dead
  coordinator/runner/mailbox) has no code overlap with handoff; the shared
  surface is the mailbox doc-fiction, recorded in P3-01 with a backlink.
- EKO-side projections of multi-agent events were not inspected (A-SUB-01 /
  A-FE-02 / X-EVT-01 scope).

## Handoff

- Conclusions downstream tasks may rely on: handoff and topology are
  framework-only, example-only features with zero EKO reachability (V01-01);
  handoff is a parallel identity/routing/lifecycle authority with a detached,
  uncancellable, timeout-less execution path (P1-01, P2-01); its documented
  context transfer is lossy or non-functional (P2-02); topology cannot see
  agent-to-agent calls and guesses node types (P2-03); docs carry mailbox
  fiction consistent with F-SUB-02-P2-03 (P3-01).
- `F-TSK-03`/`A-TSK-03`: unaffected — handoff does not touch the dispatch or
  TaskRuntime paths.
- `Q-FLT-02`: add the missing fixtures (handoff cancel mid-run, hung target +
  timeout, chain context preservation, topology poison/auto-typing); the
  handoff fixtures must fail before P1-01 lands.
- `Q-DOC-01`: re-sample README and `26-multi-agent.md` multi-agent sections
  after fixes.
- `X-BND-01`: record the handoff deletion-vs-wiring decision (P2-01) and the
  topology event-hook-vs-doc-narrow decision (P2-03).
- `S-RDM-01`: deletion targets — `src/handoff/` + `handoff` feature + lib.rs/
  advanced re-exports + demo21 (if direction (a)); `return_to_source`,
  `transfer_history` (tool), `topology.rs:199-201` auto-typing (if kept);
  mailbox doc sections.
- Reports to read: this report + V01-01..V05-01; dependency reports F-SUB-01
  and F-SUB-02 (authority/lifecycle facts cross-checked here).
- Stale triggers: changes to `src/handoff/*`, `src/topology.rs`, feature
  definitions in `echo-agent/Cargo.toml`, `lib.rs` re-exports, `agent_dispatch.rs`,
  `echo-core` `Agent`/`AgentCallback` trait surfaces, README/26-multi-agent
  multi-agent sections, or EKO enabling the `handoff`/`topology` features
  invalidate the corresponding claims.
- Follow-up task IDs (fixes not implemented in this review): F-WFL-01 (workflow
  term collision), Q-FLT-02, Q-DOC-01, X-BND-01, S-RDM-01.
