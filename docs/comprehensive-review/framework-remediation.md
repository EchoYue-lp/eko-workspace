# Framework Review Remediation

> Status: complete; all applicable executable gates and static audits passed
> Remediation date: 2026-08-14
> Scope: `echo-agent` plus the required `echo-agent-cli` adapter changes

## Inputs And Recheck

This remediation reconciles the three independent framework syntheses:

- [Codex framework review](codex/reports/synthesis/framework-review.md)
- [ZCode-ds framework review](zcode-ds/reports/synthesis/framework-review.md)
- [ZCode-glm framework review](zcode-glm/synthesis/framework-review.md)

The reports use different finding counts and severities, and some atomic
findings overlap or describe an old revision. Each proposed change was therefore
rechecked against the current definition, registration, production call path,
and tests before editing. Findings were implemented by root cause instead of
adding one local workaround per report ID.

The external design constraints came from the three B-REF-01 investigations of
Claude Code, Codex, Cursor, Devin, and Temporal. The convergent choices used here
are: Plan remains a versioned artifact; events have stable typed identities and
terminal facts; Subagents have isolated, bounded, cancellable attempts; policy
is separate from sandbox enforcement; recovery uses durable trajectory/history
and attempt-scoped idempotence rather than deterministic replay of an LLM loop.

## Ownership Decision

| Boundary | Authority after remediation |
|---|---|
| Generic framework mechanism | Typed events and outcomes, cancellation/deadlines, Task DAG and claims, Subagent lifecycle, provider-neutral streaming, Tool execution, atomic/corruption-safe stores, bounded I/O, retry and telemetry primitives |
| EKO product policy | Domain profiles, reviewer and worktree policy, UI projections, local retention values, workspace routing, and interactive surface composition |
| Adapter boundary | Lossless identity/result/event conversion plus EKO metadata and policy injection; no second DAG, retry loop, terminal inference, or recovery authority |

EKO remains file-backed and does not enable SQLite. The framework's optional
SQLite Store implementations remain supported public choices for other users.
Interactive terminal and MCP capabilities were not placed behind automated
agent permission modes. Product terminology is uniformly Subagent.

## Implemented Remediation

### Lifecycle, Events, And Providers

- Unified streaming and non-streaming ReAct execution around typed terminal
  outcomes; error, cancellation, timeout, disconnect, and truncation no longer
  become empty success.
- Made terminal delivery lossless under backpressure and preserved model tool
  call order across concurrent execution.
- Propagated cancellation and deadlines through providers, Tool batches,
  Subagent Fork/Team execution, workflows, MCP/LSP/A2A, channels, scheduler,
  headless services, and owned background handles.
- Preserved reasoning blocks, fragmented tool calls, finish reasons, usage and
  cache accounting in OpenAI/Anthropic adapters; removed the disconnected
  provider adapter authority.
- Added stable event stream identity and monotonic sequence validation through
  framework envelopes and EKO GUI/TUI/channel projections.

### Durable State And Recovery

- Added atomic file replacement, unique temporary paths, parent sync, bounded
  retention, path confinement, and fail-closed corrupt-data handling across
  memory, checkpoints, audit, workflow, plugin, scheduler, trace, and task
  stores.
- Preserved corrupt originals instead of treating them as empty state; repaired
  torn JSONL tails without accepting corruption in the committed prefix.
- Versioned and validated checkpoints, preserved paired tool-call context, and
  restored trajectory without wiping valid conversation history.
- Made Task claims ABA-safe with physical claim UUIDs while EKO restart recovery
  uses the stable `(run, task, revision, attempt)` idempotency key. Completed
  Subagent output is reviewed without redispatch; edited revisions and explicit
  retries cannot reuse stale output.
- Added stale-write and external-change protection for files, worktrees, task
  revisions, plugin lifecycle, and conversation state.

### Task, Subagent, Workflow, And HITL

- Established the revisioned Task graph as the single readiness, dependency,
  retry, cancel, settlement, and stall-detection authority. Skip and failure
  propagation now settle dependent tasks correctly.
- Folded Team and handoff execution into the Subagent lifecycle, removed the
  parallel handoff/runner authorities, and enforced bounded concurrency,
  recursion, isolation, timeout, cancellation, and result/artifact contracts.
- Corrected workflow parallel checkpoint/resume so pending fan-out branches and
  interrupt boundaries survive restart.
