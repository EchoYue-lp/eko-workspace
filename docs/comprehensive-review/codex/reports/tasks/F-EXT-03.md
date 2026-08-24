# F-EXT-03: Data, research, media, database, and Web tools

> Status: complete
> Reviewer: Codex review subagent
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d35c9a432370b923acabfbb5f32d63`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: both source repositories clean at review start; source remained read-only

## Question

Are the reusable data, research, media, database, RAG, chart, document, and Web
tool contracts truthful about validation, provenance, pagination, numerical and
resource limits, network failures, complete artifacts, and real reachability?

## Scope

- `echo-tools`: feature manifests, public modules, normal/readonly registries.
- `data.rs`, `data_quality.rs`, `statistics.rs`, `database.rs`, `rag.rs`, and
  `chart.rs`.
- `web/`, `media/`, `research/`, plus media-gated Excel/PDF/Word/text/image
  modules where registration and full-output behavior affect this task.
- Root `echo_agent::tools` facade and `ReactAgent` feature-tool assembly.
- EKO Cargo feature selection and file-backed `research_library` adapter only
  for ownership/duplicate classification.

## Out Of Scope

- Generic Tool schema enforcement, collision policy, executor cancellation,
  PageInfo projection, binary projection, and common failure projection; these
  are owned by [F-EXT-01](F-EXT-01.md).
- Shell/file/code/Git tool behavior (`F-EXT-02`), MCP (`F-INT-01`), provider
  LLM wire parsing (`F-LLM-02`/`F-LLM-03`), and EKO research product behavior.
- Source fixes, security exploit testing, live network requests, Cargo/rustc,
  tests, builds, or dynamic fixtures. These were explicitly prohibited during
  this review phase.

## Inputs

- Root `AGENTS.md`; shared `README.md`, `REPORTING.md`, and `TASKS.md`; Codex
  reviewer protocol and report templates.
- [F-EXT-01](F-EXT-01.md), read to consume the generic Tool-contract boundary
  and avoid duplicate findings.
- Current source at the commits above. No other reviewer directory was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Domain tool implementations, reusable scholarly clients, bounded fetch/read primitives, typed domain results, pagination, artifacts, RAG primitives, database implementations including SQLite, and feature registration belong in the framework. A framework API is not dead merely because EKO does not enable it. |
| EKO product policy | Enabled tool set, workspace-scoped research library, review/evidence records, renderers, artifact retention, and provider credentials/configuration belong to EKO. EKO must not enable framework SQLite/database by default, but that is not a reason to remove framework database support. |
| Adapter boundary | EKO's `research_library` persists product-specific source/review artifacts and calls reusable scholarly clients. It must remain a thin translation/policy layer rather than replacing framework search/client contracts. |
| Duplicate search | Searched both repositories for every public type/tool name, research/search/RAG/database concepts, registry calls, feature flags, Store/embedder/artifact/pagination symbols, and live constructors. EKO has a distinct product library, not a duplicate framework executor. Framework research memory is a non-implementation, not an alternative store authority. |
| Migration deletion | Preserve public framework capability options. Remove fake research-memory success or replace both tools with one injected Store-backed implementation; remove write-capable entries from the readonly registry; remove obsolete enhanced-fetch duplicate after one bounded media-capable fetch authority exists. Do not delete database/SQLite or other reasonable framework options due to EKO usage. |

The local-personal-assistant threat model does not justify broad permission
gates for interactive tools. The readonly registry is different: it explicitly
promises physical non-mutation for readonly Subagents, so violating that
declared isolation contract is a correctness defect, not Web-service hardening.

## Current Path

```text
echo_agent / echo_tools feature
  -> echo-tools public cfg module
  -> register_all_tools OR register_readonly_tools
  -> ReactAgent::register_feature_gated_tools
       AgentConfig.enable_tool + readonly_tools
  -> ToolManager -> ReAct tool execution

web/research/media network tool
  -> model parameters -> provider/request -> response body/parser
  -> ToolResult output (+ metadata only for selected tools)
  -> generic ReAct output projection (F-EXT-01 boundary)

database sql_query
  -> read-only lexical check -> AnyPool -> fetch_all full result
  -> String-only row decoding -> in-memory PageRequest
  -> optional page artifact metadata -> ToolResult

