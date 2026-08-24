# A-TSK-01: TaskRuntime file authorities and typed adapter

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean

## Question

Do plan/events/run-state files have unambiguous authority, and is conversion
to framework task types thin and lossless?

**Answer: Yes in the steady state and on the write path — `events.jsonl` is
the sole write authority and `plan.json`/`run-state.json` are derived
projections rebuilt from it; the EKO adapters are thin and the conversions
are lossless for every field and every status producible on live paths.
Two crash-consistency/recovery defects remain: a torn tail in `events.jsonl`
permanently bricks the run (P1, executed proof), and the read path never
rebuilds from events when a projection is missing/corrupt (P2). Two P3
notes: a latent status-projection loss (framework `Retrying`/`Paused`) and
the absence of cross-process writer exclusion.**

## Scope

- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/`: `types.rs`
  (EkoTaskSpec/EkoTaskExecution/PlanTask/TodoItem/PlanRevision/TaskRun/
  TaskUpdateRequest + conversions), `file_shadow.rs` (append/rebuild/atomic
  write primitives), `file_store.rs` (read API), `event_rebuild.rs`
  (event-sourced fold), `store.rs` (TaskRuntimeStore: CAS, task status
  writes, boot recovery), `revisioned_adapter.rs` (EkoRevisionedTaskStore +
  EkoTaskToolPolicy + commit/update adapters), `register.rs` (tool
  registration), `task_tools.rs` (TaskCapabilityCatalog boundary only).
- Framework boundary for the round trip: `echo-agent/echo-orchestration/src/
  tasks/runtime.rs` (TaskSpec/TaskExecution/TaskStatus/TaskClaim), `tasks/
  revisioned.rs` (TaskPlanPatchOp/TaskPatchEngine/TaskGraphContext/
  TaskGraphCommit/RevisionedTaskGraph), `tasks/task_tools.rs` (schema side,
  classification only).
- Reachability anchors: `echo-agent-cli/src/main.rs`, `src/tauri/desktop.rs`,
  `src/tauri/commands/task_runtime.rs`, app-core `state.rs`,
  `tool_exposure.rs`, `tasks/service.rs`.

## Out Of Scope

- EKO controller/dispatcher and framework executor semantics (claims, waves,
  retries, stalls) — A-TSK-03 / F-TSK-03.
- Recovery claims, stale-write rejection, event replay ordering across
  restarts — A-TSK-04 (boot recovery here only as a consumer of file reads).
- EKO authoring tools and tool schemas — A-TSK-02.
- Frontend projections — A-FE-01/02; legacy `TaskManager`/`TaskExecutor`
  surface — F-TSK-01-P3-01; blocked-propagation string contract and skip
  propagation — F-TSK-02-P1-01/P2-01 (accepted as dependencies).

## Inputs

- Root `AGENTS.md` (single task-relation API; TaskPlan artifact / TodoItem
  UI projection; adapter thin-and-lossless gate; framework-vs-app layering;
  UTF-8/panic safety).
- Shared `README.md`, `REPORTING.md`, `TASKS.md` (A-TSK-01 card),
  `zcode-ds/README.md`.
- Dependency reports read: `zcode-ds/reports/tasks/F-TSK-01.md` (canonical
  model singular; legacy surface P3-01; EKO adapter boundary = conversion
  only) and `F-TSK-02.md` (single structural validator; no second EKO
  frontier/validator; blocked-reason string contract P2-01; skip stalls
  P1-01).
- Historical documents treated as hypotheses: `echo-agent-cli/docs/
  MASTER-PLAN.md:170-240`, `echo-agent-cli/docs/2026-07-21-dynamic-plan-runtime.md`,
  `echo-agent-cli/docs/2026-07-28-task-tools-framework-migration-design.md`
  (all classified in V05-01).

## Layering Decision

- Generic mechanism (framework, correctly placed — consistent with
  F-TSK-01): `TaskSpec`/`TaskExecution`/`TaskStatus`/`TaskClaim` model,
  `TaskRevisionService`/`TaskPatchEngine`/`TaskPlanPatchOp`,
  `PlanValidator` structural validation, `RevisionedTaskStore` trait. EKO
  never re-implements these (V01-01).
- EKO product policy (application): `TaskRun`/`TaskPlan`/`PlanTask`/
  `TodoItem`/`TodoStatus`/`DomainProfile` projections, the file layout
  (`events.jsonl`+`plan.json`+`run-state.json`), boot recovery,
  `TaskCapabilityCatalog` capability checks, `planner.rs` file-ownership
  policy.
- Adapter boundary: `EkoRevisionedTaskStore` implements only
  `load`/`compare_and_commit` (revisioned_adapter.rs:26-56) — no patch
  engine, no validator, no ready frontier, no retry/cancel/deadlock logic
  (AGENTS.md gate 4 passes); `EkoTaskToolPolicy` supplies schema
  extensions, scope resolution, EKO metadata round trip, capability checks;
  conversions live in `types.rs` (`EkoTaskSpec::to_task_spec`,
  `TaskUpdateRequest::to_task_plan_patch`, `PlanTask::to_task`/
  `try_from`) and `store.rs` (CAS plan mapping). `EkoTaskExecution.status`
  stores the framework `TaskStatus` verbatim — the file projection is not a
  UI-only badge.
- Duplicate search terms (both repos, V01-01): `TaskPlan`, `PlanTask`,
  `TodoItem`, `TodoStatus`, `EkoTaskSpec`, `TaskSpec`, `TaskExecution`,
  `TaskStatus`, `TaskKind`, `PlanRevision`, `TaskRun`, `todo_write`,
  `plan_create`, `plan_patch`, `plan_execute`, `PlanValidator`,
  `validate_task_snapshot`, `detect_cycle`, `topolog*`, `Sqlite`,
  `rusqlite`, store structs. Result: one model family, one structural
  validator, no forbidden CRUD, no SQL in app-core `tasks/`.
- Migration deletion check: no new deletion target identified; the only
  un-implemented cleanup is the torn-tail repair the code comment promises
  (P1-01) and the read-side rebuild fallback (P2-01) — both additions, not
  deletions.

## Current Path

Verified data flow (V01-01/V02-01):

1. Bootstrap: `TaskRuntimeStore::new()` (file shadow rooted at
   `~/.eko/tasks/`) constructed in TUI (`src/main.rs:37`) and GUI
   (`state.rs:548`); `recover_incomplete` runs at boot (`main.rs:52`,
   `state.rs:557`); tools registered post-hoc (`register.rs:45-130`) for
   GUI (`desktop.rs:201-217`) and TUI (`main.rs:177-192`) with the
   framework `task_create/task_update/task_list` + EKO
   `task_execute`/`create_complex_task`/`check_run_status`/`cancel_run`.
2. Write path (every mutation): `with_run_lock(run_id)` (per-run in-process
   mutex) -> validate by reading the files -> `append_event_line`
   (`events.jsonl`, per-run seq, fsync) -> `rewrite_plan` (rebuild
   `plan.json`/`run-state.json` from the full event stream, atomic renames)
   — store.rs:302-357, 387-428, 953-1176, 755-885; file_shadow.rs:118-280.
3. Revision path: `task_create/task_update` -> `TaskRevisionService` ->
   `EkoRevisionedTaskStore::load/compare_and_commit` ->
   `load_revisioned_task_graph` (plan.json + run-state.json -> framework
   `Task` incl. exact `TaskStatus`) / `compare_and_commit_revisioned_task_graph`
   (CAS on revision, converts `commit.next` to `PlanRevision` + persists
   `skipped_task_ids`/`reset_task_ids` effects, appends
   `PlanRevisionCommitted`, rebuilds) — store.rs:676-885. The planner path
   (`commit_eko_task_plan`, revisioned_adapter.rs:344-388) uses the same
   service/CAS.
4. Read path: `get_run`/`get_plan`/`list_todos`/`list_events` delegate to
   `FileTaskStore` over the shadow — every read goes through the files; the
   only in-memory state is the per-run seq cache (self-healing from line
   count) and operational cancel tokens (store.rs:1482-1526, 1778-1808).
5. Execution state writes go exclusively through TodoStatus-typed store
   methods (`claim_task`, `set_claimed_task_status`, `requeue_claimed_task`,
   `set_task_status`); the framework `TaskStatus` is reconstructed exactly
   from `TodoStatus`+`status_detail`+`claim` on every read-back
   (V02-01/V03-01).

## Findings

### A-TSK-01-P1-01: Torn tail line in `events.jsonl` (crash mid-append) makes the run permanently unreadable and unrecoverable

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_shadow.rs:362-379`
  (`read_events` hard-errors `ShadowError::Decode` on any malformed line),
  `:289-299` (`next_seq` seeds from the same read, so writes fail too),
  `:114-117` (code comment: "A crash mid-append can at worst lose the last
  partial line — `read_events` skips empty lines and a future hardening pass
  (gate 2) will truncate a partial tail" — truncation is NOT implemented;
  grep for `truncate`/partial-tail repair in `task_runtime/` returns zero),
  `file_store.rs:37-55` (error propagation), `store.rs:1635-1638`
  (`recover_incomplete` warns and skips the run).
