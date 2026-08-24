# S-RDM-01: Prioritized Iteration Roadmap

> Status: complete
> Reviewer: Codex review subagent
> Executor: Codex review subagent
> Accepted by: Codex primary reviewer
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: framework external changes and CLI `Cargo.lock` were excluded; only Codex review reports were read or written.

## Decision

Iterate by converging existing authorities, not by rewriting both repositories.
The order is P0 containment -> generic identity/terminal/cancellation and durable
primitives -> canonical Task/Subagent/Tool execution -> one EKO aggregate and
generation -> lossless surface adapters -> same-milestone deletion -> quality
and parity execution. The roadmap contains 53 fresh-task-sized milestones and
does not authorize an indefinite compatibility layer.

Static synthesis and primary roadmap acceptance are complete. Separately, eight
Q tasks still require executable evidence; roadmap acceptance does not turn
those `not_run` attempts into green claims.

## Inputs And Frozen Evidence

- [B-REF-01 mature implementation matrix](../tasks/B-REF-01.md).
- [S-FW-01 framework synthesis](framework-review.md): 38 tasks, 294 findings
  (`P0=13`, `P1=180`, `P2=92`, `P3=9`), 834 validations.
- [S-APP-01 application synthesis](application-review.md): 29 tasks, 150
  findings (`P0=9`, `P1=109`, `P2=32`, `P3=0`), 422 validations.
- [S-X-01 cross-repository synthesis](cross-repository-review.md): 10 tasks,
  37 findings (`P0=2`, `P1=29`, `P2=6`), 124 validations.
- [S-QA-01 quality synthesis](quality-and-validation-review.md): 13 tasks,
  24 findings (`P1=8`, `P2=12`, `P3=4`), 191 attempts
  (`64 passed / 44 failed / 16 inconclusive / 67 not_run`). Five Q tasks are
  statically complete; eight still need executable evidence.

This roadmap adds no product finding ID. It retains atomic IDs as evidence and
assigns implementation milestone IDs `RDM-00` through `RDM-52` only.

## Non-Negotiable Architecture Constraints

| Ref | Roadmap constraint | Mature evidence and limit |
|---|---|---|
| MR-PLAN | `TaskPlan` is a versioned artifact/projection of the revisioned Task graph, never a second CRUD store or approval runtime state. | Cursor's editable Plan artifact is established by [B-REF V07](../validations/B-REF-01/V07-01.md). Claude's internal representation is explicitly unconfirmed by [V15](../validations/B-REF-01/V15-01.md); no stronger claim is made. |
| MR-EVT | One typed, versioned event/history contract carries stable identities, order and one terminal; UI and text streams are projections. | Codex typed lifecycle [V03](../validations/B-REF-01/V03-01.md), Codex resume/history [V04](../validations/B-REF-01/V04-01.md), Temporal durable replay [V13](../validations/B-REF-01/V13-01.md). |
| MR-SUB | One Subagent lifecycle owns lineage, context, attempt, cancel/deadline, typed result and artifacts. | Codex [V06](../validations/B-REF-01/V06-01.md), Cursor [V08](../validations/B-REF-01/V08-01.md), Devin [V10](../validations/B-REF-01/V10-01.md), Temporal child execution [V14](../validations/B-REF-01/V14-01.md). |
| MR-TRY | Retry is attempt-scoped with stable parent lineage and idempotence/settlement facts; never replay already committed effects invisibly. | Temporal [V14](../validations/B-REF-01/V14-01.md). |
| MR-POL | Automated-action approval policy is separate from sandbox enforcement and from direct user interaction. Local terminal, file picker, MCP and Browser availability are not gated by an automation mode. | Claude [V02](../validations/B-REF-01/V02-01.md), Codex [V05](../validations/B-REF-01/V05-01.md), Devin [V11](../validations/B-REF-01/V11-01.md), convergence [V17](../validations/B-REF-01/V17-01.md). |
| MR-PLG | Skill/Plugin sources retain source-scoped identity, snapshot/generation, enable/disable and cleanup receipts. | Claude [V01](../validations/B-REF-01/V01-01.md), Codex [V06](../validations/B-REF-01/V06-01.md), Cursor [V09](../validations/B-REF-01/V09-01.md), Devin [V12](../validations/B-REF-01/V12-01.md). |
| MR-REC | Recovery uses stable identity plus authoritative history/checkpoints; stale attempts and corrupt generations fail closed. | Codex [V04](../validations/B-REF-01/V04-01.md), Temporal [V13](../validations/B-REF-01/V13-01.md). |

Project constraints add three hard gates:

1. EKO uses file/in-memory persistence and must not enable framework SQLite;
   framework SQLite remains a reasonable optional public capability.
2. GUI, TUI, CLI, channels, cron and background share complete Agent facts and
   capabilities. Only triggers, rendering and interaction-required responses vary.
3. There is only `Subagent` terminology and one `TaskRun -> PlanTask ->
   SubagentRun` authority.

## Placement And Migration Gate

| Classification | Placement | Gate before implementation |
|---|---|---|
| Generic mechanism | `echo-agent`: typed identity/order/terminal, cancellation/deadline/join, Task DAG/revision/claim/attempt settlement, Subagent invocation, Tool requested/effective invocation/result/artifact, atomic/corruption primitives and extension lifecycle receipts. | Search both repositories for existing definitions, exports, registrations and real callers. Extend the existing authority and add no facade unless one production EKO caller switches in the paired cutover. |
| EKO product policy | `echo-agent-cli`: workspace/conversation generation, DomainProfile, local file retention, worktree/review/acceptance, prompt policy, surface availability, direct-user interaction policy and UI presentation. | One application service owns policy and persisted aggregate; EKO does not reimplement generic DAG, retry, terminal, artifact verification or cancellation semantics. |
| Adapter boundary | EKO app-core/Tauri/channel/TUI/CLI adapters: lossless type conversion, product identity/policy injection and rendering/transport. | Preserve all generic facts round-trip. An adapter with a scheduler, retry loop, terminal inference, recovery truth, raw artifact authority or separate registry fails acceptance. |
| Cross-repository delivery | Framework contract first, EKO caller cutover second, fully displaced framework path last. | Keep main usable at every merge; never commit an absolute worktree dependency. If a delivery cannot finish, `docs/MASTER-PLAN.md` must name the temporary path and exact next deletion milestone. |

## Scope Scale

Scope is relative engineering size, not calendar time: `S` = one focused module
or 2-5 production files; `M` = one authority across 5-10 files; `L` = one bounded
cross-module authority across 10-18 files. Every row is one fresh task. If a row
exceeds its bound, split by its listed acceptance cases rather than widening it.

