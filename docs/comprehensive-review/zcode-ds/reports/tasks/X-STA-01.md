# X-STA-01: Persistence, recovery, and identity continuity

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63 (baseline 9b0e0fa)
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5 (baseline b3b2e81)
> Worktree state: clean (both repositories, verified via `git status --porcelain` before and after)

## Question

Do conversation, snapshot, task, Subagent, artifact, and frontend identities
survive restart without duplication or stale overwrite?

**Answer: Mostly yes in the steady state, with three P1-class recovery gaps and
one new P1 deletion-cascade gap.** Identity *generation* is single-sourced per
class and every persisted identity is keyed stably (conversation id → record
file + checkpoint dir; execution id `{run}:{task}:{revision}:{attempt}` →
`events.jsonl` + rebuilt projections; subagent run = execution id; tool
artifact = fresh `detail_ref`; frontend rows keyed by (owner, call_id) /
`{runId}\0{subagent_run_id}`), and the task-claim protocol rejects stale
revision writes so steady-state replay does not duplicate or regress
terminals. However, restart survival is broken by the already-filed recovery
defects (F-RCT-05-P1-01 restore-wipe of the whole context after a poisoned
checkpoint; A-TSK-01-P1-01 torn `events.jsonl` tail permanently bricking the
run; A-TSK-04-P1-01 pause-in-wave → permanent cancel; A-TSK-04-P1-02 mid-wave
fault stranding sibling claims until restart; A-STATE-01-P1-01/P2-01 the same
conversation id living in multiple physical roots across the workspace
lifecycle) and by three new findings: (P1) conversation deletion leaves the
full runtime transcript + plan on disk — the framework `RuntimeStateStore`
trait has no delete API and no EKO path cleans `runtime_state/<id>/`; (P2)
the runtime-checkpoint store root diverges from the conversation-store root at
workspace exit, so snapshot and transcript of the same id are split across
inconsistent roots and the workspace copy is orphaned until workspace
deletion; (P2) fresh TUI sessions without `/resume` drive Task-mode runs into
the fixed shared `"message:task"` grouping bucket across processes while the
same turns' transcripts are saved under a per-process id — task identity
grouping cross-contaminates.

## Scope

- Identity table construction and verification (V01): conversation ids
  (`echo-agent-cli/src/main.rs:128-153`, `src/tauri/commands/conversations.rs:409-436`,
  `web-frontend/src/stores/conversationStore.ts:93-95`, `echo-agent-app-core/src/runtime.rs:88-91`,
  `infra.rs:152-154`, `src/tui/mod.rs:855/1959/4790/4910`), message ids
  (`echo-agent/echo-state/src/memory/file_conversation.rs:350-380`), runtime
  checkpoint (`echo-agent/src/state/file.rs:53-64`, `echo-agent-cli/echo-agent-app-core/src/infra.rs:1246-1266`,
  `state.rs:921-928`, `state.rs:1090-1094`), task run/plan/revision/attempt
  (`task_runtime/task_tools.rs:951`, `revisioned_adapter.rs:252`, `store.rs:986-1029`,
  `echo-agent/echo-orchestration/src/tasks/runtime.rs:221-223`), subagent run
  (`task_runtime/executor.rs:174-180`, `types.rs:1654-1696`), tool detail
  (`tool_execution.rs:191-247`), uploads (`attachments.rs:57-63,150-172`),
  frontend keys (`web-frontend/src/stores/toolExecutionStore.ts:46-48,206-217`,
  `subagentRunStore.ts:157-159,407-441`).
- Crash-point matrix (V02): resume/restore chain
  (`echo-agent/src/agent/react/mod.rs:1680-1752`, `run/context.rs:216-251,502,571`,
  `src/state/mod.rs:186-231`, `run/phases/tools.rs` cancel/error arms),
  task ledger (`file_shadow.rs:289-299,355-379`, `store.rs:1631-1776`,
  `executor.rs:570-582`, `echo-agent/echo-orchestration/src/tasks/runtime_executor.rs:348-421`),
  store-root lifecycle (`state.rs:905-912,921-928,1078-1084,1090-1094`),
  frontend reload (stores per A-FE-02).