RAG
  -> automatically registered tools -> process-global embedder/store
  -> chunk -> sequential embeddings -> global index -> search output
```

Positive evidence includes UTF-8-safe truncation and a hard streamed 10 MiB cap
in core `web_fetch` (`web/fetch.rs:218`), source URLs in Web results, normalized
provider/provider-ID/DOI fields in scholarly clients (`research/clients.rs:19`),
finite-value filtering and safe quantile access in the new exploratory
statistics tool (`statistics.rs:72`), and complete SQL-page artifact persistence
when ToolContext enables it (`database.rs:494`). The findings below are where
other implementations bypass or contradict those patterns.

## Findings

### F-EXT-03-P1-01: The readonly registry contains file writers and a shared-index mutator

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-tools/src/registry.rs:91`,
  `echo-agent/echo-tools/src/registry.rs:98`,
  `echo-agent/echo-tools/src/registry.rs:112`,
  `echo-agent/echo-tools/src/registry.rs:117`,
  `echo-agent/echo-tools/src/excel.rs:282`,
  `echo-agent/echo-tools/src/text.rs:357`,
  `echo-agent/echo-tools/src/rag.rs:205`
- Reachability: readonly ReactAgent construction calls
  `register_readonly_tools`; media registers `excel_to_csv` and `text_export`,
  while RAG registers `rag_index`.
- Expected invariant: the API documented as making readonly Subagents
  physically unable to mutate state contains no file writer or shared-state
  mutator.
- Observed behavior: both export tools create/write output files and declare
  Write permission. `rag_index` mutates the process-global vector store and
  also declares Write.
- Impact: a caller relying on the advertised readonly composition can still
  cause filesystem writes or alter retrieval state. This breaks framework
  isolation and makes readonly Subagent behavior feature-dependent.
- Root cause: registry membership was classified by broad domain category
  comments instead of the implementation's side effects/permissions.
- Direction: derive readonly membership from one explicit capability contract
  and exclude `ExcelToCsvTool`, `TextExportTool`, and `RagIndexTool`; keep
  genuinely read-only readers/searchers. Delete contradictory registry comments
  and add a membership/side-effect invariant test.
- Regression validation: enumerate all registered tools under every feature;
  reject Write/shell/process capabilities; execute representative output paths
  and assert no files or shared index change.
- Validation reports: [V01](../validations/F-EXT-03/V01-01.md),
  [V02](../validations/F-EXT-03/V02-01.md),
  [V03](../validations/F-EXT-03/V03-01.md)

### F-EXT-03-P1-02: Research memory acknowledges persistence while discarding every record

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-tools/src/research/memory.rs:26`,
  `echo-agent/echo-tools/src/research/memory.rs:90`,
  `echo-agent/echo-tools/src/research/memory.rs:96`,
  `echo-agent/echo-tools/src/research/memory.rs:104`,
  `echo-agent/echo-tools/src/research/memory.rs:159`,
  `echo-agent/echo-tools/src/registry.rs:174`
- Reachability: both tools are automatically registered under `research` in
  normal and readonly agents.
- Expected invariant: a successful remember operation makes the record
  available to recall and, per description, across sessions.
- Observed behavior: remember constructs `_entry`, never writes it, then
  returns "stored successfully". Recall ignores its parsed limit and always
  says no findings exist.
- Impact: models and framework consumers can discard research evidence while
  receiving a success terminal, causing silent knowledge loss and false user
  expectations.
- Root cause: placeholder tools were promoted into the public live registry
  without a Store dependency or unsupported terminal.
- Direction: inject one framework `Store`-backed research repository and make
  remember/recall use it, or remove both registrations until implemented.
  Delete the placeholder `_entry`/fixed-message bodies. Do not bind this generic
  API to EKO's product-specific review library or require SQLite.
- Regression validation: write/read, restart durability for FileStore,
  InMemory behavior, limit/search/provenance, concurrent writes, corruption,
  and storage-error terminals.
- Validation reports: [V01](../validations/F-EXT-03/V01-01.md),
  [V04](../validations/F-EXT-03/V04-01.md)

### F-EXT-03-P1-03: Automatically registered RAG tools have no configured embedder path

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-tools/src/registry.rs:274`,
  `echo-agent/echo-tools/src/rag.rs:165`,
  `echo-agent/echo-tools/src/rag.rs:173`,
  `echo-agent/echo-tools/src/rag.rs:178`,
  `echo-agent/echo-tools/src/rag.rs:279`