## P0 Ledger

All 24 P0 IDs route before lower-priority convergence. Cross findings are
acceptance evidence on the owning fix, not a second implementation stream.

| Milestone | Canonical P0 evidence | Root outcome |
|---|---|---|
| RDM-00 | [F-EVO-01-P0-01, F-EVO-01-P0-02](../tasks/F-EVO-01.md), [F-OPS-01-P0-01](../tasks/F-OPS-01.md), [F-MEM-01-P0-03](../tasks/F-MEM-01.md) | Validated, namespace-safe identifiers and confined roots before filesystem mutation. |
| RDM-01 | [F-EXT-02-P0-01, F-EXT-02-P0-02](../tasks/F-EXT-02.md) | Non-destructive checkpoint/worktree transaction with verified rollback and cleanup. |
| RDM-02 | [F-MEM-01-P0-01, F-MEM-01-P0-02](../tasks/F-MEM-01.md), [F-RCT-05-P0-04](../tasks/F-RCT-05.md) | Atomic file generation and corruption quarantine, never decode-to-empty overwrite. |
| RDM-03 | [F-OPS-01-P0-02](../tasks/F-OPS-01.md), [F-SEC-01-P0-01](../tasks/F-SEC-01.md) | Redact secrets before any trace/audit/HITL sink and bound retention. |
| RDM-04 | [F-HITL-01-P0-01](../tasks/F-HITL-01.md) | Execute exactly the approved effective Tool invocation. |
| RDM-05 | [F-OPS-01-P0-03](../tasks/F-OPS-01.md) | Durable cron occurrence fires once and reaches typed terminal. |
| RDM-06 | [A-CFG-01-P0-01](../tasks/A-CFG-01.md), [A-PROJ-01-P0-01](../tasks/A-PROJ-01.md) | Resolve and fence one canonical workspace root for config/drafts/writes. |
| RDM-07 | [A-STATE-01-P0-01](../tasks/A-STATE-01.md), [A-MEM-01-P0-01](../tasks/A-MEM-01.md) | CAS/generation-fenced conversation and memory writes across primary/pool Agents. |
| RDM-08 | [A-DOM-01-P0-01, A-DOM-01-P0-02](../tasks/A-DOM-01.md) | Immutable artifact versions and merge-not-empty domain enrichment. |
| RDM-09 | [A-EVO-01-P0-01](../tasks/A-EVO-01.md) | Transactional rule promotion with rollback and truthful receipt. |
| RDM-10 | [A-OBS-01-P0-01](../tasks/A-OBS-01.md), [A-SRF-02-P0-01](../tasks/A-SRF-02.md), [X-AUT-01-P0-01](../tasks/X-AUT-01.md) | Redact outbound webhook and terminal persistence before enqueue/write. |
| RDM-02/RDM-07 | [X-STA-01-P0-01](../tasks/X-STA-01.md) | Cross-store corrupt state remains quarantined through EKO publication. |

## Implementation Milestones

### P0: Contain Data Loss, Corruption, Secret Exposure And Broken Automation

| ID | Owner / scope | Evidence | Depends on | Deliverable and same-task deletion/switch | Measurable regression acceptance | Research |
|---|---|---|---|---|---|---|
| RDM-00 | framework / M | P0 ledger plus [F-SEC-01-P1-08](../tasks/F-SEC-01.md) | none | One reusable safe-ID/safe-path validation boundary for store, eval and file mutation inputs; switch affected callers and delete duplicate lexical-only checks. | Traversal, reserved-name, symlink-parent, Chinese/emoji and empty-ID tables fail before mutation; valid nested paths round-trip. | MR-REC |
| RDM-01 | framework / M | P0 ledger plus [F-EXT-02-P1-01, F-EXT-02-P1-02](../tasks/F-EXT-02.md) | RDM-00 | Make checkpoint/worktree enter/rollback/exit transactional and cancellation-owned; remove force-delete-on-uncertain-state and check-then-truncate paths. | Dirty unrelated edits survive; injected Git/write/cleanup failures return typed partial state; cancel joins child processes; no success after cleanup failure. | MR-TRY, MR-REC |
| RDM-02 | framework / M | P0 ledger, [X-STA-01-P1-02](../tasks/X-STA-01.md) | RDM-00 | One atomic file-generation/corruption primitive with checksum, temp-sync-rename and quarantine; switch FileStore/checkpoint/JSONL-tail users, deleting decode-to-empty fallbacks. | Concurrent handles lose no acknowledged write; torn tail preserves valid prefix; corrupt primary is quarantined and never overwritten; restart yields one generation. | MR-REC |
| RDM-03 | framework / S | P0 ledger, [F-OPS-01-P1-04, F-OPS-01-P1-05, F-OPS-01-P1-06](../tasks/F-OPS-01.md) | none | Redact structured sensitive fields before trace/audit/HITL serialization and add retention/flush ownership; delete raw URL/token/content serialization branches. | Token, query credential and Tool secret fixtures are absent from disk/logs; bounded rotation works; shutdown awaits flush and reports failure. | MR-POL |
| RDM-04 | framework / S | P0 ledger, [X-TOL-01-P1-01](../tasks/X-TOL-01.md) | none | Carry requested and approved effective Tool invocation as typed facts; execute only effective arguments; delete Approved projections that discard modifications. | ModifiedArgs executes byte-for-byte effective input; audit/event persists requested and effective values; reject/timeout never invokes Tool. | MR-POL, MR-EVT |
| RDM-05 | framework / S | P0 ledger, [F-OPS-01-P1-01, F-OPS-01-P1-02](../tasks/F-OPS-01.md) | RDM-02 | Persist occurrence identity and claim one due cron fire; delete future-only tick calculation and serialized global fire loop. | Deterministic clock proves missed/due/duplicate/restart cases fire exactly once with one terminal; one slow schedule does not block another. | MR-TRY, MR-REC |
| RDM-06 | application / M | P0 ledger, [A-CFG-01-P1-02, A-CFG-01-P1-05](../tasks/A-CFG-01.md) | RDM-00 | Resolve one `WorkspaceGeneration` before any config/draft/write; switch GUI/CLI/TUI callers and delete relative-root/current-directory split writers. | Two-workspace same-relative-path fixture never crosses roots; failed persistence reports failure; switch completion exposes one root/generation everywhere. | MR-REC |
| RDM-07 | application / M | P0 ledger, [X-MEM-01-P1-01, X-MEM-01-P1-02](../tasks/X-MEM-01.md) | RDM-02, RDM-06 | One CAS/generation commit for conversation, memory and primary/current/future pool binding; delete independently snapshotted FileStore and stale-prefix overwrite paths. | Concurrent autosave/finalize keeps complete transcript; switch/restart gives all Agents one generation; partial promotion reports reconciliation required. | MR-EVT, MR-REC |
| RDM-08 | application / M | P0 ledger, [A-OUT-01-P1-02, A-OUT-01-P1-05](../tasks/A-OUT-01.md) | RDM-02, RDM-06 | Content-address/version artifacts and merge domain enrichment without empty-field replacement; delete overwrite-in-place artifact paths. | Old run locators retain bytes after rerun; partial refresh preserves prior fields; digest/revision/renderer lineage survives export and restart. | MR-EVT, MR-REC |
| RDM-09 | application / M | P0 ledger, [A-EVO-01-P1-03, A-EVO-01-P1-04, A-EVO-01-P1-05](../tasks/A-EVO-01.md) | RDM-02, RDM-06 | Transactional rule/evidence/Skill mutation receipt with compensation and generation refresh; delete mutate-before-log and success-on-partial paths. | Read/corrupt/append/reload fault table leaves old rules usable or one explicit partial receipt; undo is restart-safe; all surfaces can perform reviewed actions. | MR-PLG, MR-REC |
| RDM-10 | application / S | P0 ledger, [A-OBS-01-P1-02, A-OBS-01-P1-03](../tasks/A-OBS-01.md) | RDM-03 | Apply framework redaction to webhook/terminal sinks before queue/persistence, bound payload/error length, and delete raw preview/full-endpoint logging. | Secret corpus absent from app log, terminal store and webhook payload; queue overflow/delivery failure has typed bounded terminal; no credential URL in errors. | MR-POL, MR-EVT |

