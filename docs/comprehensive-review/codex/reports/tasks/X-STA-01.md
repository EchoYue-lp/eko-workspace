# X-STA-01: Persistence, recovery, and identity continuity

> Status: complete
> Reviewer: Codex review subagent
> Executor: Codex review subagent
> Accepted by: Codex primary reviewer
> Review date: 2026-08-13
> `echo-agent` commit: 3aa7929928442aab91e4dce9c426d909a5f0a1ab
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: framework externally dirty and inspected only through committed `HEAD`; CLI `Cargo.lock` externally dirty and excluded

## Question

Do conversation, snapshot, Task, Subagent, artifact, and frontend identities
survive restart without duplication, stale overwrite, orphaning, or concealed
recovery evidence?

## Scope

- Framework committed `FileStore`, `FileConversationStore`, file runtime state,
  checkpoint restore/save, and their static persistence tests.
- EKO conversation restore/update/delete, pooled Agent continuity, TaskRuntime
  file authority and boot recovery, Subagent execution identity, RuntimeArtifact
  persistence, and frontend conversation/Task/Subagent projections.
- Definition/registration/reachability, identity table, corrupt/partial-file
  admission, crash-point recovery matrix, and retention/deletion cascade.

## Out Of Scope

- Atomic defects and their exact fixes already owned by `F-RCT-05`, `F-MEM-01`,
  `A-STATE-01`, `A-TSK-04`, and `A-FE-02`, except where needed to establish a
  cross-store contract.
- SQLite, which EKO must not enable; general framework SQLite capability remains
  valid and is not a deletion target.
- Source fixes, migration implementation, Cargo/rustc/frontend commands,
  executable crash fixtures, and network access.
- General artifact rendering and lazy output, owned by `A-FE-02`.

## Inputs

- Root `AGENTS.md`, exact `TASKS.md` card, shared `REPORTING.md`, Codex README,
  and report templates.
- Authorized Codex dependency reports: `F-RCT-05`, `F-MEM-01`, `A-STATE-01`,
  `A-TSK-04`, and `A-FE-02` only.
- Current CLI source at the pinned commit and framework committed blobs through
  `git show HEAD:<path>`/`git grep HEAD`; no other reviewer output was read.

## Layering Decision

Generic storage implementations may differ, but framework persistence contracts
must distinguish missing, corrupt, and partial data and must publish complete
checkpoint generations. EKO owns the product aggregate that links a local
conversation to TaskRuns, Subagent executions, tool/input artifacts, frontend
projections, and deletion policy. The adapter must carry typed identities and
generation tokens losslessly; it must not parse an opaque execution ID or add a
second recovery authority. Duplicate searches covered `conversation_id`,
`run_id`, `plan_revision`, `claim`, `attempt`, `execution_id`, `artifact_id`,
checkpoint save/load, recovery, restore, delete, cleanup, retention, and
frontend hydration in both repositories. Existing authorities should be
extended, not replaced by new stores.

## Current Path

### Identity authority table

| Domain | Durable identity and authority | Restart/projection join | Continuity result |
|---|---|---|---|
| Conversation | `conversation_id` -> one `FileConversationStore` JSON record; message numeric IDs are reconciled with `_meta.json` | GUI/TUI store, pooled Agent, tool/input artifact scopes | Stable record/message IDs, but no conversation generation/CAS token |
| ReAct checkpoint | one `AgentCheckpoint` slot keyed by `conversation_id`; timestamp is payload data | `restore_thread_context` -> field-wise restore of messages/plan/skills/cwd | Same key survives, but corrupt load resets and restore is not one published generation |
| TaskRun/PlanTask | `run_id`; Task identity joins plan revision and persisted `TaskClaim` attempt | boot recovery and `latest_run_for_conversation` | Exact normal claim identity exists; recovery itself is multi-write |
| SubagentRun | `{run_id}:{task_id}:{plan_revision}:{attempt}` execution ID | durable release events and Zustand key `(run_id, execution_id)` | History remains distinct, but current selector drops revision |
| RuntimeArtifact | `artifact.id`, `run_id`, optional `task_id`; persisted as `ArtifactProduced` with `step_id=None` | reconstructed from TaskRuntime events | No typed producer revision/attempt, so artifacts from replaced attempts lack exact lineage |
| Frontend | conversation load generation; TaskRuntime load/refresh generation and event `seq` | async fetch guards then store projections | Conversation/Task fetch races are fenced; Subagent current selection and terminal enrichment are not generation-complete |

