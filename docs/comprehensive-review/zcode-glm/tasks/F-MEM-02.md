# F-MEM-02: SQLite framework capabilities

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa
> `echo-agent-cli` commit: not-applicable
> Worktree state: clean (read-only review)

## Question

Are `SqliteStore` and `SqliteConversationStore` valid independent framework
options with correct concurrency, schema, search, and feature gates?

## Scope

Primary source paths and behaviors inspected (read-only):

- `echo-agent/echo-state/src/memory/sqlite_store.rs` (full, 1-1242) —
  `SqliteStore` struct, `Mutex<Connection>` model, `init_tables` schema +
  migration, FTS5 search + LIKE fallback, `semantic_search_impl`, cosine
  similarity / vector (de)serialization, `Store` impl incl. `put` transaction,
  `delete` three-table transaction, `prune_expired`, unit tests.
- `echo-agent/echo-state/src/memory/sqlite_conversation.rs` (full, 1-465) —
  `SqliteConversationStore` struct, `tokio::sync::Mutex<Connection>` model,
  PRAGMA batch, conversation/message schema with `ON DELETE CASCADE`,
  `ConversationStore` impl incl. `save_messages` `BEGIN IMMEDIATE` transaction.
- `echo-agent/echo-state/src/memory/mod.rs` (full) — `#[cfg(feature = "sqlite")]`
  module + re-export gating.
- `echo-agent/echo-state/src/util.rs` (full) — `expand_tilde`,
  `memory_io_error` (sqlite-gated).
- `echo-agent/echo-state/Cargo.toml` — `sqlite = ["dep:rusqlite"]`,
  `rusqlite` optional `bundled`.
- `echo-agent/Cargo.toml` `[features]` — `sqlite = ["echo_state/sqlite",
  "dep:rusqlite"]`, `database` (separate sqlx), `full`, example
  `required-features`.
- `echo-agent/src/lib.rs:190-205` and `echo-agent/src/state/mod.rs:290-296` —
  root facade re-exports.
- `echo-agent/src/state/sqlite.rs` — `SqliteRuntimeStateStore` (read for
  cross-reference of PRAGMA / migration patterns; out of scope as a Store).
- `echo-agent/examples/demo27_sqlite_memory.rs` — exercised example consumer.

## Out Of Scope

Deferred to downstream tasks / explicitly excluded:

- **`SqliteRuntimeStateStore`** (`src/state/sqlite.rs`) is a
  `RuntimeStateStore`, not a memory `Store`/`ConversationStore`. It is only
  cross-referenced for PRAGMA/migration-pattern consistency. Its own
  correctness is owned by the runtime-checkpoint surface (F-MEM-01 noted
  `SnapshotManager` is in-memory; cross-restart is `RuntimeStateStore`).
- **CLI usage** — per AGENTS.md "echo-agent-cli 不需要 SQLite"; the CLI does
  not enable `sqlite`. This is explicitly NOT a deletion criterion and is not
  re-litigated here (see Layering Decision).
- **`EmbeddingStore` deep audit** — owned by F-MEM-01 (it shares the
  semantic-search surface but is a JSON-backed decorator, not a SQLite store).
- **`sqlx` / `database` feature** — separate from `sqlite`; not inspected
  beyond confirming the two features do not conflate.
- **Multi-process stress testing** — not executed; concurrency findings are
  static PRAGMA/code inspection plus single-process unit tests.

## Inputs