- Reachability: definition -> `append_event_line` (every mutation, fsync
  append; process crash during `write_all` leaves a torn final line) ->
  every subsequent read (`get_run`/`get_plan`/`list_todos`/CAS load) and
  write (`next_seq`) of that run fails -> boot `recover_incomplete` detects
  the run as zombie every restart but can never transition it
  (transition_run -> get_run -> read_events -> Err).
- Expected invariant: the append-only authority tolerates a torn final line
  (the project's own stated intent): reads succeed with the partial event
  dropped, and seq continues from the last valid event.
- Observed behavior: a torn tail returns `ERROR line 3: EOF while parsing`
  (executed reproduction of the exact parsing logic, V03-02); the run is
  unreadable and unwritable forever — only manual file editing recovers it.
- Impact: a process crash during any event append (OOM kill, power loss,
  `kill -9`) can permanently hide all task data of that run from the app:
  UI shows errors, run cannot be resumed/cancelled/listed correctly, boot
  recovery silently gives up.
- Root cause: `append_event_line` is not crash-atomic (plain `O_APPEND`
  write with no framing/length prefix) and the read side has no tail-repair
  logic despite the comment's promise.
- Direction: on a decode error of the FINAL line, truncate `events.jsonl`
  to the last valid line boundary (or verify the tail before returning the
  error) and rebuild; or prefix each line with its length and skip a
  truncated final record; add a regression test writing a run, tearing the
  last event line, and asserting reads succeed and seq continues.
- Regression validation: fixture "events.jsonl with torn final line still
  yields get_run/get_plan and the next append gets the correct seq"; boot
  recovery test with a torn-tail run.
- Validation reports: [V03-02](../validations/A-TSK-01/V03-02.md),
  [V05-01](../validations/A-TSK-01/V05-01.md)

### A-TSK-01-P2-01: Read path never rebuilds from the authoritative event log — missing/corrupt projections permanently brick a run

- Priority: P2
- Confidence: high (mechanism) / medium (occurrence window)
- Layer: application
- Evidence: `file_shadow.rs:208-280` (`rewrite_plan` writes `plan.json`
  then `run-state.json` as two separate atomic renames), `:339-359`
  (`read_plan`/`read_run_state` return None on missing file, hard Decode
  error on corrupt file), `file_store.rs:37-55` (`load` returns `Ok(None)`
  when run-state.json is absent even though plan.json and events.jsonl
  exist), `store.rs:676-686` (`load_revisioned_task_graph` -> `RunNotFound`
  on missing run-state), `store.rs:1631-1639` + `file_store.rs:69-80`
  (`recover_incomplete` discovers runs only through run-state.json).
- Reachability: crash between the two renames in `rewrite_plan` (or external
  corruption/deletion of a projection) -> run invisible to `get_run`/
  `get_plan`, invisible to `list_runs`/`recover_incomplete` -> no further
  write ever occurs -> run stuck; a corrupt `plan.json` hard-errors every
  read instead of falling back to the events.
- Expected invariant: since `events.jsonl` is the declared recovery
  authority and `rebuild_plan_from_events` can reproduce both projections,
  a missing/corrupt projection must be regenerated from events on read (or
  at boot), never treated as absence/error.
- Observed behavior: the rebuild happens only on the next WRITE
  (`rewrite_plan`); there is no read-side or boot-side self-heal, so the
  "events.jsonl can authoritatively rebuild plan.json" guarantee (module
  doc event_rebuild.rs:1-9) does not cover reads after a crash.
- Impact: small but real crash window (two renames) plus any manual/corrupt
  file leaves a run permanently invisible or erroring, with no app-side
  recovery path.
- Root cause: projections are written as an unordered pair without a
  recovery scan; reads trust the projections instead of the event log.
- Direction: on load, if a projection is missing or fails to decode while
  events.jsonl exists and has a `RunCreated`, rebuild it from
  `rebuild_plan_from_events` (write-side code already exists and is tested);
  boot recovery should scan run dirs by `events.jsonl` presence, not
  run-state.json; write both projections under a single ordered pair with a
  "last rebuilt seq" marker if stricter atomicity is desired.
- Regression validation: fixture "run with events.jsonl only (no
  projections) is listable and readable after boot"; fixture "corrupt
  plan.json decodes from events"; crash-window test asserting
  plan.json-new/run-state.json-old reads either old or rebuilt state, never
  None.
