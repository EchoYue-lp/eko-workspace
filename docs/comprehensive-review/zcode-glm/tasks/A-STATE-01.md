# A-STATE-01: Conversation persistence and restore

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only review)

## Question

Are file-backed conversations authoritative, atomic, restorable, searchable,
and cleaned with their dependent artifacts?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent-cli/echo-agent-app-core/src/persistence.rs` (full, 1-479) —
  application `Persistence` struct, `SavedSession`/`SavedMessage`/
  `AttachmentsPayload`, CLI save/load path, atomic-write helper.
- `echo-agent-cli/echo-agent-app-core/src/conversation_file.rs` (full, 1-225) —
  in-memory `SessionSearchEngine`, reindex-on-start, snippet builder.
- `echo-agent-cli/echo-agent-app-core/src/sessions/{mod,search}.rs` (full) —
  re-export shim that exposes `SessionSearchEngine` as `sessions::SessionSearchEngine`.
- `echo-agent-cli/src/tauri/commands/conversations.rs` (full, 1-748) — Tauri
  `list_conversations`/`save_conversation`/`get_conversation`/
  `update_conversation`/`delete_conversation`/`export_conversation`/
  `restore_conversation`/`search_conversations` commands plus
  `project_saved_messages` UI-metadata merger.
- `echo-agent-cli/src/tui/events.rs` (3050-3105, 3305-3325) — TUI
  `/rename`, `/delete-session`, slash-command surface for conversations.
- `echo-agent-cli/echo-agent-app-core/src/state.rs` (355-525, 880-920,
  1070-1090) — `StorageState` shape, `conversation_store`/`persistence`/
  `search_engine`/`tool_executions` wiring, workspace-switch re-init.
- `echo-agent-cli/echo-agent-app-core/src/infra.rs` (1213-1240, 1240-1270) —
  `create_conversation_store`, `inject_conversation_store`,
  `create_runtime_state_store`.
- `echo-agent-cli/echo-agent-app-core/src/tool_execution.rs` (450-560) —
  `ToolExecutionRepository::remove_conversation` (deletion cascade).
- `echo-agent-cli/echo-agent-app-core/src/prepared_turn.rs` (140-200) —
  `cleanup_user_input_scope` (user-input artifact deletion).
- `echo-agent/echo-state/src/memory/file_conversation.rs` (full, 1-580) —
  authoritative framework `FileConversationStore`, `atomic_write`,
  `safe_segment`, search/list/delete.
- `echo-agent/echo-state/src/memory/conversation.rs` (full, 1-220) —
  `project_message`/`restore_message` canonical projection.

## Out Of Scope

Deferred to downstream tasks:

- **A-TSK-01 / A-TSK-04**: TaskRuntime file authorities (`task_runtime/` file
  store, ledger, revisioned adapter, claim/revision recovery). Only
  conversation JSON is in scope here.
- **A-MEM-01**: hot memory / instruction provider / Dreaming persistence.
- **F-RCT-05 / X-STA-01**: `RuntimeStateStore` cross-restart checkpoint
  (full agent state including plan/skills). `infra.rs:1240-1267` constructs it
  but it is a separate store; covered by F-RCT-05.
- **A-INP-01**: input/attachment preparation path itself; this task only
  audits the deletion cascade that removes user-input artifacts.
- **F-MEM-02**: SQLite backend. Per AGENTS.md, CLI does not enable it.
- **Frontend reducer round-trip**: A-FE-01/A-FE-02 own the TypeScript
  message DTO contract.

## Inputs

- Required repository documents read:
  - `AGENTS.md` (root) — "echo-agent-cli does not need SQLite",
    framework/application boundary, panic-safety and UTF-8 rules,
    "code cleanup: outdated code can be deleted" section.
  - `docs/comprehensive-review/REPORTING.md`,
    `docs/comprehensive-review/templates/{task-report,validation-report}.md`.
- Dependency task reports read:
  - **F-MEM-01** (complete, 2026-08-12) — established that
    `FileConversationStore::atomic_write` is the canonical durability recipe
    (uuid temp + fsync + rename + parent-dir fsync) and that `FileStore` and
    `EmbeddingStore` fail to match it. P2-01/P2-02 are the directly
    applicable priors for this task's application-layer atomic-write audit.
  - **B-REF-01** (complete) — session JSONL + resume pattern (Claude Code,
    Codex) is the industry baseline.
- Historical documents treated as hypotheses: the module docstrings at
  `persistence.rs:1-8` ("保存/加载对话历史") and `conversation_file.rs:1-13`
  ("framework FileConversationStore is the authority; this module owns the
  in-memory index") — both verified below.

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism | `FileConversationStore` (framework) is the single authority for conversation JSON. `project_message`/`restore_message` (framework) is the canonical transcript projection. Application must not duplicate these. |
| EKO product policy | The Tauri `/save_conversation` projection (`project_saved_messages` in `src/tauri/commands/conversations.rs:65-150`) is application-specific: it merges frontend-only metadata (`message_id`, `display_content`, `thinking_segments`, `execution_rounds`, `attachments`) into `attachments_json` on top of the framework's `_echo_message_version` envelope. This is the correct layer for that policy. |
| Adapter boundary | The Tauri adapter is mostly thin (CRUD over the framework trait). It does own one piece of business logic — the alignment heuristic in `project_saved_messages` that pairs UI messages with canonical transcript positions. Conversion is lossless when a canonical transcript exists (`merge_projection_json` preserves the `_echo_message_version` marker); a UI-only save (no canonical transcript) writes a non-canonical record that `restore_messages` will reconstruct from `content`/`tool_calls_json`/`tool_result_json` but will not round-trip `thinking_segments`/`execution_rounds` to the agent runtime. |
| Duplicate search | `pub struct Persistence`, `pub struct SessionSearchEngine`, `save_session`, `load_session`, `list_sessions`, `load_conversation`, `export_conversation_markdown`, `search_engine`, `reindex_all`, `index_session`, `remove_session`. Result: two parallel application storage authorities (`Persistence` and `SessionSearchEngine`) overlap with the framework `ConversationStore` but are not used in production — see P2-01 and V01-01. |
| Migration deletion | `Persistence::{save_session, load_session, load_session_raw, list_sessions, write_json, convert_message, export_conversation_markdown, conversations_dir, session_path}` and `SessionSearchEngine::{index_session, remove_session, search, reindex_all}` are recommended for deletion (or resurrection with a real caller). See P2-01. |

## Current Path

Two authorities are constructed at startup but only one is exercised:

**Framework authority (live):**
`infra.rs:1215-1231 create_conversation_store` builds a
`FileConversationStore` rooted at `~/.eko/` (`echo_agent::paths::user_data_dir()`).
`AppState::from_shared` (`state.rs:454-506`) stores it in
`StorageState.conversation_store: RwLock<Option<Arc<dyn ConversationStore>>>`
(`state.rs:363`). Workspace activation re-roots it under
`<workspace>/sessions/` (`state.rs:880-920`) and deactivation restores the
global path (`state.rs:1070-1090`).
All Tauri commands (`src/tauri/commands/conversations.rs:370-747`) and the
TUI slash commands (`src/tui/events.rs:2990, 3050-3105`) acquire this `Arc`
and call `ConversationStore` methods directly.

**On-disk format** (`file_conversation.rs:42-46, 168-175`):
one JSON file per conversation at
`<base>/conversations/<safe_id>.json` containing
`{conversation: Conversation, messages: Vec<StoredMessage>}`, plus
`<base>/conversations/_meta.json` monotonic id counter. `safe_segment`
(`file_conversation.rs:465-486`) sanitizes ids to `[A-Za-z0-9\-_.:~]` and
rejects empty / `.` / `..` / path separators.

**Atomic write (framework):**
`FileConversationStore::write_record` (168-175) and `persist_meta` (140-146)
route through `atomic_write` (494-523): create a uuid-suffixed temp
(`.{file}.{uuid}.tmp`), `File::create` + `write_all` + `sync_all` (fsync the
temp's bytes), `fs::rename`, then `sync_parent_directory` (Unix only,
525-528). On any I/O error after temp creation the temp is removed (514, 518).
All operations are serialized by an in-process `Mutex<StoreMeta>` (70-71).

**Round-trip projection (framework):**
`echo_state::memory::project_message` (`echo-state/src/memory/conversation.rs:35-88`)
serializes a runtime `Message` to a `StoredMessage`: text content goes to
`content` (searchable), structured `MessageContent::Parts`/`Empty`,
`reasoning_content`, `name`, and `tool_call_id` go into `attachments_json`
under an `_echo_message_version: 1` envelope. Tool calls go to
`tool_calls_json`; tool-role messages also write a `{tool_call_id, name}`
blob to `tool_result_json`. `restore_message` (107-167) is the inverse and
preserves all four canonical roles + tool calls + reasoning. See V03-01.

**UI metadata projection (application):**
Tauri `save_conversation` calls `project_saved_messages`
(`src/tauri/commands/conversations.rs:65-150`) which detects an existing
canonical transcript (`is_framework_projection` 39-42 OR `tool_calls_json`
OR `tool_result_json`). If present, it merges frontend-only fields
(`message_id`, `display_content`, `thinking_segments`, `execution_steps`,
`execution_rounds`, `attachments`) into the existing `attachments_json` via
`merge_projection_json` (44-63) without overwriting the `_echo_message_version`
envelope. If absent (UI-only save), it builds a `StoredMessage` and packs the
UI fields into `attachments_json` via `pack_ui_projection` (9-37).
On load, `get_conversation` (453-541) calls framework `get_messages` and
extracts UI fields via `AttachmentsPayload::parse` (persistence.rs:127-145),
which supports both the new object form and the legacy plain-array form.

**Deletion cascade (Tauri):**
`delete_conversation` (`src/tauri/commands/conversations.rs:586-640`) calls
framework `store.delete_conversation(id)` (removes `<safe_id>.json`),
`tool_executions.remove_conversation(id)` (moves the per-conversation
artifact subtree to `.trash/<scope>-<uuid>` and drops index entries), and
spawns a `tokio::task::spawn_blocking` that runs the framework's
`echo_agent::tools::artifact::cleanup_tool_output_scope(&config, &id, None)`
plus the application's
`prepared_turn::cleanup_user_input_scope(&user_input_spill_dir, &id)`.

**Search path:**
The live search path is `FileConversationStore::search_conversations`
(`file_conversation.rs:398-440`) — scans on-disk JSON files for
case-insensitive substring match in title or any message content, ordered by
`updated_at DESC`, then truncates to `limit`. Both Tauri `search_conversations`
(`src/tauri/commands/conversations.rs:727-747`) and TUI
(`src/tui/events.rs:2990`) call this. Deletion is therefore reflected
automatically in the next search because the file is gone.

## Findings

### A-STATE-01-P2-01: `Persistence` and `SessionSearchEngine` are constructed but never read in production (dead-code authority)

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/persistence.rs:211,242,247,286,321`
    (five public methods `save_session`/`load_session`/`list_sessions`/
    `export_conversation_markdown`/`load_conversation` with no callers).
  - `echo-agent-cli/echo-agent-app-core/src/conversation_file.rs:74,99,107,138`
    (four `SessionSearchEngine` methods; only `reindex_all` is invoked once at
    startup).
  - `echo-agent-cli/echo-agent-app-core/src/state.rs:493-506` constructs both
    but no field is read after construction.
