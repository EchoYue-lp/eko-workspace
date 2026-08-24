# F-MEM-02: SQLite framework capabilities

> Status: complete
> Reviewer: Codex primary reviewer, with isolated subagent evidence
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d35c9a432370b923acabfbb5f32d63`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: both source repositories clean; review touched neither source repository

## Question

Are `SqliteStore` and `SqliteConversationStore` valid independent framework
options with coherent feature/export reachability, concurrency, schema,
search/vector, namespace, identity, and error semantics?

## Scope

- `echo-agent/echo-state/src/memory/sqlite_store.rs` and
  `sqlite_conversation.rs`.
- Root/`echo_state` SQLite feature definitions, cfg modules/re-exports, facade
  and prelude visibility, docs, and official examples.
- Static schema/migration, transaction/connection ownership, concurrency,
  main/FTS/vector consistency, keyword/semantic/hybrid search, numeric/error
  boundaries, ConversationStore parity, and existing-test coverage.
- F-MEM-01's accepted lossless namespace and generic trait-default conclusions
  as canonical cross-backend requirements.

## Out Of Scope

- SQLite runtime checkpoint state (`src/state/sqlite.rs`), SQL database tools,
  and EKO persistence selection.
- Requiring EKO to enable SQLite. EKO intentionally does not use SQLite; that is
  not evidence that these reusable framework APIs should be deleted.
- Implementing fixes, migrating retained user data, or preserving backward
  compatibility. This development-stage project may choose atomic recreation.
- New Cargo/rustc/test/build/fixture execution. Each executable regression below
  is future validation, not a blocker for source-conclusive review.

## Inputs

- Root `AGENTS.md`.
- Shared `README.md`, `REPORTING.md`, and the `F-MEM-02` task card in
  `TASKS.md`.
- Codex track protocol `codex/README.md`.
- Accepted dependencies [F-MEM-01](F-MEM-01.md) and
  [F-FEAT-01](F-FEAT-01.md), limited to canonical Store contracts/namespace
  identity and SQLite feature isolation/export evidence.
- Framework SQLite docs/examples were treated as hypotheses and classified.
- No other reviewer's report was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | SQLite-backed Store/ConversationStore, durable transactions, FTS/vector indexes, connection/concurrency policy, and feature gates are legitimate reusable framework capabilities. |
| EKO product policy | EKO choosing file/in-memory stores and forbidding CLI-side SQLite remains application policy; it neither invalidates nor owns framework SQLite. |
| Adapter boundary | Independent consumers inject `Arc<dyn Store>` or `Arc<dyn ConversationStore>` through existing Agent APIs. No EKO adapter or second SQLite authority is needed. |
| Duplicate search | Searched both repositories for SQLite types/impls/constructors, all Store/ConversationStore impls, feature cfgs/exports, examples, SQL tables, and namespace/search helpers. Only `echo_state` owns these implementations. |
| Migration deletion | No API deletion is recommended. Repairs belong in current SQLite implementations. Lossless namespace encoding must be one shared mechanism across F-MEM-01/F-MEM-02; do not create a SQLite-only encoder. |

## Current Path

```text
Cargo feature sqlite
  -> echo_state/sqlite + optional rusqlite
  -> cfg modules sqlite_store / sqlite_conversation
  -> echo_state::memory::{SqliteStore, SqliteConversationStore}
  -> echo_agent::memory::*

SqliteStore
  Mutex<rusqlite::Connection>
  store_items (authority)
    + store_fts (keyword index)
    + store_vectors (optional semantic index)
  -> Arc<dyn Store>
  -> builder.with_memory_tools / agent.set_memory_store

SqliteConversationStore
  tokio::Mutex<rusqlite::Connection>
  conversation + message ON DELETE CASCADE
  -> Arc<dyn ConversationStore>
  -> agent.set_conversation_store
  -> framework transcript finalization
