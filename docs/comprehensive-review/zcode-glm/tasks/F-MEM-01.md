# F-MEM-01: General memory and conversation stores

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa
> `echo-agent-cli` commit: not-applicable
> Worktree state: clean (read-only review)

## Question

Are the `Store` / `ConversationStore` contracts and the in-memory / file
implementations durable, atomic, path-safe, and semantically aligned?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/echo-core/src/memory/store.rs` (full, 1-736) — `Store` trait,
  `StoreItem`, `SearchMode`, `SearchQuery`, RRF scoring.
- `echo-agent/echo-core/src/memory/conversation.rs` (full, 1-205) —
  `ConversationStore` trait, `Conversation`, `StoredMessage`,
  `ConversationFilter`, default `search_conversations` / `ensure_conversation`.
- `echo-agent/echo-core/src/memory/types.rs` (full) — `MemoryMeta`,
  `TypedMemoryValue`, lifecycle enums.
- `echo-agent/echo-core/src/memory/scope.rs` (full) — `MemoryScope` lifetimes.
- `echo-agent/echo-state/src/memory/store.rs` (full, 1-736) — `InMemoryStore`,
  `FileStore`.
- `echo-agent/echo-state/src/memory/file_conversation.rs` (full, 1-780) —
  `FileConversationStore`, `safe_segment`, `atomic_write`, meta reconciliation.
- `echo-agent/echo-state/src/memory/conversation.rs` (full, 1-220) —
  `project_message` / `restore_message` round-trip.
- `echo-agent/echo-state/src/memory/snapshot.rs` (full, 1-305) —
  `SnapshotManager`, `StateSnapshot`.
- `echo-agent/echo-state/src/memory/typed_store.rs` (full) — `TypedMemoryStore`.
- `echo-agent/echo-state/src/memory/embedding_store.rs` (selections:
  170-208, 320-330) — atomic-write comparison.
- `echo-agent/echo-state/src/memory/mod.rs` (full) — re-export layering.

## Out Of Scope

Deferred to downstream tasks:

- **F-MEM-02**: SQLite backend (`sqlite_store.rs`, `sqlite_conversation.rs`)
  — its concurrency model, schema, feature gates, and semantic-search edge
  cases. Per AGENTS.md, SQLite is a valid framework option for non-CLI
  consumers; it is neither flagged for deletion here nor audited for
  SQLite-specific defects.
- **F-CTX-01 / F-CMP-01**: context compression and `ContextManager`.
- **A-MEM-01**: application-layer memory policies (DomainProfile, reviewer,
  promotion).
- Runtime checkpoints / `RuntimeStateStore` (cross-process resume) — out of
  memory-store scope per `echo-state/src/memory/mod.rs:11-13`.

## Inputs

- Required repository documents read:
  - `AGENTS.md` (root) — framework/application boundary, "Store trait has
    multiple implementations" guidance, UTF-8 safety rule, panic-safety rule.
  - `docs/comprehensive-review/REPORTING.md` and
    `docs/comprehensive-review/templates/{task-report,validation-report}.md`.
- Dependency task reports: F-CORE-01 has no `Store`-specific conclusions
  (grep returned no hits); this report is the first framework-memory audit.
- Historical documents: none directly cited; module docs treated as
  hypotheses and verified against code.

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism | The `Store` / `ConversationStore` traits and the three concrete backends (`InMemoryStore`, `FileStore`, `FileConversationStore`) are generic agent-framework capabilities. Any `echo-agent` consumer may need pluggable memory backends. They belong in the framework (`echo-core` traits, `echo-state` impls). |
| EKO product policy | None here. EKO picks `FileStore`/`FileConversationStore` because the CLI does not use SQLite, but that selection lives at the application layer; the framework correctly offers all backends as a menu. |
| Adapter boundary | Not applicable — no EKO adapter is in scope. `TypedMemoryStore` is a thin decorator that serializes `MemoryMeta` into `StoreItem.value` JSON without changing the `Store` contract. |
| Duplicate search | `pub trait Store`, `pub trait ConversationStore`, `impl Store for`, `impl ConversationStore for`, `safe_segment`, `atomic_write`, `fs::rename`, `sync_all`, `sync_parent_directory`, `project_message`, `restore_message`, `SnapshotManager`, `StateSnapshot`. Result: single trait definition per concept; impl matrix in V01-01. |
| Migration deletion | No deletion recommended by this task. SQLite stays (F-MEM-02 scope). |

## Current Path

**Store trait** (`echo-core/src/memory/store.rs:182-257`): eight methods, two
with default impls — `search_with` (dispatches to `search` for keyword mode,
returns `MemoryError::Unsupported` for semantic/hybrid), `prune_expired` and
`dedup_by_content` (no-op defaults). Trait is `Send + Sync` with async
`BoxFuture` returns.

**Store impls** (`echo-state/src/memory/store.rs`):
- `InMemoryStore` (20-215): `RwLock<HashMap<ns_key, HashMap<key, StoreItem>>>`.
  Pure in-process; `put_raw` (43-48) preserves caller timestamps for tests.
- `FileStore` (220-483): same HashMap shape, persisted as a single JSON blob.
  Every mutating op (`put`, `delete`, `put_batch`, `prune_expired`,
  `dedup_by_content`) calls `flush()` (254-278), which serializes the entire
  map to `<path>.tmp`, `sync_all`s the temp, and `rename`s into place.
- `EmbeddingStore` (`embedding_store.rs:328`): decorator wrapping an inner
  `Store`, overrides `search_with` for semantic/hybrid via a vector index
  persisted to `vec_path` with its own `flush_index` (181-208).
- `SqliteStore` (`sqlite_store.rs:464`, `sqlite` feature): out of scope.

**ConversationStore trait** (`echo-core/src/memory/conversation.rs:98-205`):
CRUD over `Conversation` + `StoredMessage`. Default impls:
`ensure_conversation` (get-or-create) and `search_conversations` (naive scan
of all conversations, fallback for backends without FTS).

**FileConversationStore** (`echo-state/src/memory/file_conversation.rs`):
one JSON file per conversation under `<base>/conversations/<safe_id>.json`,
plus `_meta.json` monotonic id counter. All ops serialize through an
in-process `Mutex<StoreMeta>` (70-71). `read_meta` (100-138) self-heals the
counter by scanning records on startup. `atomic_write` (494-523) does the
full durable-write recipe: uuid-suffixed temp, `sync_all` temp, rename,
`sync_parent_directory` (Unix). Corrupt JSON surfaces as
`MemoryError::SerializationError` (149-166, 215-242). `safe_segment`
(465-486) rejects empty / `/` / `\` / `..` / `.` / non-`[A-Za-z0-9\-_.:~]`.

**Projection layer** (`echo-state/src/memory/conversation.rs`):
`project_message` (35-88) → `StoredMessage` with optional versioned
`MessageProjectionMeta` envelope in `attachments_json` (version 1);
`restore_message` (107-167) is the inverse. Unknown role or wrong projection
version is an error, not silent. See V04-01 for round-trip evidence.

**SnapshotManager** (`echo-state/src/memory/snapshot.rs`): in-memory ring
buffer of `StateSnapshot { id, iteration, messages, metadata, created_at }`.
Pure value semantics — capture clones, rollback clones. Not persisted (by
design — module doc at `echo-state/src/memory/mod.rs:11-13` points
cross-restart recovery users to `RuntimeStateStore`).

**Live callers** (grep):
`SnapshotManager` is consumed by `echo-agent/src/agent/react/{mod,builder}.rs`
and `echo-agent/src/agent/react/subsystems/memory.rs` for in-run rollback.
`Store` / `FileStore` / `FileConversationStore` are re-exported by
`echo-agent/src/lib.rs:201-202` and used downstream by the application and by
`ContextManager` (compression promotes evicted messages into a `Store`).

## Findings

### F-MEM-01-P1-01: FileStore silently swallows corrupt JSON on load (data loss)

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/store.rs:232-241`
- Reachability: `FileStore::new` is the only constructor; called wherever a
  persistent JSON-backed `Store` is needed (re-exported at
  `echo-agent/src/lib.rs:201`). Any caller that points `FileStore` at an
  existing corrupted file triggers this path on startup.