### P1: Establish Generic Runtime Authorities

| ID | Owner / scope | Evidence | Depends on | Deliverable and same-task deletion/switch | Measurable regression acceptance | Research |
|---|---|---|---|---|---|---|
| RDM-11 | framework / L | [F-CORE-01-P1-03](../tasks/F-CORE-01.md), [F-RCT-02-P1-01, F-RCT-02-P1-02, F-RCT-02-P1-03](../tasks/F-RCT-02.md), [F-RCT-03-P1-01, F-RCT-03-P1-03, F-RCT-03-P1-05](../tasks/F-RCT-03.md) | RDM-03 | One typed `TurnOutcome` and terminal commit shared by stream/non-stream/direct answer; delete branch-local success publication and error-to-empty fallbacks. | Success/failed/cancelled/interrupted/unknown each emit and persist exactly one matching terminal; stream closure and partial answer cannot become success. | MR-EVT |
| RDM-12 | framework / L | [F-RCT-03-P1-02, F-RCT-03-P1-04](../tasks/F-RCT-03.md), [F-REL-01-P1-02](../tasks/F-REL-01.md), [F-RCT-04-P1-03, F-RCT-04-P1-05](../tasks/F-RCT-04.md) | RDM-11 | One cancellation/deadline/join scope owns provider, Tool batch, Subagent and transport children; remove detached/abandoned task paths. | Disconnect, timeout and explicit cancel stop new retries, join every child within a bound and produce one terminal with no post-terminal event. | MR-EVT, MR-SUB, MR-TRY |
| RDM-13 | framework / M | [F-RCT-04-P1-01, F-RCT-04-P1-02, F-RCT-04-P1-04](../tasks/F-RCT-04.md), [X-TOL-01-P1-02, X-TOL-01-P1-04](../tasks/X-TOL-01.md) | RDM-04, RDM-11, RDM-12 | Canonical Tool call/result pair with unique call ID, effective invocation, typed terminal and complete artifact descriptor; delete partial-batch and parent-status inference. | Duplicate/empty IDs reject before side effects; ordered serial/concurrent batch has one result per call; full artifact survives streaming, timeout and replay. | MR-EVT, MR-TRY |
| RDM-14 | framework / M | [F-TSK-01-P1-01, F-TSK-01-P1-02](../tasks/F-TSK-01.md), [F-TSK-02-P1-01, F-TSK-02-P1-03, F-TSK-02-P1-04](../tasks/F-TSK-02.md), [X-TSK-01-P1-02](../tasks/X-TSK-01.md) | RDM-02 | Make one revision service/validator/DAG analyzer authoritative for state transitions, cycles, blocking and ready frontier; remove one-wave readiness and rich-record graph authority. | Table tests cover skip/fail transitive propagation, cycle/no-cycle, pause/resume/restart and stale revision; exactly one ready frontier is callable. | MR-PLAN, MR-REC |
| RDM-15 | framework / L | [F-TSK-03-P1-01, F-TSK-03-P1-02, F-TSK-03-P1-03, F-TSK-03-P1-04](../tasks/F-TSK-03.md), [X-TSK-01-P1-03](../tasks/X-TSK-01.md) | RDM-12, RDM-14 | Claim epoch + physical attempt identity owns retry/cancel/settlement and sibling safe points; delete multiple attempts per claim and controller-local settlement. | ABA/restart/stale completion rejected; each physical retry has a new attempt; forced abort settles; completed siblings remain committed after wave failure. | MR-TRY, MR-REC |
| RDM-16 | framework / L | [F-SUB-01-P1-01, F-SUB-01-P1-02, F-SUB-01-P1-03, F-SUB-01-P1-05, F-SUB-01-P1-06](../tasks/F-SUB-01.md), [F-SUB-02-P1-01, F-SUB-02-P1-02, F-SUB-02-P1-03, F-SUB-02-P1-05, F-SUB-02-P1-07](../tasks/F-SUB-02.md), [X-BND-01-P1-02](../tasks/X-BND-01.md) | RDM-12, RDM-13, RDM-15 | One Subagent catalog/invocation/result/checkpoint lifecycle; fold Team/Handoff behavior into it and delete separate registries, schedulers and result classifiers. | Foreground/background/team dispatch carries role/source/generation/parent/attempt; cancel and timeout settle; artifacts prove provenance; restart resumes or rejects stale topology. | MR-SUB, MR-TRY |
| RDM-17 | framework / M | [F-PLG-01-P1-01, F-PLG-01-P1-02, F-PLG-01-P1-03, F-PLG-01-P1-04, F-PLG-01-P1-06, F-PLG-01-P1-07](../tasks/F-PLG-01.md) | RDM-02, RDM-16 | Source-scoped extension transaction and lifecycle receipt across all manifest components; delete scope-blind replacement and targeted helpers that suppress ownership/errors. | Duplicate scope behavior deterministic; corrupt registry fails closed; partial wire/uninstall returns receipt and compensates; every component unloads. | MR-PLG, MR-REC |
| RDM-18 | framework / M | [F-INT-01-P1-01, F-INT-01-P1-02, F-INT-01-P1-03, F-INT-01-P1-04, F-INT-01-P1-05, F-INT-01-P1-06, F-INT-01-P1-08, F-INT-01-P1-09, F-INT-01-P1-10](../tasks/F-INT-01.md) | RDM-12, RDM-13 | Apply typed ownership/timeout/cancel/result contracts to MCP/LSP/A2A adapters; delete advertised-but-unread response paths and success-on-incomplete projections. | Transport fault/Unicode/frame-bound/cancel/restart tables return typed outcomes, release processes/streams and preserve rich Tool results. | MR-EVT, MR-TRY |