- Reachability:
  - `Persistence` is constructed in `state.rs:495` (`Persistence::new()`) and
    re-instantiated on workspace switch (`state.rs:894, 1058`). Grep for
    `save_session|load_session|list_sessions|export_conversation_markdown|`
    `load_conversation\b|conversations_dir` across `echo-agent-cli/src` and
    `echo-agent-app-core/src` returns hits only inside `persistence.rs` and
    the workspace migration at `workspace/migration.rs:77-189` (which reads
    the directory layout directly, not via these methods). Zero production
    callers.
  - `search_engine` is constructed in `state.rs:496-506`, which invokes
    `reindex_all()` once. Grep for
    `app_state\.storage\.search_engine|storage\.search_engine|state\.search_engine`
    returns zero hits. Tauri `search_conversations` and TUI `/search` both
    call `FileConversationStore::search_conversations` instead.
- Expected invariant: per AGENTS.md "动手前先查是不是已经有了" and
  "代码清理: 无需兼容, 过时代码可直接删", an application storage authority
  that overlaps a framework authority must either be live or be deleted —
  not silently retained. The `conversation_file.rs:1-13` docstring claims
  `SessionSearchEngine` "drives the sessions-search UI", but the UI does not
  call it.
- Observed behavior: Both types are wired into `AppState` and reindex on
  startup (`SessionSearchEngine::reindex_all` walks
  `~/.eko/conversations/*.json`), then never read. `Persistence` is
  instantiated and acquires the global `~/.eko/sessions/` directory but no
  command writes or reads through it. The actual transcript authority is the
  framework `FileConversationStore`.
