# F-RCT-05: Steer, interrupt, snapshot, and resume

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: not-applicable (framework-only task)
> Worktree state: clean

## Question

Can a running or interrupted Agent resume without replaying completed
side effects or losing canonical context?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/src/agent/steer.rs` (199 lines) — `TurnSteerMailbox`,
  `ActiveTurn`, `ActiveTurnLease`: same-turn input injection (steer)
  with FIFO queue, turn-id match, steerable gate, RAII-scoped
  lifecycle.
- `echo-agent/src/agent/snapshot.rs` (1678 lines) — `AgentRunSnapshot`
  (live runtime holder), `RuntimeConfig`, `ToolRuntime`,
  `GuardRuntime`, `save_runtime_checkpoint` (`:570-631`),
  `save_transcript_projection` (`:648-716`), `finalize_run`
  (`:548-559`), `hydrate_running_nodes` (`:763-796`), `auto_snapshot`
  (`:1116-1141`), `check_tool_approval` (`:798-869`).
- `echo-agent/echo-state/src/memory/snapshot.rs` (305 lines) —
  `StateSnapshot`, `SnapshotPolicy`, `SnapshotManager` (in-memory ring
  buffer for message-history rollback; not persisted).
- `echo-agent/src/state/mod.rs` (361 lines) — `AgentCheckpoint`,
  `restore_messages`, `validate_tool_message_pairing`,
  `completed_tool_call_ids`, `RuntimeStateStore` trait,
  `TaskNode`/`TaskNodeStatus`.
- `echo-agent/src/state/file.rs` (368 lines) — `FileRuntimeStateStore`:
  corrupt-JSON-as-error, atomic write, path-safe ids.
- `echo-agent/src/agent/react/mod.rs:1680-1755` —
  `resume_from_state_store` (the cross-process resume body).
- `echo-agent/src/agent/react/run/context.rs:216-261` —
  `reset_messages`, `restore_thread_context` (resume entry + error
  fallback).
- `echo-agent/src/agent/react/run/direct.rs` (46 lines) — `run_direct`
  (calls restore) vs `run_chat_direct` (does NOT call restore).
- `echo-agent/src/agent/react/run/stream_channel.rs:483-756` —
  `run_core_loop` (iteration boundaries, drain_steer sites, checkpoint
  call sites).
- `echo-agent/src/agent/react/run/phases/think.rs:26-103, 260-351` —
  think-start intervention cancel/block + cancel-aware LLM stream.
- `echo-agent/src/agent/react/run/phases/tools.rs:130-443` — concurrent
  + serial cancellation, 5 s grace, batch-timeout asymmetry.
- `echo-agent/src/agent/react/run/phases/compact.rs:21-116` — compact
  phase checkpoint ordering (save before mutate).
- `echo-agent/src/agent/react/builder.rs:87-88, 167-168, 1006-1008` —
  `snapshot_policy` default `None` (SnapshotManager is opt-in).

## Out Of Scope

Deferred to named task IDs:

- The streaming-entry-specific cancellation/steer tests
  (`stream_channel.rs:757-2161` test module) — F-RCT-02 sampled these
  for terminal/error behavior; this task references them only for the
  interrupt-point characterization.
- Batch-timeout asymmetry (`tools.rs:284-292` skipping checkpoint +
  `ToolBatchEnd`) — already filed as F-RCT-04-P2-01; this task
  references it only for its resume consequence (V02 Deviation 1, V03
  Deviation 2).
- Trace-finalization gap on abandoned/cancelled/blocked arms — already
  filed as F-RCT-02-P3-01; this task references it only for the
  think-phase interrupt characterization.
- Application-layer (echo-agent-cli) conversation load UI
  (`WelcomeScreen.handleResume` → `loadConversation`) — that path
  loads the user-visible transcript from `ConversationStore` for
  display, which is a separate concern from the framework's
  `RuntimeStateStore` resume. Whether/how the app layers the two is an
  application task.
- `SqliteRuntimeStateStore` (`sqlite` feature) — same trait, different
  backend; its concurrency/schema is out of scope per AGENTS.md
  (CLI does not enable SQLite).

## Inputs

Required repository documents read:

- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/AGENTS.md` (in full
  via system reminder — especially the framework-vs-application
  layering gate, the "first check if it already exists" rule, the
  no-panic / UTF-8 safety rules, and the prompt-driven-over-state-
  machine guidance informed by the Claude Code / Codex research).
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/REPORTING.md`
  (in full).
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/templates/task-report.md`
  and `templates/validation-report.md` (in full).

Dependency task reports read:

- `docs/comprehensive-review/zcode-glm/tasks/F-RCT-02.md` (in full).
  Establishes: `run_core_loop` is the single loop body; the four
  terminal `finalize_*` helpers + abandoned/cancelled/blocked
  short-circuits are exhaustive; `finalize_completed_run` and the
  abandoned arms skip `finalize_run` (trace gap). F-RCT-05 consumes
  the loop-body transition model and the terminal-arm enumeration.
- `docs/comprehensive-review/zcode-glm/tasks/F-RCT-04.md` (in full).
  Establishes: tool-batch cancellation has a 5 s grace; batch-timeout
  does NOT save a checkpoint or emit `ToolBatchEnd` (F-RCT-04-P2-01);
  concurrent results push in completion order. F-RCT-05 references
  these for the interrupt-point characterization (V03) and the
  resume-edge case (V02 Deviation 1).
