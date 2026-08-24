# F-RCT-01: ReAct construction and canonical prompt assembly

> Status: complete
> Reviewer: Codex primary reviewer, with isolated subagent evidence
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d35c9a432370b923acabfbb5f32d63`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: `echo-agent` clean; `echo-agent-cli` concurrently dirty only under `web-frontend/src/generated/*.ts` (not read or modified); review artifacts are outside both source repositories

## Question

Does builder/config assembly produce a deterministic ReAct Agent with one tool
registry, correct instructions, budgets, hooks, project rules, and truthful
feature state?

## Scope

- `echo-agent/src/agent/react/builder.rs`, `config.rs`, `mod.rs`, capabilities,
  subsystems, snapshots, and request-schema assembly.
- `echo-agent/echo-execution/src/tools.rs` registry identity and duplicate-name
  semantics.
- `echo-agent/echo-core/src/compression.rs` and
  `echo-agent/echo-state/src/compression/mod.rs` canonical context/cardinality.
- Framework construction options through their first production runtime reader.
- Read-only EKO call-site inspection solely to establish reachability of framework
  prompt/memory surfaces; no EKO policy review.
- Existing source tests as a coverage matrix. No new test/build execution.

## Out Of Scope

- Non-streaming and streaming ReAct transition/terminal behavior (`F-RCT-02` and
  its streaming counterpart).
- Retry arithmetic, circuit breaking, and time correctness (`F-REL-01`).
- Generic public API documentation defects already owned by `F-API-01`, including
  its `DefaultAgentFactory` finding.
- Task graph, subagent scheduler, provider protocol, permission-policy semantics,
  memory quality, and application UI state beyond the bounded reachability cited.
- Source fixes, index changes, network research, Cargo/rustc/tests/builds, or
  generated-file inspection.

## Inputs

- Root `AGENTS.md`; shared review `README.md`, `REPORTING.md`, `TASKS.md`; Codex
  `README.md`; shared task/validation templates.
- Codex dependency reports `F-CORE-01.md` and `F-API-01.md`. F-API-01-P2-06 was
  consumed only to prevent a duplicate factory finding.
- Current source and current git history/blame. No other reviewer directory was
  read and no unaccepted review conclusion was imported.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Builder/config mapping, prompt composition, canonical compression authority, tool registry/filtering, budgets, hooks, and feature construction are reusable framework mechanisms and belong in `echo-agent`. |
| EKO product policy | EKO chooses prompt bodies, enabled capabilities, working directories, and memory stores. Those choices remain application policy; the framework must apply them faithfully. |
| Adapter boundary | CLI/TUI/GUI should call one framework prompt/capability API and render the outcome. They must not reimplement composition, canonical reinjection, allowlist filtering, or feature detection. |
| Duplicate search | Searched builder/config fields and setters, all constructors, ToolManager definitions/registrations/snapshot consumers, prompt and canonical setters/readers, project rules, template/notebook/detector options, feature cfgs, budgets/hooks, memory state, and live callers across both repositories. |
| Migration deletion | Converge on one prompt mutation/composition authority and delete `mutable_system_prompt` plus the duplicate setter semantics; centralize allowlist filtering and delete registration-method filtering; remove public no-op options if they are not wired; separate memory capability state from default-store policy. |

No framework behavior should move into EKO. The app call sites prove impact but
do not own the root causes.

## Current Path

```text
ReactAgentBuilder::build
  -> validate model + subagent/tool coupling
  -> AgentConfig (budgets, token limits, flags, profile, working dir, callbacks)
  -> ReactAgent::new_inner
       -> build_system_prompt(base + COT + profile suffix + project rules)
       -> ContextManager.with_system(composed prompt)
       -> CanonicalContext {
            system_prompt: composed prompt,
            project_rules: same rules again,
          }
       -> one ToolManager
            -> core + feature + builtin tools
            -> Arc<ToolManager> in ToolExecutionSubsystem
            -> AgentRunSnapshot -> tools_for_llm -> request schema
       -> one shared HookRegistry -> run phases and subagent executor
  -> inject custom tools/store/pipeline/router/template manager/visibility policy

runtime mutations
  Agent::set_system_prompt(&self) -> mutable_system_prompt only
  ReactAgent::set_system_prompt(&mut self) -> config + active message only
  set_working_dir -> recomposed message + canonical system only
  compression -> restore canonical system + supplemental canonical project rules
```

Positive conclusions:

- Normal construction and request assembly share one ToolManager. Duplicate names
  replace deterministically; definitions are sorted and cache invalidation is
  centralized.
- Builder `run_budget`, iteration limit, token limit/budget, hooks, tool pipeline,
  intent router, and visibility horizon reach live runtime readers. The same hook
  registry Arc is captured by the subagent executor and run snapshots.
- `enable_tool=false` prevents tool schemas from being sent even though the
  manager retains framework-internal tools.

## Findings

### F-RCT-01-P1-01: Runtime system-prompt mutation has two broken authorities

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/react/mod.rs:676`,
  `echo-agent/src/agent/react/mod.rs:984`,
  `echo-agent/src/agent/react/mod.rs:2750`,
  `echo-agent/src/agent/react/mod.rs:3017`,
  `echo-agent/src/agent/react/capabilities.rs:1348`,
  `echo-agent-cli/src/tui/events.rs:2774`,
  `echo-agent-cli/echo-agent-app-core/src/runtime.rs:190`
- Reachability: TUI `/system` imports `Agent` and invokes the `&self` trait setter
  through an immutable read guard, then reports success. App bootstrap and CLI/GUI
  paths invoke the separate async inherent setter. Compression later consults
  ContextManager canonical state.
- Expected invariant: one mutation API recomposes all prompt sections, updates the
  active system message and canonical authority atomically, and becomes observable
  on the next model call.
- Observed behavior: the trait setter writes only `mutable_system_prompt`; no
  turn-start path reads it. The inherent setter changes the config base and active
  message, but omits COT/model-profile/project-rule recomposition and canonical
  update. A later compression can restore the old prompt. `current_system_prompt`
  returns the raw base/override rather than the assembled prompt, so app baseline
  injection reaches the divergent setter during normal startup.
- Impact: TUI tells the user a prompt changed when the model continues with the old
  instructions; other modes can temporarily change it and then silently revert or
  lose framework instruction sections after compression.
- Root cause: mutable prompt state was added beside config and canonical context
  without a single asynchronous mutation authority; comments/tests assert a
  nonexistent turn-start application step.
- Direction: introduce one framework-owned async prompt update that accepts the
  base prompt, calls canonical composition once, and atomically updates config,
  current context, and canonical context. Make trait/API callers delegate to it or
  change the trait to a fallible async contract. Delete `mutable_system_prompt`,
  the inert setter, and stale turn-start comments once callers migrate.
- Regression validation: mutate through trait object, TUI and inherent API; assert
  next request and post-compression prompt are identical and contain COT/profile/
  current project rules exactly once.
- Validation reports: [V03-02](../validations/F-RCT-01/V03-02.md),
  [V03-08](../validations/F-RCT-01/V03-08.md),
  [V05-01](../validations/F-RCT-01/V05-01.md)

### F-RCT-01-P1-02: Project rules are duplicated after compression and stale after a directory switch

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/react/mod.rs:676`,
  `echo-agent/src/agent/react/mod.rs:358`,
  `echo-agent/src/agent/react/mod.rs:907`,
  `echo-agent/echo-core/src/compression.rs:345`,
  `echo-agent/echo-state/src/compression/mod.rs:862`,
  `echo-agent/echo-state/src/compression/mod.rs:877`
- Reachability: `new_inner` always builds ContextManager and canonical context;
  every compression path calls canonical reinjection. Worktree/checkpoint paths
  call `set_working_dir`, which refreshes only part of canonical state.
- Expected invariant: the current workspace's rules appear once in effective
  system instructions and are the only rules restored after compression.
- Observed behavior: initial composition embeds rules inside canonical
  `system_prompt` and also saves the same body in `canonical.project_rules`.
  Reinjection exact-dedups the complete prompt but emits a differently wrapped
  second rule message. `set_working_dir` replaces only the complete canonical
  prompt while preserving the old separate rules, so compression can inject the
  previous workspace's instructions beside the new prompt.
- Impact: instruction weight changes after compression, and worktree/project
  switches can contaminate a run with old repository rules.
- Root cause: `CanonicalContext` documents its system prompt as the base without
  rule extensions, but construction stores the fully composed prompt while also
  populating the supplemental rule slot.
- Direction: choose one representation. Prefer canonical base prompt plus
  separately versioned rule/skill sections, compose effective messages once, and
  update all sections atomically on cwd/prompt changes. Delete raw rule injection
  into the canonical base or delete the redundant supplemental slot.
- Regression validation: build with a real rule file, force repeated compression,
  count exact rule-body occurrences; then switch between two roots and assert only
  the new root survives.
- Validation reports: [V03-01](../validations/F-RCT-01/V03-01.md),
  [V03-03](../validations/F-RCT-01/V03-03.md),
  [V03-08](../validations/F-RCT-01/V03-08.md)

### F-RCT-01-P1-03: AgentConfig tool allowlist is bypassed by most registration paths

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/config.rs:54`,
  `echo-agent/src/agent/config.rs:470`,
  `echo-agent/src/agent/react/mod.rs:386`,
  `echo-agent/src/agent/react/capabilities.rs:36`,
  `echo-agent/src/agent/react/capabilities.rs:43`,
  `echo-agent/src/agent/snapshot.rs:190`,
  `echo-agent/src/agent/react/mod.rs:3009`
- Reachability: every model request obtains schemas from AgentRunSnapshot; tools
  enter the shared manager through initial registration, builder custom tools,
  `add_tool(s)`, replacement, and the `Agent` trait.
- Expected invariant: the documented nonempty allowlist means only listed tools
  can be exposed and called, independent of registration API.
- Observed behavior: only batch `add_tools` reads `config.allowed_tools`.
  Builtins, core tools, builder custom tools (`add_tool`), replacement, and trait
  registration bypass it. Snapshot filtering has disabled/skill/visibility/plan
  policies but no root config allowlist, and execution uses the same manager.
- Impact: consumers relying on the public config contract can expose and execute
  tools they explicitly omitted. This violates automated agent capability policy
  even in EKO's local threat model.
- Root cause: policy enforcement lives in one convenience registration method
  instead of the request/execution choke point.
- Direction: snapshot the root allowlist into the same authoritative eligibility
  policy used by schema exposure and execution. Delete `add_tools`-only filtering
  so all registration APIs have identical semantics.
- Regression validation: register allowed/disallowed tools through every API plus
  builtins; assert both schemas and direct execution reject the disallowed set.
- Validation reports: [V02-01](../validations/F-RCT-01/V02-01.md),
  [V03-05](../validations/F-RCT-01/V03-05.md),
  [V03-08](../validations/F-RCT-01/V03-08.md)

### F-RCT-01-P2-04: Builder silently accepts capabilities absent from the compiled feature set

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/Cargo.toml:65`,
  `echo-agent/src/agent/react/builder.rs:339`,
  `echo-agent/src/agent/react/builder.rs:345`,
  `echo-agent/src/agent/react/builder.rs:411`,
  `echo-agent/src/agent/react/builder.rs:854`,
  `echo-agent/src/agent/react/mod.rs:438`,
  `echo-agent/src/agent/react/mod.rs:467`
- Reachability: root defaults compile with no features; the three builder methods
  remain public in that build and `build` returns an Agent after only model and
  subagent/tool coupling checks.
- Expected invariant: an explicitly requested feature-dependent capability is
  constructed or rejected with a typed unsupported-capability error.
- Observed behavior: without `human-loop`, HITL flags have no registration branch;
  without `subagent`, subagent infrastructure and `agent_tool` registration are
  compiled out. Build still succeeds with the corresponding config flags true.
- Impact: downstream code receives a successfully constructed but capability-
  incomplete Agent and can only discover the mismatch when a live operation is
  missing or errors.
- Root cause: setters/config flags are ungated while all implementation branches
  are cfg-gated; construction validation does not know compiled capabilities.
- Direction: cfg-gate unavailable builder methods or make `build` reject requested
  absent capabilities. Delete flags that cannot be meaningful in that build.
- Regression validation: no-default compile surface plus construction tests for
  each absent capability; feature-enabled controls must still construct.
- Validation reports: [V03-06](../validations/F-RCT-01/V03-06.md),
  [V03-08](../validations/F-RCT-01/V03-08.md),
  [V04-01](../validations/F-RCT-01/V04-01.md)

### F-RCT-01-P2-05: Three public construction options are stored but never consumed by ReAct

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/react/builder.rs:830`,
  `echo-agent/src/agent/react/mod.rs:1370`,
  `echo-agent/src/agent/config.rs:169`,
  `echo-agent/src/agent/config.rs:807`,
  `echo-agent/src/agent/config.rs:1024`,
  `echo-agent/src/agent/react/loop_detector.rs:42`
- Reachability: public builder/config APIs let consumers configure prompt
  templates, notebook recording, and loop detection. Repository-wide production
  searches end at storage/setters; only standalone engines/modules/tests consume
  their internals.
- Expected invariant: public Agent construction options alter prompt rendering,
  tool-call recording, or loop control as documented.
- Observed behavior: ReAct never calls the prompt manager's render APIs, never
  constructs/records a Notebook, and never constructs a LoopDetector from the
  configured value.
- Impact: integrations compile and appear configured but receive none of the
  advertised behavior, increasing false confidence and configuration surface.
- Root cause: standalone components were exposed through AgentConfig/builder
  before a runtime owner and lifecycle integration existed.
- Direction: either wire each through one ReAct runtime owner with explicit
  lifecycle semantics, or delete the Agent option/docs and keep the standalone
  framework component as an independently usable API.
- Regression validation: final model request renders a registered template;
  tool calls produce notebook cells; configured repeated-call thresholds alter
  loop behavior. If deleted, compile-fail/API documentation checks replace them.
- Validation reports: [V01-02](../validations/F-RCT-01/V01-02.md),
  [V03-04](../validations/F-RCT-01/V03-04.md),
  [V03-08](../validations/F-RCT-01/V03-08.md)

### F-RCT-01-P2-06: External memory-store construction reports memory disabled while memory is active

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/react/builder.rs:663`,
  `echo-agent/src/agent/react/builder.rs:945`,
  `echo-agent/src/agent/react/builder.rs:980`,
  `echo-agent/src/agent/react/mod.rs:750`,
  `echo-agent/src/agent/react/mod.rs:1075`,
  `echo-agent-cli/src/tauri/commands/config.rs:83`
- Reachability: EKO injects an external store through this builder path and later
  exposes `agent.config().is_memory_enabled()` in its config response.
- Expected invariant: capability introspection is true whenever the agent has the
  store, memory tools, promotion, and automatic recall behavior enabled.
- Observed behavior: builder rewrites `enable_memory=false` solely to suppress
  default FileStore creation, then injects the external store and registers memory
  tools. Runtime recall is store-driven, but config introspection remains false.
- Impact: UI/config consumers receive state contradictory to the constructed
  Agent and can make incorrect update/display decisions.
- Root cause: one boolean owns two distinct concerns: memory capability state and
  default-store creation policy.
- Direction: separate store provisioning/default-store policy from capability
  enablement. Preserve `enable_memory=true` after injected construction and delete
  the temporary false rewrite.
- Regression validation: external-store build reports enabled, has exactly one
  store/tool set, recalls from that store, and never constructs a FileStore.
- Validation reports: [V03-07](../validations/F-RCT-01/V03-07.md),
  [V03-08](../validations/F-RCT-01/V03-08.md),
  [V04-01](../validations/F-RCT-01/V04-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V00-01 | Protocol, scope, dependencies, and de-duplication | yes | passed | [report](../validations/F-RCT-01/V00-01.md) |
| V00-02 | Commit, dirty-state, and disk snapshot | yes | passed with documented optional-du deviation | [report](../validations/F-RCT-01/V00-02.md) |
| V01-01 | Broad definition/duplicate discovery | no (superseded) | inconclusive due truncation | [report](../validations/F-RCT-01/V01-01.md) |
| V01-02 | Bounded construction authority and duplicate search | yes | passed | [report](../validations/F-RCT-01/V01-02.md) |
| V02-01 | One ToolManager registration-to-request trace | yes | passed | [report](../validations/F-RCT-01/V02-01.md) |
| V02-02 | Budgets/hooks/pipeline/router option-to-runtime trace | yes | passed | [report](../validations/F-RCT-01/V02-02.md) |
| V03-01 | Project-rule cardinality source proof | yes | passed | [report](../validations/F-RCT-01/V03-01.md) |
| V03-02 | System-prompt mutation authority and EKO reachability | yes | passed | [report](../validations/F-RCT-01/V03-02.md) |
| V03-03 | Working-directory canonical refresh | yes | passed | [report](../validations/F-RCT-01/V03-03.md) |
| V03-04 | Template/notebook/loop option consumer search | yes | passed | [report](../validations/F-RCT-01/V03-04.md) |
| V03-05 | Root tool allowlist chokepoint trace | yes | passed | [report](../validations/F-RCT-01/V03-05.md) |
| V03-06 | Disabled-feature construction inspection | yes | passed | [report](../validations/F-RCT-01/V03-06.md) |
| V03-07 | External memory-store state mapping | yes | passed | [report](../validations/F-RCT-01/V03-07.md) |
| V03-08 | Existing test coverage matrix | yes | passed | [report](../validations/F-RCT-01/V03-08.md) |
| V04-01 | Targeted executable fixtures | no per review instruction | not run; future validation defined | [report](../validations/F-RCT-01/V04-01.md) |
| V05-01 | Current history/comment drift | yes | passed | [report](../validations/F-RCT-01/V05-01.md) |
| V90-01 | Wrong ToolManager path attempt | evidence integrity | inconclusive, not adopted | [report](../validations/F-RCT-01/V90-01.md) |
| V90-02 | Wrong ContextManager path attempt | evidence integrity | inconclusive, not adopted | [report](../validations/F-RCT-01/V90-02.md) |
| V90-03 | Wrong Codex-template path attempt | evidence integrity | inconclusive, not adopted | [report](../validations/F-RCT-01/V90-03.md) |
| V99-01 | Final report/link/executor/source-clean integrity | yes | passed | [report](../validations/F-RCT-01/V99-01.md) |
| V30 | Primary static source reconstruction and acceptance | yes | mixed, final passed | [01](../validations/F-RCT-01/V30-01.md), [02](../validations/F-RCT-01/V30-02.md), [03](../validations/F-RCT-01/V30-03.md), [04](../validations/F-RCT-01/V30-04.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| Runtime override is applied at turn start via `build_system_prompt` | stale | [V03-02](../validations/F-RCT-01/V03-02.md), [V05-01](../validations/F-RCT-01/V05-01.md) |
| Canonical reinjection preserves the system prompt without duplication | current only for exact complete prompt; regressed for project rules | [V03-01](../validations/F-RCT-01/V03-01.md) |
| Working-directory refresh keeps cwd-derived instructions accurate | regressed | [V03-03](../validations/F-RCT-01/V03-03.md) |
| One ToolManager is authoritative for ReAct schemas and execution | current | [V02-01](../validations/F-RCT-01/V02-01.md) |
| External memory store is equivalent to store + enable_memory | regressed for introspection, current for store/tools/recall | [V03-07](../validations/F-RCT-01/V03-07.md) |

## Coverage And Uncertainty

- No Cargo, rustc, tests, builds, Clippy, feature matrix, or dynamic fixture was
  executed, per explicit user and Codex review protocol. V04-01 records future
  regression cases. Static exact-branch/string evidence is conclusive for the
  findings, but timing/provider behavior was not measured.
- The review did not establish whether every application mode reaches every
  builder option; framework public reachability plus bounded EKO call sites was
  sufficient for the reported impacts.
- Direct `AgentConfig` + `ReactAgent::new` and config-file construction provide
  additional validation-bypass surfaces; their complete public-constructor policy
  remains a follow-up question rather than a finding here.
- Concurrent CLI generated-file modifications were not inspected. They do not
  affect Rust call-chain evidence, but primary should resample commits and source
  anchors before accepting.
- Codex primary independently sampled prompt/canonical, root allowlist, feature,
  no-op consumer, and memory-introspection anchors in V30-01..03. The source is
  conclusive and this task is accepted as `complete` without executing builds or tests.

## Handoff

- `F-RCT-02` may rely on the single ToolManager, HookRegistry, budget, and prompt
  ownership call graph, but must not assume the mutation/cardinality defects are
  fixed.
- Iteration synthesis should treat prompt authority and project-rule cardinality
  as one coordinated framework change, while keeping the two finding IDs for
  separate regression acceptance.
- Tool policy work should centralize `AgentConfig.allowed_tools` at snapshot and
  execution choke points and remove `add_tools`-specific authority.
- Feature/no-op cleanup must preserve standalone framework components when they
  are reasonable public APIs; delete only their misleading Agent construction
  integration if no runtime owner is chosen.
- This report becomes stale if builder/config fields, prompt/canonical setters,
  ToolManager replacement/filtering, Cargo feature gates, or external-store
  construction change, or if either reviewed commit changes before acceptance.
- Primary acceptance requires reading this report plus V01-02, V02-01/V02-02,
  V03-01 through V03-08, V05-01, and the explicit not-run boundary V04-01.
