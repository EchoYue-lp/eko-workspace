# F-INTENT-01: Intent classification and supervisory routing

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean

## Question

Are intent classification and trigger supervision generic, explainable,
bounded, and separate from runtime state authority?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/src/intent/mod.rs` (166 lines) — `Intent` enum (3 variants),
  `IntentClassifier` trait, `IntentRouterConfig`, `IntentRouter` struct and
  its threshold/gating `classify` wrapper.
- `echo-agent/src/intent/classifier.rs` (903 lines) — `KeywordClassifier`
  (weighted keyword scoring, 0 tokens), `LlmIntentClassifier` (LLM JSON
  prompt + response parser), `ChainedClassifier`, `KeywordClassifierConfig`,
  `SkillDescription`, and the 18 unit tests.
- `echo-agent/src/intent/trigger_supervisor.rs` (157 lines) —
  `TriggerSupervisor`: three-source fusion (keyword → LLM → hook slot) and
  its 5 fusion-rule unit tests.
- `echo-agent/src/agent/react/mod.rs:225-258, 624-629, 977-982, 1490-1492` —
  `ReactAgent::intent_router` field, `hook_activation_cache` slot, the
  `allows_direct_answer_shortcut` projector-gate, `set_intent_router` /
  `hook_activation_cache` accessors.
- `echo-agent/src/agent/react/builder.rs:103-104, 179, 614-631, 1038-1040` —
  builder field, default, `intent_router()` setter, and the `build()` direct
  write to `agent.intent_router` (a Category-C bypass per F-RCT-01-P3-03).
- `echo-agent/src/agent/react/run/react_loop.rs:600-751` — non-streaming
  `run_chat_direct` intent gate (DirectAnswer shortcut, SkillRequired
  activation, Fallback fall-through) and `direct_answer()` helper.
- `echo-agent/src/agent/react/run/stream_channel.rs:124-280, 794-820,
  1070-1092, 1230-1352` — streaming `run_stream_channel` intent gate
  (converged structure), the `AlwaysDirectClassifier`/`RoutingProjection`
  test fixtures, and the two projection-boundary parity tests.
- `echo-agent/src/agent/react/run/context.rs:305, 444-555, 600-623` —
  `fire_lifecycle_hook` (UserPromptSubmit → ActivateSkill direct activation
  + clear-on-success) and the two `prepare_*_context` writers of the
  hook activation cache.
- `echo-agent/src/agent/react/capabilities.rs:960-997` — `activate_skill`,
  the single skill-activation authority the router reuses.
- `echo-agent/src/agent/react/tests.rs:1223-1285` —
  `intent_router_skill_activation_survives_compression_markers`.
- `echo-agent-cli/echo-agent-app-core/src/runtime.rs:25-35, 294-356` — EKO
  bootstrap: populates `KeywordClassifier` from skill descriptors, constructs
  `TriggerSupervisor` (keyword + optional LLM + hook cache), wires
  `IntentRouter` onto the agent.
- `echo-agent/echo-core/src/llm/mod.rs:20-80` — `LlmClient::chat_simple`
  contract (no timeout in `SimpleChatOptions`).
- `echo-agent/echo-integration/src/providers/openai.rs:256-258, 472-509` —
  provider `chat_simple` and its 120 s reqwest client timeout (the only
  bound on a hung classification call).

## Out Of Scope

Deferred to named task IDs:

- Skill discovery, `SkillRegistry` internals, and progressive-disclosure
  registry → **F-SKL-01**. This task only confirms the router reuses the
  single `activate_skill` authority and does not parallelize it.
- `PreModelContextProjector` semantics and the `allows_direct_answer_shortcut`
  gate's effect on task-context injection → **F-CTX-01**. This task only
  confirms the gate is consulted and the two projection-boundary tests pass.
- Hook system internals (`HookRegistry`, `HookEvent::UserPromptSubmit`,
  `ActivateSkill` action) → the hooks sub-task of **F-PLG-01** / **A-PLG-01**.
  This task only traces the `activate_skill` field handoff into the cache.
- The `LlmClient` provider timeouts (stream first-chunk/idle/overall) and
  retry/circuit-breaker → **F-LLM-01/02/03** and **F-REL-01**. This task
  only notes the absence of an intent-layer timeout.
- TaskRun / PlanTask / SubagentRun runtime state authority → **F-TSK-01** /
  **F-SUB-01**. This task only confirms the router does not touch them.

## Inputs

Required repository documents read:

- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/AGENTS.md` (in full, via
  system reminder — especially the framework-vs-application layering rule,
  the "first check if it already exists" rule, the "only Subagents, not
  Workers" terminology rule, and the TUI/GUI/CLI feature-parity mandate).
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/REPORTING.md`
  (in full).
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/templates/task-report.md`
  and `templates/validation-report.md` (in full).
- `docs/comprehensive-review/TASKS.md` F-INTENT-01 task card (lines 444-452).

Dependency task reports read:

- `docs/comprehensive-review/zcode-glm/tasks/F-RCT-01.md` (in full).
  F-RCT-01 establishes the construction path: `intent_router` is wired as a
  Category-C bypass in `build()` (F-RCT-01-P3-03, builder.rs:1038-1040
  writes directly to `agent.intent_router` with no `AgentConfig` field).
  This report confirms that bypass is benign for the router (the field is
  initialised to `None` in `new_inner`, mod.rs:571) and traces the runtime
  reachability F-RCT-01 did not own.
- `docs/comprehensive-review/zcode-glm/tasks/B-REF-01.md` (in full).
  B-REF-01's convergence C1 (plan is an artifact, not a runtime approval
  state machine) and C4 (permission is a launch-time mode) frame the
  layering check: intent routing must remain a pre-loop classification
  hint, not a runtime authority. This report confirms it does not introduce
  a parallel plan/subagent/skill authority.