```

Feature ownership/export is coherent and F-FEAT-01 previously compiled the
standalone SQLite feature. Five official examples construct SqliteStore for
persistence, FTS, embeddings, and Agent integration, establishing reasonable
external use independent of EKO. SqliteConversationStore is public through the
memory facade, although the prelude omits it and current docs show the wrong
constructor.

SqliteStore serializes one instance through a single synchronous connection.
Put and delete generally transactionally update main/FTS/vector tables, but
embedder failure and maintenance take inconsistent branches. ConversationStore
enables foreign keys and cascades deletes, and message replacement uses a manual
transaction on one tokio-locked synchronous connection.

## Findings

### F-MEM-02-P1-01: Schema migration errors are logged and returned as a usable store

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/sqlite_store.rs:81`, `:130`, `:145`, `:518`
- Reachability: every SqliteStore constructor calls `init_tables`; later live
  puts always reference the ALTER-added importance/expires_at columns.
- Expected invariant: successful construction proves every required table and
  column exists; an unexpected migration/DDL failure aborts or atomically
  recreates the store.
- Observed behavior: non-duplicate ALTER failures only log warning and
  initialization continues. No schema version/postcondition check exists, so
  the returned store can fail immediately on normal writes.
- Impact: recovery from a partial, malformed, read-only, or otherwise failed
  schema is delayed into arbitrary runtime operations, making startup appear
  healthy and failures non-local.
- Root cause: migration handling uses error-string filtering plus warn-and-
  continue instead of one versioned/fail-closed schema transition.
- Direction: choose explicit versioned migrations or atomic development-stage
  recreation. Propagate every unexpected DDL error and verify required columns/
  indexes before returning. Delete warning-only continuation.
- Regression validation: old/partial/wrong-type/read-only schemas and injected
  ALTER failure; constructor must fail closed or recreate atomically, then all
  live SQL prepares.
- Validation reports: [V02-01](../validations/F-MEM-02/V02-01.md),
  [V08-01](../validations/F-MEM-02/V08-01.md)

### F-MEM-02-P1-02: Manual conversation rollback/commit can poison later operations

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/sqlite_conversation.rs:307`, `:317`, `:326`, `:347`, `:359`, `:364`
- Reachability: framework transcript finalization calls
  `ConversationStore::save_messages`; independent SQLite consumers inject this
  implementation through `set_conversation_store`.
- Expected invariant: any transaction failure leaves the shared connection in a
  known non-active state before another operation acquires it.
- Observed behavior: each statement failure issues `ROLLBACK` but discards its
  result. COMMIT failure returns immediately without an explicit rollback. The
  same connection remains shared for all later calls.
- Impact: one I/O/constraint/commit fault can leave the connection inside an
  active or uncertain transaction, causing subsequent unrelated conversation
  reads/writes to fail and masking whether replacement committed.
- Root cause: transaction state is hand-managed with SQL strings rather than a
  RAII transaction primitive whose drop/rollback outcome is controlled.
- Direction: use rusqlite transaction/savepoint ownership with one error path;
  explicitly recover/replace the connection if rollback or commit state is
  unknown. Delete ignored rollback results.
- Regression validation: inject delete/insert/update/commit failures; after
  every returned error, old transcript remains intact and a fresh transaction
  succeeds on the same or deliberately reopened connection.
- Validation reports: [V03-01](../validations/F-MEM-02/V03-01.md),
  [V08-01](../validations/F-MEM-02/V08-01.md)

### F-MEM-02-P1-03: Main, FTS, and vector state diverge on supported mutations

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/sqlite_store.rs:482`, `:518`, `:530`, `:545`, `:710`, `:734`, `:909`
- Reachability: official embedding examples call `with_embedder` and Store put/
  search; prune is a public Store method for independent consumers.
- Expected invariant: after a successful put/delete/prune, every secondary index
  describes exactly the current main row or is intentionally absent.
- Observed behavior: overwriting a vectorized key when embedding fails updates
  main+FTS but retains its old vector; semantic search ranks the new value using
  old content. Prune deletes only main rows and leaves FTS/vector orphans. Delete
  ignores vector delete failure and commits main+FTS deletion.
- Impact: semantic ranking can return materially wrong memories; orphan rows
  consume bounded candidate slots/storage and secondary search work; successful
  mutation no longer means indexes agree.
- Root cause: optional semantic degradation is modeled as “skip vector write”
  rather than “remove now-invalid vector,” and maintenance/deletion do not share
  one transactional dependency policy.
- Direction: encode indexes as dependent state. On embedding failure remove the
  old vector or fail the put according to explicit policy; delete/prune all
  dependent rows in one transaction (or use enforced cascades). Never ignore
  cleanup failure.
