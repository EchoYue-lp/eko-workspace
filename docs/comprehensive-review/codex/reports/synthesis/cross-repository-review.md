# S-X-01: Cross-repository review synthesis

> Status: complete
> Reviewer: Codex review subagent
> Accepted by: Codex primary reviewer
> Synthesis date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: current external framework changes and CLI `Cargo.lock` change excluded; only Codex synthesis/validation reports written

## Executive Conclusion

The repository boundary is conceptually sound but operationally incomplete.
`echo-agent` already contains the reusable Agent, event, Task, Subagent, Tool,
artifact, Store, integration and lifecycle primitives. EKO correctly owns local
workspace identity, product policy, file persistence, worktree/review behavior
and surface presentation. The dominant cross-repository failure is that this
separation stops inside the adapters: identity and typed outcomes are collapsed,
then EKO reconstructs scheduling, terminal, retry, artifact, recovery or surface
meaning from partial fields.

The remediation is authority convergence, not a rewrite. Extend one framework
contract where the mechanism is generic; project it losslessly into one EKO
application service/generation; switch real callers; then delete the displaced
framework or adapter-local mechanism. Do not add another event bus, Task graph,
permission engine, artifact reader, surface registry or persistence authority.

All 10 X tasks and both phase syntheses are primary-complete at the pinned
commits. The X phase has 37 findings (`P0=2`, `P1=29`, `P2=6`) backed by 124
immutable validation reports. This synthesis adds no finding ID and preserves
all original owners. Static S-X-01 is complete. Unexecuted dynamic gates remain
quality debt and do not become implied runtime proof.

Validation: [V00](../validations/S-X-01/V00-01.md),
[V02](../validations/S-X-01/V02-01.md),
[V08](../validations/S-X-01/V08-01.md).

## Inputs And Scope

Consumed completed Codex dependencies only:

- Phase syntheses: [S-FW-01](framework-review.md) and
  [S-APP-01](application-review.md).
- Cross contracts: [X-AUT-01](../tasks/X-AUT-01.md),
  [X-BND-01](../tasks/X-BND-01.md),
  [X-EVT-01](../tasks/X-EVT-01.md),
  [X-INV-01](../tasks/X-INV-01.md),
  [X-MEM-01](../tasks/X-MEM-01.md),
  [X-PLG-01](../tasks/X-PLG-01.md),
  [X-SRF-01](../tasks/X-SRF-01.md),
  [X-STA-01](../tasks/X-STA-01.md),
  [X-TOL-01](../tasks/X-TOL-01.md), and
  [X-TSK-01](../tasks/X-TSK-01.md).

Out of scope: new source inspection beyond the accepted reports, source edits,
builds, tests, dynamic fixtures, application/browser launch and network. Current
uncommitted source and lockfile state was not used as evidence.

## Boundary Gate

| Classification | Required answer | Synthesis decision |
|---|---|---|
| Generic mechanism | Would unrelated framework consumers need it? | Typed identity/order/terminal, cancellation/deadline/join, revisioned Task DAG/claim settlement, canonical Subagent execution, Tool schema/effective invocation/result/artifact, Store corruption/atomicity and extension lifecycle receipts belong to `echo-agent`. |
| EKO product policy | What local-assistant decision makes it specific? | Workspace/conversation generations, DomainProfile, worktree/review/acceptance, local file retention, surface availability, pool propagation, webhook labels and direct-user interaction policy belong to EKO. |
| Adapter boundary | Is conversion thin and lossless? | Target adapters preserve every generic field, inject product identity/policy, call one authority and return typed facts. They never own a scheduler, retry loop, terminal inference, persistence truth or weaker artifact reader. Current adapters fail this test in the specialized X findings below. |
| Duplicate search | What was searched? | The X phase searched definitions, traits, fields, exports, registrations and live paths for events, Task graphs, Subagent lifecycles, Tool/artifact flows, stores/recovery, plugin/memory generations, surfaces, permissions and legacy/disconnected APIs across both repositories. |
| Migration deletion | What disappears after cutover? | Every staged authority has a named deletion target in the ledger below. Public framework APIs are deleted only after framework-wide replacement and reasonable external-use review; EKO non-use alone is insufficient. |

The seven explicit X layering tables, X-BND capability map, X-TSK authority map
and X-INV canonical-owner audit collectively satisfy the gate. Validation:
[V01](../validations/S-X-01/V01-01.md).

