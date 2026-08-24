# F-SUB-01: Subagent definitions, registry, and prompt context

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean

## Question

Are Subagent identity, catalog snapshot, role prompts, history inheritance,
tool/permission selection, and results coherent?

## Scope

Primary source paths inspected (deep read):

- `echo-agent/src/agent/subagent/types.rs` (full, 960 lines) — identity
  (`ExecutionMode`, `ObservedIsolation`, `SubagentStatus`), definition,
  result protocol (`SubagentOutcome`, `parse_subagent_outcome`,
  `render_result_contract`, `split_subagent_output`, artifact hydration).
- `echo-agent/src/agent/subagent/registry.rs` (full) — catalog snapshot,
  registration, factory instantiation.
- `echo-agent/src/agent/subagent/prompt.rs` (full) — `SubagentPromptCompiler`
  contract, `DefaultSubagentPromptCompiler`, `filter_history`,
  `with_compiled_task`.
- `echo-agent/src/agent/subagent/context.rs` (full) — `ContextInheritance`,
  `SubagentContext::from_parent`, `MemoryScope`/`OutputSchema`.
- `echo-agent/src/agent/subagent/context_builder.rs` (full) — `ContextBuilder`,
  `SubagentOutput`.
- `echo-agent/src/agent/subagent/builder.rs` (full) — `SubagentBuilder`.
- `echo-agent/src/agent/subagent/executor.rs:1-1889` — `DispatchRequest`,
  `dispatch`/`dispatch_sync`/`dispatch_fork`/`dispatch_teammate`/
  `dispatch_team`, `compile_invocation`, `execute_agent_streaming`,
  `merge_observed_evidence`, `subagent_status_from_error`.
- `echo-agent/src/tools/builtin/agent_dispatch.rs` (full) — `AgentDispatchTool`,
  `ParentContextFactory`, `SubagentCatalogEntry`, result serialization.
- `echo-agent/src/agent/react/capabilities.rs:285-460` — registration +
  dispatch-catalog maintenance.
- `echo-agent/src/agent/react/mod.rs:440-500, 2270-2630` — dispatch tool
  construction, programmatic delegation helpers, `build_parent_context_with`.
- `echo-agent-cli/echo-agent-app-core/src/infra.rs:580-1042` (EKO adapter),
  `subagent_prompt.rs:160-305` (EKO compiler), `subagent_loader.rs`
  (EKO definition), `plugin_components.rs:470-545`.
- `echo-agent/src/agent/subagent/isolated.rs`, `events.rs`, `usage.rs`,
  `hooks.rs` (small files, full or sampled).

## Out Of Scope

- Execution-mode lifecycles, team/manager internals, worktree/workspace
  factories, timeouts/cancellation ownership details → F-SUB-02
  (executor dispatch logic read only to trace identity/result/prompt flows).
- EKO catalog composition, loader precedence, pooled-agent refresh →
  A-SUB-01.
- Delegation depth/HITL permission gates → F-HITL-01, A-HITL-01.
- Frontend subagent projections → A-FE-02.

## Inputs

- Root `AGENTS.md` (Subagent-only terminology, UTF-8/panic safety, cleanup,
  layering gates), shared `README.md`, `REPORTING.md`, `TASKS.md`
  (F-SUB-01 card), `zcode-ds/README.md`, report templates.
- Dependency task reports read: zcode-ds `F-RCT-01` (complete — builder/
  registry/prompt assembly facts), `F-CORE-01` (complete — event envelope /
  identity construction used by `execute_agent_streaming`).
