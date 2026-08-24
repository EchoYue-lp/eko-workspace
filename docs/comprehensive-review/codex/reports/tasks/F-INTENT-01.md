# F-INTENT-01: Intent classification and supervisory routing

> Status: complete
> Reviewer: Codex primary reviewer
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean; only Codex review reports changed

## Question

Are intent classification and trigger supervision generic, explainable, bounded,
mode-equivalent, and separate from runtime state authority?

## Scope

- `echo-agent/src/intent/{mod,classifier,trigger_supervisor}.rs`.
- ReactAgent builder/storage and streaming/non-streaming entry integration.
- Hook activation handoff, current skill activation contract and typed events.
- EKO bootstrap/pool/skill-refresh inspection only to prove the real consumer
  and framework contract.
- Existing tests, scoped history, panic/UTF-8/overflow inspection.

## Out Of Scope

- Skill discovery/loading internals: F-SKL-01.
- Generic prompt mutation/compression: F-RCT-01/F-CMP-01.
- Provider wire behavior and retry: F-LLM/F-REL tasks.
- EKO product-specific skill taxonomy or routing policy.
- Source fixes, builds, tests, dynamic fixtures, or network calls.

## Inputs

- Root AGENTS instructions, shared review protocol/task card, Codex rules.
- Accepted B-REF-01 and F-RCT-01 dependency conclusions.
- Current source and narrowly scoped Git history only; no other reviewer
  directory or conclusion was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Typed classifier/router interface, bounded decision context, catalog-fenced skill resolution, deterministic ambiguity, cancellation/deadline, and correlated decision facts belong in the framework. |
| EKO product policy | Which skills/triggers are installed, direct-answer policy, selected classifier model, confidence thresholds and reload timing remain application policy. |
| Adapter boundary | EKO supplies a source-scoped current catalog and policy; the framework returns one typed decision and activation outcome. It must not snapshot application descriptors forever or own TaskRun/ReAct state. |
| Duplicate search | Searched classifier/router/supervisor/trigger/skill activation across both repos. One router exists, but hook preparation is duplicated across streaming entry and shared core. |
| Migration deletion | Preserve the small three-variant intent surface and stream/non-stream convergence. Replace the global optional hook slot and immutable catalog snapshots; remove duplicate streaming hook execution and obsolete ChainedClassifier only if no reasonable external consumer remains. |

The capability is reusable even though its real policy inputs come from EKO. No
finding recommends moving product skill taxonomy into the framework.

## Current Path

```text
EKO startup
  -> snapshot primary SkillDescriptor list
  -> KeywordClassifier + optional LlmIntentClassifier
  -> TriggerSupervisor(keyword -> LLM -> shared hook slot)
  -> IntentRouter(threshold/options)
  -> ReactAgent.set_intent_router

turn entry
  -> context/memory + UserPromptSubmit hook (streaming only)
  -> router.classify (no invocation/cancel/deadline/catalog revision)
  -> DirectAnswer OR activate_skill OR normal ReAct
  -> shared run_core_loop
  -> UserPromptSubmit hook (all ordinary paths)
```

Positive evidence: the Intent model has only DirectAnswer, SkillRequired, and
Fallback after the dead WorkflowRequired branch was deleted. Streaming and
non-streaming both invoke the same router and converge on the same core ReAct
loop. Invalid LLM JSON and provider errors safely fall back. Skill activation
uses a replaceable protected context projection rather than a second store.

## Findings

### F-INTENT-01-P1-01: Hook activation fallback is rejected by its own router and can leak across turns

- Priority: P1
- Confidence: high
- Layer: framework contract / EKO wiring
- Evidence: `echo-agent/src/agent/react/run/context.rs:463`, `:547`;
  `echo-agent/src/intent/trigger_supervisor.rs:48`, `:75`, `:87`;
  `echo-agent/src/intent/mod.rs:136`;
  `echo-agent-cli/echo-agent-app-core/src/runtime.rs:340`