- Validation reports: [V03-02](../validations/A-TSK-01/V03-02.md),
  [V01-01](../validations/A-TSK-01/V01-01.md)

### A-TSK-01-P3-01: EKO's `TodoStatus` projection cannot represent framework `Retrying`/`Paused` — latent lossless-conversion gap

- Priority: P3
- Confidence: medium
- Layer: adapter
- Evidence: `types.rs:423-439` (`try_from_task_status` errors on
  `Retrying`/`Paused`), `:443-456` (`project_task_status` silently collapses
  Retrying->Running, Paused->Blocked), `:1162-1201` (`PlanTask::try_from`
  uses the erroring conversion), `:1203-1216` (detail extraction handles
  both); `echo-orchestration/src/tasks/runtime.rs:90-107` (10-state
  `TaskStatus`); production constructors: `Retrying` has ZERO construction
  sites in either repo (grep V03-01), `Paused` only in legacy
  `manager.rs:446` (production-unreachable, F-TSK-01-P3-01).
- Reachability: definition -> `PlanTask::try_from` (only call site is a
  test, types.rs:1988) and `from_parts` (live: file_store.rs:146,
  event_rebuild.rs:160). The file itself (`EkoTaskExecution.status`) stores
  the framework status verbatim, so the authority is lossless; the loss is
  at the EKO projection layer only.
