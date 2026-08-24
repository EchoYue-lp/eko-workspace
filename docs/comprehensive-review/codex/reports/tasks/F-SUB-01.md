# F-SUB-01: Subagent definitions, registry, and prompt context

> Status: complete
> Reviewer: Codex primary reviewer, with isolated subagent evidence
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d35c9a432370b923acabfbb5f32d63`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: `echo-agent` clean; externally owned concurrent changes were disclosed at `echo-agent-cli/web-frontend/src/generated/ApiError.ts` and `StreamingEvent.ts`, neither read nor modified by this review; the final status snapshot returned clean; review reports are outside the source repositories

## Question

Do Subagent definition, executable identity, model-facing catalog, role prompt,
context/tool/attachment selection, and terminal result form one coherent
framework contract on real dispatch paths?

## Scope

- `echo-agent/src/agent/subagent`: definition/builder, registry/factory,
  context/builder, prompt compiler, executor boundary, and typed result parser.
- `echo-agent/src/tools/builtin/agent_dispatch.rs`: catalog schema, ToolContext
  identity/cancel conversion, request construction, and parent result projection.
- Root ReactAgent construction and registration/delegation capabilities.
- EKO default/plugin Agent construction and TaskRuntime result consumption only
  where needed to prove framework reachability and adapter impact.
- Static searches for duplicate authority, panic-prone indexing, and UTF-8
  truncation across both repositories.

## Out Of Scope

- Source fixes or API changes.
- Sync/Fork/Teammate/Team scheduling, isolation cleanup, retry, and complete event
  lifecycle (`F-SUB-02`).
- EKO catalog source precedence, role routing, pool refresh, and product prompt
  policy (`A-SUB-01`).
- Task DAG execution and claims (`F-TSK-03` / application TaskRuntime tasks).
- Generic Tool registry/execution defects already covered by `F-EXT-01`.
- Plugin activation rollback beyond its effect on the Subagent catalog
  (`F-PLG-01` / `A-PLG-01`).
- Cargo, rustc, tests, builds, or dynamic fixtures. V12 records the future
  executable matrix without claiming it ran.

## Inputs

- Root `AGENTS.md`; shared `README.md`, `REPORTING.md`, and `TASKS.md`; Codex
  reviewer protocol.
- Dependency [F-RCT-01](F-RCT-01.md): the accepted construction boundary is one
  shared registry/executor/hook set per built Agent graph.
- Dependency [F-CORE-01](F-CORE-01.md): typed identity, cancellation, error, and
  terminal semantics are the accepted generic boundary.
- Current source and scoped git history. No other reviewer directory/report was
  read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Definition identity, atomic registration/readiness, executable catalog snapshots, prompt/context transfer, attachment preservation, runtime terminal status, and result provenance are reusable framework mechanisms. |
| EKO product policy | Role markdown precedence, readonly/writer selection, default role set, EKO prompt sections, domain routing, TaskRuntime acceptance, plugin choice, and UI catalog rendering remain application policy. |
| Adapter boundary | EKO may compile a role definition into a concrete Agent and provide product payloads/tool subsets, but the adapter must produce one effective definition and must not own a second registry revision, execution-mode default, history slicer, terminal classifier, or artifact-provenance authority. |
| Duplicate search | Searched definition/builder fields, registry maps/factories, catalog handles/snapshots, context builders/inheritance, prompt compilers, result/output schemas, dispatch requests, and all registration/delegation callers across both repositories. The EKO source definition/catalog are legitimate product adapters; dormant framework context/output APIs and per-Agent mutable catalog vectors are overlapping authorities. |
| Migration deletion | Consolidate registry records and catalog revision in framework. Preserve the EKO compiler/source adapter. Remove inert definition/context/output fields only after proving they are not reasonable public framework options; otherwise make them effective through the one factory/dispatch contract. Delete local catalog mutation and duplicate inheritance slicing once the shared snapshot/policy is authoritative. |

## Current Path

```text
EKO .md / plugin definition
  -> EKO builds concrete ReactAgent + factory
  -> framework SubagentDefinition
  -> ReactAgent::register_subagent_with_definition
  -> SubagentRegistry {agents, definitions, factories} (separate maps)
  -> ReactAgent local agent_tool catalog Vec (separate projection)