The live application path is real: CLI main and GUI `AppState` construct
TaskRuntimeStore and call `recover_incomplete`; GUI/TUI conversation commands
reach the file conversation store; React conversation and TaskRuntime stores
hydrate mounted UI; framework ReAct startup reaches checkpoint restore and
execution reaches checkpoint save.

### Crash-point recovery matrix

| Crash/error point | Durable state after restart | Admission/continuity |
|---|---|---|
| full-record temp write before rename | old FileConversationStore/checkpoint snapshot remains | positive atomic full-record behavior |
| malformed FileConversationStore record | bytes retained; read returns error | positive fail-closed behavior |
| malformed FileStore JSON | opened as empty; next mutation flushes empty/new map | destructive admission, P0-01 |
| malformed ReAct checkpoint | messages reset and run continues fresh | no quarantine/recovery barrier, P0-01 |
| partial nonempty TaskRuntime JSONL tail | every event read fails at tail line | valid prefix becomes unavailable, P1-02 |
| after TaskRun `Running -> Paused`, before all task resets | run is Paused while a Task/claim remains Running | next boot scans only Running runs; aggregate can strand, P1-03 |
| conversation loaded before Agent restore succeeds | persisted/UI transcript can be current while pooled Agent is empty/stale | dependency-owned symptom of absent generation, P1-03 |
| terminal Subagent live event before richer durable event | first terminal wins; durable enrichment discarded | restart/arrival-order-dependent projection, P1-04 |
| conversation deleted before late writer/cascade finishes | selected records removed, TaskRuntime/checkpoint/projections remain | resurrection/orphaned history possible, P1-05 |

## Findings

### X-STA-01-P0-01: Corrupt persistent state is admitted as a fresh generation and can overwrite the only recovery evidence

- Priority: P0
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/store.rs:227`; `echo-agent/echo-state/src/memory/store.rs:235`; `echo-agent/echo-state/src/memory/store.rs:254`; `echo-agent/src/agent/react/run/context.rs:230`; `echo-agent/src/agent/react/run/context.rs:245`; dependency reports `F-MEM-01-P0-01` and `F-RCT-05-P0-04`
- Reachability: public `FileStore::new` or production ReAct startup reads corrupt
  durable bytes -> decode/load error becomes an empty context -> ordinary writes
  remain enabled -> atomic rename replaces the same durable slot.
- Expected invariant: corruption differs from absence, preserves/quarantines the
  original bytes, blocks destructive writeback, and requires explicit recovery
  or an independently verified generation.
- Observed behavior: FileStore starts from an empty map after JSON failure;
  ReAct logs a failed checkpoint load, resets messages, and continues. Neither
  path carries a corrupt-state barrier into subsequent writes.
- Impact: one later memory/checkpoint write can permanently erase the only copy
  of recoverable data and repeat external work from an apparently fresh session.
- Root cause: persistence APIs return a usable empty authority instead of a
  typed `Missing | Valid(generation) | Corrupt(evidence)` admission result.
- Direction: add a generic fail-closed admission/quarantine contract and write
  generation/CAS precondition, reusing FileConversationStore's typed corrupt
  error pattern. Do not add EKO-only policy to framework stores; delete
  parse-error-to-empty fallbacks once callers handle typed admission.
- Regression validation: corrupt both files, attempt a normal mutation, and
  assert original bytes remain recoverable and no fresh generation is published
  until explicit recovery.
- Validation reports: [V03](../validations/X-STA-01/V03-01.md), [V04](../validations/X-STA-01/V04-01.md), [V08](../validations/X-STA-01/V08-01.md), [V09](../validations/X-STA-01/V09-01.md)

### X-STA-01-P1-02: A crash-truncated TaskRuntime tail makes the valid append-only history unreadable

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_shadow.rs:105`; `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_shadow.rs:114`; `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_shadow.rs:361`; `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_shadow.rs:369`
- Reachability: every Task mutation appends one JSONL line -> process dies after
  writing a nonempty prefix of the final line -> boot/list/recovery calls
  `read_events` -> decoding that tail returns an error for the entire vector.
- Expected invariant: an append-only authority returns the last complete prefix,
  records/quarantines the partial tail, and does not silently accept malformed
  complete interior lines.