- Expected invariant: A persistent store must surface disk corruption as an
  error so the caller can recover, alert, or fail loudly. The sister
  implementation `FileConversationStore` documents and enforces this
  (`file_conversation.rs:18-22`: "Corrupt JSON is an error ... rather than
  silently returning None / an empty list").
- Observed behavior: `FileStore::new` parses the existing file with
  `serde_json::from_str(&raw).unwrap_or_else(|e| { tracing::warn!(...);
  HashMap::new() })` (store.rs:235). A truncated or malformed file is
  indistinguishable from a fresh install. The next mutation triggers
  `flush()` (254-278), which writes the now-empty map back over the
  corrupted file via `tokio::fs::rename`, destroying any chance of manual
  recovery.
- Impact: Silent permanent loss of all long-term memory in that file. The
  user sees only a `warn!` log line; the application continues running with
  an empty store, and the next flush makes the loss unrecoverable. A local
  disk error or partial write (e.g. previous crash between temp-write and
  rename, or external editor leaving the file half-written) becomes total
  data loss.
- Root cause: `FileStore::new` was written before the corrupt-as-error
  contract was codified for `FileConversationStore`; the two backends never
  converged.
- Direction: Replace `unwrap_or_else` at store.rs:235 with
  `.map_err(|e| MemoryError::SerializationError(format!("parse store file {}: {e}", path.display())))?`,
  matching `FileConversationStore::read_record`. Add a regression test
  (`std::fs::write(path, b"{ bad"); assert!(FileStore::new(path).is_err());`)
  mirroring `corrupt_record_surfaces_as_error_not_empty`.
- Regression validation: new unit test asserting `FileStore::new` returns
  `Err` on corrupt input; existing tests still pass (none rely on the
  silent-fallback behavior).
- Validation reports: [V02-01](../validations/F-MEM-01/V02-01.md)

### F-MEM-01-P2-01: FileStore and EmbeddingStore omit parent-directory fsync after rename (less crash-durable than FileConversationStore)

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/store.rs:254-278` (FileStore);
  `echo-agent/echo-state/src/memory/embedding_store.rs:181-208` (EmbeddingStore).
  Contrast with the correct recipe at
  `echo-agent/echo-state/src/memory/file_conversation.rs:494-533`.
- Reachability: Every `FileStore::put` / `delete` / `put_batch` /
  `prune_expired` / `dedup_by_content` and every `EmbeddingStore` mutation
  flows through these flush routines.
- Expected invariant: An "atomic write" should be crash-durable: the
  temp's content fsynced, the rename atomic, and the new directory entry
  durable via a parent-dir fsync. This is the canonical recipe (SQLite,
  `FileConversationStore`, and the `atomic-write` RFC all do it).
- Observed behavior: Both routines fsync the temp file and rename, but
  neither calls `sync_parent_directory` afterwards. On Linux ext4 mounted
  with default options, the rename may not reach disk before a crash even
  though the temp's bytes are durable. `FileConversationStore::atomic_write`
  does call `sync_parent_directory` (file_conversation.rs:521, 525-528).
- Impact: After a crash, the store may appear to revert to a slightly older
  state (or, worse, the file may be missing if the directory entry was
  lost). For `FileStore`, which stores the entire memory map in one file,
  the window covers all data. Local desktop use makes this low-frequency
  but not negligible (power loss, forced reset).
- Root cause: Each backend reimplemented the atomic-write recipe
  independently; only `FileConversationStore` was updated to include the
  parent-dir sync.
- Direction: Factor a single `pub(crate) fn atomic_write(path, bytes)
  -> io::Result<()>` (and `sync_parent_directory`) out of
  `file_conversation.rs` into a shared `echo-state::util` module, then route
  `FileStore::flush` and `EmbeddingStore::flush_index` through it. Delete
  the two now-redundant inline implementations.
- Regression validation: Test asserting `atomic_write` invokes
  parent-dir sync (e.g. via a stubbed `File::sync_all` counter). Existing
  flush tests still pass.
- Validation reports: [V03-01](../validations/F-MEM-01/V03-01.md)

### F-MEM-01-P2-02: FileStore and EmbeddingStore use static temp-file names (cross-instance collision)

- Priority: P2
- Confidence: medium
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/store.rs:258`
  (`format!("{}.tmp", self.path.display())`);
  `echo-agent/echo-state/src/memory/embedding_store.rs:189`
  (`path.with_extension("json.tmp")`). Contrast with the uuid-suffixed
  pattern at
  `echo-agent/echo-state/src/memory/file_conversation.rs:502-506`.
- Reachability: Same as P2-01 — every mutation of either store.
- Expected invariant: A temp file used for atomic rename should have a
  unique name so that two concurrent writers cannot clobber each other's
  bytes (the failure mode where writer A's temp is truncated by writer B
  mid-write, then A renames B's partial content as if it were A's).
- Observed behavior: Both stores derive the temp name purely from the final
  path, so any overlap in final path produces overlap in temp path. The
  in-process `RwLock` / `Mutex` serializes writers within one store instance
  but does NOT coordinate across two `FileStore` instances pointed at the
  same path, nor across two processes.
- Impact: If a caller ever constructs two `FileStore`s on the same path
  (misconfiguration, or a test fixture colliding with the live store, or a
  multi-process setup), flushes race on the same `.tmp` file and can leave
  the store file with mixed/garbled content. `FileConversationStore` calls
  this out explicitly at file_conversation.rs:499-501 ("belt-and-suspenders
  ... multi-process safety") and uses uuid-suffixed temps. Low likelihood,
  medium severity.
- Root cause: Same as P2-01 — recipe drift across reimplementations.
- Direction: Same shared `atomic_write` helper fixes this for free (it
  generates a uuid temp name). Delete the two inline tmp-name constructions.
- Regression validation: Covered by the shared-helper test in P2-01.
- Validation reports: [V03-01](../validations/F-MEM-01/V03-01.md)

### F-MEM-01-P3-01: tokenize filters tokens by byte length, inconsistent with UTF-8 safety rule

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/store.rs:519-520`
- Reachability: `tokenize` is called by both `InMemoryStore::search`
  (store.rs:103) and `FileStore::search` (store.rs:358), i.e. every keyword
  search over both stores.
- Expected invariant: AGENTS.md "Rust 编码硬性约束 §1" — string length
  checks that decide semantic behavior must use `chars().count()`, not
  `str::len()` (which returns bytes). The intent of `s.len() > 1` is
  "drop single-character tokens"; the byte-length check gets this wrong for
  non-ASCII.
- Observed behavior: `.filter(|s| !s.is_empty() && s.len() > 1)` treats a
  single ASCII char (1 byte) as too short to search but a single CJK char
  (3 bytes in UTF-8) as a valid token. So `search(ns, "a", 10)` returns no
  results (assuming no value contains the literal "a" as a token boundary),
  while `search(ns, "中", 10)` happily matches.
- Impact: Minor. Search over single-char ASCII queries is silently a no-op;
  search over single-char CJK queries is not. Inconsistent user-facing
  behavior, no panic, no data risk.
- Root cause: Casual use of `str::len()` for a length filter.
- Direction: Replace `s.len() > 1` with `s.chars().count() > 1`. One-line
  fix. Add a unit test for `search` with a single-char query asserting the
  intended behavior.
- Regression validation: New unit test on `tokenize`/`search` for both
  single-char ASCII and CJK inputs.
- Validation reports: [V01-01](../validations/F-MEM-01/V01-01.md) (scope)

### F-MEM-01-P3-02: search with empty query returns every item with score 1.0

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/store.rs:528-531`
  (`value_relevance_score` returns 1.0 when `keywords.is_empty()`),
  triggered by `tokenize("")` returning an empty vector (517-525).
- Reachability: `InMemoryStore::search` and `FileStore::search` both feed
  `tokenize(query)` into `value_relevance_score`; an empty query yields an
  empty keyword set and the score short-circuits to 1.0 for every item.
- Expected invariant: Either an empty query is documented as "return
  everything" (then the behavior is fine but should be documented on the
  trait), or it should return no results / be rejected. Today the behavior
  is undocumented at the trait level (`Store::search` doc at store.rs:198
  only says "returns at most `limit` items sorted by relevance").
- Observed behavior: `store.search(ns, "", limit)` returns up to `limit`
  items, all with `score = Some(1.0)`. Surprising to callers who expect an
  empty query to mean "no filter" vs "match everything".
- Impact: Minor. No correctness or durability risk. Potential caller
  confusion if they treat empty query as a no-op.
- Root cause: Convenience short-circuit in `value_relevance_score` without a
  matching trait-level doc.
- Direction: Pick one — either document on `Store::search` that empty query
  matches all, or short-circuit `search` to return `Ok(vec![])` on empty
  query. Documentation-only fix is the lower-risk choice.
- Regression validation: Add a doc-test or a unit test pinning the chosen
  behavior.
- Validation reports: [V01-01](../validations/F-MEM-01/V01-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search across both repos | yes | passed | [V01-01](../validations/F-MEM-01/V01-01.md) |
| V02 | Corrupt-file handling: FileStore vs FileConversationStore | yes | failed | [V02-01](../validations/F-MEM-01/V02-01.md) |
| V03 | Path safety + atomic-write durability (temp + fsync + rename + parent-dir fsync) | yes | failed | [V03-01](../validations/F-MEM-01/V03-01.md) |
| V04 | Projection round-trip + SnapshotManager identity | yes | passed | [V04-01](../validations/F-MEM-01/V04-01.md) |
| V05 | Historical-document drift | conditional | not_applicable | — |

V05 is not applicable: there are no prior F-MEM-01 reports or memory-store
design docs to classify as current/fixed/stale/regressed. The two
in-tree module docs (`store.rs:1-6`, `file_conversation.rs:1-26`) make
falsifiable claims that were verified directly in V01-V04 and are recorded
under "Current Path" above.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `file_conversation.rs:18-22` "Corrupt JSON is an error" | current | `file_conversation.rs:149-166, 215-242` enforce it; tests at 623. The sister `FileStore` violates the spirit of this contract — see P1-01. |
| `file_conversation.rs:14-19` "Path-safe ids ... prevents path-traversal" | current | `safe_segment` at 465-486; tests at 644, 772. |
| `file_conversation.rs:23-25` "Unique temp names + parent-dir sync" | current | `atomic_write` at 494-533; `sync_parent_directory` at 525-528. |
| `store.rs:1-6` "Concrete implementations (InMemoryStore, FileStore) live in echo_state" | current | V01-01 confirms. |
| `echo-state/src/memory/mod.rs:11-13` "Runtime checkpoints ... live in RuntimeStateStore" | current | `SnapshotManager` confirms — it is in-memory only. |
| `echo-state/src/memory/conversation.rs:91-95` "round-trips losslessly for the four canonical roles" | current | V04-01 confirms via `round_trip_messages_via_project_and_restore` (file_conversation.rs:654). |

## Coverage And Uncertainty

- **SQLite backend** explicitly excluded — F-MEM-02 owns it. No claims here
  about `SqliteStore` / `SqliteConversationStore` correctness, concurrency,
  or feature isolation.
- **Executable tests not run.** All four validations are static code + test
  inspection. P1-01 and P2-01/P2-02 should be confirmed by new executable
  unit tests when the fixes land (the existing `corrupt_record_surfaces_*`
  and `path_traversal_id_is_rejected` tests already prove the
  `FileConversationStore` side).
- **`EmbeddingStore` deep audit** is partial — only its atomic-write and
  `Store` impl signature were inspected (lines 170-208, 320-330). Its
  vector-search numerical edge cases belong to F-MEM-02 (it shares the
  SQLite/semantic-search surface).
- **Multi-process safety** for `FileStore` / `FileConversationStore` is
  documented as out of scope (single-process local agent). The in-process
  `RwLock` / `Mutex` is sufficient for the local-assistant threat model in
  AGENTS.md; cross-process concurrency is the SQLite backend's job.
- **`FileStore` write amplification** (whole-map serialized on every `put`)
  is a known performance characteristic, not a correctness finding; not
  promoted to a finding.

## Handoff

- Downstream tasks may rely on: the `Store` / `ConversationStore` trait
  matrix in V01-01; the projection round-trip losslessness in V04-01; the
  path-safety of `FileConversationStore` (V03-01). These are stable inputs
  for F-MEM-02, F-CTX-01 (compression promotes into a `Store`), A-MEM-01
  (application memory policies build on `TypedMemoryStore`), and A-CFG-01 /
  F-CMP-01 (which read memory-store identities).
- Downstream tasks must read: V01-01 (impl matrix), V02-01 (corrupt-file
  contract), V03-01 (atomic-write contract).
- This report becomes stale if: `FileStore::new` changes its error
  handling; the three atomic-write routines converge on a shared helper;
  the `Store` trait gains or loses methods; the projection envelope
  version bumps from 1.
- Follow-up task IDs (no fixes implemented here):
  - **F-MEM-02** should confirm whether `SqliteStore` already implements
    the durability contract `FileStore` is missing (it likely does, via
    SQLite's WAL), and use this report's `FileConversationStore` recipe as
    the canonical framework reference.
  - A dedicated cleanup task should factor the shared `atomic_write` helper
    out of `file_conversation.rs` and converge all three routines (P2-01,
    P2-02).
