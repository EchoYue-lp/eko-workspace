# Tools And App-Core Boundary Assessment

Review date: 2026-08-29

## Decision

No bulk move is required between `echo-agent` and `echo-agent-cli`.
`echo-agent` remains the reusable framework owner of generic Tool protocols,
built-ins, integrations, and domain tool packs. `echo-agent-cli` remains the
EKO owner of product policy, local persistence, workspace lifecycle, review
and worktree policy, and UI/TUI/CLI projections.

The rule for future changes is: introduce a framework primitive when its
semantics are product-neutral, independently testable, and complete without EKO
policy; current consumer count is not an admission gate. Keep EKO-specific
policy in app-core and expose it through a thin adapter.

## Current Ownership

| Source path | Current responsibility | Decision | Reason |
| --- | --- | --- | --- |
| `echo-agent/src/tools/builtin/` | Think, final answer, memory, human-loop, cell, and Subagent dispatch primitives | Keep in framework | These participate in the generic Agent execution contract and are not tied to EKO storage or UI. |
| `echo-agent/src/tools/lsp.rs` | LSP Tool protocol and registration | Keep in framework | LSP is a reusable integration boundary; process/profile policy is supplied by consumers. |
| `echo-agent/echo-tools/` and `echo-agent/src/tools/mod.rs` | Files, shell, Git, web, media, data, research, database, RAG, statistics, and public facade re-exports | Keep in framework | They are framework capability packs and public API surface, even when EKO selects only a subset. |
| `echo-agent-cli/echo-agent-app-core/src/tool_control.rs` | Runtime Tool enable/disable and generation control | Keep in app-core | EKO owns direct-user visibility policy and workspace-scoped publication. |
| `echo-agent-cli/echo-agent-app-core/src/tool_exposure.rs` | Product policy for direct-user versus automated invocation | Keep in app-core | This is an EKO interaction policy, not a reusable Tool protocol or permission implementation. |
| `echo-agent-cli/echo-agent-app-core/src/tool_execution.rs` | Product execution records and artifact retention | Keep in app-core | EKO chooses file layout, retention, detail cursors, and UI projections; typed `ToolResult.artifact` remains framework-owned. |
| `echo-agent-cli/echo-agent-app-core/src/tool_execution_projection.rs` | Lossless event-to-UI/detail projection | Keep in app-core | Projection is transport/application shaping and must not become a second execution authority. |
| `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/` | TaskRun file authority, review/worktree policy, continuation, and EKO lifecycle | Keep in app-core | The framework supplies generic Task DAG, status, timeout, and execution primitives; EKO owns product persistence and policy. |
| `echo-agent-cli/echo-agent-app-core/src/infra.rs`, `state.rs`, `workspace/` | Application composition, workspace identity, pool, config, and local lifecycle | Keep in app-core | These concepts depend on EKO's local product shape and do not hold for arbitrary framework consumers. |

## Guardrails

1. Do not remove framework public Tool APIs merely because EKO does not use
   every capability. The framework is an independent toolbox and keeps
   capability packs that are useful to future consumers.
2. Do not move EKO visibility, artifact, workspace, review, or TaskRuntime file
   policy into the framework under a generic name. That would couple the
   reusable crate to one desktop product.
3. A new app adapter may convert types, add EKO metadata, apply product policy,
   and publish UI events. It must not reimplement Tool execution, retry,
   cancellation, Task DAG traversal, or a second terminal/status authority.
4. When a genuinely product-neutral mechanism is identified, add a framework
   contract, framework test, example, and documentation; then switch one real
   EKO path to it before removing any displaced adapter logic. A second outside
   consumer is useful adoption evidence but is not required to begin.

## Evidence

The R1 closure ledger already audited all 151 app-core Rust files and recorded
the final dispositions for `tool_control`, `tool_execution`,
`tool_execution_projection`, `tool_exposure`, `infra`, `state`, and
`task_runtime`. This assessment makes that conclusion explicit for future
reorganization work; it does not create a second authority.

The framework capability placement correction and the first candidate-by-
candidate audit are recorded in
[`2026-08-30-framework-capability-placement-audit`](./2026-08-30-framework-capability-placement-audit.md)
and framework [ADR 0014](../echo-agent/docs/adr/0014-framework-capability-placement.md).