- Historical documents treated as hypotheses: `docs/MASTER-PLAN.md`
  subagent claims (classified in Historical Claim Status below).

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism | `SubagentDefinition`, `SubagentRegistry`, prompt-compiler trait, `SubagentContext`/`ContextInheritance`, result contract — correctly placed in the framework (`echo-agent`). |
| EKO product policy | Role prompt *content*, readonly/writer tool split, worktree/workspace policy, per-task `allowed_tools` from TaskRuntime — application-owned; EKO keeps its own loader definition (`subagent_loader.rs`) that actually drives agent construction. |
| Adapter boundary | `EkoSubagentPromptCompiler` (`subagent_prompt.rs`) is the thin adapter exercising the framework `compile_system`/`compile_invocation` contracts at registration/dispatch; `ParentContextFactory` is the framework-side context snapshot adapter. No repository movement recommended. |
| Duplicate search | Terms: `worker`, `SubAgentMap`, `SubagentRegistry`, `AgentDispatchTool`, `SubagentDefinition`, `tool_filter`, `allowed_tools`, `inherit_tools`, `ContextBuilder`, `SubagentContext`, `MemoryScope`, `OutputSchema`, `compile_system`, `compile_invocation`, `render_result_contract`, `parse_subagent_outcome`, `merge_observed_evidence`, `sub-agent`, `SubAgent`. Results: single registry authority; `SubAgentMap` is a `pub(crate)` alias consumed only by the registry; `ContextBuilder` is a dead parallel context-construction API (P2-03); `worker` zero matches (V01). |
| Migration deletion | If P2-01/P2-03 directions are taken, delete: the 7 dead definition fields (and their builder setters) or wire them; delete `ContextBuilder`/`SubagentOutput`/`OutputSchema`/`MemoryScope`; delete `isolated.rs`; remove the `SubAgent` casing strings. |

## Current Path

Identity: subagent identity = registry key (`SubagentDefinition.name`) +
per-dispatch `execution_id` from `ExternalRunContext` (`agent_tool-{uuid}`,
`agent_dispatch.rs:151`; TaskRuntime `{task_id}:{attempt}` in EKO). Events
carry `parent`/`agent`/`execution_id`/`run_id`; envelope identity via
`EventIdentity::from_invocation` (`executor.rs:1168`).

Definition→registration→dispatch (verified): `SubagentBuilder` /
`SubagentDefinition::new` → `SubagentRegistry` (agents + definitions +
factories maps) via `register*` → agent-level registration
(`capabilities.rs:305-356, 360-401`) also updates the model-facing dispatch
catalog (`update_dispatch_catalog`, sorted, deduped) → `agent_tool`
`parameters()` projects catalog as `agent_name` enum
(`agent_dispatch.rs:388-416`) → `execute` builds `DispatchRequest`
(mode forcing for isolation roles, `:244-252`) → `SubagentExecutor::dispatch`
(`executor.rs:407`) routes by resolved `ExecutionMode` → `dispatch_sync`/
`dispatch_fork`/`dispatch_teammate`/`dispatch_team` compile the invocation
via the injected `SubagentPromptCompiler` (`compile_invocation`,
`:1117-1145`) and run the isolated agent; result flows back as
`SubagentResult.outcome` (contract JSON) → tool returns
`serialize_parent_result(outcome)` (`agent_dispatch.rs:336-343`) and events
`DispatchCompleted/Failed/Cancelled` carry the same `SubagentOutcome`
(`events.rs:64-113`). Registry is the execution authority; the catalog is a
schema projection reconciled via `sync_subagent_dispatch_catalog`
(`capabilities.rs:434-438`).

## Findings

