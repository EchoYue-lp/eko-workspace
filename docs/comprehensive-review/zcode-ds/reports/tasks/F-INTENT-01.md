# F-INTENT-01: Intent classification and supervisory routing

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean

## Question

Are intent classification and trigger supervision generic, explainable,
bounded, and separate from runtime state authority?

## Scope

- `echo-agent/src/intent/` (full read): `mod.rs` (Intent / IntentClassifier /
  IntentRouter / IntentRouterConfig), `classifier.rs` (KeywordClassifier,
  LlmIntentClassifier, ChainedClassifier, KeywordClassifierConfig,
  SkillDescription), `trigger_supervisor.rs` (TriggerSupervisor, fusion rules).
- Router integration points: `src/agent/react/run/react_loop.rs:598-823`
  (non-streaming), `src/agent/react/run/stream_channel.rs:1-315` (streaming,
  G2), `src/agent/react/run/context.rs:430-623` (prepare phase, hook cache),
  `src/agent/react/run/phases/prepare.rs:57-88` (core-loop hook), builder
  registration (`builder.rs:103-104,606-631,1037-1039`,
  `react/mod.rs:225-226,571,624-629,1489-1492`), `capabilities.rs:940-1029`
  (`activate_skill`).
- EKO wiring: `echo-agent-cli/echo-agent-app-core/src/runtime.rs:270-356,594-670`,
  `infra.rs:420-450` (projector), `chat_driver.rs` (chat agent reachability).
- Tests: `src/intent/classifier.rs` and `trigger_supervisor.rs` unit tests,
  `src/agent/react/tests.rs:1224-1285`, `stream_channel.rs:759-1450` routing
  fixtures, EKO `runtime.rs:594-670`.

## Out Of Scope

- Hook registry/rules execution internals (double-fire root cause belongs to
  F-RCT-02-P2-03 / F-SKL-01; referenced, not re-derived).
- Memory-evolution triggers (`src/evolution/triggers.rs`, `TriggerDetector`)
  — F-EVO-01 scope; classified as a name overlap only.
- Skill loading/activation internals beyond the `activate_skill` boundary —
  F-SKL-01 scope.
- Tool surface / permission gates — F-EXT-01 / F-HITL-01 scope.
- DirectAnswer LLM-call internals (retry semantics of `call_llm_with_retry`)
  — F-RCT-02 scope; only the shortcut boundary is reviewed here.

## Inputs

- Root `AGENTS.md`, shared `README.md`, `REPORTING.md`, `TASKS.md`
  (F-INTENT-01 card), `zcode-ds/README.md`, templates.
- Dependency task reports read: `F-RCT-01.md` (complete) and `B-REF-01.md`
  (complete).
- Historical documents treated as hypotheses: `docs/MASTER-PLAN.md`,
  `echo-agent-cli/docs/MASTER-PLAN.md`,
  `echo-agent-cli/docs/runtime-architecture-audit.md`,
  `echo-agent-cli/docs/skills-taxonomy.md`, `echo-agent/AUDIT_REPORT.md`
  — classified in the Historical Claim Status section.

## Layering Decision

- Generic mechanism (framework): the whole `src/intent` module — `Intent`
  label contract, `IntentClassifier` trait, `IntentRouter` with configurable
  threshold and per-shortcut enable flags, zero-keyword `KeywordClassifier`,
  `LlmIntentClassifier`, `ChainedClassifier`, and the `TriggerSupervisor`
  three-source fusion with a hook activation slot (works with any hook
  registry, not EKO-specific). Integration into both ReAct entry points
  (shortcut / skill activation / fallback) is framework-level. Any
  echo-agent consumer can attach a router via
  `ReactAgentBuilder::intent_router` or `set_intent_router`. Correctly
  placed.
- EKO product policy (application): populating the keyword classifier and LLM
  skill descriptions from EKO's skill descriptors, wiring
  `TriggerSupervisor` + `IntentRouter` with threshold 0.7 onto the main agent
  (`runtime.rs:294-356`), and the always-on pre-model projector that disables
  the DirectAnswer shortcut (`infra.rs:446-450`). The DomainProfile
  "intent router inference" doc claim (`task_runtime/types.rs:26`) is EKO
  policy documentation that is not implemented (P3-04).
