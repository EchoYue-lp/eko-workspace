# A-TSK-01: TaskRuntime file authorities and typed adapter

> Status: complete
> Reviewer: Codex primary reviewer (delegated static evidence independently sampled)
> Executor: Codex primary reviewer
> Review date: 2026-08-13
> `echo-agent` commit: 3aa7929928442aab91e4dce9c426d909a5f0a1ab
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean; report-only files added outside them

## Question

Do plan/events/run-state files have unambiguous authority, and is conversion to framework task types thin and lossless?

## Scope

- Framework generic Task/TaskSpec/TaskExecution/status, revision service, patch engine, and structural plan validator.
- EKO TaskRun/TaskPlan/PlanTask/SubagentRun/Todo projections and framework conversions.
- EKO `events.jsonl`, `plan.json`, `run-state.json` write/read/rebuild paths.
- Revisioned adapter registration and production reachability from TUI/GUI/background callers.
- Missing projection, corrupt JSONL tail, malformed committed-plan, revision, and cycle fixtures.

## Out Of Scope

- Planner/tool authoring UX: A-TSK-02.
- Executor scheduling/resource arbitration: A-TSK-03.
- Claims, retry, cancellation, completion, and recovery policy beyond file reconstruction: A-TSK-04.
- TUI/GUI/channel/cron feature parity beyond adapter registration: A-TSK-05/A-TSK-06 and cross-surface tasks.
- Framework-wide task-family legitimacy: F-TSK-01/F-TSK-02.
- Source fixes, SQLite, permission gates, and index updates.

## Inputs

- Read root `AGENTS.md`, shared `README.md`, `REPORTING.md`, `TASKS.md`, Codex `README.md`, and report templates.
- Required dependency reports `F-TSK-01` and `F-TSK-02` are complete and were
  consumed by primary review for framework-authority ownership and deduplication.
- No historical reviewer report was read or copied. Source comments and tests were treated as hypotheses and independently checked.

## Layering Decision

| Classification | Current answer |
|---|---|
| Generic mechanism | Revisioned Task graph, patch semantics, CAS trait, structural DAG validation, TaskSpec/TaskExecution, and lifecycle status belong in `echo-agent`; unrelated consumers can use them. |
| EKO product policy | File layout, TaskRun identity, domain profile, attended mode, UI todo projection, local capability catalog, and subagent event hydration belong in `echo-agent-cli`. |
| Adapter boundary | Production revision calls are thin and preserve current fields, but persistence/rebuild is application policy and currently violates its declared event-authority contract. |
| Duplicate search | Searched type names, status/field names, patch/validator traits, constructors, registrations, read/write callers, and SubagentRun event/DTO paths across both repositories. |
| Migration deletion | Repair must leave one operational file authority; if event sourcing remains, delete read paths that trust unversioned projections. For SubagentRun, delete the unused aggregate/generated contract or delete the separate frontend re-model after adopting the aggregate. |

No CLI SQLite dependency or recommendation is involved.

## Current Path

Production task graph authoring follows:

```text
TUI main / Tauri desktop / tool exposure
  -> register_task_tools_on_agent
  -> framework task_create/task_update/task_list
  -> TaskRevisionService
       -> TaskPatchEngine
       -> EkoTaskToolPolicy (EKO metadata/capabilities)
       -> PlanValidator
       -> EkoRevisionedTaskStore
  -> TaskRuntimeStore::compare_and_commit_revisioned_task_graph
  -> append PlanRevisionCommitted to events.jsonl
  -> rewrite plan.json and run-state.json projections
```

GUI `update_tasks` reaches the same service through `apply_eko_task_update` ([V02](../validations/A-TSK-01/V02-01.md)); background DAG submission reaches `create_prepared` through `commit_eko_task_plan`. `PlanTask` maps current framework spec/execution/context fields explicitly ([V03](../validations/A-TSK-01/V03-01.md)).

