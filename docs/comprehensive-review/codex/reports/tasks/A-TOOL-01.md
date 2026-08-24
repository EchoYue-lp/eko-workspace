# A-TOOL-01: Tool exposure, execution, sandbox, and terminal

> Status: complete
> Reviewer: Codex primary reviewer (delegated evidence independently sampled)
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: both source repositories clean; only Codex reports written

## Question

Does every EKO Agent role/mode expose an actually executable Tool surface with common bounded output/error/artifact behavior, while keeping interactive terminal use separate from Agent `run_code` policy?

## Scope

- EKO primary/pool/readonly/writer Agent construction, shared ToolManager and sandbox propagation.
- Chat/Task/Auto invocation visibility, disabled tools and `tool_search` reachability.
- Task plan role/allowlist validation through selected Subagent execution.
- Tauri interactive PTY versus Agent automation permission path.
- GUI/TUI/CLI/channel tool progress, failure, cancellation, large-output and artifact projection.
- Current tests and scoped implementation/history claims.

## Out Of Scope

- Framework Tool schema/registry/cancellation/artifact defects owned by F-EXT-01.
- File/Git/shell/run-code/remote-sandbox implementation defects owned by F-EXT-02 and security findings owned by F-SEC-01.
- Application project-root and process cleanup defects owned by A-BOOT-01.
- Browser/MCP/LSP product integration, frontend general architecture, source fixes and shared indexes.
- Cargo, rustc, tests, builds, dynamic fixtures and network operations.

## Inputs

- Root `AGENTS.md`; shared review README/REPORTING/TASKS; Codex README.
- Completed Codex dependencies [F-EXT-01](F-EXT-01.md), [F-EXT-02](F-EXT-02.md), and [A-BOOT-01](A-BOOT-01.md), used only for ownership/de-duplication.
- Current source, tests, MASTER-PLAN and scoped EKO implementation documents. No other reviewer directory was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Canonical Tool contract/registry/executor, schema enforcement, cancellation, sandbox primitives, output artifacts and typed failure remain framework-owned. |
| EKO product policy | Interaction-mode first-turn exposure, Subagent role capabilities, unattended/interactive policy, PTY presentation and surface rendering are EKO decisions. |
| Adapter boundary | EKO supplies invocation-visible/disabled sets and selected-role allowlists, passes sandbox/context/cancel identity, and projects canonical events without re-executing or reclassifying Tools. |
| Duplicate search | Searched Tool registration, names, `initial_visible_tools`, `disabled_tools`, `tool_search`, `allowed_tools`, Subagent role/readonly/plan mode, sandbox manager, `run_code`, terminal permission checks, Tool events/repositories/artifacts across both repositories. |
| Migration deletion | Keep one framework ToolManager/executor. Remove permanent writer plan-mode mutation and global-only task capability catalog after role-scoped runtime capabilities are authoritative; replace the channel wildcard drop with one common bounded event projection. |

No framework public API is classified dead because EKO does not use it.

## Current Path

```text
EKO bootstrap
  -> ReactAgentBuilder.enable_tools -> framework ToolManager
  -> local SandboxManager probe
     unavailable -> remove run_code from main + every writer/fork
  -> readonly Subagent: readonly registry
  -> writer Subagent: full registry + same sandbox, then set_plan_mode(true)

conversation/pool Agent
  -> shared drive_chat
  -> EKO Chat/Task/Auto visible + disabled sets
  -> framework snapshot + tool_search + ToolVisibilityStage
  -> canonical Tool execution/events

Task task_create/update
  -> TaskCapabilityCatalog(main Agent names + Subagent name metadata)
  -> selected readonly/writer Subagent
  -> allowed_tools converted to disabled set from child registry

rendering
  GUI -> durable ToolExecutionRepository + detail/artifact paging
  TUI -> bounded execution messages + artifact metadata/open action
  CLI -> ToolResult/typed ToolError + artifact path
  channel -> token/control/terminal events; ordinary Tool lifecycle discarded

interactive terminal
  user opens PTY -> first xterm input confirms session -> write/resize/close
  no Agent permission_mode lookup; separate from shell/run_code Tool path
```

Positive conclusions:

- Mode visibility is invocation-scoped over one live registry and execution-enforced; `tool_search` cannot activate disabled/ineligible tools.
- EKO statically closes its application-level bare `run_code` fallback for main, writer, fork and pool paths; readonly roles never register it.
- GUI has a durable UTF-8-safe paged tool-detail repository; TUI/CLI retain bounded output, typed error metadata and framework artifact references.
- Interactive terminal use is not gated by Agent automation permission mode and the first genuine frontend input establishes its session consent.