### F-SUB-01-P1-01: Per-role tool/permission selection declared on `SubagentDefinition` is never enforced — the LLM-facing `agent_tool` cannot restrict a subagent's tools

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/subagent/types.rs:144` (`tool_filter`,
  doc "Restrict available tools by name"), setter `builder.rs:126`; zero
  production readers in either repo (V01). The only live tool-restriction
  mechanism is `AgentInvocationContext.disabled_tools` fed by
  `invocation_disabled_tools` (`executor.rs:120-135`), which reads only
  `SubagentContext.allowed_tools` — populated only by the programmatic
  delegation helpers (`react/mod.rs:2370-2377, 2517-2518`) and consumed
  only in `dispatch_fork` (`executor.rs:1672-1676, 1744-1745`). The
  LLM-facing `agent_tool` has no tool parameter
  (`agent_dispatch.rs:224-234, 286-299`) and its `ParentContextFactory`
  never sets `allowed_tools` (`context.rs:229-281`, hardcoded `None` at
  `:277`). Sync/Teammate/Team invocations carry no `disabled_tools` at all
  (`executor.rs:1495-1499, 908-912`).
- Reachability: definition `tool_filter` → registered (`capabilities.rs:
  305-325`) → dispatch (`executor.rs:444-452`) → **no consumer**; LLM
  dispatch via `agent_tool` → Fork parent context without allowlist → full
  tool surface; TaskRuntime per-task allowlist
  (`echo-agent-cli/.../tasks/task_runtime/executor.rs:2833, 2941-2955`) is
  the only production path that restricts tools.
- Expected invariant: a definition-declared tool restriction bounds the
  subagent's model-facing and executable tool surface (MASTER-PLAN:54
  "Subagent 使用独立上下文、独立工具/权限配置").
- Observed behavior: `tool_filter` is inert; per-role restriction exists
  only as EKO's build-time readonly/writer split (`infra.rs:639-677,
  906-991`) and per-task `allowed_tools` on the programmatic path.
- Impact: framework consumers configuring `.tools([...])` on
  `SubagentBuilder` get an unrestricted subagent (false scope guarantee,
  same defect class as F-RCT-01-P2-01); an LLM-dispatched restricted role
  can call every parent tool; the framework's documented tool/permission
  selection contract is unimplemented on its primary dispatch surface.
- Root cause: tool selection was implemented as invocation-time
  `disabled_tools` wired only into the programmatic delegation path and
  only for Fork; the declarative definition field was never connected to a
  consumer.
- Direction: consume `tool_filter` at dispatch — map it into
  `AgentInvocationContext.disabled_tools` in `dispatch_sync`/
  `dispatch_fork`/`dispatch_teammate` (single authority at the invocation),
  or delete the field and re-document; keep per-task `allowed_tools`
  precedence over the definition filter. Add a test that a definition with
  `tool_filter` hides out-of-list tools from the model and blocks their
  execution.
- Regression validation: `SubagentExecutor` unit test with a mock agent:
  register definition with `tool_filter(vec!["read_file"])`, assert
  `tools_for_llm`-equivalent invocation surface contains only allowlisted
  tools and calling an out-of-list tool is rejected; keep V04 tests green.
- Validation reports: [V01](../validations/F-SUB-01/V01-01.md),
  [V02](../validations/F-SUB-01/V02-01.md), [V03](../validations/F-SUB-01/V03-01.md)

### F-SUB-01-P2-01: Seven `SubagentDefinition` configuration fields are never read in production — the framework definition is not the authority for role prompt, model, or runtime limits; EKO keeps a parallel loader definition that actually drives construction

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: dead fields `model` (`types.rs:140`), `system_prompt` (`:141`),
  `max_iterations` (`:146`), `token_limit` (`:148`), `inherit_memory`
  (`:153`), `can_delegate` (`:157`), `lightweight` (`:164`, no setter
  anywhere — doc promises an "infrastructure-sharing" execution mode that
  does not exist). Zero production readers (V01). The executor consumes
  only `name/description/execution_mode/inherit_history/timeout_secs/
  isolate_worktree/isolate_workspace/team/is_background` (V02). EKO sets
  these dead fields via the builder (`infra.rs:718-729`:
  `.model()`, `.max_iterations()`, `.can_delegate()`) while the real
  config comes from its own loader definition at build
  (`infra.rs:620-638, 639-677`; `plugin_components.rs:481-510`).
- Reachability: any `SubagentBuilder::new(...).model(...)/system_prompt(...)
  /tools(...)/max_iterations(...)/token_limit(...)/inherit_memory()/
  can_delegate()` (public API, `builder.rs:113-152`) — field written at
  registration, never read at dispatch; `lightweight` not even writable.
- Expected invariant: the declarative `SubagentDefinition` is the
  dispatch-time configuration authority for the subagent it describes.
- Observed behavior: dispatch-time role prompt, model, iteration/token
  limits, memory inheritance, and delegation capability are determined
  elsewhere (EKO build-time wiring, `NestedDelegationPolicy` at
  `agent_dispatch.rs:121-135, 286-299`); the definition carries an inert
  copy.
- Impact: misleading public framework API — independent consumers cannot
  configure role prompt/model/limits through the documented surface;
  two configuration authorities (framework definition vs EKO loader) can
  silently diverge; `lightweight` documents a nonexistent mode.
- Root cause: the definition API predates the executor; execution wiring
  was built on a subset of fields; EKO duplicated configuration in its
  loader instead of extending the framework contract.
- Direction: either (a) wire the fields at dispatch (model/system_prompt/
  max_iterations/token_limit → per-dispatch construction; inherit_memory →
  parent-context inheritance; can_delegate → dispatch-tool registration
  gate; lightweight → real shared-subsystem mode), or (b) delete the dead
  fields and setters and re-document the definition as registration
  metadata only (EKO loader becomes the single construction authority).
  Prefer (b) as the minimal correct fix unless a consumer needs (a).
- Regression validation: `cargo check -p echo_agent --features subagent`
  plus the V04 subagent suite; a compile-fail or grep test asserting no
  remaining reader-less fields; EKO build smoke test after field removal.
- Validation reports: [V01](../validations/F-SUB-01/V01-01.md),
  [V02](../validations/F-SUB-01/V02-01.md)

### F-SUB-01-P2-02: `SubagentDefinition.inherit_history` semantics are violated on the LLM fork path — `None` (doc: "don't inherit") inherits up to 2 messages and `Some(n>2)` is silently capped at 2

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: field doc "None = don't inherit, Some(0) = inherit all,
  Some(n) = last n messages" (`types.rs:149-151`); `ParentContextFactory`
  builds the fork context with `ContextInheritance::for_mode(Fork)` =
  `fork_default` (`agent_dispatch.rs:268-274`, `context.rs:74-81`), which
  pre-slices parent messages to the last 2 raw messages
  (`context.rs:254-260`) **before** the compiler; the executor then passes
  the definition's `inherit_history` only as the *limit* to
  `filter_history` (`executor.rs:1117-1145`, `prompt.rs:123-130, 182-186`),
  where `None` means no limit (`prompt.rs:182`). Result:
  `None` → inherits up to 2 messages; `Some(10)` → still 2; `Some(1)`/`Some(0)`
  behave as documented. Additionally, for Sync/Teammate/Team the transfer
  policy is always `Fresh` (`executor.rs:1123-1132`), so a definition
  `inherit_history` never applies there.
- Reachability: `agent_tool` with `mode=fork` on any agent with
  `register_agent_dispatch_tool` (EKO delegates), definition
  `inherit_history=None` (framework default) or `Some(n>2)`; EKO itself
  always sets `Some(2)` (`infra.rs:689` via `fork_mode()`;
  `plugin_components.rs:498`), so the app is unaffected.
- Expected invariant: the field doc contract holds on every dispatch path
  that can inherit history.
- Observed behavior: `None` silently inherits conversation the caller
  explicitly declined; larger explicit values are truncated; the `agent_tool`
  `mode=fork` description "inherit parent system prompt + recent messages"
  (`agent_dispatch.rs:429`) is additionally inaccurate — parent system
  prompts are never transferred (`context.rs:26-27`).
- Impact: framework consumers get unexpected conversation scope leakage
  (privacy/context pollution) or a silent cap; doc/behavior mismatch.
- Root cause: two independent truncation points — the parent-context factory
  default slice and the compiler's limit — with the definition value applied
  only at the second, and `None` overloaded as "no limit" at the second.
- Direction: make the definition the single authority: either (a) build the
  parent context with the definition's `inherit_history` (factory reads the
  definition), or (b) treat `None` as "don't inherit" in `filter_history`
  (distinguish "no limit" from "no inheritance"); fix the `mode=fork`
  description. Add a unit test asserting the three documented values at the
  executor level.
- Regression validation: executor test with `inherit_history` `None`/
  `Some(0)`/`Some(1)`/`Some(10)` asserting the compiled history length on
  the fork path; keep `filter_history` unit tests green (V04).
- Validation reports: [V03](../validations/F-SUB-01/V03-01.md),
  [V04](../validations/F-SUB-01/V04-01.md)

### F-SUB-01-P2-03: `ContextBuilder`/`SubagentOutput`/`OutputSchema`/`MemoryScope` and the `SubagentContext` scoped fields are dead in production — two parallel context-construction authorities with divergent semantics

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `context_builder.rs` (whole file; `ContextBuilder::
  build_scoped_context` `:145-180`, `SubagentOutput::to_json` `:261-339`)
  has zero production callers (V01); re-exported at `subagent/mod.rs:26`.
  The live construction path `SubagentContext::from_parent`
  (`context.rs:229-281`) never populates `assigned_task`/`relevant_files`/
  `relevant_artifacts`/`constraints`/`memory_scope`/`output_schema`
  (hardcoded defaults `:272-279`); production setters exist only for
  `allowed_tools` (`react/mod.rs:2376, 2518`) and `parent_goal`
  (`context.rs:235`). `MemoryScope::Relevant` is a stub "same as None"
  (`context.rs:157-163`); `OutputSchema` is consumed only by the dead
  `SubagentOutput::to_json`.
- Reachability: none in production; only unit tests construct
  `ContextBuilder`/`SubagentOutput`/`OutputSchema`/`MemoryScope` values.
- Expected invariant: one context-construction authority per concept; no
  dead public API advertising parallel semantics (AGENTS.md cleanup +
  "动手前先查/能复用就不新建").
- Observed behavior: two builder APIs for the same `SubagentContext` type —
  `ContextBuilder` (copies scoped fields) vs `from_parent` (defaults them) —
  with only one reachable; the "Step 6 scoped context" feature is
  unimplemented in production (EKO's planned-task context is rendered by
  its compiler payload instead, `subagent_prompt.rs:247-262` — a separate,
  coherent mechanism).
- Impact: misleading public API; dead code per cleanup rules; risk that a
  consumer wires the unused builder and gets silent no-ops for the scoped
  fields.
- Root cause: a planned scoped-context feature was scaffolded, then the
  dispatch path was implemented on `from_parent` + compiler payloads
  without deleting the scaffold.
- Direction: delete `ContextBuilder`, `SubagentOutput`, `OutputSchema`,
  `MemoryScope` (and the never-populated `SubagentContext` fields
  `assigned_task`/`relevant_files`/`relevant_artifacts`/`constraints`/
  `memory_scope`/`output_schema`) and their tests; keep `parent_goal`/
  `allowed_tools`/`messages`/`tool_definitions`/`store`. Re-export removal
  in `subagent/mod.rs:26`.
- Regression validation: `cargo check -p echo_agent --features subagent`
  after removal; grep for the removed names returns nothing; V04 subagent
  suite green (tests referencing these types are deleted with them).
- Validation reports: [V01](../validations/F-SUB-01/V01-01.md)

### F-SUB-01-P3-01: `isolated.rs` (`run_isolated`/`IsolatedSubAgentConfig`) is a dead legacy subagent execution path using outdated "sub-agent" naming

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/subagent/isolated.rs` (whole file; builds
  its own subagent via `ReactAgentBuilder` and `sub.execute(task)`, bypassing
  the registry/executor); zero production callers in either repo (V01);
  `subagent/mod.rs:11` declares the module but does not re-export its types.