- Impact:
  - **Misleading API surface.** A new contributor reading
    `persistence.rs` or `state.rs` will reasonably assume `Persistence`
    and `SessionSearchEngine` are the conversation storage and search
    authorities (they are named and commented as such). They will waste time
    tracing a path that is never executed, or worse, build new features on
    top of the dead code instead of the framework trait.
  - **Index/disk drift undetected.** The startup reindex walks files written
    by the framework; if the two ever diverge in shape (e.g. framework
    introduces a new field), the silently-empty index hides the mismatch.
  - **No correctness risk** to current users — the live path is correct.
- Root cause: A prior SQLite/FTS5 removal (see `conversation_file.rs:1-13`
  comment "replaces FTS5") created the in-memory engine as a UI-index, but
  the GUI search commands were later rewired directly to the framework
  trait, leaving the engine orphaned. `Persistence` predates
  `FileConversationStore` and was similarly superseded when the framework
  gained the conversation trait.
- Direction: Pick one of:
  1. **Delete the dead surface** (recommended for now): remove
     `Persistence::{save_session, load_session, load_session_raw,
     list_sessions, write_json, convert_message, session_path,
     conversations_dir, export_conversation_markdown, load_conversation}`,
     the `persistence: RwLock<Persistence>` field on `StorageState`
     (replaced with the workspace path directly if needed elsewhere), and
     the entire `SessionSearchEngine` plus its re-export shim in
     `sessions/{mod,search}.rs`. The `AttachmentsPayload` and `SavedMessage`
     projections are still used by Tauri commands and must stay.
  2. **Resurrect**: wire `search_conversations` Tauri/TUI commands through
     `SessionSearchEngine` for substring performance (avoiding the on-disk
     scan in `FileConversationStore::search_conversations`); add
     `index_session`/`remove_session` calls to `save_conversation` and
     `delete_conversation`.
  Option 1 matches the YAGNI / clean-when-superseded guidance in AGENTS.md.