model agent_tool call
  -> ToolContext -> ExternalRunContext + child cancellation token
  -> agent_name/task/mode/constraints
  -> parent context: Fresh or fixed Fork preset
  -> DispatchRequest {mode_override: Some(...), message: None}
  -> SubagentExecutor -> registry definition + Agent/factory
  -> prompt compiler -> task/history
  -> AgentInvocationContext -> streaming Agent
  -> runtime-observed evidence + model ## Result
  -> SubagentOutcome -> JSON ToolResult / events / EKO TaskRuntime result
```

Positive invariants are substantial. Runtime context preserves conversation,
run, turn/message, execution, trace, and invocation cancellation identities
(`agent_dispatch.rs:142`). Programmatic multimodal dispatch replaces only text
parts and retains binary attachments (`prompt.rs:189`). History filtering drops
incomplete tool/reasoning turns and retains whole safe messages (`prompt.rs:149`).
The result parser owns terminal status, bounds all model arrays/text with
UTF-8-safe character iteration, downgrades model verification to `Reported`,
and merges separately observed checks/files/tool artifacts
(`types.rs:452`, `executor.rs:1197`). No production `unwrap`, `expect`, panic,
unreachable branch, direct string byte slice, or unchecked vector index was
found in the scoped paths.

## Findings

### F-SUB-01-P1-01: Registry updates can bind a definition to the wrong executable

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/subagent/registry.rs:139`,
  `echo-agent/src/agent/subagent/registry.rs:161`,
  `echo-agent/src/agent/subagent/registry.rs:190`,
  `echo-agent/src/agent/subagent/registry.rs:219`,
  `echo-agent/src/agent/subagent/registry.rs:298`,
  `echo-agent/src/agent/subagent/registry.rs:318`
- Reachability: ReactAgent registration uses `register_sync`; EKO registers each
  default/plugin role first as a prebuilt Agent and then as a factory. Plugin
  reload repeats same-name registration.
- Expected invariant: one successful name registration atomically identifies
  its definition and executable/factory; a failed update changes nothing and
  ordinary duplicate registration cannot silently cross-bind versions.
- Observed behavior: instances, definitions, and factories live under separate
  locks and are updated in separate critical sections. `register_sync` returns
  true and emits `Registered` after inserting the instance even if definition
  insertion fails. Definition-only registration overwrites only the definition;
  `get_agent` returns an older cached instance first. Every map insert silently
  replaces its same-name value.
- Impact: lookup/catalog can describe one role/model/prompt/tool/mode while
  dispatch executes a previous instance or fails for no executable. Concurrent
  reads can observe mixed revisions, and plugin reload order determines identity.
- Root cause: a logical registration is split across three maps without a
  revisioned record or conflict contract.
- Direction: store one atomic name record containing definition, readiness, and
  instance/factory generation; make ordinary duplicate registration typed and
  explicit, then delete split-map promotion/replacement paths.
- Regression validation: sync lock contention after instance acquisition;
  async concurrent read/replace/remove; definition-only promotion; duplicate
  plugin/default name; factory failure and retry; prove catalog and execution
  generation agree.
- Validation reports: [V01](../validations/F-SUB-01/V01-01.md),
  [V02](../validations/F-SUB-01/V02-01.md),
  [V03](../validations/F-SUB-01/V03-01.md),
  [V10](../validations/F-SUB-01/V10-01.md)

