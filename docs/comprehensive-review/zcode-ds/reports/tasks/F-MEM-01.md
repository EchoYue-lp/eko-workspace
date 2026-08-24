# F-MEM-01: General memory and conversation stores — durability, atomicity, path-safety, semantic alignment

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean (both repositories)

## Question

Are the `Store`/`ConversationStore` contracts and their in-memory/file
implementations durable, atomic, path-safe, and semantically aligned?

## Scope

- Trait layer: `echo-agent/echo-core/src/memory/` — `store.rs` (Store,
  StoreItem, SearchMode, rrf), `conversation.rs` (ConversationStore +
  data types), `mod.rs`, `scope.rs`, `types.rs`.
- Implementation layer: `echo-agent/echo-state/src/memory/` — `store.rs`
  (InMemoryStore, FileStore), `file_conversation.rs`
  (FileConversationStore, safe_segment, atomic_write), `conversation.rs`
  (project/restore_message), `embedding_store.rs` (EmbeddingStore Store
  impl + index persistence), `typed_store.rs` (wrapper, matrix only),
  `mod.rs`.
- Facade: `echo-agent/src/memory.rs` (re-export of `echo_state::memory`).
- Reachability: `echo-agent/src/agent/react/mod.rs:755-778`
  (framework auto-FileStore), `echo-orchestration/src/scheduler/cron_task.rs`
  (CronTaskStore over Store), EKO construction sites
  `echo-agent-cli/src/tauri/desktop.rs:177`, `src/cli/modes.rs:48`,
  `echo-agent-cli/echo-agent-app-core/src/infra.rs:1215-1350`,
  `echo-agent-app-core/src/state.rs:905,1079`.

## Out Of Scope

- SQLite backends (`echo-state/src/memory/sqlite_store.rs`,
  `sqlite_conversation.rs`) — task F-MEM-02.
- `FileRuntimeStateStore` / root `src/state/` runtime checkpoints (task
  F-RCT-05 / state-store task) — spot-checked only for the MASTER-PLAN
  claim in V03.
- Memory promotion / compression consumers (evolution layer), snapshot,
  embedder HTTP client.
- This review evaluates correctness only; it proposes no deletion of
  framework capabilities (per AGENTS.md deletion rules).

## Inputs

- Root `AGENTS.md`, shared `REPORTING.md`, `TASKS.md` (F-MEM-01 card),
  `zcode-ds/README.md`.
- Dependency report: zcode-ds `F-CORE-01` (event envelope / identity
  contracts — event_id determinism feeds persistence idempotency, no
  direct interaction with the stores reviewed here).
- Dependency report: zcode-ds `B-ARCH-01` (facade ownership — memory
  facade `src/memory.rs` is a pure re-export, consistent with its
  placement conclusions).
- Historical documents treated as hypotheses: `echo-agent-cli/docs/MASTER-PLAN.md`
  Iteration 3 row (line 70) and the S3 paragraph (lines 449-451).

## Layering Decision

- Generic mechanism: `Store`/`ConversationStore` traits plus
  `InMemoryStore`/`FileStore`/`EmbeddingStore`/`FileConversationStore`
  are framework capabilities, correctly placed in `echo_core`/`echo_state`
  (echo-agent-cli MASTER-PLAN Iteration 3 explicitly migrated the file
  backends down to the framework; EKO consumes them without SQLite).
- EKO product policy: store *selection* and failure fallback
  (`create_memory_store_at` returning None → memory disabled,
  `create_conversation_store` → None → persistence disabled,
  desktop/modes falling back to `InMemoryStore`) are application-layer
  decisions; `SessionSearchEngine` and UI projections stay in EKO.
- Adapter boundary: EKO constructs framework stores directly
  (infra.rs:1215, 1332; state.rs:905, 1079) — thin construction, no
  state authority in the adapter; `CronTaskStore`
  (echo-orchestration) is a framework adapter over `dyn Store`.
- Duplicate search terms: `Store`, `ConversationStore`, `InMemoryStore`,
  `FileStore`, `FileConversationStore`, `SearchMode`, `StoredMessage`,
  `safe_segment`, `atomic_write`, `search_conversations`,
  `prune_expired`, `dedup_by_content`, `restore_message`. Single
  authoritative definitions in echo_core/echo_state; `safe_segment` is
  duplicated (file_conversation.rs:465, src/state/file.rs:253) with
  identical logic — duplication, not a second authority.

## Current Path

