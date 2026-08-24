# A-STATE-01: Conversation persistence and restore

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean (both repositories)

## Question

Are file-backed conversations authoritative, atomic, restorable, searchable,
and cleaned with their dependent artifacts?

## Scope

- EKO conversation store construction and lifecycle: `echo-agent-app-core/src/infra.rs:1215-1237`,
  `state.rs:490-510` (AppState init + search engine), `state.rs:843-930`
  (switch_workspace), `state.rs:1053-1084` (exit_workspace),
  `workspace/layout.rs:21-45`, `persistence.rs` (legacy module),
  `workspace/migration.rs:77-189`.
- Surface adapters: `src/tauri/commands/conversations.rs` (8 commands),
  `src/tauri/commands/session.rs:124-160`, `src/tui/events.rs`
  (/clear /sessions /resume /fork /rename /delete-session /rewind),
  `src/main.rs:129-226` (CLI `--continue`/`--resume`), `src/tauri/desktop.rs:174-177`.
- Framework contract consumed: `echo-agent/echo-state/src/memory/file_conversation.rs`
  (record format, atomicity, search, delete), `echo-state/src/memory/conversation.rs`
  (project/restore), `echo-core/src/memory/conversation.rs` (trait),
  `echo-agent/src/agent/snapshot.rs:648-705` (per-turn save), `react/mod.rs:1260,1667`
  (get_messages/load_messages).
- Frontend trigger: `web-frontend/src/stores/conversationStore.ts:230-330`,
  `web-frontend/src/api/endpoints.ts:400-460`, `useTauriChat.ts:197`, `chatStore.ts:121`.

## Out Of Scope

- `FileConversationStore` implementation robustness (corrupt errors, atomic
  write protocol, path-safe ids, id counter) — framework task F-MEM-01
  (re-verified here at V04-01/V04-02).
- SQLite framework backends — F-MEM-02 (EKO uses none, verified V01-01).
- RuntimeStateStore checkpoints / resume_from_state_store — F-RCT-05 /
  runtime-state-store task.
- Uploads-dir retention policy — A-INP-01-P3-01 (owned there).
- Frontend rendering of restored conversations — A-SRF-03/A-FE-01.

## Inputs

- Root `AGENTS.md` (full), shared `README.md`, `REPORTING.md`, `TASKS.md`
  (A-STATE-01 card), `zcode-ds/README.md`, report templates.
- Dependency reports: zcode-ds `F-MEM-01` (framework store contract and
  robustness; used to avoid re-reviewing the implementation), zcode-ds
  `A-INP-01` (deletion cascade facts, uploads gap, spill scopes).
- Historical documents treated as hypotheses:
  `echo-agent-cli/docs/MASTER-PLAN.md`, `docs/MASTER-PLAN.md` (root),
  `echo-agent-cli/docs/2026-07-28-app-core-full-audit.md`.

## Layering Decision

- Generic mechanism (framework, correct): `ConversationStore` trait,
  `FileConversationStore`, `project_message`/`restore_message(s)`, the
  per-turn `save_transcript_projection` (snapshot.rs:648-705), record format
  `<base>/conversations/<safe_id>.json` + `_meta.json`. Placement matches
  MASTER-PLAN Iteration 3 and F-MEM-01.
- EKO product policy (application, correct): store selection and
  None→disabled fallback (infra.rs:1215-1227), per-workspace store scoping,
  delete-cascade scope selection (tool-output + user-input +
  tool-executions), UI-metadata merge, uploads retention, legacy
  `persistence.rs` projection.
- Adapter boundary: Tauri/TUI/CLI commands are thin over the trait. Two
  adapter defects found: the workspace store-root selection passes the wrong
  base directory to the framework constructor (P1-01, P2-01), and no
  cross-process serialization exists where two EKO processes share one store
  (P2-02). These are adapter/application mistakes, not framework defects —
  the framework explicitly documents single-process semantics
  (file_conversation.rs:43-45) and a SQLite option for multi-process use
  that EKO is prohibited from taking (AGENTS.md).
