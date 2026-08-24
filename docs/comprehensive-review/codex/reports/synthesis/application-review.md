# S-APP-01: Application review synthesis

> **Superseded for cross-review decisions:** this independent report remains
> evidence, but the authoritative three-review reconciliation is
> [../../../application-review.md](../../../application-review.md).

> Status: complete
> Reviewer: Codex review subagent
> Accepted by: Codex primary reviewer
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: framework external dirty source excluded through pinned committed objects; CLI external `Cargo.lock` excluded; only new Codex synthesis reports written

## Executive Conclusion

EKO already has the right major building blocks: one shared chat driver, prepared user turns, a file conversation store, a revisioned Task graph adapter, TaskRuntime file authority, generated Task/Subagent DTOs, Agent pooling, and reusable framework integrations. The application is not missing a wholesale architecture.

The dominant failure is **incomplete authority convergence**. Workspace/config/memory generations, conversation turn settlement, Task claim/revision/artifact identity, and surface projections each stop at different boundaries. Bootstrap, recovery, Tauri/TUI/channel state, detached tasks, and frontend reducers then reconstruct or override semantics. This produces data overwrite, false success, lost cancellation, stale Agent context, irrecoverable history, and real GUI/TUI/CLI/channel capability gaps.

The iteration strategy is therefore: protect destructive data first; make one workspace and one durable generation; make one terminal/revision identity cross every adapter; then complete surface parity. Do not build another store, Task model, event bus, or approval state machine. EKO remains file/in-memory based with no SQLite, and local interactive capabilities remain usable without online-service permission gates.

Static review is closed at the pinned commits, but application release readiness is not: Q-CLI, Q-GUI, and Q-WEB executable commands were not run. The synthesis task is complete while those command reports correctly remain `needs_evidence`.

## Evidence Coverage

- Application catalog: 29/29 A tasks complete, 422 immutable validation reports.
- Atomic application findings: 150 total: P0 9, P1 109, P2 32, P3 0.
- Cross-contract dependencies: 10/10 X reports complete; they validate the authority clusters below without replacing A IDs.
- Formal quality dependencies: [Q-CLI-01](../tasks/Q-CLI-01.md), [Q-GUI-01](../tasks/Q-GUI-01.md), and [Q-WEB-01](../tasks/Q-WEB-01.md) exist but remain `needs_evidence`.
- Finding IDs and exact titles have no collisions. Synthesis creates no new defect IDs.

Validation: [V01](../validations/S-APP-01/V01-01.md), [V02](../validations/S-APP-01/V02-01.md), [V05](../validations/S-APP-01/V05-01.md).

## Application Architecture

### Authorities to preserve

| Domain | Existing authority to extend | Positive evidence | Parallel semantics to remove |
|---|---|---|---|
| User turn | `PreparedUserTurn` plus shared `drive_chat` | A-INP-01/A-CHAT-01 definition and reachability | TUI steer bypass; surface-local terminal/cancel/queue rules |
| Conversation | file conversation record plus pooled Agent keyed by conversation | A-STATE-01 positive file behavior | GUI display edits without Agent rewind; stale autosave/finalize writers; dormant persistence/search objects |
| Task | framework revision service/validator through EKO adapter, one TaskRuntime event authority | A-TSK-01..06 and [X-TSK-01](../tasks/X-TSK-01.md) positive mapping | bootstrap side effects before commit; application retry/settlement loops; background cross-run scheduler |
| Subagent | one catalog/pool and revision+attempt execution identity | A-SUB-01/A-FE-02 | primary/pool/plugin generation split; dead nominal `SubagentRun`; current selector without revision |
| Artifacts | TaskRuntime artifact event/record with opaque full content | A-TSK-06/A-FE-02/A-OUT-01 | overwrite-in-place paths, basename matching, renderer-only identities, eager full-output copies |
| Config/workspace/memory | one resolved config source and one workspace generation distributed to stores/Agents/integrations | A-CFG-01/A-MEM-01/A-INT-01 | cwd rediscovery, per-Agent FileStore snapshots, bootstrap-bound LSP, primary-only refresh |
| Surface projection | typed canonical event/snapshot reduced by identity | A-FE-01/02, A-SRF-03 positive generated records | Tauri/channel loss, prose capability matrix, UI-local focus/terminal inference |

