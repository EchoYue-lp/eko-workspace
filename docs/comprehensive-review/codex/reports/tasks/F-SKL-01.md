# F-SKL-01: Skill loading and execution

> Status: complete
> Reviewer: Codex review subagent
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean at start and final source check; only Codex reports changed outside them

## Question

Are Skill definition, discovery precedence, activation, runtime injection,
enable/disable, recovery and cleanup deterministic, bounded and reachable?

## Scope

- Code-based `Skill` trait, root exports, builtin implementations and Agent
  registration lifecycle.
- File-based `SkillLoader`, frontmatter/types, dependency probing,
  `SkillRegistry`, progressive activation/resource/script tools and hooks.
- ReactAgent discovery, policy reconciliation, source unload, catalog/Skill
  projections, allowed-tools, invocation snapshots and checkpoint recovery.
- Narrow EKO bootstrap inspection only to prove framework discovery and baseline
  helper reachability; EKO marketplace/config policy is not reviewed here.
- Existing tests, scoped history and `MASTER-PLAN.md` Skill claims.

## Out Of Scope

- Generic Tool registry collision/execution contracts owned by `F-EXT-01`.
- Canonical system-prompt mutation defects owned by `F-RCT-01`.
- Cross-component plugin activation/rollback owned by `F-PLG-01` and
  application plugin policy owned by `A-PLG-01`.
- General hook event/action correctness, script sandbox policy and EKO SkillsHub
  install/sync security.
- Source fixes, Cargo, rustc, builds, tests or dynamic fixtures.

## Inputs

- Root AGENTS; shared README/REPORTING/TASKS; Codex README and templates.
- Completed Codex dependencies `B-REF-01`, `F-RCT-01` and `F-EXT-01`.
- `F-HITL-01` was named by the assignment but had no authorized Codex report at
  review time; no conclusion depends on it.
- Current source, existing tests, scoped git history and `MASTER-PLAN.md`.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Deterministic discovery/collision, parsed descriptors, dependency validation, activation authority, progressive loading, prompt/tool/hook injection, source ownership, unload cleanup and size budgets belong in `echo-agent`. |
| EKO product policy | Bundled/user/workspace roots, enabled/baseline selections, marketplace/install/sync, routing keywords and local curator state remain application policy. User-supplied Skills are trusted local extensions; this review does not impose cloud/multi-tenant permission gates. |
| Adapter boundary | EKO supplies ordered scopes and a `SkillLoadPolicy`, then invokes framework discovery/reconcile/activate APIs. It must not maintain a second runtime registry or reimplement precedence/dependency traversal. |
| Duplicate search | Searched `Skill`/registry/loader/source/policy/discover/load/activate/reconcile/unregister/shutdown, catalog/projection, allowed-tools, hooks, checkpoint, resource/script and EKO entry points across both repositories. Two live framework registries copy the same file descriptors and own separate activation sets. |
| Migration deletion | Converge file descriptors/activation on one shared registry and delete the copied catalog/progressive state. A code-Skill lifecycle fix must delete the metadata-only registration path or the unreachable `shutdown` contract. Keep public reusable Skill capability even if one EKO path does not use it. |

## Current Path

```text
SkillLoader::discover(ordered scopes)
  -> recursive unsorted read_dir -> parse SKILL.md/hooks -> first name wins
  -> ReactAgent::discover_skills_inner
       -> copy descriptor into catalog SkillRegistry
       -> copy descriptor into progressive SharedRegistry
       -> register hooks
       -> replace one catalog projection
       -> replace activate/resource/script tools over SharedRegistry

activation path A (model tool)
  ActivateSkillTool -> progressive registry.activate -> tool result prompt block
                    -> optional invocation ToolVisibility mutation

activation path B (hook/intent/public API)
  ReactAgent::activate_skill -> catalog registry.activate
                             -> named context projection

turn snapshot
  -> merge activation names/allowed-tools from both registries once
  -> model tool can mutate visibility, but not frozen names/allowlist
  -> final checkpoint persists frozen snapshot names

resume
  -> restore transcript -> mark catalog registry names only
  -> resource/script tools still query progressive registry
```

Positive conclusions:

- Tier-1 catalog and programmatic Tier-2 instructions use replaceable protected
  projections; repeated additive discovery does not append duplicate catalogs.
- File-Skill policy/source unload removes catalog/progressive descriptors,
  activation/sandbox metadata, hooks and instruction projections, then refreshes
  tools and catalog.
- Resource/script paths reject absolute/parent/canonical escapes, resource bytes
  are bounded, descriptor/catalog ordering APIs sort their output, and
  allowed-tools reaches both schema and execution filters on a later snapshot.