## Target Ownership Model

| Domain | Framework authority | EKO authority | Thin adapter contract |
|---|---|---|---|
| Agent events | versioned EventEnvelope, stable identity/order/parent and typed terminal | durable ordinary-turn ownership, retention, replay cursor and UI state | carry the full envelope; add product refs; never infer terminal from transport completion |
| Task | one revision service, validator, DAG analyzer, attempt/claim/retry/cancel/settlement safe point | DomainProfile, resource policy, Subagent dispatch, worktree, review and UI projection | field-complete Task conversion and typed dispatch facts; no second frontier or settlement loop |
| Subagent | one definition/catalog/invocation/context/cancel/deadline/outcome/event lifecycle | role sources, prompt policy, pool generation, files and acceptance | prepare source/product context and dispatch through the one lifecycle |
| Tool/artifact | schema, requested/effective invocation, typed terminal, complete artifact descriptor and verified paging | conversation/run ownership, retention root and lazy UX | persist/render the canonical terminal and descriptor without raw-path paging or parent-status inference |
| Store/recovery | reusable atomicity, corruption, checksum and typed recovery primitives | file-backed aggregate, generation/tombstone, retention and cross-store transaction policy | publish one validated generation and report partial commit explicitly |
| Memory/plugin | generic Store/compression/Skill/Hook/extension lifecycle mechanisms | enabled roots, instruction precedence, rule policy, primary/current/future pool generation | consume typed receipts and reconcile exactly one source generation everywhere |
| Surface/workspace | reusable mechanisms and typed capabilities | one application capability manifest, active-turn registry, workspace identity and renderer behavior | bind triggers/presentation to shared services and canonical snapshots |
| Permission/security | automated Tool policy, protected-path/sandbox primitives and secret-safe observability | direct local interaction policy, workspace write revisions and endpoint labels | apply automation policy only to automated actions; retain light malformed-input and data-loss checks |

This retains framework independence: optional SQLite Stores, compressors,
integrations, workflow and Tool domains are not deletion targets merely because
EKO does not select them. EKO remains file/in-memory based and does not enable
SQLite. Validation: [V04](../validations/S-X-01/V04-01.md).

## Lossless Adapter Contract

The migration is accepted only when these field groups survive definition ->
adapter -> persistence -> surface/replay without inference:

| Boundary | Required canonical facts | Current loss/extra authority |
|---|---|---|
| Event/turn | schema, event ID, sequence, timestamp, conversation/run/turn/execution/parent, payload kind and one terminal | GUI strips envelope; reducers and transport independently infer success/cancel; ordinary chat lacks durable replay |
| Tool | requested and effective name/arguments, typed failure, cancel/timeout/interrupted/unknown, stream observations, finality | GUI stores requested input while a rewritten invocation executes; parent status becomes Tool cancellation |
| Artifact | stable artifact/detail ID, complete source, digest, bytes, truncation, retention and snapshot-bound cursor | rich result collapses to text; EKO raw-path reader omits digest/snapshot checks and may prefer partial chunks |
| Task | run/revision/task/attempt/claim/execution, retry decision, side-effect facts and one settlement | specification round-trip is positive, but bootstrap precedes commit and controller/store own generic retry/terminal semantics |
| Subagent | source/catalog generation, role/context, invocation, cancel/deadline, result/error and artifact lineage | Team/Handoff and pool/plugin projections can bypass or diverge from canonical execution/catalog state |
| Durable generation | workspace/conversation/run generation, revision/order/checksum, typed corruption/partial commit and tombstone | decode failure can become empty state; JSONL tail can hide valid history; recovery publishes stores sequentially |
| Plugin/memory | source ID, generation, committed component/layer receipt, partial failure and reconciliation obligation | primary/current/future Agents can expose different component/memory generations; late failure suppresses refresh |
| Surface | capability identity, supported/available reason, canonical event/snapshot/artifact and active-turn handle | handwritten composition and renderer-specific projections omit real services, cancellation and evidence |

Validation: [V03](../validations/S-X-01/V03-01.md).

## Canonical Finding Reconciliation

Every original X finding remains a separately fixable/backlinked ID. The table
prevents umbrella architecture findings from double-counting specialized field
defects.

