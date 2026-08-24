# F-API-01: Public facade and documentation contract

> Status: complete
> Reviewer: Codex primary reviewer
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d35c9a432370b923acabfbb5f32d63`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: both source repositories clean; only Codex report artifacts added

## Question

Do root re-exports, split-crate APIs, examples, rustdoc and Markdown guides
expose one coherent framework contract that an independent consumer can compile
and use without accidental dependencies or guaranteed runtime failure?

## Scope

- All eight framework `lib.rs` surfaces, the root facade/prelude/advanced
  modules, root adapters, and public duplicate names.
- Root README, English/Chinese API guides, example target declarations, and
  rustdoc examples.
- External consumers that declare only `echo_agent` and reproduce EventBus,
  context, prelude, split-import, and factory contracts.
- Doctests for the facade and all seven split crates with all features.

## Out Of Scope

- Runtime semantics inside individual providers, tools, memory, workflow, and
  Subagent implementations; their atomic F tasks own those.
- Feature topology defects already owned by `F-FEAT-01`.
- Macro expansion/hygiene and the split macro facade dependency already owned
  by `F-MAC-01` and `B-ARCH-01-P1-01`.
- Broad documentation freshness already owned by `B-DOC-01`; this task reports
  only concrete public-consumer failures and facade promises.
- Editing source or selecting a compatibility migration. The repository has no
  backward-compatibility requirement.

## Inputs

- Root `AGENTS.md`, shared `README.md`, `REPORTING.md`, and this task card.
- Codex track rules in `codex/README.md`.
- Completed [B-ARCH-01](B-ARCH-01.md), [B-DOC-01](B-DOC-01.md),
  [F-CORE-01](F-CORE-01.md), and [F-FEAT-01](F-FEAT-01.md), used to avoid
  duplicate architecture, EventBus reachability, feature, and general-doc
  findings.
- No report from another reviewer directory was read.

## Layering Decision

The split crates are valid independent framework packages and may expose
domain-specific APIs. The root facade owns the single-dependency experience it
advertises: imports in facade documentation must either resolve through the
facade or explicitly declare a split-crate dependency. Rustdoc examples belong
to their defining package and cannot reverse-import the facade. EKO policy is
not involved.

Duplicate search covered all public type/trait/enum names across the seven
library layers, their facade aliases/re-exports, definitions and real callers.
Namespace-specific duplicates such as MCP/LSP JSON-RPC types, A2A/task states,
Subagent inheritance `MemoryScope`, and task/workflow checkpoint stores describe
different wire or domain contracts and are not findings. The generic builder
trait is deliberately surfaced as `AgentBuilderTrait` while the facade alias is
`AgentBuilder`. Only the public default factory has a guaranteed-failure
implementation under a capability-identical name.

## Current Path

The root package includes README as crate rustdoc (`src/lib.rs:23`), publishes
always-on facade modules plus feature-gated capabilities, and exposes curated
`prelude` and `advanced` modules (`src/lib.rs:28-331`). Domain tool modules live
in `echo_tools` and are registered through `register_all_tools`; facade
`tools` re-exports selected modules, while prelude re-exports only Think plus
two web and two media tool types (`src/lib.rs:162-180`).

The live event transport is `EventEnvelope`; `EventBus::send` accepts that type
and has no `BusEvent` or `send_for_run` (`src/event_bus.rs:10-45`). The English
and Chinese config guides still describe the superseded interface. The context
guide describes an older `ContextSources`, `ContextBudget`, and selector API,
while current code exposes different fields and consuming signatures
(`src/context/mod.rs:33-252`, `selector.rs:10-105`).

Seven split crates compile/publish independently. Their crate docs generally
say most users should depend on `echo_agent`; nevertheless 22 bilingual guide
files contain 62 direct `echo_core`/`echo_tools`/other split imports while the
installation guide declares only `echo-agent`. Rust does not make transitive
crates importable by name.

## Findings

### F-API-01-P2-01: Public EventBus guide only compiles against a removed contract

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/docs/en/28-config-reference.md:558`,
  `echo-agent/docs/zh/28-config-reference.md:556`,
  `echo-agent/src/event_bus.rs:10`