### F-SUB-01-P1-02: Per-Agent catalog projections are neither snapshots nor executable sets

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/react/capabilities.rs:327`,
  `echo-agent/src/agent/react/capabilities.rs:376`,
  `echo-agent/src/agent/react/capabilities.rs:426`,
  `echo-agent/src/tools/builtin/agent_dispatch.rs:389`,
  `echo-agent-cli/echo-agent-app-core/src/infra.rs:812`,
  `echo-agent-cli/echo-agent-app-core/src/plugin_components.rs:444`,
  `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs:1157`
- Reachability: every delegation-capable Agent owns a local schema vector.
  Built-in child Agents receive a one-time sync; plugin registration/unload
  changes the shared registry and the primary Agent while already-created child
  Agents remain live.
- Expected invariant: a model-facing catalog is one immutable revision of roles
  that can execute, and replacement/removal updates every consumer coherently.
- Observed behavior: `sync_subagent_dispatch_catalog` only upserts and never
  clears missing names; registry events do not refresh other Agents.
  Definition-only roles are advertised before readiness. EKO does not resync
  built-in children after plugin changes, and delegation-capable plugin Agents
  are constructed with an empty local catalog. Unregister only edits the Agent
  on which it was called.
- Impact: a child can be told no role exists, invoke a removed/stale role, or be
  offered a definition that deterministically fails at execution. Nested
  delegation and hot reload are inconsistent across Agents sharing one registry.
- Root cause: synchronous Tool schema generation introduced mutable local
  vectors without a registry revision/readiness contract.
- Direction: generate every schema from one immutable executable registry
  snapshot, or replace all projections by revision through one broadcast. Delete
  local upsert-only mutation and exclude non-ready definitions from executable
  enum values (expose readiness separately for discovery).
- Regression validation: initial/default/plugin add, replace, failed hydration,
  unload, rollback, and child created before/after each revision; compare schema
  names to dispatchable generation.
- Validation reports: [V02](../validations/F-SUB-01/V02-01.md),
  [V04](../validations/F-SUB-01/V04-01.md),
  [V10](../validations/F-SUB-01/V10-01.md)

### F-SUB-01-P1-03: Builtin dispatch overrides registered mode and cannot honor history declarations

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/subagent/types.rs:137`,
  `echo-agent/src/agent/subagent/types.rs:149`,
  `echo-agent/src/agent/subagent/context.rs:70`,
  `echo-agent/src/agent/subagent/context.rs:253`,
  `echo-agent/src/tools/builtin/agent_dispatch.rs:207`,
  `echo-agent/src/tools/builtin/agent_dispatch.rs:241`,
  `echo-agent/src/tools/builtin/agent_dispatch.rs:266`,
  `echo-agent/src/agent/subagent/executor.rs:449`,
  `echo-agent/src/agent/subagent/executor.rs:1117`
- Reachability: EKO registers normal roles as Fork and team definitions as Team;
  the model-facing tool describes `mode` as optional and sends every call through
  this normalization.
- Expected invariant: omitted mode uses `SubagentDefinition.execution_mode`;
  `Some(0)` history means all and `Some(n)` means the last `n` eligible messages.
- Observed behavior: omitted or invalid mode becomes Sync and is sent as an
  explicit override, except isolation flags force Fork. An omitted Team default
  therefore runs Sync instead of team dispatch. Explicit Fork snapshots only
  the last two raw messages before applying the definition limit, so `n > 2`
  cannot work. `Some(0)` means empty in `from_parent` but unlimited in
  `filter_history`.
- Impact: declared team/default execution can silently select the wrong runtime;
  custom consumers cannot obtain their requested parent history, and identical
  configuration values mean different things at two stages.
- Root cause: execution default, inheritance choice, parent snapshot, and
  prompt filtering are separate authorities.
- Direction: keep omitted mode as `None`, derive one effective definition/policy,
  then snapshot/filter history once with a single zero/all semantic. Invalid enum
  input must be rejected, not normalized to omitted.
- Regression validation: omitted Sync/Fork/Teammate/Team; explicit overrides;
  isolation forcing; history None/0/1/2/10 with filtered tool turns and Unicode.
- Validation reports: [V05](../validations/F-SUB-01/V05-01.md),
  [V10](../validations/F-SUB-01/V10-01.md),
  [V11](../validations/F-SUB-01/V11-01.md)

### F-SUB-01-P1-04: Primary model dispatch drops the active request's attachments

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/llm/types.rs:90`,
  `echo-agent/src/agent/subagent/prompt.rs:149`,
  `echo-agent/src/agent/subagent/prompt.rs:189`,
  `echo-agent/src/tools/builtin/agent_dispatch.rs:266`,
  `echo-agent/src/tools/builtin/agent_dispatch.rs:286`,
  `echo-agent/src/agent/subagent/executor.rs:1148`,
  `echo-agent/src/agent/react/mod.rs:2481`
- Reachability: `agent_tool` is the LLM-callable ad hoc delegation surface on the
  main Agent and any delegation-capable child. Fresh context is its default.
- Expected invariant: a delegation made while handling an attachment-bearing
  user request forwards the original Message, or explicitly reports that the
  target cannot receive attachments.
- Observed behavior: the low-level multimodal API is correct, but builtin
  `agent_tool` always sends `message: None`. Fresh context contains no history;
  only the text parent goal/task remains. Fork can clone a multimodal history
  message only when it also contains text, but that is opt-in and not the active
  request transport.
- Impact: a specialized Subagent asked to inspect an image/file/audio receives
  only a textual assignment and can return a plausible but ungrounded result.
- Root cause: the current invocation Message is absent from `ToolContext`, so
  builtin dispatch cannot use the already-implemented attachment path.
- Direction: carry an invocation-scoped active Message/attachment reference into
  the dispatch tool and reuse `with_compiled_task`; do not encode binary content
  into prompt text or duplicate attachment storage.
- Regression validation: real trait-object `agent_tool` calls with file/image/
  audio plus text, attachment-only message, Fresh/Fork, retry/delegate, and
  provider serialization; verify bytes/URLs and task text are preserved once.
- Validation reports: [V07](../validations/F-SUB-01/V07-01.md),
  [V10](../validations/F-SUB-01/V10-01.md),
  [V11](../validations/F-SUB-01/V11-01.md)

### F-SUB-01-P1-05: Builtin dispatch collapses timeout and running cancellation into permanent failures

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent/src/agent/subagent/executor.rs:569`,
  `echo-agent/src/agent/subagent/executor.rs:636`,
  `echo-agent/src/agent/subagent/executor.rs:1501`,
  `echo-agent/src/tools/builtin/agent_dispatch.rs:301`,
  `echo-agent/src/tools/builtin/agent_dispatch.rs:327`,
  `echo-agent/echo-core/src/tools/mod.rs:372`