- Reachability: both registries advertise `rag_index`/`rag_search` whenever the
  `rag` feature is enabled; repository-wide search finds no call to
  `set_rag_embedder` outside its definition.
- Expected invariant: a built-in tool registered as available is constructed
  with required dependencies, or registration reports that the capability is
  unavailable.
- Observed behavior: the first tool execution discovers the unset process-
  global embedder and returns an embedding failure.
- Impact: the primary RAG index/search capability is unusable through default
  framework assembly, yet remains visible to the model and wastes tool turns.
- Root cause: dependency injection is an out-of-band global setter that is not
  part of registration/readiness.
- Direction: construct RAG tools with an `Arc<dyn Embedder>` and scoped store,
  register only a ready instance, and delete the global setter/implicit
  readiness check after migration.
- Regression validation: configured/unconfigured construction, normal/readonly
  registration, provider failure, cancel, and two-agent store isolation.
- Validation reports: [V02](../validations/F-EXT-03/V02-01.md),
  [V05](../validations/F-EXT-03/V05-01.md)

### F-EXT-03-P1-04: A valid four-value IQR analysis panics on index four

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-tools/src/data_quality.rs:232`,
  `echo-agent/echo-tools/src/data_quality.rs:249`,
  `echo-agent/echo-tools/src/data_quality.rs:253`
- Reachability: `OutlierDetectionTool` is automatically registered by the data
  feature; its execution accepts columns with at least four finite values and
  invokes `detect_iqr_outliers` for the default IQR method.
- Expected invariant: every cardinality accepted by the public tool returns a
  result without direct-index panic.
- Observed behavior: for `n = 4`, `3 * n / 4.min(n - 1)` equals 4, then
  `sorted[4]` indexes one past the end.
- Impact: a small valid dataset can panic a tool invocation and violate the
  repository's no-panic rule.
- Root cause: operator precedence applies `.min(n - 1)` to the denominator
  literal rather than to the computed quartile index; direct indexing hides the
  boundary.
- Direction: centralize quantile interpolation with checked access (reuse the
  safe statistics helper or one shared utility) and delete this independent
  quartile formula.
- Regression validation: 0 through 5 values, constant/NaN/infinity/extreme
  values, IQR/z-score parity, and property tests proving indices are bounded.
- Validation reports: [V06](../validations/F-EXT-03/V06-01.md),
  [V10](../validations/F-EXT-03/V10-01.md)

### F-EXT-03-P1-05: SQL pagination fetches the entire backend result before applying its limit

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-tools/src/database.rs:74`,
  `echo-agent/echo-tools/src/database.rs:107`,
  `echo-agent/echo-tools/src/database.rs:172`,
  `echo-agent/echo-tools/src/database.rs:423`,
  `echo-agent/echo-tools/src/database.rs:447`
- Reachability: `sql_query` is registered in normal and readonly database
  feature paths; every accepted query enters `execute_readonly_query` before
  in-memory PageRequest pagination.
- Expected invariant: a page limit of at most 100 bounds database reads and
  process memory, while a cursor continues through a stable query snapshot.
- Observed behavior: both SQLite and server branches call `fetch_all` with the
  caller's unmodified query; only after all rows are materialized does the tool
  select a page.
- Impact: a query returning millions of rows can consume unbounded backend,
  network, and memory resources despite the advertised hard page maximum; each
  continuation repeats the full query.
- Root cause: pagination is an output slicing helper rather than a database
  query/stream protocol.
- Direction: stream and stop at a bounded window or implement dialect-aware
  keyset/offset paging with a stable identity; preserve complete results only
  through an explicit bounded export path. Delete fetch-all paging once the
  database iterator is authoritative.
- Regression validation: very large result, cursor pages, changed database,
  slow stream, cancel/timeout, pool cleanup, and dialect matrix.
- Validation reports: [V07](../validations/F-EXT-03/V07-01.md)