- Reachability: both public guides prescribe a facade-only consumer; the exact
  example was compiled against current `echo_agent` with no default features.
- Expected invariant: a versioned public transport guide names current types,
  constructs the required envelope identity/sequence, and compiles.
- Observed behavior: `BusEvent` is unresolved, `send_for_run` is absent,
  `send(AgentEvent)` has the wrong type, and received `Arc<EventEnvelope>` has no
  `event` field. The guide also promises direct `run_id`/`agent_id` filtering
  using the old wrapper.
- Impact: an independent observer cannot implement the documented integration;
  following the guide yields four compile errors before reaching the already
  reported lack of a live producer.
- Root cause: `dba349e` replaced the old bus event shape with the versioned
  envelope but the 2026-06 guides were never migrated.
- Direction: after the EventBus authority decision in `F-CORE-01-P1-02`, either
  delete this unrealized API/guide or document an instance-scoped live envelope
  observer with explicit identity, lag and terminal semantics. Do not revive a
  parallel `BusEvent` wrapper.
- Regression validation: compile the bilingual canonical example as an
  external crate and observe envelopes from a real Agent run.
- Validation reports: [V02 corrected](../validations/F-API-01/V02-02.md),
  [V03](../validations/F-API-01/V03-01.md),
  [V04 reproduced](../validations/F-API-01/V04-02.md)

### F-API-01-P2-02: Context-system guide specifies an API that no longer exists

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/docs/en/40-context-system.md:20`,
  `echo-agent/docs/zh/40-context-system.md:20`,
  `echo-agent/src/context/mod.rs:33`, `echo-agent/src/context/mod.rs:81`,
  `echo-agent/src/context/selector.rs:32`, `echo-agent/src/llm.rs:108`
- Reachability: the guide is an English/Chinese public framework chapter; its
  basic budget/source/selector consumer was compiled with only `echo_agent`.
- Expected invariant: named fields, imports, method receivers and selector
  signatures match the current public types.
- Observed behavior: the probe produces eight independent compiler errors:
  private `llm::Message`, two absent budget fields, three absent source fields,
  owned-versus-borrowed `assemble`, and missing `select_files`. Later guide
  sections also show nonexistent builder integration.
- Impact: the chapter cannot teach the current framework; consumers must
  reverse-engineer source and may design around a component explicitly not used
  by the default ReactAgent path.
- Root cause: the guide froze a proposed/older context model while source moved
  to developer/project rules, optional memory text, owned assembly, and
  `score_files`/`select_relevant`; no Markdown compile gate exists.
- Direction: rewrite both languages from current public types and clearly state
  that `ContextAssembler` is a custom-loop building block, not ReactAgent's
  current context authority. Delete obsolete fields/methods from the guide
  rather than recreating a parallel context system.
- Regression validation: extract and compile each canonical guide block with
  only documented dependencies; add a field-level fixture matching the live
  example.
- Validation reports: [V02 corrected](../validations/F-API-01/V02-02.md),
  [V05 corrected](../validations/F-API-01/V05-02.md),
  [V06 reproduced](../validations/F-API-01/V06-02.md)

### F-API-01-P2-03: Facade documentation silently requires undeclared split-crate dependencies

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/docs/en/getting-started.md:41`,
  `echo-agent/docs/en/28-config-reference.md:544`,
  `echo-agent/echo-core/src/lib.rs:18`
- Reachability: getting-started declares only `echo-agent`; 22 English/Chinese
  guide files contain 62 imports from split crates. A facade-only consumer of
  the model-window example was compiled.
- Expected invariant: after the advertised installation, examples import only
  the declared facade, or the page explicitly adds each independent split crate.