- Duplicate search (V01-01): `ConversationStore`, `FileConversationStore`,
  `SqliteConversationStore`, `save_messages`, `get_messages`,
  `list_conversations`, `delete_conversation`, `project_message`,
  `restore_message(s)`, `ensure_conversation`, `search_conversations`,
  `SessionSearchEngine`, `index_session`, `reindex_all`, `remove_session`,
  `SavedMessage`, `SavedSession`, `AttachmentsPayload`, `conversations`
  (directory), `sqlite`. Result: one storage authority; second-store lookalikes
  are dead legacy code (`persistence.rs` session methods, zero callers) and a
  dead search engine (`SessionSearchEngine`, zero callers of
  `search`/`index_session`/`remove_session`) — findings P3-01; zero SQLite in
  EKO dependency tree. No `worker` terminology.

## Current Path

Verified data flow (V02-01):

1. **Store construction (three different roots across the GUI lifecycle):**
   boot `create_conversation_store()` = `FileConversationStore::new(user_data_dir())`
   -> `~/.eko/conversations/` (infra.rs:1215-1227; desktop.rs:174-177,
   main.rs:129-137); workspace switch -> `new(WorkspaceLayout::conversations(root))`
   -> `{ws}/.eko/conversations/conversations/` (state.rs:905-912);
   workspace exit -> `new(Persistence::base_dir())` = `~/.eko/sessions` ->
   `~/.eko/sessions/conversations/` (state.rs:1078-1084). Framework `new()`
   always appends `conversations` (file_conversation.rs:70-75).
2. **Write:** framework `run_core_loop` finalization calls
   `save_transcript_projection` (finalize.rs:79-81/164-168/251-255) ->
   `filter_user_visible_transcript` (snapshot.rs:65-71) ->
   `project_messages` -> `ensure_conversation` + `save_messages`
   (snapshot.rs:680-705); `save_messages` is a lock-scoped read-modify-write
   replacing the whole record (file_conversation.rs:350-376). GUI additionally
   saves after each turn via the frontend (useTauriChat.ts:197) ->
   `save_conversation`/`update_conversation` (conversations.rs:392-450,
   543-583) which merge UI metadata into the canonical transcript through
   `project_saved_messages`/`pack_ui_projection`/`merge_projection_json`
   (:65-150).
3. **Read/restore:** GUI `restore_conversation` (conversations.rs:679-724),
   TUI `/resume` (events.rs:4831-4878), CLI `--resume`/`--continue`
   (main.rs:201-226) all do `get_messages` -> `restore_messages` ->
   `load_messages` (context replacement, react/mod.rs:1667-1669).
4. **Search:** live path is the framework `search_conversations`
   (file_conversation.rs:398-440, full record scan + substring) used by TUI
   `/sessions <q>` (events.rs:2984-2990) and GUI `search_conversations`
   (conversations.rs:726-747, frontend endpoints.ts:452). The in-memory
   `SessionSearchEngine` is reindexed at boot (state.rs:496-502) but never
   queried.
5. **Delete:** GUI `delete_conversation` removes store record +
   `tool_executions.remove_conversation` (disk-backed manifests) +
   tool-output scope + user-input scope (conversations.rs:585-640). TUI
   `/delete-session` removes store record + tool-output + user-input scopes
   only (events.rs:3069-3095).
6. **Restore agent context** = `load_messages` replacing context messages;
   `resume_from_state_store` (runtime checkpoints) is a separate mechanism
   (F-RCT-05 scope).

## Findings

