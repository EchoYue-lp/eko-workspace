# F-MAG-01: Handoff, topology, and multi-agent coordination

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: not-applicable (framework-only task)
> Worktree state: clean

## Question

Are handoff and topology APIs coherent with the Subagent-only model, or
do they create overlapping identity, routing, ownership, or lifecycle
authorities?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/src/handoff/mod.rs` (391 lines) — `HandoffTarget`
  (3 fields), `HandoffContext` (3 fields), `HandoffResult` (4 fields),
  `HandoffManager` (`HashMap<String, Arc<dyn Agent>>` map + `register` /
  `register_boxed` / `register_shared` / `handoff` / `handoff_chain`),
  the prompt-building body of `handoff`, and the in-module tests.
- `echo-agent/src/handoff/tool.rs` (114 lines) — `HandoffTool` (the
  LLM-facing tool with name `"handoff"`), its `Tool` impl, and the
  `params → HandoffTarget/HandoffContext` mapping.
- `echo-agent/src/topology.rs` (611 lines) — `NodeType` (5 variants),
  `TopologyNode` (4 fields, render DTO), `TopologyEdge` (6 fields),
  `TopologyTracker` (RwLock-protected nodes/edges maps + Mermaid/DOT/JSON
  exporters + `clear`), `TopologyCallback` (the `AgentCallback` impl
  that auto-records tool calls), and the in-module tests.
- `echo-agent/src/lib.rs:295-325` — the `#[cfg(feature = "handoff")]` and
  `#[cfg(feature = "topology")]` re-exports that surface these types
  through the framework prelude.
- `echo-agent/Cargo.toml:74-81` — feature definitions (`handoff = []`,
  `topology = []`, both feature-empty / standalone-compile).

Cross-references inspected for the comparison (not the primary subject):
- `echo-agent/src/agent/subagent/types.rs:130-381` — `SubagentDefinition`
  (22 fields), `SubagentOutcome`, `SubagentArtifact`, `SubagentVerification`,
  `SubagentTouchedFiles`, `render_result_contract`.
- `echo-agent/src/agent/subagent/registry.rs:72-244` — `SubagentRegistry`
  fields, six registration entry points, factory instantiation guard.
- `echo-agent/src/agent/subagent/context.rs:229-281` —
  `SubagentContext::from_parent` (the live context constructor).
- `echo-agent/src/tools/builtin/agent_dispatch.rs:33-55, 360` —
  `AgentDispatchTool` named `agent_tool`, `ParentContextFactory`.
- `echo-agent/echo-core/src/agent/mod.rs:533, 920-988` — the `Agent::execute`
  signature and the `AgentCallback` trait.
- `echo-agent/examples/demo21_handoff.rs`, `demo24_topology.rs`,
  `demo47_enterprise.rs` — the only consumers of the handoff/topology
  surface outside their own declarations.

## Out Of Scope

Deferred to named task IDs:

- Subagent definition/registry/catalog/result contract details →
  **F-SUB-01** (already complete; this task consumes its conclusions).
- Subagent execution-mode lifecycle (Sync/Fork/Teammate/Team),
  cancellation propagation, timeout ownership, isolation factories →
  **F-SUB-02** (already complete; this task consumes its conclusions).
- `AgentCallback` trait's broader observer/intervention semantics beyond
  `TopologyCallback`'s use of it → a hooks-focused task.
- The Tauri/web-frontend rendering of any topology output → a
  frontend-focused task (no such consumer exists today — see V01).
- Application-layer (EKO) handoff/topology wiring → not applicable; no
  echo-agent-cli production code references either surface.

## Inputs

Required repository documents read:

- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/AGENTS.md` (in full via
  system reminder). Key constraints applied: Subagent-only terminology
  (no Worker), framework-vs-application layering, "first check if it
  already exists" (this task is precisely that check for handoff/topology),
  dead-code cleanup (no backward-compat burden), UTF-8 safety,
  cross-repository boundary gate, framework-API retention default
  (`echo-agent` is not `echo-agent-cli`'s private library).
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/REPORTING.md`
  (in full).
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/templates/task-report.md`
  and `templates/validation-report.md` (in full).

Dependency task reports read:

- `docs/comprehensive-review/zcode-glm/tasks/F-SUB-01.md` (in full).
  Established: `SubagentRegistry` is the sole identity authority,
  `AgentDispatchTool` is the sole LLM-facing dispatch entry, and
  `SubagentOutcome` is the sole parent-facing result contract. Confirmed
  "no parallel worker/subagent identity layer exists" *inside the
  subagent module*. F-MAG-01 extends the search to the rest of the
  framework root crate and confirms the parallel layer exists — but
  outside the subagent module, in `handoff/`. F-SUB-01 also established
  the dead-surface pattern (`tool_filter`, `compile_system`,
  `SubagentOutput`/`ContextBuilder`, `lightweight`) that F-MAG-01
  analogously identifies for handoff/topology.
- `docs/comprehensive-review/zcode-glm/tasks/F-SUB-02.md` (in full).
  Established the spawn-detach anti-pattern in Team mode
  (`tokio::spawn` without `JoinHandle::abort()` on timeout/cancel,
  F-SUB-02-P1-02). F-MAG-01 finds the same anti-pattern in
  `HandoffManager::handoff`, and uses the F-SUB-02 framing to
  characterize it. F-SUB-02's handoff explicitly deferred "handoff /
  topology multi-agent coordination APIs → F-MAG-01."

Historical documents treated as hypotheses:

- `README.md:70` — "Multi-agent | SubAgent + Handoff | Graph | Crew |
  Conversation" — treated as product-marketing claim; **code evidence
  shows Handoff is a parallel unused surface, not a peer of SubAgent**
  (see F-MAG-01-P2-01).
- `README.md:127-128` — "`handoff` | yes | Agent handoff/collaboration"
  and "`topology` | yes | Multi-agent topology tracking" — treated as
  feature-list claims; **code evidence shows both features are
  example-only, with zero production wiring** (see F-MAG-01-P2-01 /
  F-MAG-01-P3-01).
- `README.md:207` — "Agent Handoff | Context-aware transfer between
  agents | `HandoffManager::new()`" — treated as design intent; **code
  evidence shows the `HandoffManager::handoff` spawn pattern is
  detached, not context-aware in the SubagentContext sense** (see
  F-MAG-01-P2-02 and V03).
- `handoff/mod.rs:261` comment — "Use spawn to avoid holding the lock"
  — treated as design intent; **code evidence shows there is no lock
  (`agents` is a plain `HashMap`), so the comment's justification is
  stale** (see V04 / F-MAG-01-P2-02).
- `topology.rs:1-4` — "Records call relationships between Agents at
  runtime and supports export to DOT / Mermaid" — treated as design
  intent; **code evidence shows the recorder works but the produced
  graph is structurally incomplete** (misclassified nodes, silent error
  swallow, no `SubagentEvent` integration — see V04).

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | Partially. The *concepts* (named-agent store, delegation descriptor, delegation result, call-graph recorder) are generic and any `echo-agent` consumer might want them — which is why they live in the framework root crate. The *implementations* are minimal: `HandoffManager` is a 3-method wrapper around a HashMap; `TopologyTracker` is a 4-method wrapper around two RwLock<HashMap>s. Neither depends on EKO product policy. |
| EKO product policy | None at this layer. The framework declares the surfaces; no EKO-specific decision is baked in. The application does not consume them (zero echo-agent-cli production references — V01). |
| Adapter boundary | There is no adapter between handoff/topology and the subagent stack — that is the core finding. The two stacks are not bridged: `HandoffManager` does not consult `SubagentRegistry`, and `TopologyCallback` does not subscribe to `SubagentEvent`. Each lives in its own silo. |
| Duplicate search | Searched names: `HandoffManager`, `HandoffTarget`, `HandoffContext`, `HandoffResult`, `HandoffTool`, `handoff`, `handoff_chain`, `register_boxed`, `register_shared`, `TopologyTracker`, `TopologyNode`, `TopologyEdge`, `TopologyCallback`, `TopologyStats`, `TopologyData`, `NodeType`, `to_mermaid`, `to_dot`, `to_json`. Searched both `echo-agent` and `echo-agent-cli`. Result: the handoff stack (`HandoffManager` + `HandoffTarget` + `HandoffResult` + `HandoffTool`) is a **parallel identity/dispatch authority** for the subagent stack — overlapping conceptually but incompatible API-wise, with zero production consumers (only `demo21_handoff.rs` + `demo47_enterprise.rs` call `HandoffManager::handoff` directly, and `HandoffTool::new` has zero callers anywhere). The topology stack does NOT overlap with the subagent execution stack — it is observational only. |
| Migration deletion | No deletion proposed in this review. The handoff and topology surfaces are public framework APIs gated behind their own features (`handoff`, `topology`). Per AGENTS.md's framework-API retention default ("a public framework API is retained unless framework-wide evidence shows it is obsolete or fully replaced"), the dead-within-this-workspace findings here support either (a) deletion or (b) re-wiring onto the live subagent stack, but the choice is a follow-up action, not part of this review task. The README's "SubAgent + Handoff" marketing framing must be reconciled with whichever choice is made. |

## Current Path

Verified handoff and topology call graphs at commit `9b0e0fa`:

```text
Handoff path (parallel to subagent dispatch):
  caller (only demo21_handoff.rs:49, demo47_enterprise.rs:583, or test)
    └─ manager.handoff(target, context)                       [mod.rs:190]
         ├─ self.agents.get(&target.agent_name)               [mod.rs:195]
         │     └─ source: HashMap<String, Arc<dyn Agent>>
         │        (NOT SubagentRegistry — no indirection)
         ├─ on miss → Err(ReactError::Agent(...))             [mod.rs:195-201]
         ├─ build full_prompt (free-text concatenation)       [mod.rs:212-253]
         │     [Handoff source] + [Context metadata] + [Conversation history] + [Task]
         ├─ tokio::spawn({ agent.execute(full_prompt); tx.send })   [mod.rs:265-269]
         │     ★ detached: JoinHandle dropped; no cancel/timeout/abort
         │                (F-MAG-01-P2-02, same shape as F-SUB-02-P1-02)
         └─ rx.await → HandoffResult { output, return_to_source: false, .. }   [mod.rs:271-286]
                ★ no SubagentEvent emitted; no SubagentOutcome structure

HandoffTool path (declared, never constructed):
  HandoffTool::new(manager, source_agent)                     [tool.rs:19]
    └─ ZERO CALLERS across both repos (V02 census)
  were it constructed, Tool::execute would:
    └─ manager.handoff(target, context)                       [tool.rs:105]
           (delegates to the parallel path above)

Subagent path (the live authority, for contrast):
  LLM invokes agent_tool(agent_name, task, mode?, constraints?, background?)
    └─ AgentDispatchTool::dispatch_with_context               [agent_dispatch.rs]
         └─ SubagentExecutor::dispatch                        [executor.rs:407]
              ├─ registry.get(name) → SubagentDefinition + agent  [registry.rs:298-309]
              ├─ mode selection (def.execution_mode or override)
              ├─ dispatch_sync / dispatch_fork / dispatch_teammate / dispatch_team
              └─ execute_agent_streaming → SubagentEvent stream
                    → SubagentOutcome { status, summary, artifacts, verification, … }

Topology path (observational, never wired by default):
  caller (only demo24_topology.rs, demo47_enterprise.rs, or test)
    ├─ tracker.add_node(TopologyNode { id, label, node_type, metadata })   [topology.rs:186]
    ├─ tracker.record_call(from, to, label)                                [topology.rs:193]
    │     └─ ensure_node(from, Subagent); ensure_node(to, Tool)            [topology.rs:200-201]
    │        ★ hardcoded NodeType defaults — F-MAG-01-P3-03
    └─ tracker.to_mermaid() / to_dot() / to_json()                         [topology.rs:283, 334, 396]
           └─ ZERO production consumers of the exported output (V01 census)

TopologyCallback path (AgentCallback hook, never wired by default):
  ReactAgent.add_callback(Arc::new(TopologyCallback::new(tracker)))   [topology.rs:478]
    └─ ZERO production registration sites (V01 census)
  were it wired, on_tool_start(agent, tool, _args) would:
    └─ add_node(agent, NodeType::Subagent)                              [topology.rs:491-492]
       add_node(tool,  NodeType::Tool)                                  [topology.rs:493-494]
       record_call(agent, tool, "call")                                 [topology.rs:495]
          ★ misclassifies orchestrator/external/planner agents as Subagent