### P1: Converge EKO State And Thin Adapters

| ID | Owner / scope | Evidence | Depends on | Deliverable and same-task deletion/switch | Measurable regression acceptance | Research |
|---|---|---|---|---|---|---|
| RDM-19 | application / M | [A-CFG-01-P1-01, A-CFG-01-P1-03, A-CFG-01-P1-04](../tasks/A-CFG-01.md), [X-MEM-01-P1-01](../tasks/X-MEM-01.md) | RDM-06, RDM-07 | One validated config/workspace generation with last-known-good and declared live/restart fields; delete default-provider fallback on explicit invalid input and detached reload mutation. | Invalid config never silently starts another provider; successful switch publishes one root/config/model generation; failed reload retains old live state. | MR-REC |
| RDM-20 | application / L | [A-STATE-01-P1-02, A-STATE-01-P1-03, A-STATE-01-P1-04, A-STATE-01-P1-05](../tasks/A-STATE-01.md), [X-EVT-01-P1-04, X-EVT-01-P1-05](../tasks/X-EVT-01.md), [X-STA-01-P1-03, X-STA-01-P1-05](../tasks/X-STA-01.md) | RDM-07, RDM-11, RDM-12 | One durable conversation-turn envelope log, replay cursor, active-turn owner and tombstoned deletion cascade; delete display-only edit/regenerate and anonymous pooled reuse. | Edit/regenerate rewinds both view and Agent context; restore failure is non-resumable; deletion cancels/joins and fences late writes/uploads/Browser state; replay is idempotent. | MR-EVT, MR-REC |
| RDM-21 | application / L | [A-TSK-01-P1-01, A-TSK-01-P1-02, A-TSK-01-P1-03](../tasks/A-TSK-01.md), [A-TSK-04-P1-01, A-TSK-04-P1-02, A-TSK-04-P1-03](../tasks/A-TSK-04.md), [X-TSK-01-P1-01, X-TSK-01-P1-04](../tasks/X-TSK-01.md) | RDM-02, RDM-14, RDM-15 | Make TaskRuntime one crash-atomic event/projection aggregate; bootstrap effects enter the revision transaction; delete decode-empty plan and non-atomic projection publication. | Truncated tail keeps valid events; stale plan/claim cannot write; restart recovers or settles orphan; event and projection expose one revision/attempt terminal. | MR-PLAN, MR-TRY, MR-REC |
| RDM-22 | adapter / M | [A-TSK-03-P1-01, A-TSK-03-P1-02, A-TSK-03-P1-03](../tasks/A-TSK-03.md), [X-BND-01-P1-03, X-BND-01-P1-04](../tasks/X-BND-01.md), [X-TSK-01-P2-05](../tasks/X-TSK-01.md) | RDM-15, RDM-16, RDM-21 | Reduce EKO Task executor to DomainProfile/resource/worktree/review policy plus typed framework dispatch; delete EKO retry, ready-frontier, dependency polling and settlement loops. | Source search finds one generic scheduler/retry/settlement authority; adapter round-trip is field-complete; pause/cancel/restart fault table has one terminal per attempt. | MR-PLAN, MR-SUB, MR-TRY |
| RDM-23 | application / M | [A-TSK-05-P1-01, A-TSK-05-P1-02, A-TSK-05-P1-03, A-TSK-05-P1-04](../tasks/A-TSK-05.md), [A-TSK-06-P1-01, A-TSK-06-P1-02, A-TSK-06-P1-03](../tasks/A-TSK-06.md), [X-STA-01-P1-04](../tasks/X-STA-01.md) | RDM-08, RDM-13, RDM-21, RDM-22 | Claim-bound worktree/file/artifact lineage and bounded downstream context/review feedback; delete branch-at-settlement and basename-only artifact matching. | Attempt can integrate only its claimed base/branch; retry receives feedback; artifact locator/digest/revision reaches parent and survives restart; cleanup has durable owner. | MR-SUB, MR-TRY, MR-REC |
| RDM-24 | application / M | [A-PLG-01-P1-01, A-PLG-01-P1-02, A-PLG-01-P1-03, A-PLG-01-P1-04](../tasks/A-PLG-01.md), [A-SUB-01-P1-01, A-SUB-01-P1-02, A-SUB-01-P1-03](../tasks/A-SUB-01.md), [X-PLG-01-P1-01, X-PLG-01-P1-02](../tasks/X-PLG-01.md) | RDM-17, RDM-19 | One plugin/Skill/Subagent/prompt/router generation across primary/current/future pool Agents with truthful compensation; delete additive refresh and frozen bootstrap catalogs. | Install/disable/reload/rollback/restart matrix reports one generation on every Agent; failed save/wire never reports durable success; declared roles are executable. | MR-PLG, MR-SUB, MR-REC |
| RDM-25 | adapter / M | [A-CHAT-01-P1-01, A-CHAT-01-P1-02, A-CHAT-01-P1-03, A-CHAT-01-P1-04, A-CHAT-01-P1-05](../tasks/A-CHAT-01.md), [X-EVT-01-P1-01, X-EVT-01-P1-02, X-EVT-01-P1-03](../tasks/X-EVT-01.md) | RDM-11, RDM-12, RDM-20 | `drive_chat` and sinks carry full versioned envelope and one terminal; delete transport-success inference, pre-stream silent returns and sink-specific final payload loss. | GUI/TUI/CLI/channel fixture sees identical identity/order/FinalAnswer/typed terminal; stream error/cancel/disconnect never emits complete; persistence failure is visible. | MR-EVT |
| RDM-26 | adapter / M | [A-TOOL-01-P1-01, A-TOOL-01-P1-02, A-TOOL-01-P1-03](../tasks/A-TOOL-01.md), [X-TOL-01-P1-01, X-TOL-01-P1-02, X-TOL-01-P1-04](../tasks/X-TOL-01.md) | RDM-13, RDM-16, RDM-23, RDM-25 | Preserve requested/effective invocation, Subagent tool policy, rich result and complete artifact through EKO; delete PendingToolCompletion/raw-path authority and read-only writer projection. | Selected Subagent policy governs; all surfaces distinguish failure/timeout/cancel/interrupted; verified complete artifact opens/copies without parent-status inference. | MR-EVT, MR-SUB |
| RDM-27 | application / M | [A-INP-01-P1-01, A-INP-01-P1-02, A-INP-01-P1-03, A-INP-01-P1-04, A-INP-01-P1-05](../tasks/A-INP-01.md), [A-OUT-01-P1-01, A-OUT-01-P1-03, A-OUT-01-P1-04, A-OUT-01-P1-06](../tasks/A-OUT-01.md) | RDM-08, RDM-20, RDM-25 | One PreparedUserTurn/admission transaction owns attachment identity, cleanup and complete export/delivery artifact; delete partial-batch acceptance and pre-admission orphan writes. | Any attachment failure rejects the turn with typed details; content identity survives repeated reads; cancel/deletion cleans ownership; Unicode/full export parity holds. | MR-EVT, MR-REC |
| RDM-28 | application / M | [A-OBS-01-P1-02, A-OBS-01-P1-03, A-OBS-01-P1-04, A-OBS-01-P1-05](../tasks/A-OBS-01.md), [Q-PERF-01-P1-01](../tasks/Q-PERF-01.md) | RDM-10, RDM-20, RDM-21, RDM-25 | Durable bounded outbox/cursor for hooks/webhooks outside TaskRuntime locks; diagnostics bind to historical root/config facts; delete detached unbounded delivery and under-lock waits. | Slow/full sink cannot block authoritative writes; delivery retries are bounded/observable; correlation and terminal agree; historical diagnostics do not use current prompt state. | MR-EVT, MR-TRY, MR-REC |