- `docs/comprehensive-review/zcode-glm/tasks/F-MEM-01.md` (in full).
  Establishes: `SnapshotManager` is in-memory only, not persisted;
  `FileStore` silently swallows corrupt JSON (F-MEM-01-P1-01);
  `FileConversationStore` surfaces corruption as error.
  F-RCT-05-P2-02 compounds F-MEM-01-P1-01's theme (silent corruption
  handling) on the resume consumer side.

Historical documents treated as hypotheses:

- `echo-agent/src/agent/snapshot.rs:1-6` module docstring claiming
  `AgentRunSnapshot` "captures agent state for `'static` streaming"
  via Arc-composition. Treated as **current** — verified by V01.
- `echo-agent/echo-state/src/memory/snapshot.rs:1-13` module docstring
  (Chinese) claiming the snapshot is for in-run rollback to a
  known-good state. Treated as **current but narrowly scoped** —
  verified by V01; the struct name `StateSnapshot` oversells the
  scope (messages only), filed as F-RCT-05-P3-02.
- `echo-agent/src/state/file.rs:14-19` docstring claiming "Corrupt
  JSON is an error ... rather than silently returning None". Treated
  as **current at the store layer** — verified by V04; but the resume
  consumer (`restore_thread_context`) suppresses the error, so the
  end-to-end behavior is silent (F-RCT-05-P2-02).
- `echo-agent/src/agent/react/mod.rs:1671-1679` docstring on
  `resume_from_state_store` claiming it "Loads the most recent
  AgentCheckpoint ... deserializes the saved messages, and restores
  them into the context manager." Treated as **current** — verified
  by V01/V02.

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | Yes. `TurnSteerMailbox` (same-turn steer), `AgentRunSnapshot` (runtime holder), `AgentCheckpoint` + `RuntimeStateStore` (cross-process resume), `validate_tool_message_pairing` (replay invariant), `SnapshotManager` (in-run rollback), and the checkpoint call sites in the phase functions are generic agent-runtime machinery any `echo-agent` consumer needs. They live correctly: the steer mailbox, run snapshot, and checkpoint logic in `echo-agent` (root crate); `StateSnapshot`/`SnapshotManager` in `echo-state`; the `RuntimeStateStore` trait + `FileRuntimeStateStore` in `echo-agent::state`. |
| EKO product policy | None at this layer. The framework resume takes pure framework inputs (`state_store`, `conversation_id`, `snapshot_manager`, `cancel_token`, intervention callbacks). EKO product policy (which conversations to resume, how to alert the user on corruption, whether to enable the in-memory snapshot) enters only through the injected `RuntimeStateStore` and the caller's choice to call `run_direct` vs `run_chat_direct`. |
| Adapter boundary | `restore_thread_context` is the thin seam: it tries `resume_from_state_store`, on success restores messages+plan+skills+working_dir, on Err falls back to `reset_messages`. The application decides whether to wire a `state_store` and whether to surface the Err; the framework currently swallows the Err (F-RCT-05-P2-02). |
| Duplicate search | Searched names: `StateSnapshot`, `SnapshotManager`, `SnapshotPolicy`, `AgentCheckpoint`, `RuntimeStateStore`, `FileRuntimeStateStore`, `resume_from_state_store`, `restore_thread_context`, `restore_messages`, `validate_tool_message_pairing`, `completed_tool_call_ids`, `save_runtime_checkpoint`, `auto_snapshot`, `TurnSteerMailbox`, `ActiveTurnLease`, `hydrate_running_nodes`, `CheckpointResumed`. Result: one canonical definition per concept. Two distinct snapshot types (`StateSnapshot` in-memory vs `AgentCheckpoint` persisted) with cleanly separated scopes — not a duplicate. |
| Migration deletion | No deletion proposed. The two snapshot types serve different purposes (in-run rewind vs cross-process resume); consolidating them would violate the separation. |

## Current Path

Verified resume/steer/snapshot call graph at commit `9b0e0fa`:

```text
CROSS-PROCESS RESUME (RuntimeStateStore path)
─────────────────────────────────────────────
Agent::execute(task) / stream entry (StreamMode::Execute)
   ↓
ReactAgent::run_direct(task)                                [direct.rs:9]
   │  restore_thread_context().await                        [:11]
   │      state_store.is_some()?                            [context.rs:235]
   │      resume_from_state_store().await                   [context.rs:236, react/mod.rs:1680]
   │          store.get_checkpoint(conv_id)                 [react/mod.rs:1689]
   │          cp.restore_messages()?                        [:1691]
   │              serde_json::from_str(messages_json)?      [state/mod.rs:158]
   │              validate_tool_message_pairing(&messages)? [:166]
   │          set_messages(messages)                        [react/mod.rs:1694]
   │          plan_state ← cp.current_plan                  [:1697-1700]
   │          skill_registry.mark_activated(name) per skill [:1703-1705]
   │          hydrate_running_nodes() (Running → Hydrated)   [:1714-1715]
   │          set_working_dir(cp.working_dir)               [:1718-1720]
   │          record CheckpointResumed trace event          [:1731-1737]
   │      ── on Err: warn! + reset_messages()               [context.rs:245-248]  ★ F-RCT-05-P2-02
   │  run_react_loop(task)                                  [direct.rs:23]

Agent::chat(message) / stream entry (StreamMode::Chat)
   ↓
ReactAgent::run_chat_direct(message)                        [direct.rs:29]
   │  (NO restore_thread_context call)                      ★ F-RCT-05-P2-01
   │  run_react_loop(message)                               [:42]

IN-RUN SNAPSHOT (SnapshotManager path; opt-in, never persisted)
──────────────────────────────────────────────────────────────
run_core_loop iteration:
   after tool batch / at finalize ── snap.auto_snapshot(context, iteration)
                                     [tools.rs:434, finalize.rs:150]
                                     snapshot_manager.should_capture(iteration)?
                                     [snapshot.rs:1121-1128]
                                     if Some(mgr): mgr.capture(iteration, messages)  [:1138]
   ReactAgent::rollback(steps_back) / rollback_to(id)       [react/mod.rs:1520-1555]
       mgr.rollback(steps_back) → set_messages(snapshot.messages)

SAME-TURN STEER (TurnSteerMailbox path)
───────────────────────────────────────
run_react_loop ── active_turn_lease = mailbox.begin(turn_id)  [react_loop.rs:620]
run_core_loop iteration top ── drain_steer_into_context       [stream_channel.rs:538]
run_core_loop after think   ── drain_steer_into_context       [:663]
external caller ── mailbox.steer(turn_id, message)           [steer.rs:72]
    has_content? turn_id match? steerable?  → pending.push_back
loop sets steerable=true/false at safe points                [steer.rs:63-70]
loop task ends ── ActiveTurnLease::drop ── mailbox.finish     [steer.rs:143-147]

CHECKPOINT CADENCE (save_runtime_checkpoint sites)
──────────────────────────────────────────────────
compact.rs:35    every iteration, before ContextManager::prepare (full context)
tools.rs:203     concurrent cancel-grace elapsed (partial batch)
tools.rs:258     concurrent per-result cancel-observed
tools.rs:296     concurrent post-loop cancel-observed
tools.rs:309     serial pre-iteration cancel
tools.rs:336     serial cancel-grace elapsed
tools.rs:414     serial tool error
tools.rs:419     serial post-exec cancel-observed
tools.rs:429     post-batch normal completion ("every call has a result")
tools.rs:439     periodic (react_checkpoint_interval)
finalize.rs:79   finalize_completed_run (tool branch)
finalize.rs:164  emit_final_text (text branch)
finalize.rs:251  finalize_max_iterations
```

Key invariants verified (full evidence in V01–V04):

- **Two snapshot types, separated scopes.** `StateSnapshot`
  (echo-state) is in-memory, message-only, opt-in, never persisted.
  `AgentCheckpoint` (echo-agent::state) is persisted, carries
  messages+plan+skills+working_dir+blocked_reason. They are not
  duplicates (V01).
- **Resume = trajectory + new turn.** `resume_from_state_store`
  restores the message trajectory and product metadata; it does NOT
  restore budget counters, iteration number, trace IDs, or
  recently-read-files — those are recomputed for the new turn. This
  matches B-REF-01's convergence finding (Claude Code: checkpoint +
  rewind into new turn, not event replay) (V01).
- **Replay protection is structural.** `validate_tool_message_pairing`
  rejects orphan/duplicate/mismatched tool results before the
  trajectory is loaded. `completed_tool_call_ids` is trace-only, not a
  gate. Checkpoint placement keeps the on-disk trajectory paired on
  the normal paths (V02).
- **Three interrupt classes, positioned checkpoints.** Think-phase
  forwards error + exits; tool-phase cancel drains 5 s + checkpoints
  (timeout does not — F-RCT-04-P2-01); compact-phase has no internal
  cancel but is crash-safe by checkpoint-before-mutate. Steer is
  cooperative, turn-scoped, never preempts (V03).
- **Detection works; consumption swallows.** The store and validator
  layers surface corruption as Err; the resume consumer
  (`restore_thread_context`) catches all Err and silently resets to
  empty context, then the next turn overwrites the corrupt file (V04).

## Findings