```

Key invariants verified by this graph (full evidence in V01-V04):

- **Two identity/dispatch authorities exist for the same conceptual
  operation.** `HandoffManager` (parallel, name-only registration,
  plain-string result, detached spawn) and `SubagentRegistry` +
  `AgentDispatchTool` (canonical, definition-driven registration,
  structured `SubagentOutcome`, cancel-token-aware dispatch) both answer
  "look up a named agent and run a task against it." They are not
  bridged. AGENTS.md's "first check if it already exists" rule is the
  lens: the canonical implementation already existed when handoff was
  added, and the two were never reconciled.
- **Handoff's LLM-facing surface is fully unreachable.** `HandoffTool`
  (the `handoff` LLM tool) has zero construction sites in the entire
  repository. The only live usage of the handoff API is two example
  demos that call `HandoffManager::handoff` directly, bypassing the LLM
  tool. An LLM in a production agent has no way to invoke handoff.
- **Topology is an observational silo.** `TopologyTracker` records
  tool-call edges via the `AgentCallback` trait, not via the
  `SubagentEvent` lifecycle. Subagent-to-subagent dispatch is invisible
  to the topology graph (an `agent_tool` dispatch shows up as a single
  tool edge on the orchestrator; the child's internal tool calls are
  only recorded if the child also has a `TopologyCallback` registered,
  which is never the default).
- **No `Worker`/`worker_` terminology** in `echo-agent/src/handoff/` or
  `echo-agent/src/topology.rs`. AGENTS.md Subagent-only rule is
  respected.

## Findings

### F-MAG-01-P2-01: `HandoffManager` + `HandoffTool` are a parallel identity/dispatch authority with zero production consumers

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/handoff/mod.rs:144-146` — `pub struct HandoffManager
    { agents: HashMap<String, Arc<dyn Agent>> }`. Plain map, not a
    `SubagentRegistry` view.
  - `echo-agent/src/handoff/mod.rs:157-171` — three registration entry
    points (`register`, `register_boxed`, `register_shared`), all
    name+agent-only. Contrast with `SubagentRegistry`'s six
    definition-driven entry points (`registry.rs:131-244`).
  - `echo-agent/src/handoff/tool.rs:11-29` — `HandoffTool` is a
    separately-named LLM tool (`"handoff"`) with its own JSON schema,
    parallel to `AgentDispatchTool` (`"agent_tool"`,
    `agent_dispatch.rs:360`).
  - `echo-agent/src/handoff/mod.rs:195-201` — the lookup path
    `self.agents.get(&target.agent_name)` confirms handoff resolves
    names against its own map, not the registry.
  - Whole-repo caller census (V02): `HandoffTool::new` has **zero**
    callers; `HandoffManager::new` has 2 test sites + 2 example sites;
    `.handoff(` has 2 example sites; **zero production callers** in
    `echo-agent-cli` or any non-`examples/` code in `echo-agent`.
  - `echo-agent/README.md:70, 127, 207, 639, 1085` and
    `CHANGELOG.md:160` market "SubAgent + Handoff" as peer top-line
    multi-agent capabilities.
- Reachability: the only way to reach `HandoffManager::handoff` is to
  construct a `HandoffManager`, register agents into it, and call it
  directly. No production agent does this. The LLM-facing `HandoffTool`
  is never constructed, so even an agent whose tool list contains
  `handoff` does not exist in the codebase.
- Expected invariant: per AGENTS.md "first check if it already exists"
  and "if you find two systems doing the same thing, delete the old
  one," the framework should have one identity/dispatch authority for
  named-agent delegation — the one `AgentDispatchTool` /
  `SubagentRegistry` already provides. A second authority with no
  consumer is a duplicate that misleads consumers (the README's
  "SubAgent + Handoff" framing suggests two live, peer capabilities).
- Observed behavior: two parallel authorities coexist. The canonical
  one (`SubagentRegistry` + `AgentDispatchTool` + `SubagentOutcome`)
  is live and feature-complete (F-SUB-01 / F-SUB-02). The handoff one
  is unwired, has a thinner API, returns an unstructured result
  (F-MAG-01-P2-03), and uses an unsafe spawn primitive
  (F-MAG-01-P2-02).
- Impact: maintainability and API clarity. ~505 lines of public
  framework API (`handoff/mod.rs` 391 + `handoff/tool.rs` 114)
  duplicate the canonical delegation path with no consumer. A
  framework consumer reading the README believes two delegation
  mechanisms exist and are peers; a consumer reading the prelude
  (`lib.rs:302-304`) sees both `HandoffTool` and `AgentDispatchTool`
  exported and must guess which to use. The handoff one is the wrong
  choice (no events, no structure, no cancellation — see V03/V04).
- Root cause: handoff predates the Sprint 5+ subagent unification
  (F-SUB-01's `SubagentRegistry`/`SubagentDefinition`/`AgentDispatchTool`
  consolidation). The unification replaced the ad-hoc delegation model
  in production but did not delete the older `handoff/` module; the
  README still markets both. The same pattern as F-SUB-01's
  `SubagentOutput`/`ContextBuilder`/`tool_filter` dead surfaces, but at
  module scale rather than field scale.
- Direction: pick one. Recommended per AGENTS.md's "delete over retain"
  and "no backward-compat burden": **delete the `handoff` module
  entirely** and reconcile the README to drop "Handoff" from the
  multi-agent marketing (it is subsumed by SubAgent). If a true
  "control-transfer-with-return" semantic is wanted that differs from
  subagent dispatch (e.g. the agent literally yields the conversation
  rather than delegating a sub-task), it should be built *on top of*
  `SubagentRegistry`/`AgentDispatchTool` rather than alongside, so the
  identity, routing, events, and result contract remain unified. Per
  AGENTS.md's framework-API retention caveat, if there is any known
  external consumer of `HandoffManager`, document them before deletion;
  none were found in this workspace.
- Regression validation: after deletion, `cargo test --workspace
  --all-features`; remove `demo21_handoff.rs` and the handoff section
  of `demo47_enterprise.rs`; remove the `handoff` feature from
  `Cargo.toml`; update `README.md`/`README.zh.md`/`docs/{zh,en}/README.md`/`CHANGELOG.md`
  to drop "Handoff" from the multi-agent line.