- Reachability: UserPromptSubmit requests a skill -> direct activation fails ->
  request is cached -> live TriggerSupervisor/router classification.
- Expected invariant: a current-turn hook recommendation is either activated or
  returns a visible rejection, and its slot is consumed on every decision path.
- Observed behavior: supervisor assigns hook decisions confidence 0.6 while the
  live router rejects below 0.7. High keyword/LLM decisions return before
  consuming the shared slot, allowing a later fallback turn to see stale intent.
- Impact: the advertised fallback cannot work, and a skill request can be
  attributed to the wrong user turn.
- Root cause: producer/supervisor/router own independent confidence and lifetime
  semantics around an unscoped `Mutex<Option<...>>`.
- Direction: carry hook results in a turn-scoped typed decision input, consume
  once regardless of selected source, and apply one configured acceptance rule.
- Regression validation: failed activation at turn N followed by high-confidence
  and fallback turns; no stale activation and exact reason/source preserved.
- Validation reports: [V02](../validations/F-INTENT-01/V02-01.md)

### F-INTENT-01-P1-02: Pre-routing LLM classification ignores invocation cancellation and has no bounded deadline

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/intent/mod.rs:69`, `:124`;
  `echo-agent/src/intent/classifier.rs:400`;
  `echo-agent/echo-core/src/llm/mod.rs:74`;
  `echo-agent/src/agent/react/run/stream_channel.rs:185`;
  `echo-agent/src/agent/react/run/react_loop.rs:623`
- Reachability: every unmatched turn in EKO with an LLM client awaits
  LlmIntentClassifier before entering the ReAct loop.
- Expected invariant: invocation cancellation/deadline bounds all pre-routing
  work, with timeout/cancel producing deterministic fallback or cancellation.
- Observed behavior: classifier inputs contain only text/history. `chat_simple`
  constructs a ChatRequest without a cancel token; router has no timeout. Both
  entries install external cancellation only after classification.
- Impact: cancelled work can remain blocked on an extra model request before the
  actual agent request begins, harming all interaction modes and shutdown.
- Root cause: classification predates value-carried invocation context.
- Direction: add a typed decision context with cancel/deadline/catalog revision;
  select on it and make classification timeout a bounded fallback policy.
- Regression validation: non-responsive classifier with cancellation/deadline in
  streaming and non-streaming entries; no late activation or second LLM request.
- Validation reports: [V03](../validations/F-INTENT-01/V03-01.md)

### F-INTENT-01-P1-03: SkillRequired is not fenced to the available catalog and activation failure is silent

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/intent/classifier.rs:354`;
  `echo-agent/src/intent/mod.rs:136`;
  `echo-agent/src/agent/react/capabilities.rs:964`;
  `echo-agent/src/agent/react/run/react_loop.rs:664`;
  `echo-agent/src/agent/react/run/stream_channel.rs:254`
- Reachability: LLM classifier output flows directly into both entry points.
- Expected invariant: accepted skill names resolve against the exact catalog
  revision used for classification and produce a typed activation outcome.
- Observed behavior: any non-null string is accepted. Router checks confidence
  only. Uninstalled activation returns `Ok(())`; other errors are logged and the
  ordinary loop continues without a decision fact.
- Impact: the system can claim to route to a capability that never activated,
  making behavior and debugging dependent on hidden logs.
- Root cause: free-form model output is treated as executable identity.
- Direction: parse into a catalog-issued stable identity/revision, reject unknown
  values to Fallback, and emit activated/rejected/failed outcome.
- Regression validation: unknown, unloaded, replaced, duplicate-name and
  activation-error cases across both entries.
- Validation reports: [V04](../validations/F-INTENT-01/V04-01.md)

