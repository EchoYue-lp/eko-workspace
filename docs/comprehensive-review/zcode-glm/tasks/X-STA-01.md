# X-STA-01: Persistence, recovery, and identity continuity

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only cross-cutting synthesis; both repos
> `git status --short` empty)

## Question

Do conversation, snapshot, task, Subagent, artifact, and frontend
identities survive restart without duplication or stale overwrite?

## Scope

This is a **cross-cutting synthesis task**. It consumes five dependency
reports (F-RCT-05, F-MEM-01, A-STATE-01, A-TSK-04, A-FE-02) and
re-verifies the identity, crash-point, corruption, and cascade facts
they assert against the live code at the pinned commits.

Primary source paths inspected directly (not via the dependencies) for
identity, crash-point, corruption, and cascade analysis:

- **Identity generation sites**:
  - `echo-agent-cli/web-frontend/src/stores/conversationStore.ts:91-95`
    — frontend `generateId` = `conv-${Date.now()}-${Math.random()...}`.
  - `echo-agent-cli/web-frontend/src/hooks/useTauriChat.ts:181-222` —
    Tauri chat `message_key`/`conversation_id` derivation, the
    "first-turn create conversation before send_chat_message" boundary.
  - `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:200-287,
    289-336` — `drive_chat`'s `turn_id = root_message_id` and
    `formal_run_id_for_turn(turn_id)` derivation.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/
    task_tools.rs:178-180, 945-1004` —
    `formal_run_id_for_turn` (`taskrun:{turn_id}`) and the
    `create_complex_task` background `run_id = Uuid::new_v4()` path.
  - `echo-agent-cli/src/tui/events.rs:1364` (`turn_id`), `:4840-4846`
    (TUI `set_conversation_id`), `:4912` (workspace activation).
  - `echo-agent-cli/src/cli/repl.rs:495-500` (`turn_id` + fallback
    `conversation_id = Uuid::new_v4()`).
  - `echo-agent-cli/src/cli/channels.rs:202` (`turn_id`).
  - `echo-agent/echo-orchestration/src/tasks/runtime.rs:177-224` —
    `TaskSpec::stable_hash`, `TaskClaim::execution_id =
    "{run_id}:{task_id}:{revision}:{attempt}"`.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs:
    801-829 (TaskRun), 985-1075 (PlanTask), 1418-1426 (Artifact),
    1656-1700 (SubagentRun.subagent_run_id), 1705-1717 (ReviewResult.id)`.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/review.rs:
    118-128` — `ReviewResult.id = Uuid::new_v4().to_string()`.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:
    1403-1430 (add_review), 1432-1460 (add_artifact)` — note: only
    `add_review` has production callers; `add_artifact` is dead.

- **Crash-point / corruption surfaces**:
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/
    file_shadow.rs:105-178 (append_event_line), 208-280 (rewrite_plan),
    289-299 (next_seq seeds from read_events), 362-379 (read_events
    returns Err on first malformed line), 396-422 (atomic_write — no
    parent-dir fsync), 424-436 (append_line O_APPEND + sync_all)`.
  - `echo-agent/echo-state/src/memory/file_conversation.rs:148-166
    (read_record — Err on corrupt), 211-241 (read_all_records — list
    fails loud), 488-533 (atomic_write + sync_parent_directory)`.
  - `echo-agent/src/state/file.rs:66-85, 155-187, 206-246` —
    `FileRuntimeStateStore` corrupt-as-error, fully crash-safe
    atomic_write.
  - `echo-agent-cli/echo-agent-app-core/src/tool_execution.rs:464-506
    (remove_conversation tombstone+async cleanup), 508-552
    (rebuild_index_and_recover), 770-809 (read_journal_repairing_last_line
    — DOES truncate partial tail)`.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:
    1528-1594 (active_*_boundaries), 1596-1776 (recover_incomplete),
    2039-2078 (recoverable_subagent_result), 2081-2130
    (list_recovery_blockers fail-closed)`.

- **Deletion cascade paths**:
  - `echo-agent-cli/src/tauri/commands/conversations.rs:585-640`
    (Tauri `delete_conversation`).
  - `echo-agent-cli/src/tui/events.rs:3067-3102` (TUI `/delete-session`).
  - `echo-agent/echo-core/src/tools/artifact.rs:305-328, 384-392`
    (`cleanup_tool_output_scope`, `artifact_scope_component`).
  - `echo-agent-cli/echo-agent-app-core/src/prepared_turn.rs:140-200`
    (`cleanup_user_input_scope`).
  - `echo-agent/src/state/file.rs:189-203` (`clear_conversation` — the
    framework API with ZERO CLI callers).