- Regression validation: After deletion, `cargo check --workspace` plus the
  Tauri GUI feature check from AGENTS.md must pass. Tauri `save_conversation`,
  `get_conversation`, `delete_conversation`, `search_conversations` exercise
  the live path; ensure at least one integration test covers each.
- Validation reports: [V01-01](../validations/A-STATE-01/V01-01.md)

### A-STATE-01-P2-02: TUI `/delete-session` does not clean up tool execution artifacts (TUI/GUI parity gap)

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/tui/events.rs:3067-3102` (`Some(SlashCommand::DeleteSession)`
    branch): calls `store.delete_conversation(id)`,
    `cleanup_tool_output_scope`, and `cleanup_user_input_scope`, but never
    calls `tool_executions.remove_conversation(id)`.
  - Contrast with `echo-agent-cli/src/tauri/commands/conversations.rs:600-608`,
    which DOES call `state.app_state.storage.tool_executions.remove_conversation(&id)`.
- Reachability: TUI `/delete-session <id>` is a live user command
  (`src/tui/events.rs:3067`). `ToolExecutionRepository::remove_conversation`
  (`echo-agent-app-core/src/tool_execution.rs:464-506`) is the only path that
  removes the per-conversation rows from the in-memory `summaries`/`details`/
  `active` indexes and tombstones the artifact subtree.
- Expected invariant: AGENTS.md "多模式功能对等: TUI 与 GUI 是功能完全一样的
  Agent 完全体" — both surfaces must run the same deletion cascade for the
  same user action.
- Observed behavior: After a TUI `/delete-session`, the conversation JSON and
  the framework-owned tool-output artifacts (`<artifact_root>/<conv>/`) are
  gone, but `ToolExecutionRepository.summaries` still contains entries whose
  `conversation_id == Some(id)`, and the per-conversation detail manifests +
  event journals under the tool-execution root remain on disk. The next
  `summaries_for_conversation(id)` (`tool_execution.rs:452-462`) still returns
  stale rows.
- Impact:
  - **Orphaned artifacts and stale UI.** Tool execution detail JSON and
    JSONL journals persist indefinitely under the tool-execution root;
    any TUI/GUI screen that lists prior tool executions for a deleted
    conversation shows ghost rows until restart rebuilds the index (and
    even rebuild leaves the detail files on disk because they live under
    a path derived from `conversation_id`).
  - **Disk growth.** Repeated create/delete cycles accumulate detail files
    that the user cannot see or clean from the TUI.
- Root cause: TUI was wired before `ToolExecutionRepository` was added as a
  cascade participant; the TUI command was never updated when the Tauri
  command gained the `tool_executions.remove_conversation` call.
- Direction: Extract the cascade into a single helper (e.g.
  `AppState::delete_conversation_cascade(id)`) that calls
  `store.delete_conversation`, `tool_executions.remove_conversation`, and
  the artifact/user-input cleanups, and call it from both Tauri
  `delete_conversation` and TUI `/delete-session`. Delete the duplicated
  inline sequences in both call sites.
- Regression validation: New test asserting that after `/delete-session`,
  `tool_executions.summaries_for_conversation(id)` is empty and the detail
  directory does not exist. Tauri side keeps its existing behavior.
- Validation reports: [V04-01](../validations/A-STATE-01/V04-01.md)

### A-STATE-01-P3-01: Tauri `save_conversation` performs get-then-update-then-save without a single transaction (lost-update window)

- Priority: P3
- Confidence: medium
- Layer: application
- Evidence:
  `echo-agent-cli/src/tauri/commands/conversations.rs:411-444`:
  `get_conversation` → `update_conversation` → `get_messages` →
  `project_saved_messages` → `save_messages`. Each call acquires and
  releases the framework `Mutex<StoreMeta>` independently.
- Reachability: every Tauri `save_conversation` invocation.
- Expected invariant: a save should be atomic with respect to concurrent
  saves of the same conversation; otherwise the second writer wins silently.
- Observed behavior: two concurrent `save_conversation` calls for the same
  `id` race. Both read the existing record, both project on top of the same
  baseline, both call `save_messages` — the second `save_messages`
  overwrites the first because `FileConversationStore::save_messages`
  (`file_conversation.rs:350-376`) replaces `record.messages` wholesale
  rather than merging.
- Impact: Low for the local-assistant threat model (single user, the
  frontend serializes its own saves), but a frontend bug that fires two
  rapid saves — or a Tauri command retry — silently drops one batch of
  messages. The framework's `Mutex` only guarantees each *operation* is
  atomic, not the multi-step Tauri sequence.
- Root cause: the Tauri command treats a non-atomic check-then-act sequence
  as if it were transactional.
- Direction: Either (a) accept the documented single-writer assumption and
  add a per-conversation application-level `Mutex` (or `RwLock` write guard)
  held across the get-update-save sequence, or (b) document on the
  `save_conversation` command that concurrent invocations for the same id
  are undefined and the frontend must serialize. (a) is the lower-risk
  option and is one small change.
- Regression validation: Test spawning two concurrent `save_conversation`
  calls and asserting both batches appear in the final `get_messages`.
- Validation reports: [V03-01](../validations/A-STATE-01/V03-01.md)

### A-STATE-01-P3-02: UI-only thinking segments and execution rounds do not reach the agent runtime on restore

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/tauri/commands/conversations.rs:680-724`
    (`restore_conversation`) calls framework `echo_agent::memory::restore_messages(&stored)`
    which only reads the canonical fields (`role`, `content`,
    `tool_calls_json`, `tool_result_json`, and the `_echo_message_version`
    projection envelope). It does not read the UI `thinking_segments` /
    `execution_rounds` keys.
  - `echo-agent/echo-state/src/memory/conversation.rs:169-191`
    (`restore_projection_meta`) explicitly ignores any
    `attachments_json` payload that lacks `_echo_message_version`.