- Adapter boundary: `runtime.rs` conversion of skill descriptors to
  `KeywordClassifier`/`SkillDescription` and of the agent LLM client to
  `LlmIntentClassifier` is thin and lossless; no scheduling or state
  authority is duplicated in the adapter.
- Duplicate search (both repositories): terms `intent`, `IntentRouter`,
  `TriggerSupervisor`, `KeywordClassifier`, `LlmIntentClassifier`,
  `ChainedClassifier`, `IntentClassifier`, `Intent::`, `classify`,
  `activate_skill`, `hook_activation_cache`, `trigger`. Results (V01): one
  definition site; EKO is the only consumer; `ChainedClassifier` has zero
  production consumers; the evolution `TriggerDetector` is a different
  mechanism (memory triggers); HITL/subagent "intent" matches are prose.
  No parallel classification engine exists. **The intent router does not
  touch the task relation graph, revisioned stores, or run state machine —
  the "separate from runtime state authority" invariant holds (V03).**

## Current Path

Verified data flow (anchors in V02):

`ReactAgentBuilder::intent_router` (builder.rs:630-631) -> `agent.intent_router`
(builder.rs:1037-1039; mod.rs:226) -> per-turn `router.classify(message,
context)` at both run entries — `run_react_loop` (react_loop.rs:622-682,
after `prepare_react_context` at :603) and `run_stream_channel` G2
(stream_channel.rs:181-280, after `prepare_stream_context` at :129-134).
Decisions: `DirectAnswer` -> shortcut only when `allows_direct_answer_shortcut()`
(no pre-model projector, mod.rs:624-629) -> `direct_answer`/`direct_answer_stream`
(direct LLM call, trace finalize, assistant message pushed); `SkillRequired` ->
`activate_skill` (capabilities.rs:964-997: no-op for uninstalled, projection
injection when activated; activation errors warn-only); `Fallback` -> normal
ReAct loop.

EKO: `runtime.rs:294-356` builds `KeywordClassifier` from skill descriptors,
optional `LlmIntentClassifier` from the agent LLM client, `TriggerSupervisor`
sharing the agent's `hook_activation_cache` slot (written by the streaming
prepare after `fire_lifecycle_hook(UserPromptSubmit)`, context.rs:544-552),
wrapped in `IntentRouter` (threshold 0.7, both shortcuts enabled), installed
on the main agent used by `drive_chat`. The main agent always carries a
`TaskRuntimeContextProjector` (infra.rs:446-450), so the DirectAnswer ACTION
arm is unreachable in EKO while classification still runs every turn.

Hook path: `fire_lifecycle_hook` activates the requested skill directly on
success and clears the result (context.rs:463-481); on failure it retains the
request for the supervisor's retry (context.rs:478-479), and the streaming
prepare caches it (context.rs:548-552). `TriggerSupervisor::fuse` rule 3
adopts the hook slot with fixed confidence 0.6 (trigger_supervisor.rs:55-60).
`IntentRouter::classify` then re-applies the threshold: 0.6 < 0.7 -> Fallback
(mod.rs:136-150). The non-streaming path never writes the slot (its prepare
fires no hook; the core-loop `prepare_turn` hook at phases/prepare.rs:57-88
ignores `activate_skill` results and runs after classification).

## Findings

### F-INTENT-01-P1-01: TriggerSupervisor hook-fusion branch is dead — the documented "P4 retry" of failed hook-driven skill activation can never fire

- Priority: P1
- Confidence: high (deterministic constant comparison; static chain fully
  verified)
- Layer: framework
- Evidence: `echo-agent/src/intent/trigger_supervisor.rs:55-60` (hook adopted
  with fixed `confidence: 0.6`), `:14` (`HIGH_CONFIDENCE = 0.7`);
  `echo-agent/src/intent/mod.rs:136-150` (`IntentRouter::classify` converts
  `SkillRequired` with confidence < `confidence_threshold` to `Fallback`;
  default threshold 0.7 at mod.rs:92); EKO wiring `runtime.rs:343` (threshold
  0.7); retry design `echo-agent/src/agent/react/run/context.rs:478-479`
  ("Leave activate_skill in result so supervisor (P4) can retry").