## Findings

### F-SKL-01-P1-01: Two activation authorities diverge during a turn and after recovery

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/react/capabilities.rs:659`, `:964`;
  `echo-agent/echo-execution/src/skills/external/activate_tool.rs:122`;
  `echo-agent/src/agent/snapshot.rs:206`, `:604`;
  `echo-agent/src/agent/react/mod.rs:1702`;
  `echo-agent/echo-execution/src/skills/external/resource_tool.rs:92`;
  `echo-agent/echo-execution/src/skills/external/run_script_tool.rs:189`
- Reachability: EKO/framework discovery copies descriptors to both registries;
  model calls the registered activation tool, while hook/intent routing calls the
  public activation API. Every run snapshots both registries and finalization
  saves `snapshot.tools.active_skill_names`; resume marks only the catalog copy.
- Expected invariant: one activation authority atomically controls instructions,
  active allowed-tools, resources/scripts, invocation state and checkpoint
  recovery.
- Observed behavior: model activation writes only the progressive registry;
  programmatic activation and resume write only the catalog registry. Snapshot
  merges them only once before the loop. A model activation can update mutable
  tool visibility but not the frozen allowed-tools/names saved at finalization;
  hook/resumed activation appears active while progressive resource/script tools
  reject it as inactive.
- Impact: Skill constraints can be delayed until a future turn, activations can
  disappear from checkpoints, and a resumed or auto-activated Skill cannot use
  its bundled resources/scripts despite its instructions being active.
- Root cause: a copied registry was introduced to satisfy async tools instead of
  sharing one state owner; invocation snapshot and checkpoint then captured a
  third temporal view.
- Direction: store descriptors and activation in one shared registry. Activation
  should return a typed delta that atomically updates current invocation
  eligibility/names plus projection, and checkpoint should read the authoritative
  state at the save safe point. Delete catalog/progressive state merging.
- Regression validation: activate a restricted resource-bearing Skill via model,
  hook and public API; use it in the same turn, checkpoint, reconstruct Agent and
  use it after resume. All paths must expose identical names/policy/content.
- Validation reports: [V03](../validations/F-SKL-01/V03-01.md),
  [V05](../validations/F-SKL-01/V05-01.md),
  [V08](../validations/F-SKL-01/V08-01.md)

### F-SKL-01-P1-02: Cyclic Skill dependencies recurse without a runtime terminal guard

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-execution/src/skills/external/loader.rs:487`;
  `echo-agent/echo-execution/src/skills/registry.rs:370`, `:468`
- Reachability: any discovered SKILL.md may declare `depends_on`; activation tool
  or public activation recursively activates each unmet dependency.
- Expected invariant: a missing/cyclic required dependency rejects catalog commit
  or activation with a typed, bounded error.
- Observed behavior: discovery only warns and registers invalid graphs. Missing
  dependencies are silently skipped. For A -> B -> A, activation checks only the
  completed `activated` set; neither node is inserted before descendants finish,
  and boxed async recursion has no visiting set/depth limit.
- Impact: activating a malformed local Skill graph never reaches a valid terminal
  result and can exhaust memory/stack-like future depth or hang the Agent turn.
- Root cause: discovery's cycle detector is observational and activation does
  not carry its own traversal state.
- Direction: validate dependencies into an immutable acyclic graph before
  registry commit and return a typed error for missing/cyclic dependencies; keep
  an activation-time visiting set for programmatic descriptors. Delete warning-
  only `validate_and_sort_dependencies` or make it produce the committed order.
- Regression validation: self-cycle, two-node cycle, deep chain, missing
  dependency and diamond graph; every invalid case must terminate without
  partial activation.
- Validation reports: [V04](../validations/F-SKL-01/V04-01.md),
  [V08](../validations/F-SKL-01/V08-01.md)