- Corrupt/partial files (V03): `file_conversation.rs`, `store.rs` (FileStore),
  `file.rs` (checkpoint/nodes), `file_shadow.rs` (ledger), `embedding_store.rs`,
  `toolExecutionStore.ts`.
- Retention and deletion cascade (V04): `src/tauri/commands/conversations.rs:585-640`,
  `src/tui/events.rs:3069-3095`, `workspace/registry.rs:295-335`, trait surface
  `echo-agent/src/state/mod.rs:246-273`.
- Task-mode run grouping (`echo-agent-app-core/src/chat_driver.rs:292-340`,
  `src/tui/events.rs:1419`, `task_tools.rs:178-180`).

## Out Of Scope

- Framework internals of checkpoint/steer/resume semantics → F-RCT-05 (canonical
  P1-01/P1-02/P1-03/P2-01/P2-02/P3-01), not re-reviewed.
- Framework store implementation robustness → F-MEM-01 (canonical P1-01/P2-01/
  P3-01/P3-02/P3-03), SQLite → F-MEM-02.
- EKO conversation-store lifecycle → A-STATE-01 (canonical P1-01/P2-01/P2-02/
  P3-01..03); its store-root defects are consumed, not re-filed.
- Task claim/recovery internals → A-TSK-01..04 (canonical P1-01/P1-02/P1-03,
  P2-01, P3-01), F-TSK-03.
- Frontend projection internals → A-FE-02 (canonical P2-01..03, P3-01..02),
  A-SRF-02/A-SRF-03 (producer chain).
- Uploads retention → A-INP-01-P3-01 (canonical); non-ASCII spill collision →
  A-INP-01-P3-02.
- Dynamic restart harnesses (GUI/TUI process restarts, LLM end-to-end) →
  Q-E2E-01/Q-FLT-02 (this task is static + source-traced).
- SQLite backends, cross-process locking design, workspace worktree internals.

## Inputs

- Root `AGENTS.md` (UTF-8/panic safety; one-authority; framework-vs-app
  layering; surface parity; deletion/cleanup policy; read-only review), shared
  `README.md`, `REPORTING.md`, `TASKS.md` (X-STA-01 card), `zcode-ds/README.md`,
  report templates.
- Dependency task reports (complete, read in full):
  `zcode-ds/reports/tasks/F-RCT-05.md`, `F-MEM-01.md`, `A-STATE-01.md`,
  `A-TSK-04.md`, `A-FE-02.md`; targeted reads: `A-INP-01.md` (P3-01/P3-02),
  `A-TSK-01.md` (P1-01 canonical wording).
- Historical documents treated as hypotheses: app
  `echo-agent-cli/docs/MASTER-PLAN.md` (deletion-cascade and latest-attempt
  claims), framework `docs/MASTER-PLAN.md` (resume/recovery claims),
  `echo-agent/src/state/file.rs` module doc, `runtime.rs` doc comment.

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism (framework, correctly placed) | `ConversationStore`+`FileConversationStore`, `RuntimeStateStore`+`FileRuntimeStateStore`, `AgentCheckpoint`, `Store`/`FileStore`, claim/execution identity in `echo-orchestration` — all generic. The `RuntimeStateStore` trait (state/mod.rs:246-273) lacks a delete API; adding one (mirroring `ConversationStore::delete_conversation`) is a framework capability question, not EKO-only. |
| EKO product policy (application, correctly placed) | Store-root selection per lifecycle phase (state.rs:905-928, 1078-1094), delete-cascade scope (conversations.rs:585-640), `"message:task"` fallback grouping (chat_driver.rs:305), fresh-id-per-primary-session (runtime.rs:88-91, main.rs:153), uploads staging (attachments.rs), frontend store key/selector semantics. Findings P1-01, P2-01, P2-02 are EKO decisions/omissions (with the framework trait gap in P1-01). |
| Adapter boundary | Tauri/TUI commands are thin over stores; `ensure_task_mode_run` (chat_driver.rs:292-340) is a thin adapter whose `unwrap_or("message:task")` is the P2-02 defect site. |
| Duplicate search (V01-01) | Terms searched in both repositories: `RuntimeStateStore`, `runtime_state`, `checkpoint.json`, `delete_conversation`, `remove_conversation`, `cleanup_user_input_scope`, `cleanup_tool_output_scope`, `message:task`, `generateId`, `conversation_id`, `subagent_run_id`, `execution_id`, `detail_ref`, `executionIdentity`, `subagentRunStoreKey`, `set_state_store`, `recover_incomplete`, `primary-`. Result: one store per identity class; zero parallel implementations; zero `runtime_state` cleanup sites outside workspace deletion; zero `worker` terms. |
| Migration deletion | P1-01 fix deletes nothing but adds a trait method + EKO call sites; P2-01 fix changes two reset lines in `exit_workspace` (state.rs:1078-1084) to use the same base-dir convention as the runtime-store reset; P2-02 fix removes the `"message:task"` fallback by threading the agent's actual conversation id into TUI state (or documents the bucket). |