Historical documents treated as hypotheses:

- `echo-agent/src/intent/mod.rs` module docstring (lines 1-25) claims three
  intents (`DirectAnswer`/`SkillRequired`/`Fallback`) and that the router is
  placed "at the ReAct loop entry". Confirmed current by V01/V02.
- `echo-agent/src/intent/classifier.rs:26-29` claims "The classifier contains
  zero hardcoded keywords." Confirmed current by V02 — `KeywordClassifier::
  default()` is empty and the `test_keyword_classifier_empty_returns_fallback`
  test asserts every input maps to `Fallback` when no keywords are registered.
- `echo-agent/src/intent/trigger_supervisor.rs:42-47` documents the fusion
  rule order. Confirmed current by V02 and the 5 fusion unit tests.

Note: a sibling task **F-INT-01 (MCP integration)** exists in this reviewer's
directory and is unrelated to intent classification. The task IDs F-INT-01
and F-INTENT-01 are distinct; this is the first F-INTENT-01 report.

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | Yes. `Intent`, `IntentClassifier`, `IntentRouter`, `KeywordClassifier`, `LlmIntentClassifier`, `ChainedClassifier`, and `TriggerSupervisor` are generic agent-runtime machinery. Any `echo-agent` consumer that wants a pre-loop classification hint can use them, and the framework ships the classifiers with **zero hardcoded keywords** (`KeywordClassifier::default()` is empty — V02). Lives correctly in `echo-agent` (root crate). |
| EKO product policy | The product layer (EKO) supplies the actual triggers: `runtime.rs:298-313` reads `skill_descriptors()` and populates `KeywordClassifier` + `SkillDescription` from each skill's frontmatter `triggers`. EKO also decides the thresholds (`confidence_threshold: 0.7`), constructs the `TriggerSupervisor`, and wires the shared `hook_activation_cache`. All product policy is correctly in `echo-agent-app-core`, not the framework. |
| Adapter boundary | `TriggerSupervisor` is a thin fusion adapter over three framework classifiers plus a shared `Arc<Mutex<Option<_>>>` slot. It owns no scheduler, registry, DAG, or state machine — it only fuses three `Intent` producers and applies a fixed priority. `IntentRouter` is likewise a thin threshold/gating wrapper. Clean. |
| Duplicate search | Searched names: `Intent`, `IntentClassifier`, `IntentRouter`, `IntentRouterConfig`, `KeywordClassifier`, `LlmIntentClassifier`, `ChainedClassifier`, `TriggerSupervisor`, `SkillDescription`, `activate_skill`, `hook_activation_cache`, `set_intent_router`, `intent_router`, `allows_direct_answer_shortcut`. Searched behaviours: intent classification, skill activation, direct-answer shortcut, hook-slot handoff. Result: one canonical definition per concept (V01); the router's `SkillRequired` arm reuses the **single** `activate_skill` → `SkillRegistry` path (V04), creating no parallel authority to `SubagentRegistry` or `SkillRegistry`. |
| Migration deletion | No migration proposed. No parallel implementation exists to delete. |

## Current Path

Verified intent-classification data flow at commit `9b0e0fa`:

```text
[Product layer — echo-agent-cli/runtime.rs:294-356]
  KeywordClassifier::new()
    ← add_skill_keywords(name, desc.triggers)        [runtime.rs:300-301]
  SkillDescription { name, description, example_triggers }
    ← from skill_descriptors()                       [runtime.rs:302-307]
  hook_cache = agent.hook_activation_cache()          [runtime.rs:318, mod.rs:980]
  TriggerSupervisor::new(kw, Option<LlmIntentClassifier>, hook_cache)
                                                     [runtime.rs:325]
  IntentRouter::new(Box<supervisor>, config{0.7, true, true})
                                                     [runtime.rs:340-347]
  agent.set_intent_router(router)                    [runtime.rs:351, mod.rs:1490]