- Observed behavior: the consumer fails `E0433` because `echo_core` is not a
  declared dependency. The 62 imports span core 24, tools 16, state 12,
  integration 4, orchestration 4, and execution 2.
- Impact: many official examples fail for normal facade users even when the
  underlying API exists; users are pushed toward accidental workspace paths and
  version skew.
- Root cause: workspace-internal imports leaked into facade docs, while the
  facade intentionally omits some APIs and even omits tools/macros from its
  `workspace` escape hatch as already reported by B-ARCH.
- Direction: choose per example: re-export a genuinely common API through a
  coherent facade path, or explicitly document the independently versioned
  split dependency. Do not delete useful split APIs merely because the CLI does
  not use them.
- Regression validation: compile every Markdown block in two dependency modes:
  documented facade-only and explicitly documented split-crate use.
- Validation reports: [V02 corrected](../validations/F-API-01/V02-02.md),
  [V07](../validations/F-API-01/V07-01.md),
  [V08 reproduced](../validations/F-API-01/V08-02.md)

### F-API-01-P2-04: Prelude's advertised all-tool surface omits registered tool types

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/README.md:181`, `echo-agent/src/lib.rs:162`,
  `echo-agent/echo-tools/src/registry.rs:198`
- Reachability: README is included as crate rustdoc and claims all 67 registered
  tools are accessible through one prelude import; a `full` external consumer
  attempted representative file, shell and Git registered types.
- Expected invariant: the documented one-import public surface makes the
  advertised registered tool types nameable, or accurately says that prelude
  exposes only common traits/builders.
- Observed behavior: `ReadFileTool`, `ShellTool`, and `GitStatusTool` are all
  unresolved after `use echo_agent::prelude::*`. Prelude exports Think and four
  optional web/media types, while other domains require module imports and some
  tools modules are not facade-re-exported at all.
- Impact: the primary crate documentation overstates the facade and sends
  consumers to guess module paths; feature enablement alone does not fulfill the
  promised API.
- Root cause: registry count/capability marketing and curated prelude exports
  are maintained as unrelated lists.
- Direction: make the README honest: keep prelude curated and document canonical
  module paths/automatic registration. Avoid flooding prelude with dozens of
  collision-prone concrete types merely to preserve a false sentence.
- Regression validation: compile the advertised quick path and a metadata-driven
  table of each registered tool's canonical facade/module path.
- Validation reports: [V02 corrected](../validations/F-API-01/V02-02.md),
  [V09 accepted attempt](../validations/F-API-01/V09-04.md)

### F-API-01-P2-05: Facade and core rustdoc gates already fail on current APIs

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/testing/mod.rs:24`,
  `echo-agent/echo-core/src/compression.rs:40`,
  `echo-agent/echo-core/src/plugin/mod.rs:130`
- Reachability: rustdoc is part of the published packages and was run with all
  features for the workspace and each split crate.
- Expected invariant: all non-ignored rustdoc examples compile inside their
  defining package.
- Observed behavior: facade doctest fails because `CompressionInput` gained
  required `focus_instructions` but the testing example omitted it. Independent
  core doctest fails because a core example imports `echo_agent::paths`, a
  reverse dependency unavailable to core. Five split crates pass; macros and
  tools technically pass while executing zero examples (10 and 1 ignored).
- Impact: the documented submission gate is red, docs.rs/package confidence is
  reduced, and split-crate consumers receive a core example that cannot compile
  in its own crate.
- Root cause: public struct changes and cross-layer examples are not covered by
  an enforced workspace rustdoc gate; liberal `ignore` hides additional drift.
- Direction: fix both examples at their owning layer (`focus_instructions: None`;
  core-only plugin setter example or valid facade example elsewhere), then
  classify and progressively enable ignored compile examples. Do not make core
  depend on the facade.