### P1: Deliver Surface Parity From One Application Contract

| ID | Owner / scope | Evidence | Depends on | Deliverable and same-task deletion/switch | Measurable regression acceptance | Research |
|---|---|---|---|---|---|---|
| RDM-29 | application / L | [X-SRF-01-P1-01, X-SRF-01-P1-03, X-SRF-01-P1-04](../tasks/X-SRF-01.md), [A-SRF-02-P2-05](../tasks/A-SRF-02.md) | RDM-19, RDM-20, RDM-22, RDM-24, RDM-25, RDM-26, RDM-27 | One app-core capability manifest, service composition and active-turn registry; move workflow/diff business authority out of Tauri and delete surface-local registries/startup lists. | Machine matrix proves supported/available reason for every capability; one service instance/generation and active handle per scope; removing a binding fails a fixture. | MR-EVT, MR-SUB |
| RDM-30 | adapter / M | [A-SRF-01-P1-01, A-SRF-01-P1-02, A-SRF-01-P1-03](../tasks/A-SRF-01.md), [A-CHAT-01-P1-03, A-CHAT-01-P1-04](../tasks/A-CHAT-01.md), [A-INP-01-P1-03](../tasks/A-INP-01.md) | RDM-29 | Bind TUI to all shared capabilities, PreparedUserTurn, Task/Subagent/HITL/memory/artifact/export and active cancel; delete fixed-mode or bypass routes. | TUI completes every parity scenario with same canonical facts/artifacts as app-core; `/steer` retains attachments; cancel waits for terminal. | MR-EVT, MR-SUB, MR-POL |
| RDM-31 | adapter / M | [A-SRF-02-P1-02, A-SRF-02-P1-03, A-SRF-02-P1-04](../tasks/A-SRF-02.md), [A-FE-01-P1-01](../tasks/A-FE-01.md), [X-EVT-01-P1-01, X-EVT-01-P1-03](../tasks/X-EVT-01.md) | RDM-29 | Generate/register one GUI command/event contract from app-core and preserve typed terminals/Subagent facts; delete duplicate Tauri setup and handwritten lossy unions. | GUI-only command registration compiles; unknown required event fails explicitly; close/EOF cancels and joins; generated TS matches serde optionality and terminal variants. | MR-EVT, MR-SUB |
| RDM-32 | adapter / M | [A-SRF-03-P1-01, A-SRF-03-P1-02, A-SRF-03-P1-03](../tasks/A-SRF-03.md), [A-CHAT-01-P1-03, A-CHAT-01-P1-04](../tasks/A-CHAT-01.md) | RDM-29 | Bind interactive CLI to the same service/snapshot/input/Task/Subagent/HITL/artifact/export and active cancel; delete mode-local lifecycle ownership. | CLI parity fixture returns same typed terminal/final artifact and can cancel/recover; trigger and text rendering may differ, facts may not. | MR-EVT, MR-SUB, MR-POL |
| RDM-33 | adapter / M | [A-SRF-04-P1-01, A-SRF-04-P1-03, A-SRF-04-P1-04, A-SRF-04-P1-06](../tasks/A-SRF-04.md), [X-SRF-01-P1-02](../tasks/X-SRF-01.md) | RDM-29 | Add typed noninteractive CLI plus channel adapters with stable routing identity, attachment/artifact delivery and cancel handle; delete text-only/anonymous success projections. | JSONL contract preserves envelope/terminal/artifact; group chats isolate by route identity; disconnect cancels; all complete content is retrievable. | MR-EVT, MR-SUB |
| RDM-34 | adapter / M | [A-SRF-04-P1-02, A-SRF-04-P1-05](../tasks/A-SRF-04.md), [F-OPS-01-P1-02](../tasks/F-OPS-01.md), [X-SRF-01-P1-01, X-SRF-01-P1-03](../tasks/X-SRF-01.md) | RDM-05, RDM-29, RDM-33 | Bind channel-only, cron and background triggers to the same service and lifecycle supervisor; delete detached service tasks and inert automation options. | Start/shutdown joins all services; cron/background carry canonical identity/terminal/artifact; missing interaction returns typed `interaction_required`, not silent omission. | MR-EVT, MR-SUB, MR-POL |

### P2: Delete Displaced Authorities, Bound Resources And Make Contracts Executable