- Reachability: a UserPromptSubmit hook requests `activate_skill` ->
  `fire_lifecycle_hook` direct activation fails (context.rs:476-479) ->
  slot cached (context.rs:548-552) -> `TriggerSupervisor::classify` consumes
  the slot via `take` (trigger_supervisor.rs:87-91) -> `fuse` emits
  `SkillRequired { confidence: 0.6 }` (rule 3) -> `IntentRouter::classify`
  rejects (0.6 < 0.7) and returns `Fallback` (mod.rs:136-150) -> ReAct
  proceeds without the skill; the slot is consumed so nothing retries.
  Additionally the non-streaming path (`run_react_loop`) never writes the
  slot at all: its prepare (`react_loop.rs:508-591`) fires no UserPromptSubmit
  hook, and the core-loop `prepare_turn` hook (phases/prepare.rs:57-88) runs
  after classification, ignores `activate_skill`, and does not write the
  cache.
- Expected invariant: documented fusion rule 3 ("Both low but hook slot has
  activation → adopt hook", trigger_supervisor.rs:47) and the "P4 retry"
  contract (context.rs:478-479) yield an actionable `SkillRequired`.
- Observed behavior: the hook branch can never produce an actionable intent
  through the documented router wrapper; after a failed direct hook
  activation the skill is silently never activated and the turn degrades to
  plain ReAct with only a warn log.
- Impact: trigger supervision silently drops hook-requested skill activations
  exactly in the failure case the fusion was built to recover; EKO's
  "已根据上下文自动激活技能" behavior is unreliable when the first activation
  attempt fails, with no user-visible signal. Deterministic logic
  contradiction between two components of the same feature.
- Root cause: two independent threshold authorities with mismatched values —
  the fusion hardcodes hook confidence 0.6 (below any router threshold in
  use) while the router re-applies `confidence_threshold`; the supervisor
  unit tests only exercise `fuse` in isolation and never feed its output
  through `IntentRouter::classify`, so the contradiction ships green
  (V04-01). The non-streaming arm is a second cause: the hook->cache write
  exists only on the streaming prepare path.
- Direction: align the fusion hook confidence with the router threshold
  (e.g. emit `confidence = config threshold` or have the router trust the
  supervisor's hook decision via a distinct intent/flag), and write the cache
  from the same single prepare path used by both run entries (resolve the
  F-RCT-02-P2-03 ordering so classification always follows the hook).
  Delete the now-dead "P4 retry" comment path if the fusion approach is
  removed.
- Regression validation: unit/integration test — hook slot set + keyword/LLM
  low -> classify through `IntentRouter::classify` must yield
  `SkillRequired`; a hook-driven activation failure followed by a turn must
  end with the skill activated; run the same fixture through the non-streaming
  entry.
- Validation reports: [V01](../validations/F-INTENT-01/V01-01.md),
  [V03](../validations/F-INTENT-01/V03-01.md),
  [V04-01](../validations/F-INTENT-01/V04-01.md)

### F-INTENT-01-P2-01: Intent classification has no timeout, cancel, retry, or budget — a hanging classifier stalls every turn on the agent

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/intent/classifier.rs:411`
  (`llm.chat_simple(messages).await` — no timeout wrapper, no cancel token,
  no retry/circuit breaker; `chat_simple` builds `ChatRequest{..Default::default()}`
  with `cancel_token: None`, `echo-core/src/llm/mod.rs:74-95`); the call runs
  while the agent holds its `execution_mutex` (stream_channel.rs:77 ->
  :181-280; react_loop.rs:600 -> :622-682); contrast the main loop's
  `call_llm_with_retry` (react_loop.rs:23-90: retries + circuit breaker);
  only provider-level bound exists (OpenAI non-streaming request timeout
  120 s, `echo-integration/src/providers/openai.rs:257`).
- Reachability: every EKO chat turn whose keyword pass scores below 0.7
  (runtime.rs wiring; trigger_supervisor.rs:76-84) performs this unbounded
  await before ReAct; a provider that hangs (cf. F-INT-01-P1-01 HTTP 202
  transport) blocks all subsequent turns on that agent, and user cancellation
  does not interrupt classification.
- Expected invariant: classification is bounded and cancellable like other
  LLM work (timeout/cancel/budget knobs), and must not hold the agent
  execution mutex for an unbounded LLM call.
- Observed behavior: no framework-level bound; the classification await is
  effectively serialized under the execution mutex.
- Impact: worst-case per-turn latency equals the provider timeout (120 s per
  provider default) with the whole agent blocked; a stuck classifier
  degrades chat availability silently; cancellation has no effect on the
  classifier call.
- Root cause: the classifier path predates the shared retry/circuit-breaker
  helper and was never given a timeout/cancel plumbing; `chat_simple` is a
  convenience API without request-level control.
- Direction: run classification with `tokio::time::timeout` (config knob in
  `IntentRouterConfig`, e.g. default 5-10 s), pass a cancel token (derive from
  the turn's cancellation), reuse `retry_llm_call`/circuit breaker for the
  classifier call, and/or move classification out of the mutex hold window;
  fallback to `Fallback` on timeout (same as existing error fallback,
  classifier.rs:411-417).
- Regression validation: fixture with a never-responding mock LLM client —
  classification must return `Fallback` within the configured timeout and the
  next turn on the same agent must start promptly; cancel-mid-classification
  test.
- Validation reports: [V03](../validations/F-INTENT-01/V03-01.md)

### F-INTENT-01-P2-02: EKO enables DirectAnswer routing it can never take — every non-keyword chat turn pays a discarded LLM classification call

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: EKO always installs a `TaskRuntimeContextProjector` on the main
  agent (`echo-agent-cli/echo-agent-app-core/src/infra.rs:446-450`);
  `allows_direct_answer_shortcut()` returns `projector.is_none()`
  (`echo-agent/src/agent/react/mod.rs:624-629`) so the shortcut arm is
  unreachable; EKO still wires `enable_direct_answer: true`
  (`runtime.rs:342-346`); `TriggerSupervisor` invokes the LLM classifier on
  every turn where keywords score < 0.7 (trigger_supervisor.rs:76-84), and the
  resulting `DirectAnswer` is discarded at the non-shortcut arm
  (`react_loop.rs:657-663` / `stream_channel.rs:247-253`, debug log).
- Reachability: default EKO chat (drive_chat on the main agent) —
  guaranteed; empirically pinned by V04-02 (`*_direct_answer_routes_through_projection_boundary`
  tests prove the projector gate on both run entries).
- Expected invariant: EKO does not pay for a routing decision it structurally
  cannot use; `enable_direct_answer` reflects reachable behavior.
- Observed behavior: ~500-token LLM call + latency on every chat turn whose
  keywords do not strongly match, with the DIRECT outcome always discarded.
- Impact: added per-turn cost/latency on the primary chat path; misleading
  configuration surface (`enable_direct_answer: true` never fires).
- Root cause: the router's shortcut gate lives in the agent (projector
  check) while the classifier always runs; EKO wiring did not disable the
  DirectAnswer label it cannot honor.
- Direction: either (a) EKO sets `enable_direct_answer: false` in
  `IntentRouterConfig` (runtime.rs:342-346) so `TriggerSupervisor` can skip
  the LLM call when only the DIRECT outcome is possible — requires the
  supervisor/router to know the label is unusable, or (b) framework: let the
  router consult a "shortcut available" predicate before classifying and
  skip the classifier entirely when all outcomes are unusable; add a test
  asserting no LLM classifier call on turns under an installed projector.
- Regression validation: EKO runtime fixture with projector installed +
  MockLlmClient recording calls — a non-keyword turn must not invoke the LLM
  classifier (after fix); keep V04-02 fixtures green.
- Validation reports: [V02](../validations/F-INTENT-01/V02-01.md),
  [V04-02](../validations/F-INTENT-01/V04-02.md)

### F-INTENT-01-P2-03: Dual threshold authority — `TriggerSupervisor::fuse` hardcodes 0.7 while `IntentRouterConfig.confidence_threshold` is configurable

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `trigger_supervisor.rs:14` (`HIGH_CONFIDENCE: f32 = 0.7`,
  used at :49-53 and :76) vs `mod.rs:82` (`IntentRouterConfig.confidence_threshold`,
  applied at :129,:141); `KeywordClassifier` also embeds its own acceptance
  threshold 0.7 (classifier.rs:139-141) and its own `enable_direct_answer`
  flag (classifier.rs:50,98-100) parallel to the router's
  `enable_direct_answer` (mod.rs:84).
- Reachability: any consumer configuring `confidence_threshold: 0.5` finds
  the fusion still refuses < 0.7 keyword/LLM outputs; any consumer toggling
  only `KeywordClassifier::set_enable_direct_answer(false)` while the router
  flag stays true (or vice versa) keeps the OTHER gate active.
- Expected invariant: one documented threshold and one enable flag per
  decision, applied once.
- Observed behavior: three copies of the acceptance threshold and two copies
  of the direct-answer flag with independent defaults.
- Impact: config knobs are partially inert and misleading; the mismatch is
  the structural cause of P1-01's dead branch; future threshold tuning must
  touch three sites.
- Root cause: the fusion supervisor and classifier evolved after the router
  contract; constants were duplicated instead of threaded from config.
- Direction: thread `confidence_threshold`/enable flags from
  `IntentRouterConfig` into `TriggerSupervisor` and `KeywordClassifier` (or
  document + enforce a single framework-wide constant), keeping the router as
  the single enforcement point; delete `HIGH_CONFIDENCE` duplication.
- Regression validation: test classifying with threshold 0.5 via the router —
  a 0.6-confidence output must be actionable; a flag-off must suppress the
  label at every layer.
- Validation reports: [V01](../validations/F-INTENT-01/V01-01.md),
  [V03](../validations/F-INTENT-01/V03-01.md)

### F-INTENT-01-P3-01: Intent decisions are invisible to consumers — no typed event, no trace record, silent no-op for unknown skills

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: no `AgentEvent` variant for intent decisions
  (`echo-core/src/agent/mod.rs:143` variant scan, V03); no trace
  `RunEvent` for classification (`src/trace/` scan); decisions are
  `tracing` logs only (react_loop.rs:630-634,668-680; stream_channel.rs:
  192-196,258-278); `activate_skill` returns `Ok(())` for uninstalled skills
  (capabilities.rs:965-967) so an LLM-hallucinated skill name is silently
  dropped; activation errors are warn-only (react_loop.rs:674-676).
- Reachability: any turn that triggers a shortcut or skill activation; any
  `SkillRequired` naming an uninstalled skill (the LLM classifier prompt
  lists available skills, but does not constrain output).
- Expected invariant: routing decisions and their confidence are observable
  to consumers (typed event or trace), and a trigger naming an unavailable
  target is either rejected or reported.
- Observed behavior: GUI/TUI cannot distinguish a DirectAnswer shortcut from
  a normal answer; failed or impossible activations leave only log lines.
- Impact: "explainable" criterion of the task question is unmet; debugging
  and surface projections (e.g. showing "activated skill X") are impossible
  without log scraping; silent no-ops can mask classifier regressions.
- Root cause: the router predates the typed event envelope and was wired with
  logging only; no validation of the skill name at the routing boundary.
- Direction: add an `AgentEvent::Intent { intent, confidence, action }`
  (or trace `RunEvent`) emitted after classification; validate the skill name
  against the registry in the `SkillRequired` arm and log/report rejection.
- Regression validation: event-envelope test asserting a `SkillRequired`/
  `DirectAnswer` turn yields the new event; fixture with an unknown skill
  name asserting a visible rejection signal.
- Validation reports: [V03](../validations/F-INTENT-01/V03-01.md)

### F-INTENT-01-P3-02: Hook activation slot can leak across turns — guard-blocked or router-less turns leave a stale activation for the next classification

- Priority: P3
- Confidence: medium (static chain verified; not dynamically executed)
- Layer: framework
- Evidence: slot written in prepare (context.rs:548-552) and consumed once by
  `take()` in classify (trigger_supervisor.rs:87-91); guard-blocked streaming
  turns return before classification (stream_channel.rs:141-179) leaving the
  slot; any framework consumer that runs prepare without a router accumulates
  the slot; next classification turn adopts the stale activation (fuse rule 3).
- Reachability: guard blocking after a UserPromptSubmit hook requested a
  skill; consumers with hooks but no router.
- Expected invariant: a hook's activation request applies to the turn that
  produced it, and never to a later turn.
- Observed behavior: the stale slot is consumed by the next turn's classifier
  and the unrelated skill gets activated (projection injected into context).
- Impact: unintended skill activation/context pollution in the following
  turn; low frequency.
- Root cause: the slot is turn-scoped by convention only; nothing clears it
  when classification is skipped.
- Direction: clear the slot when a turn ends without classification
  (guard-block paths) or tag the cache with the turn id and discard mismatched
  entries; add a guard-block + hook fixture.
- Regression validation: turn 1 hook requests skill + guard blocks -> turn 2
  plain input must not activate the skill.
- Validation reports: [V03](../validations/F-INTENT-01/V03-01.md)

### F-INTENT-01-P3-03: Duplicate trigger words silently overwrite — the last registered skill shadows earlier registrations with no warning

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `classifier.rs:75-77` (`skill_keywords.insert(word.to_lowercase(),
  skill_name)` — HashMap insert, last wins) called from
  `add_skill_keywords`/`add_skill_keyword_map`; EKO populates from all skill
  descriptors (`runtime.rs:295-313`).
- Reachability: any two skills declaring the same trigger word (e.g. a user
  skill reusing a built-in trigger); the earlier skill's trigger silently
  stops routing.
- Expected invariant: trigger registration is collision-free or collisions
  are reported/deduped deterministically.
- Observed behavior: silent last-writer-wins with no log, no error, no test
  coverage.
- Impact: a skill's trigger supervision silently breaks after adding another
  skill with an overlapping trigger; hard to diagnose.
- Root cause: HashMap-based registration without collision handling.
- Direction: detect collisions in `add_skill_keywords` and warn (or expose
  the collision list); optionally prefer longest/most-specific trigger
  deterministically; add a collision fixture.
- Regression validation: register two skills sharing a trigger -> both
  skills' routing state is deterministic and the collision is observable.
- Validation reports: [V01](../validations/F-INTENT-01/V01-01.md)

### F-INTENT-01-P3-04: EKO doc drift — DomainProfile "intent router inference" is unimplemented; "ChainedClassifier" wiring comment is stale

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs:26`
  documents selection order "3. Intent router inference" for `DomainProfile`;
  no code implements intent-based profile inference — planner/service always
  construct `DomainProfile::General` (planner.rs test helper :213; service.rs:
  268,327-338,924-1008) or parse a user string (`task_tools.rs:890-891`);
  `runtime.rs:294` labels the wiring "13. ChainedClassifier (Keyword → LLM)"
  while the code wires `TriggerSupervisor` (:315-356); heading numbering
  duplicated ("── 12." at :291 "Startup hook" and :315 "TriggerSupervisor");
  `runtime-architecture-audit.md:34-37`
  describes the Keyword → LLM chain without `TriggerSupervisor`
  (V05 classification: current components, stale chain).
- Reachability: documentation and comments only.
- Expected invariant: documented selection order and wiring labels match
  behavior.
- Observed behavior: doc promises a selection step and a wiring topology that
  do not exist.
- Impact: misleading API docs for EKO developers; dead doc claim about the
  intent router; `ChainedClassifier` appears wired when it has zero
  production consumers.
- Root cause: docs written during design; wiring migrated to the fusion
  supervisor and DomainProfile selection deferred without doc updates.
- Direction: implement the intent-based DomainProfile inference (deferred
  product decision) or remove step 3 from the doc; fix the `runtime.rs:294`
  comment and numbering; update `runtime-architecture-audit.md` to the
  TriggerSupervisor chain (note `ChainedClassifier` remains a framework API
  without current consumers — keep or document per X-BND-01).
- Regression validation: none (doc); grep `intent router inference` after
  change returns only intentional references.
- Validation reports: [V05](../validations/F-INTENT-01/V05-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search across both repositories | yes | passed | [V01-01](../validations/F-INTENT-01/V01-01.md) |
| V02 | Registration and runtime reachability trace (builder -> both run entries -> EKO main agent) | yes | passed | [V02-01](../validations/F-INTENT-01/V02-01.md) |
| V03 | Invariant/edge inspection: label contract, timeout/fallback, explainability, state-authority separation, hook-slot lifecycle | yes | passed | [V03-01](../validations/F-INTENT-01/V03-01.md) |
| V04 | Targeted tests: intent module (19), direct_answer routing (3), intent_router (1), EKO classifier (6) | yes | passed (exit 0 each) | [V04-01](../validations/F-INTENT-01/V04-01.md) [V04-02](../validations/F-INTENT-01/V04-02.md) [V04-03](../validations/F-INTENT-01/V04-03.md) [V04-04](../validations/F-INTENT-01/V04-04.md) |
| V05 | Historical-document drift check | yes | passed | [V05-01](../validations/F-INTENT-01/V05-01.md) |

All required validations executed; every reported command has a known exit
code (0); no validation is pending.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| skills-taxonomy.md:15-18,99,106 — skills auto-triggered via IntentRouter | current | `runtime.rs:294-356`; `react_loop.rs:664-677`; [V05-01](../validations/F-INTENT-01/V05-01.md) |
| runtime-architecture-audit.md:34-37 — component inventory (IntentRouter/Keyword/Llm/Chained) | current (components) / stale (chain: EKO wires TriggerSupervisor, not ChainedClassifier) | `runtime.rs:315-356`; `ChainedClassifier` zero production consumers; [V05-01](../validations/F-INTENT-01/V05-01.md) |
| MASTER-PLAN.md:458 — old plan-approval route/classifier DTOs deleted | current | no such DTOs remain; unrelated to intent classification; [V05-01](../validations/F-INTENT-01/V05-01.md) |
| task_runtime/types.rs:26 — DomainProfile selection includes "3. Intent router inference" | stale | no inference code exists; planner/service default `General`; [V05-01](../validations/F-INTENT-01/V05-01.md) |
| runtime.rs:294 comment "13. ChainedClassifier (Keyword → LLM)" | stale | block actually wires `TriggerSupervisor`; [V05-01](../validations/F-INTENT-01/V05-01.md) |
| Both MASTER-PLANs contain no intent-module entry | current (absent) | feature exists undocumented in architecture docs; informational |

## Coverage And Uncertainty

- Static analysis throughout; no dynamic run executed the hook-fusion
  failure/retry scenario (the 0.6 < 0.7 rejection is a deterministic
  comparison of constants — confidence high, but the end-to-end hook failure
  path was not executed).
- `LlmIntentClassifier` was not exercised against a real provider (read-only
  task; no network fixtures); prompt parsing is covered by unit tests
  (V04-01).
- The guard-block + stale-slot scenario (P3-02) is derived, not executed.
- The provider timeout bound (openai.rs:257) was verified statically; other
  providers (Anthropic/Ollama) may differ — not enumerated.
- Subagent/team paths were not traced for intent interaction (they do not
  receive routers); a future X-SRF-01 parity check should confirm no surface
  needs the router beyond the main agent.
- EKO GUI/TUI event projection of intent decisions was not reviewed (no such
  event exists — P3-01).

## Handoff

- Downstream tasks may rely on: single intent authority in the framework
  (V01); reachability map (V02); separation from runtime state authority
  holds (V03); green test state (V04); the P1-01 dead fusion branch, P2-01
  unbounded classification, P2-02 EKO shortcut/waste, and P2-03 threshold
  duplication as the primary follow-up items.
- Reports to read: this report + V01-01..V05-01; F-RCT-01 (router option
  registration context); F-RCT-02 (P2-03 UserPromptSubmit double-fire — the
  hook-ordering root of P1-01's non-streaming arm); F-RCT-03 (streaming
  event flow); F-SKL-01 (skill activation semantics); F-EVO-01 (evolution
  TriggerDetector naming overlap).
- Stale triggers: any change to `src/intent/*`, `react_loop.rs` intent block,
  `stream_channel.rs` G2, `context.rs` prepare/cache, `capabilities.rs`
  `activate_skill`, EKO `runtime.rs` wiring, `infra.rs` projector, or
  `task_runtime/types.rs` DomainProfile doc invalidates the corresponding
  claims.
- Follow-up task IDs (fixes are not implemented in this review): F-RCT-02
  (hook ordering resolution), F-HITL-01/X-AUT-01 (classifier cancel/timeout
  policy interplay), X-BND-01 (ChainedClassifier public-API fate; threshold
  authority), A-HITL-01/A-SRF-03 (surface visibility of routing decisions),
  Q-TST-01 (add router-through-fusion fixtures), S-RDM-01 (roadmap ordering:
  P1-01 before P2-*).
