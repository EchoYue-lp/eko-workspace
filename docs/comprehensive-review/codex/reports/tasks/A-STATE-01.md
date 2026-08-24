# A-STATE-01: Conversation persistence and restore

> Status: complete
> Reviewer: Codex primary reviewer
> Executor: Codex primary reviewer
> Review date: 2026-08-13
> `echo-agent` commit: 3aa7929928442aab91e4dce9c426d909a5f0a1ab
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories were clean during source review;
> nine unrelated echo-agent paths changed concurrently during final integrity
> checking and were excluded without reading, modification, or rollback

## Question

Are EKO file-backed conversations authoritative, atomic, restorable,
searchable, and deleted together with their dependent lifecycle state?

## Scope

- Framework `FileConversationStore` use from EKO and the application-owned UI
  projection around `StoredMessage`.
- GUI save/get/update/restore/delete, frontend auto-save/load/edit/regenerate,
  TUI resume/rewind/fork, AgentPool conversation identity, and terminal
  transcript persistence.
- Authority/format mapping, interleaved writers, corrupt structured fields,
  empty restore, active deletion, search reachability, cleanup ownership, and
  static test inventory.

## Out Of Scope

- Framework file-store corruption, multi-instance locking, namespace, and
  reserved-name defects are owned by [F-MEM-01](F-MEM-01.md).
- Upload ownership and detached artifact cleanup are owned by
  [A-INP-01](A-INP-01.md), especially A-INP-01-P1-05.
- Workspace root/store generation divergence is owned by
  [A-CFG-01](A-CFG-01.md), especially A-CFG-01-P1-02. This report adds precise
  constructor/path evidence but does not duplicate that finding.
- TaskRun/checkpoint authority, frontend rendering design, source fixes, and
  dynamic execution.
- Cargo, rustc, tests, builds, fixtures, and network activity, per the user's
  review-only instruction.

## Inputs

- Root `AGENTS.md`; review `README.md`, `REPORTING.md`, exact A-STATE-01 card in
  `TASKS.md`, and templates.
- Completed Codex dependencies [F-MEM-01](F-MEM-01.md) and
  [A-INP-01](A-INP-01.md), plus A-CFG-01 only for finding ownership.
- Current clean source at the commits above. No other reviewer directory was
  read.

## Layering Decision

| Classification | Current answer |
|---|---|
| Generic mechanism | Atomic record writes, strict structured-message restore, path-safe IDs, and one `ConversationStore` contract correctly belong in `echo-agent`. File and SQLite remain reasonable framework alternatives; EKO must not enable SQLite. |
| EKO product policy | Conversation titles, GUI thinking/tool display metadata, autosave timing, edit/regenerate semantics, workspace root selection, uploads, tool-detail repository, and surface deletion behavior belong in `echo-agent-cli`. |
| Adapter boundary | EKO should add UI metadata by stable message identity and submit a revision/CAS mutation to the canonical transcript. It must not replace the whole Agent-owned transcript from a stale UI snapshot. |
| Duplicate search | Searched both repositories for `ConversationStore`, `FileConversationStore`, `SavedMessage`, projection/restore, every save/update/delete/restore caller, Agent finalization, pool acquisition/eviction, `Persistence`, and `SessionSearchEngine`. |
| Migration deletion | Keep the framework store. Delete the application `SavedSession`/`ConversationRecord` file authority and unused `SessionSearchEngine`; replace GUI whole-transcript writes with one identity/revision-aware metadata mutation, then delete positional reconciliation. |

## Current Authority Map

```text
ReactAgent context (runtime authority)
  -> finalization -> project_messages -> ConversationStore::save_messages
                                            |
GUI ChatMessage[] -> 300 ms autosave -> update_conversation
  -> get_messages -> positional merge -> ConversationStore::save_messages
                                            |
GUI/TUI resume <- get_messages <- restore_messages <- same JSON record
```