The framework/application split is stable: generic DAG/claim/cancel mechanics belong in framework owners, while EKO controls workspace policy, file/worktree ownership, reviewed artifacts, UI projections and concrete retention. [X-BND-01](../tasks/X-BND-01.md) and X-TSK-01 show where the current adapter still owns generic semantics; fixes must move those semantics to the existing framework service, not clone them inside EKO.

Validation: [V03](../validations/S-APP-01/V03-01.md).

## Priority 0: Stop Destructive And Sensitive Writes

All nine application P0 findings are release blockers:

| Risk family | Canonical findings | Immediate invariant |
|---|---|---|
| Cross-workspace overwrite | [A-CFG-01-P0-01](../tasks/A-CFG-01.md), [A-PROJ-01-P0-01](../tasks/A-PROJ-01.md) | every mutation is bound to an immutable resolved workspace/config identity plus revision precondition |
| Conversation/memory lost update | [A-STATE-01-P0-01](../tasks/A-STATE-01.md), [A-MEM-01-P0-01](../tasks/A-MEM-01.md) | one generation commits transcript/Agent/store projections or rejects stale writers |
| Artifact/research history destruction | [A-DOM-01-P0-01](../tasks/A-DOM-01.md), [A-DOM-01-P0-02](../tasks/A-DOM-01.md) | immutable run/revision artifacts and enrichment merge never erase prior nonempty evidence |
| Evolution mutation loss | [A-EVO-01-P0-01](../tasks/A-EVO-01.md) | read error cannot become empty authority; file/memory/log/catalog change is recoverable as one saga |
| Sensitive persistence | [A-OBS-01-P0-01](../tasks/A-OBS-01.md), [A-SRF-02-P0-01](../tasks/A-SRF-02.md) | secrets/tool arguments/terminal text are redacted or omitted before every durable/log/webhook sink |

These are independent acceptance cases even when they share primitives. A generic safe-write/redaction helper may be reused, but each owner retains domain rollback and lineage rules.

Validation: [V04](../validations/S-APP-01/V04-01.md).

## Main Risk Clusters

### 1. Workspace and generation split

Representative owners: A-BOOT-01-P1-01, A-CFG-01-P1-02/P1-03/P1-04/P1-05, A-MEM-01-P1-02..P1-05, A-INT-01-P1-01..P1-04, A-PLG-01-P1-02/P1-03, A-SUB-01-P1-01, A-PROJ-01-P1-02, plus [X-MEM-01](../tasks/X-MEM-01.md) and [X-PLG-01](../tasks/X-PLG-01.md).

A workspace switch can update cwd/UI instructions while memory Tools, compression, LSP, pooled Agents, review integration and config source still point elsewhere. Config/plugin reload similarly publishes partial generations. The fix is one application transaction/saga with a generation token, prepare/validate/apply order, rollback or degraded receipt, and last-known-good state. Consumers acknowledge the same generation before success is reported.

### 2. Terminal, cancellation and delivery split

Representative owners: A-CHAT-01-P1-01..P1-05, A-SRF-03-P1-01/P1-02, A-SRF-04-P1-03/P1-05/P1-06, A-HITL-01-P1-01..P1-05, A-OBS-01-P1-02/P1-03, A-BOOT-01-P1-04, plus [X-EVT-01](../tasks/X-EVT-01.md) and [X-SRF-01](../tasks/X-SRF-01.md).

Agent outcomes, transport completion, persistence, render events, queued-turn ownership and task handles are separate. Failure can become completed, disconnect can lose cancel, remount can detach execution, and detached delivery has no terminal. Define one typed application turn outcome/envelope with conversation/message/invocation identity, sequence, terminal status, durable cursor and owner handle. Sinks render it; they do not settle it. One lifecycle supervisor owns cancel/join/shutdown.

### 3. Durable state and recovery split

Representative owners: A-STATE-01-P1-02..P1-05, A-TSK-01-P1-01..P1-03, A-TSK-04-P1-01..P1-03, A-EVO-01-P1-03/P1-04, A-INP-01-P1-04/P1-05, plus [X-STA-01](../tasks/X-STA-01.md).

