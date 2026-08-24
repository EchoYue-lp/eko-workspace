# F-MEM-01: General memory and conversation stores

> Status: complete
> Reviewer: Codex primary reviewer, with isolated subagent evidence
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d35c9a432370b923acabfbb5f32d63`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: `echo-agent` clean; `echo-agent-cli` had 79 unrelated modified generated TypeScript files at the final baseline; review touched neither source repository

## Question

Are the `Store`/`ConversationStore` contracts and their in-memory/file
implementations durable, atomic, path-safe, semantically aligned, panic-safe,
and reachable as independent framework capabilities?

## Scope

- Canonical contracts and data types in `echo-agent/echo-core/src/memory`.
- `InMemoryStore`, `FileStore`, `FileConversationStore`, `EmbeddingStore`, and
  `TypedMemoryStore` in `echo-agent/echo-state/src/memory`.
- Root facade/re-exports, framework Agent construction/finalization, and
  definition-registration-live-caller traces.
- EKO constructors and consumers only to establish real reachability; the
  framework APIs were evaluated independently of whether EKO uses each option.
- Static durability/atomicity, corrupt/truncated input, namespace/ID path
  safety, round-trip/search/pagination semantics, UTF-8/panic/overflow risks,
  and existing-test coverage.

## Out Of Scope

- SQLite stores, schema, feature isolation, and database concurrency
  (`F-MEM-02`). EKO's lack of SQLite is not a reason to delete framework SQLite
  APIs.
- Runtime checkpoint/snapshot recovery (`F-RCT-05`), compression (`F-CMP-01`),
  EKO conversation artifact lifecycle (`A-STATE-01`), and EKO memory refresh
  policy (`A-MEM-01`).
- Implementing fixes or editing source/index files.
- New Cargo, rustc, test, build, Clippy, or dynamic fixture execution. Per the
  review-stage rule, executable regressions below are future validations and do
  not block source-conclusive findings.

## Inputs

- Root `AGENTS.md`.
- Shared `README.md`, `REPORTING.md`, and the `F-MEM-01` task card in
  `TASKS.md`.
- Codex reviewer protocol `codex/README.md`.
- Dependency report [F-CORE-01](F-CORE-01.md), limited to framework identity,
  error, and facade ownership boundaries.
- Framework memory/subagent documentation and EKO `docs/MASTER-PLAN.md` were
  treated as historical hypotheses and classified below.
- No other reviewer's report was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Store/ConversationStore contracts, namespace identity, generic in-memory/file persistence, atomic replacement, corruption behavior, search/pagination semantics, and typed/embedding decorators are reusable framework mechanisms. |
| EKO product policy | Workspace/global path selection, hot-memory promotion, which surfaces display/search conversations, retention, and workspace refresh remain application policy. |
| Adapter boundary | EKO may select paths and inject one `Arc<dyn Store>`/`Arc<dyn ConversationStore>`; it should not reimplement persistence, search, namespace encoding, ID allocation, or recovery semantics. |
| Duplicate search | Searched both repositories for the traits, all `impl Store`/`impl ConversationStore`, File/InMemory/Embedding/Typed types, constructors, re-exports, `join("/")`, conversation search, and production callers. Canonical traits exist once in `echo_core`; EKO contains consumers, not a competing store authority. |
| Migration deletion | No cross-repository move is recommended. Repairs belong in the existing framework implementations; replace/delete only the lossy namespace encoder, fall-open parser, fixed-temp/full-snapshot write protocol, and misleading defaults after callers/tests migrate. |

## Current Path

```text
echo_core
  Store + StoreItem + SearchQuery
  ConversationStore + Conversation/StoredMessage/Filter
       |
       v
echo_state
  InMemoryStore             process-local Store implementation
  FileStore                 whole-map JSON Store implementation
  EmbeddingStore            Store decorator + optional vector JSON
  TypedMemoryStore          typed/filtering facade over Store
  FileConversationStore     one JSON record per conversation + _meta.json
       |
       v
echo_agent facade
  ReactAgent::new -> setup_memory_store -> FileStore
                    -> remember/recall/search_memory/forget
  builder.store/with_memory_tools -> caller-selected Store
  run finalization -> ensure_conversation -> save_messages
       |
       v
EKO adapters
  workspace Agent -> create_memory_store_at -> FileStore
  CLI/Tauri scheduler -> FileStore (InMemory fallback)
  AppState/Agent -> create_conversation_store -> FileConversationStore
  GUI/TUI history search -> ConversationStore::search_conversations