- Memory: `ReactAgentBuilder::setup_memory_store`
  (`echo-agent/src/agent/react/mod.rs:755-778`) constructs
  `FileStore::new(config.memory_path)` (default
  `~/.echo-agent/store.json`, config.rs:246; EKO overrides with
  workspace `.eko/memory/store.json` via infra.rs:1332-1350) and
  optionally wraps it in `EmbeddingStore`; `remember`/`recall`/
  `search_memory`/`forget` tools and evolution layers write through
  `Store::put` → `FileStore::flush` (store.rs:254-278) on every mutation.
- Scheduler: EKO builds the cron store at
  `Persistence::base_dir()/scheduler_store` in both GUI
  (desktop.rs:175-180) and CLI (modes.rs:47-53); `SchedulerRunner` /
  `CronTaskStore::save_all` (echo-orchestration/src/scheduler/cron_task.rs:141-160)
  does `backend.put(...)` → `FileStore::put` → flush per save.
- Conversations: `FileConversationStore` constructed at
  `user_data_dir()/conversations` (infra.rs:1215-1227) and workspace
  switch paths (state.rs:905, 1079); TUI/GUI session history reads via
  the trait; `SessionSearchEngine` reindexes from the JSON files on start.
- Every file backend operation is serialized in-process (RwLock for
  FileStore, Mutex for FileConversationStore); no cross-process locking
  anywhere.

## Findings

### F-MEM-01-P1-01: FileStore silently discards a corrupt/truncated store file and overwrites it with empty state on the next write

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/store.rs:235-238`
  (`serde_json::from_str(&raw).unwrap_or_else(|e| { warn!; HashMap::new() })`);
  `:254-278` (flush writes the full in-memory state, including the empty
  state, over the original file); `:258` (single fixed tmp name)
- Reachability: `FileStore::new` is the framework's default long-term
  memory store — `echo-agent/src/agent/react/mod.rs:757` (auto-store for
  every agent with `enable_memory`) and EKO production sites
  `echo-agent-cli/echo-agent-app-core/src/infra.rs:1332`
  (workspace memory store), `echo-agent-cli/src/tauri/desktop.rs:177` and
  `src/cli/modes.rs:48` (scheduler store, written on every cron
  add/remove/status change via `CronTaskStore::save_all`
  echo-orchestration/src/scheduler/cron_task.rs:154-160). EKO's
  fallback guards (infra.rs:1332-1350, desktop.rs:177-179, modes.rs:48-52)
  only react to `Err` from `new()`; corruption returns `Ok` so the guards
  never trigger.
- Expected invariant: a corrupt/truncated store file surfaces an explicit
  error and is never overwritten (the pattern the project itself hardened
  into `FileConversationStore`, file_conversation.rs:20-22 and :153-157,
  with a passing test at :624-641).
- Observed behavior: warn + start from empty; the next `put`/`delete`/
  `prune_expired`/`dedup_by_content`/`flush_public` atomically overwrites
  the corrupt file with the empty state — all long-term memory (or cron
  definitions) permanently lost with no error surfaced.
- Impact: any truncation/corruption of `store.json` or `scheduler_store`
  (partial write, disk issue, manual edit, downgrade) silently erases the
  entire memory/scheduler dataset at the next write. No user-visible
  signal; recovery impossible after overwrite.
- Root cause: legacy lenient-parse pattern predating the Iteration-3
  hardening ("explicit corrupt-JSON errors" was applied to the migrated
  impls but not to the pre-existing FileStore); no pre-flush integrity
  check or backup.
- Direction: make `FileStore::new` return
  `MemoryError::SerializationError` on unparseable content (mirror
  file_conversation.rs:153-157); additionally refuse the first flush when
  the loaded file failed to parse, or move the corrupt file aside
  (`.corrupt` backup) before overwriting; add FileStore corrupt-file
  tests (currently none exist — V04-01).
- Regression validation: test A — write truncated JSON, `FileStore::new`
  errors; test B — corrupt file + `put` → error and original bytes
  untouched; existing 43 echo_state + 14 echo_core tests stay green.
- Validation reports: [V02-01](../validations/F-MEM-01/V02-01.md),
  [V03-01](../validations/F-MEM-01/V03-01.md), [V04-01](../validations/F-MEM-01/V04-01.md)

### F-MEM-01-P2-01: FileStore's write protocol diverges from the hardened file-backend pattern (fixed tmp name, no parent-dir fsync, partial cleanup); two EKO processes on the same store path lose updates silently

- Priority: P2
- Confidence: medium
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/store.rs:258` (tmp name is
  `format!("{}.tmp", self.path.display())` — not unique); `:265-267`
  (write/sync failure returns via `?` leaving the tmp file orphaned);
  `:272-277` (rename with cleanup, but **no parent-directory fsync after
  rename**); contrast the hardened `atomic_write` in
  `echo-state/src/memory/file_conversation.rs:494-523` (uuid temp,
  cleanup on both failure paths, parent sync on Unix)