- **Restart linkage**:
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/
    file_store.rs:84-123` — `latest_run_for_conversation` /
    `find_in_progress_run_by_conversation` rebuild the conversation→run
    link by scanning on-disk run-state.json files.
  - `echo-agent-cli/web-frontend/src/stores/taskRuntimeStore.ts:219-283`
    — `loadByConversation` calls `latestRunForConversation` and rebuilds
    the right-rail panel from the run-state.json snapshot + events.jsonl.

## Out Of Scope

Deferred to named task IDs:

- **F-RCT-05**: framework resume path (`resume_from_state_store`,
  `restore_thread_context`, `validate_tool_message_pairing`,
  `restore_messages`). This task references F-RCT-05's two P2s
  (chat-restore gap, corrupt-checkpoint swallow) as load-bearing
  priors; it does not re-audit them.
- **F-MEM-01**: framework `Store`/`ConversationStore` trait contract.
  This task inherits F-MEM-01's `FileStore`-silently-corrupt (P1-01)
  and atomic-write parent-dir-fsync-gap (P2-01) priors.
- **A-STATE-01**: application conversation persistence authority and
  cascade. This task extends A-STATE-01's TUI-vs-GUI cascade-gap
  finding (P2-02) by adding the missing TaskRuntime + RuntimeState
  cascade.
- **A-TSK-04**: TaskRuntime state machine and recovery. This task
  inherits A-TSK-04's claim-identity matrix (V01) and the
  `recover_incomplete` non-atomicity (P3-02).
- **A-FE-02**: frontend reducer identities. This task inherits
  A-FE-02's per-attempt subagent identity key (V01) without re-auditing
  the TS reducer internals.
- **A-TSK-05 / A-TSK-06**: worktree ownership and review/artifact
  backend. Only the deletion-cascade seam is in scope here.
- **F-MEM-02**: SQLite backend. Per AGENTS.md, CLI does not enable it;
  no claim is made about SQLite crash semantics.
- **Multi-process concurrency**: out of scope per AGENTS.md local-
  assistant threat model. The in-process `Mutex`/`RwLock` is
  sufficient.

## Inputs

Required repository documents read in full:

- Repository root `AGENTS.md` via system reminder. Load-bearing
  sections: "产品定位与安全边界" (local personal assistant; no online
  threat model), "数据持久化:echo-agent-cli(EKO)不需要 SQLite",
  "多模式功能对等:TUI 与 GUI 是功能完全一样的 Agent 完全体",
  framework-vs-application layering gate, "动手前先查是不是已经有了"
  rule, no-panic / UTF-8 safety rules, "代码清理: 无需兼容, 过时
  代码可直接删".
- `docs/comprehensive-review/REPORTING.md`.
- `docs/comprehensive-review/templates/{task-report,validation-report}.md`.

Dependency reports read in full:

- **F-RCT-05** (complete) — established that
  `AgentCheckpoint`/`RuntimeStateStore` is the cross-process resume
  authority; `StateSnapshot`/`SnapshotManager` is in-run message-only
  (V01); resume = trajectory + new turn (V01); chat mode NEVER calls
  `restore_thread_context` (P2-01); corrupt checkpoint is silently
  swallowed then destroyed by the next save (P2-02); no version tag
  on AgentCheckpoint. Load-bearing for V01 (resume identity), V02
  (mid-checkpoint crash), V03 (corrupt checkpoint swallow).
- **F-MEM-01** (complete) — established that `FileConversationStore`
  is the canonical atomic-write recipe (uuid temp + fsync + rename +
  parent-dir fsync); `FileStore` silently swallows corrupt JSON (P1-01);
  `FileStore`/`EmbeddingStore` omit parent-dir fsync (P2-01). Load-
  bearing for V02 (atomic-write comparison), V03 (corruption
  handling).
- **A-STATE-01** (complete) — established that the framework
  `FileConversationStore` is the conversation authority;
  `Persistence`/`SessionSearchEngine` are dead duplicate authority
  (P2-01); TUI `/delete-session` does not call
  `tool_executions.remove_conversation` (P2-02); Tauri
  `save_conversation` is a non-transactional get-update-save (P3-01);
  UI-only thinking segments don't reach runtime on restore (P3-02).
  Load-bearing for V01 (conversation identity), V02 (mid-save crash),
  V04 (deletion cascade on conversation side).
- **A-TSK-04** (complete) — established the claim identity matrix
  `(revision, attempt, spec_hash)` → `execution_id` (V01); stale
  claim rejection before any event append (V02); per-run strict seq
  under per-run lock (V03); `recover_incomplete` is non-atomic across
  transition_run and per-task reset, INTERRUPTED filter only catches
  Running (P3-02); `set_task_status` is non-claim-guarded (P3-01).
  Load-bearing for V01 (claim/execution identity), V02 (mid-recovery
  crash), V04 (TaskRuntime cascade missing).
- **A-FE-02** (complete) — established the frontend identity model:
  subagent store key = `${runId}\u0000${subagentRunId}` where
  `subagentRunId = {run_id}:{task_id}:{plan_revision}:{attempt}`
  (V01); taskRuntimeStore is generation-protected against stale loads
  (V02). Load-bearing for V01 (frontend identity echo), V04 (frontend
  reload after delete).

Historical documents treated as hypotheses:

- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/
  file_shadow.rs:1-7` module docstring claiming the file system is
  the read/write authority. Treated as **current** — verified by V01.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/
  file_shadow.rs:113-117` docstring on `append_event_line` claiming
  "A crash mid-append can at worst lose the last partial line; a
  future hardening pass (gate 2) will truncate a partial tail."
  Treated as **partially-stale** — verified that the *write* side is
  crash-safe (O_APPEND atomic per-call) but `read_events` ERRORS on
  the partial line instead of truncating, so the run is unwritable
  until manual repair. See P2-01.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/
  store.rs:1-9` module docstring claiming "every state mutation
  appends a `RuntimeTaskEvent` and refreshes only the affected
  projection from the full event stream." Treated as **current** —
  verified by V02 (A-TSK-04 inheritance).
- `echo-agent/src/state/file.rs:14-19` docstring claiming "Corrupt
  JSON is an error rather than silently returning None / an empty
  list." Treated as **current at the store layer** — F-RCT-05-P2-02
  showed the resume consumer swallows the Err end-to-end.

## Layering Decision

This is a **cross-cutting synthesis** at the application layer. No
framework code is touched; no new code paths are proposed. The four
findings identify missing cascade wiring and inconsistent corrupt-
file handling; resolution is owned by the dependency reports' follow-
up task IDs.