### F-SKL-01-P1-03: Methodology baseline injection reads a non-existent nested SKILL.md path

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-execution/src/skills/external/types.rs:85`;
  `echo-agent/echo-execution/src/skills/registry.rs:610`, `:627`;
  `echo-agent-cli/echo-agent-app-core/src/runtime.rs:170`
- Reachability: EKO loads bundled descriptors, selects enabled methodology
  baselines at startup, calls the public framework helper, then applies the
  returned prompt and logs successful injection.
- Expected invariant: each enabled methodology baseline reads its descriptor's
  actual SKILL.md body exactly once or returns a visible error.
- Observed behavior: `descriptor.location` is the absolute SKILL.md file, but the
  helper computes `desc.location.join("SKILL.md")`. The read targets
  `.../SKILL.md/SKILL.md`, failure is silently continued, and the caller still
  logs success.
- Impact: the default brainstorming/debugging/verification/planning instructions
  are absent despite configuration and startup telemetry claiming they are
  active, changing core Agent behavior.
- Root cause: descriptor location semantics changed from directory to file while
  this helper retained the old assumption.
- Direction: read `descriptor.location` directly and return a result containing
  injected/missing names. Do not add an EKO-side file reader. F-RCT-01 separately
  owns canonical prompt mutation after the helper returns.
- Regression validation: descriptor at a Unicode/nested path; assert each enabled
  baseline body occurs exactly once and missing files cause a typed visible
  failure.
- Validation reports: [V05](../validations/F-SKL-01/V05-01.md),
  [V08](../validations/F-SKL-01/V08-01.md)

### F-SKL-01-P2-04: Collision winner and same-name rediscovery freshness are not deterministic contracts

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-execution/src/skills/external/loader.rs:120`,
  `:183`; `echo-agent/src/agent/react/capabilities.rs:687`
- Reachability: custom/project roots can contain nested categories or multiple
  source directories; app/plugin reload calls discovery again on a live Agent.
- Expected invariant: every duplicate has a stable precedence key and repeated
  discovery either replaces one owned source atomically or rejects collision.
- Observed behavior: explicit scope order is deterministic, but same-scope
  candidates use unsorted filesystem `read_dir` and first encounter wins. A new
  discovery loader can parse changed same-name content, then ReactAgent silently
  skips it because the old name is installed, preserving stale descriptor, hook,
  activation and projection state.
- Impact: identical trees can select different Skill content across filesystems,
  and ordinary hot reload reports no error while keeping old behavior.
- Root cause: name is both global identity and collision policy; discovery has no
  source/generation reconcile transaction outside plugin-specific unload.
- Direction: sort canonical candidate paths, define source/scope precedence, and
  expose a reconcile API that diffs owned descriptors/hooks/projections. Delete
  silent `is_installed` skipping for same-source refresh; cross-source collisions
  should be typed. Generic Tool-name collisions remain `F-EXT-01`.
- Regression validation: randomized directory creation order and changed
  same-name SKILL.md/hooks; winner and replacement result must be stable.
- Validation reports: [V02](../validations/F-SKL-01/V02-01.md),
  [V08](../validations/F-SKL-01/V08-01.md)

### F-SKL-01-P2-05: Code-based Skill shutdown and unload are unreachable public contracts

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/tools/skill.rs:28`, `:53`;
  `echo-agent/src/agent/react/capabilities.rs:563`;
  `echo-agent/echo-execution/src/skills/registry.rs:550`
- Reachability: any framework consumer can install a code Skill through the
  public Agent API; tools and prompt injection become live immediately.
- Expected invariant: the Agent retains ownership needed to disable/unload a
  Skill, remove its tools/prompt and invoke `shutdown` exactly once.
- Observed behavior: `add_skill` extracts fresh tools/prompt, stores only
  `SkillInfo`, then drops the boxed Skill. There is no production `shutdown`
  caller or code-Skill removal API; removing individual tools cannot reverse
  appended prompt text or invoke the declared lifecycle.
- Impact: consumers cannot safely reload or disable stateful code Skills, and
  the public shutdown hook creates false confidence about resource cleanup.
- Root cause: eager installation discarded the lifecycle object and ownership
  receipts after registration.
- Direction: retain an installed code-Skill record with owned tool names and a
  replaceable prompt marker, provide fallible unload/reload, and call shutdown;
  otherwise delete `shutdown` and document immutable installation.
- Regression validation: stateful Skill with two tools and prompt; unload must
  remove all three registrations, call shutdown once and permit clean re-add.
- Validation reports: [V06](../validations/F-SKL-01/V06-01.md),
  [V08](../validations/F-SKL-01/V08-01.md)

### F-SKL-01-P2-06: Progressive-disclosure size claims are not enforced at discovery or activation

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-execution/src/skills/external/types.rs:65`;
  `echo-agent/echo-execution/src/skills/external/loader.rs:194`, `:345`;
  `echo-agent/echo-execution/src/skills/registry.rs:260`, `:391`, `:428`
- Reachability: every file Skill is read during discovery and again on activation;
  catalog and prompt block are sent to the model; resources are enumerated.
- Expected invariant: Tier 1 metadata, catalog aggregate, Tier 2 body and resource
  listing obey configurable byte/count/token budgets with explicit truncation or
  typed rejection.
- Observed behavior: the documented 64-character name rule only warns and the
  1024-character description maximum is not checked. Whole Skill/hook files,
  skill count, catalog, activation body and resource entry count are unbounded.
  Only individual resource file reads have a byte maximum.