- Reachability: EKO GUI and CLI both construct a FileStore at the same
  path `Persistence::base_dir()/scheduler_store`
  (`echo-agent-cli/src/tauri/desktop.rs:175` and
  `echo-agent-cli/src/cli/modes.rs:47`); each instance keeps its own full
  in-memory copy (store.rs:220-252) with no file locking; last flusher
  wins. Memory store likewise per-process
  (infra.rs:1332).
- Expected invariant: atomic write = unique temp name + fsync + rename +
  parent-dir fsync + cleanup on all failure paths (MASTER-PLAN Iteration 3
  claim, echo-agent-cli/docs/MASTER-PLAN.md:70); single-writer assumption
  documented where not guaranteed.
- Observed behavior: content-level crash consistency holds (tmp+fsync+
  rename), but: (a) two processes writing concurrently race on the fixed
  `.tmp` name (one rename may fail with NotFound); (b) without parent
  fsync the rename itself can be lost on power failure, silently reverting
  to the previous state; (c) write/sync failure leaves `.tmp` orphaned;
  (d) a second process with a stale in-memory snapshot overwrites the
  first process's accepted writes with no error.
- Impact: concurrent GUI+CLI use (plausible for a local assistant) can
  silently lose cron/scheduler definitions and memory entries; power-loss
  durability weaker than the sibling FileConversationStore despite the
  identical "Atomic write" comment (store.rs:259).
- Root cause: FileStore predates the Iteration-3 hardening and was not
  migrated; the pattern divergence is invisible because both stores
  share the "tmp + fsync + rename" comment.
- Direction: hoist the hardened `atomic_write` (uuid temp + parent sync +
  full cleanup) into a shared helper used by FileStore flush, embedding
  index flush, and FileConversationStore; add a cross-process guard or
  document single-writer assumption; fix the misleading comment.
- Regression validation: unit test with two FileStore instances on one
  path asserting last-write ordering and no lost updates (single-process
  interleaving at least); existing test suites green. Power-failure
  scenarios are out of scope for automated regression.
- Validation reports: [V03-01](../validations/F-MEM-01/V03-01.md)