- Expected invariant: the documented "lossless" conversion is total over the
  framework status enum, or the framework and EKO state spaces are aligned.
- Observed behavior: today unreachable on EKO live paths; if any future
  framework path (e.g., F-TSK-03 retry semantics) emits `Retrying` or
  `Paused` into a graph read back through EKO, `PlanTask::try_from` hard-
  fails and `project_task_status` silently downgrades the state.
- Impact: none today; latent silent-loss/error risk at the adapter boundary.
- Root cause: EKO's UI status model predates the framework's
  `Retrying`/`Paused` variants and was never extended.
- Direction: either extend `TodoStatus` with `Retrying`/`Paused` (and their
  event/serde names) or, if the states are genuinely framework-internal,
  keep the error path but make `project_task_status` document the loss and
  add a round-trip test that pins the reachable state space; ensure any new
  framework producer of these states goes through A-TSK-04 review.
- Regression validation: test asserting `PlanTask::try_from` succeeds for
  every framework `TaskStatus` variant (or documents exactly which are
  rejected and why).
- Validation reports: [V03-01](../validations/A-TSK-01/V03-01.md)

### A-TSK-01-P3-02: File authority has no cross-process writer exclusion — GUI and TUI share `~/.eko/tasks/` without a file lock

- Priority: P3
- Confidence: low
- Layer: application
- Evidence: `file_shadow.rs:26-34,185-195` (per-run write locks are
  in-process `Mutex`es keyed in a process-local map), `:26` (seq_cache is
  process-local, "Shared across clones via Arc" — same-process only),
  `:127-131` (the duplicate-seq race this design fixed was observed
  in-process); two processes construct the store over the same default root:
  `echo-agent-cli/src/main.rs:37` (TUI) and `echo-agent-app-core/src/state.rs:548`
  (GUI).