- Regression validation: overwrite with failed embed, prune expired item, and
  inject secondary-delete failure; assert exact table state and search output.
- Validation reports: [V04-01](../validations/F-MEM-02/V04-01.md),
  [V08-01](../validations/F-MEM-02/V08-01.md)

### F-MEM-02-P1-04: Search discards relevance order and accepts corrupt numeric vectors

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/sqlite_store.rs:221`, `:254`, `:324`, `:395`, `:608`
- Reachability: `Store::search/search_with` are used directly by demo27/demo46
  and through Agent memory tools when this public backend is injected.
- Expected invariant: returned items stay relevance-sorted and malformed/non-
  finite vector data cannot silently influence results.
- Observed behavior: FTS/semantic produce ordered keys, then batch fetch uses
  `WHERE key IN (...)` without ORDER BY and does not restore key order. Scores
  remain attached but vector result order becomes database-dependent. Blob
  decoding ignores trailing bytes; NaN/Inf vectors are accepted; cosine can
  become non-finite and sorting treats incomparable scores as equal. Dimension
  mismatch returns zero rather than distinguishing schema/model drift.
- Impact: callers requesting top relevance receive nondeterministic ordering,
  hybrid ranks are built from the wrong semantic sequence, and corrupt/model-
  drifted embeddings can silently degrade retrieval.
- Root cause: ranked-key identity and row hydration were split without an order
  restoration step; vector persistence has no dimension/finite/integrity
  metadata validation.
- Direction: hydrate by key then reorder exactly by ranked input; validate blob
  length, finite components/scores, and one stored embedding dimension/model
  identity. Treat corrupt/drifted rows as typed errors or rebuild candidates.
- Regression validation: multi-hit exact order across reopen, malformed/trailing
  blob, NaN/Inf/zero, and embedding dimension/model change.
- Validation reports: [V05-01](../validations/F-MEM-02/V05-01.md),
  [V07-01](../validations/F-MEM-02/V07-01.md),
  [V08-01](../validations/F-MEM-02/V08-01.md)

### F-MEM-02-P1-05: Transcript replacement invalidates persisted message identity

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/memory/conversation.rs:43`, `:63`; `echo-agent/echo-state/src/memory/sqlite_conversation.rs:307`, `:331`, `:371`
- Reachability: framework projection sends None IDs, but the public
  ConversationStore contract accepts `StoredMessage { id: Some(..) }` for
  independent import/restore consumers. Conversation summary stores a
  `compressed_before_id` boundary.
- Expected invariant: an explicit persisted message ID and compression boundary
  remain meaningful across the documented upsert/replace operation, or the API
  rejects/renames unsupported identity input.
- Observed behavior: save_messages deletes every row, ignores all supplied IDs,
  and inserts fresh auto-increment IDs. It does not remap/clear
  compressed_before_id. A successful replacement can therefore leave the
  conversation boundary/external references pointing to deleted IDs.
- Impact: compression resume can use a stale boundary, and imported or linked
  message identity is lost despite being part of the public model.
- Root cause: `StoredMessage.id` simultaneously represents durable identity and
  optional insert metadata, while replace-all SQL treats it as disposable.
- Direction: decide one authority: preserve validated supplied IDs/upsert rows,
  or remove supplied identity from write input and atomically recompute/clear
  every dependent boundary. Do not patch identity in application adapters.
- Regression validation: replace a summarized transcript with explicit and None
  IDs; assert stable identity or an explicit remap plus valid compression
  boundary and ordered restore.
- Validation reports: [V06-01](../validations/F-MEM-02/V06-01.md),
  [V08-01](../validations/F-MEM-02/V08-01.md)