```

`Store` models namespace as an ordered string array
(`echo-core/src/memory/store.rs:1-6`, `:182-239`). Both in-memory and file
implementations instead flatten it to one slash-delimited `String`. FileStore
loads a full map once per instance and persists full snapshots. Its lock is
instance-local. FileConversationStore uses a separate JSON file per conversation
and one `_meta.json`; normal writes use unique temporary names, file sync,
rename, cleanup, and Unix directory sync. Its `Mutex<StoreMeta>` serializes one
instance, and the implementation overrides the trait's non-atomic ensure and
bounded default search.

## Findings

### F-MEM-01-P0-01: Corrupt FileStore input is accepted as empty and later overwritten

- Priority: P0
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/store.rs:227`, `:232`, `:254`, `:309`; `echo-agent/echo-agent-app-core` is not an authority (live EKO caller is `echo-agent-cli/echo-agent-app-core/src/infra.rs:1315`)
- Reachability: framework `ReactAgent::new` calls `setup_memory_store` and
  constructs FileStore (`echo-agent/src/agent/react/mod.rs:750`); EKO injects
  the same implementation for workspace memory. Parse failure returns `Ok`, so
  both callers retain the store and memory tools can issue the next put.
- Expected invariant: malformed/truncated durable data fails closed, preserves
  recoverable bytes, and cannot be silently converted into a successful empty
  store.
- Observed behavior: `serde_json::from_str` failure logs and substitutes an
  empty map. The next mutation/flush serializes the empty/current snapshot and
  renames it over the original file.
- Impact: one truncated or partially corrupt memory file can be silently reduced
  to only post-corruption writes, permanently losing prior cross-session
  knowledge.
- Root cause: constructor recovery policy conflates “new file” with “existing
  file failed to parse,” and no quarantine/backup/generation is retained.
- Direction: return a typed corruption error (or enter an explicit read-only
  recovery mode), preserve/quarantine original bytes, and delete the
  `unwrap_or_else(HashMap::new)` fall-open branch. Do not silently fall back to
  InMemoryStore for an existing corrupt durable file.
- Regression validation: corrupt and truncate valid files at multiple offsets;
  assert open/mutation fails closed, original bytes survive, and an explicit
  recovery path can restore a last-known-good generation.
- Validation reports: [V03-01](../validations/F-MEM-01/V03-01.md),
  [V08-01](../validations/F-MEM-01/V08-01.md)

### F-MEM-01-P0-02: Shared FileStore handles lose committed updates

- Priority: P0
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/store.rs:220`, `:227`, `:254`, `:258`, `:280`; `echo-agent/docs/en/06-subagent.md:152`; `echo-agent/docs/en/03-memory.md:104`
- Reachability: framework docs explicitly configure multiple Subagents with the
  same `memory_path("./store.json")` and unique namespaces. Each default Agent
  independently constructs FileStore through `setup_memory_store`.
- Expected invariant: successful writes through two handles to one advertised
  file are serialized/merged or explicitly rejected; a durable rename survives
  a crash.
- Observed behavior: each handle owns a stale full-map snapshot behind a
  different RwLock. A write replaces the whole file, so the later stale writer
  removes the earlier handle's committed item. Concurrent flushes also share
  the same `<path>.tmp`; the parent directory is not synced after rename.
- Impact: the documented multi-Subagent setup can lose valid memory even when
  namespaces differ. Overlap can also produce spurious rename/remove failure,
  and a crash can lose the directory entry after reported success.
- Root cause: a process-local snapshot/lock is presented as shared-file storage;
  the flush protocol lacks cross-instance coordination, unique temp ownership,
  generation/CAS, and directory durability.
- Direction: select one explicit contract. Prefer a process-scoped shared owner
  keyed by canonical path plus unique temp files/generation checking and parent
  sync; alternatively reject a second writer and update the public docs. Delete
  fixed-temp full-snapshot replacement once the authority changes.
- Regression validation: interleave two independently opened handles across
  namespaces and keys; assert no lost update, temp collision, or stale delete;
  inject crash points before/after rename and reopen.
- Validation reports: [V03-02](../validations/F-MEM-01/V03-02.md),
  [V09-01](../validations/F-MEM-01/V09-01.md)

### F-MEM-01-P0-03: `_meta` is accepted as a conversation ID and aliases metadata

- Priority: P0
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/file_conversation.rs:86`, `:91`, `:177`, `:206`, `:457`
- Reachability: `FileConversationStore` is public, EKO constructs it at
  `echo-agent-cli/echo-agent-app-core/src/infra.rs:1214`, and framework
  finalization calls `ensure_conversation` for the configured conversation ID
  at `echo-agent/src/agent/snapshot.rs:684`. Typical EKO IDs avoid `_meta`, but
  the public contract accepts caller-supplied external IDs.