[Per-turn — run_chat_direct / run_stream_channel]
  prepare_*_context(message)
    ├─ context.push(Message::user(input))            [context.rs:536/607]
    ├─ fire_lifecycle_hook(UserPromptSubmit, input)  [context.rs:545/613]
    │     └─ on ActivateSkill:
    │          activate_skill(skill) direct          [context.rs:467]
    │          Ok  → result.activate_skill = None    [context.rs:474]
    │          Err → leave for supervisor retry      [context.rs:478]
    └─ if hook_result.activate_skill.is_some():
          hook_activation_cache = activate_skill     [context.rs:548-552/615-619]
             (only written on hook-activation failure)

  if let Some(router) = self.intent_router:          [react_loop.rs:623 / stream_channel.rs:185]
    intent = router.classify(msg, &ctx_msgs).await
      └─ IntentRouter::classify                      [mod.rs:124-153]
           raw = classifier.classify(...)            (TriggerSupervisor)
           apply threshold + enable_* gates → maybe Fallback
      └─ TriggerSupervisor::classify                 [trigger_supervisor.rs:66-95]
           kw = keyword.classify(...)                (0 tokens)
           if kw.confidence() >= 0.7 → return kw     [line 76-78] (slot NOT taken)
           llm = llm_classifier?.classify(...)       (~500 tokens, Err→Fallback)
           hook = hook_activation_slot.take()        [line 87-91]
           fuse(kw, llm, hook)                       [line 93 → fuse():48-62]
    match intent:
      DirectAnswer if allows_direct_answer_shortcut():
           → direct_answer(msg) / direct_answer_stream(...)
           → single LLM call, no tools, return FinalAnswer
           (streaming pushes assistant msg to ctx; non-streaming does NOT — P2-01)
      DirectAnswer else:
           → fall through (pre-model projector present)   [react_loop.rs:657-663]
      SkillRequired { skill_name, .. }:
           → activate_skill(skill_name)              [capabilities.rs:964-997]
           → inject skill as replaceable projection  [capabilities.rs:983-989]
           → fall through to core ReAct loop
      Fallback:
           → fall through to core ReAct loop
