# A-SUB-01: EKO Subagent catalog, pool, and prompt compilation

> Status: complete
> Reviewer: Codex review subagent
> Accepted by: Codex primary reviewer
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: CLI clean at evidence collection; framework had concurrent
> dirty paths excluded by V00; only Codex A-SUB-01 reports were written

## Question

Does EKO add role definitions and product prompt/routing policy while reusing
one framework Subagent lifecycle and one immutable effective executable catalog
across the primary Agent, existing pool Agents, and future pool Agents?

## Scope

- Builtin, project, user, and plugin Subagent definition discovery, collision,
  and precedence.
- Default Task role routing and its production reachability.
- EKO system-prompt compilation, section cardinality, language policy, and
  plugin-role parity.
- Pre-construction versus successfully registered catalog authority.
- Plugin reload propagation to primary, existing pool, and future pool Agents.
- Task/registry/execution identity and existing static test inventory.

## Out Of Scope

- Generic framework registry atomicity, executable readiness, per-Agent local
  catalog projection, prompt context/result, attachments, and terminal errors,
  already owned by `F-SUB-01`.
- Sync/Fork/Teammate/Team lifecycle, background execution, isolation, checkpoint,
  and Team policy, owned by `F-SUB-02`.
- General workspace/config resource transitions owned by `A-CFG-01`.
- Generic Tool execution and application Tool exposure, source fixes, shared
  index edits, frontend rendering, Cargo, rustc, tests, builds, fixtures, runtime
  launch, and network.

## Inputs

- Root `AGENTS.md`; shared `README.md`, `REPORTING.md`, and exact `TASKS.md` card;
  Codex reviewer protocol.
- Authorized complete Codex dependencies `F-SUB-01`, `F-SUB-02`, and
  `A-CFG-01`.
- Current clean CLI source at the revision above. Framework conclusions use the
  allowed complete dependencies; current concurrent framework dirty contents
  and diffs were not inspected or adopted.
- V09 discloses an accidental read of two unauthorized Codex task reports. No
  technical content from those reports supports this report. Primary acceptance
  therefore requires independent source sampling.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Atomic definition/readiness/executable identity, immutable registry catalog revisions, generic prompt-compiler interface, dispatch, and terminal Subagent identity belong to `echo-agent`. |
| EKO product policy | `.eko/subagents` source precedence, builtin roles, domain default routes, EKO prompt sections/language, plugin activation, and primary/pool propagation policy belong to `echo-agent-cli`. |
| Adapter boundary | EKO should resolve and prepare one product definition set, compile each with the EKO policy, then commit one executable catalog generation through framework registration. It must not own a second registry or scheduler. |
| Duplicate search | Searched role definition/name/source, loader/merge, prompt compiler/system prompt, catalog snapshot/capability, default route, registry/dispatch, plugin reload, pool create/update, execution ID, and tests across both repositories; current framework dirty contents were excluded. |
| Migration deletion | After one effective catalog generation exists, delete independent pre-build `SubagentCatalogSnapshot` copies and updateless prompt/Task projections. Do not preserve a second catalog for compatibility or add a parallel pool registry. |

## Current Path

```text
create_agent
  -> discover project / user / builtin definitions
  -> pre-build SubagentCatalogSnapshot
       -> primary/pool system prompt
       -> default-route startup validation
       -> pool Task capability snapshot
  -> create shared framework Subagent registry + EKO prompt compiler
  -> construct each role best-effort
       -> success: register instance/factory
       -> failure: warn and continue
  -> primary post-bootstrap Task tools reuse the pre-build snapshot

AgentPool::from_runtime -> shares primary ToolManager
  -> future pool create_agent independently rediscovers disk definitions
       -> compiles system prompt and Task snapshot
       -> later replaces ToolManager with primary shared manager

plugin reload
  -> primary registry and local dispatch catalog mutate
  -> no primary system-prompt/Task-snapshot generation replacement
  -> no existing-pool or future-pool definition/prompt generation transition

Task capability role -> framework Subagent registry/executor
  -> execution_id -> EKO subagent_run_id projection
```

Positive conclusions:

- Cross-scope precedence is explicit: project overrides user, which overrides
  builtin. Plugin definitions use a separate duplicate-rejecting activation path.
- Production Task omission routes through centralized `default_subagent_for`,
  startup name validation, and candidate Task capability validation. The DTO
  `PlanTask::default` value `general` is not the live omission route.
- EKO delegates execution to the framework registry/executor and projects the
  concrete `execution_id` as `subagent_run_id`; pool Agent identities do not
  introduce a second Subagent execution identity.

## Findings