- Reachability: definition -> user runs the TUI and GUI concurrently and
  the same run is mutated from both -> two independent seq caches (duplicate
  seqs break `list_events(since_seq)` deltas) and independent per-run locks
  (interleaved append + rebuild without mutual exclusion).
- Expected invariant: the file authority serializes same-run writers
  regardless of process, or the product documents single-process usage.
- Observed behavior: serialization is in-process only; concurrent
  GUI+TUI use of one run can duplicate seqs and race `rewrite_plan`.
- Impact: edge case for a local personal assistant; when it happens,
  event-seq monotonicity and projection consistency break with no error.
- Root cause: the store was designed around one process; locks and caches
  live in memory.
- Direction: a per-run file lock (e.g., advisory `flock` on `events.jsonl`
  or a `.lock` file) held across append+rebuild, and seq derived from the
  file length without the in-memory fast path when the lock is contended;
  or document that only one surface may run at a time.
- Regression validation: two-process fixture appending to one run asserting
  strictly increasing unique seqs.
- Validation reports: [V02-01](../validations/A-TSK-01/V02-01.md),
  [V01-01](../validations/A-TSK-01/V01-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition + duplicate search (file-authority table; model/validator/store duplication; forbidden CRUD) | yes | passed | [V01-01](../validations/A-TSK-01/V01-01.md) |
| V02 | Registration + runtime reachability (store/service/tools/CAS on all entry points) | yes | passed | [V02-01](../validations/A-TSK-01/V02-01.md) |
| V03 | Field-by-field round-trip (spec/execution/patch/commit/status; reachable state space) | yes | passed | [V03-01](../validations/A-TSK-01/V03-01.md) |
| V03 | Corrupt and partial state reconstruction (torn tail; missing/corrupt projections) | yes | failed (2 findings) | [V03-02](../validations/A-TSK-01/V03-02.md) |
| V04 | `cargo test -p echo-agent-app-core --locked tasks::task_runtime::types` | yes | passed (exit 0, 51 ok) | [V04-01](../validations/A-TSK-01/V04-01.md) |
| V04 | `cargo test -p echo-agent-app-core --locked tasks::task_runtime::event_rebuild` | yes | passed (exit 0, 3 ok) | [V04-02](../validations/A-TSK-01/V04-02.md) |
| V04 | `cargo test -p echo-agent-app-core --locked tasks::task_runtime::file_shadow` | yes | passed (exit 0, 9 ok) | [V04-03](../validations/A-TSK-01/V04-03.md) |
| V04 | `cargo test -p echo-agent-app-core --locked tasks::task_runtime::file_store` | yes | passed (exit 0, 5 ok) | [V04-04](../validations/A-TSK-01/V04-04.md) |
| V04 | `cargo test -p echo-agent-app-core --locked tasks::task_runtime::store` | yes | passed (exit 0, 34 ok) | [V04-05](../validations/A-TSK-01/V04-05.md) |
| V04 | `cargo check -p echo-agent-app-core --locked` | yes | passed (exit 0) | [V04-06](../validations/A-TSK-01/V04-06.md) |
| V05 | Historical-document drift (MASTER-PLAN, dynamic-plan-runtime, migration design) | conditional | passed | [V05-01](../validations/A-TSK-01/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN: `events.jsonl` recovery authority; `plan.json` immutable plan spec; `run-state.json` execution projection | current | file_shadow.rs:6-7, 208-280; file_store.rs:3-11 (V05-01) |
| MASTER-PLAN: Todo status from run-state.json; older Task events cannot override a later skip/reset | current | file_store.rs:190-199; V04-04 test passed |
| MASTER-PLAN: run-state.json stores TaskStatus detail independent from failure_fingerprint | current | types.rs:1015-1019, 922-929 |
| MASTER-PLAN M13: framework owns frontier/waves/validation; EKO validates only catalog + file-ownership | current | store.rs:96-101, 921-923; task_tools.rs:49-81; planner.rs:8-9 |
| dynamic-plan-runtime: authority split; TaskRuntimeStore validates and atomically commits plan revisions | current | store.rs:755-885 (V01-01/V04-05) |
| task-tools-migration: adapters must not retain a second patch engine or validator; envelope fields preserve losslessly | current | revisioned_adapter.rs:26-56; V03-01 |
| file_shadow comment: torn tail tolerated/truncated by "gate 2" | regressed (unimplemented) | file_shadow.rs:362-379 hard-errors; V03-02 -> A-TSK-01-P1-01 |
| F-TSK-01/F-TSK-02 handoffs: EKO store CAS unverified (now verified); no second validator/frontier | current | store.rs:676-885; V01-01 |

## Coverage And Uncertainty

- `executor.rs` (EKO controller) was inspected only at its store-call sites;
  dispatch/wave/retry semantics are A-TSK-03; claim identity/replay across
  restarts is A-TSK-04.
- The P1-01/P2-01 findings are crash-consistency defects: the mechanisms are
  certain and P1-01 was executed, but the real-world crash probability is
  not quantified; both are cheap to fix and test.
- Cross-process concurrency (P3-02) was not exercised (two live processes);
  judgment is by code inspection.
- `TaskExecutionSummary::to_runtime_summary` and `SubagentTaskResult`
  conversions were checked structurally but are consumed by A-TSK-06
  (review/compact-context); verification-to-string flattening there is
  outside this task's authority question.
- Frontend (ts-rs bindings) round trips were not reviewed (A-FE-01/02).

## Handoff

- Downstream tasks may rely on: unambiguous steady-state file authority
  (events.jsonl sole write authority; projections derived; reads via files)
  and a thin, lossless adapter for all live-path fields/statuses (V01-01,
  V03-01, V04-01..06); EKO CAS verified at store.rs:676-885; crash
  consistency gaps P1-01/P2-01; status-space note P3-01; cross-process note
  P3-02.
- Reports to read: all 12 validation reports above; F-TSK-01 (canonical
  model + legacy surface), F-TSK-02 (validator/frontier + skip/blocked
  semantics), F-TSK-03 (executor semantics) when it is available.
- Stale conditions: this report becomes stale if `file_shadow.rs`
  read/append/rebuild behavior, `store.rs` CAS or status-write paths,
  `types.rs` conversion functions, or `file_store.rs` read semantics change;
  also if a production constructor of `TaskStatus::Retrying`/`Paused`
  appears.
- Follow-up task IDs: A-TSK-03 (controller boundary), A-TSK-04 (recovery/
  claims; P1-01/P2-01 reachability in restart scenarios), X-TSK-01 (field
  round-trip across framework and EKO adapters), Q-FLT-02 (crash fixtures
  for P1-01/P2-01), S-RDM-01 (roadmap: torn-tail repair + read-side rebuild
  + status-space alignment + cross-process lock).