- Reachability: none.
- Expected invariant: no dead execution path; single subagent execution
  authority (the executor).
- Observed behavior: a compiled-in legacy path exists that duplicates a
  subset of dispatch (timeout + fresh agent) without events, hooks, or
  result contract.
- Impact: dead code and a second, inferior execution story for readers;
  legacy hyphenated "sub-agent" wording conflicts with the Subagent-only
  terminology rule.
- Root cause: pre-executor implementation retained during the unified
  dispatch migration.
- Direction: delete `isolated.rs` and its module declaration; grep
  `sub-agent` in `usage.rs:1` docs and normalize.
- Regression validation: `cargo check -p echo_agent --features subagent`
  after removal; `cargo test` green.
- Validation reports: [V01](../validations/F-SUB-01/V01-01.md)

### F-SUB-01-P3-02: A dispatch with an already-cancelled token returns a cancelled result without emitting any `SubagentEvent` or running hooks

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `executor.rs:407-423` — `if parent_cancel.is_cancelled() {
  return Ok(SubagentResult::cancelled(...)) }` before the loop that emits
  `DispatchStarted` and terminal events; the event contract elsewhere is
  "every Start has exactly one Stop" (`executor.rs:614-616`).
- Reachability: `DispatchRequest` with a pre-cancelled token — e.g. parent
  cancelled between `child_cancel_token` derivation
  (`agent_dispatch.rs:164-178, 283-284`) and `dispatch`.