- Reachability: every Tauri `restore_conversation` after a Tauri
  `save_conversation` where the assistant message had thinking segments or
  ordered tool rounds.
- Expected invariant: a restore that returns `message_count = N` should
  reload the same N messages the user sees, including reasoning traces, so
  the agent continues with full context.
- Observed behavior: On the very first save of a turn, the agent runtime
  has not yet called `project_message`, so `has_canonical_transcript` is
  false and `project_saved_messages` writes a non-canonical record whose
  `attachments_json` carries only UI keys (`thinking_segments`,
  `execution_rounds`). When `restore_conversation` later rebuilds the
  runtime messages, `restore_projection_meta` returns `None` for those
  records, so the agent's `Message.reasoning_content` is empty even though
  the frontend still shows the thinking text via `get_conversation`.
- Impact: Low. The LLM loses its prior reasoning trace on reload, which can
  degrade continuity for chain-of-thought tasks. The user-facing UI is
  unaffected because `get_conversation` reads the UI projection directly.
  In practice the framework `RuntimeStateStore` (constructed at
  `infra.rs:1251-1267`) is the preferred resume path and stores the full
  runtime message including `reasoning_content`; this gap only bites when
  `restore_conversation` is used instead of the runtime-state resume.
- Root cause: `project_saved_messages` has two branches — UI-only and
  canonical-aware — but only the canonical branch produces records that
  `restore_message` can fully read. The Tauri command does not require the
  framework to project the assistant message before saving UI metadata.