- Regression validation: run all eight per-package doctest commands and require
  zero failures; separately track ignored count with an intentional allowlist.
- Validation reports: [V10](../validations/F-API-01/V10-01.md),
  [V11](../validations/F-API-01/V11-01.md),
  [V12](../validations/F-API-01/V12-01.md),
  [V13](../validations/F-API-01/V13-01.md),
  [V14](../validations/F-API-01/V14-01.md),
  [V15](../validations/F-API-01/V15-01.md),
  [V16](../validations/F-API-01/V16-01.md),
  [V17](../validations/F-API-01/V17-01.md)

### F-API-01-P2-06: A public default factory is guaranteed to fail

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/agent/factory.rs:150`,
  `echo-agent/src/agent/default_factory.rs:32`,
  `echo-agent/src/agent/default_factory.rs:54`
- Reachability: `echo_core::agent` publicly re-exports
  `DefaultAgentFactory`; the facade additionally exposes it as
  `CoreDefaultAgentFactory`. An external facade consumer invoked it with its own
  public default config.
- Expected invariant: a public concrete type named Default either implements the
  advertised trait usefully or is not exported as an implementation.
- Observed behavior: every `create_agent` invocation returns an error telling
  callers to use the separate facade implementation. The external probe exits
  zero only because it prints this guaranteed error.
- Impact: split-crate and facade consumers can select a compile-valid public
  implementation that can never create an Agent; two same-named implementations
  obscure the real authority.
- Root cause: core needed to own the trait/config but also retained a placeholder
  concrete type despite being unable to construct the facade ReactAgent.
- Direction: delete the core placeholder and `CoreDefaultAgentFactory` re-export;
  keep only the trait/config in core and the concrete default implementation in
  facade. No compatibility shim is required.
- Regression validation: compile-fail import of the removed core placeholder and
  a facade factory consumer that constructs a mock/offline agent successfully.
- Validation reports: [V01](../validations/F-API-01/V01-01.md),
  [V18 reproduced](../validations/F-API-01/V18-02.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Public export/duplicate definition and reachability map | yes | passed | [V01](../validations/F-API-01/V01-01.md) |
| V02 | Public docs/import/feature consistency inventory | yes | failed invariant after invalid-path attempt | [V02-01](../validations/F-API-01/V02-01.md), [V02-02](../validations/F-API-01/V02-02.md) |
| V03 | EventBus guide-to-source mapping | yes | failed invariant | [V03](../validations/F-API-01/V03-01.md) |
| V04 | Facade-only EventBus external compile | yes | failed invariant twice | [V04-01](../validations/F-API-01/V04-01.md), [V04-02](../validations/F-API-01/V04-02.md) |
| V05 | Context guide-to-source field/method mapping | yes | failed invariant after invalid-path attempt | [V05-01](../validations/F-API-01/V05-01.md), [V05-02](../validations/F-API-01/V05-02.md) |
| V06 | Facade-only context external compile | yes | failed invariant twice | [V06-01](../validations/F-API-01/V06-01.md), [V06-02](../validations/F-API-01/V06-02.md) |
| V07 | Split-crate import inventory | yes | failed invariant | [V07](../validations/F-API-01/V07-01.md) |
| V08 | Facade-only split-import external compile | yes | failed invariant twice | [V08-01](../validations/F-API-01/V08-01.md), [V08-02](../validations/F-API-01/V08-02.md) |
| V09 | Prelude external compile | yes | failed after two environment attempts | [V09-01](../validations/F-API-01/V09-01.md), [V09-02](../validations/F-API-01/V09-02.md), [V09-03](../validations/F-API-01/V09-03.md), [V09-04](../validations/F-API-01/V09-04.md) |
| V10 | Workspace all-feature doctest command | yes | failed | [V10](../validations/F-API-01/V10-01.md) |
| V11 | `echo_core` doctest | yes | failed | [V11](../validations/F-API-01/V11-01.md) |
| V12 | `echo_macros` doctest | yes | passed, all ignored | [V12](../validations/F-API-01/V12-01.md) |
| V13 | `echo_execution` doctest | yes | passed | [V13](../validations/F-API-01/V13-01.md) |
| V14 | `echo_tools` doctest | yes | passed, all ignored | [V14](../validations/F-API-01/V14-01.md) |
| V15 | `echo_integration` doctest | yes | passed | [V15](../validations/F-API-01/V15-01.md) |
| V16 | `echo_state` doctest | yes | passed | [V16](../validations/F-API-01/V16-01.md) |
| V17 | `echo_orchestration` doctest | yes | passed | [V17](../validations/F-API-01/V17-01.md) |
| V18 | Core default-factory external behavior | yes | failed invariant twice | [V18-01](../validations/F-API-01/V18-01.md), [V18-02](../validations/F-API-01/V18-02.md) |
| V19 | Historical claim/source evolution | yes | passed | [V19-01](../validations/F-API-01/V19-01.md), [V19-02](../validations/F-API-01/V19-02.md), [V19-03](../validations/F-API-01/V19-03.md) |
| V20 | Final report integrity/source isolation | yes | passed after preserved self-reference failure and diagnostics | [V20-01](../validations/F-API-01/V20-01.md), [V20-02](../validations/F-API-01/V20-02.md), [V20-03](../validations/F-API-01/V20-03.md), [V20-04](../validations/F-API-01/V20-04.md), [V20-05](../validations/F-API-01/V20-05.md), [V20-06](../validations/F-API-01/V20-06.md), [V20-07](../validations/F-API-01/V20-07.md) |

Failed invariant validations mean the public contract is false and are captured
as findings. V09-01/V09-02 are environmental attempts, not product failures.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| README: one prelude import exposes all 67 tools | regressed/false | P2-04; V09-03 |
| Config guide: global `BusEvent` supports direct run filtering | stale after `dba349e` | P2-01; V03/V04 |
| Context-system field and builder integration | stale | P2-02; V05/V06 |
| Split crate docs: most users should depend only on facade | current intent, contradicted by guides | P2-03; V07/V08 |
| Core default factory is supplied by facade | current intent, misleading placeholder remains | P2-06; V18 |
| B-ARCH facade/workspace/macro findings | current, not duplicated | V01 and dependency report |
| B-DOC broad current-doc drift | current and concretized here | P2-01 through P2-05 |

## Coverage And Uncertainty

All eight package doctest commands reached terminal exit results. This task did
not compile every Markdown code block; it compiled four representative public
consumer contracts after a repository-wide import/field inventory. Ignored
rustdoc blocks remain unverified, especially macros and tools where zero tests
actually ran. The prelude consumer required one network-enabled dependency
fetch after sandbox and offline failures; only its final compiler errors support
P2-04. External probes and their targets live in `/private/tmp` and may be
removed after evidence capture.

The duplicate-name inventory is not itself a defect list. Task/workflow
checkpoint stores and memory/inheritance scopes were judged distinct contracts;
later subsystem reviews may independently find convergence issues in behavior.

## Handoff

- `F-MAC-01` should add real compile-pass/fail coverage because all ten macro
  rustdocs are ignored; consume B-ARCH's macro layering finding.
- `F-EXT-01..03` should define canonical facade module paths for registered
  tools rather than expand prelude indiscriminately.
- `Q-FW-01` must include per-package doctests; `Q-FW-02` should fail on ignored
  coverage drift; `Q-DOC-01` should compile bilingual Markdown snippets with
  exactly documented dependencies.
- `S-FW-01` may merge P2-01 with F-CORE bus authority only as one remediation
  epic while retaining both reachability and public-contract evidence.
- This report becomes stale when root/split exports, README/include behavior,
  the inspected guides, rustdoc snippets, EventBus/context/factory contracts,
  or reviewed commits change.