### F-RCT-05-P2-01: Chat-mode turns never restore from RuntimeStateStore — cross-process chat resume silently starts from empty context

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/react/run/direct.rs:29-46` —
    `run_chat_direct` does NOT call `restore_thread_context`; it goes
    straight to `run_react_loop`. Comment at `:28`: "Multi-turn
    conversation: do not reset context, append message then enter
    ReAct loop".
  - `echo-agent/src/agent/react/run/context.rs:500-507` — the
    streaming entry's `prepare_stream_context` only calls
    `restore_thread_context` when `mode == StreamMode::Execute`; the
    `StreamMode::Chat` arm is explicitly empty with comment "Multi-turn
    chat mode: do not reset context".
  - `echo-agent/src/agent/react/run/context.rs:569-573` — the
    multimodal variant `prepare_stream_context_with_message` has the
    same Execute-only guard.
  - `echo-agent/src/agent/react/run/react_loop.rs:603` —
    `prepare_react_context` (the shared non-streaming prep) does not
    call restore at all; restore happens only via the
    `run_direct`/`prepare_stream_context` Execute paths above.
- Reachability: every multi-turn chat invocation after a process
  restart. The application (echo-agent-cli) wires a `RuntimeStateStore`
  during bootstrap (`echo-agent-app-core/src/runtime.rs:80-82`), so
  `state_store.is_some()` is true on the live path. But a chat turn
  after restart skips `restore_thread_context`, so the in-memory
  `ContextManager` (empty in a fresh process) is used directly. The
  on-disk checkpoint exists but is ignored.
- Expected invariant: if a `RuntimeStateStore` is configured and a
  checkpoint exists for the `conversation_id`, every entry mode that
  continues a conversation should restore the trajectory — otherwise
  the persistence is wasted for the chat path. At minimum, the
  chat-vs-execute asymmetry should be documented as intentional.
- Observed behavior: a user who closes and reopens the app, then sends
  a chat message in an existing conversation, gets a response computed
  from an empty context (just the new message + system prompt). The
  prior conversation history — which is on disk in the checkpoint — is
  invisible to the model. The `save_runtime_checkpoint` calls during
  the new chat turn will then write a checkpoint containing only the
  new (context-less) exchange, overwriting the richer pre-restart
  checkpoint.
- Impact: silent loss of conversation context on the chat path after a
  restart. For a local desktop assistant (EKO) where restarts are
  common (app close/open, crash recovery), this means multi-turn chat
  does not actually resume across restarts despite the framework
  having the machinery to do so. The Execute path (one-shot tasks)
  resumes correctly; only the Chat path is broken. The product
  positioning in AGENTS.md (TUI/GUI feature parity, Claude-Code-like
  continuity) implies chat resume should work.
- Root cause: the chat path was written assuming the in-memory
  `ContextManager` persists across turns within one process (true for
  a long-running session). Cross-process resume was added later via
  `restore_thread_context`, but the call was placed only on the
  Execute path, not the Chat path. The "do not reset context" comment
  reflects the within-process intent, not the cross-process reality.
- Direction: two options.
  (a) **Restore on chat too, conditionally**: in
  `run_chat_direct` and the `StreamMode::Chat` arms of
  `prepare_stream_context(_with_message)`, call
  `restore_thread_context` when the in-memory context is empty
  (detected via `context.lock().await.messages().is_empty()` or a
  "first turn of process" flag). This preserves the within-process
  behavior (no redundant reload when context is already populated)
  while enabling cross-process chat resume.
  (b) **Document the asymmetry**: if chat resume is intentionally the
  application's job (the app loads `ConversationStore` history and
  calls `load_messages`), add a doc comment on `run_chat_direct`
  stating that the framework does NOT restore on chat and the
  application must populate the context via `load_messages` first.
  Then verify echo-agent-cli actually does this on the chat path.
  Prefer (a) unless the application layer has an explicit chat-load
  flow that makes the framework restore redundant — a grep of
  echo-agent-cli did not surface such a flow on the live chat path
  (only the `WelcomeScreen.handleResume` → `loadConversation` UI flow,
  which loads for display, not for the model's context).
- Regression validation: a test that (1) runs a chat turn that
  checkpoints, (2) drops the agent / simulates a fresh process
  (re-create `ReactAgent` with the same `conversation_id` +
  `state_store`), (3) runs a second chat turn, and asserts the second
  turn's context contains the first turn's messages. Today this test
  would fail; after the fix it should pass.
- Validation reports: [V01](../validations/F-RCT-05/V01-01.md).

### F-RCT-05-P2-02: Corrupt or schema-incompatible checkpoint is silently swallowed and then destroyed (compounds F-MEM-01-P1-01)

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/react/run/context.rs:245-248` — the resume
    error arm:
    ```rust
    Err(e) => {
        warn!(agent = %agent, error = %e, "⚠️ Failed to load RuntimeStateStore checkpoint, starting from empty context");
        self.reset_messages().await;
    }
    ```
    No error propagated to the caller; no event emitted; no user
    signal.
  - `echo-agent/src/agent/react/run/context.rs:216-220` —
    `reset_messages` clears the context and pushes only the system
    prompt.
  - `echo-agent/src/state/file.rs:170-187` — `save_checkpoint` is an
    unconditional `atomic_write` (upsert/replace). The next successful
    turn (which calls `save_runtime_checkpoint` at compact/tools/
    finalize) will overwrite the corrupt file with the now-empty
    context, destroying the corrupt checkpoint.
  - `echo-agent/src/state/mod.rs:116-135` — `AgentCheckpoint` has no
    `version` field. Only `working_dir` has `#[serde(default)]`. A
    forward-incompatible checkpoint (missing a required field) fails
    inside `serde_json::from_str` with a generic "missing field"
    error, routed to the same silent-fallback arm.
  - `echo-agent/src/state/file.rs:158-167` — the store layer correctly
    returns `Err` for corrupt JSON (verified by
    `corrupt_nodes_file_surfaces_as_error`). The validator layer
    (`state/mod.rs:157-231`) correctly returns `SerializationError`.
    The defect is purely in the consumer's suppression.
- Reachability: any corrupt `checkpoint.json` (partial write from a
  crash between temp-create and rename, disk error, external edit, or
  a schema change between versions) on a conversation that is later
  resumed. The `FileRuntimeStateStore` uses atomic writes
  (`file.rs:210-246`) so partial writes are unlikely, but disk errors,
  manual edits, and version skew remain possible.
- Expected invariant: a persistent resume mechanism should either (a)
  surface resume failure to the caller/user so they can recover, or
  (b) preserve the corrupt checkpoint for manual recovery before
  overwriting. Silently resetting to empty + destroying the corrupt
  file achieves neither. The sister store layer explicitly documents
  "Corrupt JSON is an error ... rather than silently returning None"
  (`file.rs:14-19`) — the consumer violates the spirit of this
  contract end-to-end.