- Validation reports: [V01](../validations/F-MAG-01/V01-01.md),
  [V02](../validations/F-MAG-01/V02-01.md).

### F-MAG-01-P2-02: `HandoffManager::handoff` spawns detached, non-cancellable agent execution; the "lock" comment is stale

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/handoff/mod.rs:261-273`:
    ```rust
    // Use spawn to avoid holding the lock and blocking other handoff requests during execute
    let agent_arc_clone = agent_arc.clone();
    let (tx, rx) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        let agent = agent_arc_clone.as_ref();
        let result = agent.execute(&full_prompt).await;
        let _ = tx.send(result);
    });
    let output = rx
        .await
        .map_err(|_| ReactError::Other("Handoff task failed to complete".to_string()))??;
    ```
  - `echo-agent/src/handoff/mod.rs:144-146` — `agents: HashMap<String,
    Arc<dyn Agent>>`. The comment claims a "lock" but `agents` is a
    plain HashMap; the immutable borrow at mod.rs:195 releases at the
    end of the `let agent_arc = ...` statement. There is no lock to
    hold.
  - `echo-agent/src/handoff/mod.rs` and `echo-agent/src/handoff/tool.rs`
    — `grep -n "cancel\|timeout\|abort\|CancellationToken"` returns
    **zero hits**. The spawn produces a `JoinHandle` that is dropped
    immediately (only `tx`/`rx` is observed).
  - Contrast with `SubagentExecutor::dispatch_sync` / `dispatch_fork` /
    `dispatch_teammate`, all of which derive
    `execution_cancel = req.cancel.child_token()` and race it in a
    `select!` arm (F-SUB-02 V02/V03). Contrast also with F-SUB-02-P1-02
    (Team mode leak), which is the same anti-pattern at a different
    site.
- Reachability: any caller of `HandoffManager::handoff` (only the 2
  example demos today). On caller-side drop of the `handoff()` future,
  `rx.await` returns `Err(RecvError)`; the spawned task continues to
  completion; the LLM call runs in full; `tx.send(result)` hits a
  dropped receiver and is silently ignored (`let _ = ...`).
- Expected invariant: per F-SUB-02's handoff ("all modes share one
  lifecycle without detached execution") and AGENTS.md's
  prompt-driven-over-state-machine-but-still-safe principle, a
  delegation primitive should not produce background work that
  outlives the caller's interest in the result.
- Observed behavior: the spawned `agent.execute` is fully detached.
  There is no path to cancel it, no timeout, and no way for the caller
  to observe its completion once the receiving future is dropped. The
  LLM/tool budget is burned regardless.
- Impact: low today (handoff has zero production callers per
  F-MAG-01-P2-01), but the API is unsafe for any future consumer. The
  misleading "lock" comment further obscures the actual reason for the
  spawn (lifetime escape from `&'a self`), making it harder for a
  maintainer to recognize the detach as a defect. This is the same
  shape as F-SUB-02-P1-02, which is filed P1 because Team mode is live;
  here it is P2 only because handoff is dead.
- Root cause: the spawn pattern was chosen to decouple the agent
  future's `&'a str` borrow from `&self`, but it simultaneously
  decouples the future from the caller's cancellation domain. The
  author conflated "release the borrow" with "release a lock"; the
  comment fossilized that confusion. Cancellation was never part of
  the handoff design.
- Direction: simplest fix is to drop the spawn entirely and `.await`
  the cloned-Arc future inline:
  ```rust
  let agent_arc = agent_arc.clone();
  let output = agent_arc.execute(&full_prompt).await?;
  ```
  This preserves the borrow-decoupling (the cloned `Arc` owns the
  reference, not `&self`) without detaching. If true concurrency
  between handoff requests is wanted, take a `CancellationToken` (or
  integrate with `SubagentExecutor` and accept a `DispatchRequest`).
  Update or delete the stale "lock" comment. Pair this with the
  F-MAG-01-P2-01 decision: if handoff is deleted, this finding is moot.
- Regression validation: add a `handoff_cancelled_does_not_leak` test
  that cancels the `handoff()` future mid-flight and asserts the
  underlying `agent.execute` is no longer running (mirror the
  `*_timeout_cancels_detached_stream_producer` test shape from
  F-SUB-02). After deletion of handoff: not applicable.
- Validation reports: [V02](../validations/F-MAG-01/V02-01.md),
  [V04](../validations/F-MAG-01/V04-01.md).

### F-MAG-01-P2-03: `HandoffResult` is an unstructured plain-string result; `return_to_source` is dead

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/handoff/mod.rs:128-139` —
    ```rust
    pub struct HandoffResult {
        pub target_agent: String,
        pub source_agent: Option<String>,
        pub output: String,
        pub return_to_source: bool,
    }
    ```
    No status, no artifacts, no verification, no touched_files, no
    schema version. The `output` field is plain `String` with no UTF-8
    bounding (`output_len` is logged at mod.rs:277 but never truncated).
  - `echo-agent/src/handoff/mod.rs:281-286` — `return_to_source: false`
    is the only assignment site; no code path sets it to `true`.
  - `echo-agent/src/handoff/mod.rs:231-245` — context preservation
    flattens the structured `Vec<Message>` history to a free-text
    `[Conversation history]` section via
    `filter_map(|msg| msg.content.as_text_ref())`. Tool calls,
    reasoning content, and structured payloads are silently dropped
    by `as_text_ref()`.
  - Contrast with `SubagentOutcome` (`types.rs:329-345`): 6 field
    families, runtime-owned status, UTF-8-bounded at parse time
    (F-SUB-01 V04). Contrast with `SubagentContext::from_parent`
    (`context.rs:229-281`): structured `messages` (mode-bounded),
    `parent_goal`, `working_dir`, `runtime_context` carrying run_id /
    trace_sink / cancel (F-SUB-02).
  - `grep -n "SubagentEvent" echo-agent/src/handoff/` returns **zero
    hits** — the handoff path emits no lifecycle events.
- Reachability: every `HandoffManager::handoff` call produces a
  `HandoffResult`; every `handoff_chain` step accumulates one. The
  result is observable to direct callers (the 2 example demos).