| Classification | Required answer |
|---|---|
| Generic mechanism | The framework supplies the right durability primitives: `FileConversationStore::atomic_write` (uuid temp + fsync + rename + parent-dir fsync), `FileRuntimeStateStore::atomic_write` (same recipe), `FileConversationStore::read_record` (corrupt-as-error), `FileRuntimeStateStore::clear_conversation` (the missing cascade primitive), `TaskClaim::execution_id` (deterministic per-attempt identity), `validate_tool_message_pairing` (replay-safe trajectory invariant), `cleanup_tool_output_scope` (artifact cascade). All live in `echo-agent`. EKO composes them; the gaps are application-layer wiring, not framework defects. |
| EKO product policy | The application's id-generation policy (frontend `conv-{ts}-{rand}`, TUI/REPL/channels `Uuid::new_v4` for turn_id and conversation_id, `formal_run_id_for_turn` deriving `taskrun:{turn_id}` for Task mode, `Uuid::new_v4` for background runs) is correctly in the application layer. The per-mode cascade gaps (TUI missing `tool_executions.remove_conversation`; both TUI+GUI missing TaskRuntime/RuntimeState cascade) are missing wiring, not framework gaps. |
| Adapter boundary | `latest_run_for_conversation` (file_store.rs:84-92) and `find_in_progress_run_by_conversation` (file_store.rs:96-107) are the thin adapter that re-derives the conversation→run link from persisted state. They do not own authority; they project. The cascade on delete is missing because no caller invokes the analogous `delete_runs_for_conversation`. |
| Duplicate search | Searched both repos for parallel id/cascade authorities: `conversation_id`, `turn_id`, `run_id`, `execution_id`, `task_id`, `subagent_run_id`, `artifact_id`, `clear_conversation`, `delete_run`, `remove_run`, `cleanup_tool_output_scope`, `cleanup_user_input_scope`, `remove_conversation` (on ToolExecutionRepository). Result: ONE definition per identity; ONE production `cleanup_tool_output_scope`; ZERO production callers for `FileRuntimeStateStore::clear_conversation`; ZERO production callers for `add_artifact`; ZERO `delete_runs_for_conversation` (does not exist). The frontend's `subagentRunStoreKey` is a faithful echo of the backend `subagent_run_id`, not a parallel authority. |
| Migration deletion | No deletion proposed. The findings identify missing cascade wiring and an inconsistent partial-tail recovery; resolution is left to the dependency reports' follow-up task IDs. `add_artifact` being dead (zero production callers) is recorded but not actioned here — owned by the A-TSK-06 artifact-preservation track. |

## Current Path

### Verified identity model (V01)

The application ships **eight** durable identity types. Seven are
backend-persisted; one (`message_key`) is the Tauri chat turn
correlation id. Full matrix in [V01-01](../validations/X-STA-01/V01-01.md).

Headline: only **conversation_id** and **task_id** survive restart
with stability. The others are fresh-per-execution by design,
matching the "trajectory + new turn" recovery model F-RCT-05
established from Claude Code / Codex / Cursor / Devin convergence.

```text
Identity                       | Source                                       | Restart-stable? | Persisted at
conversation_id                | Frontend: conv-{ts}-{rand}                   | YES             | ~/.eko/conversations/<safe>.json
conversation_id (TUI/CLI)      | App-supplied / Uuid::new_v4()                | YES             | (same)
turn_id (= root_message_id)    | Uuid::new_v4() per turn                      | NO (by design)  | TaskRun.root_message_id
run_id (Task mode, attended)   | taskrun:{turn_id} — deterministic from turn  | NO (per turn)   | ~/.eko/tasks/{run_id}/run-state.json
run_id (create_complex_task)   | Uuid::new_v4().to_string()                   | NO (per dispatch)| ~/.eko/tasks/{run_id}/run-state.json
execution_id                   | {run_id}:{task_id}:{revision}:{attempt}      | YES (derived)   | events.jsonl step_id
task_id                        | LLM-generated slug at plan creation          | YES             | plan.json (TaskPlan.tasks[].id)
subagent_run_id (= exec_id)    | Aligns with execution_id                     | YES (derived)   | events.jsonl
artifact_id                    | Uuid::new_v4() at Artifact construction      | N/A — DEAD CODE | (would be events.jsonl ArtifactProduced payload)
message_key (Tauri chat)       | crypto.randomUUID() per chat turn            | NO              | (transient)
message_id (frontend)          | frontend-only                                | NO              | attachments_json (UI projection only)
```

The **conversation→run link is rebuilt at query time** from the on-disk
run-state.json files:

```text
loadByConversation(conversationId)                            [taskRuntimeStore.ts:219]
  → taskRuntimeApi.latestRunForConversation(conversationId)
  → store.latest_run_for_conversation(conversation_id)        [store.rs:1514-1520]
  → file_store.latest_run_for_conversation(conversation_id)   [file_store.rs:84-92]
      list_runs()  →  list_run_ids()  →  read_dir(root)
      filter r.conversation_id == conversation_id
      return latest by created_at
```

This means identity continuity is structurally preserved: after
restart, a conversation_id still resolves to its run records, because
each `TaskRun.conversation_id` is persisted in run-state.json.

### Verified crash-point recovery matrix (V02)

Eleven distinct crash points characterized. Full matrix in
[V02-01](../validations/X-STA-01/V02-01.md). Headline:

- **Atomic-write surfaces** (conversation JSON, plan.json,
  run-state.json, checkpoint.json, nodes.json) survive a mid-write
  crash with the OLD file intact. The unique-tmp + rename pattern
  is atomic at the filesystem level; the tmp is orphaned but the
  target is consistent. **Inconsistency**: file_shadow.rs:405
  `atomic_write` OMITS parent-dir fsync, while the framework
  `FileConversationStore::atomic_write` and `FileRuntimeStateStore
  ::atomic_write` include it. Inherited from F-MEM-01-P2-01.
- **Append-only events.jsonl** uses `O_APPEND + sync_all`. Per-call
  atomic; mid-append crash leaves at worst a partial last line.
- **Mid-event (between append and rewrite_plan)** self-heals:
  events.jsonl is authoritative; the next read triggers
  `rewrite_plan` which rebuilds plan.json/run-state.json from the
  event stream.
- **Mid-tool / mid-subagent execution** is reconciled at boot:
  `recover_incomplete` (A-TSK-04 verified) sweeps Running runs →
  Paused, classifies in-flight work via `active_*_boundaries`, marks
  mutating in-doubt tools/subagents as Blocked with
  `RecoveryBlocked`, and requeues replay-safe ones as Pending.
- **Mid-recover_incomplete crash** is the documented gap
  (A-TSK-04-P3-02): INTERRUPTED filter only catches Running; the
  next boot skips the now-Paused run, leaving stuck Running todos.
- **Mid-save_conversation (Tauri)** is the get-update-save race
  (A-STATE-01-P3-01): low impact under single-user serialization.
- **Mid-checkpoint save** is fully crash-safe (FileRuntimeStateStore
  uses uuid temp + fsync + rename + parent-dir fsync).
- **Mid-checkpoint load** is read-only; corruption routes to
  `restore_thread_context`'s silent-swallow arm (F-RCT-05-P2-02).

### Verified corrupt-file handling (V03)