- Observed behavior: a user whose checkpoint is corrupt (for any
  reason) sees, on next resume: a `warn!` log line (visible only to a
  developer tailing logs), an empty conversation context, and — after
  their next message — the corrupt file overwritten with the
  context-less exchange. The original corruption is unrecoverable.
  This is the same silent-data-loss pattern F-MEM-01-P1-01 flagged for
  `FileStore::new`, but worse because the overwrite makes it
  unrecoverable.
- Impact: silent permanent loss of the conversation trajectory on any
  checkpoint corruption. For a local desktop assistant where the
  conversation history is a primary artifact, this is a meaningful
  data-loss path. The absence of a version tag compounds it: users
  upgrading the app cannot tell whether a resume failure is corruption
  or version skew, and maintainers cannot migrate.
- Root cause: `restore_thread_context`'s `Err` arm was written to
  "fail open" (start fresh rather than block the user) without
  considering that the fresh start overwrites the evidence. The
  `warn!`-only signal assumes a developer is watching logs, which is
  not true for end users of a desktop app.
- Direction:
  (a) **Preserve before overwrite**: in `save_checkpoint`, if the
    target file exists and fails to parse as `AgentCheckpoint`, rename
    it to `checkpoint.json.corrupt-{timestamp}` before writing the new
    one. This keeps the recovery window open.
  (b) **Surface the failure**: have `restore_thread_context` return a
    `ResumeOutcome` enum (`Restored | NewSession | CorruptRecovered`)
    or emit an `AgentEvent::ResumeFailed { reason }` so the
    application can alert the user (e.g. "Your last session could not
    be restored; it was backed up to ..."). The framework currently
    returns `()` and the caller cannot distinguish the cases.
  (c) **Add a version tag**: add `version: u32` to `AgentCheckpoint`
    with `#[serde(default = "current_version")]` so older checkpoints
    deserialize with a known version and a future migrator can branch
    on it. This is cheap and unblocks future schema evolution.
  Prefer all three; (a) is the highest-value (prevents unrecoverable
  loss), (b) is the user-facing fix, (c) is forward-looking.
- Regression validation: a test that seeds a corrupt
  `checkpoint.json`, resumes, and asserts (1) the corrupt file was
  moved to a `.corrupt-*` backup, (2) a `ResumeFailed` event (or
  equivalent) was emitted, (3) the new turn proceeds with empty
  context without panicking. Today no such test exists.
- Validation reports: [V04](../validations/F-RCT-05/V04-01.md).

### F-RCT-05-P3-01: Replay protection is structural (trajectory + pairing validation), not an idempotency gate — `completed_tool_call_ids` is trace-only

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/state/mod.rs:186-231` —
    `validate_tool_message_pairing` enforces the structural invariant
    (orphan/duplicate/mismatch rejection) but does NOT consult any
    external "already executed" registry.
  - `echo-agent/src/state/mod.rs:170-183` — `completed_tool_call_ids`
    is a pure projection over `Role::Tool` messages.
  - `echo-agent/src/agent/react/mod.rs:1731-1737` — the IDs are
    consumed only by the `CheckpointResumed` trace event. `grep -rn
    "completed_tool_call_ids" echo-agent/src` returns exactly two
    hits: the definition and this emission.
  - `echo-agent/src/agent/react/run/phases/tools.rs` (full file) and
    `echo-agent/src/agent/snapshot.rs:1189-1279`
    (`execute_tool_with_policy`) — no site consults a completed-id set
    to skip a tool call.
- Reachability: every resume. The structural protection applies; the
  non-gated nature applies whenever the model re-issues a tool call
  after resume.
- Expected invariant: a framework advertising "completed tool calls
  are skipped on resume" (per the docstring at `state/mod.rs:152-156`
  and the `CheckpointResumed` event's `completed_tool_call_ids` field)
  implies a runtime skip gate. The framework provides only the
  structural invariant + an informational trace field.
- Observed behavior: on resume, the full trajectory (including
  completed tool_results) is loaded. The model sees the prior results
  in context and typically does not re-request them. But if the model
  DOES re-issue an identical tool call (same name + args, new
  `tool_call_id`), the framework executes it again — no idempotency
  gate, no "this call was already completed" short-circuit. The
  `completed_tool_call_ids` list is emitted to the trace store and
  never consulted by the executor.
- Impact: low. The structural protection is sound for the common case
  (the model sees the result and moves on). The gap only matters for
  a model that deterministically re-issues a call post-resume — rare,
  and arguably the model's problem (prompt-driven design). But the
  docstring + trace-field naming oversells: a reader expects a gate.
  This is consistent with AGENTS.md's prompt-driven-over-state-machine
  rule (informed by the Claude Code / Codex research) — the framework
  intentionally does not impose tool-specific verification, deferring
  to the prompt/tool layer.
- Root cause: design choice. `completed_tool_call_ids` was added for
  trace observability (so a resume can be audited), not as a runtime
  gate. The replay safety comes from `validate_tool_message_pairing`
  ensuring the trajectory is provider-valid + the model seeing prior
  results.
- Direction: document. Add a doc comment on `completed_tool_call_ids`
  (`state/mod.rs:170`) stating that the list is for trace/audit only;
  the framework does NOT skip tool calls by id on resume; replay
  safety is structural (pairing validation) + prompt-driven (the model
  sees prior results). Optionally, surface the `ToolFailure` /
  side-effect metadata in the resumed trajectory so the model can
  avoid re-issuing calls with side effects (overlaps with
  F-RCT-04-P3-02). Do NOT add a framework idempotency gate — that
  would require tool-specific semantics in the framework (AGENTS.md
  layering violation).
- Regression validation: doc-only. Optionally add a test asserting
  that a model re-issuing a tool call post-resume results in a second
  execution (pinning the non-gated behavior as intentional).
- Validation reports: [V02](../validations/F-RCT-05/V02-01.md).

### F-RCT-05-P3-02: In-memory `StateSnapshot` captures only messages, not full agent state; opt-in (default off) — name oversells scope

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-state/src/memory/snapshot.rs:23-35` —
    `StateSnapshot { id, iteration, messages, metadata, created_at }`.
    No plan, no skills, no budget counters, no working_dir, no
    TaskNode DAG.
  - `echo-agent/echo-state/src/memory/snapshot.rs:56-166` —
    `SnapshotManager` is a pure in-memory `Vec<StateSnapshot>`; no
    serialization, no persistence. Module doc (`:1-13`) describes it
    as "对话历史快照" (conversation-history snapshot).
  - `echo-agent/echo-state/src/memory/mod.rs:11-13` (cited by
    F-MEM-01) — explicitly defers cross-restart recovery to
    `RuntimeStateStore`, confirming `SnapshotManager` is in-run only.
  - `echo-agent/src/agent/react/mod.rs:1505-1555` —
    `ReactAgent::snapshot` / `rollback` / `rollback_to` restore ONLY
    `snapshot.messages` via `set_messages` + push loop. Nothing else.
  - `echo-agent/src/agent/react/builder.rs:87-88, 167-168, 1006-1008`
    — `snapshot_policy: Option<SnapshotPolicy>` defaults to `None`;
    `set_snapshot_manager` is called only when `policy.is_some()`.
  - `echo-agent/src/agent/snapshot.rs:1116-1141` — `auto_snapshot`
    short-circuits when `snapshot_manager.is_none()` (the default).
- Reachability: the in-memory snapshot path is reached only when a
  consumer explicitly sets a `SnapshotPolicy` via the builder. The
  persisted `RuntimeStateStore` path (the primary resume mechanism) is
  independent and works without `SnapshotManager`. So
  `StateSnapshot`/`SnapshotManager` is a secondary, opt-in mechanism.
- Expected invariant: a struct named `StateSnapshot` (and a manager
  named `SnapshotManager`) implies it captures agent state. In
  practice it captures only the message list. The scope mismatch is
  the issue, not the behavior — the message-only rewind is a
  legitimate feature.
- Observed behavior: `ReactAgent::rollback(steps_back)` restores the
  message history to an earlier point but does NOT rewind: budget
  counters (the `LoopState.budget` fields — reported_model_tokens,
  usage_complete, wind_down_emitted, final_only), the current
  iteration number, the TaskNode DAG, recently_read_files, plan/skills
  state. So a rollback mid-run can produce a state where the message
  history says "iteration 3" but the budget counters say "iteration 8"
  — an inconsistent snapshot by the standards of a full-state
  checkpoint.
- Impact: low for the framework's own use (the in-memory snapshot is
  opt-in and rarely used). The risk is misuse: a consumer that calls
  `ReactAgent::rollback` expecting a full-state rewind will get a
  message-only rewind and may be surprised by the budget/iteration
  inconsistency. The name `StateSnapshot` invites this
  misunderstanding.