| Canonical implementation family | Original IDs retained | Reconciliation rule |
|---|---|---|
| Boundary and duplicate authorities | `X-BND-01-P1-01`, `X-BND-01-P1-02`, `X-BND-01-P1-03`, `X-BND-01-P1-04`, `X-BND-01-P2-05` | Architecture placement/deletion gates. For field implementation, cite the specialized owner below as well. |
| Event lifecycle | `X-EVT-01-P1-01`, `X-EVT-01-P1-02`, `X-EVT-01-P1-03`, `X-EVT-01-P1-04`, `X-EVT-01-P1-05`, `X-EVT-01-P2-06` | Own envelope fields, terminal contradiction, versioning, live/replay identity, ordinary durable replay and boundary tests. |
| Surface parity | `X-SRF-01-P1-01`, `X-SRF-01-P1-02`, `X-SRF-01-P1-03`, `X-SRF-01-P1-04`, `X-SRF-01-P2-05` | Own production capability composition, renderer parity, active-turn ownership, pool prompt projection and executable parity contract. Event/Tool fields retain X-EVT/X-TOL ownership. |
| Tool and artifacts | `X-TOL-01-P1-01`, `X-TOL-01-P1-02`, `X-TOL-01-P2-03`, `X-TOL-01-P1-04` | Own effective invocation, typed terminal/complete result, verified paging and Tool outcome classification. |
| Task graph/adapter | `X-TSK-01-P1-01`, `X-TSK-01-P1-02`, `X-TSK-01-P1-03`, `X-TSK-01-P1-04`, `X-TSK-01-P2-05` | Own bootstrap transaction, shared DAG behavior, attempt settlement, crash-atomic projection and remaining parallel schedulers. X-BND remains the placement gate. |
| Persistence and identity | `X-STA-01-P0-01`, `X-STA-01-P1-02`, `X-STA-01-P1-03`, `X-STA-01-P1-04`, `X-STA-01-P1-05` | Own corrupt-state admission, tail recovery, cross-store generation, durable lineage and deletion tombstone. Event merge and Tool artifact format retain their specialized owners. |
| Memory generation | `X-MEM-01-P1-01`, `X-MEM-01-P1-02` | Own workspace binding publication and truthful promotion receipt/reconciliation, not generic Store implementation choice. |
| Plugin generation | `X-PLG-01-P1-01`, `X-PLG-01-P1-02` | Own atomic primary/pool component generation and live routing refresh, not generic Skill/Hook APIs. |
| Local permission/data safety | `X-AUT-01-P0-01`, `X-AUT-01-P1-02`, `X-AUT-01-P2-03` | Own webhook URL redaction, direct-user workspace over-gating and duplicate non-revisioned writer. |
| Repository hard constraints | X-INV-01 adds no finding ID | Retain its canonical backlinks; positive Subagent terminology, no-CLI-SQLite and relative-path conclusions remain gates. |

Specific semantic overlaps are intentionally split:

- `X-BND-01-P1-03` diagnoses thick adapters; X-EVT, X-TOL and X-SRF own
  exact lost fields and consumers.
- `X-BND-01-P1-04` gates placement; `X-TSK-01-P1-03` owns attempt/retry/cancel/
  settlement behavior.
- `X-BND-01-P1-01` owns the second framework Task graph; X-TSK-01-P2-05 also
  owns the EKO cross-run polling scheduler.
- `X-STA-01-P1-03` owns durable aggregate recovery; X-MEM-01-P1-01 and
  X-PLG-01-P1-01 own live per-Agent generation publication.
- `X-STA-01-P1-04` owns durable artifact lineage; X-EVT-01-P1-04 owns
  live/replay event merge and X-TOL-01-P1-02 owns complete Tool output.

No contradiction required reopening source. Validation:
[V02](../validations/S-X-01/V02-01.md).

## Cross-Repository Merge Order

Repository order is part of correctness because `echo-agent-cli` depends on
`echo-agent`.

1. **Contain independent P0s.** In EKO, redact credential-bearing webhook URLs
   (`X-AUT-01-P0-01`) and fail closed/quarantine corrupt state before any fresh
   overwrite (`X-STA-01-P0-01`). Apply the phase-synthesis P0 data/secret
   containment in the owning repository without changing shared authority.
2. **Add/repair framework contracts first.** Make typed envelope/terminal,
   effective Tool invocation and complete artifact descriptor, Task
   DAG/attempt/cancel/settlement, canonical Subagent dispatch, corruption and
   lifecycle receipts coherent in `echo-agent`. Merge this repository first.
   Additions are allowed only to enable the named EKO cutover, not as parallel
   facades without a caller.