The persistence path is weaker than its architecture claim. `TaskRuntimeStore` declares `events.jsonl` authoritative at `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:58`, but mutations return from `append_event_line` before separately rebuilding projections (`store.rs:657-669`, `867-881`, `1158-1174`). Individual projection files are atomically renamed (`file_shadow.rs:396-421`), but readers directly deserialize `plan.json`/`run-state.json` (`file_store.rs:37-54`) without an applied sequence or load-time rebuild.

Subagent attempts are actually persisted as `SubagentAssigned`/`SubagentReleased` events (`store.rs:1898-1969`) and hydrated by frontend `taskRuntimeSubagentExecutionEvents`. The Rust aggregate `SubagentRun` and its generated TypeScript DTO have no production constructor/consumer ([V14](../validations/A-TSK-01/V14-01.md)).

## Findings

### A-TSK-01-P1-01: Event and projection commits are not crash-atomic or self-healing

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:58`, `:657`, `:669`, `:867`, `:881`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_shadow.rs:208`, `:267`, `:273`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_store.rs:37`
- Reachability: all production run/task/plan mutations append to `events.jsonl`, then call `rewrite_plan`; all normal `get_run`/`get_plan`/scheduler reads delegate to `FileTaskStore` and direct projection reads.
- Expected invariant: the declared event authority can recover every accepted mutation, and readers never accept/return state from a missing or stale deterministic projection.
- Observed behavior: event append and one/two projection renames are separate durable operations. Projections contain no last-applied event sequence; normal reads neither compare them with the log nor rebuild. A valid RunCreated event plus deleted `run-state.json` produces `events=1 get_run=None`.
- Impact: a crash or IO error after event fsync can make an existing run disappear, leave task status/revision stale, or combine new `plan.json` with old `run-state.json`; scheduler/recovery and UI then act on a non-authoritative snapshot even though the committed event remains.
- Root cause: append-only event authority was layered under projection-first reads without a transaction marker, generation manifest, or load-time repair.
- Direction: keep this EKO-local. Choose `events.jsonl` as the one canonical record; add a commit/applied-seq (and plan revision) to projections or a single generation manifest, repair/rebuild under the per-run lock on load/startup, and atomically publish one consistent projection generation. Delete direct projection trust paths once repair is canonical; do not add SQLite.
- Regression validation: inject failure after event append, after plan rename, and before run-state rename; restart a fresh store and assert one revision/status across get_run/get_plan/task_list/executor.
- Validation reports: [V05](../validations/A-TSK-01/V05-01.md), [V06](../validations/A-TSK-01/V06-01.md), [V13](../validations/A-TSK-01/V13-01.md)

### A-TSK-01-P1-02: A crash-truncated JSONL tail disables the entire authoritative run log

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_shadow.rs:105`, `:114`, `:282`, `:361`
- Reachability: `read_events` serves event APIs, every rebuild, recovery folds, and `next_seq` on the first append after restart.
- Expected invariant: a crash during the last append may lose only that incomplete event; earlier newline-complete events remain readable and future appends can continue safely.
- Observed behavior: `read_events` deserializes every non-empty line and returns on the first decode error. A partial final line produced `InvalidPlan(... line 2: EOF ...)`; `next_seq` calls the same failing reader, so later writes cannot self-heal. The code comment itself defers tail truncation to a future pass.
- Impact: one process interruption at the append boundary can make history, recovery, UI hydration, and subsequent mutations unusable for the run until manual file repair.
- Root cause: append durability exists without a recovery parser that distinguishes an incomplete final record from interior corruption.
- Direction: on locked startup/read, retain all newline-complete valid records, quarantine/truncate only an invalid non-newline tail, fsync the repaired file/directory, reset sequence from validated last seq, and hard-fail on invalid interior lines, duplicate/non-monotonic seq, or cross-run IDs.
- Regression validation: partial bytes at every JSON token boundary; valid prior history remains; interior corrupt line remains a hard error; next append produces exactly last valid seq + 1.
- Validation reports: [V07-01](../validations/A-TSK-01/V07-01.md), [V07-02](../validations/A-TSK-01/V07-02.md), [V07-03](../validations/A-TSK-01/V07-03.md)