### F-INTENT-01-P2-04: Equal-score keyword routing is nondeterministic and duplicate triggers silently overwrite

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/intent/classifier.rs:73`, `:109`, `:118`
- Reachability: all product descriptor triggers are loaded into the live keyword
  classifier; overlap is expected in a large user-extensible skill catalog.
- Expected invariant: conflicts are rejected, explicitly prioritized, or return
  explainable ambiguity independent of collection seed/discovery order.
- Observed behavior: one trigger maps to one silently replacing skill. Aggregate
  ties sort by score only from HashMap iteration and may remain above the hard
  internal 0.7 threshold, arbitrarily selecting a winner.
- Impact: identical input/configuration can activate different skills across
  runs or load order.
- Root cause: registration and ranking omit source identity/stable tie policy.
- Direction: retain all owners with source/priority, sort deterministically, and
  make equal top scores an ambiguity/Fallback unless policy resolves them.
- Regression validation: duplicate trigger, equal/multiple scores and different
  insertion/hash seeds with exact stable result.
- Validation reports: [V05](../validations/F-INTENT-01/V05-01.md)

### F-INTENT-01-P1-05: Router catalogs are immutable startup snapshots and pooled agents lack equivalent routing

- Priority: P1
- Confidence: high
- Layer: framework/application adapter
- Evidence: `echo-agent-cli/echo-agent-app-core/src/runtime.rs:294`, `:319`;
  `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:493`, `:934`;
  `echo-agent-cli/src/tauri/commands/panels.rs:447`
- Reachability: EKO constructs the primary router once, while GUI skill changes
  refresh descriptors in existing/future pooled Agents.
- Expected invariant: classification and activation consume the same current
  source-scoped catalog; mode-equivalent primary/pooled agents expose the same
  routing capability.
- Observed behavior: keyword/LLM descriptor lists are cloned once and never
  rebuilt on refresh. Pool refresh registers descriptors only, and pooled Agent
  construction has no IntentRouter installation.
- Impact: removed skills remain classifiable, added skills remain invisible, and
  otherwise equivalent EKO execution modes make different routing decisions.
- Root cause: application owns two descriptor snapshots without a revisioned
  classifier adapter shared by agent construction/reload.
- Direction: framework router should resolve one injected live catalog snapshot;
  EKO must install the same policy for every primary/pooled mode and reload it
  atomically.
- Regression validation: add/remove/replace a skill after startup and compare
  primary, pooled, streaming and non-streaming outcomes.
- Validation reports: [V06](../validations/F-INTENT-01/V06-01.md)

### F-INTENT-01-P2-06: Intent decisions have no typed correlated observability

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/intent/mod.rs:33`;
  `echo-agent/src/agent/react/run/react_loop.rs:627`;
  `echo-agent/src/agent/react/run/stream_channel.rs:189`
- Reachability: every routed turn emits only tracing text at the decision sites.
- Expected invariant: applications can observe a bounded decision fact with
  invocation identity, source, action, confidence, catalog revision and terminal
  activation result without parsing logs.
- Observed behavior: Intent carries action/name/confidence only. AgentEvent,
  trace and audit contain no intent decision; source competition, hook reason,
  revision and activation result disappear.
- Impact: routing regressions cannot be reliably correlated or explained to
  GUI/TUI/CLI consumers and offline evaluation.
- Root cause: classification was integrated as a log-only shortcut.
- Direction: emit one typed event/trace fact; do not persist hidden model
  reasoning or create a second runtime state machine.
- Regression validation: each intent/source/result with stable invocation ID and
  exactly one event in both entries.
- Validation reports: [V07](../validations/F-INTENT-01/V07-01.md)