## Current Path

Verified data flow (anchors in V01-01/V02-01):

1. **Conversation identity**: GUI frontend `generateId()` (`conv-{Date.now()}-{rand}`, conversationStore.ts:93-95) → `save_conversation` → `create_conversation`/`update_conversation` (conversations.rs:409-436) into `FileConversationStore` (`<base>/conversations/<safe_id>.json`, file_conversation.rs:70-75); message ids from the persisted `_meta.json` counter (:350-380). TUI/CLI primary agent gets a fresh uuid per process (main.rs:153) unless `--resume`/`--continue`/`/resume` (main.rs:128-153, tui/mod.rs:1959); runtime.rs:88-91 mints `primary-{uuid}` only when no id is supplied.
2. **Snapshot identity**: `save_runtime_checkpoint` (snapshot.rs:570-631) writes `AgentCheckpoint` under `~/.eko/runtime_state/<safe_id>/checkpoint.json` (single-row upsert, file.rs:5, 170-184); workspace switch swaps the store to `{ws}/.eko/sessions/runtime_state/` (state.rs:921-928), exit resets to global (state.rs:1090-1094). Restore at every Execute-mode turn start (context.rs:502/571) → `resume_from_state_store` (react/mod.rs:1680-1752).
3. **Task identity**: `run_id = uuid` (task_tools.rs:951), `plan_id = plan_{uuid}` (revisioned_adapter.rs:252), revision/attempt in `TaskClaim`; `execution_id = {run}:{task}:{revision}:{attempt}` (runtime.rs:221-223) persisted in the `TaskStarted` event → `events.jsonl` (file_shadow.rs), rebuilt at boot (event_rebuild.rs:229-231); terminal writes claim-guarded (store.rs:1032-1121).
4. **Subagent identity**: `subagent_run_id` = execution id (executor.rs:174-180); durable `subagent_assigned`/`subagent_released` events for TaskRuntime runs; inline runs realtime-only (A-FE-02-P3-02).
5. **Artifact identity**: tool `detail_ref = uuid` per `start()` (tool_execution.rs:200) → `<scope>/details/{detail_ref}.json/.jsonl`; task artifacts dead end-to-end (A-TSK-06-P2-01); uploads `{uuid}_{name}` (attachments.rs:150-172).
6. **Frontend identity**: hydration merges by `(owner+run_id, call_id)` (toolExecutionStore.ts:46-48); live ingest by wire `id` (:206-217); subagent rows keyed `{runId}\0{subagent_run_id}` (subagentRunStore.ts:157-159); latest-attempt selector parses only trailing `:attempt` (:407-414, 417-441).
7. **Deletion**: GUI cascade = record + tool_executions + tool-output + user-input (conversations.rs:585-640); TUI = record + tool-output + user-input (events.rs:3069-3095); workspace deletion removes the whole root / `.eko/` dir (registry.rs:295-335); nothing removes `runtime_state/<id>/` at conversation granularity (V04-01).