- Expected invariant: every accepted external ID maps injectively into a record
  namespace disjoint from internal files.
- Observed behavior: `safe_segment("_meta")` succeeds and `conv_path` becomes
  `_meta.json`, the exact metadata path. On a fresh store, create writes the
  record, `persist_meta` immediately overwrites it, and create returns success;
  subsequent record reads parse StoreMeta as ConversationRecord and fail.
- Impact: a valid-looking public input creates an acknowledged conversation that
  is immediately corrupted/lost and can disrupt metadata reads.
- Root cause: path safety validates traversal characters but does not reserve
  internal filenames or make internal/external namespaces structurally
  disjoint.
- Direction: reject every internal/reserved name before I/O or encode record IDs
  into a separate records directory/injective filename representation. Delete
  the direct `<id>.json` mapping if a disjoint layout replaces it.
- Regression validation: table-test `_meta`, internal temp patterns, case-folded
  variants on supported filesystems, traversal, Unicode, and normal generated
  IDs; rejected IDs must leave no file.
- Validation reports: [V05-01](../validations/F-MEM-01/V05-01.md),
  [V08-01](../validations/F-MEM-01/V08-01.md)

### F-MEM-01-P1-01: Namespace flattening violates Store isolation and prefix semantics

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/memory/store.rs:1`, `echo-agent/echo-state/src/memory/store.rs:44`, `:138`, `:399`; `echo-agent/echo-state/src/memory/embedding_store.rs:223`; `echo-agent/docs/en/03-memory.md:104`
- Reachability: InMemoryStore and FileStore are live framework/EKO backends;
  EmbeddingStore is registered when embedding is configured. Agent names and
  public callers supply namespace components.
- Expected invariant: ordered string arrays have lossless identity, and prefix
  filtering compares whole components.
- Observed behavior: `['a/b','c']` and `['a','b/c']` both encode as `a/b/c`;
  decoding changes their shape. Raw `starts_with` also makes prefix `['user']`
  match `['user2', ...]`. Embedding vectors use the same lossy key.
- Impact: logically isolated agents/users can read, overwrite, delete, or
  semantically retrieve each other's records when components contain separators
  or share textual prefixes; namespace enumeration lies about structure.
- Root cause: an unescaped presentation string is used as canonical identity.
- Direction: use a structured key (`Vec<String>` with Hash/Eq), a length-prefixed
  encoding, or lossless serialized components. Centralize it once for every
  Store/decorator and delete all local `join/split/starts_with` encoders.
- Regression validation: property-test injectivity and round-trip for arbitrary
  Unicode/separator components; test prefix component boundaries and parity
  across InMemory/File/Embedding and F-MEM-02 backends.
- Validation reports: [V04-01](../validations/F-MEM-01/V04-01.md),
  [V08-01](../validations/F-MEM-01/V08-01.md),
  [V09-01](../validations/F-MEM-01/V09-01.md)

### F-MEM-01-P1-02: FileStore maintenance self-deadlocks before persistence

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/store.rs:254`, `:430`, `:448`
- Reachability: `prune_expired` and `dedup_by_content` are public Store
  capabilities implemented by FileStore. Repository search found no current
  production caller, but an independent framework consumer can invoke them and
  CLI non-use is not a deletion criterion.
- Expected invariant: maintenance terminates, releases mutation locks before
  I/O, and reports an error if removed data was not persisted.
- Observed behavior: both methods hold the FileStore write guard while awaiting
  `flush`, which awaits a read guard on the same tokio RwLock. Any branch that
  removes at least one item waits indefinitely. The flush result is also
  discarded.
- Impact: an agent/service enabling expiration or deduplication can hang its
  maintenance operation indefinitely; after the lock issue is fixed alone,
  failed persistence could still be reported as successful deletion.
- Root cause: persistence was called inside the mutation critical section and
  error handling was intentionally erased.
- Direction: compute/mutate under the write guard, take a coherent snapshot or
  release the guard, persist through the same transactional write primitive,
  and propagate/rollback failures. Delete the ignored-result branches.
- Regression validation: force at least one expiry and duplicate under timeout,
  reopen for durability, and inject serialization/write/rename failures to
  assert no false success or retained half-state.
- Validation reports: [V03-03](../validations/F-MEM-01/V03-03.md),
  [V08-01](../validations/F-MEM-01/V08-01.md)