## Findings

### A-TOOL-01-P1-01: Writer Subagents are permanently left in read-only plan mode

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/infra.rs:875-964`; `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:2103-2227`; `echo-agent/src/agent/snapshot.rs:223-253`; `echo-agent/src/agent/react/run/pipeline.rs:986-1015`.
- Reachability: every Implementation/Debugging PlanTask dispatches the registered writer fork factory. Both initial and factory writer constructors end with `set_plan_mode(true)`; no TaskRuntime caller disables it.
- Expected invariant: a writer execution inside its assigned worktree/data workspace can use the allowed registered file-write and shell tools, while planning-only interactions remain read-only.
- Observed behavior: plan mode filters file-write tools and shell out of the model snapshot and blocks them in the execution pipeline. The writer has a full registry and isolation but cannot use its advertised normal mutation surface.
- Impact: implementation/debugging tasks cannot reliably edit files or run shell verification through their selected writer Subagent; they may stall, return prose, or fail despite successful plan validation and worktree setup.
- Root cause: interactive primary-Agent plan mode was reused as a permanent Subagent construction flag instead of invocation-scoped role/execution policy.
- Direction: construct writer Subagents with plan mode off for TaskRuntime execution, or pass an immutable invocation policy that permits only the selected tools inside observed isolation. Keep readonly roles physically readonly; delete unconditional writer `set_plan_mode(true)`.
- Regression validation: writer/fork Implementation and Debugging tasks with `write_file`, `edit_file`, `shell`, cancellation and missing sandbox; assert mutation only in assigned isolation and one typed terminal.
- Validation reports: [V03](../validations/A-TOOL-01/V03-01.md), [V09](../validations/A-TOOL-01/V09-01.md)

### A-TOOL-01-P1-02: Plan allowlists are validated against the main Agent instead of the selected Subagent

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/task_tools.rs:29-81`; `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/revisioned_adapter.rs:284-294`; `echo-agent-cli/echo-agent-app-core/src/subagent_loader.rs:154-240`; `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:2143-2227`; `echo-agent/src/agent/subagent/executor.rs:120-135,1744-1755`.
- Reachability: all framework task_create/update candidates pass the EKO policy, then real TaskRuntime dispatch supplies their persisted allowlist to the selected Subagent.
- Expected invariant: plan commit rejects names absent/disabled on that exact role and the runtime receives the same validated capability snapshot.
- Observed behavior: one catalog contains main-Agent names for every role; Subagent metadata contains no tool set. A readonly role can accept `shell`, then dispatch disables every tool it actually has because its registry lacks the sole allowed name. Writer-hidden tools also pass.
- Impact: valid-looking committed plans can produce tool-less or incapable Subagent runs and fail only after scheduling/claim/isolation work, making plan review materially misleading.
- Root cause: role discovery and Tool discovery are independent global snapshots instead of one source-scoped executable capability catalog.
- Direction: derive immutable capabilities from each registered Subagent factory/definition after sandbox and plan-mode policy, validate role + tool together, and carry the same snapshot/revision into dispatch. Delete global main-name validation once cut over.
- Regression validation: every builtin/custom readonly/writer role with allowed, absent, disabled, wildcard and empty/omitted lists; sandbox availability change and stale capability revision.
- Validation reports: [V04](../validations/A-TOOL-01/V04-01.md), [V09](../validations/A-TOOL-01/V09-01.md)