```

Key invariants verified by this graph (full evidence in V01-V04):

- **Optional and non-authoritative.** `intent_router: Option<IntentRouter>`
  (`mod.rs:226`) defaults to `None` (`mod.rs:571`). A minimal framework
  agent never classifies and runs the plain ReAct loop. When present, the
  router is a *hint* layer: it can shortcut (DirectAnswer), inject a skill
  projection (SkillRequired), or fall through (Fallback). It cannot spawn
  subagents, create tasks, mutate `TaskRun`/`PlanTask`/`SubagentRun`, or
  bypass the guard/projector.
- **Single skill-activation authority.** The `SkillRequired` arm reuses
  `activate_skill` (the same method the hook system and product runtime
  call), which routes through the single `SkillRegistry`
  (`tool_exec.rs:34`). No parallel skill or subagent registry is created
  (V04).
- **Two converged call sites.** The non-streaming (`run_chat_direct`,
  react_loop.rs:622-682) and streaming (`run_stream_channel`,
  stream_channel.rs:181-280) gates have identical match structure. The
  projection-boundary behaviour is identical (both fall through when
  `allows_direct_answer_shortcut()` is false — V04 confirms via two passing
  parity tests).
- **Confidence-bounded.** `IntentRouterConfig::default()` sets
  `confidence_threshold = 0.7`; the router demotes any sub-threshold
  classification to `Fallback` (mod.rs:127-152). The keyword scorer
  additionally requires `confidence >= 0.7` before emitting `SkillRequired`
  (classifier.rs:141).

## Findings

### F-INTENT-01-P2-01: Non-streaming DirectAnswer does not persist the assistant reply to context; diverges from streaming and breaks multi-turn continuity

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/react/run/react_loop.rs:635-655` — the
    non-streaming DirectAnswer shortcut does `return match self.direct_answer(message).await { Ok(answer) => { finalize_trace_run(...); Ok(answer) } ... }`.
    It returns the answer without ever calling
    `self.memory.context.lock().await.push(Message::assistant(answer))`.
  - `echo-agent/src/agent/react/run/stream_channel.rs:237-242` — the
    streaming DirectAnswer shortcut **does** push, with an explicit
    comment: `// Push assistant message so the agent remembers this turn.`
    followed by `self.memory.context.lock().await.push(Message::assistant(content))`.
  - `echo-agent/src/agent/react/run/context.rs:536, 607` — both prepare
    paths push the user message before classification, so after a
    non-streaming DirectAnswer turn the context holds `[..., user_msg]`
    with no paired assistant message.
- Reachability: any caller of the non-streaming entry point
  (`run_chat_direct`, react_loop.rs:590) with an `IntentRouter` that
  returns `DirectAnswer` while `allows_direct_answer_shortcut()` is true
  (i.e. no `PreModelContextProjector` installed). The streaming entry
  point (`run_stream_channel`) is unaffected.
- Expected invariant: per AGENTS.md's TUI/GUI/CLI feature-parity rule,
  the streaming and non-streaming DirectAnswer paths must have identical
  memory-continuity semantics. A turn's assistant reply must be visible
  to the next turn's context.
- Observed behavior: after a non-streaming DirectAnswer turn the
  assistant reply is absent from `memory.context`; the next turn's
  context starts with an unanswered user message, producing two
  consecutive user messages with no assistant between them. The agent
  "forgets" its own direct answers in non-streaming mode.
- Impact: framework consumers using `run_chat_direct` (non-streaming
  callers; some CLI/headless paths) silently lose multi-turn continuity
  for direct-answer turns. EKO's GUI/TUI streaming path is unaffected,
  but the framework contract is violated and the divergence is exactly
  the class of bug AGENTS.md's parity mandate exists to prevent.
- Root cause: when the streaming push was added (stream_channel.rs:237
  comment), the non-streaming twin was not updated. The two gates were
  otherwise converged but this one line diverged.
