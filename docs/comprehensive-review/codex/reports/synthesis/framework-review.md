# S-FW-01: Framework review synthesis

> Status: complete
> Reviewer: Codex review subagent
> Accepted by: Codex primary reviewer
> Synthesis date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` boundary commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: external source/lock changes excluded; this synthesis changed
> only Codex synthesis and S-FW-01 validation reports

## Executive Conclusion

`echo-agent` has a credible reusable foundation, but it is not yet a
production-reliable framework. The problem is not lack of surface area: the
eight-package workspace provides substantial Agent, provider, Tool, memory,
Task, Subagent, integration, workflow and testing capability. The dominant
failure pattern is that public contracts outgrew their runtime authorities.
Identity, cancellation, terminal state, persistence, feature selection and
extension lifecycle are repeatedly represented in several partial mechanisms;
errors then become success/default/absence at their boundaries.

All 38 framework atomic tasks are complete. They contain 294 findings
(`P0=13`, `P1=180`, `P2=92`, `P3=9`) backed by 834 immutable validations. These
are not 294 independent projects. This synthesis retains every atomic ID and
reconciles them into nine root-cause families with one intended authority per
mechanism. It adds no duplicate synthesis finding IDs.

Static synthesis is complete even though executable quality evidence is not:
Q-FW-01/02 and Q-FLT-01/02 correctly remain `needs_evidence`. Their exact
unexecuted commands/scenarios are quality debt, not a reason to mislabel the
source synthesis incomplete.

## Evidence Coverage

### Atomic framework catalog

- Catalog: 38 F tasks.
- Codex reports: 38; all `complete`; no missing or extra IDs.
- Atomic validation reports: 834; minimum 10 per task.
- Atomic findings: 13 P0, 180 P1, 92 P2, 9 P3.
- Exact finding-ID collisions: 0; exact normalized-title collisions: 0.
- Direct F dependency closure: complete. Earlier parser/count attempts remain
  immutable history; [V01-05](../validations/S-FW-01/V01-05.md) and
  [V03-03](../validations/S-FW-01/V03-03.md) are the final counts.

All covered IDs:

`F-CORE-01`, `F-API-01`, `F-FEAT-01`, `F-REL-01`, `F-MAC-01`,
`F-LLM-01`, `F-LLM-02`, `F-LLM-03`, `F-RCT-01`, `F-RCT-02`,
`F-RCT-03`, `F-RCT-04`, `F-RCT-05`, `F-CTX-01`, `F-MEM-01`,
`F-MEM-02`, `F-CMP-01`, `F-TSK-01`, `F-TSK-02`, `F-TSK-03`,
`F-SUB-01`, `F-SUB-02`, `F-MAG-01`, `F-HITL-01`, `F-EXT-01`,
`F-EXT-02`, `F-EXT-03`, `F-SKL-01`, `F-PLG-01`, `F-INT-01`,
`F-INT-02`, `F-WFL-01`, `F-INTENT-01`, `F-NBK-01`, `F-OPS-01`,
`F-EVO-01`, `F-TST-01`, `F-SEC-01`.

### Quality and cross-cutting inputs

| Input | Status | Framework contribution |
|---|---|---|
| [Q-FW-01](../tasks/Q-FW-01.md) | needs_evidence | Submission commands not run; CI does not execute the exact all-target test gate; contributor commands drift. |
| [Q-FW-02](../tasks/Q-FW-02.md) | needs_evidence | Current static feature/example topology complete; current feature/example/doctest commands not run; no new duplicate finding. |
| [Q-STA-01](../tasks/Q-STA-01.md) | complete | New reachable UTF-8 parser, unsafe environment API and unchecked public arithmetic roots; duplicate atomic issues excluded. |
| [Q-FLT-01](../tasks/Q-FLT-01.md) | needs_evidence | Ten deterministic ReAct/Tool fault families specified, all not_run. |
| [Q-FLT-02](../tasks/Q-FLT-02.md) | needs_evidence | Ten deterministic Task/Subagent fault families specified, all not_run. |
| [Q-DEP-01](../tasks/Q-DEP-01.md) | complete | Public RS256 key/algorithm mismatch and non-executable advisory/license policy affect framework releases. |
| [Q-TST-01](../tasks/Q-TST-01.md) | complete | Known-red ReAct terminal test ignored; target-specific CI and cache propagation test credibility gaps. Frontend-only finding deferred to S-APP. |
| [Q-PERF-01](../tasks/Q-PERF-01.md) | complete | Framework hook backpressure participates in the application write-lock stall; application retention findings stay with S-APP. |

## Positive Conclusions

The synthesis must preserve what already works:

- The eight-package layering gives real reusable boundaries. Core types,
  execution, integrations, tools, state and orchestration are not one facade
  module pretending to be an architecture.
- Facade default features are empty. Independent consumers can select optional
  capabilities; framework SQLite is a legitimate optional Store implementation
  even though EKO must not enable it.
- The phase-based ReAct loop is the live execution authority. The old step
  processor is dead and removable; a wholesale second loop is unnecessary.
- Tool execution already has typed failure categories and conservative retry
  behavior for possible side effects. Those should be retained while outcome,
  identity and recovery are unified.
- Focused unit-test density is substantial. The app-facing `runtime_state_e2e`
  style demonstrates that real constructor seams can be tested without network
  providers. Structural stream mocks at `3aa7929` now represent separate delta,
  terminal usage and mid-stream error chunks.
- Registry dependencies are checksum-pinned and there are no Git-sourced
  packages in the reviewed framework lock. Multiple Store, compressor and
  provider choices are reasonable framework capability menus, not dead code.
- Q-STA's bounded scan established no pre-test production explicit unwrap,
  expect, panic, todo or unreachable call. Remaining panic/UTF-8/overflow
  findings are narrower but still material.

These positives mean the correct strategy is consolidation and hardening, not
rewriting the framework inside EKO or deleting public choices because one
application does not call them.

## Priority Synthesis

### P0: protect user data, secrets and core operation first

Every atomic P0 remains current at the synthesis boundary:

| Root risk | Canonical atomic findings |
|---|---|
| Destructive path escape / unverified cleanup | `F-EVO-01-P0-01`, `F-EVO-01-P0-02`, `F-EXT-02-P0-01`, `F-EXT-02-P0-02`, `F-OPS-01-P0-01` |
| Durable corruption, aliasing and lost updates | `F-MEM-01-P0-01`, `F-MEM-01-P0-02`, `F-MEM-01-P0-03`, `F-RCT-05-P0-04` |
| Secret persistence or logging | `F-OPS-01-P0-02`, `F-SEC-01-P0-01` |
| Approval fidelity | `F-HITL-01-P0-01` |
| Scheduler core path unusable | `F-OPS-01-P0-03` |

Fix direction: one path-confinement/atomic-file primitive, fail-closed durable
decode with original-byte preservation, secret redaction before every sink,
effective approved arguments as the sole execution/audit value, and a corrected
scheduler occurrence transition. These fixes remain in their owning framework
modules; no EKO state table is needed.

### P1: lifecycle and recovery correctness

The 180 P1 findings reduce to five dominant obligations:

1. Every invocation has one typed monotonic terminal. EOF, error, cancellation,
   block and partial output never become successful empty/final output.
2. Cancellation and deadlines own and join provider, Tool, Subagent, workflow,
   transport and background work; timeout is a lifecycle transition, not an
   early-return string.
3. Durable state is atomic, identity-bound and corruption-preserving. Restart
   cannot reset to empty, replay completed effects or accept an old claim.
4. IDs are canonical before side effects and preserved through events, traces,
   stores and protocol adapters.
5. Input/output, queues, retention, context and retry arithmetic are explicitly
   bounded with complete artifact/cursor recovery for truncated model views.

Representative owners include `F-CORE-01-P1-03`, `F-RCT-02-P1-01..03`,
`F-RCT-03-P1-02..05`, `F-RCT-04-P1-01..05`, `F-RCT-05-P1-01..05`,
`F-REL-01-P1-01..02`, `F-LLM-01..03`, `F-MEM-01..02`, `F-TSK-01..03`,
`F-SUB-01..02`, `F-MAG-01`, `F-INT-01..02`, `F-WFL-01`, `F-PLG-01`,
`F-CTX-01`, `F-CMP-01`, `F-OPS-01`, `F-EXT-01..03` and `F-SEC-01`.

### P2: remove duplicate authority and misleading contracts

P2 work should follow the P0/P1 authority decisions, not precede them:

- Delete dead/parallel runtime authorities after their live replacements own
  the path: old `process_steps`, disconnected provider adapter, one-wave Task
  readiness executor, dormant TeamRunner/Coordinator/Mailbox lifecycle and
  separate Handoff Agent execution lifecycle.
- Align facade/prelude/feature/example/docs with actual ownership. Do not retain
  no-op feature selectors, misleading default factories or undocumented split-
  crate requirements.
- Make Tool/schema/registration, provider selection, Task graph, snapshot and
  permission configuration have one enforceable owner instead of descriptive
  fields beside another runtime authority.
- Replace permissive implicit-success mocks and prose-only validation with
  strict scripts/production-connected negative controls.

### P3: localized cleanup

P3 items are real but should not distract from lifecycle repair: stale mock and
guard docs, safe zero-dimension constructors, bounded counters, epoch helper
duplication, notebook export correctness and similar local cleanup. Delete or
correct them when their owning modules are touched.

## Canonical Root-Cause Reconciliation

Atomic IDs remain authoritative. This table defines consolidation, not new
findings:

| Root family | Intended authority | Representative backlinks | Do not create |
|---|---|---|---|
| Typed terminal and error | One framework `TurnOutcome`/terminal commit, projected losslessly | F-CORE, F-RCT-02/03, F-LLM, F-EXT, F-SUB, F-WFL | Adapter-specific success inference or second EKO terminal state machine |
| Cancellation, deadline and join | Invocation-scoped cancellation/deadline propagated through owned child handles | F-REL, F-LLM, F-RCT-03/04, F-SUB-02, F-MAG, F-INT, F-WFL, F-OPS | Detached timeout wrappers or per-adapter cancellation semantics |
| Identity and correlation | Core invocation/turn/call/claim IDs canonicalized before side effects | F-CORE, F-RCT-04, F-TSK-03, F-SUB, F-PLG, F-OPS, F-INT | Late local ID repair or name-keyed ownership |
| Durable atomicity and corruption | Store-specific atomic commit plus common fail-closed corruption invariant | F-MEM, F-RCT-05, F-WFL, F-PLG, F-EVO, F-OPS | Universal second Store authority or corruption-as-empty fallback |
| Path and destructive mutation | Shared framework path/atomic-file primitives; domain owner decides mutation policy | F-EVO, F-EXT-02, F-OPS, F-SEC, F-WFL | String-prefix confinement or app-layer repairs to framework tools |
| Bounded I/O and retention | Reusable budget/cursor/artifact primitives; concrete retention policy at owner | F-CTX, F-CMP, F-RCT-03, F-EXT, F-INT, F-OPS, F-NBK | Silent truncation without complete locator or unbounded materialize-then-page |
| Task/Subagent execution | One revisioned Task graph/claim lifecycle and one Subagent invocation lifecycle | F-TSK-01..03, F-SUB-01..02, F-MAG, F-WFL | legacy non-Subagent execution terminology, second readiness wave, Handoff/Team parallel lifecycle |
| Provider/Tool/integration contract | One neutral LLM contract, one Tool schema/registry/executor, thin protocol adapters | F-LLM, F-EXT-01, F-INT-01, F-PLG | Disconnected ProviderAdapter, silent registration replace or rich-result flattening |
| Public feature/API/test truth | Cargo/public facade plus executable docs/tests as maintained contract | F-FEAT, F-API, F-MAC, F-TST, Q-FW/Q-TST/Q-DEP | No-op selectors, prose claims or historical pass relabeled current |

Provider/MCP/LSP/channel/storage manifestations are not erased by this table.
They need protocol-specific regression cases even when the generic primitive is
fixed once.

## Framework/Application Placement

### Framework-owned generic mechanisms

- Typed terminal/error, event identity/order and cancellation/deadline/join.
- Revisioned Task DAG, claim/attempt settlement and Subagent invocation modes.
- Tool schema/validation/registry/execution/result/artifact contracts.
- Neutral LLM request/response/stream/usage/cache contracts and thin provider
  adapters.
- Store atomicity/corruption semantics, path/atomic-file helpers, bounded queue/
  I/O/context/retry primitives and reusable telemetry redaction hooks.
- Deterministic strict mocks, clocks, fault injectors and feature/public API
  contract tests.

### Application-owned EKO policy

- Worktree/review/acceptance workflow, product artifact presentation and UI
  projections.
- Concrete local retention periods/byte caps, workspace selection, GUI/TUI/CLI/
  channel composition and local-assistant defaults.
- EKO TaskRuntime projection/outbox and transport choice. It may use framework
  lifecycle primitives but must not own a second DAG/claim/Subagent scheduler.
- EKO must not enable SQLite. This does not justify deleting framework SQLite.

### Adapter boundary

Provider, MCP/LSP/A2A/channel and EKO adapters translate identities, typed
outcomes, rich results, cancellation and cursors losslessly. They may inject
product policy, but they do not infer success, own retries/scheduling, mutate
canonical state under an extension callback, or repair missing identity after
execution.

## Required Authority Consolidation And Deletion

Deletion is part of completion, not a later indefinite phase:

1. Route all ReAct terminals through one typed commit; delete branch-specific
   terminal persistence and collector fallback success.
2. Delete dead `process_steps` and its PostToolBatch implementation after the
   live phase loop owns the hook.
3. Delete disconnected `ProviderAdapter` authority after the neutral provider
   registry/facade owns all callers.
4. Delete Task rich-record/one-wave readiness duplicates after the revisioned
   graph/controller owns live paths.
5. Fold Handoff and Team execution into the Subagent invocation lifecycle;
   delete their parallel Agent registry/run/terminal authorities while retaining
   genuinely distinct routing/topology policy.
6. Remove implicit mock success fallbacks and generic duplicate local stubs
   after one strict script can represent their contract. Keep protocol-byte
   fixtures and specialized cases where semantically distinct.
7. Remove duplicate replacement/config/helper paths after one source-aware
   registry and atomic file mutation path is live.
8. Delete stale/no-op feature/doc/example claims after Cargo/public ownership is
   aligned. Do not delete useful optional framework capabilities for EKO non-use.

Each staged cutover must switch a real caller and delete the replaced path in
the same milestone or record the exact next deletion target in the roadmap.

## Commit Freshness

Thirty-one atomic reports cite `9b0e0fa`; seven cite current `3aa7929`. This
does not make 31 reports stale. The transition is one commit changing six files:

- `src/agent/react/run/phases/tools.rs`
- `src/agent/react/run/pipeline.rs`
- `src/agent/react/run/stream_channel.rs`
- `src/testing/mock_llm.rs`
- `src/testing/mock_tool.rs`
- `src/testing/mod.rs`

Path-aware result ([V06-02](../validations/S-FW-01/V06-02.md)):

- Concurrent Tool results are now collected and emitted in model call order.
  This fixes an unnumbered ordering defect. It does not fix any of
  `F-RCT-04-P1-01..05` or `P2-06..07`: partial checkpoints, call-ID validation,
  cancel fairness, serial barriers, timeout settlement, overflow and dead hook
  ownership remain.
- New stream/mocking fixtures improve structural evidence but do not change the
  F-RCT-02/03 production terminal/cancellation defects. The deterministic
  truncated-stream fixture is intentionally red and ignored.
- F-TST-01 already re-reviewed the changed mocks at current commit and remains
  authoritative.
- `F-API-01-P2-05` is partially fixed: the facade testing doctest source now
  includes `focus_instructions: None`; the echo-core reverse-dependency doctest
  defect remains. No current doctest command ran, so no green claim is made.
- All other numbered findings are unaffected by the six-path diff and remain
  current. A future HEAD change requires another path-aware rebase.

## Unexecuted Quality Debt

Static review completion is not a submission/release pass:

- Q-FW-01: five framework submission commands are not run at current commit.
- Q-FW-02: current standalone feature compilation, 27 required-feature example
  groups and eight per-package doctest groups are not run. Historical feature
  passes at `9b0e0fa` are precedent only.
- Q-FLT-01: ten ReAct/Tool malformed/Unicode/large/timeout/cancel/disconnect/
  crash/partial-effect/effective-input/artifact scenarios are not run.
- Q-FLT-02: ten Task/Subagent revision/ABA/cancel/timeout/crash/sibling/DAG/
  worktree/review/artifact scenarios are not run.
- Q-STA, Q-DEP, Q-TST and Q-PERF retain dynamic Unicode/extreme/unsafe cases,
  current advisory/license lookup, mutation controls, native target lanes and
  performance/backpressure profiles as future evidence.
- One deterministic ReAct terminal regression is excluded with `#[ignore]`.
- CI does not execute the exact documented all-target framework test gate, and
  README/CONTRIBUTING publish weaker command variants.