| Surface | Behavior |
|---|---|
| Conversation JSON | `read_record` returns `Err(SerializationError)`; `read_all_records` (list path) fails loud — does NOT skip the corrupt record. (F-MEM-01 verified.) |
| Conversation `_meta.json` | `read_meta` self-heals by scanning records; corrupt meta is reconstructed. |
| RuntimeStateStore `checkpoint.json` | `get_checkpoint` returns `Err`; resume consumer silently swallows + `reset_messages()` (F-RCT-05-P2-02). |
| RuntimeStateStore `nodes.json` | `read_nodes_file` returns `Err`. |
| TaskRuntime `events.jsonl` | `read_events` returns `Err(Decode)` on first malformed line — run becomes unreadable AND unwritable (next_seq uses read_events too). **NO partial-tail truncation.** |
| TaskRuntime `plan.json` / `run-state.json` | `read_plan` / `read_run_state` return `Err(Decode)`; both are disposable projections rebuilt by `rewrite_plan`. |
| Tool-execution journal (`events.jsonl` under tool_execution root) | `read_journal_repairing_last_line` (tool_execution.rs:770-809): if malformed line is the LAST line → truncate and continue; if mid-file → return `Err`. **INCONSISTENT** with TaskRuntime events.jsonl. |

Full matrix in [V03-01](../validations/X-STA-01/V03-01.md). The
headline inconsistency is recorded as P2-02.

### Verified deletion cascade (V04)

Full matrix in [V04-01](../validations/X-STA-01/V04-01.md). Headline:

```text
Tauri delete_conversation(id)                                  [conversations.rs:585]
  ✓ store.delete_conversation(id)                              — removes <safe(id)>.json
  ✓ tool_executions.remove_conversation(id)                    — tombstone + async remove (GUI only)
  ✓ cleanup_tool_output_scope(config, id, None)                — removes tool-output artifacts
  ✓ cleanup_user_input_scope(spill_dir, id)                    — removes user-input artifacts
  ✗ TaskRuntime runs ~/.eko/tasks/{run_id}/                    — NOT removed (P2-03)
  ✗ RuntimeStateStore ~/.eko/runtime_state/<safe(id)>/         — NOT removed (P2-03)

TUI /delete-session <id>                                       [events.rs:3067]
  ✓ store.delete_conversation(id)
  ✓ cleanup_tool_output_scope(config, id, None)
  ✓ cleanup_user_input_scope(spill_dir, id)
  ✗ tool_executions.remove_conversation(id)                    — A-STATE-01-P2-02 prior
  ✗ TaskRuntime runs / RuntimeStateStore                       — NOT removed (P2-03)
```

## Findings

The headline result is **strongly positive on identity and atomic
durability**: identities are deterministic where they need to be
(conversation_id, task_id, execution_id), fresh where they should be
(turn_id, run_id), and the conversation→run link is rebuilt at query
time so restart preserves continuity. Atomic-write recipes are sound
at the framework layer; crash-point recovery is well-characterized.

Four findings aggregate the dependency reports' priors into a
cross-cutting view and identify one new gap (P2-03: TaskRuntime /
RuntimeStateStore cascade missing on conversation delete). The fix
direction for each is owned by the cited dependency-report finding or
follow-up task.

### X-STA-01-P2-01: TaskRuntime `events.jsonl` has no partial-tail recovery — a single truncated final line makes the run unreadable and unwritable (cross-filed from F-RCT-05-P2-02 / F-MEM-01-P1-01 theme)

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/
    file_shadow.rs:362-379` — `read_events` parses each line with
    `serde_json::from_str(line).map_err(|e| ShadowError::Decode(
    format!("line {}: {}", i + 1, e)))?`. The `?` propagates the
    error after the first malformed line; it does NOT skip-and-
    continue or truncate.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/
    file_shadow.rs:289-299` — `next_seq` seeds the seq cache by
    calling `read_events(run_id)?.len()`. If `read_events` returns
    `Err`, `next_seq` returns `Err`, and `append_event_line` fails.
    A run with a partial tail cannot accept new events.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/
    file_shadow.rs:208-280` — `rewrite_plan` calls `read_events`
    and propagates `Err` if any line is malformed.
  - `echo-agent-cli/echo-agent-app-core/src/tool_execution.rs:
    770-809` — `read_journal_repairing_last_line` IS graceful: if
    the malformed line is the last line in the file
    (`reader.fill_buf()?.is_empty()`), it logs a warning, truncates
    the file to the last good offset via `set_len`, and continues.
    Mid-file corruption still returns `Err`.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/
    file_shadow.rs:113-117` docstring acknowledges the gap:
    "A crash mid-append can at worst lose the last partial line;
    `read_events` skips empty lines and a future hardening pass
    (gate 2) will truncate a partial tail." Gate 2 is not
    implemented.
- Reachability: any crash mid-`append_line` (the `f.write_all(bytes)?;
  f.sync_all()?` pair at file_shadow.rs:427-435), disk error, or
  external edit that leaves a partial final JSONL line. Mid-append
  crashes are realistic: a SIGKILL between `write_all` and the
  implicit newline flush, or a partial fsync page, leave a half-
  written line.
- Expected invariant: a partial final line in an append-only JSONL
  store should be recoverable by truncating to the last well-formed
  line — the same recovery the application already implements for
  the tool-execution journal. The TaskRuntime event store is more
  load-bearing than the tool-execution journal (it carries plan +
  subagent + recovery state), yet has weaker recovery.
- Observed behavior: a partial last line makes the entire run
  unreadable. `recover_incomplete` cannot list todos or boundaries
  for it (the run is invisible because `read_run_state` reads from
  `rewrite_plan` which fails). The user cannot list, resume, or
  cancel the run from the GUI; the events.jsonl is also unwritable
  (`append_event_line` fails because `next_seq` fails). The only
  recovery is manual: edit `~/.eko/tasks/{run_id}/events.jsonl` to
  remove the partial line.
- Impact: medium. The trigger requires a crash mid-append (narrow
  window), but the failure mode is total: the run is bricked without
  manual intervention. For a local desktop assistant, "edit the JSONL
  by hand" is not a reasonable recovery for a non-developer user.