The framework store writes one atomic `{conversation, messages}` record and
serializes each method on one instance. The application nevertheless creates a
second whole-record writer. Its `get_messages` and later `save_messages` are
separate locked calls, so the read/merge/write transaction is not atomic with
framework finalization. Stable `message_id` exists only inside optional UI
metadata; canonical records are aligned by role position instead.

Production surfaces are not symmetric:

| Operation | GUI | TUI/CLI |
|---|---|---|
| Normal terminal save | framework finalizer plus frontend autosave | framework finalizer |
| Restore malformed structured field | logs backend restore failure, still displays record | loads empty Agent history, displays record and “Resumed” |
| Restore empty record | reports success without clearing pooled Agent | explicit CLI startup loads empty; TUI command loads empty |
| Edit/regenerate | truncates UI/persistent projection, does not rewind Agent | TUI rewind truncates store and reloads Agent |
| Delete | removes record first; no turn cancel/pool eviction; artifact cleanup detached | removes record and scoped artifacts; no active-Agent ownership transaction |

## Findings

### A-STATE-01-P0-01: GUI autosave and Agent finalization can replace a complete transcript with a stale prefix

- Priority: P0
- Confidence: high
- Layer: application/adapter
- Evidence: `echo-agent-cli/web-frontend/src/stores/chatStore.ts:118`, `:129`,
  `:153`, `:231`; `echo-agent-cli/src/tauri/commands/conversations.rs:65`,
  `:115`, `:566`; `echo-agent/src/agent/react/run/phases/finalize.rs:168`;
  `echo-agent/src/agent/snapshot.rs:652`;
  `echo-agent/echo-state/src/memory/file_conversation.rs:350`.
- Reachability: adding a user/assistant/tool fact schedules the live 300 ms GUI
  autosave. Every successful Agent terminal separately calls framework
  transcript persistence for the same conversation.
- Expected invariant: one revisioned authority commits each transcript change;
  UI metadata cannot remove canonical messages or tool facts.
- Observed behavior: `update_conversation` reads the record, builds a full
  replacement, then awaits a separate `save_messages`. Framework finalization
  can save a newer complete record between those calls; the delayed GUI write
  then replaces it with the older vector. When a canonical transcript already
  exists, extra pending UI messages are intentionally discarded rather than
  appended, so timing decides which writer wins.
- Impact: a completed response, tool calls/results, or an entire latest turn
  can disappear from durable history after both writers individually report
  success. Crash/reload then makes the loss permanent.
- Root cause: EKO treats a UI projection as a second transcript authority and
  emulates a transaction with two independently locked trait calls.
- Direction: make Agent/framework persistence the only transcript writer. Add
  UI metadata through stable message IDs plus revision/CAS or one store-level
  atomic mutation. Delete whole-vector GUI replacement and positional merge.
- Regression validation: deterministically interleave GUI read, framework
  final save, and GUI write in both orders; assert monotonic revision and exact
  preservation of tool/result/reasoning facts.
- Validation reports: [V03](../validations/A-STATE-01/V03-01.md)