- Root cause: the in-memory snapshot was designed for a narrow purpose
  (message-history rewind for in-run error recovery) and named
  generically. The full-state resume concern was later addressed by
  `AgentCheckpoint`/`RuntimeStateStore`, leaving `StateSnapshot` as a
  misleadingly-named subset.
- Direction: two options.
  (a) **Rename for clarity** (preferred under the no-compat-burden
    rule): rename `StateSnapshot` → `MessageHistorySnapshot` (or
    `MessageCheckpoint`) and `SnapshotManager` →
    `MessageHistoryRollbackManager`, with a doc comment stating it is
    in-run, message-only, not persisted. Update the call sites in
    `react/mod.rs`, `snapshot.rs`, `builder.rs`, `subsystems/memory.rs`.
  (b) **Document the scope**: add a doc comment on `StateSnapshot`
    stating "captures only `messages`; for full-state resume use
    `AgentCheckpoint`/`RuntimeStateStore`." Cheaper but leaves the
    misleading name.
  Prefer (a) — the rename is mechanical and the name is the primary
  source of confusion.
- Regression validation: after (a), `cargo check --workspace` and
  `cargo test --lib -p echo_state -- snapshot` +
  `cargo test --lib -p echo_agent -- snapshot rollback`. No behavioral
  change.