- Root cause: the TaskRuntime shadow was written before the
  tool-execution journal's gate-2 hardening; the two never converged.
  The docstring explicitly defers gate 2.
- Direction: factor the `read_journal_repairing_last_line` repair
  logic into a shared `read_jsonl_repairing_last_line` helper and
  route `file_shadow::read_events` through it. The truncation must
  happen under the per-run write lock. Add a regression test that
  seeds a partial tail, calls `read_events`, and asserts the file
  was truncated to the last good line. Owned by a TaskRuntime
  robustness follow-up.
- Regression validation: a test that writes three events, appends a
  truncated fourth line, calls `read_events`, and asserts the result
  is exactly three events AND the file is truncated to its pre-tail
  length.
- Validation reports: [V03-01](../validations/X-STA-01/V03-01.md).

### X-STA-01-P2-02: TaskRuntime `atomic_write` omits parent-directory fsync (cross-filed from F-MEM-01-P2-01)

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/
    file_shadow.rs:396-422` — `atomic_write` creates a unique tmp
    (`tmp.{pid}.{ts}.{counter}` suffix — good), `File::create` +
    `write_all` + `sync_all` (temp fsync — good), `std::fs::rename`
    (atomic — good), but NO `sync_parent_directory` call after the
    rename.
  - Contrast with `echo-agent/echo-state/src/memory/
    file_conversation.rs:494-533` — same uuid-temp + fsync + rename
    pattern, but DOES call `sync_parent_directory` (525-528 on Unix).
  - Contrast with `echo-agent/src/state/file.rs:210-246` —
    FileRuntimeStateStore's `atomic_write` includes
    `sync_parent_directory(parent)?` at line 234.
  - The three sister implementations did not converge.
- Reachability: every `rewrite_plan` call (which writes plan.json
  and/or run-state.json via `atomic_write`). That is, every state-
  mutating TaskRuntime operation.
- Expected invariant: an atomic write should be crash-durable: the
  temp's content fsynced, the rename atomic, and the new directory
  entry durable via a parent-dir fsync. This is the canonical recipe
  (SQLite, `FileConversationStore`, `FileRuntimeStateStore` all do
  it). F-MEM-01 established this as the framework contract.
- Observed behavior: after a crash, the rename of plan.json /
  run-state.json may not reach disk before the crash even though the
  temp's bytes are durable. On Linux ext4 with default mount options,
  the directory entry update can be reordered after the temp's data.
  The window is small (sub-second), and the system self-heals on the
  next `rewrite_plan` (because events.jsonl is authoritative), but a
  crash between the rename and the parent-dir sync leaves the
  projection stale until the next event append.
- Impact: low-medium. The events.jsonl is the authoritative record;
  plan.json / run-state.json are disposable projections. A stale
  projection is automatically rebuilt on the next mutation. The risk
  is purely a one-cycle delay in seeing the new projection after a
  crash. For EKO's local-assistant threat model this is robustness
  hardening, not a live data-loss path.
- Root cause: each backend reimplemented the atomic-write recipe
  independently; only the two framework backends were updated to
  include parent-dir sync.
- Direction: factor a shared `atomic_write` + `sync_parent_directory`
  pair (one in framework `echo-state::util`, one available to the
  application) and route `file_shadow::atomic_write` through it.
  Delete the now-redundant inline implementation. Owned by the same
  cleanup task that should fix F-MEM-01-P2-01 / P2-02.
- Regression validation: a test asserting `atomic_write` invokes
  parent-dir sync (e.g. via a stubbed `File::sync_all` counter), or
  simply assert the shared helper is called.
- Validation reports: [V02-01](../validations/X-STA-01/V02-01.md).

### X-STA-01-P2-03: Conversation deletion does not cascade to TaskRuntime runs or RuntimeStateStore checkpoints — both directories leak permanently (NEW cross-cutting finding)

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/tauri/commands/conversations.rs:585-640`
    (Tauri `delete_conversation`) — calls
    `store.delete_conversation(id)` (removes conversation JSON),
    `tool_executions.remove_conversation(id)` (tombstone + async
    cleanup), `cleanup_tool_output_scope(config, &id, None)`, and
    `cleanup_user_input_scope(spill_dir, &id)`. Does NOT call any
    `delete_runs_for_conversation` (which does not exist) and does
    NOT call `FileRuntimeStateStore::clear_conversation`.
  - `echo-agent-cli/src/tui/events.rs:3067-3102` (TUI
    `/delete-session`) — same set of cleanups; ALSO missing
    `tool_executions.remove_conversation` (A-STATE-01-P2-02 prior).
  - `echo-agent/src/state/file.rs:189-203` — the framework
    `clear_conversation` API exists: `remove_dir_all(self.conv_dir(
    conversation_id)?)` with NotFound tolerated. `grep -rn
    "clear_conversation" echo-agent-cli` returns **ZERO** production
    callers.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/
    file_store.rs:84-107` — `latest_run_for_conversation` /
    `find_in_progress_run_by_conversation` can locate runs by
    conversation_id (read-side), but there is no symmetric
    `delete_runs_for_conversation` write-side method.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/
    store.rs:1522-1526` — `list_runs_in` returns runs across all
    conversations; nothing filters the deletion target.
  - On-disk layout (verified): a deleted conversation leaves:
    - `~/.eko/tasks/{run_id}/events.jsonl + plan.json + run-state.json`
      for every run whose `TaskRun.conversation_id == id` (often
      multiple runs per conversation — Task mode creates a fresh
      `taskrun:{turn_id}` per turn; `create_complex_task` creates
      fresh `Uuid` runs).
    - `~/.eko/runtime_state/<safe(id)>/checkpoint.json + nodes.json`
      for the conversation's runtime checkpoint.
- Reachability: every conversation deletion. Live on every GUI/TUI
  delete.
- Expected invariant: deleting a conversation should remove every
  on-disk artifact whose lifetime is bounded by that conversation.
  The framework supplies the primitives (`FileRuntimeStateStore::
  clear_conversation`, `cleanup_tool_output_scope`); the application
  wired three of them but missed two.
- Observed behavior: after deleting a conversation, the conversation
  JSON, tool-execution tombstones, tool-output artifacts, and user-
  input spill artifacts are gone — but the TaskRuntime run directory
  and the runtime_state checkpoint directory persist. Repeated
  create/delete cycles accumulate disk usage indefinitely.