3. **Cut EKO app-core to the contracts.** Introduce one active-turn/capability
   service, lossless event/Tool projection, recoverable Task commit/store,
   workspace aggregate generation, and plugin/memory reconciler. Switch at
   least one real GUI/TUI/CLI/channel/Task path in each item before continuing.
4. **Update Rust wires, generated TypeScript and reducers together.** Carry the
   same identity/terminal/artifact schema through Tauri and frontend, then bind
   all surfaces to the application service. Generated consumers change with
   their CLI Rust producer, never ahead of it.
5. **Delete EKO displaced authority in the cutover.** Remove adapter-local
   schedulers, inference, raw readers, service registries and per-surface
   cleanup/terminal logic once the production caller has moved.
6. **Delete fully displaced framework authority last.** After the updated CLI
   no longer calls it and framework-wide search rejects reasonable public use,
   remove old Task graph/store semantics, direct Team/Handoff execution
   lifecycles and disconnected provider/loop paths. Do not retain deprecation
   compatibility. Run the deferred Q boundary fixtures and quality commands
   only after the source migration.

This `echo-agent` additive -> `echo-agent-cli` cutover -> `echo-agent` removal
order keeps intermediate mains usable. If one delivery cannot finish all three,
the temporary authority and exact removal milestone must be recorded in
`docs/MASTER-PLAN.md`. Validation: [V05](../validations/S-X-01/V05-01.md).

## Deletion Ledger

| Cutover authority | Delete after real caller migration | Acceptance/deletion criterion |
|---|---|---|
| EventEnvelope -> one EKO envelope/replay | handwritten payload-only live unions, dormant StreamingEvent/ServerMessage and terminal inference | all identity fields round-trip; unknown material/terminal variant fails explicitly; one terminal survives replay |
| Framework Tool terminal/artifact service | EKO PendingToolCompletion, raw-path paging/cursor parsing, stream-over-final precedence and parent-to-cancel inference | detail/copy returns verified complete source and distinct failed/timed_out/cancelled/interrupted outcomes |
| Revisioned Task service/executor | legacy framework graph mutation/store/readiness after external-use review; EKO retry/settlement helpers and background dependency polling | one revision/attempt/claim owns every physical attempt and terminal; no second ready frontier |
| Canonical Subagent lifecycle | Team/Handoff raw Agent registries, schedulers, result classifiers and name-only topology inference | every target/member dispatch carries canonical invocation/cancel/outcome identity |
| EKO capability/active-turn service | surface-local service startup/command lists, anonymous cancellation ownership and Tauri workflow/diff algorithms | all applicable surfaces expose the same live service and canonical snapshot; trigger/presentation is the only variation |
| EKO aggregate generation | decode-to-empty fallbacks, split sequential recovery publication and per-surface deletion cascades | corrupt/partial state is quarantined; one generation publishes atomically; tombstone fences late writes |
| Plugin/memory generation reconciler | additive Skill refresh, primary-only Subagent/style mutation, frozen bootstrap router and surface `if Ok { refresh }` sequences | primary/current/future pooled Agents report the same source generation after success, rollback and restart |
| Canonical revisioned workspace writer | registered native non-revisioned writer and duplicate bridge | stale overwrite is rejected through the sole writer; no production/registered bypass remains |
| Production capability/parity manifest | prose-only surface matrix and duplicate registration lists | removing a real binding or canonical field fails an executable contract fixture |

Framework SQLite Stores, generic compressors, integrations, workflows and Tool
domains are explicitly excluded from deletion based solely on EKO usage.

## Priority Order Across Boundaries

1. **P0 containment:** corrupt-state overwrite and credential-bearing webhook
   logging, plus phase-synthesis P0 data-loss/secret paths.
2. **Identity and terminal spine:** EventEnvelope -> typed turn outcome ->
   Task/Subagent/Tool identity -> durable replay. This is prerequisite to honest
   UI status, cancellation and retry.
3. **Durable generation:** Task commit/store, workspace/memory/plugin binding,
   checksum/quarantine/tombstone and complete artifact lineage.
4. **Authority deletion:** remove second Task/Subagent/Tool/surface engines as
   their real callers move.