- Connected the live human-input path, effective edited arguments, timeout
  propagation, and audit value. Removed wildcard session approval behavior while
  keeping user-interactive local tools independent of agent automation mode.

### Tools, Integrations, And Safety

- Hardened file/Git/worktree/database/RAG/research/media tools for atomicity,
  UTF-8 safety, pagination, bounded bodies, cancellation, partial side effects,
  SQL statement validation, and structured failures.
- Fixed MCP HTTP asynchronous response handling, LSP timeout cleanup, A2A
  cancellation terminality, channel shutdown loops, scheduler occurrence
  calculation, and secret-free bounded run records.
- Centralized redaction before logs, traces, audit, webhooks, and persistence;
  corrected ContentGuard redaction behavior and sensitive URL/config handling.
- Removed fabricated research memory success, obsolete enhanced fetch paths,
  dead loop detector/approval/execution/handoff authorities, no-op capability
  surfaces, and stale examples/docs.

### Budgets, Compression, Extensions, And Tests

- Made token arithmetic checked/saturating, provider context windows explicit,
  category budgets borrow unused capacity, and protected/tool/output reservations
  part of the effective input calculation.
- Enforced token bounds and message invariants through sliding, summary, hybrid,
  and adaptive compression; system summaries no longer accumulate indefinitely
  or split tool-call/result pairs.
- Made Skill/Plugin registration deterministic and source-aware, cycle-safe,
  unloadable, and collision-explicit; corrected intent activation thresholds and
  plugin lifecycle rollback.
- Corrected procedural macro facade paths and diagnostics and added downstream
  facade compilation coverage.
- Replaced permissive mock defaults with fail-closed scripted behavior and added
  multi-chunk, usage, mid-stream error/cancel, rich Tool result, Unicode,
  corruption, race, restart, and lifecycle regression fixtures.

## Validation

### 2026-08-14 Re-audit Corrections

The claimed closure was rechecked against production call paths rather than
accepted from the ledger. Two entries were falsely closed and were reopened:

- `F-EXT-02-P1-03`: Git and worktree Tools still used synchronous child
  processes inside async execution. They now share a bounded async process
  runner with timeout, cancellation, process-group termination, prompt-free Git
  environment, concurrent pipe draining, and capped retained output.
- `F-EXT-02-P1-05`: `GitBranchTool` still declared read-only permissions while
  exposing create/delete/checkout mutations. Its contract now declares
  read/write/execute permissions and dangerous risk; user-invoked terminal and
  MCP features remain independent of this agent automation policy.

The same re-audit also found that a provider could emit valid partial content
and then fail before a terminal finish reason, while `run_think` discarded the
buffer. The buffered content is now emitted before the typed failure terminal,
with framework, app-core, and frontend regressions proving it remains visible.

These changes stay in the generic framework because cancellable bounded child
process execution and typed stream failure are reusable mechanisms. EKO
workspace transaction policy and surface rendering remain in the application.

All applicable repository gates completed successfully after the changes:

- `echo-agent`: format check; both all-feature clippy gates including forbidden
  panic APIs; workspace all-target/all-feature tests; no-default library check;
  all-feature doctests; standalone `sqlite`, `subagent`, `human-loop`, `mcp`,
  `lsp`, `a2a`, `git`, `database`, `rag`, `chart`, `web`, and `media` checks.
- `echo-agent-cli`: format check; both all-feature clippy gates; full workspace
  tests; app-core no-default check; GUI binary check; GUI feature tests.
- Frontend: the initial recorded gate passed 101 Vitest tests together with
  Prettier, TypeScript compilation, and the Vite production build; the final
  rerun below passed the expanded 105-test suite.
- Static boundary checks: no internal Worker terminology, no CLI SQLite
  dependency, no absolute/worktree Cargo paths, and clean `git diff --check`.

The CLI workspace reported 659 app-core unit tests passed with two documented
live-provider tests ignored, five app-core integration tests passed, 90 CLI
library tests passed, and nine binary tests passed. The GUI-only matrix passed
51 tests. Frontend Vitest reported 105 tests passed. Build caches were cleaned when disk availability crossed the
repository's pressure threshold; source and durable project data were not
removed.

The atomic reconciliation accounts for 295 raw Codex entries: 294 unique
canonical IDs plus one explicit deduplication backlink. The per-ID ledger is
[framework-finding-ledger.md](framework-finding-ledger.md); report-level DS/GLM
aliases and rejected over-scoped proposals are recorded in
[framework-finding-closure.md](framework-finding-closure.md).