### F-INTENT-01-P1-07: Streaming ordinary turns execute UserPromptSubmit hooks twice

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/react/run/context.rs:541`, `:610`;
  `echo-agent/src/agent/react/run/stream_channel.rs:124`, `:302`, `:509`;
  `echo-agent/src/agent/react/run/phases/prepare.rs:57`;
  `echo-agent/src/agent/react/run/react_loop.rs:508`
- Reachability: every streaming SkillRequired/Fallback turn prepares context,
  classifies, then enters the shared core. GUI/TUI commonly use this path.
- Expected invariant: a lifecycle hook executes exactly once per turn and every
  interaction mode observes the same side effects/result.
- Observed behavior: streaming preparation executes UserPromptSubmit once before
  routing; `run_core_loop -> prepare_turn` executes it again. Non-streaming runs
  only the shared core hook. The second activation result is not routed.
- Impact: injected messages, external hook actions, logging and other side
  effects can duplicate only in streaming modes, violating mode parity.
- Root cause: pre-routing hook needs were added to the entry without moving the
  canonical prepare hook or passing its result forward.
- Direction: establish one shared pre-classification preparation result and pass
  it into core preparation; delete the second invocation.
- Regression validation: counting/side-effect/block/inject/activate hooks in
  DirectAnswer, SkillRequired and Fallback for both entries.
- Validation reports: [V12](../validations/F-INTENT-01/V12-01.md)

## Validation Matrix

| ID | Claim | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition/export/registration/reachability map | yes | passed | [report](../validations/F-INTENT-01/V01-01.md) |
| V02 | Hook fusion threshold and turn ownership | yes | failed | [report](../validations/F-INTENT-01/V02-01.md) |
| V03 | Classification cancellation and timeout | yes | failed | [report](../validations/F-INTENT-01/V03-01.md) |
| V04 | Skill identity and activation outcome | yes | failed | [report](../validations/F-INTENT-01/V04-01.md) |
| V05 | Keyword collision and deterministic ranking | yes | failed | [report](../validations/F-INTENT-01/V05-01.md) |
| V06 | Catalog reload and pooled/mode parity | yes | failed | [report](../validations/F-INTENT-01/V06-01.md) |
| V07 | Typed correlated observability | yes | failed | [report](../validations/F-INTENT-01/V07-01.md) |
| V08 | Existing test coverage inventory | yes | passed | [report](../validations/F-INTENT-01/V08-01.md) |
| V09 | Panic/UTF-8/overflow inspection | yes | passed | [report](../validations/F-INTENT-01/V09-01.md) |
| V10 | Authorized historical drift | yes | passed | [report](../validations/F-INTENT-01/V10-01.md) |
| V11 | Future executable regression matrix | deferred | not_run | [report](../validations/F-INTENT-01/V11-01.md) |
| V12 | Streaming duplicate UserPromptSubmit hook | yes | failed | [report](../validations/F-INTENT-01/V12-01.md) |
| V99-01 | Final gate harness with zsh PATH collision | yes | inconclusive | [report](../validations/F-INTENT-01/V99-01.md) |
| V99-02 | Corrected shell variable but pre-file self-link failed | yes | inconclusive | [report](../validations/F-INTENT-01/V99-02.md) |
| V99-03 | Final link/header/isolation/source gate | yes | passed | [report](../validations/F-INTENT-01/V99-03.md) |

## Coverage And Uncertainty

- No build/test/dynamic/network command ran. V11 records future execution only.
- Static findings are directly source-conclusive. Runtime timing and provider
  behavior remain remediation regressions, not blockers for this review.
- DirectAnswer/core terminal defects are owned by F-RCT-02/F-RCT-03 and not
  duplicated. Skill registry internals and budgets remain F-SKL-01.
- The LLM prompt says weather/time may be DirectAnswer despite no tools; this is
  policy-sensitive and was not promoted without a product freshness contract.
- ChainedClassifier remains a reasonable public composition option despite no
  live EKO use; no deletion finding is based on application non-use.

## Handoff

- Fix order: one shared prepare/hook result -> turn-scoped decision context with
  cancel/deadline -> catalog-fenced identity/revision -> deterministic conflicts
  -> typed decision event -> live EKO primary/pool refresh.
- Preserve the three-variant Intent surface, protected activation projection,
  invalid-response Fallback, and stream/non-stream router convergence.
- F-SKL-01 and application mode-parity synthesis must consume P1-03/P1-05/P1-07.
- This report becomes stale if intent traits/router/supervisor, entry preparation,
  activation, skill refresh, or decision event contracts change.