- Impact:
  - **Disk growth.** A long-running EKO install with many create/
    delete cycles will accumulate orphaned run directories. Each run
    directory contains events.jsonl (potentially large — includes
    full subagent traces + tool payloads) and the runtime_state
    checkpoint contains the full message list (potentially large).
  - **Stale data on restart.** If a NEW conversation happens to
    reuse a previously-deleted conversation_id (frontend
    `generateId` uses `Date.now()` + 6-char random, so collision is
    astronomically unlikely — but REPL/channels use
    `Uuid::new_v4()` outright, and a user could explicitly re-use
    an id), the new conversation's `latestRunForConversation` query
    would surface the OLD run. The framework `safe_segment` does
    not hash, so the path is the same.
  - **Privacy / right-to-erasure.** A user who deletes a
    conversation containing sensitive content reasonably expects the
    content to be gone. The TaskRuntime events.jsonl typically
    contains the full task transcripts, including user goal text and
    subagent outputs. This is a "user意图删除但数据残留" gap.
  - **Inconsistency with the tool-execution cascade.** The same
    `delete_conversation` command carefully tombstones tool-
    execution detail files (tool_execution.rs:464-506) but skips the
    TaskRuntime and RuntimeState directories that hold strictly more
    data.
- Root cause: the deletion cascade was assembled incrementally as new
  persistence layers were added. The conversation store cleanup
  came first; tool-execution was added with its own `remove_conversation`;
  tool-output and user-input artifacts came via the framework helpers.
  TaskRuntime and RuntimeStateStore were never added to the cascade.
- Direction:
  1. Add a `delete_runs_for_conversation(conversation_id)` method
     to `TaskRuntimeStore` that scans runs via `list_runs_in(... all
     statuses ...)` filtered by `r.conversation_id == id`, then for
     each match removes `~/.eko/tasks/{run_id}/` (via `fs::remove_dir_all`).
     Tolerate NotFound. Acquire the per-run lock to serialize against
     in-flight writes.
  2. Wire the application `state_store` (created in
     `infra.rs:1246-1267`) into `TauriState` so
     `delete_conversation` can call
     `state_store.clear_conversation(&id)`.
  3. Extract the cascade into a single helper (e.g.
     `AppState::delete_conversation_cascade(id)`) that calls
     conversation_store + tool_executions + cleanup_tool_output +
     cleanup_user_input + delete_runs_for_conversation +
     state_store.clear_conversation. Call it from BOTH Tauri
     `delete_conversation` and TUI `/delete-session`. This also
     resolves A-STATE-01-P2-02.
- Regression validation: a test that seeds a conversation with a
  run, a checkpoint, tool executions, and tool-output artifacts;
  calls the cascade; asserts every directory and JSON is gone and
  `latestRunForConversation(id)` returns None. Pair with the
  TUI/GUI parity test from A-STATE-01-P2-02.
- Validation reports: [V04-01](../validations/X-STA-01/V04-01.md).

