# F-RCT-05: Steer, interrupt, snapshot, and resume

> Status: complete
> Reviewer: Codex primary reviewer
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d35c9a432370b923acabfbb5f32d63`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: framework source clean; concurrent generated EKO files were excluded and not modified

## Question

Can a running or interrupted Agent accept steering and resume from snapshots
without losing canonical state or replaying completed side effects?

## Scope

- `TurnSteerMailbox`, public/handle steer APIs, streaming and non-stream turn
  identity and drain points.
- Public `StateSnapshot`/`SnapshotManager` capture and rollback.
- `AgentCheckpoint`, `RuntimeStateStore`, file/SQLite backends, checkpoint
  creation, Execute-mode restore, message-pair validation, and TaskNode hydration.
- EKO GUI/TUI steer and session snapshot callers only to prove real reachability.
- Static definitions, callers, state-field mapping, failure paths, tests, history,
  panic/UTF-8/overflow inspection.

## Out Of Scope

- Tool batch scheduling mechanics owned by F-RCT-04; this task owns only their
  durable recovery consequences.
- Non-stream/stream terminal publication defects owned by F-RCT-02/F-RCT-03.
- Store-level FileStore/ConversationStore defects owned by F-MEM-01.
- Subagent team checkpoint semantics owned by F-SUB-02.
- EKO TaskRuntime recovery policy and UI reducer behavior.
- Source fixes, builds, tests, Cargo/rustc, or dynamic fixtures.

## Inputs

- Root AGENTS.md and shared/codex review protocols.
- Completed [F-RCT-02](F-RCT-02.md) and [F-MEM-01](F-MEM-01.md).
- Current source/tests and scoped git history. F-RCT-04 was not consumed because
  its delegated report was still incomplete; shared findings are backlinked by
  task boundary rather than copied.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Stable turn identity, cancellation/steering safe points, checkpoint schema/validation, exact restore, completed-effect identity, and fail-closed persistence are framework mechanisms. |
| EKO product policy | Which UI offers steer/branch/resume, workspace recovery prompts, and user decisions after a barrier remain application policy. |
| Adapter boundary | EKO supplies stable IDs and renders typed outcomes; it must not repair identity mismatches, infer completed effects, or implement a second checkpoint validator. |
| Duplicate search | Searched steer, snapshot, rollback, checkpoint, resume, restore, node hydration, active skills, plan, working directory, and blocked state across both repositories. Two framework “snapshot” authorities exist: message-only in-memory rollback and durable runtime checkpoint. |
| Migration deletion | Keep durable RuntimeStateStore as the execution-recovery authority. Either rename/restrict StateSnapshot to transcript branching or extend it to a truly typed state boundary; delete overbroad state-rollback claims. Use one canonical turn ID for mailbox begin/accept/drain. |

## Current Path

```text
AgentInvocationContext.runtime { run_id?, turn_id?, ... }
  -> run_stream_channel chooses mailbox id = turn_id.or(run_id)
  -> public steer(expected turn_id) -> mailbox pending FIFO
  -> AgentRunSnapshot stores current_run_id and current_turn_id separately
  -> run_core_loop drain_steer_into_context(current_run_id)

runtime checkpoint
  ContextManager messages + plan + active-skill names + blocked reason + cwd
  -> save_runtime_checkpoint (best effort, returns ())
  -> File/SQLite RuntimeStateStore latest checkpoint
  -> Execute-mode restore_thread_context
     -> resume_from_state_store
        -> replace messages
        -> optionally set plan/cwd
        -> add skill activation flags
        -> best-effort hydrate Running nodes
     -> continue a new ReAct run

message snapshot
  SnapshotManager capture(messages, iteration, free metadata)
  -> auto capture after complete tool batch/final answer, or manual GUI action
  -> rollback/rollback_to truncates snapshot ring
  -> ReactAgent replaces ContextManager messages only
```

Positive behavior is meaningful: file checkpoint writes are atomic and reject
corrupt JSON; `AgentCheckpoint::restore_messages` rejects missing, orphan,
duplicate, and name-mismatched tool results; a full successful tool batch is
checkpointed before the next model call; steer mailbox FIFO and turn mismatch
are typed; scoped production paths use character-safe strings and bounded
arithmetic.

## Findings

### F-RCT-05-P0-04: A corrupt durable checkpoint is reset and can be overwritten

- Priority: P0
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/state/file.rs:143-163`;
  `echo-agent/src/agent/react/run/context.rs:230-263`;
  `echo-agent/src/agent/snapshot.rs:570-631`
