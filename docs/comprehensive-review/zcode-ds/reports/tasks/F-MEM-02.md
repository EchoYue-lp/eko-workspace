# F-MEM-02: SQLite framework capabilities — SqliteStore / SqliteConversationStore

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean (both repositories)

## Question

Are `SqliteStore` and `SqliteConversationStore` valid independent framework
options with correct concurrency, schema, search, and feature gates?

## Scope

- `echo-agent/echo-state/src/memory/sqlite_store.rs` (SqliteStore: KV +
  FTS5 + optional vector search; Store impl; 15 unit tests).
- `echo-agent/echo-state/src/memory/sqlite_conversation.rs`
  (SqliteConversationStore; ConversationStore impl; zero tests).
- Feature gates: `echo-state/Cargo.toml:22,34`; root
  `echo-agent/Cargo.toml` `sqlite` feature + `full`; `echo-state/src/memory/
  mod.rs:23-26,47-50`; `echo-agent/src/lib.rs:197-199` (prelude);
  `echo-state/src/util.rs:3,23`; root `src/state/mod.rs:291-296`
  (SqliteRuntimeStateStore — a second, distinct sqlite consumer).
- Examples: demo27_sqlite_memory, demo45_customer_service, demo46_data_analyst,
  demo48_personal_assistant (required-features verified).
- Comparison surface: `echo-state/src/memory/store.rs` (FileStore/
  InMemoryStore) and `file_conversation.rs` for semantic alignment.

## Out Of Scope

- `FileRuntimeStateStore` / `src/state/sqlite.rs` (SqliteRuntimeStateStore)
  — runtime-state task (F-RCT-05 area); root `sqlite` feature's `dep:rusqlite`
  existence noted only.
- `embedding_store.rs` / snapshot / evolution layers (F-MEM-01 covers the
  file-backed EmbeddingStore).
- Real cross-process concurrency load testing (no harness exists in repo).
- Echo-agent-cli application behavior (except the one stale doc comment,
  P3-06, which is sqlite-topic-adjacent).
- This review proposes no deletion of the sqlite backends — AGENTS.md
  explicitly forbids removing framework capability because the CLI does not
  use it.

## Inputs

- Root `AGENTS.md` (SQLite positioning: CLI non-use is an application
  decision; framework capability menu retention rule), shared `REPORTING.md`,
  `zcode-ds/README.md`, report templates.
- Dependency report: `zcode-ds F-MEM-01` (FileStore/FileConversationStore
  review — trait contract anchors, P3-01 namespace flaw, P3-02 get_messages
  ordering, write-protocol findings).
- Dependency report: `zcode-ds F-FEAT-01` (sqlite feature isolation verified
  at manifest level; P3-03 confirmed root `sqlite` keeps its rusqlite dep for
  `src/state/sqlite.rs`).
- Historical documents treated as hypotheses: the sqlite_store.rs module docs
  (lines 62-64: "single connection eliminates SQLITE_BUSY storms"), the
  "Production-grade" claims in both module headers, and the
  file_conversation.rs:364-366 comment claiming id reuse "matches the SQLite
  autoincrement".

## Layering Decision

- Generic mechanism: `SqliteStore`/`SqliteConversationStore` are framework
  capability-menu options in `echo_state` — multi-implementations of the
  `Store`/`ConversationStore` traits (4 and 2 impls respectively). They belong
  in the framework per the AGENTS.md deletion rules: the CLI not enabling the
  `sqlite` feature is an EKO product decision
  (echo-agent-cli/Cargo.toml:50, echo-agent-app-core/Cargo.toml:10-15), not a
  framework-API judgment. `sqlite` feature wiring (echo_state -> dep:rusqlite;
  root -> echo_state/sqlite + dep:rusqlite for SqliteRuntimeStateStore) is
  correct and complete (V02).
- EKO product policy: store *selection* (FileStore/FileConversationStore) is
  application-level; nothing sqlite-related leaks into EKO (V01).