## Findings

### X-STA-01-P1-01: Conversation deletion leaves the complete runtime transcript, plan, and active skills on disk — the `RuntimeStateStore` trait has no delete API and no EKO path cleans `runtime_state/<id>/`

- Priority: P1
- Confidence: high (static call graph; zero-cleanup grep proof)
- Layer: application (cascade scope) with a framework API gap (`RuntimeStateStore` trait)
- Evidence: GUI cascade `delete_conversation` removes only the store record, `tool_executions.remove_conversation`, tool-output scope, and user-input scope (`echo-agent-cli/src/tauri/commands/conversations.rs:585-640`); TUI `/delete-session` removes record + tool-output + user-input only (`src/tui/events.rs:3069-3095`); the checkpoint directory `~/.eko/runtime_state/<safe_id>/checkpoint.json` + `nodes.json` is written every turn (compact.rs:35, finalize terminals; root infra.rs:1246-1247) with the FULL unfiltered message list (snapshot.rs:570-631 — no user-visible filter at save time); the `RuntimeStateStore` trait exposes only save_node/load_nodes/update_status/get_checkpoint/save_checkpoint — **no delete** (`echo-agent/src/state/mod.rs:246-273`); repository-wide grep for `runtime_state` cleanup in `echo-agent-cli/src` and `echo-agent-app-core/src` returns only creation sites (infra.rs:1246-1266, state.rs:921-928, 1090-1094); workspace deletion does remove it (registry.rs:307/312) — conversation granularity does not.
- Reachability: definition (trait + file store) → registration (every agent gets `state_store` + `conversation_id`, runtime.rs:80-91; pool agents agent_pool.rs:133-148, 825-894) → live caller: `delete_conversation` on the GUI (conversations.rs:585) and `/delete-session` on the TUI for any conversation that produced at least one checkpoint (i.e., any executed turn).
- Expected invariant: the deletion cascade removes every persisted artifact bound to the conversation identity (AGENTS.md surface parity; app MASTER-PLAN deletion-cascade claim; `docs/MASTER-PLAN.md:115`); deleting a conversation must not leave the most complete copy of its content on disk.
- Observed behavior: after a GUI or TUI delete, `~/.eko/runtime_state/<id>/checkpoint.json` (full messages incl. tool hand-offs and injected instructions), `nodes.json` (plan DAG), and the workspace-scope copy under `{ws}/.eko/sessions/runtime_state/<id>/` all survive; TaskRuntime run data keyed by the conversation id also survives; only workspace deletion cleans these.
- Impact: user-visible privacy/retention violation (delete is not delete — the unfiltered transcript persists), unbounded disk growth (one copy per deleted conversation, plus a second root per visited workspace), and a resurrection vector: a reused conversation id would silently restore the deleted context at the next Execute-mode turn (`restore_thread_context`, context.rs:502/571) — the exact "stale overwrite" class this task tracks. Inconsistent with the hardened-store posture of the rest of the persistence layer.
- Root cause: the checkpoint subsystem was designed with save/load/overwrite semantics and no lifecycle counterpart; the conversation-delete cascade was scoped before the runtime-state store existed and was never extended; the framework trait lacks the primitive, so no consumer can fix it locally.
- Direction: (a) add `delete_conversation(&self, conversation_id)` to `RuntimeStateStore` and implement it in `FileRuntimeStateStore` (remove `<base>/runtime_state/<safe_id>/` — mirror `ConversationStore::delete_conversation`); (b) call it from the GUI `delete_conversation` and TUI `/delete-session` cascades (shared app-core helper so both surfaces stay in parity); (c) decide and document the cascade scope for TaskRuntime run data keyed by the conversation id (delete or archive); (d) regression test asserting `runtime_state/<id>/` is gone after delete and that a re-created id starts from an empty checkpoint.
- Regression validation: fixture — agent turn produces a checkpoint, `delete_conversation(id)` → assert `runtime_state/<id>/` absent and `get_checkpoint(id)` = None; TUI parity fixture; workspace-scope variant.
- Validation reports: [V04-01](../validations/X-STA-01/V04-01.md), [V01-01](../validations/X-STA-01/V01-01.md)