### A-STATE-01-P1-01: Exiting a workspace re-roots the conversation store to the legacy `~/.eko/sessions/conversations/` dir — global history becomes invisible and post-exit writes land in a store the boot path never reads

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/state.rs:1078-1083`
  (`FileConversationStore::new(Persistence::base_dir())`);
  `echo-agent-app-core/src/persistence.rs:204` (`base_dir() =
  echo_agent::paths::user_data_path("sessions")`);
  boot site `infra.rs:1215-1227` (`FileConversationStore::new(user_data_dir())`,
  `~/.eko/conversations/`); framework `new()` appends `conversations`
  (`echo-agent/echo-state/src/memory/file_conversation.rs:70-75`).
- Reachability: Tauri command `workspace::exit_workspace`
  (`echo-agent-cli/src/tauri/commands/workspace.rs:113`) ->
  `AppState::exit_workspace` (state.rs:1053) — live GUI flow; the store swap
  also drives the frontend sessions list on the same call
  (workspace.rs:150-163).
- Expected invariant: one canonical global store root
  (`~/.eko/conversations/`) that boot and workspace-exit both use, so
  conversation identity and visibility are stable across the workspace
  lifecycle.
- Observed behavior: after exit, the store instance is rooted at
  `~/.eko/sessions/conversations/` (a pre-U1c legacy location); the boot store
  stays at `~/.eko/conversations/`. The GUI sessions list then shows only the
  legacy dir (typically empty), new per-turn saves and GUI saves go to the
  legacy dir, and after the next app restart the boot store again reads
  `~/.eko/conversations/` — the post-exit conversations disappear and the
  pre-exit ones reappear.
- Impact: conversation history silently splits across two physical roots
  depending on when it was written; sessions list/restore/search see only one
  side; conversations "vanish" and "reappear" without user action; a
  conversation id can exist in both roots with different content.
- Root cause: `exit_workspace` reuses the legacy `Persistence::base_dir()`
  constant instead of the canonical store constructor — the boot path was
  migrated to `user_data_dir()` but this reset path was not.
- Direction: in `exit_workspace` use `infra::create_conversation_store()`
  (or `FileConversationStore::new(echo_agent::paths::user_data_dir())`) so the
  reset matches boot; delete the `Persistence::base_dir()`-based conversation
  construction. Keep `Persistence::base_dir()` only for scheduler_store and
  tasks.db if still needed there.
- Regression validation: GUI flow — create conversation in global scope, enter
  a workspace, exit it, assert the conversation is still listed and readable;
  then save a message after exit, restart the app, assert the message is
  present (not in `~/.eko/sessions/conversations/`).
- Validation reports: [V01-01](../validations/A-STATE-01/V01-01.md),
  [V02-01](../validations/A-STATE-01/V02-01.md),
  [V03-01](../validations/A-STATE-01/V03-01.md)

### A-STATE-01-P2-01: Workspace-switch store is rooted one level too deep (`{ws}/.eko/conversations/conversations/`), diverging from the canonical layout and from the migration target; one conversation id can live in two stores with split transcripts

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/state.rs:905-907` passes
  `WorkspaceLayout::conversations(&workspace.root)` as the store base;
  `workspace/layout.rs:43-45` returns `{root}/.eko/conversations`;
  `FileConversationStore::new` joins `conversations` again
  (file_conversation.rs:70-75) -> records at
  `{ws}/.eko/conversations/conversations/<id>.json`; migration copies legacy
  records to `WorkspaceLayout::conversations(root)` =
  `{ws}/.eko/conversations/<id>.json` (migration.rs:157-189) — one level above
  the live store; the global layout is `<base>/conversations/<id>.json` with
  base = `~/.eko` (infra.rs:1215).
- Reachability: GUI `switch_workspace` (workspace.rs:131 -> state.rs:843/905)
  and the workspace migration flow both live; after the switch the primary
  agent keeps its `conversation_id` while `set_conversation_store` swaps the
  store (state.rs:912), so the next per-turn save
  (snapshot.rs:648-705) creates a second record with the same id in the
  workspace store while the global record retains the pre-switch messages.
- Expected invariant: workspace records live at `{ws}/.eko/conversations/`
  (same relative layout as global); all readers (store, reindex, migration)
  agree on the location; a conversation id is unique across stores for a
  given user scope.
- Observed behavior: workspace records land one directory deeper than the
  canonical layout and than migration output; `create_dir_all(&conv_dir)`
  (state.rs:906) creates the wrong (parent) directory and the log line
  "Switched conversation store to workspace {conv_dir}" names a directory the
  store does not use; migrated conversations are invisible in the workspace;
  the active conversation's transcript is split across the global and
  workspace stores (each side updated by per-turn saves while in scope);
  deletion/search/restore operate on only the current store's copy.