- Expected invariant: every dispatch attempt produces exactly one
  `DispatchStarted` and one terminal event (`DispatchCompleted`/
  `DispatchFailed`/`DispatchCancelled`).
- Observed behavior: zero events for this path; the UI sees no card, hooks
  do not run; the caller still gets the structured `Cancelled` outcome.
- Impact: missing lifecycle observability in a rare edge; no data loss.
- Root cause: the early-return shortcut predates the event contract.
- Direction: emit `DispatchStarted` + `DispatchCancelled` (or skip dispatch
  at the caller and surface the cancelled result as a tool error) before
  returning.
- Regression validation: executor unit test dispatching with a pre-cancelled
  token asserting exactly one terminal event.
- Validation reports: [V03](../validations/F-SUB-01/V03-01.md)

### F-SUB-01-P3-03: `register_sync` can register an agent without its definition under lock contention, making `contains()`/`get()` disagree with `get_agent()`

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `registry.rs:161-187` — `agents.try_write()` success + `defs
  .try_write()` failure leaves the agent in `agents` without a definition
  (and no event); `contains()` checks only `definitions` (`:422-425`);
  `get_agent()` checks only `agents` (`:318-327`).
- Reachability: only under `try_write` contention from synchronous
  registration contexts (builder/main); EKO registers from async contexts
  (`register_sync` used at `capabilities.rs:320`, `register_definition_sync`
  at `:347`).