### F-MEM-02-P2-01: Synchronous SQLite work blocks async executor threads

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/sqlite_store.rs:62`, `:124`, `:464`; `echo-agent/echo-state/src/memory/sqlite_conversation.rs:20`, `:97`
- Reachability: every Store/ConversationStore operation returns a BoxFuture but
  directly invokes synchronous rusqlite prepare/query/execute while holding the
  connection lock. Full-table semantic scans and conversation searches are
  live public operations.
- Expected invariant: potentially blocking database I/O does not monopolize
  Tokio worker threads or serialize unrelated async work invisibly.
- Observed behavior: no `spawn_blocking`/dedicated executor exists.
  SqliteStore uses `std::sync::Mutex`; SqliteConversationStore uses an async
  mutex but still performs blocking I/O after acquisition. Conversation
  connections also lack busy_timeout for multiple instances.
- Impact: a slow disk, large scan, WAL contention, or 5-second Store busy wait
  can stall runtime workers and unrelated Agent futures; independently opened
  conversation stores may immediately surface busy failures.
- Root cause: a synchronous connection was placed directly behind async traits
  without a blocking boundary or pool/service owner.
- Direction: move rusqlite work to a bounded blocking executor or dedicated
  serialized DB service; make busy/retry policy consistent across both stores.
  Keep one framework authority rather than adding async SQL adapters elsewhere.
- Regression validation: constrained Tokio worker pool with slow/busy concurrent
  Store and ConversationStore operations; unrelated timer/Agent future must
  progress and contention must end in bounded typed outcomes.
- Validation reports: [V03-01](../validations/F-MEM-02/V03-01.md),
  [V08-01](../validations/F-MEM-02/V08-01.md)

### F-MEM-02-P2-02: Read/search paths silently convert corrupt database rows into omissions

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/sqlite_store.rs:294`, `:304`, `:362`, `:371`, `:646`, `:675`, `:802`, `:815`, `:842`
- Reachability: keyword/semantic search and list are public memory tool paths;
  hybrid fallback is used with `SearchQuery::hybrid`.
- Expected invariant: database corruption/type mismatch is distinguishable from
  “no memory matched,” and typed errors are matched by type/category.
- Observed behavior: `rows.flatten()`/`filter_map(r.ok())` drop row conversion
  failures; batch JSON parse failure skips items; list turns corrupt JSON into
  `Value::Null` and drops row errors. Hybrid identifies missing embedder by
  formatting the error and substring matching. Extreme usize/u64 values also
  cross unchecked i64 conversions.
- Impact: corrupted durable knowledge disappears from results without a failure
  signal, operators cannot distinguish empty search from data damage, and error
  wording changes can alter hybrid behavior.
- Root cause: convenience iterator filtering and display strings replaced a
  fail-fast typed persistence boundary.
- Direction: collect rows/JSON with `Result`, propagate contextual typed
  corruption errors, and pattern-match MemoryError variants. Add checked numeric
  conversion/resource limits; delete all error-dropping iterators in durable
  reads.
- Regression validation: corrupt row type/JSON/vector, unsupported mode, and
  usize/u64 extremes; assert exact typed failure rather than omission/Null.
- Validation reports: [V05-01](../validations/F-MEM-02/V05-01.md),
  [V07-01](../validations/F-MEM-02/V07-01.md),
  [V08-01](../validations/F-MEM-02/V08-01.md)

## Cross-Task Findings Not Duplicated

- SQLite uses the same lossy slash namespace encoder and textual prefix as
  non-SQLite stores (`sqlite_store.rs:401,472,569,615,712,756,766,787,910`).
  Canonical ownership remains **F-MEM-01-P1-01**; its repair/validation must
  include SQLite.