- Impact: workspace conversations not found after migration; partial history
  for the active conversation across a switch; deleting a conversation in one
  scope silently leaves the other scope's copy and artifacts.
- Root cause: the layout helper returns the leaf directory while the
  framework constructor expects a base directory — an off-by-one at exactly
  this one construction site (boot and exit pass base dirs).
- Direction: pass `WorkspaceLayout::state_dir(&workspace.root)` (or add a
  dedicated `conversations_base(root)` helper returning `{root}/.eko`) at
  state.rs:905 and keep the migration target unchanged; update the log line.
- Regression validation: switch_workspace test asserting a saved conversation
  lands at `{ws}/.eko/conversations/<id>.json` and is listed/readable;
  migration-import test asserting migrated files are listed by the workspace
  store; identity test: conversation active across switch continues in one
  record, not two.
- Validation reports: [V01-01](../validations/A-STATE-01/V01-01.md),
  [V02-01](../validations/A-STATE-01/V02-01.md),
  [V03-01](../validations/A-STATE-01/V03-01.md)

### A-STATE-01-P2-02: No cross-process write serialization for the shared global conversation store — concurrent GUI+CLI use loses records and can duplicate message ids

- Priority: P2
- Confidence: medium
- Layer: adapter
- Evidence: GUI (`echo-agent-cli/src/tauri/desktop.rs:174-177`) and CLI
  (`echo-agent-cli/src/main.rs:129-137`) both construct
  `FileConversationStore::new(user_data_dir())` at the same
  `~/.eko/conversations/`; the store Mutex is in-process only
  (file_conversation.rs:50-51) and the module doc explicitly limits it to a
  single process, pointing multi-process users at the SQLite backend
  (file_conversation.rs:43-45) — which AGENTS.md forbids for EKO; the
  `_meta.json` id counter is read at open and persisted on mutation
  (file_conversation.rs:92-118, 373) per process; `save_messages` replaces the
  whole record under the process-local lock (file_conversation.rs:350-376).
- Reachability: `eko --cli --continue` while the GUI is open — both processes
  write per-turn transcript projections for overlapping conversation ids; a
  documented local-assistant scenario (same class as F-MEM-01-P2-01 for the
  scheduler store, desktop.rs:175-180 vs modes.rs:47-53).
- Expected invariant: two EKO processes sharing one store must not lose
  accepted saves and must not assign duplicate message ids.