### A-SUB-01-P1-01: Plugin reload cannot commit one catalog generation across primary and pool Agents

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs:201`,
  `:804`, `:1157`; `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:188`,
  `:824`; `echo-agent-cli/echo-agent-app-core/src/infra.rs:233`;
  `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/register.rs:26`.
- Reachability: registered plugin reload commands mutate the live primary Agent;
  GUI conversations and background/channel/task paths acquire existing or future
  pool Agents through the inspected pool construction path.
- Expected invariant: a successful plugin generation exposes the same role
  names, prompt descriptions, Task-authoring validator, and executable identities
  to the primary Agent, every existing pool Agent, and every subsequently created
  pool Agent.
- Observed behavior: reload mutates the primary registry and its local dispatch
  catalog but does not replace the primary system prompt or immutable Task
  capability snapshots and has no pool refresh. Existing pool prompts remain
  old. Future pool Agents rediscover only disk roles for their prompt/Task
  snapshots, then receive the primary shared ToolManager containing plugin roles.
- Impact: after add/replace/remove, Agents can advertise removed roles, omit live
  plugin roles, reject Tasks that execution could dispatch, or emit Tasks for a
  role unavailable in that Agent's explanatory catalog. Behavior depends on
  surface and Agent creation time.
- Root cause: EKO represents effective catalog state as independent immutable
  copies plus mutable registries, without a product-owned generation commit or
  AgentPool propagation boundary.
- Direction: prepare one EKO effective definition generation, register/validate
  it, then atomically replace prompt, Task capability, and executable projections
  for primary and pool construction; explicitly update or retire existing pool
  Agents. Delete the independent pre-build/updateless snapshots once replaced.
- Regression validation: plugin add/replace/remove/failure across primary,
  already-created conversation/background Agents, and future pool Agents; assert
  identical generation, role enum, prompt cardinality, Task validation, and
  dispatch result.
- Validation reports: [V05](../validations/A-SUB-01/V05-01.md).

### A-SUB-01-P1-02: Plugin Subagents bypass the EKO prompt compiler

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/infra.rs:626`;
  `echo-agent-cli/echo-agent-app-core/src/subagent_prompt.rs:177`;
  `echo-agent-cli/echo-agent-app-core/src/plugin_components.rs:511`.
- Reachability: default/project/user role construction calls the EKO compiler;
  every activated plugin Subagent follows the separate component build path.
- Expected invariant: every EKO Subagent definition contributes one role body
  and passes exactly once through the same product compiler, including language,
  protocol, capabilities, result-quality, suggestions, and result-contract
  sections.
- Observed behavior: plugin markdown is assigned directly as the system prompt.
  Delegation-capable plugin roles additionally install the framework default
  compiler rather than EKO's compiler. The EKO product sections are absent.
- Impact: plugin roles can answer in a different language/format, omit required
  evidence and structured result behavior, and delegate under a different
  contract from builtin/project/user roles even though the product presents one
  Subagent catalog.
- Root cause: plugin activation builds a concrete Agent separately from the
  canonical EKO role compiler adapter.
- Direction: route plugin definitions through the same EKO compiler and keep the
  framework compiler as a generic fallback for unrelated consumers. Delete the
  raw-system-prompt branch after field/cardinality parity is proven.
- Regression validation: compile equivalent builtin/project/user/plugin roles;
  assert each required system section occurs once, language overrides match, and
  delegation invocation compilation preserves the EKO contract.
- Validation reports: [V03](../validations/A-SUB-01/V03-01.md).

### A-SUB-01-P1-03: Startup advertises discovered roles before proving they are executable

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent-cli/echo-agent-app-core/src/infra.rs:233`, `:489`,
  `:612`; `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/register.rs:26`;
  `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:824`.
- Reachability: every application Agent construction builds the system prompt
  and validates default routes from discovered definitions before per-role Agent
  construction; individual construction failures warn and skip.
- Expected invariant: every role advertised to model-facing prompts or accepted
  by Task authoring is ready in the committed executable catalog, or required
  role construction fails startup atomically.
- Observed behavior: prompt and both primary/pool Task snapshots use all
  discovered roles. Later role build failures merely skip registration. The
  same pre-build `subagent_catalog_snapshot` Arc is passed to primary Task
  capability after registration; pool construction repeats the sequence.
- Impact: the model can select a role that never registered, yielding avoidable
  execution failure; required/default roles can be missing while startup appears
  successful, and prompt versus Task validation can disagree.
- Root cause: discovery, construction readiness, and catalog publication are
  separate phases with publication occurring first.
- Direction: prepare all candidate Agents, treat required/default role failure
  as startup failure, and commit only successfully executable optional roles in
  one generation. Delete pre-construction model/Task catalog publication.
- Regression validation: inject provider/model/prompt/tool construction failure
  for required, default, and optional roles; assert atomic startup policy and
  exact equality among prompt, Task capability, registry, and dispatch.
- Validation reports: [V04](../validations/A-SUB-01/V04-02.md).

### A-SUB-01-P2-04: Same-scope duplicate role definitions resolve by filesystem iteration order

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/subagent_loader.rs:262`,
  `:350`, tests at `:804`; plugin duplicate handling at
  `echo-agent-cli/echo-agent-app-core/src/plugin_components.rs:96`.
- Reachability: application startup recursively scans project and user
  Subagent directories before every Agent construction, including future pool
  Agents.
- Expected invariant: duplicate names inside one scope are rejected with both
  source paths or resolved by a documented stable precedence.