### X-STA-01-P2-01: At workspace exit the conversation store and the runtime-checkpoint store reset to different root conventions — the same conversation id's transcript and snapshot never share a consistent location, and in-workspace checkpoint copies become orphaned

- Priority: P2
- Confidence: high (static call graph; both reset sites verified)
- Layer: application
- Evidence: `exit_workspace` re-roots the conversation store to `Persistence::base_dir()` = `~/.eko/sessions` → records at `~/.eko/sessions/conversations/` (`echo-agent-cli/echo-agent-app-core/src/state.rs:1078-1084`; `persistence.rs:204`), while the runtime store is reset to `user_data_dir()` → `~/.eko/runtime_state/` (`state.rs:1090-1094`; infra.rs:1246-1247); boot uses `user_data_dir()` for the conversation store too (`infra.rs:1215-1227`) — so the exit reset target differs between the two stores and differs from boot; during a workspace visit the checkpoint store is swapped to `{ws}/.eko/sessions/runtime_state/` (state.rs:921-928) while the conversation store goes to `{ws}/.eko/conversations/conversations/` (off-by-one, A-STATE-01-P2-01); the conversation id is unchanged across the switch (A-STATE-01-P2-01), so the same id has up to three checkpoint copies / four transcript copies on disk at once; workspace deletion (registry.rs:295-335) is the only path that removes the workspace-side copies.
- Reachability: GUI workspace enter → chat → exit → restart is a live flow (`workspace::switch_workspace`/`exit_workspace`, src/tauri/commands/workspace.rs:113/131).
- Expected invariant: one canonical root convention for all stores of a given scope; after exit the active conversation's snapshot and transcript are both readable from the same place the boot path reads; no copy of an id becomes unreadable-by-default while remaining on disk.
- Observed behavior: after exit+restart the GUI reads the boot conversation store and the global checkpoint store; in-workspace messages and checkpoints are invisible (and the in-workspace checkpoint is stale if re-entered later — the model context and UI history of the same id then disagree); the exit reset itself is internally inconsistent (conversation store → legacy root, runtime store → global root).
- Impact: the snapshot identity of a conversation silently forks across roots; post-restart model context can be older than the UI history (or vice versa); orphaned `{ws}/.eko/sessions/runtime_state/` trees persist until workspace deletion (ties into P1-01's retention scope); the "vanish/reappear" user-visible behavior of A-STATE-01-P1-01 extends to the checkpoint store.
- Root cause: the exit reset was written against two different path conventions (legacy `Persistence::base_dir()` vs canonical `user_data_dir()`); the runtime-store swap was added later without reconciling the two reset sites.
- Direction: make `exit_workspace` reset BOTH stores via the same canonical constructors (conversation store via `infra::create_conversation_store()` = `user_data_dir()`, matching the runtime store reset); delete the `Persistence::base_dir()`-based conversation construction; keep workspace-scope checkpoint cleanup tied to workspace deletion (already correct).
- Regression validation: GUI flow — conversation created in global scope → enter workspace → exit → assert the conversation is listed, readable, and its checkpoint is the pre-exit one; save a message after exit, restart, assert the message and checkpoint are both present from the boot roots.
- Validation reports: [V02-01](../validations/X-STA-01/V02-01.md), [V01-01](../validations/X-STA-01/V01-01.md)

### X-STA-01-P2-02: Fresh TUI sessions (no `/resume`) drive Task-mode runs into the fixed shared `"message:task"` grouping bucket across processes, while the same turns' transcripts are saved under a per-process uuid — task identity grouping cross-contaminates

- Priority: P2
- Confidence: medium (mechanism high and static; impact depends on TUI Task-mode usage)
- Layer: application
- Evidence: `ensure_task_mode_run` keys `create_run(..., conversation_id.unwrap_or("message:task"), ...)` (`echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:305`); the TUI app starts with `conversation_id: None` (src/tui/mod.rs:855) and sets it only on `/resume` (mod.rs:1959), `/fork` (events.rs:4910), `/clear` (events.rs:4790); TUI turns pass `conv_id: app.conversation_id.clone()` (events.rs:1419) which is None for a fresh session; the framework agent itself is built with a fresh uuid (src/main.rs:153, thread through AgentCreateParams at :154-163), so per-turn transcript projections (snapshot.rs:648-705) land under the per-process id while Task-mode runs land under `"message:task"`; Task-mode `run_id = "taskrun:{turn_id}"` (chat_driver.rs:451-452, task_tools.rs:178-180) keeps runs distinct but the grouping key is shared; `latestRunForConversation` and any conversation-scoped task cleanup operate on the shared bucket (endpoints.ts:526-531).
- Reachability: TUI user in Task mode on a fresh session (no `/resume`), then the same flow in another process, or after restart.
- Expected invariant: task-run grouping uses the same conversation identity as the transcript of the same turn; a run's conversation key must never be a fixed constant shared across independent processes (identity continuity + no cross-contamination).
- Observed behavior: every conversation-less Task-mode run from every TUI process lands in one `"message:task"` bucket; the transcript of the same turn lives under a per-process uuid — the two identities of one turn never match; a `latestRunForConversation("message:task")` query returns another process's runs; a delete-cascade on that key would remove unrelated runs.
- Impact: cross-process identity contamination for task runs (the "duplication/stale" class this task tracks), fragmentation between the transcript view and the task-run view of the same turn, and a latent wrong-deletion hazard for conversation-scoped task cleanup.
- Root cause: the TUI never syncs the runtime-minted conversation id into `app.conversation_id`, and the adapter's None-fallback chose a constant instead of deriving the id from the turn/agent.
- Direction: thread the agent's actual conversation id into TUI app state at bootstrap (or pass `app.conversation_id` explicitly from the runtime-minted id), removing the `"message:task"` fallback; if a fallback is kept, make it per-process unique (e.g., the agent's own id) and document it; add a TUI task-mode test asserting the run's conversation key equals the transcript conversation key.
- Regression validation: TUI task-mode turn without `/resume` → assert `create_run` receives the same id as the agent's `conversation_id`; two processes → distinct grouping keys.
- Validation reports: [V01-01](../validations/X-STA-01/V01-01.md), [V02-01](../validations/X-STA-01/V02-01.md)