These debts must be new immutable attempts after fixes or explicit execution
authorization. They are not silently waived by S-FW completion.

## Validation Matrix

| ID | Claim | Final status | Report |
|---|---|---|---|
| V01 | 38-task catalog/report/validation coverage | attempts 01/03 failed; 02/04/05 passed, final 05 | [01](../validations/S-FW-01/V01-01.md), [02](../validations/S-FW-01/V01-02.md), [03](../validations/S-FW-01/V01-03.md), [04](../validations/S-FW-01/V01-04.md), [05](../validations/S-FW-01/V01-05.md) |
| V02 | Direct dependency closure | attempts 01/02 failed; 03 passed | [01](../validations/S-FW-01/V02-01.md), [02](../validations/S-FW-01/V02-02.md), [03](../validations/S-FW-01/V02-03.md) |
| V03 | Finding IDs, title uniqueness and priority counts | attempt 01 failed; 02/03 passed | [01](../validations/S-FW-01/V03-01.md), [02](../validations/S-FW-01/V03-02.md), [03](../validations/S-FW-01/V03-03.md) |
| V04 | Semantic duplicate/canonical-owner reconciliation | attempts 01/02 passed | [01](../validations/S-FW-01/V04-01.md), [02](../validations/S-FW-01/V04-02.md) |
| V05 | P0-P3 root-risk aggregation | attempts 01/02 passed | [01](../validations/S-FW-01/V05-01.md), [02](../validations/S-FW-01/V05-02.md) |
| V06 | `9b0e0fa -> 3aa7929` stale-commit classification | attempt 01 inconclusive; 02 passed | [01](../validations/S-FW-01/V06-01.md), [02](../validations/S-FW-01/V06-02.md) |
| V07 | Required Q dependency/status coverage | 01 passed; 02 failed scripting; 03 passed correction | [01](../validations/S-FW-01/V07-01.md), [02](../validations/S-FW-01/V07-02.md), [03](../validations/S-FW-01/V07-03.md) |
| V08 | Interim report integrity | attempt 01 failed; 02 passed | [01](../validations/S-FW-01/V08-01.md), [02](../validations/S-FW-01/V08-02.md) |
| V09 | Positive framework conclusions | passed | [V09-01](../validations/S-FW-01/V09-01.md) |
| V10 | Framework/application/adapter placement and deletion gate | passed | [V10-01](../validations/S-FW-01/V10-01.md) |
| V11 | Unexecuted quality-debt audit | passed | [V11-01](../validations/S-FW-01/V11-01.md) |
| V99 | Final synthesis link/status/source integrity | passed | [V99-01](../validations/S-FW-01/V99-01.md) |
| V30 | Primary coverage, P0 and freshness sampling | passed | [V30-01](../validations/S-FW-01/V30-01.md) |
| V31 | Mandatory Subagent terminology | passed | [V31-01](../validations/S-FW-01/V31-01.md) |