- Observed behavior: the writer comment says a crash can lose the last partial
  line and a future hardening pass will truncate it, but `read_events` attempts
  to decode every nonempty line and errors on the tail. Only empty tails are
  skipped.
- Impact: one interrupted append can hide all earlier run/task/Subagent/artifact
  facts, preventing boot recovery and making a resumable run appear unavailable.
- Root cause: append durability and recovery parsing were designed separately;
  the reader has no EOF-tail classification or repair receipt.
- Direction: distinguish final partial EOF bytes from corrupt committed lines,
  retain the valid prefix, quarantine/truncate only under a durable recovery
  protocol, and delete the inaccurate comment once behavior is implemented.
- Regression validation: crash after each byte of a UTF-8 JSONL final event and
  assert complete preceding events survive with monotonic `seq`; corrupt an
  interior line and assert fail-closed diagnostics.
- Validation reports: [V03](../validations/X-STA-01/V03-01.md), [V07](../validations/X-STA-01/V07-01.md), [V09](../validations/X-STA-01/V09-01.md)

### X-STA-01-P1-03: Recovery publishes cross-store state before validating and completing one generation

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent/src/agent/react/run/context.rs:230`; dependency `F-RCT-05-P1-02/P1-05`; `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:1631`; `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:1653`; `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:1706`; dependencies `A-STATE-01-P0-01/P1-03/P1-04` and `A-TSK-04-P1-02`
- Reachability: framework startup applies a checkpoint; GUI loads persistent
  transcript then restores a pooled Agent and frontend; EKO boot moves TaskRun
  to Paused then iterates task resets/blockers. Each path is a production restart
  path and exposes intermediate writes/errors.
- Expected invariant: recovery reads and validates the complete aggregate,
  stages all changes under one generation/receipt, then publishes once; retry is
  idempotent and knows whether publication completed.
- Observed behavior: checkpoint fields are applied additively, conversation UI,
  durable transcript, and pooled Agent lack a shared revision, and TaskRuntime
  persists Paused before task reset/blocker writes. A second crash or local error
  leaves a mixed generation; the next Task recovery scan ignores Paused runs.
- Impact: the user can see one transcript while the Agent reasons over another,
  or see a Paused TaskRun containing an orphaned Running claim that never becomes
  dispatchable again.
- Root cause: each subsystem has atomic files/locks, but the recovered product
  aggregate has no recovery generation, staged manifest, or completion receipt.
- Direction: keep generic checkpoint transaction semantics in framework; add an
  EKO recovery coordinator/manifest at the application boundary that validates
  identities, stages projections, commits once, and resumes incomplete receipts.
  Extend the existing authorities and delete field-wise/ordered publication,
  rather than adding parallel stores.
- Regression validation: inject failure/kill before and after every recovery
  mutation, restart twice, and assert either the old aggregate or one complete
  new generation with no Running claim under Paused.
- Validation reports: [V02](../validations/X-STA-01/V02-01.md), [V04](../validations/X-STA-01/V04-01.md), [V07](../validations/X-STA-01/V07-01.md), [V08](../validations/X-STA-01/V08-01.md), [V09](../validations/X-STA-01/V09-01.md)

### X-STA-01-P1-04: Revision and attempt identity stops before artifact lineage and current frontend selection

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs:1415`; `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs:1434`; `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:1432`; `echo-agent-cli/web-frontend/src/stores/subagentRunStore.ts:10`; `echo-agent-cli/web-frontend/src/stores/subagentRunStore.ts:407`; `echo-agent-cli/web-frontend/src/stores/subagentRunStore.ts:416`; dependency `A-FE-02-P1-01/P1-02`
- Reachability: TaskRuntime dispatch creates exact execution IDs -> Subagent
  retries/revisions append release/artifact facts -> restart hydration replays all
  events -> message Task UI selects one current execution and its outputs.
- Expected invariant: `(conversation, run, plan_revision, task, attempt,
  execution)` remains typed through every produced artifact and projection;
  current selection compares revision before attempt and terminal duplicates may
  enrich the same identity.
- Observed behavior: Subagent execution IDs contain revision/attempt, but
  RuntimeArtifact carries only artifact/run/optional task and is appended with
  no `step_id`; producer attempt is at best untyped metadata. The frontend keeps
  the opaque ID but groups `(run, task)` and compares only its attempt suffix;
  an initial terminal also blocks later durable enrichment.