- Adapter boundary: none — EKO never constructs sqlite types.
- Duplicate search terms: `SqliteStore`, `SqliteConversationStore`,
  `sqlite_store`, `sqlite_conversation`, `rusqlite`, `memory_io_error`,
  `apply_json_metadata`, `cosine_similarity`, `bytes_to_vec` across both
  repositories. Single authoritative definitions; rusqlite confined to the
  two modules (plus root `src/state/sqlite.rs` for the runtime state store).

## Current Path

- `SqliteStore::new/with_embedder` -> `open` (expand_tilde, mkdir, single
  `Connection` in `std::sync::Mutex`, WAL + synchronous=NORMAL +
  busy_timeout=5000 PRAGMAs) -> `init_tables` (store_items with composite PK
  (namespace,key) + importance/last_accessed/expires_at migration columns,
  `store_fts` FTS5 virtual table with unicode61, `store_vectors` blob table).
- `Store::put` computes the embedding before acquiring the connection
  (sqlite_store.rs:478-497, keeps the future Send), then runs one transaction
  over main-table upsert + FTS delete/insert + vector upsert (:499-557).
  `delete` similarly transactions all three tables (:710-745).
- `Store::search` -> FTS5 MATCH with per-keyword quoted OR terms, bm25 score,
  `WHERE key IN (...)` fetch; LIKE fallback for CJK when FTS returns nothing.
  `search_with` Semantic -> `semantic_search_impl` (embed query, scan
  `store_vectors`, cosine, top-10k candidate cap); Hybrid -> RRF merge with a
  keyword-only fallback when no embedder is configured.
- `Store::prune_expired` deletes from `store_items` only, keyed on
  `json_extract(value,'$.expires_at')`; `dedup_by_content` is NOT overridden
  (trait no-op).
- `SqliteConversationStore` (tokio Mutex + single Connection): conversation
  table (AUTOINCREMENT id, UNIQUE conversation_id) + message table with FK
  ON DELETE CASCADE (foreign_keys=ON); `save_messages` = manual
  BEGIN IMMEDIATE + DELETE-all + INSERT-all + COMMIT with ROLLBACK on error.
- Reachability today: framework examples only (demo27/45/46/48, all gated on
  `required-features = ["sqlite", ...]`); no framework runtime path and no EKO
  path constructs either type — they are optional capability menu entries.

## Findings