### A-STATE-01-P1-02: GUI edit and regenerate change the display transcript but do not rewind the Agent context

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/web-frontend/src/stores/chatStore.ts:478`, `:510`;
  `echo-agent-cli/web-frontend/src/components/chat/ChatPanel.tsx:82`;
  `echo-agent-cli/web-frontend/src/stores/conversationStore.ts:231`;
  `echo-agent-cli/src/tauri/commands/conversations.rs:544`;
  `echo-agent-cli/src/tauri/commands/chat.rs:489`.
- Reachability: the live message actions call `prepareRegenerate` or
  `prepareEditAndResend`, schedule persistence, and immediately send the new
  prompt through the same conversation's pooled Agent.
- Expected invariant: edit/regenerate chooses one historical boundary and
  applies it to UI, durable transcript, runtime Agent context, checkpoint, and
  tool/task projections before the replacement turn starts.
- Observed behavior: only Zustand messages and the conversation JSON projection
  are truncated/edited. No backend rewind/load/reset is invoked. The pooled
  Agent still contains the removed assistant answer and original user message,
  then receives the replacement prompt as an additional turn.
- Impact: the model answers against hidden history the user believes was
  removed; finalization may restore that hidden history to disk, and retry/edit
  behavior differs from TUI's explicit rewind-and-load path.
- Root cause: frontend presentation mutations are mistaken for runtime history
  mutations.
- Direction: implement one application rewind operation keyed by stable turn
  identity that atomically updates canonical history and loads the resulting
  context before send. Delete frontend-only truncation as an authority.
- Regression validation: edit and regenerate around tool calls, compression,
  active cancellation, restart, and pooled Agent reuse; compare exact model
  input, UI, file record, and checkpoint.
- Validation reports: [V04](../validations/A-STATE-01/V04-01.md)

### A-STATE-01-P1-03: Restore failures can still present a conversation as resumable while the Agent has empty or stale history

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent/echo-state/src/memory/conversation.rs:93`, `:107`;
  `echo-agent-cli/src/tauri/commands/conversations.rs:487`, `:521`, `:703`;
  `echo-agent-cli/web-frontend/src/stores/conversationStore.ts:302`, `:309`,
  `:375`; `echo-agent-cli/src/tui/events.rs:4832`, `:4842`, `:4863`.
- Reachability: GUI history selection fetches UI data and invokes backend
  restore separately. TUI `/resume` directly uses the same stored messages.
- Expected invariant: structured transcript corruption fails closed and the
  surface remains outside live-chat mode until runtime restore succeeds.
- Observed behavior: GUI `get_conversation` silently drops malformed
  attachments/tool-call display JSON; the frontend catches a strict backend
  restore error, then still replaces visible messages and sets history view
  false (“Agent has context”). TUI catches the strict error, loads `Vec::new()`
  into the Agent, displays stored text, and adds a “Resumed conversation” line.
- Impact: the next answer silently lacks history/tool causality visible to the
  user. GUI may retain an existing pooled Agent's unrelated/stale context;
  TUI definitely uses empty context.
- Root cause: display hydration and runtime admission are separate operations,
  and restore failure is downgraded to logging instead of a typed surface state.
- Direction: parse once into a typed restored aggregate; commit both runtime
  and display projections only on success. Expose recovery/export for corrupt
  records without allowing continuation under a false resumed state.
- Regression validation: corrupt role, tool calls, tool result, framework
  projection, and UI projection independently; assert no live continuation and
  preservation of recoverable bytes.
- Validation reports: [V05](../validations/A-STATE-01/V05-01.md)

### A-STATE-01-P1-04: Restoring an empty conversation does not clear a reused pooled Agent

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/src/tauri/commands/conversations.rs:700`, `:702`,
  `:706`; `echo-agent-cli/echo-agent-app-core/src/state.rs:309`;
  `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:296`;
  `echo-agent/src/agent/react/mod.rs:1667`.
- Reachability: GUI can clear a saved record with `update(... messages: [])`,
  while the pool retains the conversation Agent for up to its idle eviction.
  Selecting the record invokes registered `restore_conversation`.
- Expected invariant: a successful restore replaces Agent context with exactly
  the stored vector, including an empty vector.
- Observed behavior: `restore_conversation` calls `agent_for` and
  `load_messages` only inside `if !stored.is_empty()`. It nevertheless returns
  success and `message_count: 0`. An already pooled Agent therefore keeps its
  old context.
- Impact: a visually empty conversation can send hidden prior messages to the
  provider and later repersist them.
- Root cause: empty is treated as “nothing to do” rather than a valid complete
  state replacement.
- Direction: always restore exactly one parsed vector to the identified Agent;
  absence of a record and an empty record must remain distinct.
- Regression validation: create/use/clear/restore one pooled conversation and
  compare exact context before next send; repeat after pool eviction.
- Validation reports: [V06](../validations/A-STATE-01/V06-01.md)

### A-STATE-01-P1-05: Conversation deletion is not coordinated with active turns or pooled Agent ownership

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/src/tauri/commands/conversations.rs:586`, `:595`;
  `echo-agent-cli/web-frontend/src/stores/conversationStore.ts:390`;
  `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:296`, `:744`;
  `echo-agent/src/agent/snapshot.rs:684`, `:703`.