### F-EXT-03-P1-06: Database formatting silently replaces non-string cells with `"?"`

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-tools/src/database.rs:463`,
  `echo-agent/echo-tools/src/database.rs:475`,
  `echo-agent/echo-tools/src/database.rs:478`,
  `echo-agent/echo-tools/src/database.rs:480`
- Reachability: every query/list/describe result is converted through
  `format_db_rows` before result or artifact construction.
- Expected invariant: structured SQL results preserve NULL/string/integer/
  real/boolean/binary values or return an explicit unsupported-type error with
  column identity.
- Observed behavior: the formatter only attempts `Option<String>` and
  `String`; all decode failures become the indistinguishable literal `"?"`.
- Impact: valid analytical values can be silently corrupted in visible output
  and saved artifacts, invalidating downstream calculations and citations.
- Root cause: a heterogeneous `AnyRow` was flattened through one string decoder
  with a success fallback instead of a typed conversion table.
- Direction: decode by `AnyTypeInfo` into JSON scalar/binary envelopes, retain
  SQL type facts, and fail or mark individual unsupported cells explicitly.
  Delete the `"?"` sentinel.
- Regression validation: each supported scalar, NULL, blob, decimal, date/time,
  JSON, unsupported type, and all three database drivers.
- Validation reports: [V07](../validations/F-EXT-03/V07-01.md),
  [V10](../validations/F-EXT-03/V10-01.md)

### F-EXT-03-P1-07: Specialized network tools read unbounded bodies and discard complete truncated payloads

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-tools/src/web/fetch.rs:237`,
  `echo-agent/echo-tools/src/media/image_fetch.rs:177`,
  `echo-agent/echo-tools/src/media/image_fetch.rs:185`,
  `echo-agent/echo-tools/src/media/image_fetch.rs:264`,
  `echo-agent/echo-tools/src/media/image_fetch.rs:287`,
  `echo-agent/echo-tools/src/media/web_fetch_enhanced.rs:315`,
  `echo-agent/echo-tools/src/media/web_fetch_enhanced.rs:369`,
  `echo-agent/echo-tools/src/research/pdf_fetch.rs:85`,
  `echo-agent/echo-tools/src/research/clients.rs:716`
- Reachability: image fetch, enhanced fetch, research PDF fetch, and scholarly
  clients are public; the first three are automatically registered under media
  or research. Core `web_fetch` demonstrates the intended streamed hard cap and
  artifact path but does not cover them.
- Expected invariant: limits bound bytes while receiving chunked/unknown-length
  bodies, arithmetic is checked, and every successful truncation retains a
  model-readable complete artifact.
- Observed behavior: these implementations use `.bytes()` or `.text()` before
  post-read checks/truncation. `image_fetch` computes model-controlled
  `max_size_mb * 1024 * 1024` without checked arithmetic. Image outputs expose
  only a 200/1000-character data-URI prefix; enhanced text and research PDF
  outputs truncate without ToolContext artifact persistence.
- Impact: a remote response can allocate far beyond configured output limits;
  after success, the model cannot recover the complete image/document. Huge
  size parameters may overflow in debug or wrap in release.
- Root cause: multiple fetch implementations predate or bypass the shared
  streamed/artifact pattern, and the enhanced fetch remains a registered
  parallel authority.
- Direction: factor one cancellable streamed reader with a hard byte cap and
  typed response errors; make media/PDF clients use it and persist complete
  payloads with a bounded preview. Replace/delete the duplicate enhanced fetch
  after capability parity. Use checked conversions and multiplication.
- Regression validation: chunked oversized, false/missing Content-Length,
  decompression expansion, maximum integer, timeout/cancel, redirect, artifact
  write failure, binary MIME, and full-artifact hash/readback.
- Validation reports: [V08](../validations/F-EXT-03/V08-01.md),
  [V10](../validations/F-EXT-03/V10-01.md)