- Observed behavior: read-modify-write happens under a per-process lock, so
  two processes race on whole-record replacement (last flusher wins, the
  other process's accepted turn is silently dropped until its next save) and
  both may hand out the same next ids from a stale `_meta.json`
  (duplicate StoredMessage ids across records; ids drive ordering per
  F-MEM-01-P3-02).
- Impact: silently lost turns and duplicate message ids after concurrent
  GUI+CLI use; recovery (restore/export) shows a truncated or mis-ordered
  transcript.
- Root cause: no cross-process lock or optimistic concurrency on records or
  the id counter; the framework's documented remedy (SQLite) is unavailable
  to EKO by product decision.
- Direction (application layer, matching the "no SQLite" constraint): add a
  cross-process advisory file lock (e.g., `fd-lock`/`fs2` on the
  `conversations/` dir or `_meta.json`) around compound operations, or add a
  record version/etag compare-and-swap into `save_messages` usage at the EKO
  layer; alternatively document single-writer-per-conversation. Framework
  option: expose the same in a shared helper only if a generic need is shown
  (YAGNI default: keep it in EKO).
- Regression validation: two-process test — process A saves turn N, process B
  with a stale snapshot saves -> assert turn N still present; assert no
  duplicate message ids after interleaved saves.
- Validation reports: [V01-01](../validations/A-STATE-01/V01-01.md),
  [V03-01](../validations/A-STATE-01/V03-01.md)

### A-STATE-01-P3-01: `SessionSearchEngine` is registered but unreachable — the live sessions search is the framework `search_conversations`; module comments falsely claim it drives the sessions-search UI

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/conversation_file.rs:1-9,40-168`
  (engine + reindex + comments "drives the sessions-search UI",
  "replaces FTS5"); `state.rs:365` (field), `state.rs:496-502` (construction +
  `reindex_all` at AppState boot); `sessions/search.rs` re-export; zero
  production callers of `search`/`index_session`/`remove_session`
  (grep-verified); live search = framework `search_conversations`
  (echo-agent/echo-state/src/memory/file_conversation.rs:398-440) called by
  TUI (events.rs:2984-2990) and GUI (conversations.rs:726-747;
  web-frontend/src/api/endpoints.ts:452).
- Reachability: engine never queried; `reindex_all` still runs at every
  AppState boot reading every conversation file (conversation_file.rs:138-167).
- Expected invariant: the search capability described in the module exists
  and is reachable, or the dead engine is removed.
- Observed behavior: dead engine + startup reindex cost + misleading comments;
  `make_snippet` (conversation_file.rs:190-200) also mixes a byte offset from
  the lowercased string with a char index into the original (wrong snippet
  window for non-ASCII content) — cosmetic and unreachable.
- Impact: ~150 lines of dead code, a false authority story (the project's own
  doc narrative says the in-memory engine replaced FTS5 while the real search
  is the framework scan), and per-boot I/O for an unused index.
- Root cause: the engine predates the framework `search_conversations`
  implementation and was never wired or removed.
- Direction: delete `conversation_file.rs` (engine + reindex +
  `ReindexRecord`/`make_snippet`), `sessions/search.rs` re-export,
  `state.search_engine` field and its boot-time `reindex_all` call; if the
  sessions-search UI needs substring+recency results beyond the framework
  scan, wire the engine to `search_conversations` command instead of keeping
  it orphaned.
- Regression validation: `cargo test -p echo-agent-app-core --lib` green after
  removal (V04-03 covers the module's only test); GUI/TUI search smoke via
  `search_conversations` unchanged.
- Validation reports: [V01-01](../validations/A-STATE-01/V01-01.md),
  [V02-01](../validations/A-STATE-01/V02-01.md),
  [V04-03](../validations/A-STATE-01/V04-03.md)

### A-STATE-01-P3-02: TUI `/delete-session` omits the tool-execution repository cleanup that the GUI delete performs — disk-backed manifests survive conversation deletion

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: GUI `delete_conversation` calls
  `tool_executions.remove_conversation(&id)` (conversations.rs:600-607;
  repository is disk-backed — tool_execution.rs:163-232, manifests + output
  files); TUI `/delete-session` (events.rs:3069-3095) only deletes the store
  record + tool-output scope + user-input scope.
- Reachability: `/delete-session <id>` on the TUI while tool executions were
  recorded for that conversation; the GUI frontend later lists them via
  `toolExecutionApi.list(conversationId)` (endpoints.ts:459-460).
- Expected invariant: deleting a conversation removes its dependent artifacts
  identically on every surface (AGENTS.md surface parity; MASTER-PLAN
  deletion-cascade claim).
- Observed behavior: TUI deletion leaves the tool-execution manifests and
  output files on disk; the record is gone but its details remain queryable
  by id.
- Impact: orphaned disk data and a cross-surface cascade asymmetry (the
  detail files can also reference the deleted conversation's artifacts).
- Root cause: the TUI delete path predates the repository cleanup added to the
  GUI command and has no access to the app-core repository handle.
- Direction: mirror the GUI cleanup in TUI `/delete-session` (thread the
  `ToolExecutionRepository` through the TUI app state or centralize the
  cascade in a shared app-core helper both surfaces call).
- Regression validation: TUI integration — run a tool, delete the session,
  assert `toolExecutionApi.list` returns empty and the manifest dir is gone;
  compare with GUI behavior.
- Validation reports: [V02-01](../validations/A-STATE-01/V02-01.md),
  [V03-01](../validations/A-STATE-01/V03-01.md)

### A-STATE-01-P3-03: TUI `/fork` persists the unfiltered runtime transcript (system/internal messages included), diverging from the per-turn projection filter

- Priority: P3
- Confidence: medium
- Layer: adapter
- Evidence: `echo-agent-cli/src/tui/events.rs:4888-4908` —
  `agent.get_messages()` (returns the full runtime context,
  echo-agent/src/agent/react/mod.rs:1260-1263) + `project_messages` without
  filtering; the canonical per-turn save filters system/internal messages via
  `filter_user_visible_transcript` (snapshot.rs:65-71, 648-670) and restore
  expects the user-visible shape.
- Reachability: `/fork <title>` on the TUI; forked record is later restored
  by `/resume` (events.rs:4831-4878) or GUI/CLI restore into the context.
- Expected invariant: persisted transcripts use one projection policy —
  user-visible messages only (the shape `restore_messages` and the UI
  projections are built for).
- Observed behavior: forked records can contain system prompts and
  internal-transcript messages (memory/context markers excluded from the
  per-turn filter are still excluded only when the prefix rules match; system
  role messages are included wholesale).
- Impact: forked conversations are larger and, on resume, may feed internal
  messages (including injected instructions) back into the model context —
  fidelity divergence between fork and per-turn persistence.
- Root cause: fork predates the shared projection filter and does not reuse
  `filter_user_visible_transcript` (not public in the framework today).
- Direction: reuse the framework's user-visible filter for the fork
  projection (export the filter or a `project_user_visible_messages` helper),
  or document fork as full-context persistence.
- Regression validation: unit test — context with a system message and a
  `[Memory`-prefixed user message forks into a record containing neither;
  resume of the fork reproduces the per-turn transcript.
- Validation reports: [V02-01](../validations/A-STATE-01/V02-01.md),
  [V03-01](../validations/A-STATE-01/V03-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Format/authority map + duplicate search (both repos; SQLite absence) | yes | passed (3 dead/legacy duplicates, 1 stale comment) | [V01-01](../validations/A-STATE-01/V01-01.md) |
| V02 | Registration and runtime reachability (commands -> store; per-turn save; restore; search; delete) | yes | passed (P1-01/P2-01 store-root evidence) | [V02-01](../validations/A-STATE-01/V02-01.md) |
| V03 | Invariants/edge cases (corrupt file, round-trip, deletion cascade, concurrent writes, store-root consistency) | yes | passed (P1-01/P2-01/P2-02 evidence) | [V03-01](../validations/A-STATE-01/V03-01.md) |
| V04 | `cargo test -p echo_state --lib --locked file_conversation::` | yes | passed, exit 0 | [V04-01](../validations/A-STATE-01/V04-01.md) |
| V04 | `cargo test -p echo_state --lib --locked memory::conversation` | yes | passed, exit 0 | [V04-02](../validations/A-STATE-01/V04-02.md) |
| V04 | `cargo test -p echo-agent-app-core --lib --locked conversation_file` | yes | passed, exit 0 | [V04-03](../validations/A-STATE-01/V04-03.md) |
| V04 | `cargo test -p echo-agent-cli --bin echo-agent-cli --locked` | yes | passed, exit 0 | [V04-04](../validations/A-STATE-01/V04-04.md) |
| V04 | `cargo test -p echo-agent-cli --features gui --lib --locked conversations` | yes | passed, exit 0 | [V04-05](../validations/A-STATE-01/V04-05.md) |
| V05 | Historical-document drift | yes | passed | [V05-01](../validations/A-STATE-01/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `echo-agent-cli/docs/MASTER-PLAN.md:70` — Iteration 3 migration with corrupt-JSON errors, path-safe ids, unique temp names, parent-dir sync; single-process serialized read/modify/write; lossless projection round-trip | current | [V04-01](../validations/A-STATE-01/V04-01.md), [V04-02](../validations/A-STATE-01/V04-02.md); F-MEM-01 V02/V03 |
| `echo-agent-cli/docs/MASTER-PLAN.md:70` — "App-only SessionSearchEngine ... remains in EKO" | current (placement), but the engine is dead production code and its own comment falsely claims it drives the sessions-search UI | [V02-01](../validations/A-STATE-01/V02-01.md), finding P3-01 |
| `echo-agent-cli/docs/MASTER-PLAN.md:359` — conversation deletion removes tool-output + user-input scopes; 30-day TTL | current | [V03-01](../validations/A-STATE-01/V03-01.md), A-INP-01 |
| `echo-agent-cli/docs/MASTER-PLAN.md:362-365` — role-order merge; `display_content` keeps artifact references | current | [V04-05](../validations/A-STATE-01/V04-05.md) |
| `echo-agent-cli/docs/MASTER-PLAN.md:472-475` — deletion returns before background cleanup finishes | current | conversations.rs:618-636 |
| `docs/MASTER-PLAN.md:95` — file persistence; CLI no SQLite | current | [V01-01](../validations/A-STATE-01/V01-01.md) |
| `docs/MASTER-PLAN.md:115` — conversation deletion cascade | partially current (TUI omits tool-execution cleanup; uploads never cleaned — A-INP-01-P3-01) | finding P3-02 |
| `docs/MASTER-PLAN.md:291` — persistence de-data-URL (deferred) | current (deferred) | [V05-01](../validations/A-STATE-01/V05-01.md) |
| `docs/2026-07-28-app-core-full-audit.md:26-27,171-181` — S2/S3 migration; `persistence.rs` stays in app as older projection | current; its session methods are dead | [V01-01](../validations/A-STATE-01/V01-01.md) |

## Coverage And Uncertainty

- Store-root defects (P1-01, P2-01) and the cross-process race (P2-02) are
  static call-graph proofs; no process was launched (read-only review).
  Q-E2E-01 should add: GUI workspace enter/exit with an existing global
  conversation; GUI+CLI concurrent chat on one store.
- The in-process GUI get-then-save window (`save_conversation`/`update_conversation`
  span multiple lock acquisitions while the framework per-turn save may
  interleave) was analyzed: the merge design drops UI-tail messages until the
  framework saves the next turn, bounding practical loss to transient UI
  metadata. Not promoted to a finding; a dynamic interleaving test is
  recommended for the roadmap.
- `filter_user_visible_transcript` is private; the fork divergence
  (P3-03) could not be dynamically confirmed.
- The `make_snippet` byte/char-index mismatch is noted inside P3-01; the
  engine is dead so no separate finding.
- GUI frontend `chatMessagesToSaved` field mapping was inspected at the API
  contract level (endpoints.ts), not component-by-component (A-FE-01).
- Workspace conversations for the same id across stores and the migration
  interplay were verified statically only.

## Handoff

- Downstream tasks may rely on: single storage authority
  (framework `FileConversationStore` at `<base>/conversations/`), EKO with
  zero SQLite; per-turn framework save is the automatic writer; GUI/TUI/CLI
  restore paths are wired and tested at the unit level; corrupt files surface
  as errors on every surface; the three store-root inconsistencies
  (P1-01/P2-01) and their exact line anchors; the cross-process gap (P2-02);
  dead `SessionSearchEngine` (P3-01) as a deletion target.
- Reports to read: this report + F-MEM-01 (framework robustness) + A-INP-01
  (deletion cascade and uploads) + validation reports V01-V05.
- Findings owned elsewhere: uploads retention — A-INP-01-P3-01; runtime-state
  checkpoint restore — F-RCT-05; message-id ordering contract — F-MEM-01-P3-02.
- X-STA-01 should use the store-root findings for its identity-continuity
  matrix (the same conversation id across stores is exactly its class of
  defect); X-SRF-01 should add a TUI-vs-GUI delete-cascade parity row (P3-02);
  Q-E2E-01 scenarios listed in Coverage And Uncertainty.
- This report becomes stale if: the three store construction sites
  (infra.rs:1215, state.rs:905, state.rs:1078) change; `save_transcript_projection`
  or `filter_user_visible_transcript` changes; `conversation_file.rs` or the
  framework `search_conversations` changes; delete-cascade code in
  conversations.rs/events.rs changes.