- Required repository documents read: root `AGENTS.md` (sections "删除框架代码
  的判定", "echo-agent-cli 不需要 SQLite", "Rust 编码硬性约束 §1/§2"),
  `docs/comprehensive-review/REPORTING.md`, both report templates, the
  `F-MEM-02` task card in `TASKS.md`.
- Dependency task reports read:
  - **F-MEM-01** — `Store`/`ConversationStore` trait matrix (V01-01), corrupt-
    file contract (V02-01), atomic-write recipe (V03-01). F-MEM-01 explicitly
    deferred the SQLite backend here and hypothesised (Handoff) that SQLite
    likely satisfies the durability contract via WAL. This report confirms that
    hypothesis and adds the SQLite-specific findings F-MEM-01 could not see.
  - **F-FEAT-01** — `sqlite` feature classified as a live, correctly-gated
    framework option (not dead, not a deletion candidate); forwarded via
    `echo_state/sqlite` + `dep:rusqlite`.
- Historical documents treated as hypotheses: module-level doc comments in
  `sqlite_store.rs:1-43` and `sqlite_conversation.rs:1-3` (verified against
  code, not trusted on faith).

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism | Both `SqliteStore` and `SqliteConversationStore` are generic agent-framework capabilities. Any `echo-agent` consumer needing persistent, concurrent, FTS-capable long-term memory and/or multi-user conversation history (e.g. a multi-tenant service, a research agent with large memory, a server-side deployment) may reasonably pick the SQLite backend. They belong in the framework (`echo-state`) alongside `FileStore`/`FileConversationStore` as a menu option. The `Store`/`ConversationStore` traits they implement live in `echo-core`. |
| EKO product policy | EKO (echo-agent-cli) picks `FileStore`/`FileConversationStore` because the local CLI does not use SQLite. That selection is an application-layer decision (verified in B-BASE-01) and does NOT propagate to "delete the framework SQLite option" — per AGENTS.md "删除框架代码的判定", a framework pub API is retained unless framework-wide evidence shows it is obsolete or fully replaced. |
| Adapter boundary | Not applicable — no EKO adapter is in scope. Both stores implement the framework traits directly. |
| Duplicate search | `pub struct SqliteStore`, `pub struct SqliteConversationStore`, `impl Store for`, `impl ConversationStore for`, `Connection::open`, `PRAGMA journal_mode`, `expand_tilde`, `memory_io_error`, `cosine_similarity`, `vec_to_bytes`/`bytes_to_vec`. Result: single definition per concept; no parallel SQLite-backed Store/ConversationStore authority. `SqliteRuntimeStateStore` is a different trait (`RuntimeStateStore`), not a duplicate. |
| Migration deletion | No deletion recommended. SQLite stays as a framework option. The only dead-code-adjacent observation is the largely-unread migration columns (P3-03), which is a schema-cleanup note, not a deletion mandate. |

## Current Path

**`SqliteStore`** (`echo-state/src/memory/sqlite_store.rs`):

- `pub struct SqliteStore { embedder: Option<Arc<dyn Embedder>>, conn: Mutex<Connection> }`
  (65-68). Two constructors: `new` (72, FTS5-only) and `with_embedder` (77,
  adds vector search). Re-exported at `echo-state/src/memory/mod.rs:50`
  (`#[cfg(feature = "sqlite")]`) and at the facade `echo-agent/src/lib.rs:199`
  (same gate + `#[cfg_attr(docsrs, doc(cfg(feature = "sqlite")))]`).
- `open` (81-106) creates parent dir, opens the connection, runs `init_tables`,
  logs the row count. `open_connection_at` (108-120) sets
  `PRAGMA journal_mode=WAL; synchronous=NORMAL; cache_size=10000;
  temp_store=MEMORY; busy_timeout=5000`.
- `init_tables` (130-184) creates `store_items(namespace,key,value,created_at,
  updated_at,...)`, `idx_store_ns`, the FTS5 virtual table `store_fts`, and
  `store_vectors`. Migration (147-159) `ALTER TABLE ADD COLUMN` for
  `importance`/`last_accessed`/`expires_at`.
- `Store::put` (465-561): computes the embedding **before** the transaction
  (comment at 478-481 explains `Transaction<'_>` is `!Send`), then wraps the
  main-table upsert + FTS delete/insert + vector upsert in one
  `conn.transaction()` (504-556) — atomic across all three tables.
- `Store::delete` (710-745): same three-table transaction pattern — deletes
  from `store_items`, `store_fts`, `store_vectors`, commits atomically.
- `Store::search` (608-708): FTS5 `MATCH` with `bm25(store_fts)` ranking; on
  zero hits falls back to per-keyword `LIKE %kw%` (CJK-friendly). Empty query
  returns `Ok(vec![])` (624-626).
- `semantic_search_impl` (395-457): loads all vectors for the namespace
  (`take(max_candidates)` = 10 000), computes `cosine_similarity` to the query
  vector, sorts with `partial_cmp().unwrap_or(Equal)` (NaN-safe), fetches the
  top-`limit` items. `search_with` Hybrid (833-904) RRF-fuses keyword +
  semantic results.
- `cosine_similarity` (239-251): returns 0.0 on length mismatch / empty / zero
  norm. `vec_to_bytes`/`bytes_to_vec` (222-236): little-endian f32, via
  `chunks_exact(4)`.
- 15 unit tests (956-1242), all green (see V04-01).

**`SqliteConversationStore`** (`echo-state/src/memory/sqlite_conversation.rs`):

- `pub struct SqliteConversationStore { conn: tokio::sync::Mutex<Connection> }`
  (20-22). `new` (26-60) opens the connection, runs
  `PRAGMA journal_mode=WAL; synchronous=NORMAL; cache_size=5000;
  temp_store=MEMORY; foreign_keys=ON;` — **no `busy_timeout`** (36-42).
- `init_tables` (62-92): `conversation` + `message` tables, `message` has
  `REFERENCES conversation(conversation_id) ON DELETE CASCADE`, indexes on
  `user_id` and `updated_at DESC`.
- `ConversationStore` impl (97-465): `save_messages` (307-369) wraps
  DELETE+INSERT+UPDATE in `BEGIN IMMEDIATE TRANSACTION` with manual ROLLBACK on
  error (good — atomic). `delete_conversation` (295-305) relies on the
  `ON DELETE CASCADE` for messages. `search_conversations` (422-464) does a
  `LIKE` join (the trait's default FTS-less path is overridden here with a
  real SQL search).
- Re-exported only at `echo-state/src/memory/mod.rs:48` — **NOT** re-exported
  from the `echo_agent` facade (`echo-agent/src/lib.rs`); see P3-02.
- Zero live callers in the whole workspace (grep confirmed: only doc comments
  + its own re-export). Still a valid framework menu option per AGENTS.md.

**Feature wiring**:

- `echo-state/Cargo.toml`: `sqlite = ["dep:rusqlite"]`; `rusqlite = { version
  = "0.32.1", features = ["bundled"], optional = true }`. `bundled` ships
  SQLite (with FTS5) so consumers never hit "FTS5 not compiled in".
- Root `echo-agent/Cargo.toml:71`: `sqlite = ["echo_state/sqlite",
  "dep:rusqlite"]` — forwards to sub-crate and activates the root's own
  `rusqlite` (used by `SqliteRuntimeStateStore`).
- `database` (line 87: `["dep:sqlx", "echo_tools/database"]`) is a separate
  feature — no conflation with `sqlite`.

## Findings

### F-MEM-02-P2-01: `prune_expired` orphans FTS5 and vector index entries (violates the three-table lockstep invariant `delete` upholds)

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/sqlite_store.rs:909-928`
  (`prune_expired` issues a single `DELETE FROM store_items`, no FTS/vector
  cleanup, no transaction). Contrast with `delete` at 710-745 which wraps all
  three DELETEs in one transaction.
- Reachability: every `Store::prune_expired` call on a `SqliteStore` whose
  items carry an `expires_at` JSON field.
- Expected invariant: the three tables `store_items` / `store_fts` /
  `store_vectors` are kept in lockstep — `put` writes all three atomically
  (504-556) and `delete` removes from all three atomically (716-741). Any
  removal path must preserve this so the indexes never reference a key absent
  from the main table.
- Observed behavior: `prune_expired` issues `DELETE FROM store_items WHERE ...
  json_extract(value, '$.expires_at') ... < ?2` and returns. The matching
  `store_fts` and `store_vectors` rows are left behind.
- Impact: (1) Unbounded index bloat — expired items' FTS/vector rows accumulate
  forever. (2) `search` queries `store_fts` first then fetches from
  `store_items`; orphaned FTS keys are silently dropped by `fetch_items` (the
  `key IN (...)` query returns nothing for them), so results stay correct but
  query budget is wasted and the FTS row count diverges from the main table.
  (3) `semantic_search_impl` loads orphaned vector blobs, scores them, sorts
  them, then `fetch_items` drops them — wasted compute. The invariant `delete`
  carefully preserves is silently violated by a sibling method.
- Root cause: `prune_expired` was written independently of `delete` and did not
  adopt the three-table cleanup pattern.
- Direction: mirror `delete` — either issue the three DELETEs in a transaction,
  or use a single `DELETE FROM store_fts WHERE (namespace,key) IN (SELECT
  namespace,key FROM store_items WHERE ... )` plus the analogous vector delete,
  then delete from `store_items`. Easiest is to wrap the existing DELETE with
  two companion DELETEs scoped to the same `expires_at` predicate inside one
  transaction.
- Regression validation: new test asserting that after `prune_expired`, no row
  exists in `store_fts` or `store_vectors` whose `(namespace,key)` is absent
  from `store_items`. Existing FTS/vector tests still pass.
- Validation reports: [V04-01](../validations/F-MEM-02/V04-01.md)

### F-MEM-02-P2-02: `SqliteConversationStore` omits `busy_timeout` PRAGMA (concurrency asymmetry with `SqliteStore`)

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/sqlite_conversation.rs:36-42`
  (PRAGMA batch has no `busy_timeout`) vs `echo-agent/echo-state/src/memory/
  sqlite_store.rs:112-117` (`PRAGMA busy_timeout=5000`).
- Reachability: every `SqliteConversationStore::new` configures its connection
  without a busy timeout; the setting then governs all subsequent ops on that
  connection.
- Expected invariant: two sibling SQLite stores in the same framework, intended
  for the same class of consumer, should apply the same concurrency PRAGMAs.
  `busy_timeout` is the canonical SQLite knob that makes a writer wait (up to N
  ms) for a lock instead of returning `SQLITE_BUSY` immediately.
- Observed behavior: `SqliteStore` waits up to 5000 ms on lock contention;
  `SqliteConversationStore` returns `SQLITE_BUSY` (surfaced as
  `MemoryError::IoError`) the instant another connection holds the write lock.
  Both enable WAL (which helps concurrent readers), so the omission is the
  sole meaningful divergence.
- Impact: Within the single-process-single-connection default (one
  `Mutex<Connection>` per store) there is no contention and no impact. But a
  framework consumer that opens the conversation DB from two processes (e.g. a
  service plus a background cleanup job, or two worker processes) hits
  `SQLITE_BUSY` where the memory store would have waited and retried. The two
  stores thus have different cross-process resilience for no documented reason.
- Root cause: the two PRAGMA batches were authored independently (different
  cache_size values too: 5000 vs 10000) and never reconciled.
- Direction: add `PRAGMA busy_timeout=5000;` to the batch at
  `sqlite_conversation.rs:36-42`. Optionally also converge `cache_size` while
  there.
- Regression validation: open two connections to the same conversation DB file,
  hold a write transaction on one, issue a write from the other, assert the
  second waits/retries rather than failing instantly.
- Validation reports: [V03-01](../validations/F-MEM-02/V03-01.md)

### F-MEM-02-P2-03: `SqliteStore` uses `std::sync::Mutex` with synchronous rusqlite I/O (executor-blocking, !Send-guard footgun); `SqliteConversationStore` uses `tokio::sync::Mutex`

- Priority: P2
- Confidence: medium
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/sqlite_store.rs:54` (`use
  std::sync::{Arc, Mutex}`), `:67` (`conn: Mutex<Connection>`), `:124-128`
  (`open_connection` returns `std::sync::MutexGuard`). The struct doc at 62-64
  justifies this with "SQLite serialises all writes anyway". Contrast with
  `echo-agent/echo-state/src/memory/sqlite_conversation.rs:14,21`
  (`tokio::sync::Mutex`).
- Reachability: every `SqliteStore` method holds the std mutex across
  synchronous rusqlite calls (disk I/O). The guard is held across the
  transaction body in `put` (499-557) and `delete` (713-744); in
  `semantic_search_impl` it is dropped and re-acquired (454-455) specifically
  to avoid holding it across an `.await`.
- Expected invariant: async `Store` methods should not block a tokio worker
  thread on synchronous disk I/O, and should not risk producing a `!Send`
  future (tokio multi-threaded schedulers require `Send` futures). A framework
  that offers both SQLite stores should use one consistent, safe mutex model.
- Observed behavior: (a) `SqliteStore` holds a `std::sync::Mutex` during
  blocking rusqlite I/O, stalling the executor thread for the duration of each
  query/write. (b) The `MutexGuard` is `!Send`; the code currently avoids
  holding it across `.await` (the embedder call is hoisted out of the
  transaction for this exact reason — comment at 478-481 — and
  `semantic_search_impl` drops+reacquires at 454-455), but there is no
  compile-time guard: a maintainer who adds an `.await` while the guard is
  live silently makes the future `!Send`, breaking multi-threaded tokio at
  runtime. (c) `SqliteConversationStore` chose `tokio::sync::Mutex`, which is
  await-aware (no `!Send` footgun) but still blocks the worker thread during
  the synchronous rusqlite call inside the guard. Neither store wraps its
  rusqlite work in `tokio::task::spawn_blocking`.
- Impact: latent correctness hazard for maintainers of `SqliteStore` (the
  `!Send`-guard trap) plus executor-thread blocking under disk pressure for
  both stores. No current crash (the careful await-avoidance holds today), so
  confidence is medium.
- Root cause: the std-mutex choice predates the async-awareness concern ("SQLite
  serialises writes anyway" optimises for correctness of serialisation, not for
  async runtime integration). The two stores were never converged.
- Direction: converge on `tokio::sync::Mutex` for `SqliteStore` (matching the
  conversation store) — removes the `!Send`-guard footgun and unifies the two.
  For non-blocking I/O the fuller fix is `spawn_blocking` around rusqlite
  calls, but that is a larger change; the mutex convergence is the minimal,
  consistent first step. Whichever is chosen, the struct-doc justification at
  62-64 should be updated to reflect the async-runtime concern, not just
  write serialisation.
- Regression validation: a doctest or test that constructs a `SqliteStore`,
  drives `put`+`search` on a multi-threaded tokio runtime, and asserts the
  future remains `Send` (e.g. a `fn _assert_send(f: impl Future + Send)`
  bound). Existing 15 tests still pass.
- Validation reports: [V03-01](../validations/F-MEM-02/V03-01.md)

### F-MEM-02-P3-01: `cosine_similarity` propagates NaN from embedder output; `bytes_to_vec` silently truncates non-4-aligned blobs

- Priority: P3
- Confidence: medium
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/sqlite_store.rs:239-251`
  (`cosine_similarity` — the `norm_a == 0.0 || norm_b == 0.0` guard uses `==`,
  which is `false` for NaN, so a NaN vector proceeds to `dot/(norm_a*norm_b)`
  = NaN/NaN = NaN); `:231-236` (`bytes_to_vec` uses `chunks_exact(4)`, which
  silently discards a trailing 1-3 byte remainder).
- Reachability: any `semantic`/`hybrid` search where the embedder returns a NaN
  (e.g. a malformed HTTP embedding response), or where a `store_vectors` blob
  is corrupted/truncated on disk.
- Expected invariant: vector-search scores presented to callers via
  `StoreItem.score` should be finite floats; a corrupted vector blob should be
  detected and skipped, not silently reshaped into a shorter vector.
- Observed behavior:
  - NaN: if any element of `query_vec` or a stored vector is NaN, `dot` and
    the norms become NaN; the `== 0.0` checks do not match NaN; the function
    returns NaN. The downstream sort at 451 (`partial_cmp().unwrap_or(Equal)`)
    is NaN-safe, so there is **no panic** and the NaN-scored items do not
    crash ordering, but `StoreItem.score = Some(NaN)` is returned to callers,
    where it can poison downstream arithmetic or display.
  - Empty vector: handled — `semantic_search_impl` filters `vec.is_empty()`
    (444-446) and `cosine_similarity` returns 0.0 on `a.is_empty()`.
  - Dimension mismatch: handled — `a.len() != b.len()` returns 0.0. So a
    query of dimension 768 vs stored vectors of 1536 silently score 0.0 (no
    error, no log). Reasonable but undocumented.
  - Blob truncation: a 5-byte blob → `chunks_exact(4)` yields one chunk → a
    1-element vector; a 7-byte blob → one chunk → 1-element vector. The
    shortened vector then hits the dimension-mismatch branch (0.0). No
    corruption is detected or logged; the item just silently scores zero.
- Impact: low — requires a buggy embedder (NaN) or DB corruption (truncation).
  No crash today (sort is NaN-safe). The leaked `Some(NaN)` score and the
  silent zeroing of corrupted vectors are defensive gaps, not active bugs.
- Root cause: no input validation on embedder output or on blob length.
- Direction: in `cosine_similarity`, treat any non-finite input as 0.0 (e.g.
  `if !dot.is_finite() { return 0.0; }` before the division, plus an
  `.iter().all(|x| x.is_finite())` guard, or filter NaN in
  `semantic_search_impl`). In `bytes_to_vec`, reject blobs whose length is not
  a multiple of 4 (log a warn and skip the row, like the empty-vector filter).
  Add unit tests for a NaN-containing vector and a truncated blob.
- Regression validation: new tests —
  `cosine_similarity(&[f32::NAN], &[1.0]) == 0.0`; a `semantic_search_impl`
  path where one stored blob is truncated asserts that item is skipped (not
  scored) and a warn is logged.
- Validation reports: [V04-01](../validations/F-MEM-02/V04-01.md)

### F-MEM-02-P3-02: `SqliteConversationStore` is not re-exported from the `echo_agent` facade (asymmetry with `SqliteStore`)

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/lib.rs:199` re-exports `SqliteStore` under
  `#[cfg(feature = "sqlite")]`; grep of `echo-agent/src/lib.rs` and
  `echo-agent/src/state/mod.rs` confirms `SqliteConversationStore` appears
  nowhere in the facade. It is reachable only via
  `echo_state::memory::SqliteConversationStore`
  (`echo-state/src/memory/mod.rs:48`).
- Reachability: any consumer that uses the `echo_agent` facade crate and wants
  the SQLite conversation backend.
- Expected invariant: the facade should expose the full memory-store menu
  symmetrically — if `SqliteStore` is at `echo_agent::memory::SqliteStore`,
  `SqliteConversationStore` should be at `echo_agent::memory::SqliteConversationStore`.
- Observed behavior: a consumer writing
  `use echo_agent::memory::SqliteConversationStore;` gets a compile error;
  they must instead depend on `echo_state` directly and reach
  `echo_state::memory::SqliteConversationStore`.
- Impact: minor discoverability/usability inconsistency. No correctness impact.
- Root cause: the facade re-export block at `lib.rs:197-205` was authored with
  only `SqliteStore` in mind; the conversation store was never added.
- Direction: add
  `#[cfg(feature = "sqlite")] pub use crate::memory::SqliteConversationStore;`
  alongside the existing `SqliteStore` re-export at `lib.rs:199`.
- Regression validation: `cargo check -p echo_agent --features sqlite` still
  succeeds; a doctest exercising `echo_agent::memory::SqliteConversationStore`
  compiles.
- Validation reports: [V01-01](../validations/F-MEM-02/V01-01.md)

### F-MEM-02-P3-03: schema migration swallows non-"duplicate column" errors; the migrated columns are then written but never read

- Priority: P3
- Confidence: medium
- Layer: framework
- Evidence:
  - `echo-agent/echo-state/src/memory/sqlite_store.rs:147-159` — the migration
    loop catches every `ALTER TABLE ADD COLUMN` error and only `warn!`s if the
    message does not contain `"duplicate column"`; execution continues either
    way.
  - The three migrated columns `importance`, `last_accessed`, `expires_at` are
    **written** by `put` (518-526 writes `importance`, `expires_at`) but never
    read back: `get` (588-599), `fetch_items` (305-315), `list` (799-810) all
    hard-code `importance: 5.0` and then call `apply_json_metadata` (935-945)
    which re-reads the same fields from the JSON `value` column.
    `prune_expired` (917-925) likewise reads `json_extract(value,
    '$.expires_at')`, not the SQL `expires_at` column.
  - Contrast: `echo-agent/src/state/sqlite.rs:73-84` (`SqliteRuntimeStateStore`)
    returns `Err` for any non-"duplicate column" migration failure — the
    correct pattern.
- Reachability: every `SqliteStore::new`/`open` runs the migration once. Reads
  use the JSON column, not the SQL columns.
- Expected invariant: (a) a schema migration failure (disk full, read-only DB,
  corruption) should be fatal — continuing leaves a partially-migrated schema
  where later `put`s reference columns that do not exist, failing at write
  time with a confusing error. (b) SQL columns that mirror JSON fields should
  be read or removed (YAGNI / AGENTS.md "代码清理").
- Observed behavior: (a) a non-duplicate ALTER error is logged and swallowed;
  the store opens "successfully" with missing columns. (b) The columns are
  effectively write-only dead schema — `apply_json_metadata` is the real
  source of `importance`/`expires_at`/`last_accessed` on read.
- Impact: low. The swallow is a latent failure mode (unlikely in practice —
  ALTER either succeeds or hits "duplicate column" on a healthy DB). The
  unread columns are misleading but harmless. No current data risk because
  JSON is the source of truth on read.
- Root cause: (a) the migration borrowed the "ignore duplicate column" pattern
  but made the filter too permissive. (b) Columns were added without switching
  reads to use them (or removing them).
- Direction: (a) return `Err` for non-"duplicate column" migration failures,
  matching `state/sqlite.rs:73-84`. (b) Either read the SQL columns in
  `get`/`list`/`fetch_items` (preferred for indexability) or drop the columns
  and rely on JSON — per AGENTS.md no-backward-compat, dead columns can be
  removed in place.
- Regression validation: test that a forced migration error (e.g. point the
  store at a read-only file) surfaces as `Err`; test that whichever read path
  is chosen reflects the written `importance`.
- Validation reports: [V04-01](../validations/F-MEM-02/V04-01.md)

### F-MEM-02-P3-04: empty-query behavior diverges across the three `Store` impls (`SqliteStore` returns empty; `FileStore`/`InMemoryStore` return all)

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/sqlite_store.rs:624-626`
  (`if keywords.is_empty() { return Ok(vec![]); }`) vs
  `echo-agent/echo-state/src/memory/store.rs:528-531`
  (`value_relevance_score` returns `1.0` when `keywords.is_empty()`, so
  `FileStore`/`InMemoryStore` return up to `limit` items all scored 1.0 — see
  F-MEM-01-P3-02).
- Reachability: any caller that calls `Store::search(ns, "", limit)`.
- Expected invariant: the `Store` trait should specify empty-query behavior and
  all impls should agree. Today the trait doc (`echo-core/src/memory/store.rs`
  `search` doc) only says "returns at most `limit` items sorted by relevance".
- Observed behavior: `SqliteStore` treats empty query as "no results";
  `FileStore`/`InMemoryStore` treat it as "match everything with score 1.0".
  A consumer swapping backends gets different behavior for the same call.
- Impact: minor — no data risk. Surprising divergence for callers who treat
  empty query as a no-op vs a wildcard. This is the SQLite-side half of the
  cross-store inconsistency F-MEM-01-P3-02 already raised for the file stores;
  recorded here because F-MEM-02 is where the divergence became visible.
- Root cause: the trait never fixed empty-query semantics; each impl picked its
  own convenience behavior.
- Direction: fix at the trait level (decide and document "empty query returns
  empty" vs "empty query matches all"), then make all three impls conform.
  `SqliteStore`'s current behavior is the more defensible default and would be
  the natural target.
- Regression validation: a trait-level doctest pinning the chosen behavior,
  plus impl-level tests for all three backends asserting they agree.
- Validation reports: [V04-01](../validations/F-MEM-02/V04-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Public-use justification: valid framework Store/ConversationStore impls; duplicate search across both repos; facade re-export symmetry | yes | passed_with_findings | [V01-01](../validations/F-MEM-02/V01-01.md) |
| V02 | Feature isolation: `sqlite` feature properly gated; compiles standalone ON and OFF; clean separation from `database` | yes | passed | [V02-01](../validations/F-MEM-02/V02-01.md) |
| V03 | Concurrency: WAL/busy_timeout/mutex model; both stores compared | yes | failed | [V03-01](../validations/F-MEM-02/V03-01.md) |
| V04 | Semantic-search numerical edge cases (NaN, empty, dim mismatch, blob truncation) + schema/orphan/empty-query inspection + sqlite test run | yes | failed | [V04-01](../validations/F-MEM-02/V04-01.md) |
| V05 | Historical-document drift | conditional | not_applicable | — |

V05 is not applicable: there are no prior F-MEM-02 reports or SQLite design
docs to classify. The two in-tree module docs (`sqlite_store.rs:1-43`,
`sqlite_conversation.rs:1-3`) make falsifiable claims verified directly in
V01-V04 and recorded under "Historical Claim Status" below.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `sqlite_store.rs:1-4` "Production-grade persistent storage, based on SQLite + FTS5 ... optional vector similarity" | current | V01-01 confirms `pub struct`, two constructors, FTS5 + vector tables; V04-01 confirms 15 tests green. |
| `sqlite_store.rs:62-64` "single `Mutex<Connection>` to avoid opening a new connection on every put/get/delete ... eliminates `SQLITE_BUSY` storms" | partial / regressed | The single-connection claim holds, but the std-`Mutex` choice introduces the `!Send`-guard footgun and executor-blocking documented in P2-03; "eliminates SQLITE_BUSY storms" is only true within one process and is undermined for the conversation store by the missing `busy_timeout` (P2-02). |
| `sqlite_conversation.rs:1-3` "Production-grade conversation storage ... with cascading deletes" | current | `ON DELETE CASCADE` at 80 verified; `delete_conversation` relies on it. |
| F-MEM-01 Handoff "F-MEM-02 should confirm whether `SqliteStore` already implements the durability contract `FileStore` is missing (it likely does, via SQLite's WAL)" | current | Confirmed: WAL + `synchronous=NORMAL` + per-op transactions give `SqliteStore` the crash-durability `FileStore` lacks (F-MEM-01-P2-01). |
| AGENTS.md "echo-state 的 `sqlite` feature 仅供框架其它复用方,echo-agent-cli 不启用" | current | V02-01 confirms clean `sqlite` feature gating; CLI not in scope and correctly excluded. |
| AGENTS.md "删除框架代码的判定" (sqlite is a framework option, retained even with no CLI caller) | current | V01-01 confirms both stores are pub, single-definition, trait-implementing framework options; `SqliteConversationStore` has zero live callers yet is retained as a menu option — correct per this rule. |

## Coverage And Uncertainty

- **Executed**: static read of both full SQLite source files + `mod.rs` +
  `util.rs` + both `Cargo.toml` feature tables + facade re-exports; duplicate
  search across both repos; four `cargo` commands (echo_state and echo_agent
  each compiled `--no-default-features --features sqlite` and
  `--no-default-features`); the 15 `echo_state` sqlite unit tests run green.
- **Not executed**: multi-process `SQLITE_BUSY` stress test (P2-02 is static
  PRAGMA inspection + the single-connection unit tests do not exercise
  contention); a dynamic `Send`-future assertion for P2-03 (the await-avoidance
  is verified by code reading, not a compile-time bound test); a forced
  migration-failure test for P3-03; executable NaN/truncated-blob tests for
  P3-01 (reasoned from the code; the sort's NaN-safety is verified by reading
  line 451).
- **Environmental limits**: macOS only; no Linux ext4 fsync-specific behaviour
  exercised (not relevant to SQLite, which manages its own durability via WAL).
- **Uncertain claims**: the medium-confidence P2-03 hinges on "a maintainer
  could add an `.await` under the std mutex guard" — true today only as a
  latent hazard, not an active bug. The P3-01 NaN path requires a buggy
  embedder; the in-tree `MockEmbedder` normalises (no NaN), so it is not
  reachable from existing tests.
- **`SqliteRuntimeStateStore`** (`src/state/sqlite.rs`) was read for pattern
  comparison only; its own correctness (e.g. it opens a fresh connection per
  op with no mutex and no busy_timeout) is out of F-MEM-02 scope and not
  reported as a finding here.

## Handoff

- **Conclusions downstream tasks may rely on**:
  - `SqliteStore` and `SqliteConversationStore` are valid, single-definition,
    feature-gated framework options implementing `Store` / `ConversationStore`
    respectively. They are NOT deletion candidates (AGENTS.md "删除框架代码的
    判定"). The `sqlite` feature is cleanly isolated (V02-01) and compiles
    standalone both ON and OFF.
  - `SqliteStore` satisfies the crash-durability contract `FileStore` is
    missing (F-MEM-01-P2-01): WAL + `synchronous=NORMAL` + per-op
    transactions. Framework consumers needing durable concurrent memory should
    prefer `SqliteStore`; `FileStore` remains the lightweight single-file
    option (and the CLI's choice).
  - The `Store`/`ConversationStore` trait matrix in F-MEM-01 V01-01 is
    confirmed complete; no additional SQLite-backed duplicate authority
    exists.

- **Reports downstream tasks must read**:
  - This report (F-MEM-02) for SQLite concurrency / schema / search-edge
    findings.
  - F-MEM-01 for the trait contracts, the file-backends' atomic-write recipe
    (the canonical framework durability reference), and the cross-store
    empty-query divergence (F-MEM-01-P3-02, deepened by F-MEM-02-P3-04).
  - F-FEAT-01 for the `sqlite` feature classification (live, correctly
    forwarded, not dead).

- **Conditions that make this report stale**:
  - Any change to the PRAGMA batches in either store (notably adding
    `busy_timeout` to the conversation store — fixes P2-02).
  - Any change to `SqliteStore`'s mutex type (fixes P2-03).
  - Any change to `prune_expired` that adds FTS/vector cleanup (fixes P2-01).
  - Any change to the root facade re-exports (fixes P3-02).
  - Any change to the `sqlite` feature wiring or `Cargo.toml` `[features]`.
  - A bump of the projection-envelope version (out of this task's scope but
    relevant to `SqliteConversationStore` round-trips via
    `project_message`/`restore_message`, owned by F-MEM-01 V04-01).

- **Follow-up task IDs** (no fixes implemented in this review task):
  - P2-01 (`prune_expired` orphan cleanup), P2-02 (conversation `busy_timeout`),
    P2-03 (converge mutex model / async-blocking) — framework cleanup.
  - P3-01 through P3-04 — defensive/cleanup.
  - A cross-store empty-query-semantics decision at the `Store` trait level
    spans F-MEM-01-P3-02 and F-MEM-02-P3-04; recommend a single dedicated
    trait-contract task rather than per-impl fixes.