### X-STA-01-P3-01: Frontend conversation id `conv-{Date.now()}-{rand}` has no server-side uniqueness check — a colliding id silently merges two conversations through the `save_conversation` update path

- Priority: P3
- Confidence: medium (mechanism high; collision probability low)
- Layer: application (adapter)
- Evidence: `generateId()` = `conv-{Date.now()}-{Math.random().toString(36).slice(2, 8)}` (web-frontend/src/stores/conversationStore.ts:93-95); `save_conversation` checks existence and MERGES on collision — `existing → update_conversation + project_saved_messages(&id, messages, &existing_messages)` (src/tauri/commands/conversations.rs:409-436); no uniqueness validation on `create_conversation` (conversations.rs:427).
- Reachability: id collisions require equal millisecond + equal 6-char base36 suffix; plausible only with clock skew, manual record restoration, or after a conversation record is restored from backup with an old id while the frontend generates the same id.
- Expected invariant: identity generation is collision-safe or collision-rejected; a duplicate id must never merge distinct conversations (AGENTS.md "身份跨重启无重复/无陈旧覆盖").
- Observed behavior: a colliding id is treated as the same conversation and the two transcripts are merged by role-order projection.
- Impact: silent cross-conversation merge (data-contamination) in the rare collision case; also, because the id is client-generated, any future non-ASCII/short id scheme inherits the missing server-side check.
- Root cause: client-only id generation with no server-side uniqueness or generation move; the merge-on-existing design predates any uniqueness concern.
- Direction: generate the id server-side in `create_conversation` (return it to the frontend), or at least reject `save` when the id exists but the payload's first message id does not match the record; add a collision fixture.
- Regression validation: fixture — save payload with an id that exists and different content → either new id assigned or error; never merged silently.
- Validation reports: [V01-01](../validations/X-STA-01/V01-01.md)