### F-EXT-03-P2-08: RAG's advertised character overlap is ignored and its store is process-global

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-tools/src/rag.rs:33`,
  `echo-agent/echo-tools/src/rag.rs:71`,
  `echo-agent/echo-tools/src/rag.rs:94`,
  `echo-agent/echo-tools/src/rag.rs:105`,
  `echo-agent/echo-tools/src/rag.rs:134`,
  `echo-agent/echo-tools/src/rag.rs:238`
- Reachability: the same registered RAG index/chunk/search tools use these
  helpers and globals for every ReactAgent in the process.
- Expected invariant: `chunk_size`/`overlap` count Unicode characters as
  described, overlap affects adjacent chunks, and an injected owner scopes
  indexed documents to the intended agent/consumer.
- Observed behavior: paragraph and sentence thresholds use UTF-8 byte length;
  `_overlap` is unused; the store is a process-wide OnceLock with silent oldest
  eviction at 10,000 chunks.
- Impact: non-ASCII chunks are smaller than requested, context continuity is
  absent, and unrelated agents can retrieve or evict one another's local
  indexed material.
- Root cause: a prototype global store/chunker was exposed as a configurable
  reusable service.
- Direction: inject a scoped vector-store service, count characters, implement
  overlap with progress guarantees, and expose eviction facts. Delete global
  state after callers migrate.
- Regression validation: Chinese/emoji boundaries, zero/greater-than-size
  overlap, no punctuation, huge values, two agents, eviction/provenance.
- Validation reports: [V05](../validations/F-EXT-03/V05-01.md)

### F-EXT-03-P2-09: The root tools facade omits six enabled domain capability families

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-tools/src/lib.rs:34`,
  `echo-agent/echo-tools/src/lib.rs:43`,
  `echo-agent/src/tools/mod.rs:83`,
  `echo-agent/src/tools/mod.rs:89`,
  `echo-agent/src/tools/mod.rs:95`
- Reachability: root Cargo features forward to echo_tools and ReactAgent
  auto-registers the tools, so root `echo_agent` consumers can execute them.
- Expected invariant: a root feature exposes its public capability through the
  documented root `tools` facade consistently, or explicitly documents that
  consumers must depend on `echo_tools` directly.
- Observed behavior: root facade modules exist only for web, media, and
  research; data, data_quality, statistics, database, rag, and chart are absent
  despite root features and automatic registration.
- Impact: consumers can enable and receive tools they cannot name/import through
  the root crate's public tools namespace, producing an inconsistent framework
  API and pressure for duplicate adapters.
- Root cause: facade exports were added family-by-family without a feature/API
  matrix gate.
- Direction: re-export all public root feature families consistently (or make
  the split an explicit documented API), with compile/import tests. Do not
  remove reasonable echo_tools APIs.
- Regression validation: root import per feature, no-default/full, docs links,
  examples, and registration/export name parity.
- Validation reports: [V01](../validations/F-EXT-03/V01-01.md)