- Expected invariant: registration is atomic across the agents/definitions/
  factories maps; `contains`/`get`/`get_agent` agree.
- Observed behavior: transient divergence (definition missing → `contains`
  false, `get_agent` Some; catalog entry would still be added by the caller
  since `register_sync` returned `true`).
- Impact: rare inconsistent registration state and catalog entry without a
  registry definition; benign because the executor's `registry.get` would
  fail the dispatch ("not found") — masking a registered instance.
- Root cause: independent non-atomic `try_write` calls in a sync-compat
  path.
- Direction: on definitions-map failure, roll back the agents-map insert (or
  return `false` before inserting) and emit the event only when both maps
  are consistent.
- Regression validation: unit test forcing `defs.try_write` failure (or
  pre-locking the definitions lock) and asserting rollback.
- Validation reports: [V01](../validations/F-SUB-01/V01-01.md)

### F-SUB-01-P3-04: "SubAgent" (capital A) naming in `agent_dispatch.rs` user-facing strings deviates from the project's `Subagent` terminology

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `agent_dispatch.rs:321, 340, 348, 364-376, 396-402, 424-438`
  (error strings and tool description use "SubAgent"/"SubAgents").
- Reachability: every LLM-visible `agent_tool` description and every error
  path surfaces these strings.
- Expected invariant: uniform `Subagent` terminology (AGENTS.md).
- Observed behavior: mixed casing in the most user-facing framework text.
- Impact: cosmetic; inconsistent terminology in prompts and errors.
- Root cause: older strings never normalized.
- Direction: normalize to `Subagent` in one cleanup; no behavior change.
- Regression validation: grep for `SubAgent` returns nothing; V04
  `agent_dispatch` tests green.