- Impact: after a revised plan or retry, an older high attempt and its artifacts
  can appear current while the real newer revision is running; restart timing
  can change which terminal details remain visible.
- Root cause: a typed backend identity is serialized as an opaque string and
  partial joins, while artifact lineage never requires the producer execution.
- Direction: add typed plan revision, attempt, and producer execution identity
  to the existing event/artifact/frontend contracts; compare the tuple directly
  and monotonically enrich identical terminal facts. Delete suffix parsing and
  free-form producer metadata after lossless migration. Atomic frontend reducer
  repair remains owned by `A-FE-02`.
- Regression validation: emit artifacts from revision 3 attempt 5 and revision 4
  attempt 1 in both orders, restart, and assert revision 4 is current while both
  exact histories and artifact lineages remain inspectable.
- Validation reports: [V01](../validations/X-STA-01/V01-01.md), [V06](../validations/X-STA-01/V06-01.md), [V07](../validations/X-STA-01/V07-01.md), [V08](../validations/X-STA-01/V08-01.md), [V09](../validations/X-STA-01/V09-01.md)

### X-STA-01-P1-05: Conversation deletion has no tombstone or complete retention cascade

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/src/tauri/commands/conversations.rs:586`; `echo-agent-cli/src/tauri/commands/conversations.rs:595`; `echo-agent-cli/src/tauri/commands/conversations.rs:600`; `echo-agent-cli/src/tauri/commands/conversations.rs:619`; `echo-agent-cli/src/tui/events.rs:3067`; `echo-agent-cli/src/tui/events.rs:3071`; `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:1432`; `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:1800`; dependency `A-STATE-01-P1-05`
- Reachability: GUI/TUI user deletes a persisted conversation -> cleanup removes
  selected records/scopes -> TaskRuntime lookup and frontend stores remain keyed
  by the same conversation/run -> a late finalizer or later navigation/restart
  observes/writes remaining identities.
- Expected invariant: deletion durably fences late writers with a conversation
  generation/tombstone and idempotently cascades checkpoint, TaskRun/Subagent,
  tool execution, input/tool/RuntimeArtifact bodies, and frontend projections
  according to one declared retention policy.
- Observed behavior: GUI removes conversation, tool details, tool artifact, and
  user-input scopes; TUI omits tool-detail removal. Neither removes runtime
  checkpoint or TaskRuntime runs/events/artifact facts, whose store has list/add
  but no delete API. Frontend deletion clears chat/tool state but does not clear
  TaskRuntime/Subagent history. RuntimeArtifact retention is free-form metadata,
  not executable policy, and no tombstone fences late writers.
- Impact: deletion can resurrect the conversation (dependency-owned active-turn
  race), leave historical Task/Subagent/artifact data indefinitely reachable or
  orphaned, and behave differently between GUI and TUI.
- Root cause: ownership is distributed across stores/scopes with best-effort
  cleanup, but no canonical conversation aggregate or durable deletion record.
- Direction: implement an EKO application-level idempotent deletion coordinator
  over existing store APIs, add missing delete-by-conversation/run operations and
  typed retention, publish a tombstone/generation before cleanup, and make GUI,
  TUI, CLI, and channels call the same service. Delete per-surface cleanup logic
  after cutover.
- Regression validation: delete during active and idle states from every
  surface, crash after each cascade step, restart, and assert no late write can
  resurrect the generation and every retain/delete decision is identical.
- Validation reports: [V05](../validations/X-STA-01/V05-01.md), [V07](../validations/X-STA-01/V07-01.md), [V08](../validations/X-STA-01/V08-01.md), [V09](../validations/X-STA-01/V09-01.md)

## Positive Conclusions

- `FileConversationStore` uses unique temp files, file and parent-directory
  sync, atomic rename, fail-closed decode, and startup metadata reconciliation,
  preserving conversation/message IDs across a record/meta crash boundary.
- TaskRuntime normal claims carry exact revision/attempt identity and check the
  current claim before terminal status writes; completed Subagent results are
  recovered only for the exact execution ID.
- Frontend conversation and TaskRuntime async loading use generation guards;
  TaskRuntime also resets the event cursor across run switches.
- Subagent Zustand storage keeps retry executions under distinct full execution
  IDs; the defect is current selection/enrichment, not historical record keying.

## Validation Matrix

| ID | Claim | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Inputs, commits, dependency and dirty-source isolation | yes | passed | [V00](../validations/X-STA-01/V00-01.md) |
| V01 | Cross-domain identity authority table and duplicate search | yes | failed/finding | [V01](../validations/X-STA-01/V01-01.md) |
| V02 | Definition, composition-root registration, and real reachability | yes | passed | [V02](../validations/X-STA-01/V02-01.md) |
| V03 | Corrupt/full/partial-file admission matrix | yes | failed/findings | [V03](../validations/X-STA-01/V03-01.md) |
| V04 | Static crash-point and recovery publication matrix | yes | failed/finding | [V04](../validations/X-STA-01/V04-01.md) |
| V05 | Retention, deletion, and artifact cascade | yes | failed/finding | [V05](../validations/X-STA-01/V05-01.md) |
| V06 | Frontend restore identity and monotonic projection | yes | failed/deduplicated | [V06](../validations/X-STA-01/V06-01.md) |
| V07 | Existing test inventory against edge-case matrix | yes | failed/gaps | [V07](../validations/X-STA-01/V07-01.md) |
| V08 | Dependency ownership and historical-claim classification | yes | passed | [V08](../validations/X-STA-01/V08-01.md) |
| V09 | Executable crash/restart/replay/delete fixtures | future | not_run | [V09](../validations/X-STA-01/V09-01.md) |
| V10 | Exact links, headers, IDs, isolation, and source state | yes | V10-01/V10-02 failed scripts; V10-03 passed | [V10](../validations/X-STA-01/V10-03.md) |
| V30 | Primary source-anchor and evidence-chain acceptance | yes | passed | [V30](../validations/X-STA-01/V30-01.md) |

## Historical Claim Status

| Dependency claim | Classification | Current evidence |
|---|---|---|
| `F-RCT-05-P0-04`: corrupt checkpoint resets and can overwrite | current | [V03](../validations/X-STA-01/V03-01.md) |
| `F-RCT-05-P1-02/P1-05`: additive restore and warning-only safe point | current | [V04](../validations/X-STA-01/V04-01.md) |
| `F-MEM-01-P0-01`: corrupt FileStore becomes empty and overwrites | current | [V03](../validations/X-STA-01/V03-01.md) |
| `A-STATE-01-P0-01/P1-03/P1-04/P1-05`: stale overwrite, split restore, reused Agent, deletion race | current | [V04](../validations/X-STA-01/V04-01.md), [V05](../validations/X-STA-01/V05-01.md) |
| `A-TSK-04-P1-02/P1-03`: recovery ordering and attempt reuse | current | [V04](../validations/X-STA-01/V04-01.md) |
| `A-FE-02-P1-01/P1-02`: revision-blind selection and terminal enrichment loss | current | [V06](../validations/X-STA-01/V06-01.md) |

## Coverage And Uncertainty

- This is pure static review by explicit instruction. No Cargo, rustc, frontend
  test/build, fixture, kill/restart, browser, or network command ran. V09 records
  future runtime evidence and does not block source-conclusive findings.
- Framework evidence is pinned to committed blobs because the live framework
  worktree was externally dirty. CLI `Cargo.lock` was excluded; no reviewed CLI
  source file was dirty at the pinned revision.
- Best-effort artifact deletion can vary by filesystem failure, but the absence
  of TaskRuntime/checkpoint cascade and tombstones is source-conclusive.
- No migration compatibility is required. Remediation must converge on existing
  file authorities and delete superseded per-surface/ad-hoc logic.

## Handoff

- Primary reviewer should independently verify V10, especially TaskRuntime
  partial-tail parsing, missing delete APIs, and exact report links, then move
  the task from `needs_evidence` only if the pinned source still matches.
- Suggested repair order: fail-closed corrupt admission and partial-tail
  recovery; aggregate recovery generation/receipt; typed execution/artifact
  identity; tombstoned deletion cascade. Preserve dependency ownership for the
  atomic fixes and merge duplicate roadmap items using the backlinks above.
- This report becomes stale if FileStore/checkpoint admission, TaskRuntime
  `read_events`/boot recovery, RuntimeArtifact identity, conversation deletion,
  or frontend Subagent selection changes.
- Dynamic acceptance belongs in the Q phase, especially `Q-E2E-01`; do not mark
  those future checks as already executed.
