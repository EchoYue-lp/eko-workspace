# Framework Finding Closure Ledger

> Status: complete
> Recheck date: 2026-08-14
> Scope: all three framework reviews, current `echo-agent`, and required `echo-agent-cli` adapters

## Counting And Method

The three reviews are independent assessments of different baselines and use
different severity thresholds. Their headline totals must not be added:

- Codex: 295 raw entries: 294 unique canonical atomic findings
  (`P0=13`, `P1=180`, `P2=92`, `P3=9`) plus one explicit canonical backlink.
  The backlink in `F-SEC-01` maps the duplicate audit/run-trace secret
  persistence observation to `F-OPS-01-P0-02`; it is not a 295th unique ID.
- ZCode-ds: 49 canonical P1 findings after its documented duplicate merge,
  plus lower-priority observations.
- ZCode-glm: 216 synthesized findings (`P1=16`, `P2=87`, `P3=113`).

Every Codex atomic ID was rechecked against the current definition,
registration, production call path, and tests. DS/GLM findings were then mapped
to the same owning task and root cause. A finding is closed only when its live
path was fixed or removed, or when the proposed action was rejected by an
explicit repository constraint. Validation reports are historical evidence;
the final executable gates in [framework-remediation.md](framework-remediation.md)
are the current closure evidence.

The generated [framework-finding-ledger.md](framework-finding-ledger.md) lists
all 294 IDs individually and accounts for the backlink as the 295th raw entry.
`scripts/verify-framework-finding-ledger.sh` fails if its count or ID set drifts
from the canonical task reports.

## Root-Cause Disposition

| Root family | Final authority and disposition | Main evidence |
|---|---|---|
| Terminal/error contract | One typed ReAct terminal; EOF, provider error, timeout, cancellation, truncation and tool failure cannot become empty success | ReAct phase/stream tests; structured `AgentFailure`; finish-reason tests |
| Cancellation/deadline/join | Invocation-scoped cancellation is propagated through providers, Tools, Subagents, workflows, transports, channels and owned background tasks | cancellation/timeout/race tests across ReAct, Subagent, workflow, MCP/LSP/A2A |
| Identity/correlation | Schema-v3 envelopes use validated, non-interchangeable IDs, stable event IDs/content hashes and monotonic sequence; adapters preserve them losslessly | compile-fail identity docs; envelope and EKO surface-contract tests |
| Durable state/recovery | Atomic replace, unique temp files, parent sync, corruption fail-close, bounded retention and attempt-scoped idempotency | corruption, torn-tail, restart, ABA, concurrent-write and stale-revision tests |
| Task/Subagent/workflow | One revisioned Task graph and one Subagent lifecycle own readiness, retries, cancellation, settlement and recovery | DAG, revision, recovery, dynamic insertion, worktree and Subagent lifecycle tests |
| Provider/Tool/integration | One neutral LLM/Tool contract; protocol adapters preserve reasoning, tool deltas, usage, finish reasons and structured failures | OpenAI/Anthropic stream fixtures; Tool registry/result tests; integration lifecycle tests |
| Context/memory/compression | Deterministic selection, checked budgets, protected tool pairs, bounded summaries and separate instruction/hot-memory projections | Unicode, token-bound, invariant, projection and corruption tests |
| Extension/public surface | Deterministic source-aware registries; dead parallel authorities removed; facade, features, macros, examples and docs match live code | facade compile test, strict rustdoc, feature isolation matrix |
| Local product boundary | No CLI SQLite; no automated-permission gate on user-interactive terminal/MCP; EKO policy remains in app adapters | manifests/static scans; local MCP tests; framework SQLite feature remains independently green |

## Per-Report Closure

The count column is the exact number of Codex canonical IDs in that report.
“Closed” includes all severities in the report, with DS/GLM aliases folded into
the same row.