### A-TSK-01-P1-03: Malformed committed-plan payload is silently converted to a successful empty plan

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/event_rebuild.rs:138`, `:257`, `:286`
- Reachability: every projection rewrite folds the full event log through `rebuild_plan_from_events`; the function is also the claimed authoritative reconstruction API.
- Expected invariant: a `PlanRevisionCommitted` event is atomic and schema-valid or rebuild returns a typed corruption error; it must not erase a previously visible plan or invent revision 0.
- Observed behavior: `serde_json::from_value::<PlanRevision>(...).ok()` silently ignores invalid committed plan payloads. With RunCreated plus `{plan:{plan_id:7}}`, rebuild returned `Ok` with revision 0, empty plan ID, and zero tasks.
- Impact: corrupt/partial-but-JSON-valid event data is misclassified as a legitimate empty graph; a projection rewrite can overwrite readable state with an empty plan/run-state instead of stopping for repair.
- Root cause: best-effort optional-field parsing is applied to a required authoritative commit payload, and `empty_plan_for` cannot distinguish “no plan yet” from “invalid plan commit observed.”
- Direction: make commit decoding, run ID consistency, revision monotonicity, task references, and status payloads fallible typed rebuild operations. `empty_plan_for` is valid only when no commit event exists. Refuse projection replacement on rebuild error and preserve the last valid generation for diagnosis.
- Regression validation: invalid type/missing field/run mismatch/revision regression/unknown task status fixtures all return typed errors and never overwrite prior projections.
- Validation reports: [V08](../validations/A-TSK-01/V08-01.md)

### A-TSK-01-P2-01: `SubagentRun` is a dead nominal durable model while events and frontend state own reality

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs:1428`, `:1656`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:1898`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/web-frontend/src/stores/subagentRunStore.ts:325`
- Reachability: executor persists assigned/released events; GUI hydration folds them
  into `SubagentRunState`. A public `SubagentRun::new` definition exists, but
  search found no constructor call, aggregate struct literal, typed
  return/container, or generated `SubagentRun` import.
- Expected invariant: `TaskRun -> PlanTask -> SubagentRun` has one authoritative typed model or one explicitly documented event projection, not an unused exported DTO beside a different live UI shape.
- Observed behavior: a public `SubagentRun::new` constructor exists, but full-repository
  search finds no call or struct construction. The Rust/ts-rs aggregate is never
  materialized in production, while event payloads and a separately declared
  TypeScript `SubagentRunState` define the live contract.