- Impact: one accidental huge local Skill or a large installed collection can
  consume memory and protected model context, defeating progressive disclosure
  and causing provider context failures.
- Root cause: token costs are documentation estimates rather than loader/context
  invariants.
- Direction: add framework-configurable file/metadata/catalog/body/resource-count
  budgets using checked/saturating accounting and typed outcomes. EKO may choose
  defaults, but must not reimplement enforcement.
- Regression validation: oversized Unicode metadata/body, many descriptors and
  resources, exact-boundary controls; assert no panic and deterministic report.
- Validation reports: [V07](../validations/F-SKL-01/V07-01.md),
  [V08](../validations/F-SKL-01/V08-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition/export/feature/duplicate map | yes | passed | [report](../validations/F-SKL-01/V01-01.md) |
| V02 | Discovery precedence/collision/reload | yes | failed | [report](../validations/F-SKL-01/V02-01.md) |
| V03 | Activation/snapshot/recovery authority | yes | failed | [report](../validations/F-SKL-01/V03-01.md) |
| V04 | Dependency graph termination | yes | failed | [report](../validations/F-SKL-01/V04-01.md) |
| V05 | Prompt/tool/permission/hook injection | yes | failed | [report](../validations/F-SKL-01/V05-01.md) |
| V06 | Enable/disable/unload cleanup | yes | failed | [report](../validations/F-SKL-01/V06-01.md) |
| V07 | Path/UTF-8/panic/overflow/size bounds | yes | failed | [report](../validations/F-SKL-01/V07-01.md) |
| V08 | Existing test coverage inventory | yes | failed | [report](../validations/F-SKL-01/V08-01.md) |
| V09 | Historical/reference classification | yes | passed | [report](../validations/F-SKL-01/V09-01.md) |
| V10 | Targeted executable fixtures | policy-deferred | not_run | [report](../validations/F-SKL-01/V10-01.md) |
| V11 | Layering/de-dup/local threat-model gate | yes | passed | [report](../validations/F-SKL-01/V11-01.md) |
| V12 | Report/link/source integrity gate | yes | passed | [report](../validations/F-SKL-01/V12-01.md) |
| V30 | Primary source-anchor sampling and acceptance | yes | passed | [report](../validations/F-SKL-01/V30-01.md) |

Primary static acceptance is recorded in V30. Executable fixtures remain
deliberately deferred under the review-only policy and are regression work for
the implementation phase, not missing evidence for the source-conclusive claims.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| B-REF-01: Skills/Plugins require source ownership, precedence and cleanup | current but incomplete | File plugin Skills have source unload; code Skills and ordinary same-name refresh do not. V02, V06, V09. |
| MASTER-PLAN: Skill catalog uses marker/latest-wins projection | current | `SKILL_CATALOG_PROJECTION` is replaced, not appended. V05, V09. |
| MASTER-PLAN: Skill descriptor source tracking/unload was absent | fixed for file Skills | `source`, `by_source`, both-registry unregister and projection/hook cleanup exist. V06, V09. |
| Framework docs: file Skills provide three-tier progressive disclosure | current in topology, incomplete in bounds | Catalog/activation/resources are distinct, but descriptor/body/count budgets are unenforced. V07, V09. |

## Coverage And Uncertainty

- No Cargo, rustc, test, build or dynamic fixture ran; V10 is `not_run` by
  policy. Current source control flow is sufficient for the six findings, but
  runtime timing must be regression-tested after fixes.
- `F-HITL-01` was unavailable; this task makes no HITL finding. Local user Skill
  execution is not treated as a hostile tenant boundary.
- Script process cancellation/sandbox enforcement belongs to tool/security
  reviews. This task inspected only activation/registry/path reachability.
- `F-PLG-01` must independently review atomic rollback when some plugin component
  kinds fail; source-specific file-Skill cleanup here is only one component.
- F-RCT-01 and F-EXT-01 findings were not duplicated.

## Handoff

- Fix order: one activation registry/safe-point checkpoint -> reject dependency
  cycles -> baseline path/result -> deterministic source reconcile -> code-Skill
  lifecycle -> aggregate budgets.
- Preserve named catalog/Skill projections, source-scoped file unload, sorted
  public listings, path canonicalization and invocation Tool policy choke points.
- `F-PLG-01`, `A-PLG-01` and `X-PLG-01` should consume P1-01/P2-04/P2-05 without
  introducing another registry or application-owned dependency traversal.
- This report becomes stale if `SkillRegistry` ownership, discovery ordering,
  activation tools/API, snapshot/checkpoint fields, baseline helper, or code-Skill
  registration/unload changes.