- Observed behavior: recursive `read_dir` entries are unsorted, and same-name
  definitions silently overwrite a `HashMap` entry. Cross-scope precedence is
  explicit and tested; plugin preparation separately rejects duplicates.
- Impact: identical files can select different prompt/model/tool policy across
  filesystems, restarts, or independently constructed pool Agents, making the
  effective role definition unreproducible.
- Root cause: scope merging defines overwrite behavior without first defining
  stable source identity/collision policy.
- Direction: reject same-scope duplicates and report both canonical paths;
  alternatively sort canonical paths and document precedence, but do not retain
  silent last-insertion wins.
- Regression validation: nested same-name files created in opposing orders and
  across supported filesystems/scopes; assert deterministic error/selection and
  stable primary/future-pool catalog identity.
- Validation reports: [V01](../validations/A-SUB-01/V01-01.md).

## Validation Matrix

| ID | Claim | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Revision and concurrent-dirty isolation | yes | passed | [report](../validations/A-SUB-01/V00-01.md) |
| V01 | Definition precedence and duplicate collision | yes | failed -> finding | [report](../validations/A-SUB-01/V01-01.md) |
| V02 | Default route definition and live reachability | yes | passed with V04 limitation | [report](../validations/A-SUB-01/V02-01.md) |
| V03 | Plugin EKO prompt compiler/cardinality/language | yes | failed -> finding | [report](../validations/A-SUB-01/V03-01.md) |
| V04-01 | Delegated catalog publication attempt (retained factual deviation) | retained | inconclusive | [report](../validations/A-SUB-01/V04-01.md) |
| V04-02 | Corrected catalog publication after executable readiness | yes | failed -> finding | [report](../validations/A-SUB-01/V04-02.md) |
| V05 | Reload generation across primary/existing/future pool | yes | failed -> finding | [report](../validations/A-SUB-01/V05-01.md) |
| V06 | Registry/execution identity and layering | yes | passed with inherited deviations | [report](../validations/A-SUB-01/V06-01.md) |
| V07-01 | Wildcard test inventory attempt | retained failure | inconclusive | [report](../validations/A-SUB-01/V07-01.md) |
| V07-02 | Corrected explicit static test inventory | yes | passed with gaps | [report](../validations/A-SUB-01/V07-02.md) |
| V08 | Dynamic reload/failure regression matrix | future | not_run by direction | [report](../validations/A-SUB-01/V08-01.md) |
| V09 | Dependency-report isolation | required disclosure | inconclusive | [report](../validations/A-SUB-01/V09-01.md) |
| V99-01 | Static report integrity preflight | retained failure | failed: self-report absent | [report](../validations/A-SUB-01/V99-01.md) |
| V99-02 | Static report integrity final gate | yes | passed | [report](../validations/A-SUB-01/V99-02.md) |
| V30 | Independent primary source sampling and acceptance | yes | passed | [report](../validations/A-SUB-01/V30-01.md) |

## Historical Claim Status

| Dependency claim | Classification | Current evidence |
|---|---|---|
| `F-SUB-01-P1-01` atomic registry record/readiness | current framework issue; not duplicated | V04, V06 |
| `F-SUB-01-P1-02` per-Agent mutable catalog projection | current framework issue; A-SUB only deepens EKO generation/pool propagation | V05 |
| Other `F-SUB-01` prompt context/result/attachment/terminal findings | current dependency conclusions; outside this task | V03, V06 only delimit ownership |
| `F-SUB-02` execution-mode and Team lifecycle findings | current dependency conclusions; not duplicated | V06 |
| `A-CFG-01-P1-02` non-atomic workspace generation across primary/pool | current general lifecycle pattern; A-SUB catalog generation is a distinct product capability transition | V05 |

## Coverage And Uncertainty

- No Cargo, rustc, test, build, dynamic fixture, application launch, or network
  command was run. V08 records the required future runtime matrix.
- Static control-flow evidence is strong for construction order, prompt compiler
  selection, reload ownership, pool construction, duplicate handling, and
  identity projection. Exact filesystem iteration, construction failure, and
  reload race behavior remains dynamically unexecuted.
- Concurrent framework dirty contents were excluded. Generic framework claims
  are inherited from the allowed completed dependencies and must be resampled at
  the current framework revision if primary acceptance depends on changed paths.
- V07-01 remains immutable and supports no coverage claim. V09 made this task
  ineligible for self-acceptance; V30 independently rebuilt the four evidence
  chains. V04-02 also corrects the delegated primary-catalog provenance error.

## Handoff

- First establish one framework executable catalog revision/readiness contract
  under `F-SUB-01`; then make EKO prepare and commit one product catalog generation
  to primary and pool consumers without adding a second registry.
- Route plugin roles through `EkoSubagentPromptCompiler`, reject same-scope
  collisions, and publish prompts/Task capability only after readiness.
- Remove old pre-build catalog snapshots and raw plugin prompt construction when
  the new generation path owns all real callers; preserve the current
  `execution_id -> subagent_run_id` mapping.
- Primary sampling is complete in V30. Implementation-time dynamic catalog
  generation/reload regressions remain required but do not block static review.