- Reachability: cancellation during Sync/Fork waits and timeout return executor
  errors; both blocking and background `agent_tool` branches catch them.
- Expected invariant: parent tool output and Subagent events agree on Cancelled,
  TimedOut, Failed, or Completed, with one suitable typed Tool failure category.
- Observed behavior: an `Ok(SubagentResult)` preserves its structured outcome,
  including stream-emitted cancellation. Every `Err`, including typed
  cancellation and timeout, becomes `ToolResult::error`, which always carries
  `Permanent`; the structured terminal outcome emitted by the executor is not
  included in the parent result.
- Impact: the parent cannot distinguish retryable timeout, user cancellation,
  missing role, or permanent execution failure. Model recovery, UI/tool status,
  and event/tool projections can disagree for one execution.
- Root cause: the adapter serializes only successful `Result` values and uses a
  generic string constructor for every error variant.
- Direction: project one structured terminal envelope for both `Ok` and `Err`,
  and map executor error/status to typed Tool failure without string matching.
- Regression validation: cancel before start/during stream, timeout, Agent
  error, missing role, malformed result, background start failure, and completion
  race; assert one terminal and matching event/tool categories.
- Validation reports: [V08](../validations/F-SUB-01/V08-01.md),
  [V10](../validations/F-SUB-01/V10-01.md)

### F-SUB-01-P1-06: Model-reported existing files are attested as artifacts produced by this execution

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/subagent/types.rs:383`,
  `echo-agent/src/agent/subagent/types.rs:456`,
  `echo-agent/src/agent/subagent/types.rs:600`,
  `echo-agent/src/agent/subagent/executor.rs:1373`,
  `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:685`,
  `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:737`
- Reachability: every normal framework dispatch parses the model's `## Result`;
  EKO TaskRuntime also calls the same parser and accepts framework artifacts for
  required-artifact completion.
- Expected invariant: runtime-generated availability/hash/producer facts prove
  that this execution produced or authoritatively observed the artifact.
- Observed behavior: hydration accepts any readable regular absolute path or
  working-directory-relative file, hashes it, marks it available, and overwrites
  `producer_execution_id` with the current execution. No write/tool observation
  or pre-run baseline is required. EKO acceptance treats these three fields as
  hard artifact evidence.
- Impact: a role can cite a pre-existing or unrelated file and cause a required
  artifact to pass as newly produced, so task completion/evidence is false even
  though bytes and hash are real.
- Root cause: file existence/integrity and production provenance are collapsed
  into one hydration step; model-reported and runtime-observed artifacts share
  the same type without provenance strength.
- Direction: represent `reported`, `resolved-existing`, and
  `observed-produced` separately. Assign producer identity only from observed
  write/artifact events or an isolation diff/baseline; TaskRuntime must require
  that stronger provenance for generated artifacts.
- Regression validation: pre-existing same-name file, absolute external file,
  symlink, file created during execution, tool-log artifact, missing file,
  changed-after-hash race, and required-artifact acceptance.
- Validation reports: [V09](../validations/F-SUB-01/V09-01.md),
  [V10](../validations/F-SUB-01/V10-01.md)