| Report | IDs | Closure |
|---|---:|---|
| F-CORE-01 | 7 | Closed: core errors, budgets, circuit breaker, guard behavior and public contracts hardened |
| F-API-01 | 6 | Closed: facade/prelude/docs aligned; split-crate leakage removed |
| F-FEAT-01 | 4 | Closed: feature ownership/defaults/cfg topology corrected and isolated builds verified |
| F-REL-01 | 8 | Closed: retry, breaker, timeout and cancellation policy share one live authority |
| F-MAC-01 | 6 | Closed: macro facade paths/diagnostics fixed with downstream compile coverage |
| F-LLM-01 | 13 | Closed: neutral request/response/stream/usage/reasoning contract made lossless |
| F-LLM-02 | 6 | Closed: OpenAI-family parsing, deltas, usage, finish and malformed stream handling fixed |
| F-LLM-03 | 6 | Closed: Anthropic indices, thinking, cache, usage, finish and error streams fixed |
| F-RCT-01 | 6 | Closed: context preparation and execution invariants consolidated |
| F-RCT-02 | 5 | Closed: direct/non-stream execution uses typed terminal outcomes |
| F-RCT-03 | 6 | Closed: stream backpressure, ordering and cancellation terminals are lossless |
| F-RCT-04 | 7 | Closed: concurrent Tool results preserve call order and terminal settlement |
| F-RCT-05 | 6 | Closed: snapshots/recovery preserve identity, state and valid history |
| F-CTX-01 | 8 | Closed: deterministic bounded context selection and provider limits |
| F-MEM-01 | 8 | Closed: file stores are atomic, corruption-preserving and race-safe |
| F-MEM-02 | 7 | Closed: framework store implementations retained and independently hardened |
| F-CMP-01 | 11 | Closed: compression budgets/invariants/tool pairs and summary accumulation fixed |
| F-TSK-01 | 5 | Closed: one revisioned Task graph is the relationship/readiness authority |
| F-TSK-02 | 5 | Closed: claims, transitions, retries and dependency settlement are monotonic |
| F-TSK-03 | 5 | Closed: recovery identity, revisions and attempt idempotence prevent stale reuse |
| F-SUB-01 | 8 | Closed: Subagent isolation, recursion, tools, results and usage are enforced |
| F-SUB-02 | 10 | Closed: Fork/Team cancellation, concurrency, timeout and joins are bounded |
| F-MAG-01 | 7 | Closed: Team/handoff behavior folded into the single Subagent lifecycle |
| F-HITL-01 | 10 | Closed: effective edited input, timeout propagation, scope and audit fidelity fixed |
| F-EXT-01 | 7 | Closed: Tool schema, permissions, registration, result and retry semantics aligned |
| F-EXT-02 | 9 | Closed: file/Git/worktree confinement, atomicity, cancellation and rollback fixed |
| F-EXT-03 | 10 | Closed: domain Tools bounded; false persistence removed; pagination/Unicode fixed |
| F-SKL-01 | 6 | Closed: Skill discovery, precedence, dependencies and unload lifecycle deterministic |
| F-PLG-01 | 10 | Closed: Plugin identity, activation rollback, collisions and lifecycle unified |
| F-INT-01 | 14 | Closed: MCP transport async/SSE/retry/lifecycle and bounded payload behavior fixed |
| F-INT-02 | 8 | Closed: LSP/A2A/channel timeout, cancellation, cleanup and terminality fixed |
| F-WFL-01 | 10 | Closed: fan-out checkpoint/resume, interrupt and pending-branch state preserved |
| F-INTENT-01 | 7 | Closed: activation thresholds, routing and trigger lifecycle corrected |
| F-NBK-01 | 3 | Closed by removal: aspirational notebook surface had no production authority |
| F-OPS-01 | 11 | Closed: scheduler, trace, audit, retention, redaction and background ownership fixed |
| F-EVO-01 | 14 | Closed: path confinement, eval fidelity, curator corruption and candidate lifecycle fixed |
| F-TST-01 | 6 | Closed: mocks are strict/scriptable and cover real multi-chunk/failure shapes |
| F-SEC-01 | 9 | Closed: secrets redacted before sinks; local-app threat model applied without product-breaking gates |
| **Total** | **294** | **All canonical framework IDs reconciled** |

## Rejected Or Reframed Proposals

- Framework `SqliteStore` and `SqliteConversationStore` were not deleted merely
  because EKO does not use them. They are valid optional framework Store
  implementations. EKO manifests do not enable SQLite.
- Interactive terminal and MCP connection were not gated by `full-auto` or
  another agent permission mode. MCP validation rejects malformed composition,
  traversal and insecure remote HTTP while allowing user-selected executables,
  loopback HTTP and internal HTTPS.
- Framework APIs were not declared dead from CLI call-site absence. Public
  capability-menu implementations were retained unless replaced or invalid;
  private parallel/no-op authorities were removed.
- Application-only policy was not pushed into the framework. Domain profiles,
  worktree/reviewer choices, UI projections, workspace routing and retention
  values remain EKO-owned thin adapters over generic framework mechanisms.

## Residual Status

No review-owned defect remains open. All 294 canonical IDs and the one duplicate
backlink are accounted for, all applicable repository gates pass, and the final
static audit found no forbidden framework/application boundary regression.