### F-MEM-01-P2-01: ConversationStore defaults silently weaken their public contract

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/memory/conversation.rs:145`, `:160`; `echo-agent/echo-state/src/memory/file_conversation.rs:398`, `:442`
- Reachability: these are public default trait methods inherited by any
  independent backend. FileConversationStore correctly overrides them; the live
  transcript finalization explicitly calls ensure. SQLite behavior is deferred.
- Expected invariant: a default documented as scanning all conversations has no
  hidden candidate cap, and “ensure” either guarantees atomic create-if-absent or
  clearly exposes best-effort/racy semantics.
- Observed behavior: default search requests only the latest 100 conversations,
  so an older matching record is never examined. Default ensure performs a
  get-then-create across two awaits, allowing concurrent callers to both observe
  missing and one to fail create.
- Impact: a new reusable backend that relies on reasonable defaults silently
  omits valid history and can fail an idempotent-looking ensure under
  concurrency. The public API is more reassuring than its semantics.
- Root cause: convenience defaults embed arbitrary product/performance policy
  and a non-atomic composition without naming those limits.
- Direction: remove the hidden cap and require a backend-owned scan primitive,
  or make search required; make ensure required/atomic or rename/document a
  best-effort default. Retain FileConversationStore's single-lock overrides.
- Regression validation: a minimal backend with >100 conversations and an
  old-only match; two concurrent ensures for the same ID; assert the chosen
  public semantics.
- Validation reports: [V06-01](../validations/F-MEM-01/V06-01.md),
  [V08-01](../validations/F-MEM-01/V08-01.md)

### F-MEM-01-P2-02: FileConversationStore does not enforce promised message order

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/memory/conversation.rs:136`, `echo-agent/echo-state/src/memory/file_conversation.rs:350`, `:378`
- Reachability: framework finalization writes ordered projected messages, so the
  primary hot path usually happens to satisfy the order. Public callers can
  supply/reuse explicit IDs; EKO history reads the returned vector.
- Expected invariant: `get_messages` returns messages sorted by ID ascending as
  the trait states, independent of insertion order.
- Observed behavior: `save_messages` preserves caller vector order and
  `get_messages` returns it unchanged; explicit supplied IDs are never sorted.
- Impact: imported/replayed transcript records can render or restore in the
  wrong causal order, and backend behavior diverges under one trait.
- Root cause: ordering is documented at read time but delegated implicitly to
  write-call order.
- Direction: define tie/None handling and sort/validate at one canonical
  boundary; do not make application adapters repair backend-specific ordering.
- Regression validation: save IDs `[3,1,2]`, reopen, and assert `[1,2,3]` plus a
  defined policy for None/duplicate IDs.
- Validation reports: [V06-01](../validations/F-MEM-01/V06-01.md),
  [V08-01](../validations/F-MEM-01/V08-01.md)