- Validation reports: [V01](../validations/F-SUB-01/V01-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Cross-repo definition/duplicate search (worker terms, parallel implementations, field reader audit) | yes | passed | [V01-01](../validations/F-SUB-01/V01-01.md) |
| V02 | Definition-to-registration trace; catalog route validation; runtime reachability | yes | passed | [V02-01](../validations/F-SUB-01/V02-01.md) |
| V03 | Prompt envelope/cardinality; history-inheritance and tool/permission invariants; UTF-8/terminal-ownership checks | yes | passed | [V03-01](../validations/F-SUB-01/V03-01.md) |
| V04 | `cargo test -p echo_agent --lib --features "subagent,tasks" --locked agent::subagent` and `... agent_dispatch` | yes | passed (exit 0; 127 + 11 passed) | [V04-01](../validations/F-SUB-01/V04-01.md) |
| V05 | MASTER-PLAN subagent claims vs current code | conditional | passed | [V05-01](../validations/F-SUB-01/V05-01.md) |

All required validations executed with known exit codes; no validation is
pending. Every reported command has a known exit code.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN:54 "Subagent 使用独立上下文、独立工具/权限配置,主会话只接收汇总" | current (independent context + structured summary) / regressed in part (definition-level tool/permission configuration) | `executor.rs:1148-1453`; `agent_dispatch.rs:336-343`; dead `tool_filter` `types.rs:144`; [V05-01](../validations/F-SUB-01/V05-01.md) |
| MASTER-PLAN:100 "Sync/Fork/Teammate/Team、独立上下文、timeout、checkpoint、worktree/tmpdir 隔离和 writer 文件锁基础" | current (team-checkpoint and writer-locks are EKO-scoped) | `executor.rs:552-564, 1603-1735`; `types.rs:166-188`; [V05-01](../validations/F-SUB-01/V05-01.md) |
| MASTER-PLAN:101 "agent_tool 子取消令牌以 invocation ToolContext.cancel 为权威" | current | `agent_dispatch.rs:164-178, 283-284`; test at `:559-580`; [V05-01](../validations/F-SUB-01/V05-01.md) |
| MASTER-PLAN:212-233 "结构化结果合同; 失败不被汇总文本掩盖; 状态以结构化结果为准" | current | `types.rs:386-532`; `executor.rs:229-254`; [V05-01](../validations/F-SUB-01/V05-01.md) |
| MASTER-PLAN:245 "主 Agent 汇总使用结构化 result" | current | `agent_dispatch.rs:13-17, 336`; [V05-01](../validations/F-SUB-01/V05-01.md) |
| MASTER-PLAN:482 "Subagent result 显式携带 cancelled" | current | `types.rs:757-779`; `executor.rs:1410-1412, 1443-1445`; [V05-01](../validations/F-SUB-01/V05-01.md) |

## Coverage And Uncertainty

- All conclusions are static except the V04 test runs; no live LLM end-to-end
  dispatch was executed (read-only review).
- The executor was read to the dispatch/result surfaces (lines 1-1889 plus
  the events/hooks/usage files); team internals (`team/`), worktree/
  workspace factory implementations, and background scheduling internals are
  F-SUB-02 scope.
- `inherit_history` cap behavior (P2-02) was derived statically from the
  factory default + compiler limit chain; the 2-message ceiling is
  guaranteed by `fork_default`, not executed dynamically.
- Whether the dead definition fields should be wired vs deleted is a product
  decision; the finding documents the divergence, not the choice.
- `agent_pool`/GUI/TUI projections of subagent events were not inspected
  (A-SUB-01 / A-FE-02 / X-EVT-01).

## Handoff

- Conclusions downstream tasks may rely on: single registry/catalog
  authority with registry-authoritative route validation (V02); coherent
  result contract with runtime-owned terminal status, observed/reported
  evidence separation, and UTF-8-safe bounding (V03/V04); definition-level
  tool/permission and most configuration fields are inert (P1-01, P2-01);
  `inherit_history` cap/leak on the fork path (P2-02); dead scoped-context
  machinery (P2-03); green subagent test state under `subagent`+`tasks`
  features (V04).
- `F-SUB-02`: execution modes consume `execution_mode`/isolation/team/
  timeout fields — the P1-01 fix (invocation `disabled_tools` in all modes)
  must not conflict with teammate/team tool wiring; hooks retry/delegate
  rebuild `parent_context: None` (attempts after retry are fresh-context).
- `A-SUB-01`: EKO keeps its own loader definition; if P2-01 direction (b)
  is taken, EKO's loader becomes the single construction authority and the
  framework definition shrinks to registration metadata — A-SUB-01 should
  validate loader precedence unaffected.
- `A-TSK-06`: parent summary consumes the structured `SubagentOutcome`
  from DispatchCompleted/DispatchFailed events — unchanged by these
  findings.
- `X-BND-01`: record the definition/loader duplicate-configuration
  authority decision (P2-01) and the `ContextBuilder` deletion or wiring
  decision (P2-03).
- Reports to read: this report + V01-01..V05-01; F-RCT-01 (P2-01 allowed_tools
  cross-reference), F-CORE-01 (envelope identity used by
  `execute_agent_streaming`).
- Stale triggers: changes to `subagent/types.rs`, `registry.rs`,
  `prompt.rs`, `context.rs`, `context_builder.rs`, `executor.rs`
  (dispatch/compile/result paths), `tools/builtin/agent_dispatch.rs`,
  `react/capabilities.rs` registration paths, or EKO `infra.rs` subagent
  construction invalidate the corresponding claims.
- Follow-up task IDs (fixes not implemented in this review): F-SUB-02,
  A-SUB-01, X-BND-01, X-EVT-01, Q-FLT-02 (dispatch invariants under cancel/
  timeout), Q-STA-01 (dead-field inventory), S-RDM-01 (P1-01/P2-01/P2-02/
  P2-03 deletion targets).