5. **Surface parity:** derive registrations from one capability manifest and
   prove facts/artifacts survive every renderer and restart.

This ordering prevents UI-level fixes from cementing lossy adapters and keeps
generic repair in the framework while EKO acceptance/worktree/presentation
policy remains in the application.

## Positive Conclusions

- The main EKO Task path already reuses PlanValidator, TaskRevisionService and
  RuntimeDagExecutor; Task specification and patch conversion are field-complete.
- The framework EventEnvelope and EKO TaskRuntime append-only stream are usable
  foundations for one live/durable contract.
- Framework ToolResult/artifact writing and verified artifact reading already
  contain richer facts than the current application projection.
- FileConversationStore has strong atomic rename/sync/fail-closed behavior; the
  broad problem is aggregate coordination, not absence of all durable safety.
- Shared Tool and Hook registries update live, and one-Agent memory Store
  installation rewires both memory Tools and compression promotion correctly.
- Automated Agent actions reach the framework permission service, while direct
  terminal/file/MCP/Browser interactions are not currently blocked by the stale
  IPC gate.
- Maintained source uses Subagent terminology, CLI does not enable SQLite, and
  Cargo/worktree paths satisfy the repository portability rule.
- Both phase syntheses agree that existing authorities should be extended and
  duplicate semantics deleted rather than replaced by a wholesale architecture.

Validation: [V07](../validations/S-X-01/V07-01.md).

## Historical Status And Uncertainty

- Findings and source anchors are current at the pinned commits. Current
  external dirty bodies and CLI `Cargo.lock` were excluded. Older framework
  evidence was already reconciled by S-FW-01 and is not independently adopted.
- S-APP-01 is primary-complete. Its unrun Q-CLI/Q-GUI/Q-WEB commands remain
  release-readiness debt, not missing static synthesis evidence.
- X dynamic fixtures were prohibited. V08 records the future envelope, Tool,
  Task, generation and surface matrices. No runtime timing/frequency claim is
  inferred from their absence.
- Framework public API deletion requires a current repository-wide search and
  reasonable external-consumer judgment. Unused-by-EKO is never sufficient.
- This report becomes stale when either reviewed commit changes or when a
  canonical contract/cutover above lands; reopen the smallest owning X
  validation rather than repeating the full synthesis.

Validation: [V06](../validations/S-X-01/V06-01.md),
[V08](../validations/S-X-01/V08-01.md).

## Validation Matrix

| ID | Claim | Status | Report |
|---|---|---|---|
| V00 | Dependency/status/revision/isolation completeness | passed | [V00-01](../validations/S-X-01/V00-01.md) |
| V01 | Cross-repository boundary-gate completeness | passed | [V01-01](../validations/S-X-01/V01-01.md) |
| V02 | Canonical duplicate/owner reconciliation | passed | [V02-01](../validations/S-X-01/V02-01.md) |
| V03 | Adapter field-loss and semantic-authority recheck | passed | [V03-01](../validations/S-X-01/V03-01.md) |
| V04 | Framework/EKO/adapter target placement | passed | [V04-01](../validations/S-X-01/V04-01.md) |
| V05 | Cross-repository merge order and deletion criteria | passed | [V05-01](../validations/S-X-01/V05-01.md) |
| V06 | Commit freshness and historical-evidence classification | passed | [V06-01](../validations/S-X-01/V06-01.md) |
| V07 | Positive-authority preservation | passed | [V07-01](../validations/S-X-01/V07-01.md) |
| V08 | Future executable boundary fixtures | not_run | [V08-01](../validations/S-X-01/V08-01.md) |
| V99 | Links, finding coverage, status, isolation and terminology | V99-01 failed; V99-02 passed; V99-03 passed | [01](../validations/S-X-01/V99-01.md), [02](../validations/S-X-01/V99-02.md), [03](../validations/S-X-01/V99-03.md) |
| V30 | Primary boundary, ID, merge-order and deletion sampling | passed | [V30-01](../validations/S-X-01/V30-01.md) |

## Handoff

Use the ownership matrix as the gate for every roadmap item. Start with the two
cross-contract P0s, then build the identity/terminal spine before persistence
and surface work. Every staged adapter must switch a real caller and name its
deletion target; a new facade with the old semantic loop still live is not
progress. Preserve all original finding IDs in implementation and acceptance
tracking, and run the deferred dynamic matrices only after their source owner
has changed.