- Reachability: malformed/truncated checkpoint JSON is returned as Err by the
  file backend; Execute-mode restoration catches it on normal startup.
- Expected invariant: existing corrupt recovery data fails closed and is
  preserved/quarantined until explicit recovery.
- Observed behavior: restore logs the error, calls `reset_messages`, then starts
  a fresh run. A later best-effort checkpoint writes the new context to the same
  latest checkpoint path, atomically replacing the corrupt bytes.
- Impact: the last recoverable record of an interrupted session and its completed
  side-effect history can be permanently lost, while the user is shown a normal
  fresh execution.
- Root cause: “no checkpoint” and “checkpoint failed validation/read” converge to
  fresh-session behavior above an otherwise fail-closed store.
- Direction: propagate a typed corrupt-recovery barrier; preserve/quarantine the
  file and require explicit discard/recovery. Only NotFound may start fresh.
- Regression validation: truncated, malformed, unsupported-schema, paired-history
  error, and I/O failure followed by attempted execution/save; original bytes
  must remain and no run starts implicitly.
- Validation reports: [V05](../validations/F-RCT-05/V05-01.md)

### F-RCT-05-P1-01: Accepted steering can be stranded by run/turn identity mismatch

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/react/run/stream_channel.rs:104-123`,
  `:282-301`, `:328-344`; `echo-agent/src/agent/snapshot.rs:461-480`;
  `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:485-505`;
  `echo-agent-cli/src/tauri/commands/chat.rs:745-784`
- Reachability: EKO creates one active chat `turn_id` and can also carry an
  independent Task `run_id`; public steer targets the active turn ID.
- Expected invariant: mailbox begin, acceptance, and every drain use the same
  stable turn identity.
- Observed behavior: mailbox begins with `turn_id.or(run_id)`, but the snapshot
  drains only by `current_run_id`. If both are present and differ, steer returns
  accepted and queues under turn_id, while every drain asks for run_id and gets
  no messages. The pending input disappears when the lease drops.
- Impact: user guidance and attachments can be acknowledged but never influence
  the running Task-mode Agent.
- Root cause: correlation run identity was reused as the mailbox owner despite a
  dedicated `current_turn_id`.
- Direction: add one required effective turn identity to the invocation snapshot
  and use it for begin/accept/drain/finish. Never fall back to run_id once an
  explicit turn exists.
- Regression validation: distinct run/turn IDs with steer before think,
  mid-provider, before tool, and before finalization; assert insertion exactly
  once and no accepted message is left queued.
- Validation reports: [V02](../validations/F-RCT-05/V02-01.md)

### F-RCT-05-P1-02: Checkpoint restore is additive and non-atomic rather than exact

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/state/mod.rs:117-174`;
  `echo-agent/src/agent/react/mod.rs:1680-1746`;
  `echo-agent/src/agent/snapshot.rs:763-790`
- Reachability: every Execute-mode agent with a state store automatically calls
  this path before a run.
- Expected invariant: a validated checkpoint replaces every field it owns as one
  recoverable state, including empty/None values; failure changes nothing.
- Observed behavior: messages replace first, plan changes only when Some, skill
  names are only added, cwd changes only when Some, blocked_reason is only
  logged, and node hydration failures only warn. Old plan, skills/projections,
  cwd, and blocked semantics can survive a checkpoint that explicitly omits them;
  later failure leaves already-replaced messages and partially mutated state.
- Impact: resumed model tools/instructions/workspace and UI recovery facts can
  describe a hybrid of the current Agent and an older checkpoint.
- Root cause: public checkpoint fields are hydrated as independent best-effort
  setters rather than one validated replacement transaction.
- Direction: deserialize and validate a complete typed restore candidate, resolve
  skill definitions and working directory, then atomically replace all owned
  state. Empty/None must clear. Return a recovery barrier for any failed field.
- Regression validation: dirty live agent plus checkpoint with None/empty values,
  unknown skill, invalid cwd, node-store failure, and cancellation between each
  step; assert all-old or all-new state.
- Validation reports: [V03](../validations/F-RCT-05/V03-01.md)

