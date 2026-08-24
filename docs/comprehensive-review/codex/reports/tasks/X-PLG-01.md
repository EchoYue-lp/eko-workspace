# X-PLG-01: Skill/plugin/hook lifecycle conformance

> Status: complete
> Reviewer: Codex review subagent
> Executor: Codex review subagent
> Accepted by: Codex primary reviewer
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: framework externally dirty and inspected only through committed
> HEAD; CLI externally dirty only at `Cargo.lock`, which was excluded; reports only

## Question

Are framework lifecycle primitives and EKO activation policy joined through
reversible, source-scoped, failure-safe adapters?

## Scope

- Framework committed plugin registry/integrator/lifecycle, Skill source
  reconcile, Hook source identity, MCP receipts and Subagent registration APIs.
- EKO `PluginRuntimeService`, prepared application components, load/reload/
  enable/disable/configure/uninstall paths, lifecycle callback bracketing,
  rollback, LSP/monitor replacement and live surface entry points.
- Component ownership and generation across the primary Agent, existing pooled
  Agents and future pooled Agents.
- Stale Tool/Hook/Skill/Subagent registration, Skill-based intent routing and
  current static test coverage.

## Out Of Scope

- Source fixes and all Cargo/rustc/test/build/dynamic fixture/network execution.
- Framework plugin identity, manifest multiplicity, install/discovery validity,
  `wire_all` transactionality, MCP receipt identity and lifecycle API shape,
  which remain owned by F-PLG-01.
- Framework Skill activation dual authority, dependency cycles, rediscovery and
  code-Skill unload, which remain owned by F-SKL-01.
- EKO durable mutation compensation, output-style propagation, queued Hook
  generation and direct Skill enable persistence, which remain owned by
  A-PLG-01. X-SRF-01 remains canonical for cross-surface output style.
- General MCP/LSP transport behavior, scheduler correctness, plugin UI design,
  third-party extension hardening and any online-service threat model.

## Inputs

- Root `AGENTS.md`; shared README, REPORTING, TASKS exact X-PLG-01 card; Codex
  README and report templates.
- Exact Codex dependencies [F-SKL-01](F-SKL-01.md),
  [F-PLG-01](F-PLG-01.md), and [A-PLG-01](A-PLG-01.md). No other reviewer
  directory or non-dependency task report informed the atomic findings.
- Current CLI source at pinned HEAD excluding dirty `Cargo.lock`; all framework
  source read from committed HEAD blobs because its live worktree was dirty.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Source identity, dependency ordering, lifecycle callbacks, reversible component receipts, Skill/Hook/MCP reconcile and generic Subagent registration belong in `echo-agent`; these APIs remain independently useful even when EKO has a projection defect. |
| EKO product policy | Which plugin scopes are active, construction of EKO Subagents, LSP/monitor/theme/style policy, AgentPool membership and live reload UX belong in `echo-agent-cli`. |
| Adapter boundary | One EKO-owned generation transaction should prepare once, apply framework source-scoped receipts to the primary and every pool identity, update per-Agent projections/classification, then commit or restore the prior generation. It must not implement a second generic plugin registry. |
| Duplicate search | Searched both repositories for PluginRegistry/Integrator/RuntimeService, Wired/Prepared components, source tags, register/unregister/disconnect, Skill descriptors, Subagent registries/factories, ToolManager/HookRegistry sharing, pool refresh/construction, intent classifiers/router and all live mutation commands. Framework searches used committed HEAD only. |
| Migration deletion | Preserve the single PluginRuntimeService and framework integrator. Replace and delete additive-only `refresh_skill_descriptors` and the bootstrap-frozen classifier assembly once a source-owned generation reconcile is live. Do not add a second plugin store, share EKO policy into the framework, or retain parallel refresh paths. |

EKO is a trusted local personal assistant. No permission gate or public-network
threat model is recommended for user-installed extensions; this review concerns
framework correctness, failure recovery and stale local runtime state.

## Current Path

```text
AgentRuntime::bootstrap
  -> primary Agent
  -> PluginRuntimeService::new -> initial reload into primary
       framework: Skill + Hook + MCP receipts
       EKO: Subagent + LSP + monitor + theme + output style + lifecycle
  -> snapshot current Skill descriptors into Keyword/LLM classifiers
  -> install one IntentRouter on primary

later GUI/headless startup
  -> AgentPool::from_runtime
       shares ToolManager + HookRegistry Arcs
       copies Skill descriptor Vec once
       does not share/project plugin Subagent registry or IntentRouter

live plugin mutation
  Tauri command -> one PluginRuntimeService
    prepare -> deactivate -> replace monitor -> unload primary
    -> wire/register primary -> swap LSP -> activate -> commit/event
    -> no pool generation update
    -> no classifier/router rebuild

pool execution
  GUI chat/channel/scheduler/TaskRuntime/background -> pool.acquire
    existing Agent: shared Tool/Hook update, old Skill snapshot, no plugin Subagent update
    future Agent: old pool Skill Vec, independent Subagent registry, no EKO router
```

Positive boundaries to preserve:

- Plugin load and every live command converge on one `PluginRuntimeService`;
  bootstrap no longer has a second wiring owner.
- The adapter prepares application components and replacement LSP before its
  main live replace, brackets lifecycle callbacks, restores monitor/LSP/
  components on modeled failures, and has focused primary-Agent rollback tests.
- Framework Skill source tags and `HookSource::Plugin` make primary cleanup
  explicit. Shared ToolManager and HookRegistry also give pooled Agents live
  Tool/Hook changes without manual copying.

## Findings

### X-PLG-01-P1-01: Plugin generations split between primary and pooled Skill/Subagent state

- Priority: P1; confidence: high; layer: adapter.
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/runtime.rs:276`,
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs:131`,
  `:804`, `:1157`,
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/plugin_components.rs:444`,
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:94`,
  `:219`, `:233`, `:493`, `:824`, `:882`, `:934`,
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/panels.rs:447`.
- Reachability: initial plugin wiring mutates the primary before GUI/headless
  pool construction. Live Tauri mutation commands call the same service later,
  while GUI chats, channels, scheduler runs, TaskRuntime and background work use
  pool Agents. PluginRuntimeService has no pool handle/callback.
- Expected invariant: a successful plugin transition exposes one source-owned
  component generation in the primary, every existing pool Agent and every
  future pool Agent; disable/uninstall removes that generation everywhere.
- Observed behavior: ToolManager and HookRegistry are shared and update live.
  Skill descriptors are copied into a pool Vec at construction. Plugin runtime
  never refreshes it; the unrelated GUI Skill refresh is additive for existing
  Agents and cannot remove absent descriptors. SubagentRegistry is per Agent,
  and plugin Subagents are registered only on the primary. Existing and future
  pooled Agents can therefore retain old plugin Skills and omit plugin
  Subagents while exposing current Tools/Hooks.
- Impact: a plugin can appear enabled and work through one Agent identity but be
  missing or stale in GUI conversation, channel, scheduled, background or task
  execution. Disable/uninstall can leave pool Skills visible; behavior depends
  on surface and Agent creation time.
- Root cause: EKO treats primary mutation as the plugin commit while its runtime
  actually mixes shared registries with unversioned per-Agent snapshots. There
  is no application generation owner spanning primary and pool construction.
- Direction: make PluginRuntimeService coordinate one prepared, source-scoped
  generation receipt with AgentPool. Reconcile Skills and EKO Subagent
  definitions/factories in the primary, all existing pool Agents and future pool
  state before committing. Preserve shared Tool/Hook registries and generic
  framework APIs. Delete additive `refresh_skill_descriptors` and all ad hoc
  plugin/Skill pool refresh call sites after cutover; do not introduce a second
  plugin registry or blindly share product-specific Agent instances.
- Regression validation: initial load, changed Skill/Subagent reload, disable,
  uninstall and injected rollback with primary, existing conversation,
  future conversation, channel, scheduler, background and TaskRun Agents; assert
  one source/generation for Tool/Hook/Skill/Subagent and exact cleanup.
- Validation reports: [V01](../validations/X-PLG-01/V01-01.md),
  [V02](../validations/X-PLG-01/V02-01.md),
  [V03](../validations/X-PLG-01/V03-01.md),
  [V05](../validations/X-PLG-01/V05-01.md),
  [V07](../validations/X-PLG-01/V07-01.md)

### X-PLG-01-P1-02: Plugin Skill reload leaves the live intent-routing catalog frozen at bootstrap

- Priority: P1; confidence: high; layer: adapter.
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/runtime.rs:294`,
  `:298`, `:320`, `:325`, `:340`, `:351`, `:362`,
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs:198`,
  `:551`, `:790`,
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:824`,
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/src/agent/react/run/react_loop.rs:623`,
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/src/agent/react/run/stream_channel.rs:181`
  at committed framework HEAD.
- Reachability: after initial plugin load, bootstrap copies every current Skill's
  triggers/description into KeywordClassifier/LlmIntentClassifier, wraps them in
  TriggerSupervisor and installs IntentRouter. Streaming and non-streaming ReAct
  execute that router. Tauri exposes later reload/enable/disable/install/
  configure/uninstall, but none replaces the router.
- Expected invariant: successful plugin Skill add/change/remove updates the
  automatic routing catalog in the same committed generation, across every
  Agent identity that advertises that Skill.
- Observed behavior: AgentRuntime stores a clone of the bootstrap keyword
  classifier that has no later reader/updater. PluginRuntimeService has no
  classifier/router handle. Newly loaded Skills never enter descriptor-based
  auto-routing; removed or renamed Skills remain classifier candidates until
  restart. Pool construction does not install the EKO router either.
- Impact: live plugin management can truthfully show a Skill loaded while user
  prompts do not auto-activate it, or can route toward a removed Skill and fall
  back/fail. Primary and pool behavior also diverges independent of plugin
  enabled state.
- Root cause: mutable plugin descriptors were adapted into an immutable
  one-time classifier value rather than a generation-aware live provider or
  replaceable runtime projection.