- Direction: in `run_chat_direct`'s DirectAnswer `Ok` arm
  (react_loop.rs:636-643), before returning, push the assistant message:
  `self.memory.context.lock().await.push(Message::assistant(answer.clone()));`
  mirroring stream_channel.rs:238-242. No deletion target.
- Regression validation: a non-streaming test that triggers DirectAnswer
  and asserts the next turn's context contains the prior assistant
  reply. (The existing `non_streaming_direct_answer_routes_through_projection_boundary`
  test at stream_channel.rs:1310 exercises the projector-gated path, not
  the shortcut-with-persistence path — a new test is needed.)
- Validation reports: [V04-01](../validations/F-INTENT-01/V04-01.md)

### F-INTENT-01-P3-01: Hook activation slot is never cleared at turn start; a stale skill can leak across turns when the keyword fast-path fires

- Priority: P3
- Confidence: medium
- Layer: framework
- Evidence:
  - `echo-agent/src/intent/trigger_supervisor.rs:71-78` — the keyword
    fast path: `if kw.confidence().unwrap_or(0.0) >= HIGH_CONFIDENCE { return kw; }`.
    This returns **before** `hook_activation_slot.lock()...take()` at
    lines 87-91, so the slot is not consumed on a keyword fast-path turn.
  - `echo-agent/src/agent/react/run/context.rs:548-552, 615-619` — the
    slot is written **only** when `hook_result.activate_skill.is_some()`,
    which (per context.rs:466-474) is true only when the hook-requested
    skill activation **failed** (success clears the field). The slot is
    never proactively cleared at turn start.
  - `echo-agent/src/agent/react/run/context.rs:474` — on hook-activation
    success `result.activate_skill = None`, so the next turn that
    succeeds (or has no hook request) does not overwrite the slot.
- Reachability: turn N: a `UserPromptSubmit` hook requests skill A,
  activation errors → slot = `(A, reason)`. Same turn's classification
  hits the keyword fast path for a different skill B → returns B, slot
  not taken, still `(A, reason)`. Turn N+1: hook succeeds or has no
  ActivateSkill request → slot not overwritten → still `(A, reason)`.
  Turn N+1's classification reaches the fuse step (keyword low, LLM
  low) → takes slot → activates skill A for turn N+1's input, which had
  nothing to do with A.
- Expected invariant: the hook activation slot should be scoped to
  exactly one turn. A skill requested by a hook on turn N must not be
  activated on turn N+1.
- Observed behavior: the slot is write-on-failure, take-on-fuse, never
  reset. A keyword fast-path turn skips the take, leaving a stale value
  that a later turn can consume.
- Impact: rare mis-activation of an unrelated skill (wrong instructions
  injected as a context projection). Recoverable — the ReAct loop still
  runs and the user can correct — but it is a silent correctness drift.
  Trigger is narrow (requires a failed hook activation + a high-
  confidence keyword match on the same turn + a non-overwriting next
  turn), which is why this is P3 not P2.
- Root cause: the early return optimisation skipped the take, and no
  turn-start clear was added to compensate.
- Direction: clear the slot at the start of each prepare phase
  (`prepare_stream_context` / `prepare_react_context`) unconditionally,
  i.e. `*cache = None;` before firing the hook, so only the current
  turn's hook result can populate it. Alternatively, in
  `TriggerSupervisor::classify`, take the slot **before** the keyword
  fast-path check so it is always consumed.
- Regression validation: a two-turn test where turn N leaves a stale
  slot (keyword fast path + failed hook activation) and turn N+1 asserts
  no skill activation occurs.
- Validation reports: [V03-01](../validations/F-INTENT-01/V03-01.md)