- Reachability: the GUI delete action may target the active conversation; the
  command does not inspect `active_chat_turns`, cancellation tokens, or the
  pool. Framework finalization always `ensure_conversation` before saving.
- Expected invariant: deletion first acquires/cancels conversation ownership,
  prevents subsequent writes for that generation, evicts runtime state, and
  returns a truthful cleanup outcome.
- Observed behavior: the JSON record is removed immediately while the pooled
  Agent and any active finalizer remain. A late finalizer can recreate the
  conversation and save its transcript. A GUI autosave already in flight can
  do the same. There is no tombstone/generation check or targeted pool removal.
- Impact: a deleted conversation can reappear with content, and the supposedly
  deleted Agent context remains live. Upload cleanup incompleteness is separately
  owned by A-INP-01-P1-05.
- Root cause: deletion is a file operation, not a conversation lifecycle
  transition.
- Direction: add one application conversation owner/generation: cancel/join the
  turn, reject stale writers, evict/close the pooled Agent, delete canonical
  record and dependent artifacts, then report complete or durable cleanup-pending.
- Regression validation: delete during streaming, tool execution, terminal
  save, debounced save, and idle pooled reuse; assert no resurrection.
- Validation reports: [V07](../validations/A-STATE-01/V07-01.md)

### A-STATE-01-P2-06: Dormant file persistence and search objects remain in live AppState as misleading second authorities

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/persistence.rs:17`, `:211`,
  `:321`; `echo-agent-cli/echo-agent-app-core/src/conversation_file.rs:40`,
  `:138`; `echo-agent-cli/echo-agent-app-core/src/state.rs:363`, `:495`, `:894`;
  `echo-agent-cli/src/tauri/commands/conversations.rs:727`.
- Reachability: `Persistence` and `SessionSearchEngine` are constructed and
  retained in every `AppState`; workspace switching replaces `Persistence`.
  Whole-repository caller search finds no production call to its session or
  conversation CRUD and no call to search-engine index/remove/search after
  startup. Registered GUI/TUI search calls `ConversationStore` directly.
- Expected invariant: EKO has one live file conversation authority; obsolete
  application stores are absent rather than kept as plausible state.
- Observed behavior: the old `~/.eko/sessions` JSON contract and an in-memory
  index remain public/live fields despite being disconnected. Comments claim
  `AppState.search_engine` and Tauri callers remain, but no such caller exists.
- Impact: maintenance and workspace changes update objects that cannot affect
  user behavior, encouraging future code to reconnect a second schema or stale
  index. It also obscures the actual global/workspace root bug owned by
  A-CFG-01-P1-02.
- Root cause: the framework-store migration retained old application state and
  compatibility comments after consumers moved.
- Direction: delete `Persistence` session/conversation CRUD, the unused state
  field, `SessionSearchEngine`, and re-export after confirming scheduler paths
  use an independent canonical path helper. Do not delete framework store
  options and do not add SQLite to EKO.
- Regression validation: static no-reference gate plus GUI/TUI global/workspace
  list/search/restore checks through only `ConversationStore`.
- Validation reports: [V08](../validations/A-STATE-01/V08-01.md)

## Positive Conclusions

- Current framework `FileConversationStore` fails closed for malformed full
  conversation records, atomically renames record files, sorts list/search by
  update time, and restores canonical role/tool/multimodal/reasoning data.
- EKO correctly does not enable or require SQLite; the framework SQLite option
  remains valid for unrelated consumers.
- TUI rewind truncates durable messages and reloads the resulting runtime
  vector, which is the right shape for GUI edit/regenerate convergence.
- The GUI projection preserves canonical content/tool records when its single
  read is current; the defect is identity/transaction ownership, not the intent
  to retain display metadata.

## Validation Matrix

| ID | Claim | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition, authority, layering, and duplicate search | yes | passed | [V01](../validations/A-STATE-01/V01-01.md) |
| V02 | Production save/restore/delete reachability map | yes | passed | [V02](../validations/A-STATE-01/V02-01.md) |
| V03 | Interleaved GUI/framework writer invariant | yes | failed | [V03](../validations/A-STATE-01/V03-01.md) |
| V04 | Edit/regenerate runtime-history parity | yes | failed | [V04](../validations/A-STATE-01/V04-01.md) |
| V05 | Corrupt structured-field restore behavior | yes | failed | [V05](../validations/A-STATE-01/V05-01.md) |
| V06 | Empty-record exact restore | yes | failed | [V06](../validations/A-STATE-01/V06-01.md) |
| V07 | Deletion generation/cascade ownership | yes | failed | [V07](../validations/A-STATE-01/V07-01.md) |
| V08 | Legacy authority and search reachability | yes | failed | [V08](../validations/A-STATE-01/V08-01.md) |
| V09 | Existing tests and historical claims | yes | passed | [V09](../validations/A-STATE-01/V09-01.md) |
| V10 | Targeted dynamic fault/concurrency matrix | conditional | not_run | [V10](../validations/A-STATE-01/V10-01.md) |
| V11 | Exact-ID/link/header/committed-source isolation | yes | passed | [V11](../validations/A-STATE-01/V11-04.md) |

## Historical Claim Status

| Claim | Classification | Current evidence |
|---|---|---|
| `MASTER-PLAN`: framework FileConversationStore is the authority and EKO keeps only UI projection/search | regressed | Framework is the intended authority, but GUI remains a full-vector writer and obsolete state remains; V01/V03/V08. |
| `snapshot.rs`: product layers should rely on framework finalization instead of reimplementing `save_messages` | regressed | GUI save/update still calls `save_messages`; V03. |
| `conversation_file.rs`: AppState/Tauri callers use SessionSearchEngine | stale | only construction/reindex and tests remain; V08. |
| File record writes and canonical projection are atomic/lossless | current with application caveat | Framework implementation is positive; EKO interleaved whole-record replacement violates end-to-end atomicity; V01/V03. |
| Workspace stores switch together | current defect owned elsewhere | A-CFG-01-P1-02; constructor also receives an already-suffixed conversation directory and exit uses sessions base. |

## Coverage And Uncertainty

- The interleavings, branching conditions, and missing calls are
  source-conclusive. No timing or filesystem fault was executed.
- Dynamic writer scheduling, corrupt-file fixtures, and active-delete replay are
  intentionally `not_run` and become required implementation regressions.
- AppState's obsolete fields are source-conclusive; whether external Rust code
  outside this repository imported the public app-core types is not knowable.
  This is application code under the repository's no-compatibility policy.
- Full record corruption is not duplicated from F-MEM-01. This task specifically
  covers EKO's structured-field restore/display divergence.

## Handoff

- `A-SRF-02`/`A-SRF-03`: consume restore-success/display-context split and
  edit/regenerate authority facts.
- `A-MEM-01`: consume only workspace store identity; dynamic memory is separate.
- `X-SRF-01`: require edit/restore/delete parity across GUI/TUI/CLI/channel.
- Roadmap: first establish one revisioned transcript mutation and conversation
  generation owner; then converge GUI rewind/restore/delete and remove obsolete
  state. Preserve framework/app boundary and EKO's file-only persistence choice.