### F-MEM-02-P2-01: importance/expires_at/last_accessed metadata projection differs between SqliteStore and the file/memory stores — same input, different StoreItem fields

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/sqlite_store.rs:509-516`
  (put extracts importance/expires_at from the JSON value), `:935-945`
  (apply_json_metadata — sqlite-only helper applied on every read path
  :317,:388,:599,:811) vs `echo-state/src/memory/store.rs:57-64` and
  `:299-316` (InMemoryStore/FileStore put create/modify StoreItem structs and
  never read metadata from the JSON value; `get` returns struct fields
  directly); framework's own write path embeds metadata in the value:
  `echo-agent/src/tools/builtin/memory.rs:104-117` (remember tool writes
  `"importance": N` into the JSON).
- Reachability: definition -> any framework consumer putting a JSON value
  containing `importance`/`expires_at`/`last_accessed` (the remember tool
  does this on every call) -> `SqliteStore::get/list/search` returns the
  projected field values while `FileStore`/`InMemoryStore` return the struct
  defaults (importance 5.0, expires_at/last_accessed None). Prune behavior is
  aligned (file-side `is_item_valid` also checks JSON expiry,
  store.rs:494-506), but the read-path projection diverges.
- Expected invariant: identical `Store::put` input yields equivalent
  `StoreItem` metadata from every `Store` implementation (the trait's
  capability-menu contract).
- Observed behavior: `SqliteStore` reflects JSON-embedded metadata into
  StoreItem fields; FileStore/InMemoryStore do not — a consumer switching
  backends silently sees different importance/expiry on the same data
  (SqliteStore honors the remember tool's importance, the file stores always
  report 5.0).
- Impact: backend-dependent memory semantics for framework consumers;
  importance-based ranking/decay and TTL decisions made on
  `item.importance`/`item.expires_at` silently differ per backend. Latent
  today (EKO uses the file path; sqlite consumers are examples), contract-
  level.
- Root cause: two metadata conventions (JSON-embedded vs struct-field) with no
  shared projection helper; `apply_json_metadata` exists only in
  sqlite_store.rs, `is_item_valid` only in store.rs.
- Direction: either adopt JSON-metadata projection in FileStore/InMemoryStore
  read paths (mirroring apply_json_metadata — matches the remember tool's
  convention) or document the convention split in the trait docs; add a
  cross-backend equivalence test asserting identical StoreItem metadata for
  one put input on InMemoryStore/FileStore/SqliteStore.
- Regression validation: unit test putting `{"content":"x","importance":8,
  "expires_at":<past>}` then asserting identical importance/expires_at from
  all three stores; existing 15 sqlite + file-store tests stay green.
- Validation reports: [V05-01](../validations/F-MEM-02/V05-01.md)

### F-MEM-02-P2-02: SqliteConversationStore discards supplied message ids while FileConversationStore preserves them — compression boundary silently misaligns on the SQLite backend

- Priority: P2
- Confidence: high (behavior), medium (impact — requires id-carrying re-import)
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/sqlite_conversation.rs:332-345`
  (INSERT never binds `StoredMessage.id`) vs
  `echo-state/src/memory/file_conversation.rs:363-370` (reuses `Some(id)` and
  advances `next_id`; comment claims the reuse "matches the SQLite
  autoincrement" — factually wrong for id-carrying inputs); contract anchors
  `echo-core/src/memory/conversation.rs:43-44` (compressed_before_id:
  "summary covers messages up to this id (inclusive)") and `:66-67`
  (StoredMessage.id: "Database auto-increment ID (None for new messages)").
- Reachability: any consumer that round-trips messages (get_messages ->
  modify -> save_messages) or imports records with explicit ids onto the
  SQLite backend; `compressed_before_id` then refers to ids that no longer
  exist after renumbering. FileConversationStore keeps the ids, so the two
  backends persist different message ids for identical input.
- Expected invariant: both ConversationStore implementations preserve or
  consistently re-assign message ids so that `compressed_before_id` remains a
  valid boundary reference.
- Observed behavior: SQLite backend renumbers all messages on every
  save_messages; id-carrying inputs are silently dropped. E.g., a record
  imported with ids 1000..1004 re-persists as 1..5; a stored
  compressed_before_id=1002 now points at nothing or the wrong message.
- Impact: silent corruption of the compression boundary (transcript truncation
  around the wrong message) for SQLite-backend consumers after any
  id-carrying re-import; divergent ids between backends complicate migrations.
- Root cause: SQLite AUTOINCREMENT is used unconditionally; the trait leaves
  id semantics underspecified, and FileConversationStore's mirror-comment
  papers over the divergence.
- Direction: bind `msg.id` when `Some` (with counter/max advance mirroring
  file_conversation.rs:363-370) or document that the SQLite backend does not
  preserve caller ids and update the file-side comment; add a round-trip test
  with explicit ids on SqliteConversationStore.
- Regression validation: save_messages with `[Some(1000), Some(1001)]` then
  get_messages asserting the same ids on both backends; compressed_before_id
  alignment test.
- Validation reports: [V05-01](../validations/F-MEM-02/V05-01.md),
  [V03-01](../validations/F-MEM-02/V03-01.md)

### F-MEM-02-P2-03: search/semantic results violate the trait's "sorted by relevance" contract — `WHERE key IN` rows are returned in table order and semantic similarity scores are replaced by positional scores

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: trait contract `echo-core/src/memory/store.rs:198-199` ("returns
  at most `limit` items (sorted by relevance)"); `echo-agent/echo-state/src/
  memory/sqlite_store.rs:271-321` (fetch_items: `WHERE namespace = ?1 AND key
  IN (...)` with no ORDER BY; results pushed in row order), `:338-393`
  (fetch_items_with_scores, same), `:300` (score fallback
  `1.0/(i+1)` positional), `:456` (semantic path calls
  `fetch_items(..., None)` — computed cosine scores in `scored` (:439-450)
  are used only for sort order, then discarded), `:701-706` (FTS scores
  preserved but order lost). Sibling conformance: FileStore/InMemoryStore
  sort by score before returning (store.rs:388-396 and :91-133 area).
- Reachability: any SqliteStore consumer calling `search`/`search_with` with
  multi-match results; SQLite returns rows for `IN` lists in table/rowid
  order, not IN-list order, so the final Vec order is arbitrary w.r.t.
  relevance. Existing tests never exercise multi-match ordering
  (test_fts5_search / test_fts5_search_multiple_keywords / test_with_embedder
  all assert on single-match results, V03-01).
- Expected invariant: `search`/`search_with` return items ordered by
  relevance (descending score); semantic items carry their similarity score.
- Observed behavior: results in DB row order with score fields attached
  (FTS path) or with positional fallback scores (semantic path); semantic
  similarity values are never surfaced.
- Impact: consumers that display or rank by result position get wrong order;
  consumers reading `item.score` from semantic search get positional values,
  not cosine similarity. Scores in hybrid merge remain correct (RRF computed
  from ranks, not these scores) — limited blast radius, but the documented
  contract is violated on both paths.
- Root cause: batch-fetch optimization (`WHERE key IN`) without re-sorting
  after fetch; fetch_items' default positional score masks the lost
  similarity values.
- Direction: sort the final Vec by score desc (FTS: stored score; semantic:
  carry the computed similarity into fetch via keys_with_scores or re-attach
  after fetch), and add multi-match ordering tests (e.g., two docs both
  matching a keyword with different bm25, assert top result).
- Regression validation: test inserting k1/k2/k3 where bm25 order differs
  from insertion order and asserting search()[0] is the best match; semantic
  test asserting item.score equals the cosine similarity.
- Validation reports: [V04-01](../validations/F-MEM-02/V04-01.md),
  [V05-01](../validations/F-MEM-02/V05-01.md)

### F-MEM-02-P3-01: prune_expired removes store_items rows but orphans store_fts and store_vectors entries

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/sqlite_store.rs:918-925`
  (prune deletes only from store_items) vs `:716-741` (delete() transactions
  all three tables).
- Reachability: `prune_expired` is reachable through the Store trait; after
  pruning, FTS/LIKE search may still match pruned keys, and the vector table
  still holds dead blobs. Results stay consistent today because
  fetch_items filters through store_items, and `put` re-inserts fresh FTS/vector
  rows for re-added keys — so the impact is stale-index growth and wasted
  match work, not incorrect results.
- Expected invariant: removing an item removes it from all of the store's
  indexes.
- Observed behavior: main table pruned; FTS + vector rows orphaned.
- Impact: unbounded index growth on long-running stores with TTL items;
  semantic search scans dead vectors (up to the 10k candidate cap).
- Root cause: prune path predates the three-table transaction pattern used by
  put/delete.
- Direction: extend prune_expired to delete from store_fts and store_vectors
  in the same transaction; add a prune-then-search test asserting no stale
  matches.
- Regression validation: put item with past expires_at, prune_expired, then
  assert FTS search and semantic search return no result and the vector table
  is empty (SQL-level check via a test-only query).
- Validation reports: [V05-01](../validations/F-MEM-02/V05-01.md),
  [V03-01](../validations/F-MEM-02/V03-01.md)

### F-MEM-02-P3-02: dedup_by_content is not implemented on SqliteStore (trait no-op)

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-core/src/memory/store.rs:253-256` (trait default no-op);
  SqliteStore does not override it (impl block `sqlite_store.rs:464-929`
  contains no dedup) vs InMemoryStore/FileStore implementations
  (`echo-state/src/memory/store.rs:183-215, 448-477`).
- Reachability: zero callers of `dedup_by_content` in framework or EKO today
  (grep verified) — latent; any consumer relying on the trait's documented
  dedup semantics gets silent no-op on the SQLite backend.
- Expected invariant: every Store implementation either implements the
  optional trait methods it exposes or documents the no-op.
- Observed behavior: SqliteStore silently no-ops dedup (returns 0).
- Impact: silent behavioral difference between backends for a documented
  trait method; maintenance trap (mirrors F-MEM-01-P3-03's finding for
  EmbeddingStore).
- Root cause: dedup implemented for file/in-memory backends, never ported.
- Direction: implement dedup_by_content (hash content, keep newest per hash,
  mirroring store.rs:183-215 semantics) or document the no-op; add a
  dedup test on SqliteStore.
- Regression validation: insert two same-content items, dedup, assert one
  remains and FTS/vector tables have no orphan for the removed key.
- Validation reports: [V05-01](../validations/F-MEM-02/V05-01.md)

### F-MEM-02-P3-03: LIKE-fallback escaping is ineffective in SQLite — keywords containing `%`/`_` mis-match on the CJK fallback path

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/sqlite_store.rs:660`
  (`format!("%{}%", keyword.replace('%', "\\%"))`) and `:662-669` (LIKE query
  with no `ESCAPE` clause). SQLite's LIKE has no default escape character —
  `\%` is a literal backslash followed by the `%` wildcard; `_` is never
  escaped at all.
- Reachability: the fallback path runs only when FTS5 MATCH returns zero rows
  (CJK and other non-Latin searches — the path's stated purpose, :654-656),
  for queries whose keywords contain `%` or `_` (e.g., searching a variable
  name "foo_bar" or a percentage string).
- Expected invariant: LIKE pattern escaping actually escapes wildcards so the
  fallback matches literal occurrences.
- Observed behavior: keyword `%` becomes backslash+wildcard (matches
  effectively nothing or over-broadly), keyword `_` matches any single char
  (over-broad).
- Impact: incorrect fallback results for such keywords (misses and false
  positives); limited to the fallback path.
- Root cause: MySQL-style escaping assumed; SQLite needs `ESCAPE '\'` clause
  and `_` handling.
- Direction: add `ESCAPE '\'` to the LIKE statement and escape `_` as well
  (`keyword.replace(['%','_'], ...)` or use `LIKE ... ESCAPE`), or normalize
  the pattern via `replace('\', "\\")` first; add a test with a `%`/`_`
  keyword.
- Regression validation: put content containing "foo_bar" and "50% off",
  search "foo_bar" and "50%" via the fallback path, assert exact matches.
- Validation reports: [V04-01](../validations/F-MEM-02/V04-01.md),
  [V03-01](../validations/F-MEM-02/V03-01.md)

### F-MEM-02-P3-04: semantic search silently degrades to all-zero garbage on dimension mismatch / empty vectors; malformed blobs are silently truncated

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/sqlite_store.rs:239-251`
  (cosine_similarity returns 0.0 for dimension mismatch or empty input),
  `:439-451` (semantic_search_impl never checks `query_vec.len()` against
  stored vectors; all-zero scores are ranked and returned), `:233-235`
  (bytes_to_vec uses chunks_exact(4) — non-multiple-of-4 blobs lose their
  tail silently).
- Reachability: `SearchMode::Semantic`/`Hybrid` with an embedder whose output
  dimension differs from previously stored vectors (e.g., model change,
  misconfigured HttpEmbedder) or that returns an empty vector.
- Expected invariant: a dimension mismatch or empty query vector surfaces an
  explicit error (as the no-embedder case does at :405-409); malformed blobs
  are rejected, not truncated.
- Observed behavior: all scores 0.0 -> arbitrary DB-order results with score
  0.0 returned as if valid; no error. (NaN/Inf inputs are safe — partial_cmp
  fallback at :451 — but rank arbitrarily.)
- Impact: silent wrong results during embedder drift; confusing debugging;
  no crash (panic-safety verified — this finding is about silent degradation,
  not panics).
- Root cause: cosine_similarity's defensive 0.0 doubles as a silent-failure
  signal; no dimension validation on the read path.
- Direction: validate `query_vec.len()` against stored vector dims on read
  (error `MemoryError::Unsupported` on mismatch), and either error or skip
  malformed blobs; add tests for mismatched/empty vectors.
- Regression validation: store 4-dim vectors, search with an 8-dim embedder,
  assert Err (not garbage); empty-query-vector test; odd-length blob test.
- Validation reports: [V04-01](../validations/F-MEM-02/V04-01.md)

### F-MEM-02-P3-05: SqliteConversationStore has zero test coverage and uses a manual transaction whose rollback failure is ignored

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/sqlite_conversation.rs` contains
  no `#[cfg(test)]` (grep verified; the 15-test sqlite suite covers
  sqlite_store.rs only — V03-01); `:313-365` (manual BEGIN IMMEDIATE /
  COMMIT with `let _ = conn.execute("ROLLBACK", [])` on the three error
  paths :327,:348,:360).
- Reachability: every SqliteConversationStore operation is untested; a
  ROLLBACK failure (I/O error) leaves the pooled connection inside an open
  transaction that subsequent operations inherit — uncommitted partial state
  visible to the next caller.
- Expected invariant: each store implementation has test coverage comparable
  to its file sibling (FileConversationStore has extensive tests, F-MEM-01
  V04); transaction cleanup is guaranteed on all failure paths.
- Observed behavior: zero tests; rollback errors silently swallowed; rusqlite
  `Transaction` (used correctly by SqliteStore put/delete) is not used here.
- Impact: regression risk for the conversation backend; rare transaction-
  leakage on rollback failure.
- Root cause: sqlite conversation backend predates/parallels the file backend
  without test porting; hand-rolled transaction management.
- Direction: add SqliteConversationStore tests (create/get/list/update/delete,
  save/get messages incl. explicit ids, cascade delete, search); switch to
  rusqlite `Transaction` for automatic rollback-on-drop; treat ROLLBACK errors
  as real errors.
- Regression validation: new test module exercising the trait surface;
  existing 15 sqlite tests stay green.
- Validation reports: [V03-01](../validations/F-MEM-02/V03-01.md)

### F-MEM-02-P3-06: stale "sqlite-backed" doc comment on the EKO runtime state store field

- Priority: P3
- Confidence: high
- Layer: application (documentation)
- Evidence: `echo-agent-cli/echo-agent-app-core/src/infra.rs:125` ("Shared
  runtime state store (sqlite-backed)") vs `:1254`
  (`echo_agent::state::FileRuntimeStateStore::new`); CLI enables no sqlite
  feature (echo-agent-app-core/Cargo.toml:10-15).
- Reachability: documentation only — the field's actual value is the
  file-backed store; no code path is affected.
- Expected invariant: comments reflect the implementation (this project
  deliberately keeps SQLite out of the CLI — AGENTS.md).
- Observed behavior: comment implies a SQLite runtime store that EKO never
  constructs.
- Impact: misleads maintainers about EKO's SQLite usage; contradicts the
  documented no-SQLite positioning.
- Root cause: comment written for an earlier sqlite-backed design, not updated
  after the file implementation.
- Direction: reword to "file-backed" (or drop "sqlite-backed").
- Regression validation: none needed (doc-only); grep for remaining
  "sqlite-backed" references.
- Validation reports: [V01-01](../validations/F-MEM-02/V01-01.md)

No P0/P1 findings. Panic-safety scan (AGENTS.md rule): production code in both
sqlite files contains no reachable `unwrap`/`expect`/unsafe byte-slice —
`partial_cmp(...).unwrap_or(Ordering::Equal)` (:451,:896-901),
`chunks_exact`-guaranteed `chunk[0..3]` (:234), `unwrap_or(0)` item counts,
`expand_tilde`'s `&s[2..]` (util.rs:8, guarded by `starts_with("~/")`) are all
safe; float math never panics; integer casts are in-range. The only `assert!`
is in test-only MockEmbedder.

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Public-use justification (pub exports, docs, examples required-features, CLI non-use) | yes | passed | [V01-01](../validations/F-MEM-02/V01-01.md) |
| V02 | Feature isolation (echo-state/root sqlite gates, no-feature invisibility, no-default check) | yes | passed (exit 0) | [V02-01](../validations/F-MEM-02/V02-01.md) |
| V03 | `cargo test -p echo_state --all-features --lib --locked sqlite`; concurrency + error propagation | yes | passed (exit 0, 15/15) | [V03-01](../validations/F-MEM-02/V03-01.md) |
| V04 | Semantic-search numerical edge cases (dim mismatch, empty, NaN/Inf, overflow, malformed blob) | yes | passed (panic-safe; 2 behavioral findings) | [V04-01](../validations/F-MEM-02/V04-01.md) |
| V05 | Semantic alignment FileStore/FileConversationStore vs sqlite backends | conditional (F-MEM-01 dependency) | passed (4 drifts -> findings) | [V05-01](../validations/F-MEM-02/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| sqlite_store.rs:62-64 — single Mutex<Connection> "eliminates SQLITE_BUSY storms under concurrent access" | current | in-process serialization correct and panic-safe; cross-process relies on WAL + busy_timeout=5000 (V03-01) |
| sqlite_store.rs:1-3 / sqlite_conversation.rs:2-3 — "Production-grade" storage | current (with caveats) | error propagation, transactions, WAL, FTS5 are solid; caveats filed: P2-01..P3-05 |
| file_conversation.rs:364-366 — id reuse "matches the SQLite autoincrement" | regressed | SqliteConversationStore discards caller ids (renumbers); the claim is factually wrong (P2-02) |
| F-MEM-01 P3-01 — namespace encoding non-injective (`["a/b"]` vs `["a","b"]`, string-level prefix) | current (extends to sqlite) | SqliteStore uses the same join("/") + split + `LIKE 'prefix%'` (sqlite_store.rs:401,766-777); shared flaw, not re-filed |
| F-MEM-01 P3-02 — get_messages ordering contract | current (sqlite conforms) | SqliteConversationStore orders by id ASC (sqlite_conversation.rs:381); FileConversationStore still deviates |
| F-FEAT-01 P3-03 — root `sqlite` feature's dep:rusqlite is a required exception | current (confirmed) | rusqlite used by src/state/sqlite.rs (SqliteRuntimeStateStore), gated at state/mod.rs:291-296 (V02-01) |

## Coverage And Uncertainty

- Cross-process concurrency (two processes on one DB file) verified
  statically only (WAL + busy_timeout); no in-repo multi-process test harness.
- FTS5 MATCH query-abort behavior (an FTS syntax error returns Err instead of
  falling back to LIKE, sqlite_store.rs:649) assessed but not promoted to a
  finding — quoted keywords make the error path unlikely.
- Timestamp format divergence (RFC3339 vs `datetime('now','localtime')`)
  noted in V05, not filed — no documented RFC3339 contract in the trait.
- `search_memory`/tool-layer consumers of result ordering were not traced
  command-by-command in EKO (EKO uses FileStore, so unaffected); impact of
  P2-03 is assessed against the trait contract and framework consumers.
- The 131 non-sqlite echo_state tests filtered out by the V03 command were
  not re-run (F-MEM-01 V04 already covers the memory suites); sqlite-only
  suite ran under `--all-features`.
- SqliteRuntimeStateStore (src/state/sqlite.rs) deliberately out of scope
  (runtime-state task).

## Handoff

- Downstream tasks may rely on: sqlite backends are valid framework
  capability-menu options with correct feature gates (V01/V02); the test
  suite passes 15/15 under --all-features (V03); panic-safety holds on all
  numerical and error paths (V03/V04); the concrete contract drifts listed in
  P2-01/P2-02/P2-03 and the P3 cleanups.
- F-MEM-01's P3-01 (namespace encoding) covers the sqlite backends too; the
  fix direction (reject `/` in segments, segment-aware prefix) applies to all
  three stores.
- Iteration roadmap: P2-01..P2-03 fixes belong in the echo-agent framework
  (echo-state memory) with the regression tests specified; the trait docs
  (echo-core store.rs / conversation.rs) should be clarified on the metadata
  convention and message-id semantics as part of the same change.
- Deletion targets when fixing: none — do NOT delete SqliteStore/
  SqliteConversationStore (AGENTS.md); fixes extend them.
- This report becomes stale if: sqlite_store.rs / sqlite_conversation.rs /
  echo-state store.rs / file_conversation.rs read-write paths change; the
  Store/ConversationStore trait surfaces change; feature topology changes;
  or the reviewed commits are superseded.
- Follow-up task IDs: F-MEM-01 (shared namespace flaw P3-01, ordering P3-02),
  X-BND-01 (facade authority map), Q-* dynamic gates for multi-match ordering
  and cross-process sqlite contention; B-DOC-01 for the P3-06 comment.