### F-INTENT-01-P3-02: No intent-layer timeout; a hung LLM classification stalls turn entry for the full provider HTTP timeout

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/intent/classifier.rs:409-419` —
    `LlmIntentClassifier::classify` does
    `match llm.chat_simple(messages).await { Ok(response) => ..., Err(e) => { tracing::warn!(...); Intent::Fallback } }`.
    The error path is correct; there is no `tokio::time::timeout` around
    the call.
  - `echo-agent/src/intent/mod.rs:124-153` — `IntentRouter::classify`
    awaits `self.classifier.classify(...)` with no timeout.
  - `grep -rn "timeout\|tokio::time" src/intent/` → zero matches. There
    is no timeout anywhere in the intent module.
  - `echo-agent/echo-core/src/llm/mod.rs:20-50` — `SimpleChatOptions`
    carries only `temperature` and `max_tokens`; no timeout field.
  - `echo-agent/echo-integration/src/providers/openai.rs:256-258` — the
    OpenAI reqwest client is built with `.timeout(Duration::from_secs(120))`,
    which is the only bound on a hung non-streaming `chat_simple`.
- Reachability: any agent with an `LlmIntentClassifier` (EKO constructs
  one whenever an LLM client is available, runtime.rs:320-323) whose
  provider stalls on the classification call. The classification runs
  at the top of `run_chat_direct` / `run_stream_channel`, **before**
  the core loop and its budgets, so no turn-level budget bounds it.
- Expected invariant: a feature billed as a "fast pre-classification"
  (classifier.rs:7, "0 token, fast") should have a tight bound; a hung
  call should degrade to `Fallback` quickly, not stall the turn.
- Observed behavior: on a hung provider call, the turn entry blocks for
  up to the provider's HTTP timeout (120 s for the OpenAI adapter) before
  the error surfaces and `Fallback` is returned. The error→Fallback path
  itself is correct; only the latency bound is absent at the intent layer.
- Impact: a network stall turns a sub-second greeting classification into
  a up-to-120-second hang with no diagnostic at turn entry. Not an
  infinite hang (provider timeout bounds it) and not a correctness bug
  (Fallback is correct), hence P3.
- Root cause: the intent layer was added without its own timeout, relying
  entirely on the provider's HTTP timeout. Provider timeouts vary (120 s
  for OpenAI; other adapters may differ), so the bound is implicit and
  non-uniform.
- Direction: wrap the inner classifier call in `IntentRouter::classify`
  with `tokio::time::timeout(Duration::from_millis(N), ...)` (e.g. 5-10 s)
  and return `Intent::Fallback` on elapse, with a `tracing::warn!`.
  Document the chosen bound in `IntentRouterConfig`.
- Regression validation: a test with a mock `IntentClassifier` that never
  resolves, asserting `IntentRouter::classify` returns `Fallback` within
  the configured bound.
- Validation reports: [V03-01](../validations/F-INTENT-01/V03-01.md)

### F-INTENT-01-P3-03: LLM classifier and hook fusion discard explanation/reason; intent decisions are only partially explainable

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/intent/classifier.rs:330-352` — the LLM classification
    prompt asks only for `{"intent", "skill", "confidence"}` JSON. There
    is no `reason`/`explanation` field requested.
  - `echo-agent/src/intent/classifier.rs:355-397` — `parse_response`
    extracts `intent`/`skill`/`confidence` and discards everything else;
    on any parse failure it returns `Intent::Fallback` with no diagnostic.
  - `echo-agent/src/intent/trigger_supervisor.rs:55-60` — `fuse` adopts a
    hook activation as `Intent::SkillRequired { skill_name, confidence: 0.6 }`
    and destructures the hook's reason as `_reason` (discarded):
    `if let Some((skill_name, _reason)) = hook { ... }`.
  - `echo-agent/src/agent/react/run/react_loop.rs:668-676` /
    `stream_channel.rs:258-270` — the runtime logs only
    `skill = %skill_name, confidence = confidence` on SkillRequired; no
    rationale is recorded.
- Reachability: every classification that goes through `LlmIntentClassifier`
  or the hook-fusion branch.
- Expected invariant: an explainable classifier should record *why* a
  decision was made (matched trigger, LLM rationale, or hook reason) so
  the decision is auditable and debuggable.
- Observed behavior: the keyword path is fully explainable by its
  scoring algorithm (`match_weight`, `classify_inner`), but the LLM path
  discards all rationale and the hook path discards the `reason` string
  that the hook system explicitly populated (context.rs:466 reads
  `(ref skill, ref reason)`).