| ID | Owner / scope | Evidence | Depends on | Deliverable and same-task deletion/switch | Measurable regression acceptance | Research |
|---|---|---|---|---|---|---|
| RDM-35 | framework / S audit-removal gate | [S-FW deletion ledger](framework-review.md#required-authority-consolidation-and-deletion), [X-BND-01-P1-01, X-BND-01-P1-02, X-BND-01-P2-05](../tasks/X-BND-01.md) | RDM-36 plus completed framework caller cutovers RDM-13 through RDM-18 | Verify that every earlier owner milestone already removed its displaced semantic path; delete only orphan imports/docs/tests. Any live semantic path reopens its owning milestone rather than expanding RDM-35. | Whole-repository definition/export/registration/reachability inventory has one live owner; public alternatives remain only after reasonable-consumer review. | MR-EVT, MR-SUB, MR-PLG |
| RDM-36 | application / S audit-removal gate | [S-APP risk clusters](application-review.md#main-risk-clusters), [X-BND-01-P1-03, X-BND-01-P1-04, X-BND-01-P2-05](../tasks/X-BND-01.md), [X-AUT-01-P2-03](../tasks/X-AUT-01.md) | RDM-19 through RDM-34 | Verify that each cutover already deleted its scheduler/inference/store/registry/writer; delete only orphan imports/docs/tests. Any registered or production bypass reopens its owner. | One owner per contract, zero registered bypass, lossless adapter round-trip and no dormant authority in AppState/Tauri/surfaces. | MR-PLAN, MR-EVT, MR-SUB |
| RDM-37 | application / M | [A-FE-03-P1-01, A-FE-03-P1-04, A-FE-03-P2-02, A-FE-03-P2-03](../tasks/A-FE-03.md), [Q-PERF-01-P2-01, Q-PERF-01-P2-02](../tasks/Q-PERF-01.md), [F-OPS-01-P1-04](../tasks/F-OPS-01.md) | RDM-28, RDM-31, RDM-36 | Normalize frontend selectors/reducers, lazy artifact/full-output load, bounded Task/log/cache retention and cancellable process work; remove repeated full-history scans and duplicate polling owner. | Profiler fixture has bounded rerenders/work per event; 10k-event/task fixture is sub-quadratic; logs/cache/artifacts honor retention; semantic focus/modal checks pass. | MR-EVT, MR-REC |
| RDM-38 | framework / S | [Q-FW-01-P1-01](../tasks/Q-FW-01.md), [Q-TST-01-P1-02](../tasks/Q-TST-01.md) | RDM-11 through RDM-18 | Make all-target/all-feature and panic-lint gates mandatory; activate the deterministic ReAct terminal regression; delete the weaker duplicate CI path. | Removing one required target or reintroducing the known-red behavior fails mandatory CI. | MR-EVT |
| RDM-39 | framework / S | [Q-FW-01-P2-02](../tasks/Q-FW-01.md), [Q-DOC-01-P2-02](../tasks/Q-DOC-01.md) | RDM-35 | Make feature/example/doctest and public command documentation one executable manifest; remove no-op claims. | Every documented feature/example/package maps to an executable matrix entry and resolves at HEAD. | MR-REC |
| RDM-40 | application / S | [Q-GUI-01-P2-01, Q-GUI-01-P3-02](../tasks/Q-GUI-01.md), [Q-TST-01-P2-03](../tasks/Q-TST-01.md) | RDM-31, RDM-36 | Add EKO Rust/GUI-only/native-target CI topology and one capability authority; delete the duplicate capability file. | GUI-only and supported-platform branches are mandatory lanes; capability identities are unique. | MR-EVT |
| RDM-41 | application frontend / S | [Q-WEB-01-P2-01](../tasks/Q-WEB-01.md), [Q-TST-01-P1-01](../tasks/Q-TST-01.md) | RDM-31, RDM-37 | Pin a Node/npm line compatible with the lock and add a mounted frontend transport harness; delete store-only substitute claims. | Declared Node floor installs/runs locked Vitest; mounted app observes connect/event/error/cancel/remount lifecycle. | MR-EVT |
| RDM-42 | framework / S | [Q-DEP-01-P1-01, Q-DEP-01-P2-02, Q-DEP-01-P3-03](../tasks/Q-DEP-01.md), [Q-STA-01-P3-04](../tasks/Q-STA-01.md) | RDM-35 | Align JWT algorithm/key construction, make dependency/license policy executable and remove duplicate dependency/YAML declarations. | RS256/HS fixtures use matching key families; advisory/license policy has deterministic allow/deny output; manifests have one declaration. | MR-POL |
| RDM-43 | owning source modules / S | [Q-STA-01-P1-01, Q-STA-01-P1-02, Q-STA-01-P1-03](../tasks/Q-STA-01.md) | relevant source-owner milestones | Replace external-text byte slicing, enforce environment-mutation startup preconditions and check token multiplications. | Unicode/malformed date/percent fixtures never panic; concurrent startup rejects unsafe env mutation; extreme token values return typed errors. | MR-REC |
| RDM-44 | application docs / S | [Q-DOC-01-P2-01, Q-DOC-01-P2-03, Q-DOC-01-P2-04, Q-DOC-01-P3-01](../tasks/Q-DOC-01.md) | RDM-30 through RDM-36 | Update EKO commands, roots, architecture/status and links to current product facts; remove SQLite guidance and nonexistent CLI claims. | Every internal link/path/command resolves; surface status matches the capability manifest; EKO docs contain no SQLite setup. | MR-POL |

### P3 And Release: Execute Frozen Gates As Independent Packages

| ID | Owner / scope | Evidence | Depends on | Deliverable and same-task deletion/switch | Measurable regression acceptance | Research |
|---|---|---|---|---|---|---|
| RDM-45 | quality framework submission / S | [Q-FW-01](../tasks/Q-FW-01.md) | RDM-38, RDM-39, RDM-42, RDM-43 | Run each framework submission command as its own immutable attempt at one clean framework commit. | fmt, both Clippy modes, all-target/all-feature tests and no-default check exit 0 with zero warnings. | MR-REC |
| RDM-46 | quality framework public matrix / M | [Q-FW-02](../tasks/Q-FW-02.md) | RDM-39, RDM-45 | Run independent features, all example groups and package doctests, one report per command. | Every catalog feature/example/doctest command exits 0; historical results remain historical. | MR-REC |
| RDM-47 | quality EKO Rust submission / S | [Q-CLI-01](../tasks/Q-CLI-01.md) | RDM-40, RDM-42, RDM-43, RDM-44 | Run each EKO Rust submission command at one clean CLI/framework pair. | fmt, both Clippy modes, all-feature tests and app-core no-default exit 0; dependency tree does not enable SQLite. | MR-POL, MR-REC |
| RDM-48 | quality GUI / S | [Q-GUI-01](../tasks/Q-GUI-01.md) | RDM-40, RDM-47 | Run GUI-only check/tests as separate immutable attempts. | `gui && !tui` binary and GUI tests exit 0; one capability identity is generated. | MR-EVT |
| RDM-49 | quality frontend / S | [Q-WEB-01](../tasks/Q-WEB-01.md) | RDM-41 | Run Prettier, mounted Vitest and production build separately. | All three exit 0 on the declared Node/npm line and mounted transport cases execute. | MR-EVT |
| RDM-50 | quality ReAct/Tool faults / M | [Q-FLT-01](../tasks/Q-FLT-01.md) | RDM-11 through RDM-13, RDM-18, RDM-45 | Execute the ten deterministic ReAct/Tool fault families with scripted providers/tools/clocks/stores. | Every case has one typed terminal, bounded cancel/cleanup and no replay of committed effects. | MR-EVT, MR-TRY |
| RDM-51 | quality Task/Subagent faults / M | [Q-FLT-02](../tasks/Q-FLT-02.md) | RDM-14 through RDM-17, RDM-21 through RDM-24, RDM-45, RDM-47 | Execute the ten deterministic Task/Subagent fault families. | Every claim/attempt settles once; stale results reject; restart/cancel preserves committed facts and artifact lineage. | MR-SUB, MR-TRY, MR-REC |
| RDM-52 | quality surface parity / M | [Q-E2E-01](../tasks/Q-E2E-01.md), [X-SRF-01-P2-05](../tasks/X-SRF-01.md) | RDM-30 through RDM-34, RDM-48, RDM-49, RDM-50, RDM-51 | Execute all 23 scenario/surface pairs with one immutable report per pair. | GUI/TUI/CLI/channel/cron/background preserve identity, effective input, order, terminal, complete artifact and recovery; trigger/render differences alone are allowed. | MR-EVT, MR-SUB, MR-POL, MR-REC |

## Dependency DAG

```text
P0 containment
  RDM-00 -> RDM-01
  RDM-00 -> RDM-02 -> RDM-05
  RDM-03 -> RDM-10
  RDM-04
  RDM-00 -> RDM-06 -> RDM-07 -> RDM-08
                            |      -> RDM-09
                            -> cross-store containment

Generic framework spine
  RDM-03 -> RDM-11 -> RDM-12 -> RDM-13
  RDM-02 -> RDM-14 -> RDM-15 -> RDM-16 -> RDM-17
                    |          |          -> RDM-18
                    +----------+

EKO authority convergence
  RDM-06 + RDM-07 -> RDM-19
  RDM-07 + RDM-11 + RDM-12 -> RDM-20
  RDM-02 + RDM-14 + RDM-15 -> RDM-21 -> RDM-22
  RDM-08 + RDM-13 + RDM-21 + RDM-22 -> RDM-23
  RDM-17 + RDM-19 -> RDM-24
  RDM-11 + RDM-12 + RDM-20 -> RDM-25 -> RDM-26
  RDM-08 + RDM-20 + RDM-25 -> RDM-27
  RDM-10 + RDM-20 + RDM-21 + RDM-25 -> RDM-28

Surface cutover and deletion
  RDM-19/20/22/24/25/26/27 -> RDM-29
  RDM-29 -> RDM-30, RDM-31, RDM-32, RDM-33
  RDM-05 + RDM-29 + RDM-33 -> RDM-34
  RDM-19..34 -> RDM-36 -> RDM-35
  RDM-28 + RDM-31 + RDM-36 -> RDM-37

Quality contract fixes
  RDM-11..18 -> RDM-38
  RDM-35 -> RDM-39, RDM-42
  RDM-31 + RDM-36 -> RDM-40 -> RDM-48
  RDM-31 + RDM-37 -> RDM-41 -> RDM-49
  source-owner milestones -> RDM-43
  RDM-30..36 -> RDM-44

Executable packages
  RDM-38/39/42/43 -> RDM-45 -> RDM-46
  RDM-40/42/43/44 -> RDM-47 -> RDM-48
  RDM-41 -> RDM-49
  RDM-11..18 + RDM-45 -> RDM-50
  RDM-14..17 + RDM-21..24 + RDM-45/47 -> RDM-51
  RDM-30..34 + RDM-48..51 -> RDM-52
```

No later row may be used to defer a deletion named by an earlier cutover. For
example, RDM-22 is incomplete if EKO's generic retry/settlement loop remains;
RDM-29 is incomplete if Tauri still owns workflow business logic; RDM-35/36 are
zero-residual audit gates, not permission to keep duplicates until then. RDM-35
depends on RDM-36 so framework removal cannot precede all application cutovers.

## Cross-Repository Merge Order

1. Merge independent P0 containment inside its owning repository. RDM-00..05
   land in `echo-agent`; RDM-06..10 land in `echo-agent-cli` only after any
   framework primitive they consume is on framework main.
2. Merge each generic framework contract RDM-11..18 to `echo-agent` first.
   An additive contract is acceptable only when its named EKO cutover is queued;
   it is not a permanent parallel API.
3. Merge EKO aggregate and adapter cutovers RDM-19..29 to `echo-agent-cli`, after
   merging the branch with current CLI main and resolving against framework main.
4. Merge surface adapters RDM-30..34 independently once RDM-29 is on CLI main.
   Rust producer, generated TypeScript wire and reducer changes merge together.
5. Delete displaced application paths in each cutover and run the zero-residual
   RDM-36 audit. Only then run RDM-35 and merge any remaining framework orphan
   cleanup after reasonable external-use review.
6. Merge bounded quality-contract fixes RDM-37..44 in their owning repository.
   Then execute independent packages RDM-45..52 against one pinned clean pair.

For every CLI merge, dependency paths remain relative (`../echo-agent` or the
appropriate workspace-relative parent). Framework commits precede dependent CLI
commits. No merge may contain a worktree absolute path, enable `echo-state/sqlite`
in CLI, or restore a removed compatibility authority.

## Deletion Ledger

| Canonical authority after cutover | Required deletion target | Removal milestone |
|---|---|---|
| One typed ReAct terminal/event history | Branch-specific success/persistence, error-to-empty fallback, dead phase processor and PostToolBatch | RDM-11, verified RDM-35 |
| One owned cancellation tree | Detached provider/Tool/Subagent/transport tasks and abandoned batch calls | RDM-12, verified RDM-35 |
| One Tool terminal/artifact contract | Partial batch authority, EKO PendingToolCompletion, raw-path paging, stream-over-final and parent-to-cancel inference | RDM-13/RDM-26, verified RDM-35/36 |
| One revisioned Task graph/claim executor | Framework rich-record graph/store and one-wave readiness; EKO retry/frontier/dependency polling/settlement loops | RDM-14/RDM-15/RDM-22, verified RDM-35/36 |
| One Subagent lifecycle | Separate Team/Handoff registries, schedulers, result classifiers and dormant context/output contracts | RDM-16, verified RDM-35 |
| One extension source generation | Scope-blind replacement, targeted helpers, additive refresh, primary-only mutation and frozen router | RDM-17/RDM-24, verified RDM-35/36 |
| One EKO workspace/conversation/Task aggregate | Decode-to-empty, split stores, dormant persistence/search, non-atomic projections and per-surface deletion | RDM-19/RDM-20/RDM-21, verified RDM-36 |
| One app-core capability/active-turn service | Surface startup/registries, anonymous cancellation, duplicate Tauri setup and GUI-only workflow/diff authority | RDM-29..34, verified RDM-36 |
| One revisioned writer and artifact reader | Registered non-revisioned writer, duplicate bridge, basename/raw-path artifact authority | RDM-23/RDM-26, verified RDM-36 |
| Executable public contract | Implicit mocks, no-op feature/example claims, prose parity, duplicate capability/dependency/YAML and dead links | RDM-38 through RDM-44; verified RDM-45 through RDM-52 |

Optional framework SQLite Stores, compressors, integrations, workflows and Tool
domains are not deletion targets merely because EKO does not select them.

## Priority Completion Gates

| Priority | Exit criteria |
|---|---|
| P0 | All 24 P0 IDs have merged fixes; destructive/corrupt/secret/cron fault fixtures pass; no lower-priority behavior depends on an unsafe writer or decode-to-empty store. |
| P1 | One typed terminal/cancel tree, one Task/claim executor, one Subagent lifecycle, one Tool/artifact contract and one EKO generation own all production callers. All six surface families preserve canonical facts. |
| P2 | Displaced authorities are deleted, adapters are thin/lossless, resources are bounded, CI/docs/dependency/test contracts are executable and current. |
| P3/release | Local cleanup is complete in touched paths and RDM-45 through RDM-52 create current passing evidence for every applicable command/fault/parity scenario at one pinned clean pair. |

## Validation Matrix

| ID | Claim | Status | Report |
|---|---|---|---|
| V01 Attempt 01 | Dependency completeness before S-X/S-QA acceptance | failed, preserved | [V01-01](../validations/S-RDM-01/V01-01.md) |
| V01 Attempt 02 | Final dependency and synthesis reconciliation | passed | [V01-02](../validations/S-RDM-01/V01-02.md) |
| V02 | Critical decisions backlink B-REF topic evidence | passed | [V02](../validations/S-RDM-01/V02-01.md) |
| V03 Attempt 01 | Exact 24-P0 coverage before exact-ID correction | passed claim overstated, preserved | [V03-01](../validations/S-RDM-01/V03-01.md) |
| V03 Attempt 02 | Exact 24-P0 coverage after correction | passed | [V03-02](../validations/S-RDM-01/V03-02.md) |
| V04 Attempt 01 | Milestone granularity before quality-package split | passed claim superseded, preserved | [V04-01](../validations/S-RDM-01/V04-01.md) |
| V04 Attempt 02 | Corrected fresh-task granularity and evidence fields | passed | [V04-02](../validations/S-RDM-01/V04-02.md) |
| V05 Attempt 01 | Initial DAG/deletion gate | passed claim superseded, preserved | [V05-01](../validations/S-RDM-01/V05-01.md) |
| V05 Attempt 02 | Corrected audit/deletion/merge DAG | passed | [V05-02](../validations/S-RDM-01/V05-02.md) |
| V06 | Q ledger and future executable program | passed | [V06](../validations/S-RDM-01/V06-01.md) |
| V99 | Delegated exact links, IDs, headers, isolation, terminology and status | passed | [V99](../validations/S-RDM-01/V99-01.md) |
| V30 | Atomic catalog and synthesis dependency acceptance | passed | [V30](../validations/S-RDM-01/V30-01.md) |
| V31 Attempts 01-02 | Initial primary P0/milestone extractors | inconclusive, preserved | [V31-01](../validations/S-RDM-01/V31-01.md), [V31-02](../validations/S-RDM-01/V31-02.md) |
| V31 Attempt 03 | Corrected exact P0 and milestone verification | passed | [V31-03](../validations/S-RDM-01/V31-03.md) |
| V32 | Links, terminology, product constraints and migration order | passed | [V32](../validations/S-RDM-01/V32-01.md) |
| V33 | Primary synthesis sampling and acceptance | passed | [V33](../validations/S-RDM-01/V33-01.md) |
| V34 | Final corpus count, links, statuses and entry point | passed | [V34](../validations/S-RDM-01/V34-01.md) |
| V35 Attempt 01 | Final audit found omitted synthesis-attempt links | failed, preserved | [V35-01](../validations/S-RDM-01/V35-01.md) |
| V35 Attempt 02 | Corrected synthesis attempt traceability | passed | [V35-02](../validations/S-RDM-01/V35-02.md) |
| V36 | Post-correction synthesis traceability and integrity | passed | [V36](../validations/S-RDM-01/V36-01.md) |

## Coverage And Residual Uncertainty

- This is report synthesis, not a source re-review. Atomic source evidence and
  priorities remain owned by the linked F/A/X/Q reports.
- Calendar estimates depend on staffing and are intentionally not invented.
  Relative scopes constrain task size and blast radius.
- Dynamic behavior is not proven by static synthesis. RDM-45 through RDM-52 remain mandatory;
  the 67 historical `not_run` reports stay immutable.
- Framework external dirty files and CLI `Cargo.lock` were excluded. Any source
  commit change makes affected evidence stale and requires the smallest atomic
  owner and roadmap row to be reopened.
- Public framework deletion requires framework-wide reasonable-consumer review;
  EKO non-use alone never proves dead code.

## Handoff

Start new implementation tasks at RDM-00 through RDM-10 only. Copy each row's
canonical evidence, dependency, layer, deletion and regression acceptance into
the task specification. Before implementation, perform the mandatory framework/
application/adapter placement decision and whole-repository duplicate/reachability
search. A task is not complete if its named displaced path is still a production
authority.

Primary review independently verified exact links, canonical IDs, P0 coverage,
B-REF references, prohibited terminology, CLI no-SQLite and cross-repository
merge/deletion constraints. Do not interpret roadmap completion as completion
of the eight Q executable tasks.