### F-RCT-05-P1-03: Public rollback rewinds messages but not completed side effects

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/snapshot.rs:24-37`, `:56-145`;
  `echo-agent/src/agent/react/mod.rs:1504-1555`;
  `echo-agent/src/agent/snapshot.rs:1114-1141`;
  `echo-agent/src/agent/react/run/phases/tools.rs:425-440`;
  `echo-agent-cli/src/tauri/commands/session.rs:57-119`
- Reachability: automatic snapshots are captured after completed tool batches;
  GUI can manually snapshot and restore; framework docs advertise state rollback.
- Expected invariant: an execution-state rollback either reverts/retains explicit
  side-effect facts so completed writes are not silently replayed, or is clearly
  a transcript branch with no such guarantee.
- Observed behavior: StateSnapshot contains messages and free metadata only.
  ReactAgent rollback replaces only messages and truncates later snapshots. It
  does not reconcile durable checkpoint/nodes, plan, skills, cwd, memory,
  files/artifacts, tool trace, cancellation, or concurrent execution. A later
  model sees an earlier conversation and can repeat already completed effects.
- Impact: a GUI “restore checkpoint” can lead to repeated file/shell/network
  effects and disagreement with durable resume state.
- Root cause: a conversation-history helper is presented as an Agent state
  rollback API beside the real RuntimeStateStore authority.
- Direction: rename/restrict it to transcript branching and make side-effect
  consequences explicit, or route recovery through a typed durable checkpoint
  plus compensating/observed effect policy. Do not create a third store.
- Regression validation: snapshot after write, rollback, continue, restart,
  concurrent active run, and plan/skill/cwd change; assert explicit no-replay or
  compensation semantics.
- Validation reports: [V01](../validations/F-RCT-05/V01-01.md),
  [V04](../validations/F-RCT-05/V04-01.md)

### F-RCT-05-P1-05: Required safe-point checkpoint failures are warning-only

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/snapshot.rs:570-631`;
  `echo-agent/src/agent/react/run/phases/tools.rs:425-447`;
  `echo-agent/src/agent/react/run/phases/compact.rs:23-38`;
  `echo-agent/src/agent/react/run/phases/finalize.rs:68-83`
- Reachability: completed tool batches, cancellation/errors, pre-compression, and
  terminal paths all call the helper.
- Expected invariant: at safe points whose purpose is to prevent replay of a
  completed side effect, persistence failure blocks advancement or produces a
  typed degraded-recovery outcome.
- Observed behavior: serialization/store errors only warn; the helper returns
  unit and every caller continues. In particular, after a successful write tool
  the next LLM call proceeds even if the checkpoint did not persist its matching
  tool result. A crash can restore the previous checkpoint and repeat the write.
- Impact: “checkpoint before next model call” does not provide its documented
  no-replay guarantee under disk/full/permission/store faults.
- Root cause: one best-effort telemetry-style API is used for optional periodic
  snapshots and correctness-critical effect commits.
- Direction: separate optional snapshots from required safe-point commits.
  Required commits return typed Result and gate progression/terminal publication;
  preserve the last valid checkpoint and surface recovery uncertainty.
- Regression validation: fail each save phase after a write/dangerous tool,
  cancel race, terminal save, and successful retry; assert no next model request
  and no duplicate effect after restart.
- Validation reports: [V05](../validations/F-RCT-05/V05-01.md)