No further findings. The remaining defect rows in the matrices are canonical
IDs folded in from dependency tasks (see V05-01) — most notably the P1 set:
F-RCT-05-P1-01 (poisoned checkpoint → whole-context wipe on resume),
A-TSK-01-P1-01 / A-TSK-04-P1-03 (torn `events.jsonl` tail bricks the run),
A-TSK-04-P1-01 (pause-in-wave → permanent cancel), A-TSK-04-P1-02 (mid-wave
fault strands Running claims until restart), and A-STATE-01-P1-01 (exit
workspace → conversation store re-rooted to the legacy dir). These directly
answer the task question with "no" for the crash points they cover.

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Identity table: conversation/snapshot/task/subagent/artifact/frontend identity sources + persistence locations (duplicate search included) | yes | passed (3 new findings) | [V01-01](../validations/X-STA-01/V01-01.md) |
| V02 | Crash-point recovery matrix: crash at each stage → post-restart identity/state (12 rows) | yes | passed (defects found; canonical + new) | [V02-01](../validations/X-STA-01/V02-01.md) |
| V03 | Corrupt/partial files: per-file behavior and preservation (10 rows) | yes | passed (defects found; canonical) | [V03-01](../validations/X-STA-01/V03-01.md) |
| V04 | Retention and deletion cascade: GUI/TUI/workspace granularity, per-identity matrix | yes | passed (P1-01 evidence) | [V04-01](../validations/X-STA-01/V04-01.md) |
| V05 | Cross-check with existing findings: canonical IDs folded into the matrices; new findings non-duplicate | yes | passed | [V05-01](../validations/X-STA-01/V05-01.md) |

All required validations executed (read-only static analysis; no source
modified, no commands with non-zero exits). No executable validation was
appropriate for this task: it consumes completed dependency reports plus
targeted source verification, and the dynamic restart harnesses belong to
Q-E2E-01/Q-FLT-02.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| app `docs/MASTER-PLAN.md:115` — conversation deletion removes dependent artifacts (tool-output + user-input scopes; cascade) | current for the named scopes / regressed for the full identity | GUI covers record+tool+output+input (conversations.rs:585-640); checkpoint (P1-01), uploads (A-INP-01-P3-01), TaskRuntime data, and TUI tool-execution (A-STATE-01-P3-02) escape the cascade → X-STA-01-P1-01 |
| app `docs/MASTER-PLAN.md:196-203` — frontend stores keep terminal state monotonic, default to latest attempt | regressed (tool live path, revision-blind selector) | toolExecutionStore.ts:206-217 (A-FE-02-P2-01), subagentRunStore.ts:407-414 (A-FE-02-P2-02) |
| framework `docs/MASTER-PLAN.md` — resume skips completed side effects, "已完成结果不重放" | regressed | `completed_tool_call_ids` only logged (F-RCT-05-P2-01) |
| `echo-agent/src/state/file.rs` module doc — atomic writes, corrupt JSON is an error | current (write path) | file.rs atomic_write:210+, read errors :66-74/:161; corrupt-file *handling* at restore remains warn+reset (F-RCT-05-P3-01) |
| `file_shadow.rs:114-117` — torn partial tail tolerated/truncated by future hardening | regressed | read_events hard-errors (file_shadow.rs:355-379) → A-TSK-01-P1-01 |
| `echo-agent-cli/echo-agent-app-core/src/runtime.rs:88-91` — fresh id per primary session avoids merging independent TUI/CLI runs into one "primary" row | current (transcript side) / incomplete (task-run side) | transcript per-process uuid (main.rs:153); Task-mode runs still merge under `"message:task"` → X-STA-01-P2-02 |
| A-STATE-01-P1-01/P2-01 store-root claims | current (re-confirmed) | state.rs:1078-1084 / :905-912; runtime-store side new → X-STA-01-P2-01 |