- Direction: have the same application plugin generation rebuild/swap one
  immutable router snapshot for primary, existing pool and future pool Agents,
  or make classification consult a live authoritative Skill descriptor source.
  Delete the unused stored `keyword_classifier` and bootstrap-only parallel
  assembly after the canonical path is live. Do not put EKO pool policy into the
  generic framework classifier.
- Regression validation: add, change triggers, rename and remove a plugin Skill
  after pool construction; classify on streaming/non-streaming primary,
  existing and future pooled Agents and assert exact generation plus no removed
  name. Repeat across failed reload and restart.
- Validation reports: [V02](../validations/X-PLG-01/V02-01.md),
  [V06](../validations/X-PLG-01/V06-01.md),
  [V07](../validations/X-PLG-01/V07-01.md)

## Validation Matrix

| ID | Claim or execution | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Commit and dirty-source isolation | yes | passed | [report](../validations/X-PLG-01/V00-01.md) |
| V01 | Cross-repository component ownership/duplicate map | yes | passed | [report](../validations/X-PLG-01/V01-01.md) |
| V02 | Registration and production reachability trace | yes | passed | [report](../validations/X-PLG-01/V02-01.md) |
| V03 | Load/reload/disable/unload lifecycle trace | yes | passed | [report](../validations/X-PLG-01/V03-01.md) |
| V04 | Failure rollback and canonical finding ownership | yes | passed | [report](../validations/X-PLG-01/V04-01.md) |
| V05 | Stale Tool/Hook/Skill/Subagent pool registration search | yes | failed | [report](../validations/X-PLG-01/V05-01.md) |
| V06 | Skill classifier/router generation refresh | yes | failed | [report](../validations/X-PLG-01/V06-01.md) |
| V07 | Existing static test coverage | yes | failed | [report](../validations/X-PLG-01/V07-01.md) |
| V08 | Dependency drift and finding de-duplication | yes | passed | [report](../validations/X-PLG-01/V08-01.md) |
| V09 | Integrated dynamic generation/rollback fixture | future | not run per instruction | [report](../validations/X-PLG-01/V09-01.md) |
| V99 | Report/link/executor/source-boundary integrity | yes | passed | [report](../validations/X-PLG-01/V99-01.md) |
| V30 | Primary committed-source acceptance | yes | passed | [report](../validations/X-PLG-01/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| F-SKL-01-P1-01 two activation authorities diverge | current accepted dependency | V08; retained by F-SKL-01 and not copied into X findings |
| F-SKL-01-P2-04 same-name rediscovery can remain stale | current accepted dependency | V08; plugin-to-pool generation is a separate adapter defect |
| F-SKL-01-P2-05 code-Skill unload is unreachable | current accepted dependency | V08; no deletion is inferred from EKO usage |
| F-PLG-01 plugin identity/manifest/install/wiring/receipt/lifecycle findings | current | V01, V03, V04, V08; canonical ownership retained by F-PLG-01 |
| A-PLG-01-P1-01 durable mutation compensation failures | current | V04, V08; canonical ownership retained by A-PLG-01 |
| A-PLG-01-P1-02 output style reaches only primary | current | V05, V08; explicitly excluded from new X findings |
| A-PLG-01-P1-03 queued Hook events lack registry generation | current | V01, V08; no duplicate Hook-queue finding opened |
| A-PLG-01-P1-04 direct Skill enable persistence can lie | current | V05, V08; distinct from plugin generation propagation |

## Coverage And Uncertainty

- No dynamic lifecycle or fault-injection fixture ran; V09 records the future
  validation without pretending it passed. Static call graphs make both new
  findings source-conclusive.
- Current dirty framework bodies and diffs were never read; anchors refer to
  commit `3aa7929`. Any change to PluginIntegrator, ReactAgent Skill/Subagent
  capabilities, Hook source identity or intent routing makes this report stale.
- CLI `Cargo.lock` was excluded. No dependency-resolution claim relies on it.
- The review did not quantify eventual external side effects of native lifecycle
  callbacks. F-PLG-01 owns their generic transaction semantics.
- Whole-generation EKO prepared component replacement is positive but does not
  prove framework `wire_all` or durable registry mutation atomic; dependency
  findings remain open.

## Handoff

- Primary review should sample V05 against `AgentPool::from_runtime`,
  `refresh_skill_descriptors`, `create_agent` and plugin Subagent registration;
  then sample V06 against bootstrap ordering and both committed ReAct paths.
- Downstream synthesis may rely on exactly two new P1 adapter findings. Merge
  output-style propagation with its existing A-PLG/X-SRF owner and keep
  F-SKL/F-PLG primitive findings canonical.
- The iteration unit should be one EKO plugin capability generation spanning
  primary plus pool and one replaceable router snapshot. Acceptance requires
  add/change/remove/rollback equivalence; deletion requires removal of the
  additive refresh and unused bootstrap classifier field.
- Changes to either pinned commit, plugin runtime/pool construction, framework
  component receipts, Skill reconcile, Subagent registry or IntentRouter require
  revalidation before implementation planning.