### F-RCT-05-P2-06: Two public snapshot contracts make incompatible state claims

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/memory.rs:11-39`;
  `echo-agent/src/lib.rs:201-202`; `echo-agent/README.md:228`, `:463`;
  `echo-agent/echo-state/src/memory/snapshot.rs:24-37`;
  `echo-agent/src/state/mod.rs:114-134`
- Reachability: both types are facade-visible; docs/examples and EKO expose the
  message snapshot as general checkpoint/rollback while RuntimeStateStore is
  separately described as full runtime recovery.
- Expected invariant: distinct public contracts have precise non-overlapping
  names and guarantees.
- Observed behavior: StateSnapshot is only in-memory messages/metadata and
  RuntimeStateStore is the durable execution path, yet both use state/snapshot/
  checkpoint/restore language. Neither one alone restores all claimed state.
- Impact: framework consumers can select message rollback expecting crash or
  side-effect safety, and maintainers may extend the wrong authority.
- Root cause: conversation snapshots predate durable checkpoints and retained
  their broad public positioning.
- Direction: make RuntimeStateStore the sole execution-recovery authority and
  rename StateSnapshot as transcript/context branch if retained as a reasonable
  public option. Delete stale broad docs, not the valid capability itself.
- Regression validation: public API/docs inventory and a field-by-field contract
  table proving each retained type's precise boundary.
- Validation reports: [V01](../validations/F-RCT-05/V01-01.md),
  [V04](../validations/F-RCT-05/V04-01.md),
  [V07](../validations/F-RCT-05/V07-01.md)

## Validation Matrix

| ID | Claim | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Protocol, scope, layering, de-duplication | yes | passed | [V00](../validations/F-RCT-05/V00-01.md) |
| V01 | Definition, export, duplicate and live-caller inventory | yes | passed with duplicate contract | [V01](../validations/F-RCT-05/V01-01.md) |
| V02 | Steer identity and drain mapping | yes | failed invariant | [V02](../validations/F-RCT-05/V02-01.md) |
| V03 | Durable checkpoint field round-trip/replacement | yes | failed invariant | [V03](../validations/F-RCT-05/V03-01.md) |
| V04 | Message rollback and completed-effect safety | yes | failed invariant | [V04](../validations/F-RCT-05/V04-01.md) |
| V05 | Corruption and required-save failure behavior | yes | failed invariants | [V05](../validations/F-RCT-05/V05-01.md) |
| V06 | Existing tests and panic/UTF-8/overflow inventory | yes | passed static inventory | [V06](../validations/F-RCT-05/V06-01.md) |
| V07 | History/document drift | yes | passed with stale claims | [V07](../validations/F-RCT-05/V07-01.md) |
| V08 | Dynamic recovery fixture matrix | no by review rule | not_run | [V08](../validations/F-RCT-05/V08-01.md) |
| V99 | Report integrity and source isolation | yes | passed | [V99](../validations/F-RCT-05/V99-01.md) |

## Historical Claim Status

| Claim | Classification | Evidence |
|---|---|---|
| File RuntimeStateStore corrupt JSON is an error and atomic writes preserve complete files | current at backend boundary | V05 |
| Completed tool batch is checkpointed before next LLM call so effects are not replayed | partial/regressed under save failure | P1-05 |
| RuntimeStateStore is a full runtime checkpoint | partial | Schema has multiple fields; hydration is additive/non-atomic, P1-02 |
| SnapshotManager captures/restores Agent state at any point | stale/overbroad | Only messages and metadata, P1-03/P2-06 |
| Same-turn steering preserves FIFO and rejects mismatch | current for isolated equal-ID case | Distinct run/turn integration fails, P1-01 |

## Coverage And Uncertainty

- No dynamic execution was run. V08 records the future matrix. Static identity,
  field-mapping, absent error propagation, and rollback boundaries are
  source-conclusive.
- SqliteRuntimeStateStore also silently defaults malformed active-skills JSON and
  timestamps, but this report does not add a separate finding because the
  broader exact-restore/corruption authority should be fixed once.
- Cancellation does not have a standalone interrupt/checkpoint API in this path;
  cancel behavior at tool batches/streams is owned by F-RCT-02/03/04.
- Message pairing validation is a positive invariant, but it treats any paired
  model transcript as completed-effect history; no external idempotency key or
  effect ledger exists.
- No new scoped production panic, UTF-8 slicing, or arithmetic defect was found.
  Unwraps in the inspected SnapshotManager section are test-only.

## Handoff

- F-RCT-04 must make partial batch checkpoints resumable or explicitly
  non-resumable; it must not assume best-effort save provides a safe point.
- F-SUB-02 team recovery should use the durable checkpoint authority and avoid
  copying message rollback semantics.
- Application surfaces should render typed recovery barriers; they must not
  silently reset corrupt checkpoints or claim transcript rollback reverts files.
- Remediation order: fail closed on corruption and required-save failure, fix
  exact restore, unify turn identity, then separate transcript branching from
  durable execution recovery.
- This report becomes stale if mailbox identity, AgentInvocationContext,
  StateSnapshot/SnapshotManager, AgentCheckpoint/RuntimeStateStore,
  save_runtime_checkpoint, resume_from_state_store, Execute preparation, or
  EKO steer/session adapters change.