- Expected invariant: per F-SUB-01's "single result contract"
  conclusion, the framework's delegation primitives should converge on
  one structured result contract (`SubagentOutcome`) so that consumers
  (UIs, trace pipelines, parent LLMs) can interpret results uniformly.
  A second, thinner result type that coexists with the canonical one
  fragments the contract.
- Observed behavior: handoff returns a 4-field struct whose only
  signal-carrying field is `output: String`. Failure is communicated
  only via `Err` on the call, not via a status field. The caller
  cannot ask "did the target complete successfully?" without parsing
  free text. `return_to_source` exists in the public API but the
  producer never sets it to anything but `false` — it is dead within
  the producing path.
- Impact: any consumer that adopts handoff loses the structured result
  channel that `SubagentOutcome` provides. Combined with
  F-MAG-01-P2-02 (no cancellation) and the absence of
  `SubagentEvent`s, handoff is observability-poor relative to the
  canonical path. The dead `return_to_source` field is a false promise
  in the API: a consumer reading the field doc ("Whether to suggest
  returning control to the source agent") believes a control-return
  protocol exists; it does not.
- Root cause: handoff was written before `SubagentOutcome` was
  designed; the result shape was not aligned when the canonical
  contract was introduced. `return_to_source` was provisioned for a
  control-yield semantic that was never implemented (the comment in
  README.md:207 "Context-aware transfer" hints at it).
- Direction: paired with the F-MAG-01-P2-01 decision. If handoff is
  deleted, this finding is moot. If handoff is kept and re-wired on
  top of the subagent stack, `HandoffResult` should either be replaced
  by `SubagentOutcome` or be a thin wrapper that derives from it. The
  `return_to_source` field should be deleted unless a control-yield
  semantic is actually implemented (YAGNI).
- Regression validation: if kept, add a test that asserts
  `HandoffResult.status` (or equivalent) reflects a failed
  `agent.execute`. If deleted: not applicable.
- Validation reports: [V03](../validations/F-MAG-01/V03-01.md).

### F-MAG-01-P3-01: `TopologyTracker` + `TopologyCallback` have zero production consumers; only examples and the re-export reference them

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/topology.rs:163-166` — `TopologyTracker` with
    RwLock-protected nodes/edges maps and Mermaid/DOT/JSON exporters.
  - Whole-repo caller census (V01): `TopologyTracker::new`,
    `TopologyCallback::new`, `to_mermaid`, `to_dot`, `to_json`,
    `TopologyStats`, `TopologyData` — **zero production callers**.
    Only `demo24_topology.rs` and `demo47_enterprise.rs` (examples) and
    the `lib.rs` re-export reference these symbols outside the module
    itself.
  - No production agent registers `TopologyCallback` via
    `add_callback` (`echo-agent/src/agent/react/capabilities.rs:513`).
    The default ReactAgent construction path
    (`builder.rs:927 with_callback`) does not include it.
  - No production renderer consumes the exported DOT/Mermaid/JSON
    output. `echo-agent-cli` does not reference topology at all.
- Reachability: none in production.
- Expected invariant: per AGENTS.md's framework-API retention rule, a
  public framework API without consumers is not necessarily dead
  (it is part of the framework's "capability menu"). But per the same
  rule's nuance, an entire observability subsystem with zero consumers
  AND no integration with the canonical lifecycle (`SubagentEvent`)
  is at minimum a misleading menu item.
- Observed behavior: ~611 lines of public framework API that nothing
  exercises. The README's "Multi-agent topology tracking" line
  (README.md:128) and the `topology` feature flag
  (`Cargo.toml:75`) advertise a capability that no production agent
  uses and no renderer consumes.
- Impact: low (maintainability and API clarity). The risk is drift:
  the topology recorder can evolve without anyone noticing it does not
  match what a future consumer would need. The deeper risk is that
  the topology graph is structurally incomplete even when wired
  (F-MAG-01-P3-03 / F-MAG-01-P3-04), so a future consumer that turns
  it on gets a wrong picture.
- Root cause: topology was designed as a standalone observability
  tool that hooks the generic `AgentCallback` trait, before the
  subagent lifecycle (`SubagentEvent`) was the canonical
  observability channel. The two were never bridged. No application
  ever wired it up.
- Direction: pick one. (a) **Delete the `topology` module and feature**
  (preferred per AGENTS.md "delete over retain" — no consumer, no
  integration with the canonical lifecycle, structurally incomplete
  output). Update README/Cargo.toml/examples accordingly. (b) If kept
  as a framework capability, fix the structural gaps (F-MAG-01-P3-02,
  F-MAG-01-P3-03, F-MAG-01-P3-04) so a future consumer gets correct
  output, and document that the consumer must register
  `TopologyCallback` on every agent (including child subagents) for
  the graph to be complete. Per AGENTS.md framework-API retention
  caveat, document any external consumer before deletion; none were
  found in this workspace.
- Regression validation: after deletion, `cargo test --workspace
  --all-features`; remove `demo24_topology.rs` and the topology section
  of `demo47_enterprise.rs`; remove the `topology` feature from
  `Cargo.toml`; update README/docs/CHANGELOG.
- Validation reports: [V01](../validations/F-MAG-01/V01-01.md).

### F-MAG-01-P3-02: `TopologyTracker` silently swallows every internal `RwLock` error

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/topology.rs:186-190` — `add_node`:
    `if let Ok(mut nodes) = self.nodes.write() { ... }` — silently
    drops the node on lock failure.
  - `echo-agent/src/topology.rs:198-219` — `record_call_with_duration`:
    `if let Ok(mut edges) = self.edges.write() { ... }` — silently
    drops the call.
  - `echo-agent/src/topology.rs:222-228` — `ensure_node`:
    `if let Ok(mut nodes) = self.nodes.write() { ... }` — silently
    skips node creation; the subsequent edge insertion may then
    reference a non-existent node id.
  - `echo-agent/src/topology.rs:231-261` — `nodes`, `edges`, `stats`:
    `.read().map(...).unwrap_or_default()` / `.unwrap_or(0)` — return
    empty Vecs / zero counts on lock failure.
  - `echo-agent/src/topology.rs:264-271` — `clear`:
    `if let Ok(mut ...) = ...write()` — silently leaves the graph
    intact on lock failure.
  - No `tracing::warn!` / `tracing::error!` is emitted on any of these
    branches.
- Reachability: any lock poisoning (a panic while holding the write
  lock) or sustained contention. The topology tests do not inject
  these conditions.
- Expected invariant: a thread-safe tracker that fails to record
  should at minimum log the failure so the operator knows the graph is
  incomplete. Silent swallow produces a graph that looks valid but is
  arbitrarily wrong.
- Observed behavior: the graph silently shrinks (or fails to grow)
  when locks contend. The exported DOT/Mermaid/JSON renderings would
  show fewer nodes/edges than were recorded, with no signal that
  anything is missing.
- Impact: low (the topology module has no production consumer per
  F-MAG-01-P3-01), but it is a latent trap if any consumer wires it
  up. The rendering becomes wrong without any error indication.
- Root cause: the `if let Ok(...)` pattern was chosen for ergonomics
  over explicit error propagation; lock failures were treated as
  impossible. In a real workload with multiple recording threads
  (the callback fires from inside the ReactAgent's tool loop), the
  pattern is unsafe.
- Direction: log a `tracing::warn!(target = "topology", ...)` on every
  lock-failure branch with the method name and the affected key. Or,
  if the module is deleted per F-MAG-01-P3-01, not applicable.
- Regression validation: add a test that poisons the lock and asserts
  a warn is emitted (and the graph remains internally consistent —
  e.g. no edge references a missing node).
- Validation reports: [V04](../validations/F-MAG-01/V04-01.md).

### F-MAG-01-P3-03: `TopologyCallback` and `record_call_with_duration` hardcode `NodeType::{Subagent, Tool}`, misclassifying every observed agent

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/topology.rs:483-497` — `on_tool_start`:
    ```rust
    self.tracker.add_node(TopologyNode::new(agent, NodeType::Subagent));
    self.tracker.add_node(TopologyNode::new(tool, NodeType::Tool));
    self.tracker.record_call(agent, tool, "call");
    ```
    Every observed agent is labelled `Subagent`, even an orchestrator
    or external A2A service. Every target is labelled `Tool`, even
    when the "tool" is actually another agent invoked via
    `agent_tool`.
  - `echo-agent/src/topology.rs:198-201` — `record_call_with_duration`:
    ```rust
    self.ensure_node(from, NodeType::Subagent);
    self.ensure_node(to, NodeType::Tool);
    ```
    Convenience callers inherit the same defaults.
  - The five-variant `NodeType` enum (topology.rs:46-57) defines
    `Orchestrator`, `Planner`, `External` variants that the callback
    and convenience paths can never produce. The only producers of
    those variants are `demo24_topology.rs` (which calls `add_node`
    directly) and the topology unit tests.
- Reachability: every fire of `on_tool_start` or every call to the
  convenience `record_call` / `record_call_with_duration`. In a
  production wiring, every recorded edge would carry the wrong
  `node_type` for any non-subagent orchestrator and any agent-to-agent
  dispatch.
- Expected invariant: a topology recorder that exposes a 5-variant
  `NodeType` enum should classify nodes into the correct variant; an
  orchestrator agent calling `agent_tool` to dispatch a child should
  produce an `Orchestrator → Subagent` edge, not a
  `Subagent → Tool` edge.
- Observed behavior: every edge recorded via the callback path is
  `Subagent → Tool`. The richer `NodeType` taxonomy is unreachable
  from the only path that auto-populates the graph.
- Impact: low today (no production consumer), but a future consumer
  that wires `TopologyCallback` gets a graph that mislabels every
  orchestrator and flattens agent-to-agent dispatch into tool-call
  edges. The DOT/Mermaid renderings would show a single node type
  even though the enum advertises five.
- Root cause: the callback path was written before the orchestrator
  pattern and `agent_tool` inter-agent dispatch were the canonical
  model. The defaults were not revisited when `NodeType` was expanded.
- Direction: pair with F-MAG-01-P3-01. If topology is kept: have the
  callback consult a registry (or accept a `NodeType`-hinting
  registration call) to classify the agent correctly, and detect when
  a "tool" name matches a registered subagent (then label it
  `Subagent`). If topology is deleted: not applicable.
- Regression validation: add a test that registers an orchestrator
  agent and a subagent, wires the callback on both, dispatches the
  subagent via `agent_tool`, and asserts the resulting graph has the
  correct `NodeType` on each node.
- Validation reports: [V04](../validations/F-MAG-01/V04-01.md).

### F-MAG-01-P3-04: `TopologyTracker` has no integration with `SubagentEvent`; subagent-to-subagent dispatch is invisible

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `grep -n "SubagentEvent" echo-agent/src/topology.rs` returns
    **zero hits**. Topology hooks only the generic `AgentCallback`
    trait (`on_tool_start` / `on_tool_end` / `on_tool_error`).
  - The canonical lifecycle channel (`SubagentEventBus` /
    `SubagentEvent::Registered` / `DispatchStarted` /
    `DispatchCompleted` etc., per F-SUB-01) is never bridged to
    topology.
  - In a production wiring where the orchestrator has
    `TopologyCallback` registered but the dispatched child does not,
    the child's internal tool calls are invisible: the orchestrator's
    `on_tool_start` fires for `agent_tool` (the dispatch tool), then
    nothing fires for the child's actual tool calls.
- Reachability: any wiring of `TopologyCallback` in a multi-agent
  scenario (which is exactly the scenario the README's "Multi-agent
  topology tracking" line promises).
- Expected invariant: a topology recorder marketed as tracking
  multi-agent call graphs should observe inter-agent dispatch edges,
  not just tool-call edges from a single agent.
- Observed behavior: inter-agent dispatch is recorded only as a single
  `Orchestrator_node → agent_tool_node` edge (and even that is
  misclassified per F-MAG-01-P3-03). The recursive structure
  (subagent calls another subagent calls a tool) is invisible.
- Impact: low today (no production consumer), but the README's
  marketing implies a complete call graph, and a consumer who turns
  topology on expecting that gets a flat one-level picture.
- Root cause: topology hooks the trait that pre-dates
  `SubagentEventBus`. The subagent lifecycle was not bridged to
  topology when it became the canonical channel.
- Direction: pair with F-MAG-01-P3-01. If topology is kept: bridge
  `SubagentEvent` (specifically `DispatchStarted`/`DispatchCompleted`)
  into `TopologyTracker.record_call`, and have
  `register_agent_dispatch_tool` auto-register a `TopologyCallback`
  on each dispatched subagent (or thread the tracker through
  `AgentInvocationContext`). If topology is deleted: not applicable.
- Regression validation: add a test that dispatches a subagent that
  itself calls a tool, wires topology on the orchestrator only, and
  asserts (after the fix) that both the dispatch edge and the child's
  tool edge are recorded.
- Validation reports: [V04](../validations/F-MAG-01/V04-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Concept/identity overlap grep + duplicate-authority search across both repos | yes | passed | [V01-01](../validations/F-MAG-01/V01-01.md) |
| V02 | Handoff routing trace + HandoffTool/HandoffManager caller census | yes | passed | [V02-01](../validations/F-MAG-01/V02-01.md) |
| V03 | HandoffResult vs SubagentOutcome field-by-field + context-preservation surface | yes | passed | [V03-01](../validations/F-MAG-01/V03-01.md) |
| V04 | Topology silent-swallow + misclassification + handoff spawn-detach + executable test/check/fmt | yes | passed | [V04-01](../validations/F-MAG-01/V04-01.md) |
| V05 | Historical-document drift | conditional (applicable — README/CHANGELOG/code comments treated as hypotheses; classifications in the Inputs section and the Historical Claim Status table) | passed | classified inline |

Executed cargo commands (all exit 0):

```text
cd echo-agent && cargo test --lib -p echo_agent --features handoff,topology -- topology handoff   (11 passed)
cd echo-agent && cargo check -p echo_agent --no-default-features --features handoff --lib
cd echo-agent && cargo check -p echo_agent --no-default-features --features topology --lib
cd echo-agent && cargo fmt --all -- --check
```

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `README.md:70` — "Multi-agent \| SubAgent + Handoff \| Graph \| Crew \| Conversation" | stale | Handoff is a parallel unused surface, not a peer of SubAgent. `HandoffTool::new` has zero callers (V02); only demo examples invoke `HandoffManager::handoff` directly (F-MAG-01-P2-01). |
| `README.md:127` — "`handoff` \| yes \| Agent handoff/collaboration" | stale | The feature compiles but no production agent wires it (V01/V02). |
| `README.md:128` — "`topology` \| yes \| Multi-agent topology tracking" | stale | The feature compiles but no production agent registers `TopologyCallback` and no renderer consumes the output (V01). The recorded graph is structurally incomplete (F-MAG-01-P3-03/P3-04). |
| `README.md:207` — "Agent Handoff \| Context-aware transfer between agents \| `HandoffManager::new()`" | stale | `HandoffResult` is unstructured, history is flattened to free text, and no `SubagentEvent` is emitted (V03). The transfer is detached and non-cancellable (F-MAG-01-P2-02). |
| `handoff/mod.rs:261` — "Use spawn to avoid holding the lock and blocking other handoff requests during execute" | stale | `agents` is `HashMap`, not a locked map; there is no lock to hold (V04). The comment justifies a detach with a non-existent constraint. |
| `topology.rs:1-4` — "Records call relationships between Agents at runtime and supports export to DOT (Graphviz) and Mermaid formats" | current (capability) / stale (integration) | The recorder and exporters work as advertised in isolation (11 tests pass), but the produced graph is misclassified and silent on lock errors, and is never wired in production (F-MAG-01-P3-01 through P3-04). |
| `CHANGELOG.md:160` — "Multi-agent orchestration (SubAgent, Handoff, Plan-and-Execute, Self-Reflection)" | partial drift | SubAgent is live; Handoff is example-only (F-MAG-01-P2-01). The two are listed as peers. |
| `Cargo.toml:74-75` — `handoff = []` / `topology = []` feature definitions | current | Both features compile standalone (V04 executable checks). The features exist; they are simply not consumed by echo-agent-cli. |
| `lib.rs:295-325` — `#[cfg(feature = "handoff")]` / `#[cfg(feature = "topology")]` re-exports | current | Re-exports compile and surface the types through the prelude. The types are then unused by echo-agent-cli. |
| AGENTS.md — "Only Subagent, no Worker" | current | Zero `Worker`/`worker_` hits in `echo-agent/src/handoff/` or `echo-agent/src/topology.rs` (V01). |
| F-SUB-01 handoff — "no parallel worker/subagent identity layer exists" | partially stale (scope-limited) | True *inside* `echo-agent/src/agent/subagent/`. This task extends the search to the rest of the framework root crate and finds the parallel layer (`HandoffManager`) lives in `echo-agent/src/handoff/`. |
| F-SUB-02 handoff — "handoff / topology multi-agent coordination APIs → F-MAG-01" | current (deference) | This task is the deferred F-MAG-01; the deferral is discharged here. |

## Coverage And Uncertainty

Inspected in full: `echo-agent/src/handoff/mod.rs` (391 lines),
`echo-agent/src/handoff/tool.rs` (114 lines),
`echo-agent/src/topology.rs` (611 lines),
`echo-agent/src/lib.rs:295-325` (re-exports),
`echo-agent/Cargo.toml:74-81` (feature definitions),
`echo-agent/examples/demo21_handoff.rs` and `demo47_enterprise.rs`
(handoff-relevant sections), `echo-agent/examples/demo24_topology.rs`
(in full).

Inspected partially (relevant slices only):
- `echo-agent/src/agent/subagent/types.rs:130-381` — `SubagentDefinition`
  and `SubagentOutcome` surface for the divergence comparison (full
  reading owned by F-SUB-01).
- `echo-agent/src/agent/subagent/registry.rs:72-244` — `SubagentRegistry`
  registration entry points for the parallel-authority comparison.
- `echo-agent/src/agent/subagent/context.rs:229-281` —
  `SubagentContext::from_parent` for the context-preservation
  comparison.
- `echo-agent/src/tools/builtin/agent_dispatch.rs:33-55, 360` —
  `AgentDispatchTool` surface for the LLM-tool comparison.
- `echo-agent/echo-core/src/agent/mod.rs:533, 920-988` — `Agent::execute`
  signature and `AgentCallback` trait that topology hooks.

Not inspected (out of scope):
- The application-layer (EKO) agent construction in `echo-agent-cli`
  beyond the V01 census confirming zero handoff/topology references.
- The Tauri bridge / web-frontend — there is no consumer of
  `to_mermaid`/`to_dot`/`to_json` to inspect.
- The `a2a` module — also a multi-agent coordination surface, but
  out-of-scope for F-MAG-01 (handoff + topology only). The
  `NodeType::External` variant and the demo24 label "远程 A2A Agent"
  suggest a loose coupling, but no production code path was inspected.

Environmental constraints:
- All 11 `handoff::*` and `topology::*` tests pass under
  `--features handoff,topology`. Worktree state clean (commit
  `9b0e0fa`).
- Both features compile standalone (`--no-default-features --features
  handoff` / `--features topology`). `cargo fmt --all -- --check` is
  clean.
- No probe was added or removed — all validations are read-only or use
  pre-existing tests.

Uncertain claims:
- Whether any external (out-of-repo) `echo-agent` consumer constructs
  `HandoffManager` or `TopologyTracker`. Per AGENTS.md framework-API
  retention, these pub APIs might have unknown consumers. The findings
  are framed as "dead within this workspace + the canonical
  alternative already exists" + "README marketing is stale," with the
  AGENTS.md retention default noted. The decision to delete vs re-wire
  vs retain-as-menu is left to a follow-up action.
- Whether the "control-yield with return" semantic that
  `HandoffResult.return_to_source` hints at was ever a real product
  requirement. No evidence of one was found; the field is treated as
  dead.
- Whether the topology module's structural gaps
  (F-MAG-01-P3-02/P3-03/P3-04) are defects to fix or simply symptoms
  of a never-finished feature. Either reading supports the
  delete-or-rewire direction; neither supports silent retention with
  the README's current marketing.

## Handoff

Conclusions downstream tasks may rely on:

1. **A parallel identity/dispatch authority exists in the framework
   root crate, outside the subagent module.** `HandoffManager` +
   `HandoffTool` duplicate the canonical `SubagentRegistry` +
   `AgentDispatchTool` path with an incompatible, thinner API. They
   are not bridged. Any downstream task that assumes "one identity
   authority for named-agent delegation across the whole framework"
   should be disabused: there are two, but only one
   (`SubagentRegistry`) is live.
2. **The handoff LLM tool is unreachable.** `HandoffTool::new` has
   zero callers. No production agent exposes a `handoff` tool to its
   LLM. The only path to handoff behaviour is direct
   `HandoffManager::handoff` calls in two example demos.
3. **Handoff is not cancellable and not context-preserving.**
   `tokio::spawn` detaches the agent execution from the caller's
   cancellation domain (same shape as F-SUB-02-P1-02). The result is
   an unstructured plain string. `SubagentEvent` is never emitted.
4. **Topology is observational and structurally incomplete.** It hooks
   `AgentCallback`, not `SubagentEvent`. The callback hardcodes
   `NodeType::{Subagent, Tool}`, misclassifying orchestrators and
   flattening agent-to-agent dispatch into tool-call edges. RwLock
   errors are silently swallowed. No production renderer consumes the
   output.
5. **No `Worker`/`worker_` terminology in either module.** AGENTS.md
   Subagent-only rule is respected.
6. **README/CHANGELOG marketing is stale.** "SubAgent + Handoff" is
   presented as a peer pair; in code, only SubAgent is live. "Multi-
   agent topology tracking" is advertised but unwired.

Reports they must read:

- This report (F-MAG-01) for the handoff/topology vs subagent-stack
  divergence and the parallel-authority finding.
- `tasks/F-SUB-01.md` for the canonical identity/registry/result
  contract that handoff diverges from.
- `tasks/F-SUB-02.md` for the spawn-detach anti-pattern (F-SUB-02-P1-02)
  that `HandoffManager::handoff` repeats, and for the canonical
  `CancellationToken::child_token()` propagation pattern that handoff
  lacks.
- `validations/F-MAG-01/V01-01.md` through `V04-01.md` for per-claim
  evidence and the executable test/check/fmt results.

Conditions that make this report stale:

- Deleting the `handoff` module + feature — resolves F-MAG-01-P2-01,
  F-MAG-01-P2-02, F-MAG-01-P2-03; requires re-running V01/V02.
- Re-wiring `HandoffManager` on top of `SubagentRegistry` and
  `SubagentOutcome` — resolves F-MAG-01-P2-01/P2-03; the spawn-detach
  (F-MAG-01-P2-02) would also need addressing; requires re-running
  V01/V02/V03.
- Deleting the `topology` module + feature — resolves F-MAG-01-P3-01
  through P3-04; requires re-running V01/V04.
- Adding `tracing::warn!` to topology lock-failure branches — resolves
  F-MAG-01-P3-02; requires re-running V04.
- Bridging `SubagentEvent` into `TopologyTracker` and auto-registering
  `TopologyCallback` on dispatched subagents — resolves F-MAG-01-P3-04
  (and partially P3-03); requires re-running V04.
- Updating README/CHANGELOG to drop "Handoff" from the multi-agent
  line and "topology tracking" from the feature list (or to qualify
  them) — resolves the stale Historical Claim Status entries.
- Any change that constructs `HandoffTool::new` in production or
  registers `TopologyCallback` by default in `ReactAgentBuilder` —
  invalidates the "zero production consumers" census; requires
  re-running V01/V02.

Follow-up task IDs (no implementation in this review):

- **A cleanup task** should decide, per AGENTS.md's framework-API
  retention rule, whether to delete the handoff and topology modules
  (recommended: no consumer, no integration with the canonical
  lifecycle, structurally incomplete output) or to re-wire them onto
  the `SubagentRegistry` / `SubagentEvent` / `CancellationToken`
  stack. Either way, the README/CHANGELOG marketing must be
  reconciled. The decision blocks on whether a concrete (in- or
  out-of-repo) consumer needs either surface as a "framework
  capability menu" item; none were found in this workspace.
- **A framework-robustness task** (paired with the F-SUB-02 Team-mode
  fixes) should address the spawn-detach primitive at
  `handoff/mod.rs:265-269` if handoff is kept. The same task could
  standardize the "cloned Arc + inline await" pattern as the
  framework's borrow-decoupling idiom in place of `tokio::spawn`.
- A hooks/integration task could bridge `SubagentEvent` into
  `TopologyTracker` if topology is kept, so the topology graph
  reflects inter-agent dispatch edges rather than only single-level
  tool calls.