### F-SUB-01-P2-01: Definition capability fields are descriptive metadata, not an effective contract

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/subagent/types.rs:125`,
  `echo-agent/src/agent/subagent/builder.rs:112`,
  `echo-agent/src/agent/subagent/builder.rs:124`,
  `echo-agent/src/agent/subagent/builder.rs:130`,
  `echo-agent/src/agent/subagent/builder.rs:136`,
  `echo-agent/src/agent/subagent/builder.rs:148`,
  `echo-agent/src/agent/subagent/executor.rs:1672`,
  `echo-agent-cli/echo-agent-app-core/src/infra.rs:620`
- Reachability: framework consumers publicly construct these fields; EKO
  manually mirrors many into concrete Agents and registers both definition and
  factory.
- Expected invariant: a declarative model/prompt/tool/iteration/token/memory/
  delegation capability either builds the executable through one authority or
  is validated against the supplied Agent/factory.
- Observed behavior: `tool_filter` and `lightweight` have no production reads.
  Definition `inherit_memory` does not select parent-context policy, and captured
  stores/tool definitions are not installed into the Agent. Model, system
  prompt, max iterations, token limit, and delegation are independently wired
  by EKO factories, with no framework consistency check. Invocation
  `allowed_tools` only disables Fork tools, not Sync/Teammate.
- Impact: public API values can look authoritative while doing nothing or
  disagreeing with the executable. Framework consumers can unintentionally run
  broader tools or different resource limits than their registered definition.
- Root cause: the type describes both build inputs and runtime policy while the
  factory is documented as the real construction owner.
- Direction: define one effective factory product and validate its capabilities
  at registration. Implement universally enforceable runtime fields centrally;
  remove inert fields only after the required framework-public-API reuse check.
- Regression validation: each field on prebuilt and factory registration,
  replacement mismatch, all execution modes, and schema/catalog projection of
  the effective value.
- Validation reports: [V01](../validations/F-SUB-01/V01-01.md),
  [V06](../validations/F-SUB-01/V06-01.md),
  [V10](../validations/F-SUB-01/V10-01.md)

### F-SUB-01-P2-02: Public context/output APIs form a dormant second contract

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/subagent/context.rs:115`,
  `echo-agent/src/agent/subagent/context.rs:127`,
  `echo-agent/src/agent/subagent/context.rs:163`,
  `echo-agent/src/agent/subagent/context_builder.rs:14`,
  `echo-agent/src/agent/subagent/context_builder.rs:189`,
  `echo-agent/src/agent/subagent/mod.rs:25`,
  `echo-agent/src/agent/subagent/types.rs:329`
- Reachability: all types are public framework exports. Repository-wide search
  found no non-test construction of `ContextBuilder`/`SubagentOutput` and no
  live consumer of `OutputSchema`; real dispatch/result uses prompt compiler,
  `SubagentContext`, and `SubagentOutcome`.
- Expected invariant: a public context/result model is either the runtime
  contract or a clearly separate, implemented utility with lossless conversion.
- Observed behavior: `SubagentOutput` has findings/evidence/files/
  recommendations/blockers/confidence and configurable projection, while the
  live outcome has status/artifacts/verification/remaining/touched files. There
  is no conversion. Several ContextBuilder fields are never rendered/consumed,
  and `MemoryScope::Relevant` explicitly behaves as None.
- Impact: consumers can build/serialize a plausible Subagent context/result that
  the executor silently ignores and cannot return through events or
  `agent_tool`; future changes have two incompatible shapes to maintain.
- Root cause: an earlier scoped-context/output design remained public after the
  typed runtime contract evolved elsewhere.
- Direction: first determine whether these are reasonable independent public
  framework options. If yes, connect them losslessly to the canonical context/
  outcome; if no, delete them and their tests/re-exports rather than preserving
  two authorities.
- Regression validation: compile-time public API inventory plus round-trip of
  every retained field through dispatch, event, `agent_tool`, and application
  adapter.