## Residual Uncertainty

- No build, test, dynamic fixture, fault injection, coverage profile, advisory
  query or network check was run by this synthesis.
- Static findings establish code paths and contract mismatches, not observed
  production frequency. Priorities follow impact if reached and verified live
  reachability from atomic reports.
- Semantic root families overlap and must not be summed. Atomic reports remain
  the source for exact evidence, confidence and regression scenarios.
- Source line anchors can shift under the external worktree changes. Fix tasks
  must use committed `3aa7929` blobs or a later explicitly rebased commit.

## Downstream Handoff

- `S-X-01` should preserve the generic authorities above and make EKO adapters
  lossless; it must not repair framework lifecycle by adding application state
  machines.
- `S-QA-01` owns command/scenario execution closure and must distinguish
  `not_run`, ignored, historical, static-failed and current-pass evidence.
- `S-RDM-01` should order: P0 destructive/corrupt/secret/scheduler/approval;
  typed terminal+cancel+identity; durable recovery/claim/effect ledger; bounded
  I/O/retention; authority deletion/API/feature cleanup.
- Each roadmap item must link atomic IDs, name the one final authority, list
  deleted paths/contracts, specify negative controls and state framework versus
  application ownership.
- This synthesis becomes stale if an F/Q task status or finding changes, either
  reviewed commit changes, or the six transition paths change again.