### F-MEM-01-P3-01: Store namespace encoding is not injective — `["a/b"]` collides with `["a","b"]` and prefix matching is string-level

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/store.rs:59,85,98,128`
  (InMemoryStore `namespace.join("/")` keys), `:153` (`list_namespaces`
  splits on `/` — lossy round-trip), `:148-151` (`k.starts_with(p)`
  prefix filter — `["user1"]` matches `["user10",...]`); FileStore
  identical at `:318,340,352,386,405-416`; the trait doc advertises
  "multi-user/multi-agent isolation" (`echo-core/src/memory/store.rs:3-4`)
- Reachability: current consumers use constant namespaces
  (`["agent","memories"]` evolution WARM_NAMESPACE,
  `["scheduler","cron_tasks"]`, typed-memory store) — no user-controlled
  segment today; any future consumer embedding user/agent strings in
  namespace segments (the documented use case) inherits the collision.
- Expected invariant: distinct namespace vectors are distinct storage
  buckets; `list_namespaces(prefix)` returns exactly the namespaces whose
  segments start with the prefix.
- Observed behavior: `["a/b"]` and `["a","b"]` are the same bucket; a
  namespace stored as `["a/b"]` lists back as `["a","b"]`; prefix
  `["user1"]` includes `["user10","memories"]`.
- Impact: silent cross-isolation visibility and overwrite between
  logically distinct namespaces (latent today, contract-level).
- Root cause: string-encoding of `Vec<String>` keys without escaping or
  validation; InMemoryStore and FileStore share the flaw (no drift between
  them, but the contract is underspecified).
- Direction: reject `/` in namespace segments at `put` time (or escape
  them) and make `list_namespaces`/prefix matching segment-aware; add
  isolation tests for `["a/b"]` vs `["a","b"]` and prefix `user1` vs
  `user10`.
- Regression validation: unit tests asserting the two isolation cases
  against both InMemoryStore and FileStore.
- Validation reports: [V01-01](../validations/F-MEM-01/V01-01.md)

### F-MEM-01-P3-02: FileConversationStore.get_messages does not enforce the trait's "sorted by id ASC" contract

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: trait doc `echo-agent/echo-core/src/memory/conversation.rs:136-137`
  ("sorted by id ASC") vs `echo-agent/echo-state/src/memory/file_conversation.rs:378-386`
  (returns `record.messages` in file order, no sort); ids are assigned in
  call order in `save_messages` (:350-376), and `record.messages = assigned`
  replaces wholesale
- Reachability: normal EKO flows save messages chronologically, so file
  order coincides with id order; the contract breaks only when callers
  supply explicit out-of-order ids (the store's own test
  `supplied_message_ids_advance_the_live_counter` imports id=1000 before
  None→1001 — still ascending; a batch like `[Some(5), Some(3)]` would
  not be). Restored/imported records keep their file order across
  restarts.
- Expected invariant: `get_messages` returns messages ordered by `id` ASC
  regardless of insertion order.
- Observed behavior: insertion order, which equals id order only when ids
  are assigned in-order.
- Impact: consumers keying on id ordering (pagination, diffing,
  `compressed_before_id` alignment) silently get unsorted transcripts
  after any out-of-order import.
- Root cause: file order is the persistence order; the read path never
  sorts.
- Direction: sort by `id` in `get_messages` (stable, ids are `Option<i64>`
  — sort `None` first or document), and add a test with out-of-order
  supplied ids; alternatively re-document the trait contract.
- Regression validation: test saving `[Some(5), Some(3)]` then asserting
  `get_messages` returns id 3 before id 5.
- Validation reports: [V01-01](../validations/F-MEM-01/V01-01.md),
  [V04-01](../validations/F-MEM-01/V04-01.md)

### F-MEM-01-P3-03: EmbeddingStore repeats the silent-corrupt pattern for the vector index and hides the inner store's prune/dedup capabilities

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/embedding_store.rs:117-125`
  (corrupt index file → warn + empty `VecIndex`); `:285-312` (drop-flush
  writes the in-memory index, possibly the empty one, over the corrupt
  file via `write_and_rename` with fixed tmp name
  `path.with_extension("json.tmp")`, no parent fsync — `:316-331`);
  Store impl `:328-480` keeps `prune_expired`/`dedup_by_content` at the
  trait default no-op while its inner store implements both
- Reachability: EKO memory path wraps FileStore in EmbeddingStore when
  embedding env is configured (`echo-agent/src/agent/react/mod.rs:755-778`);
  index is rebuilt only by new `put`s, so after corruption semantic/
  hybrid search silently returns nothing until entries are re-written;
  `prune_expired`/`dedup_by_content` have zero callers today in framework
  or EKO (grep-verified), so the no-op is latent.
- Expected invariant: corrupt index surfaces explicitly or is rebuilt from
  the authoritative inner store; wrapper forwards prune/dedup to the inner
  implementation.
- Observed behavior: silent empty index; the corrupt file is overwritten
  on drop; semantic search degrades silently; wrapper-level prune/dedup
  silently no-ops.
- Impact: EKO memory silently loses semantic search after index
  corruption; latent maintenance trap for prune/dedup callers.
- Root cause: lenient parse pattern copied from FileStore; wrapper does
  not delegate optional trait methods.
- Direction: error or rebuild-from-inner on corrupt index; forward
  `prune_expired`/`dedup_by_content` to the inner store; adopt the
  hardened atomic-write helper (shared with P2-01 fix).
- Regression validation: corrupt-index open test; prune_expired forwarding
  test with an expiring item in the inner store.
- Validation reports: [V02-01](../validations/F-MEM-01/V02-01.md),
  [V01-01](../validations/F-MEM-01/V01-01.md)