### X-STA-01-P3-01: `add_artifact` and the `Artifact` struct have zero production callers — artifact persistence is dead duplicate authority

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/
    store.rs:1432-1460` — `add_artifact(&self, a: &Artifact)`
    appends an `ArtifactProduced` event with the artifact payload.
  - `grep -rn "add_artifact" echo-agent-cli --include='*.rs'` returns
    exactly TWO hits: the definition at store.rs:1432 and a test
    fixture at store.rs:2263. ZERO production callers.
  - `grep -rn "ArtifactProduced" echo-agent-cli/echo-agent-app-core/src`
    returns hits only in: `surface_contract.rs:179` (a contract
    enumeration), `store.rs:1434, 1440, 1442` (the `add_artifact`
    implementation), `event_rebuild.rs:253` (the rebuild fold's
    "doesn't affect plan.json" arm). No production emitter.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/
    types.rs:1418-1426` — the `Artifact { id, run_id, task_id, kind,
    title, path, metadata }` struct exists and is `TS`-exported
    (frontend has the generated type), but the only construction
    sites are inside `file_store.rs:235` (a list-artifacts test
    fixture) and `store.rs:2250` (an `add_artifact` test).
  - `echo-agent-cli/src/tauri/commands/task_runtime.rs:113-120` —
    `list_task_artifacts` Tauri command IS registered and would
    return artifacts if any existed. It always returns empty in
    production.
  - Contrast: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/
    review.rs:118-128` constructs `ReviewResult { id:
    Uuid::new_v4().to_string(), ... }` and IS produced via
    `store.add_review` from `review.rs`. Reviews have production
    callers; Artifacts do not.
- Reachability: zero — the path is unreachable in production.
- Expected invariant: per AGENTS.md "代码清理: 无需兼容, 过时代码
  可直接删", a persisted type with no producer is dead duplicate
  authority. The A-TSK-06 review/artifact track owns the artifact
  surface; if artifacts are intended to be produced (by a reviewer
  gate or by the executor on completion), the wiring is missing.
- Observed behavior: `list_task_artifacts` always returns `[]`. The
  frontend's `artifacts` panel always shows empty. The
  `ArtifactProduced` event kind is enumerated in `RuntimeEventKind`
  but never emitted. The `Artifact` TS type is in the bundle but
  unused.
- Impact: low (no correctness risk; the feature is simply missing).
  The cost is API-surface confusion: a contributor reading the
  types or the Tauri command list will reasonably believe artifacts
  are persisted and queryable; they are not.
- Root cause: the artifact persistence was added (types + event
  kind + store method + Tauri command + frontend type) but the
  production caller — likely a reviewer gate or executor that
  promotes a subagent output into a durable Artifact — was never
  wired.
- Direction: pick one of:
  1. **Delete the dead surface** (recommended under YAGNI): remove
     `Artifact`, `ArtifactKind::Trace`, `add_artifact`,
     `list_artifacts`, `list_task_artifacts` Tauri command, and the
     `ArtifactProduced` event kind. A-FE-02-P2-01 documents the
     parallel `listReviews` gap on the read side.
  2. **Wire the producer**: have the reviewer gate or the executor
     emit `ArtifactProduced` events when a subagent produces a
     durable output (e.g. a report file, a chart, a verified diff).
     Owned by A-TSK-06's artifact-preservation follow-up.
- Regression validation: after deletion, `cargo check --workspace`
  + the GUI feature check from AGENTS.md must pass. If wired, a
  test that drives a subagent completion and asserts an
  `ArtifactProduced` event lands in events.jsonl.
- Validation reports: [V01-01](../validations/X-STA-01/V01-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Identity table: 8 identity types, generation source, restart stability, persistence location | yes | passed (with finding) | [V01-01](../validations/X-STA-01/V01-01.md) |
| V02 | Crash-point recovery matrix: 11 distinct crash points across 5 file types | yes | passed (with finding) | [V02-01](../validations/X-STA-01/V02-01.md) |
| V03 | Corrupt/partial file handling: 8 surfaces characterized; TaskRuntime events.jsonl inconsistency documented | yes | failed | [V03-01](../validations/X-STA-01/V03-01.md) |
| V04 | Retention and deletion cascade: Tauri + TUI cascade enumerated; missing TaskRuntime/RuntimeState cascade documented | yes | failed | [V04-01](../validations/X-STA-01/V04-01.md) |
| V05 | Historical-document drift | conditional | n/a | No prior X-STA-01 report under `zcode-glm/`; this is the first cross-cutting persistence synthesis. The three module docstrings treated as hypotheses are classified inline in the Inputs section. |

No `cargo` / `vitest` command was executed in this task. The
validations are static cross-report syntheses re-verified against
the pinned commits. The underlying test evidence (F-RCT-05 V01-V04,
F-MEM-01 V01-V04, A-STATE-01 V01-V04, A-TSK-04 V01-V04, A-FE-02
V01-V02) is in the dependency reports' V* reports.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `file_shadow.rs:1-7` "the file system (`events.jsonl` + `plan.json`) is the read/write authority for all task data" | current | V01 confirms; events.jsonl is the authority, plan.json/run-state.json are deterministic projections. |
| `file_shadow.rs:113-117` "A crash mid-append can at worst lose the last partial line; ... future hardening pass (gate 2) will truncate a partial tail" | partially stale | V03 confirms the write side is correct, but gate 2 is not implemented; `read_events` errors on the partial line and the run becomes unwritable. Tool-execution journal HAS gate 2. P2-01. |
| `file_shadow.rs:396-404` "atomic_write: write to a unique tmp file, fsync, rename" | current but incomplete | V02 confirms the recipe is correct on its own terms but OMITS parent-dir fsync (F-MEM-01-P2-01 inheritance). P2-02. |
| `state/file.rs:14-19` "Corrupt JSON is an error rather than silently returning None" | current at store layer | F-RCT-05-P2-02 confirmed the resume consumer swallows the Err end-to-end. V03 re-confirms at the pinned commits. |
| `store.rs:1-9` "Every state mutation appends a `RuntimeTaskEvent` to `events.jsonl`" | current | A-TSK-04 V03 re-confirms; `append_event_line` is the single write primitive. |
| F-MEM-01 handoff "FileConversationStore is canonical atomic-write recipe; FileStore silently swallows corrupt" | current | V02 / V03 confirm both: the conversation-store path is fully crash-safe + fail-loud; the TaskRuntime path is inconsistent (no parent-dir fsync, no partial-tail recovery). |
| F-RCT-05 handoff "Resume = trajectory + new turn; chat mode never restores; corrupt checkpoint silently swallowed" | current | V01 confirms identity continuity matches the trajectory model; V02 confirms mid-checkpoint crash is safe at store layer but swallow-at-consumer is unchanged. |
| A-STATE-01 handoff "TUI/GUI parity gap on tool_executions.remove_conversation; Persistence/SessionSearchEngine are dead" | current | V04 sharpens: the cascade is missing not just tool_executions (TUI) but also TaskRuntime runs + RuntimeStateStore checkpoints (both surfaces). P2-03. |
| A-TSK-04 handoff "Claim identity is deterministic and attempt-scoped; recover_incomplete is not transactional" | current | V01 confirms execution_id derivation; V02 confirms the mid-recovery crash window (P3-02 prior inherited, not re-audited). |
| A-FE-02 handoff "subagentRunStore key = ${runId}\u0000${subagentRunId}; subagentRunId = execution_id" | current | V01 confirms the frontend key is a faithful echo of the backend execution_id, not a parallel authority. |
| AGENTS.md "数据持久化:echo-agent-cli(EKO)不需要 SQLite" | current (load-bearing) | V01-V04 confirm the entire persistence stack (conversation, runtime_state, task_runtime, tool_execution) is file-backed; no SQLite dependency is introduced. |

## Coverage And Uncertainty

Inspected in full (cross-cutting lens):

- The 8 identity generation sites cited above (frontend + 4 surfaces
  + framework claim derivation).
- The 5 atomic-write / corrupt-handling paths
  (`FileConversationStore`, `FileRuntimeStateStore`, `FileTaskShadow`,
  tool-execution journal, tool-execution manifest).
- The 2 deletion-cascade call sites (Tauri + TUI) plus the missing
  `clear_conversation` framework API.
- The conversation→run linkage rebuild path
  (`latest_run_for_conversation` and the frontend
  `loadByConversation` echo).
- The recovery sweep (`recover_incomplete`) at the level of detail
  inherited from A-TSK-04.

Inspected partially (via dependencies):

- The Tauri save/load non-transactional race (A-STATE-01-P3-01) —
  the get-update-save sequence is summarized; the full projection
  pipeline is in A-STATE-01.
- The frontend reducer round-trip (A-FE-02) — the identity echo is
  load-bearing; the TS reducer internals are not re-audited.
- The framework resume path (F-RCT-05) — `resume_from_state_store`
  and `restore_thread_context` are summarized; the full restore
  semantics are in F-RCT-05.

Not inspected (out of scope or deferred):

- The framework `SqliteRuntimeStateStore` (`state/sqlite.rs`) —
  out of scope per AGENTS.md (CLI does not enable SQLite).
- The chat history "session" legacy layout — `Persistence` is dead
  code (A-STATE-01-P2-01); the live path uses
  `FileConversationStore` directly.
- Worktree cleanup on run cancel/delete — A-TSK-05 owns the
  worktree lifecycle.
- Subagent-assigned long-term memory facts (keyed on
  `taskrun:completed:{run_id}`) — the memory subsystem is owned by
  A-MEM-01; whether they cascade on conversation delete is a
  separate concern. A grep of `cleanup_*_memory_for_conversation`
  returned zero hits, so they likely do not cascade, but this is
  outside this task's scope.

Environmental constraints:

- Read-only static cross-cutting synthesis at `echo-agent` `9b0e0fa`
  and `echo-agent-cli` `b3b2e81`. No build or test execution in this
  task. The worktree is clean on both repos.

Uncertain claims:

- The exact probability of a partial-tail event in events.jsonl
  (P2-01). The trigger is a crash mid-`write_all` / mid-`sync_all`
  pair. Modern journaling filesystems (APFS on macOS, ext4 with
  data=journal on Linux) make this rare but not impossible.
- Whether any production deployment has accumulated orphaned run
  directories (P2-03). The on-disk evidence would require running
  `du -sh ~/.eko/tasks` against a live install, which is outside
  this review's scope.
- Whether `add_artifact` (P3-01) is genuinely dead or is wired via
  a reflection/plugin path that grep missed. A whole-repo grep
  across both Rust repos is the strongest static evidence; a
  dynamic trace would be definitive but is out of scope.

## Handoff

Conclusions downstream tasks may rely on:

1. **Identity continuity is structurally sound.** The only
   restart-stable identities that need to be are `conversation_id`
   (frontend-generated, persisted in conversation JSON) and `task_id`
   (LLM slug, persisted in plan.json). `execution_id` is
   deterministic in `(run_id, task_id, revision, attempt)` so it is
   stable for any given attempt. The conversation→run link is
   rebuilt at query time via `latest_run_for_conversation`. No
   duplication, no stale overwrite. (V01)
2. **Atomic-write durability is correct at the framework layer** but
   inconsistent at the application TaskRuntime layer. Three atomic-
   write routines exist (`FileConversationStore`, `FileRuntimeState
   Store`, `file_shadow`); the first two include parent-dir fsync,
   the third does not. (V02, P2-02)
3. **Crash-point recovery is well-characterized.** Eleven distinct
   crash points enumerated; ten self-heal or fail-safe. The single
   non-self-healing case is `recover_incomplete` mid-sweep
   (A-TSK-04-P3-02). (V02)
4. **Corrupt-file handling is fail-loud everywhere EXCEPT the
   TaskRuntime events.jsonl, which has no partial-tail recovery.**
   The tool-execution journal has it; converging the two is a
   localized fix. (V03, P2-01)
5. **The deletion cascade has a real gap.** TaskRuntime runs and
   RuntimeStateStore checkpoints survive conversation deletion,
   leaking disk and (for reused ids) risking stale data surfacing.
   The framework supplies the missing primitive
   (`FileRuntimeStateStore::clear_conversation`); the application
   never wires it. (V04, P2-03)
6. **`Artifact` persistence is dead code.** No production caller
   produces an `ArtifactProduced` event; the type, store method,
   and Tauri command exist but always return empty. (V01, P3-01)

Reports downstream tasks must read:

- This report (X-STA-01) for the identity table, crash-point matrix,
  corruption matrix, and the deletion-cascade gap.
- `tasks/F-RCT-05.md` for the framework resume path and the
  corrupt-checkpoint swallow + chat-mode-restore priors.
- `tasks/F-MEM-01.md` for the canonical atomic-write recipe and the
  FileStore corrupt-file priors.
- `tasks/A-STATE-01.md` for the conversation-authority audit and
  the TUI/GUI cascade parity gap.
- `tasks/A-TSK-04.md` for the claim-identity matrix, the per-run
  event ordering, and the `recover_incomplete` non-atomicity.
- `tasks/A-FE-02.md` for the frontend identity model.

Conditions that make this report stale:

- Adding a `delete_runs_for_conversation` method to
  `TaskRuntimeStore` and wiring it into `delete_conversation`
  invalidates P2-03 and V04's "TaskRuntime runs NOT removed" claim.
- Wiring `state_store.clear_conversation` into the cascade
  invalidates P2-03's RuntimeState half.
- Routing `file_shadow::read_events` through a shared
  `read_jsonl_repairing_last_line` invalidates P2-01 and V03's
  events.jsonl partial-tail claim.
- Routing `file_shadow::atomic_write` through a shared helper that
  includes parent-dir fsync invalidates P2-02 and V02's "no
  parent-dir fsync" claim.
- Adding a production caller of `add_artifact` (resolving P3-01's
  dead-code claim).
- Adding a version field to `AgentCheckpoint` partially resolves
  F-RCT-05-P2-02 (cross-referenced in V03).

Follow-up task IDs (no fixes implemented in this review):

- A **TaskRuntime events.jsonl robustness** task — resolve P2-01 by
  routing `read_events` through the shared partial-tail-recovery
  helper already implemented in `tool_execution.rs:770-809`. Touches
  `file_shadow.rs`.
- An **atomic-write convergence** task — resolve P2-02 together with
  F-MEM-01-P2-01 / F-MEM-01-P2-02 by factoring a single shared
  `atomic_write` + `sync_parent_directory` pair. Touches
  `echo-state`, `echo-agent/src/state/file.rs`, and
  `echo-agent-app-core/tasks/task_runtime/file_shadow.rs`.
- A **conversation-deletion cascade** task — resolve P2-03 by adding
  `delete_runs_for_conversation`, wiring `state_store.clear_
  conversation`, and extracting `delete_conversation_cascade` to be
  called from both Tauri and TUI. Touches `conversations.rs`,
  `events.rs`, `task_runtime/store.rs`, `state.rs`,
  `chat_resources.rs`. This task also resolves A-STATE-01-P2-02.
- An **artifact surface** task — resolve P3-01 by either deleting
  the dead `Artifact` / `add_artifact` / `ArtifactProduced` /
  `list_task_artifacts` surface OR by wiring a producer (reviewer
  gate or executor). Owned by A-TSK-06's artifact track.