### A-TOOL-01-P1-03: Channel rendering silently discards ordinary Tool lifecycle and artifacts

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent-cli/src/cli/channels.rs:76-291,500-650,735-900`; `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:430-569`; `echo-agent-cli/echo-agent-app-core/src/surface_contract.rs:109-127`.
- Reachability: every channel message uses the shared `drive_chat`, forwards every Agent envelope through `ChannelChatSink`, then calls `aggregate_by_sentence` before the channel plugin sends output.
- Expected invariant: the channel adapter emits bounded progress/results, typed ToolError/recovery, cancellation and a usable complete-output artifact reference, equivalent in fact to GUI/TUI/CLI.
- Observed behavior: the renderer handles tokens and selected control/terminal events; ToolCall, ToolOutput/ToolStream, ToolResult and ToolError fall through `_ => {}`. It has no durable detail projection or artifact command. Tests encode only token flushing and Agent-level cancellation/error.
- Impact: channel users cannot inspect commands/actions, tool failures, large-output references or recovery while the Agent acts; a recovered ToolError disappears entirely, violating the declared full-Agent surface parity.
- Root cause: the shared event transport was adopted but its channel presentation remained a text-token aggregator.
- Direction: add one bounded channel Tool renderer over canonical events, including call identity, typed failure and artifact reference; do not create another executor/repository authority. Delete the wildcard drop for material events and update the surface contract from evidence, not aspiration.
- Regression validation: success/failure/retry/cancel/timeout, interleaved calls, Unicode large stdout/stderr and artifact continuation through each channel adapter.
- Validation reports: [V06](../validations/A-TOOL-01/V06-01.md), [V07](../validations/A-TOOL-01/V07-01.md), [V09](../validations/A-TOOL-01/V09-01.md)

## Validation Matrix

| ID | Claim or execution | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Commit/source-clean boundary | yes | passed | [report](../validations/A-TOOL-01/V00-01.md) |
| V01 | Mode registry, deferred schema and execution reachability | yes | passed | [report](../validations/A-TOOL-01/V01-01.md) |
| V02 | Sandbox availability and no-bare-fallback construction | yes | passed static | [report](../validations/A-TOOL-01/V02-01.md) |
| V03 | Writer Subagent executable tool surface | yes | failed | [report](../validations/A-TOOL-01/V03-01.md) |
| V04 | Plan role/allowlist compatibility | yes | failed | [report](../validations/A-TOOL-01/V04-01.md) |
| V05 | Interactive terminal versus automation permission | yes | passed | [report](../validations/A-TOOL-01/V05-01.md) |
| V06 | Cross-surface tool/output/error/artifact projection | yes | failed channel | [report](../validations/A-TOOL-01/V06-01.md) |
| V07 | Existing tests and historical claims | yes | passed inventory | [report](../validations/A-TOOL-01/V07-01.md) |
| V08 | Initial test inventory with shell globs | yes | inconclusive; not adopted | [report](../validations/A-TOOL-01/V08-01.md) |
| V09 | Dynamic sandbox/output/cancel/surface matrix | no per instruction | not run; future | [report](../validations/A-TOOL-01/V09-01.md) |
| V99 | Final report-integrity and source-boundary gate | yes | passed | [report](../validations/A-TOOL-01/V99-01.md) |
| V30 | Primary source sampling and acceptance | yes | passed | [report](../validations/A-TOOL-01/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `2026-07-16-run-code-sandbox`: no production bare `run_code` fallback | current at EKO construction boundary | [V02](../validations/A-TOOL-01/V02-01.md) |
| Schema-budget design: Chat/Task/Auto use deferred live registry plus `tool_search` | current | [V01](../validations/A-TOOL-01/V01-01.md) |
| Writer Subagent has full shell/file/Git write capability | regressed/overbroad | Registry exists, but permanent plan mode blocks normal write/shell execution; [V03](../validations/A-TOOL-01/V03-01.md). |
| MASTER-PLAN M10: channel consumes common Tool error/artifact facts | regressed | Material Tool events are discarded by the live renderer; [V06](../validations/A-TOOL-01/V06-01.md). |
| Surface contract: channel has tool stream/failure renderer and artifact reference | aspirational, not current | [V06](../validations/A-TOOL-01/V06-01.md) |
| Interactive terminal remains separate from Agent Tool permissions | current | [V05](../validations/A-TOOL-01/V05-01.md) |

## Coverage And Uncertainty

- No Cargo, rustc, tests, builds, sandbox probe, subprocess, UI/channel session, dynamic fixture or network call ran. V09 lists required regression scenarios.
- Static construction and call graph are conclusive for permanent plan mode, global capability validation and channel wildcard dropping. Exact model recovery behavior and rendered timing are unmeasured.
- F-EXT-01 remains canonical for framework schema/registry/cancel/artifact contracts; F-EXT-02 for file/Git/shell/sandbox implementation; A-BOOT-01 for workspace root/shutdown. No dependency finding was duplicated.
- Terminal input-preview redaction remains a security-owner concern; this report only verifies absence of automation permission gating.
- Changes to Agent/Subagent construction, `TaskCapabilityCatalog`, invocation policy, channel renderer or Tool event variants stale this report.

## Handoff

- First remove the writer plan-mode contradiction; role capability tests are otherwise testing a surface that remains execution-blocked.
- Build one role-scoped executable capability snapshot after registry, sandbox and invocation policy, then use it for plan validation and dispatch.
- Project canonical Tool events into bounded channel messages with typed failures and artifact references; preserve the shared executor and framework artifact authority.
- Preserve the verified local-product boundary: interactive terminal stays outside Agent automation permission mode, while `run_code` remains fail-closed on missing OS sandbox.
- Primary must independently reconstruct the three findings before changing `needs_evidence`; V09 is future validation, not a static-review blocker.