Corruption is sometimes absence, append and projection are not one recoverable generation, recovery publishes intermediate state, and deletion has no tombstone/cascade. Keep file storage but add typed `Missing | Valid(generation) | Corrupt(evidence)`, checksums/revisions, atomic projection receipts, prefix-safe JSONL recovery, idempotent recovery, and deletion tombstones. SQLite is not required and must not be introduced.

### 4. Task claim, artifact and policy split

Representative owners: A-TSK-02-P1-01/P1-02, A-TSK-03-P1-01..P1-03, A-TSK-05-P1-01..P1-04, A-TSK-06-P1-01..P1-03, A-TOOL-01-P1-01/P1-02, A-FE-02-P1-01..P1-04, plus X-TSK-01 and [X-TOL-01](../tasks/X-TOL-01.md).

Graph validation is canonical, but run bootstrap occurs before commit; application dispatch owns retry/settlement branches; worktree/artifact identity is not claim-bound; frontend current selection drops revision. Make physical attempt/claim identity immutable through dispatch, worktree, verification, integration, artifact and UI. One canonical policy snapshot is used by every authoring entry. App adapters return typed product facts; framework owns generic retry/cancel/settlement.

### 5. Surface parity and projection loss

Representative owners: A-SRF-01-P1-01..P1-03, A-SRF-02-P2-05, A-SRF-04-P1-01/P1-02/P1-04/P1-06, A-TOOL-01-P1-03, A-OUT-01-P1-01/P1-07, A-FE-01-P1-01, A-FE-02-P1-04, A-HITL-01-P1-05, A-EVO-01-P1-05, plus X-SRF-01.

These are defects, not product choices. Build a capability matrix from definition -> composition -> command/trigger -> typed event/snapshot -> reducer/render -> cancel/recovery for GUI, TUI, CLI, channels, cron and background. Trigger/render differences are valid; missing Agent capability is not. Move GUI-only workflow business logic out of Tauri commands into app-core and supply thin adapters to every surface.

### 6. Frontend cost and accessibility

Representative owners: A-FE-03-P1-01/P1-04/P2-02/P2-03 and A-FE-02-P2-05. Normalize stores by canonical identity, use selectors/indexes and lazy artifact loading, and provide real focus-modal semantics/responsive constraints. Do this after event/identity authority is fixed so optimization does not cement wrong state.

## Iteration Sequence

| Stage | Scope and canonical owners | Acceptance | Deletion target |
|---|---|---|---|
| 0. Containment | all nine P0 owners | fault at every read/write/rename/send boundary; no cross-workspace overwrite, old bytes retained, no secret/content in sinks | raw terminal preview logging, unredacted webhook payloads, overwrite-in-place run artifacts, parse-error-to-empty mutations |
| 1. Workspace generation | A-BOOT/A-CFG/A-MEM/A-INT/A-PLG/A-SUB/A-PROJ; X-MEM/X-PLG | switch/reload with primary+pooled Agents, stores, LSP/MCP, hooks, config and UI; all expose same generation or explicit rollback/degraded receipt | cwd rediscovery on save, per-Agent duplicate FileStore authority, primary-only refresh, detached stale reconciliation |
| 2. Durable state aggregate | A-STATE/A-TSK-01/04/A-EVO/A-INP; X-STA | crash/corrupt/partial-tail at every boundary, idempotent recovery, stale writer rejection, complete deletion cascade | error-to-empty admission, multi-step published recovery, dormant second persistence/search authorities |
| 3. Turn and Task lifecycle | A-CHAT/A-HITL/A-TSK-02/03/05/06/A-TOOL/A-OBS; X-EVT/X-TSK/X-TOL | exactly one typed terminal; cancel/disconnect/remount/restart; unique claim attempt through artifact/integration; durable delivery cursor | sink-local terminal inference, application generic retry loop, unowned detached sends, queue refs as lifecycle authority |
| 4. Surface parity | A-SRF-01..04/A-FE/A-OUT/A-EVO/A-TOOL; X-SRF | matrix fixture for every capability and surface, same artifact/terminal facts, surface-specific rendering only | prose parity test, GUI-only business logic in Tauri, hidden/unreachable declared commands, lossy per-surface event models |
| 5. Performance and submission gates | A-FE-03 plus Q-CLI/Q-GUI/Q-WEB and Q-PERF | normalized selector benchmarks, bounded retention, all exact Rust/GUI/frontend commands pass at pinned toolchains | eager full-output transport, global rescans, duplicate polling owner, unpinned Node/npm and missing CI lanes |