- Direction: When `has_canonical_transcript` is false, still write
  `reasoning_content` from the first thinking segment into a
  `_echo_message_version` envelope (best-effort single-string form), so
  `restore_message` can recover at least one reasoning block. Leave the
  richer `execution_rounds` UI key untouched for the frontend. Document the
  limitation in `restore_conversation`.
- Regression validation: New test: save an assistant message with
  thinking segments via `project_saved_messages` (no canonical transcript),
  call `restore_messages`, assert `reasoning_content` is non-empty.
- Validation reports: [V03-01](../validations/A-STATE-01/V03-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Format, authority, and duplicate-storage audit | yes | passed | [V01-01](../validations/A-STATE-01/V01-01.md) |
| V02 | Atomic-write recipe audit (application vs framework) | yes | failed | [V02-01](../validations/A-STATE-01/V02-01.md) |
| V03 | Message/tool/thinking round-trip and concurrency | yes | inconclusive | [V03-01](../validations/A-STATE-01/V03-01.md) |
| V04 | Deletion cascade (Tauri vs TUI vs search) | yes | failed | [V04-01](../validations/A-STATE-01/V04-01.md) |
| V05 | Historical-document drift | conditional | not_applicable | — |

V05 is not applicable: there is no prior A-STATE-01 report and no separate
design document for conversation persistence. The two module docstrings
(`persistence.rs:1-8`, `conversation_file.rs:1-13`) make falsifiable claims
that are classified inline in the Historical Claim Status table.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `persistence.rs:1-8` "提供基于 JSON 文件的会话存储" | stale | The `Persistence` API is no longer invoked in production — see P2-01. The framework `FileConversationStore` is the actual authority. |
| `persistence.rs:355-360` "Atomic write: write to temp file then rename" | stale as a *contract* | The recipe exists but is missing fsync + parent-dir sync (F-MEM-01 P2-01 prior). And no caller exercises it. |
| `conversation_file.rs:1-13` "The framework's FileConversationStore is the authority for storage" | current | Confirmed by V01-01. |
| `conversation_file.rs:9-12` "in-memory substring index ... drives the sessions-search UI" | stale | The UI does not call `SessionSearchEngine`; it calls `FileConversationStore::search_conversations` directly (V01-01). |
| `conversation_file.rs:138-142` "malformed records are skipped (best-effort reindex)" | current | Confirmed at lines 144-165. The framework surfaces the error on direct access (V02-01 of F-MEM-01). |
| `src/tauri/commands/conversations.rs:39-42` "is_framework_projection ... _echo_message_version" | current | Confirmed by V03-01 and the framework `restore_projection_meta` at `echo-state/src/memory/conversation.rs:179`. |
| F-MEM-01 P2-01/P2-02 "framework FileStore/EmbeddingStore omit parent-dir fsync + static temp names" | current | Recurring pattern — the application `Persistence::write_json` repeats both defects. See V02-01. |

## Coverage And Uncertainty

- **Runtime state store** (`FileRuntimeStateStore`, used for full
  checkpoint/resume with plan + skills) is explicitly out of scope; only
  the conversation transcript store was inspected. F-RCT-05 / X-STA-01 own
  the runtime-state path.
- **Executable tests not run.** All four validations are static code +
  grep + test-inspection. The P2-02 cascade gap should be confirmed by an
  executable test that runs `/delete-session` and then queries
  `tool_executions.summaries_for_conversation`.
- **`Persistence` workspace migration.** `workspace/migration.rs:77-189`
  reads the sessions directory layout directly (not via `Persistence`
  methods), so P2-01's deletion recommendation must preserve the on-disk
  directory shape (i.e. `<base>/sessions/conversations/...`) — confirmed
  by inspecting that path.
- **`export_conversation` Tauri command** (`src/tauri/commands/conversations.rs:643-677`)
  builds a Markdown export directly from `store.get_messages`; it does not
  use `Persistence::export_conversation_markdown`. The latter is dead (P2-01).
- **Multi-process safety** is explicitly out of scope per AGENTS.md local-
  assistant threat model; the framework `Mutex` is sufficient.
- **Frontend reducer behavior** (does the GUI correctly render
  `thinking_segments` after reload?) is owned by A-FE-01/A-FE-02; only the
  Rust-side projection was inspected here.

## Handoff

- Downstream tasks may rely on:
  - The framework `FileConversationStore` is the single conversation storage
    authority; `Persistence` and `SessionSearchEngine` are unused (V01-01).
  - The canonical round-trip via `project_message`/`restore_message` is
    lossless for the four canonical roles (V03-01, cross-checks F-MEM-01
    V04-01).
  - The Tauri deletion cascade is correct on the conversation store side;
    the TUI side is missing the `tool_executions.remove_conversation` step
    (V04-01, finding P2-02).
- Downstream tasks must read: V01-01 (authority map), V04-01 (cascade gap).
- This report becomes stale if:
  - `Persistence` or `SessionSearchEngine` gains real production callers
    (resurrection path of P2-01).
  - The Tauri/TUI deletion cascade is consolidated.
  - The Tauri `save_conversation` becomes transactional.
- Follow-up task IDs (no fixes implemented here):
  - **A-SRF-01 / X-SRF-01**: should pick up the TUI/GUI deletion-cascade
    parity gap (P2-02).
  - **A-FE-01**: should pin the field-level contract for
    `AttachmentsPayload` round-trip in the TypeScript DTOs.
  - **X-STA-01**: identity continuity across restart — should incorporate
    the finding that `restore_conversation` does not restore UI-only
    reasoning segments into the agent runtime (P3-02).
  - A dedicated cleanup task should execute P2-01 option 1 (delete the
    dead `Persistence` methods and the entire `SessionSearchEngine`).