- Validation reports: [V01](../validations/F-SUB-01/V01-01.md),
  [V06](../validations/F-SUB-01/V06-01.md),
  [V10](../validations/F-SUB-01/V10-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate-authority search | yes | failed | [V01-01](../validations/F-SUB-01/V01-01.md) |
| V02 | Definition-to-registration and live reachability trace | yes | passed | [V02-01](../validations/F-SUB-01/V02-01.md) |
| V03 | Registry identity/atomicity inspection | yes | failed | [V03-01](../validations/F-SUB-01/V03-01.md) |
| V04 | Catalog revision/readiness route | yes | failed | [V04-01](../validations/F-SUB-01/V04-01.md) |
| V05 | Mode and history inheritance matrix | yes | failed | [V05-01](../validations/F-SUB-01/V05-01.md) |
| V06 | Role prompt/tool/memory/permission field matrix | yes | failed | [V06-01](../validations/F-SUB-01/V06-01.md) |
| V07 | Attachment-preservation trace | yes | failed | [V07-01](../validations/F-SUB-01/V07-01.md) |
| V08 | Result/error/cancel terminal trace | yes | failed | [V08-01](../validations/F-SUB-01/V08-01.md) |
| V09 | Artifact provenance and TaskRuntime acceptance trace | yes | failed | [V09-01](../validations/F-SUB-01/V09-01.md) |
| V10 | Existing test/invariant coverage inspection | yes | failed | [V10-01](../validations/F-SUB-01/V10-01.md) |
| V11 | Scoped historical drift check | yes | failed | [V11-01](../validations/F-SUB-01/V11-01.md) |
| V12 | Targeted executable matrix | conditional | not_run | [V12-01](../validations/F-SUB-01/V12-01.md) |
| V13 | Report integrity, anchors, findings, and source-dirty gate | yes | passed | [V13-01](../validations/F-SUB-01/V13-01.md) |
| V30 | Primary source-anchor acceptance | yes | passed | [V30-01](../validations/F-SUB-01/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `e124eea`: definition `inherit_history` overrides parent slicing | regressed | Parent factory first caps Fork to two messages; V05/V11 |
| `39cc8b8`: Fresh inheritance is independent from Fork execution | current in intent, partial in adapter | Explicit Fork/Fresh paths exist, but omitted mode overwrites definition; V05 |
| `bd86c54`: multimodal delegation preserves attachments | current at low-level API, partial at builtin surface | `with_compiled_task` is lossless; `agent_tool` sends None; V07/V11 |
| `8f7904f`: one prompt compiler owns framing | current for EKO default role construction/dispatch | `infra.rs:627`, `executor.rs:1117`; dormant ContextBuilder fields remain outside it |
| `5fa2036`: invocation cancellation reaches Subagent child | current | `agent_dispatch.rs:164`; V08 identifies projection loss, not token detachment |
| `954004c`: runtime owns structured terminal result status | current in parser/events, partial at Tool adapter | `types.rs:452`; executor Err becomes permanent generic ToolResult; V08 |

## Coverage And Uncertainty

- The review is static by instruction. Lock interleavings, trait-object
  attachment dispatch, reload behavior, malformed envelopes, and cancel/timeout
  races require future execution; V12 is not evidence.
- Team coordination and mode lifecycle were read only far enough to prove the
  selected definition/adapter route; F-SUB-02 owns their complete behavior.
- EKO source precedence/pool refresh and plugin lifecycle remain downstream
  ownership. The current report uses them only as concrete callers.
- External framework consumers are not present in these repositories.
  Therefore deletion of public `ContextBuilder`/`SubagentOutput` or definition
  fields requires the framework public-option judgment mandated by AGENTS.md.
- Scoped UTF-8/panic inspection found no production defect; executable Unicode
  cases remain future validation.
- The primary reviewer disclosed concurrent external edits to
  `echo-agent-cli/web-frontend/src/generated/ApiError.ts` and
  `StreamingEvent.ts`. This review did not read, modify, or revert them. The
  final Git status snapshot returned no dirty entry; V13 retains the disclosure
  because that external state changed concurrently.

## Handoff

- F-SUB-02 may rely on: registry lookup can return mixed definition/executable
  generations; builtin calls always provide a mode override; terminal executor
  errors are degraded only at the Tool adapter; low-level attachment transport
  is correct when `DispatchRequest.message` is Some.
- A-SUB-01 must read V04-V07 before judging catalog/pool/prompt refresh. It should
  separate EKO role policy from the framework snapshot and effective-definition
  defects.
- Task/acceptance reviews must read V09: `producer_execution_id` currently means
  parser attribution, not proof of production.
- Primary review independently sampled the split registry maps, local catalogs,
  mode/history adapter, attachment handoff, executor error projection, artifact
  hydration, TaskRuntime acceptance boundary, and dormant public contracts. All
  eight findings and their priorities were accepted; see V30.
- This report becomes stale after changes to Subagent registration records,
  catalog synchronization, definition fields, context/prompt compilation,
  `AgentDispatchTool`, `SubagentOutcome`, artifact hydration, or EKO
  default/plugin Agent assembly.
- Dynamic regression cases remain implementation work by explicit review rule.
  No source fix or shared index update was made.