No further findings. Panic-safety scan (AGENTS.md rule): the reviewed
production code contains no reachable `unwrap`/`expect`/byte-slice on
untrusted input — `partial_cmp(...).unwrap_or`, `unwrap_or_default`,
`map_err(poison)`, and `str::get(..8)` (returns `Option`, falls back
whole-key, used only for cosmetic tool-result previews in
src/tools/builtin/memory.rs:127,231) are all safe; `safe_segment` is
char-iterator based. The only `assert!` is in test-only `MockEmbedder`.

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Trait implementation matrix + duplicate search | yes | passed (3 drifts → findings) | [V01-01](../validations/F-MEM-01/V01-01.md) |
| V02 | Corrupt/truncated file handling (code + tests) | yes | passed (P1-01 evidence) | [V02-01](../validations/F-MEM-01/V02-01.md) |
| V03 | Path-safe IDs + atomic-write vs MASTER-PLAN claims | yes | passed (P2-01 evidence) | [V03-01](../validations/F-MEM-01/V03-01.md) |
| V04 | `cargo test -p echo_state --lib --locked memory` | yes | passed, exit 0 | [V04-01](../validations/F-MEM-01/V04-01.md) |
| V04 | `cargo test -p echo_core --lib --locked memory` | yes | passed, exit 0 | [V04-02](../validations/F-MEM-01/V04-02.md) |
| V05 | Historical-document drift (MASTER-PLAN Iteration 3) | conditional | folded into V03 | [V03-01](../validations/F-MEM-01/V03-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `echo-agent-cli/docs/MASTER-PLAN.md:70` — migrated file impls use unique temp names, fsync, atomic rename, parent-dir fsync, cleanup on failure, path-safe ids, explicit corrupt-JSON errors | current (for FileConversationStore and FileRuntimeStateStore) | [V03-01](../validations/F-MEM-01/V03-01.md) |
| MASTER-PLAN S3 (line 449-451) — file backends migrated "with bug fixes (corrupt-JSON errors, path-safe IDs, unique temp names, parent-dir sync, Result-returning restore)" | current; the fixes were applied to the migrated impls only — pre-existing FileStore retains the old patterns (P1-01, P2-01) | [V02-01](../validations/F-MEM-01/V02-01.md), [V03-01](../validations/F-MEM-01/V03-01.md) |
| `echo-core/src/memory/store.rs:3-4` — namespace `&[&str]` gives "multi-user/multi-agent isolation" | regressed (collision/prefix defects, latent) | [V01-01](../validations/F-MEM-01/V01-01.md) |

## Coverage And Uncertainty

- `FileRuntimeStateStore` (echo-agent/src/state/file.rs) verified only for
  the MASTER-PLAN write-protocol claim; full review belongs to the
  runtime-state-store task.
- SQLite backends excluded by design (F-MEM-02); semantic drift between
  FileConversationStore and SqliteConversationStore (e.g., duplicate-id
  handling in save_messages) not compared.
- Crash-durability of rename-without-parent-fsync was assessed statically;
  no power-loss harness exists in the repo.
- `list_conversations` sorts by `updated_at` via RFC3339 string compare
  (file_conversation.rs:300) — correct for a single machine/offset, minor
  DST edge not promoted to a finding.
- `restore_projection_meta` (conversation.rs:176-178) silently tolerates
  garbage `attachments_json` lacking the `_echo_message_version` marker —
  intentional (legacy data), but a truncated framework projection is
  indistinguishable from legacy and is silently dropped; noted, not a
  finding (documented design trade-off).
- EKO GUI/TUI message ordering reliance on `get_messages` was not traced
  command-by-command; the P3-02 impact claim is based on the trait
  contract, not a confirmed EKO break.

## Handoff

- Downstream tasks may rely on: FileConversationStore robustness (explicit
  corrupt errors, path-safe ids, atomic writes — V02/V03); FileStore
  silent-corrupt data-loss path (P1-01) with exact reachability
  (react/mod.rs:757, infra.rs:1332, desktop.rs:177, modes.rs:48); the
  cross-process scheduler-store exposure (P2-01).
- `F-MEM-02` should compare SQLite semantics (message id assignment,
  upsert, `compressed_before_id`) against FileConversationStore to close
  the trait-contract drift questions raised in P3-02.
- Iteration roadmap: P1-01 and P2-01 fixes belong in the echo-agent
  framework (echo-state memory) with the regression tests specified;
  EKO-side fallback guards (infra.rs:1332-1350) are already correct once
  `FileStore::new` starts erroring on corruption.
- This report becomes stale if: FileStore/FileConversationStore write or
  parse paths change; the Store trait surface changes; MASTER-PLAN
  Iteration 3 row is rewritten.
- Follow-up task IDs: F-MEM-02 (sqlite comparison), F-RCT-05
  (FileRuntimeStateStore durability), X-BND-01 (facade authority map),
  Q-* dynamic gates for the corrupt-file scenario.