- Impact: observability/debuggability gap only. When the router
  mis-classifies via the LLM or hook path, there is no trace of why.
  No correctness impact.
- Root cause: the `Intent` enum carries no explanation field; the fusion
  and prompt were written to minimise payload, not to preserve rationale.
- Direction: optionally add an `explanation: Option<String>` to the
  `SkillRequired`/`DirectAnswer` variants (or a side-channel trace field),
  request a short `reason` in the LLM prompt, and propagate the hook
  `reason` instead of `_reason`. Low priority.
- Regression validation: a test asserting the explanation propagates end
  to end.
- Validation reports: [V02-01](../validations/F-INTENT-01/V02-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition/registration/reachability (single definition site + loop reachability) | yes | passed | [V01-01](../validations/F-INTENT-01/V01-01.md) |
| V02 | Label/decision contract (intents, decision mechanism, explainability) | yes | passed | [V02-01](../validations/F-INTENT-01/V02-01.md) |
| V03 | Timeout/fallback behavior (error→Fallback, no intent-layer timeout, slot staleness) | yes | **failed** | [V03-01](../validations/F-INTENT-01/V03-01.md) |
| V04 | Representative routing fixtures + parallel-authority check (`cargo test`) | yes | passed | [V04-01](../validations/F-INTENT-01/V04-01.md) |
| V05 | Historical-document drift | conditional (not applicable — no prior F-INTENT-01 report exists; the module docstrings are first-time hypotheses, classified in Historical Claim Status) | n/a | — |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `intent/mod.rs:1-25` — three intents (`DirectAnswer`/`SkillRequired`/`Fallback`), router placed "at the ReAct loop entry" | current | V01 confirms exactly three variants and both loop entry points (`run_chat_direct`, `run_stream_channel`) consult the router before the core loop. |
| `intent/classifier.rs:26-29` — "The classifier contains zero hardcoded keywords" | current | V02: `KeywordClassifier::default()` is empty; `test_keyword_classifier_empty_returns_fallback` asserts all inputs → Fallback. The 11-skill keyword table lives only in a unit test (`test_11_skill_trigger_routing`), not in shipped code. |
| `intent/trigger_supervisor.rs:42-47` — documented fusion rule order (keyword high → LLM high → hook → Fallback) | current | V02 confirms via the 5 fusion unit tests and the `fuse` implementation; one side effect (slot not taken on keyword fast path) is recorded as F-INTENT-01-P3-01. |
| `intent/mod.rs:101-107` — router "shortcuts to a direct answer, activates a skill, or proceeds with the standard ReAct loop" | current | V04 confirms all three arms and that skill activation reuses the single `activate_skill` authority. |
| AGENTS.md: TUI/GUI/CLI feature parity | **violated (localized)** | F-INTENT-01-P2-01: the non-streaming DirectAnswer path does not persist the assistant reply, diverging from the streaming path. |

## Coverage And Uncertainty

Inspected in full: `intent/mod.rs`, `intent/classifier.rs`,
`intent/trigger_supervisor.rs`, `react_loop.rs:600-751`,
`stream_channel.rs:124-280, 794-820, 1070-1092, 1230-1352`,
`context.rs:305, 444-555, 600-623`, `capabilities.rs:960-997`,
`mod.rs:225-258, 624-629, 977-982, 1490-1492`,
`builder.rs:103-104, 179, 614-631, 1038-1040`,
`runtime.rs:290-356`, `llm/mod.rs:20-80`,
`openai.rs:256-258, 472-509`.

Not inspected (out of scope or deferred):

- `direct_answer` / `direct_answer_stream` token-accounting and trace-
  recording tails beyond the persistence check (react_loop.rs:754-800+,
  stream_channel.rs:351+) — overlap with F-RCT-03 streaming accounting.
- `SkillRegistry::activate` internals (what makes activation error) —
  F-SKL-01 owns this. This task only relies on the fact that
  `activate_skill` returns `Result` and can error.
- Other provider adapters' (Anthropic, DeepSeek) `chat_simple` timeouts —
  F-LLM-02/03 own these. The P3-02 finding anchors on the OpenAI adapter's
  120 s bound; other adapters may differ, which is part of the finding's
  "non-uniform bound" point.

Environmental constraints:

- All tests run against the pre-built `target/`; worktree was clean
  (`git status` empty) before and after. No code was modified.
- `cargo test -p echo_agent --lib intent::` and
  `cargo test -p echo_agent --lib stream_channel::tests::` both green
  (18 + 23 tests); exit codes captured in V04-01.

Uncertain claims:

- The exact reachability of F-INTENT-01-P2-01 in EKO depends on whether
  any EKO surface calls `run_chat_direct` with a router and no projector.
  EKO's GUI/TUI use streaming; the bug is framed as a framework-contract
  violation (parity) rather than an EKO-observed failure.
- The narrowness of F-INTENT-01-P3-01's trigger (failed hook activation
  is rare) makes its real-world blast radius small; the confidence is
  medium for that reason, though the code defect is unambiguous.