### F-MEM-01-P3-01: TypedMemoryStore oversampling can overflow caller limits

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-state/src/memory/typed_store.rs:238`
- Reachability: `TypedMemoryStore::search_typed` is a public framework API; no
  non-test in-repository caller was found, which limits current blast radius but
  does not make it dead.
- Expected invariant: public `usize` limits cannot panic or wrap during internal
  oversampling.
- Observed behavior: `limit * 3` is unchecked; it panics with overflow checks or
  wraps in optimized builds.
- Impact: an extreme external limit can crash a checked-build process or produce
  an unexpectedly small/large backend search limit.
- Root cause: a heuristic multiplier was used as ordinary arithmetic instead of
  a bounded resource calculation.
- Direction: use checked/saturating multiplication capped by an explicit
  backend maximum; preserve the requested final limit.
- Regression validation: call with `0`, `usize::MAX / 3`, and `usize::MAX` and
  assert no panic plus bounded backend work.
- Validation reports: [V07-01](../validations/F-MEM-01/V07-01.md),
  [V08-01](../validations/F-MEM-01/V08-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition, implementation, duplicate, and layer matrix | yes | passed | [V01-01](../validations/F-MEM-01/V01-01.md) |
| V02 | Registration and real framework/EKO reachability | yes | passed | [V02-01](../validations/F-MEM-01/V02-01.md) |
| V03a | FileStore corrupt/truncated handling | yes | failed/finding | [V03-01](../validations/F-MEM-01/V03-01.md) |
| V03b | FileStore shared-handle atomicity/durability | yes | failed/finding | [V03-02](../validations/F-MEM-01/V03-02.md) |
| V03c | FileStore prune/dedup lock and error semantics | yes | failed/finding | [V03-03](../validations/F-MEM-01/V03-03.md) |
| V04 | Namespace identity and isolation | yes | failed/finding | [V04-01](../validations/F-MEM-01/V04-01.md) |
| V05 | FileConversation path-safe IDs, corruption, atomic writes | yes | failed/finding plus positive evidence | [V05-01](../validations/F-MEM-01/V05-01.md) |
| V06 | Conversation round-trip/search/pagination/default semantics | yes | failed/finding plus positive evidence | [V06-01](../validations/F-MEM-01/V06-01.md) |
| V07 | UTF-8/panic/overflow inspection | yes | failed/finding plus positive evidence | [V07-01](../validations/F-MEM-01/V07-01.md) |
| V08 | Existing test coverage inventory | yes | failed/gaps | [V08-01](../validations/F-MEM-01/V08-01.md) |
| V09 | Historical-document drift | yes | failed/drift | [V09-01](../validations/F-MEM-01/V09-01.md) |
| V10 | New executable fixture/Cargo test | future implementation validation | not run by rule | No fake report created; Q-FW-01 or fix task must execute each named regression separately. |
| V30 | Primary static source reconstruction and acceptance | yes | mixed, final passed | [01](../validations/F-MEM-01/V30-01.md), [02](../validations/F-MEM-01/V30-02.md), [03](../validations/F-MEM-01/V30-03.md), [04](../validations/F-MEM-01/V30-04.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `echo-agent-cli/docs/MASTER-PLAN.md:70`: FileConversationStore has unique temp, fsync, atomic rename, parent sync, corruption errors, path-safe IDs, atomic ensure | current with one unhandled reserved-name defect | [V05-01](../validations/F-MEM-01/V05-01.md) |
| `echo-agent/docs/en/03-memory.md:74` and core module comment: concrete ConversationStore is SQLite | stale | FileConversationStore is public and live; [V01-01](../validations/F-MEM-01/V01-01.md) |
| `echo-agent/docs/en/03-memory.md:104`: different string-array namespaces are completely isolated | regressed | Slash encoding is non-injective; [V04-01](../validations/F-MEM-01/V04-01.md) |
| `echo-agent/docs/en/06-subagent.md:152`: multiple Subagents may use the same FileStore path with unique namespaces | regressed | Independent full snapshots lose updates; [V03-02](../validations/F-MEM-01/V03-02.md) |
| `echo-agent/docs/en/12-mock.md:253`: use SQLite stores for tests | stale/incomplete | Non-SQLite FileConversationStore now exists; [V09-01](../validations/F-MEM-01/V09-01.md) |

## Coverage And Uncertainty

- No new test/build command or dynamic fixture was executed, per the current
  review-stage rule. Primary independently sampled durability/lock, namespace/
  reserved-path, trait/default/ordering, and overflow anchors in V30-01..03;
  executable regressions are deferred, not blockers. Status is `complete`.
- Windows rename replacement, reserved filenames, and case-insensitive path
  behavior were not dynamically exercised. `_meta` collision is proven from the
  exact same path on every platform; broader platform cases remain future work.
- Current EKO construction was sufficient to prove FileStore and
  FileConversationStore reachability, but this task did not establish every
  surface's instance-sharing topology; A-BOOT-01/A-MEM-01 own that application
  lifecycle.
- EmbeddingStore was inspected at its Store/namespace/persistence boundary, not
  for numerical vector-search quality; provider/vector numerical matrices belong
  to a focused follow-up or F-MEM-02 parity.
- SQLite matches were recorded only to prevent false duplicate/deletion claims
  and were not behavior-reviewed.

## Handoff

- Primary should independently re-read the minimal anchors for all three P0s:
  `store.rs:227-278`, the two-instance full-snapshot protocol, and
  `file_conversation.rs:86-92,177-208,457-485`.
- F-MEM-02 must reuse one lossless namespace representation across SQLite and
  non-SQLite implementations; CLI non-use is not a deletion criterion.
- A-STATE-01 should consume FileConversationStore's fail-closed corruption and
  atomic-write positives, plus the `_meta`/ordering findings, without creating an
  application persistence authority.
- A-MEM-01/X-MEM-01 should treat FileStore corruption/shared-handle behavior and
  namespace identity as framework defects, while retaining EKO path/refresh
  policy in the application.
- F-RCT-05 must not assume transcript persistence succeeded merely because
  framework finalization logs and returns; persistence errors are currently
  warnings in `save_transcript_projection`.
- This report becomes stale if either source commit changes any reviewed trait,
  implementation, encoder, persistence primitive, constructor, or call path.