This order is dependency-aware, not an estimate. S-RDM-01 should split stages into reviewable batches and combine shared framework work only after S-FW/S-X reconciliation.

Validation: [V07](../validations/S-APP-01/V07-01.md).

## Quality Gate Status

| Gate | Static result | Executable result |
|---|---|---|
| EKO Rust | manifests/CI commands align; EKO excludes SQLite | five commands `not_run` in Q-CLI-01 |
| GUI-only Rust | target/config topology coherent; CI lacks isolated GUI lane | GUI check and tests `not_run` in Q-GUI-01 |
| Frontend | scripts/config/lock coherent; Node 18 contradicts locked Vitest 20+ requirement | Prettier, tests and production build `not_run` in Q-WEB-01 |

No application release/merge readiness claim is valid until new immutable attempts execute all required commands with the pinned framework and compatible Node/npm runtime. A green existing frontend suite will still not close Q-TST-01-P1-01's missing mounted transport harness.

## Commit Freshness

All 29 A reports use current CLI commit `b3b2e81...`. Twenty-seven use current framework commit `3aa792...`; A-BOOT-01 and A-CFG-01 use previous `9b0e0f...`. The intervening framework commit changes only ReAct tool/pipeline/stream files and testing mocks, not their bootstrap/config anchors, so their conclusions remain current. Live dirty framework source and CLI `Cargo.lock` were excluded.

Validation: [V06](../validations/S-APP-01/V06-01.md).

## Validation Matrix

| ID | Claim | Status | Report |
|---|---|---|---|
| V01 | A catalog/report/validation coverage and priority counts | passed | [V01](../validations/S-APP-01/V01-01.md) |
| V02 | Finding uniqueness and contradiction reconciliation | passed | [V02](../validations/S-APP-01/V02-01.md) |
| V03 | Authority/layering synthesis | passed | [V03](../validations/S-APP-01/V03-01.md) |
| V04 | Complete application P0 inventory | passed | [V04](../validations/S-APP-01/V04-01.md) |
| V05 | Q-CLI/Q-GUI/Q-WEB dependency status | passed; executable evidence missing | [V05](../validations/S-APP-01/V05-01.md) |
| V06 | Commit freshness and dirty isolation | passed | [V06](../validations/S-APP-01/V06-01.md) |
| V07 | Prioritization, layering and deletion criteria | passed | [V07](../validations/S-APP-01/V07-01.md) |
| V08 | Exact path/link/ID/executor/isolation integrity | both attempts passed; 01 used an invalid shell gate and 02 is accepted | [01](../validations/S-APP-01/V08-01.md), [02](../validations/S-APP-01/V08-02.md) |
| V30 | Primary coverage, P0, layering and quality-debt sampling | passed | [V30-01](../validations/S-APP-01/V30-01.md) |

## Residual Uncertainty

- No command, test, build, fixture, UI launch, crash injection, or network validation ran in synthesis.
- Counts describe reported manifestations; shared root-cause remediation must preserve every atomic acceptance case.
- Q-GUI's duplicate capability file has medium confidence until generated Tauri context is inspected dynamically.
- Framework generic fixes may reorder application stages after S-FW/S-X final synthesis; application product ownership does not change.

## Handoff

- S-X-01 should reconcile the lifecycle/store/Task adapter dependencies above with S-FW without merging away application acceptance cases.
- S-QA-01 should carry every Q-CLI/Q-GUI/Q-WEB `not_run` command and Q-TST mounted-harness gap.
- S-RDM-01 should use the six stages, canonical IDs, acceptance cases and deletion targets here; it must not invent a new Task/store/event/approval authority.
- Re-run V01/V02/V05/V06 if any atomic status/finding, Q gate, or pinned commit changes.