## Handoff

Conclusions downstream tasks may rely on:

1. **The intent layer is non-authoritative.** It is a pre-loop
   classification hint that can shortcut (DirectAnswer), inject a skill
   projection (SkillRequired), or fall through (Fallback). It does NOT
   spawn subagents, create/modify tasks, or touch `TaskRun`/`PlanTask`/
   `SubagentRun` state. F-TSK-01, F-SUB-01, and F-SKL-01 can rely on
   this: the router creates no parallel authority.
2. **Single skill-activation authority confirmed.** The router's
   `SkillRequired` arm reuses `activate_skill` (capabilities.rs:964),
   the same path hooks and the product runtime use. F-SKL-01 owns the
   `SkillRegistry` internals; this task confirms only the single-entry
   invariant.
3. **Generic + product-policy split is clean.** The framework ships
   classifier building blocks with zero hardcoded keywords; EKO supplies
   triggers and thresholds. No layering violation.
4. **Parity divergence exists.** F-INTENT-01-P2-01 is a concrete
   streaming-vs-non-streaming divergence; any task auditing parity
   (F-RCT-03, A-SRF-*) should be aware.

Reports they must read:

- This report (F-INTENT-01) + [V01-01](../validations/F-INTENT-01/V01-01.md),
  [V02-01](../validations/F-INTENT-01/V02-01.md),
  [V03-01](../validations/F-INTENT-01/V03-01.md),
  [V04-01](../validations/F-INTENT-01/V04-01.md).
- `tasks/F-RCT-01.md` — F-RCT-01-P3-03 explains the Category-C
  `intent_router` builder bypass this task confirms benign.
- `tasks/B-REF-01.md` — C1/C4 frame the "no parallel runtime authority"
  check this task performs.

Conditions that make this report stale:

- Addition of a new `Intent` variant, classifier, or a timeout wrapper in
  `IntentRouter::classify` — would invalidate V02 / resolve F-INTENT-01-P3-02.
- Push of the assistant message in `run_chat_direct`'s DirectAnswer arm —
  would resolve F-INTENT-01-P2-01 and require re-running V04.
- A turn-start clear of `hook_activation_cache` — would resolve
  F-INTENT-01-P3-01 and require re-running V03.
- Changes to the streaming/non-streaming gate structure in react_loop.rs
  or stream_channel.rs — would require re-running V01/V04.

Follow-up task IDs (no fixes implemented in this review task):

- A localized parity fix for F-INTENT-01-P2-01 (one-line push in
  `run_chat_direct`'s DirectAnswer arm) — bundle into a streaming/non-
  streaming parity maintenance task alongside any F-RCT-03 divergences.
- F-INTENT-01-P3-01, P3-02, P3-03 are independent robustness/observability
  cleanups; bundle into an intent-hardening task.