### F-EXT-03-P2-10: Research search tools expose only the first page and classify network failures inconsistently

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-tools/src/research/arxiv.rs:111`,
  `echo-agent/echo-tools/src/research/arxiv.rs:128`,
  `echo-agent/echo-tools/src/research/semantic_scholar.rs:85`,
  `echo-agent/echo-tools/src/research/pubmed.rs:94`,
  `echo-agent/echo-tools/src/research/clinical_trials.rs:80`,
  `echo-agent/echo-tools/src/research/clients.rs:39`,
  `echo-agent/echo-tools/src/web/search.rs:177`
- Reachability: four legacy research tools are automatically registered;
  reusable OpenAlex/Crossref/Europe PMC clients are exported to consumers.
- Expected invariant: if a provider reports more results than returned, the
  result exposes a continuation input/token; HTTP/rate-limit/server failures use
  consistent typed terminal facts without parsing an error as empty success.
- Observed behavior: tools cap at 100 and hardcode page zero/no offset, returning
  total counts without a cursor. Reusable `ScholarlySearchPage` has total and
  works only. ArXiv does not check response status before parsing. Other legacy
  tools surface untyped execution errors; only web_search attaches a retryable
  transient ToolFailure.
- Impact: evidence beyond the first provider page is unreachable through the
  public contract, and orchestration cannot consistently distinguish malformed
  input, authentication, rate limit, transient server error, and empty result.
- Root cause: each provider owns an ad hoc one-shot API contract rather than a
  shared source/provenance/page/failure envelope.
- Direction: define one model-visible scholarly page contract with provider,
  stable IDs/URLs, continuation, and typed status facts; adapt providers without
  discarding provenance. Keep the generic PageInfo projection fix owned by
  F-EXT-01.
- Regression validation: two or more pages for each provider, stable citation
  fields, malformed bodies, and 400/401/404/429/500/timeout cases.
- Validation reports: [V09](../validations/F-EXT-03/V09-01.md),
  [V10](../validations/F-EXT-03/V10-01.md),
  [V11](../validations/F-EXT-03/V11-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition/export/feature/duplicate/layer map | yes | passed | [V01](../validations/F-EXT-03/V01-01.md) |
| V02 | Registration and real ReactAgent reachability | yes | passed | [V02](../validations/F-EXT-03/V02-01.md) |
| V03 | Readonly registry side-effect invariant | yes | failed | [V03](../validations/F-EXT-03/V03-01.md) |
| V04 | Research memory persistence/recall trace | yes | failed | [V04](../validations/F-EXT-03/V04-01.md) |
| V05 | RAG readiness/chunk/provenance/ownership trace | yes | failed | [V05](../validations/F-EXT-03/V05-01.md) |
| V06 | Data/statistics numeric/panic/UTF-8/overflow inspection | yes | failed | [V06](../validations/F-EXT-03/V06-01.md) |
| V07 | Database pagination/type/artifact/resource inspection | yes | failed | [V07](../validations/F-EXT-03/V07-01.md) |
| V08 | Network timeout/body/cancel/artifact/resource inspection | yes | failed | [V08](../validations/F-EXT-03/V08-01.md) |
| V09 | Provenance/citation/pagination/typed-error inspection | yes | failed | [V09](../validations/F-EXT-03/V09-01.md) |
| V10 | Existing test inventory and future dynamic matrix | yes | passed | [V10](../validations/F-EXT-03/V10-01.md) |
| V11 | F-EXT-01 ownership and historical drift comparison | yes | passed | [V11](../validations/F-EXT-03/V11-01.md) |
| V12 | Exact headers/links/path isolation/source-clean integrity | yes | passed | [V12](../validations/F-EXT-03/V12-01.md) |
| V30 | Primary source-anchor sampling and acceptance | yes | passed | [V30](../validations/F-EXT-03/V30-01.md) |

No executable V04-style command report was created: the user explicitly
prohibited Cargo/rustc/test/build and dynamic fixtures. Primary source sampling
in V30 accepted the source-conclusive findings without runtime execution.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| F-EXT-01: schemas/validation are disconnected | current, dependency-owned | Specialized tools continue local parsing; no duplicate finding here. |
| F-EXT-01: generic executor cancellation is not consumed | current, dependency-owned | Domain tools also do not consume tokens; generic root cause remains F-EXT-01. |
| F-EXT-01: PageInfo cursor is invisible | current, dependency-owned | `web_search`/SQL still use PageInfo; F-EXT-03 reports only absent domain pagination contracts. |
| F-EXT-01: public binary output projects empty | current, dependency-owned | Media tools currently avoid binary ToolResult by returning lossy prefixes; complete-payload defect is specialized here. |
| `web_fetch` large content is recoverable | current positive exception | `web/fetch.rs:237-317`; other network tools do not share it. |

## Coverage And Uncertainty

- All scoped source families, registries, feature manifests, public exports,
  primary implementations, and existing test declarations were statically
  inspected. Deep formula correctness across all 3,751 lines of `data.rs` and
  every document parser format was sampled by risk anchors rather than proved
  exhaustively.
- No Cargo/test/fixture/network command was run. Compilation, platform-specific
  sqlx/calamine/lopdf behavior, actual provider payloads, cancellation timing,
  memory peaks, and artifact readback remain future evidence.
- Root status was captured clean. Generated EKO files mentioned outside this
  task were neither read as evidence nor modified.
- No finding recommends deleting a framework option because EKO does not use
  it. Framework database/SQLite and multiple domain implementations remain
  legitimate reusable capabilities.

## Handoff

- Primary review independently sampled registry membership, derived the
  four-value IQR index, confirmed no `set_rag_embedder` caller, and inspected SQL
  `fetch_all`/String-only conversion in V30.
- Implementation planning should read [F-EXT-01](F-EXT-01.md) first so generic
  schema/cancellation/pagination/artifact authorities are fixed once rather
  than copied into each domain tool.
- `F-EXT-03-P1-01`, P1-02, P1-03, and P1-04 are small deterministic first
  fixes; P1-05 through P1-07 require shared database/network result designs.
- This report becomes stale if registry membership, RAG construction, domain
  fetch helpers, database formatting/paging, research Store wiring, root facade
  exports, or the reviewed commits change.
- Future evidence belongs in implementation/Q tasks; no source fix is included
  here.