- Impact: developers can extend the exported Rust DTO believing it is durable and receive no runtime/UI effect; backend and frontend attempt fields can drift without compiler-enforced round-trip coverage.
- Root cause: the subagent-unification migration introduced an aggregate contract but retained event-shaped runtime/frontend ownership and did not finish one side of the migration.
- Direction: decide at the EKO application layer. Prefer a single backend event-to-`SubagentRun` projection returned to all surfaces, then delete the independently shaped frontend reconstruction; alternatively delete the unused aggregate/generated type and explicitly version the event payload schema. Do not create a framework product DTO.
- Regression validation: one assigned/released/retry stream round-trips every attempt/status/usage/result field into the chosen single type and hydrates GUI/TUI/CLI consistently.
- Validation reports: [V14](../validations/A-TSK-01/V14-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate model search | yes | passed | [V01](../validations/A-TSK-01/V01-01.md) |
| V02 | Registration and production reachability | yes | passed | [V02](../validations/A-TSK-01/V02-01.md) |
| V03 | Field-by-field adapter/context/patch mapping | yes | passed | [V03](../validations/A-TSK-01/V03-01.md) |
| V04 | Representative typed round-trip Cargo test | yes | passed | [V04](../validations/A-TSK-01/V04-01.md) |
| V05 | File-authority table and consistency protocol | yes | failed | [V05](../validations/A-TSK-01/V05-01.md) |
| V06 | Missing projection reconstruction fixture | yes | passed (defect reproduced) | [V06](../validations/A-TSK-01/V06-01.md) |
| V07 | Corrupt JSONL tail fixture | yes | failed claim after 2 environment attempts | [V07-01](../validations/A-TSK-01/V07-01.md), [V07-02](../validations/A-TSK-01/V07-02.md), [V07-03](../validations/A-TSK-01/V07-03.md) |
| V08 | Invalid committed-plan fixture | yes | failed | [V08](../validations/A-TSK-01/V08-01.md) |
| V09 | Validator/patch duplicate and reachability search | yes | passed | [V09](../validations/A-TSK-01/V09-01.md) |
| V10 | Revision insertion Cargo test | yes | passed | [V10](../validations/A-TSK-01/V10-01.md) |
| V11 | Cycle rejection Cargo test | yes | passed | [V11](../validations/A-TSK-01/V11-01.md) |
| V12 | Valid lifecycle rebuild Cargo test | yes | passed | [V12](../validations/A-TSK-01/V12-01.md) |
| V13 | Normal append/rewrite Cargo test | yes | passed | [V13](../validations/A-TSK-01/V13-01.md) |
| V14 | SubagentRun definition/construction/consumer search | yes | failed | [V14](../validations/A-TSK-01/V14-01.md) |
| V15 | Subagent-only terminology in reviewed scope | yes | passed | [V15](../validations/A-TSK-01/V15-01.md) |
| V16 | Exact task ID, links, executor, isolation, clean source | yes | passed after corrected attempt | [V16-01](../validations/A-TSK-01/V16-01.md), [V16-02](../validations/A-TSK-01/V16-02.md) |
| V30 | Primary dependency closure, current-source sampling and acceptance | yes | passed | [V30](../validations/A-TSK-01/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `store.rs`: event stream authoritative; projections deterministic | regressed/incomplete | [V05](../validations/A-TSK-01/V05-01.md), [V06](../validations/A-TSK-01/V06-01.md) |
| `event_rebuild.rs`: events can authoritatively rebuild plan | current only for valid streams | [V12](../validations/A-TSK-01/V12-01.md); corrupt cases fail in [V08](../validations/A-TSK-01/V08-01.md) |
| `file_shadow.rs`: partial append can at worst lose final partial line | regressed | [V07-03](../validations/A-TSK-01/V07-03.md) shows the partial line disables all reads/writes |
| `types.rs`: framework status remains authoritative/lossless | current for store/revision adapter | [V03](../validations/A-TSK-01/V03-01.md), [V04](../validations/A-TSK-01/V04-01.md) |
| `types.rs`: SubagentRun lifecycle/result are durable | stale/incomplete | [V14](../validations/A-TSK-01/V14-01.md) |

## Coverage And Uncertainty

- F-TSK-01 and F-TSK-02 are complete. Their framework-wide duplicate task-family
  and DAG state findings remain canonical and are not duplicated here.
- The representative round-trip test does not enumerate every status/claim combination; primary should add independent exhaustive evidence.
- No multi-process file writer fixture was run. In-process locks and seq tests exist, but cross-process coordination is unproven.
- Cargo testing stopped after four targeted commands because the parent reported only 23 GiB free and combined workspace targets at 61 GiB. No Cargo session remains running.
- TUI/GUI/channel/cron semantic parity and executor recovery are downstream tasks; this report verifies only adapter registration and persistence contracts.

## Handoff

- A-TSK-02 may rely on `TaskRevisionService` as the production authoring authority and should not introduce Todo/Plan CRUD.
- A-TSK-03/A-TSK-04 must read P1-01 through P1-03 before trusting loaded snapshots during scheduling, completion, or recovery.
- A-TSK-05/A-TSK-06 should decide whether all surfaces consume one typed SubagentRun projection or the explicitly versioned event contract.
- F-TSK-01/F-TSK-02 dependency closure and primary current-source acceptance are
  recorded in V30. Exhaustive dynamic round-trip cases remain future regression
  work rather than missing static review evidence.
- This report becomes stale if TaskSpec/TaskStatus gains fields/variants, the file layout or rebuild algorithm changes, or SubagentRun gains a real constructor/query path.