## Coverage And Uncertainty

- All conclusions are static traces verified against the current commits; no
  process was launched (read-only review), so P1-01's resurrection vector,
  P2-01's post-exit divergence, and P2-02's shared-bucket contamination are
  call-graph proofs, not dynamic reproductions. Q-E2E-01 should add: GUI
  workspace enter/exit with an existing global conversation (assert checkpoint
  + transcript consistent), GUI delete then disk scan for `runtime_state/`,
  TUI fresh-session task-mode run key inspection.
- The message-id cross-process duplication (A-STATE-01-P2-02) is consumed,
  not re-verified dynamically.
- The framework resume/checkpoint behavior is consumed from F-RCT-05
  (including its dynamic probe V04-04); anchors were re-verified at the
  current commit (V02/V03).
- `save_transcript_projection` content vs checkpoint content divergence
  (user-visible filter at projection time, unfiltered at checkpoint time —
  snapshot.rs:570-631 vs :648-705) was noted as the reason the checkpoint is
  the "most complete copy"; the exact diff of which messages each keeps was
  not re-derived (F-RCT-05/A-STATE-01 scope).
- Workspace deletion's coverage of `sessions/runtime_state` is by
  `remove_dir_all` of the whole root/`.eko/` (registry.rs:307/312); the
  legacy-subdir list (registry.rs:329) covers only custom-path workspaces.
- Frontend `latestRunForConversation("message:task")` behavior was traced at
  the endpoint level only (endpoints.ts:526-531); a live GUI query was not run.
- No P0 was found in this task's scope: the steady-state claim protocol and
  atomic writes hold; the P1-class defects were already filed under their
  canonical IDs, and the new P1-01 is a retention/delete-scope violation
  rather than active data loss.

## Handoff

- Downstream tasks may rely on: one identity generator per class (V01-01);
  the crash-point matrix with 12 rows and the canonical-ID mapping (V02-01);
  the corrupt-file family split — hardened (`FileConversationStore`) vs
  lenient/bricked (`FileStore`, checkpoint, ledger) (V03-01); the deletion
  cascade matrix with the new P1-01 gap (V04-01); the canonical-finding
  cross-reference table (V05-01).
- New findings for the roadmap: X-STA-01-P1-01 (trait delete API +
  conversation-cascade cleanup, GUI+TUI parity), X-STA-01-P2-01 (unify
  exit_workspace store-root conventions), X-STA-01-P2-02 (thread real
  conversation id into TUI task-run grouping), X-STA-01-P3-01 (server-side
  conversation id or collision rejection).
- Reports to read: this report + V01-01..V05-01; dependency reports F-RCT-05,
  F-MEM-01, A-STATE-01, A-TSK-04, A-FE-02 (all canonical IDs cited above).
- Stale conditions: this report becomes stale if the store roots change
  (infra.rs:1215/1246, state.rs:905/921/1078/1090), `chat_driver.rs`
  `ensure_task_mode_run` changes, `save_runtime_checkpoint` or
  `restore_thread_context` changes, a `RuntimeStateStore::delete_conversation`
  lands (P1-01 fixed), the deletion cascade in conversations.rs/events.rs
  changes, `generateId`/`save_conversation` merge behavior changes, or any
  canonical finding's evidence file is modified.
- Follow-up task IDs (fixes are not implemented in this review): S-RDM-01
  (roadmap: P1-01 cascade, P2-01/P2-02, P3-01), Q-E2E-01 (restart harness
  scenarios above), Q-FLT-02 (torn-tail/pause-in-wave fixtures already
  specified by A-TSK-04), X-EVT-01 (event identity conformance across
  surfaces), S-X-01 (cross-repository synthesis: P1-01's framework trait
  question is a framework/application boundary decision for that synthesis).