- SqliteConversationStore inherits the non-atomic default
  `ensure_conversation`. Canonical public-contract ownership remains
  **F-MEM-01-P2-01**; SQLite should override it atomically as one consumer of
  that fix.

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition, feature/export, duplicate, and public-use matrix | yes | passed | [V01-01](../validations/F-MEM-02/V01-01.md) |
| V02 | Schema initialization/migration fail-closed semantics | yes | failed/finding | [V02-01](../validations/F-MEM-02/V02-01.md) |
| V03 | Connection concurrency, blocking, transaction recovery | yes | failed/findings | [V03-01](../validations/F-MEM-02/V03-01.md) |
| V04 | Main/FTS/vector mutation consistency | yes | failed/finding | [V04-01](../validations/F-MEM-02/V04-01.md) |
| V05 | Keyword/semantic/hybrid order, errors, numeric edges | yes | failed/findings | [V05-01](../validations/F-MEM-02/V05-01.md) |
| V06 | ConversationStore schema/filter/ensure/identity parity | yes | failed/finding plus positive cascade/order evidence | [V06-01](../validations/F-MEM-02/V06-01.md) |
| V07 | UTF-8/panic/overflow/error/namespace parity scan | yes | failed/findings plus positive UTF-8 evidence | [V07-01](../validations/F-MEM-02/V07-01.md) |
| V08 | Existing test coverage inventory | yes | failed/gaps | [V08-01](../validations/F-MEM-02/V08-01.md) |
| V09 | Historical docs/examples drift | yes | failed/drift plus public-use evidence | [V09-01](../validations/F-MEM-02/V09-01.md) |
| V10 | New executable concurrency/error/numeric/Cargo fixture | future implementation validation | not run by rule | No fake report; execute each scenario separately in implementation/Q tasks. |
| V11 | Delegated report-integrity gate | yes | passed | [V11](../validations/F-MEM-02/V11-01.md) |
| V30 | Primary source-anchor acceptance | yes | passed | [V30](../validations/F-MEM-02/V30-01.md) |
| V31 | Primary acceptance integrity and source isolation | yes | passed | [V31](../validations/F-MEM-02/V31-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| F-FEAT-01: standalone `sqlite` feature compiles | current at reviewed commit | [F-FEAT-01](F-FEAT-01.md), [V01-01](../validations/F-MEM-02/V01-01.md) |
| `sqlite_store.rs:3`: production-grade persistence | regressed/not established | V02-V08 identify schema, index, search, blocking, and coverage defects. |
| demo27: persistent FTS/vector/Agent Store is a supported framework use | current | [V01-01](../validations/F-MEM-02/V01-01.md), [V09-01](../validations/F-MEM-02/V09-01.md) |
| English/Chinese memory docs: `SqliteConversationStore::open(...).await` | stale | Actual public constructor is synchronous `new`; [V09-01](../validations/F-MEM-02/V09-01.md) |
| Memory docs: SQLite is the sole concrete ConversationStore | stale | FileConversationStore is public/live per F-MEM-01. |

## Coverage And Uncertainty

- No new build, test, Cargo, rustc, database, or dynamic fixture ran. Findings
  are static/source-conclusive; quantitative executor-blocking and failure-
  injection behavior remain future regressions.
- SQLite library/version-specific behavior for COMMIT failure, FTS row ordering,
  parameter overflow, and malformed BLOBs was not dynamically measured. Reports
  state only what current source accepts/ignores and what SQL does not guarantee.
- `SqliteStore::dedup_by_content` uses the trait no-op because no override exists;
  that is an optional default contract, not reported as a defect absent a claim
  that SQLite supports deduplication.
- Search LIKE escaping handles `%` but not `_` and omits an explicit ESCAPE
  clause. This is a fuzzy-search semantics ambiguity, not elevated separately
  because current docs do not promise literal LIKE matching.
- Root facade's direct optional rusqlite dependency may also support root-owned
  runtime-state SQLite; dependency ownership was not reopened beyond F-FEAT-01.

## Handoff

- Primary should independently sample V02's warn-and-continue migration,
  V04's overwrite/prune table transitions, V05's ranked-key hydration, and
  V06's message-ID replacement before acceptance.
- The first implementation should centralize the F-MEM-01 lossless namespace
  encoder across all Store backends; do not patch SQLite separately.
- Fix transaction/index invariants before performance work: fail-closed schema,
  RAII transaction recovery, and exact main/FTS/vector consistency prevent data
  damage. Then move synchronous rusqlite work behind one bounded blocking owner.
- F-MEM-01-P2-01 remains canonical for atomic ensure; SqliteConversationStore
  should supply the backend-specific atomic implementation.
- F-CTX/A-MEM/X-MEM consumers may rely on SQLite being a valid optional
  framework direction, but not on current result order, silent-error behavior,
  message-ID stability, or async nonblocking behavior.
- Primary review independently sampled migration handling, ranked row hydration,
  vector/index mutation, pruning, conversation transaction ownership, and
  message identity replacement. The seven findings and priorities were
  accepted; see V30.
- This report becomes stale if the reviewed commits change SQLite schema,
  connection/transaction owner, index maintenance, search hydration/scoring,
  message replacement, namespace encoding, features, or exports.