- Validation reports: [V01](../validations/F-RCT-05/V01-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Snapshot field round-trip: `StateSnapshot` (in-memory, message-only) vs `AgentCheckpoint` (persisted); resume = trajectory + new turn | yes | passed | [V01-01](../validations/F-RCT-05/V01-01.md) |
| V02 | Completed-tool skip: structural pairing validation; `completed_tool_call_ids` trace-only; no idempotency gate | yes | passed | [V02-01](../validations/F-RCT-05/V02-01.md) |
| V03 | Interrupt safe points: think (intervention + cancel-aware stream), tool (5 s grace; timeout asymmetry), compact (checkpoint-before-mutate); steer cooperative | yes | passed | [V03-01](../validations/F-RCT-05/V03-01.md) |
| V04 | Corrupted/incomplete/version-mismatch: store+validator detect; resume consumer silently swallows + destroys; no version field | yes | failed | [V04-01](../validations/F-RCT-05/V04-01.md) |
| V05 | Historical-document drift check | conditional | n/a | No prior F-RCT-05 report exists in this reviewer directory; the four docstrings treated as hypotheses are classified inline in the Inputs section (three current, one current-but-narrowly-scoped). |

Executed cargo commands (all exit 0):

```text
cd echo-agent
cargo test --lib -p echo_agent -- resume checkpoint corrupt restore pairing hydrate steer cancel interrupt intervention
  → 12 passed; 0 failed
  (checkpoint_restores_paired_tool_history,
   checkpoint_rejects_unpaired_or_duplicate_tool_results,
   resume_records_checkpoint_origin_and_completed_tools_in_trace,
   completed_tool_batch_is_checkpointed_before_next_model_call,
   corrupt_nodes_file_surfaces_as_error,
   path_traversal_conversation_id_is_rejected,
   budget_counters_survive_async_pause_resume_boundary,
   lease_scopes_active_turn_and_preserves_fifo,
   rejects_mismatch_and_non_steerable_turns,
   file_runtime_state_lifecycle,
   + others)
```

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `snapshot.rs:1-6` — "AgentRunSnapshot ... captures agent state for `'static` streaming" via Arc-composition | current | V01 confirms the Arc-composed structure and that cloning is cheap; the snapshot is the live runtime holder for the spawned loop task. |
| `echo-state/memory/snapshot.rs:1-13` — "对话历史快照 ... 回滚到上一个 known-good 状态" (conversation-history snapshot, rollback to known-good) | current but narrowly scoped | V01 confirms the message-only scope; the struct name `StateSnapshot` oversells it (F-RCT-05-P3-02). |
| `state/file.rs:14-19` — "Corrupt JSON is an error ... rather than silently returning None" | current at the store layer | V04 confirms `get_checkpoint`/`read_nodes_file` return `Err` for corrupt JSON. The resume consumer (`restore_thread_context`) then swallows the Err — so the claim is true at the store layer but false end-to-end (F-RCT-05-P2-02). |
| `react/mod.rs:1671-1679` — `resume_from_state_store` "Loads the most recent AgentCheckpoint ... restores them into the context manager" | current | V01/V02 confirm the restore path (messages + plan + skills + working_dir + hydrate). |
| `state/mod.rs:152-156` — `restore_messages` docstring: "A checkpoint is resumable only when every assistant tool call has one matching tool result ... avoids ... replaying an already completed side effect" | current for the validation half | V02 confirms `validate_tool_message_pairing` enforces this. The "avoids replaying" claim is structurally true (trajectory loaded) but the framework provides no runtime skip gate (F-RCT-05-P3-01). |
| F-RCT-02 handoff — abandoned/cancelled/blocked arms skip `finalize_run` | current | V03 references this for the think-phase interrupt characterization; not re-audited here. |
| F-RCT-04 handoff — batch-timeout skips checkpoint + ToolBatchEnd | current | V03 references this; the resume consequence is V02 Deviation 1. |
| F-MEM-01 handoff — `SnapshotManager` is in-memory only, not persisted | current | V01 confirms (echo-state/memory/mod.rs:11-13 defers cross-restart to RuntimeStateStore). |

## Coverage And Uncertainty

Inspected in full: `steer.rs` (199 lines), `snapshot.rs:1-780`
(`AgentRunSnapshot` + `RuntimeConfig`/`ToolRuntime`/`GuardRuntime` +
`save_runtime_checkpoint`/`save_transcript_projection`/
`finalize_run`/`hydrate_running_nodes` + `auto_snapshot` +
`check_tool_approval`), `echo-state/memory/snapshot.rs` (305 lines),
`state/mod.rs` (361 lines), `state/file.rs` (368 lines),
`react/mod.rs:1680-1755` (resume body), `context.rs:216-261, 495-575`
(resume entry + Execute/Chat guards), `direct.rs` (46 lines),
`stream_channel.rs:483-756` (run_core_loop),
`phases/think.rs:26-103, 260-351` (intervention + LLM stream),
`phases/tools.rs:130-443` (cancel/timeout arms + checkpoint sites),
`phases/compact.rs:21-116` (checkpoint ordering),
`builder.rs:87-88, 167-168, 1006-1008` (snapshot_policy default).

Not inspected (out of scope or deferred):

- `snapshot.rs:780-1182` beyond the cited methods — the
  `execute_tool_with_policy` body and output-guard/truncation logic
  were sampled for the cancel-awareness claim only; their full audit
  belongs to a tool-execution task (F-RCT-04 covered the pipeline
  seam).
- `SqliteRuntimeStateStore` (`state/sqlite.rs`) — out of scope per
  AGENTS.md (CLI does not enable SQLite; the framework offers it as a
  menu option for other consumers).
- The application-layer (echo-agent-cli) chat-load flow on the live
  chat path — the grep surfaced `WelcomeScreen.handleResume` →
  `loadConversation` (display load) and the `taskRuntimeStore`
  interrupt/resume (subagent TaskRun resume, a different layer). Neither
  appears to populate the model's `ContextManager` on the chat path;
  confirming this definitively is an application-task concern
  (A-state-* / A-boot-*).
- `stream_channel.rs:757-2161` test module beyond the cited tests.

Environmental constraints:

- All cargo commands ran against the existing incremental build cache
  (`target/`); no `cargo clean` was needed. Final worktree state is
  clean.
- The feature matrix was not re-run; only the default feature set was
  exercised. The resume path is feature-independent except for the
  `#[cfg(feature = "human-loop")]` approval methods
  (`snapshot.rs:798-869` vs `:858-869` stub), which were statically
  inspected.

Uncertain claims:

- Whether echo-agent-cli has an application-layer chat-load flow that
  calls `load_messages` to populate context before `run_chat_direct`.
  If it does, F-RCT-05-P2-01's impact is reduced (the app compensates
  for the framework's missing chat-restore). The grep did not surface
  such a flow, but a definitive answer requires the application
  runtime/task-runtime audit (A-state-*/A-boot-*).
- Whether any third-party `echo-agent` consumer relies on the
  `StateSnapshot` name. The rename in F-RCT-05-P3-02 is
  framework-internal; under the no-compat-burden rule it is safe, but
  the pub-API surface (`pub struct StateSnapshot`, `pub struct
  SnapshotManager`) is technically used by the re-export at
  `echo-state/src/lib.rs`. A grep of echo-agent-cli did not surface
  direct `StateSnapshot` usage, so the rename's blast radius is
  small.

## Handoff

Conclusions downstream tasks may rely on:

1. **Two snapshot types confirmed, cleanly separated.**
   `StateSnapshot`/`SnapshotManager` (echo-state) is in-run,
   message-only, opt-in, never persisted. `AgentCheckpoint`/
   `RuntimeStateStore` (echo-agent::state) is persisted, carries
   messages+plan+skills+working_dir+blocked_reason. Any task reasoning
   about resume should target `AgentCheckpoint`; any task reasoning
   about in-run rewind should target `SnapshotManager`. They are not
   duplicates.
2. **Resume = trajectory + new turn (matches B-REF-01).** The message
   trajectory is the authoritative context; budget counters,
   iteration number, trace IDs, recently-read-files are deliberately
   NOT restored (recomputed for the new turn). This is the framework's
   recovery model and is consistent with Claude Code's checkpoint +
   rewind approach.
3. **Replay protection is structural, not gated.**
   `validate_tool_message_pairing` is the replay invariant;
   `completed_tool_call_ids` is trace-only. Any task that needs an
   idempotency gate must add it explicitly (and should reconsider
   whether that belongs in the framework per the prompt-driven design
   rule).
4. **Resume works on Execute mode, NOT on Chat mode (P2).** Until
   F-RCT-05-P2-01 is fixed, downstream tasks should not assume
   cross-process chat resume works. The application layer may
   compensate; verify in A-state-*/A-boot-*.
5. **Corrupt checkpoint = silent loss + overwrite (P2).** Until
   F-RCT-05-P2-02 is fixed, the resume path destroys corrupt
   checkpoints. Downstream tasks concerned with durability should
   treat the current behavior as non-recoverable on corruption.

Reports they must read:

- This report (F-RCT-05) for the snapshot/resume/steer/interrupt
  invariants.
- `tasks/F-RCT-02.md` for the loop-body terminal-arm model
  (abandoned/cancelled/blocked arms, finalize_run asymmetry).
- `tasks/F-RCT-04.md` for the tool-batch cancel-grace and timeout
  asymmetry (F-RCT-04-P2-01) that affects checkpoint durability on the
  timeout path.
- `tasks/F-MEM-01.md` for the `SnapshotManager` in-memory-only
  invariant and the `FileStore` silent-corruption parallel
  (F-MEM-01-P1-01).
- `tasks/B-REF-01.md` for the "trajectory + new turn" recovery-model
  convergence across Claude Code / Codex / Cursor / Devin.
- `validations/F-RCT-05/V01-01.md` through `V04-01.md` for per-claim
  evidence.

Conditions that make this report stale:

- Adding a `restore_thread_context` call to `run_chat_direct` or the
  `StreamMode::Chat` arms invalidates F-RCT-05-P2-01 and the V01
  chat-vs-execute characterization.
- Changing the `Err` arm of `restore_thread_context` to propagate or
  emit an event invalidates F-RCT-05-P2-02 and V04 claim 3.
- Adding a `version` field to `AgentCheckpoint` invalidates V04 claim
  4 and partially resolves F-RCT-05-P2-02.
- Wiring `completed_tool_call_ids` into a tool-execution skip gate
  invalidates F-RCT-05-P3-01 and V02 claim 3.
- Renaming `StateSnapshot` invalidates F-RCT-05-P3-02 (resolved) and
  requires updating the re-export at `echo-state/src/lib.rs`.
- Changing the checkpoint call sites in `phases/tools.rs` or
  `phases/compact.rs` invalidates V02 (pairing-by-placement) and V03
  (interrupt-point checkpoint behavior).

Follow-up task IDs (no fixes implemented in this review):

- A **framework robustness task** should fix F-RCT-05-P2-01
  (chat-restore) and F-RCT-05-P2-02 (preserve corrupt checkpoint +
  surface resume failure + add version tag). These are the two
  highest-value fixes in this report.
- An **application audit task** (A-state-* / A-boot-*) should verify
  whether echo-agent-cli compensates for F-RCT-05-P2-01 by loading
  conversation history into the model context on the chat path —
  this determines the real-world severity of the chat-restore gap.
- A **naming/documentation task** should action F-RCT-05-P3-02 (rename
  `StateSnapshot` → `MessageHistorySnapshot` or document its scope)
  and F-RCT-05-P3-01 (document `completed_tool_call_ids` as
  trace-only). Both are low-risk, mechanical changes.
