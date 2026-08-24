# F-EXT-03: Data, research, media, database, and Web tools

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean (both repositories)

## Question

Are domain tool contracts honest about validation, provenance, pagination,
numerical limits, network failures, and artifact output?

## Scope

`echo-tools` modules: `data.rs` (15 tools + shared polars helpers),
`data_quality.rs` (3 tools), `statistics.rs` (1 tool), `chart.rs` (1 tool),
`database.rs` (3 tools), `rag.rs` (3 tools), `web/` (fetch, search, extract +
brave/duckduckgo/tavily/utils providers), `research/` (arxiv, pubmed,
semantic_scholar, clinical_trials, pdf_fetch, bibtex, memory, clients),
`media/` (image_fetch, web_fetch_enhanced), `image.rs`, `pdf.rs`, `word.rs`,
`text.rs`, `excel.rs`, `registry.rs` (registration surface), `lib.rs`
(feature map), plus `echo-agent/src/agent/react/mod.rs:741-747`
(readonly/all registration entry) and `security.rs` ResourceLimits usage.

## Out Of Scope

- Shell/files/code/git/worktree tools → F-EXT-02 (complete).
- Tool trait/registry/pagination/artifact-writer infrastructure → F-EXT-01
  (re-read; only tool-side compliance checked here).
- PathValidator/SSRF internals → F-SEC-01 (re-read; only usage checked).
- EKO application analysis/research connectors and workbench → A-DOM-01
  (app-side policy); app-side tool policy lists (`tool_exposure.rs`,
  `chat_driver.rs`) cross-referenced only for reachability.
- Live-network provider behavior (arxiv/pubmed/openalex/tavily/brave) —
  not_run, see Validation Matrix.

## Inputs

- Root `AGENTS.md`, shared `README.md`, `REPORTING.md`, `TASKS.md`
  (F-EXT-03 card), `zcode-ds/README.md`.
- Dependency reports read: zcode-ds `F-EXT-01` (complete — contract,
  pagination, artifact-writer, WRITE_TOOLS drift), `F-EXT-02` (complete —
  scope boundary), `F-SEC-01` (complete — validator/limits semantics),
  `F-REL-01` (complete — retry classification).
- Historical documents treated as hypotheses: root `docs/MASTER-PLAN.md`
  (statistics split claims), `echo-tools/README.md`, `docs/PROJECT-ANALYSIS.md`,
  `docs/2026-07-12-echo-agent-peer-framework-iteration.md`,
  `docs/deep-iteration-plan.md`.

## Layering Decision

- Generic mechanism (framework, `echo_tools`): all domain tools reviewed
  (data/statistics/research/media/web/database/rag/chart/document), the
  readonly-vs-all registration surface, `ResearchClients` (OpenAlex/
  Crossref/Europe PMC/Zotero normalized `ScholarlyWork` contract) — correctly
  placed; these are reusable domain capabilities.
- EKO product policy (application): no new policy inside scope; EKO consumes
  the tools via `register_all_tools`/`register_readonly_tools` with the
  feature set from `echo-agent-cli/echo-agent-app-core/Cargo.toml:10-14`.
- Adapter boundary: none new inside scope.
- Duplicate search terms (both repositories): `register_all_tools` /
  `register_readonly_tools` callers; `web_fetch` / `WebFetchToolEnhanced` /
  `ImageFetchTool`; `parse_page_range`; `research_remember` /
  `research_recall` vs `remember`/`recall`/`search_memory`; `rag_index` /
  `rag_search`; `generate_chart`; `sql_query` / `list_tables` /
  `describe_table`; `write_excel` / `excel_load` / `bibtex_generate`; tool-name
  collision scan inside echo-tools. Results: no same-name collisions; four
  parallel URL-download tools (web_fetch, web_fetch_enhanced, image_fetch,
  pdf_fetch) with divergent size caps; two `parse_page_range` parsers with
  divergent limits; `research_*` memory overlaps the framework memory family
  by name only (no collision). Full inventory: [V01-01](validations/F-EXT-03/V01-01.md).

## Current Path

Verified data flow per family:

1. **Registration/reachability**: `AgentConfig.readonly_tools` →
   `register_feature_gated_tools` (`echo-agent/src/agent/react/mod.rs:741-747`)
   → `register_all_tools` (full agents) or `register_readonly_tools`
   (read-only Subagents, built at
   `echo-agent-cli/echo-agent-app-core/src/infra.rs:639-640,772-773`,
   `plugin_components.rs:530`). EKO enables `web,data,statistics,chart,
   research,media,rag` (+shell/files/git); `database` is a framework-only
   option (not enabled by the CLI). [V02-01](validations/F-EXT-03/V02-01.md)
2. **Data/statistics**: `read_data`/`filter_data`/… → `SecurityConfig::
   validate_file` (absolute path, denied list, allowed roots, max_file_size
   check at `security.rs:248-260`) → polars load → standardized envelope
   (`data_tool_response`, `data.rs:135-161`: tool/rows/columns/truncated/data)
   → `df_to_json` (`NaN/inf` → `null`, `data.rs:2840-2845`).
3. **Web**: `web_fetch` (`web/fetch.rs`) — SSRF-safe pinned-IP GET, 10 MB body
   cap, non-UTF-8 lossy fallback, char-safe truncation, artifact spill for
   oversized content; `web_search` — `PageRequest` pagination
   (limit enforced 1..=10, cursor fingerprint-bound), provider failure →
   `ToolFailure::Transient` retryable; providers (ddg/brave/tavily) — 15 s
   timeouts, error classification, no key logging (`tavily.rs:64-73`).
4. **Research**: tool layer (arxiv/pubmed/semantic_scholar/clinical_trials)
   — URL-encoded queries, `max_results` clamped to 100, structured output
   with query echo and per-result IDs; clients (`clients.rs`) — normalized
   `ScholarlyWork` (provider/provider_id provenance fields), clamps, timeouts,
   HTTP/JSON error classification.
5. **Database**: `sql_query` — scheme whitelist (`validate_db_url`,
   `database.rs:352-388`), read-only keyword filter + `SET TRANSACTION READ
   ONLY` for non-SQLite, `PageRequest` pagination (1..=100), artifact spill of
   the full page (`database.rs:494-537`); SQLite gets no DB-enforced read-only
   layer (documented trade-off, registry.rs:77).
6. **RAG**: `rag_index` → global in-memory `VectorStore` (OnceLock static,
   MAX_CHUNKS=10 000, silent oldest-eviction), embedding API key required;
   `rag_search` → cosine search with `top_k` unbounded.
7. **Media**: pdf/word/text/excel tools — `validate_file` bounded reads
   (50 MB default), page/char preview limits, UTF-8-safe truncation
   (`text.rs`, `pdf.rs:446-452`, `word.rs`); `image_fetch`/`web_fetch_enhanced`
   — SSRF-safe downloads, but no body-size cap.

## Findings

### F-EXT-03-P1-01: `research_remember`/`research_recall` are non-persistent stubs that fabricate success and silently discard user findings

- Priority: P1
- Confidence: high (static chain fully verified)
- Layer: framework
- Evidence: `echo-tools/src/research/memory.rs:66-121` — `_entry`
  (JSON with topic/findings/papers/tags/timestamp) built at `:96-102` and
  never stored; success message "Research findings stored successfully" at
  `:106-119`; comments `:90,104-105` ("In production, this would persist to
  SQLite" / "For now, return success message"). `research_recall`
  `memory.rs:159-180` always returns "No research findings found" at
  `:172-179`; `limit` parsed and discarded (`_limit`, `:166-169`).
- Reachability: registered in BOTH `register_all_tools` and
  `register_readonly_tools` (`registry.rs:174-175,398-399`); `research`
  feature enabled by the CLI (`echo-agent-cli/echo-agent-app-core/Cargo.toml:
  10-14`); no other consumer or store in either repository (grep
  `research_remember|research_recall` in CLI: zero hits).
- Expected invariant: a tool named "Store research findings … build a
  persistent knowledge base across research sessions" (description
  `memory.rs:26-28,137-140`) must persist what it accepts and be able to
  return it.
- Observed behavior: the remember tool discards the submitted content and
  reports success; the recall tool always returns "no findings". Findings are
  permanently lost while the model is told they were stored.
- Impact: silent data loss of user/research content plus a fabricated success
  that poisons downstream agent reasoning ("I already stored this, recall
  later"). The description promises a capability that does not exist; the
  tool also sits on the read-only Subagent surface as a Write-permission tool
  (see P1-02).
- Root cause: scaffolding written as a placeholder ("In production…") was
  registered and shipped as if functional; no persistence layer exists in the
  framework for it, and the SQLite comment contradicts AGENTS.md
  (echo-agent-cli is file/memory-based).
- Direction: either implement persistence on the framework Store/FileStore
  (the framework has `echo_core::memory` stores) or remove the two tools from
  both registries until implemented; never report success without storing.
  Delete the stale "SQLite" comment. Follow the AGENTS.md rule: no
  dual systems — reuse the existing `Store`/memory path rather than a new
  storage authority.
- Regression validation: call `research_remember` then `research_recall` and
  assert the recalled content matches; assert the tool does not return
  success when persistence fails; registry test asserting both tools are
  absent (if removed).
- Validation reports: [V03-04](validations/F-EXT-03/V03-04.md),
  [V02-01](validations/F-EXT-03/V02-01.md)

### F-EXT-03-P1-02: Read-only Subagent surface registers Write-permission tools — `bibtex_generate` writes arbitrary unvalidated paths, `rag_index` mutates shared state

- Priority: P1
- Confidence: high (static chain fully verified)
- Layer: framework (registry classification) with application impact
- Evidence: `echo-tools/src/registry.rs:18-19` (doc: readonly subset makes
  Subagents "physically incapable of mutating state"); rag block
  `registry.rs:55-62` registers `RagIndexTool` whose
  `permissions() == [Read, Write]` (`rag.rs:212-214`) and whose execute
  mutates the process-global vector store (`rag.rs:305-306`) and may call the
  embedding API with `EMBEDDING_API_KEY` (description `rag.rs:216-220`);
  research block `registry.rs:161-176` registers `BibtexGenerateTool` whose
  `permissions() == [Write]` (`bibtex.rs:21-23`) and whose execute writes
  `output_file` via `tokio::fs::write(path, …)` (`bibtex.rs:94-101`) with NO
  `SecurityConfig::validate_output_file`, no absolute-path check, no allowed
  roots — an arbitrary-path file write. `SqlQueryTool` is also in the readonly
  subset (`registry.rs:77`, explicitly documented risk trade-off — recorded,
  not a finding).
- Reachability: `register_readonly_tools` is the builder path for EKO
  read-only Subagents (`react/mod.rs:741-747`; `infra.rs:639-640,772-773`;
  `plugin_components.rs:530`), which the code comments promise are
  "physically no-write" (`infra.rs:615`); both features are CLI-enabled.
- Expected invariant: every tool on the read-only surface is non-mutating;
  a read-only Subagent cannot create or modify files or shared state.
- Observed behavior: a read-only Subagent can call `bibtex_generate` with
  `output_file` set to any path (e.g., `~/.zshrc`, a project file) and the
  framework writes it, and can call `rag_index` to mutate the shared global
  index and consume the user's embedding API quota.
- Impact: the physical no-write guarantee of read-only Subagents/reviewers is
  broken; arbitrary file writes from a supposedly read-only surface, plus
  silent shared-state mutation. This is a product-invariant violation on the
  exact surface F-EXT-01's plan-mode filter (P2-01) is supposed to back up.
- Root cause: the readonly subset is an ad-hoc name list maintained by hand
  (`registry.rs`) that was never checked against `Tool::permissions()` /
  `risk_level()`; tools added later (bibtex, rag) were classified by their
  primary read capability.
- Direction: derive the readonly subset from the per-tool write
  classification (`Tool::permissions()`/`risk_level()`) at registration time
  (single authority, see F-EXT-01-P2-01), or explicitly exclude
  `BibtexGenerateTool`, `RagIndexTool` (and review `SqlQueryTool`) from
  `register_readonly_tools`; add a registry test asserting no registered
  readonly tool declares `Write`/non-ReadOnly risk. `bibtex_generate` must
  also route `output_file` through `validate_output_file` regardless.
- Regression validation: unit test collecting names from
  `register_readonly_tools` and asserting every tool's `permissions()` is
  read-only; a readonly-Subagent fixture calling `bibtex_generate` with an
  output path and asserting the file is not created.
- Validation reports: [V03-03](validations/F-EXT-03/V03-03.md),
  [V02-01](validations/F-EXT-03/V02-01.md)

### F-EXT-03-P1-03: `outlier_detection` (IQR) panics on a numeric column with exactly 4 values — quantile index out of bounds

- Priority: P1
- Confidence: high (index arithmetic replicated and confirmed)
- Layer: framework
- Evidence: `echo-tools/src/data_quality.rs:249-255`
  (`let q1 = sorted[n / 4.min(n - 1)]; let q3 = sorted[3 * n / 4.min(n - 1)];`)
  with caller guard `if values.len() < 4 { error }` at `:232-235`. Operator
  precedence makes this `sorted[(3 * n) / 4.min(n - 1)]`; for `n == 4` the
  index is `12 / 3 == 4` on a 4-element vector → index-out-of-bounds panic
  (debug AND release; Vec indexing always panics). Arithmetic replication for
  `n = 4..=20` shows the OOB only at `n == 4` ([V03-01](validations/F-EXT-03/V03-01.md)).
- Reachability: `OutlierDetectionTool` registered (`registry.rs:150-152`) under
  `data` feature, CLI-enabled; trigger: `outlier_detection(data_path=…,
  method="iqr")` on a file whose numeric column has exactly 4 finite values.
- Expected invariant: no tool panics on valid, documented input (AGENTS.md
  panic rule); IQR quartiles must be computed with in-bounds indices for the
  whole accepted domain `n >= 4`.
- Observed behavior: `sorted[4]` panics for `n == 4`; for `n == 5..` the same
  formula computes quartiles that are statistically wrong (e.g., n=5 →
  q3 = sorted[3], the 60th percentile) — the formula is also incorrect for
  small n even where it does not crash.
- Impact: tool crash (and likely the enclosing run task, since no catch_unwind
  barrier was found in the F-EXT-01/02 tool paths) on a perfectly ordinary
  small dataset; the existing test only covers 9 values
  (`data_quality.rs:563-579`).
- Root cause: `4.min(n - 1)` was presumably meant as `(4).min(n)` guard or a
  plain `n / 4`, and `3 * n / …` was never bounded; the min-guard only works
  for the first quantile.
- Direction: compute quantile indices with a correct, bounds-checked helper
  (e.g., `(3 * n).div_ceil(4)` clamped to `n - 1`, or use
  `polars::Series::quantile`), and add a test with a 4-value column.
- Regression validation: unit test running `detect_iqr_outliers` with
  `values = [1.0, 2.0, 3.0, 100.0]` asserting a successful result with
  bounded q1/q3; same fixture through `OutlierDetectionTool::execute`.
- Validation reports: [V03-01](validations/F-EXT-03/V03-01.md),
  [V04-02](validations/F-EXT-03/V04-02.md)

### F-EXT-03-P2-01: Four parallel URL-download tools with divergent safety contracts — `web_fetch_enhanced`, `image_fetch`, and `pdf_fetch` have no body-size cap and no artifact spill

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `web_fetch` caps the response at 10 MB and spills oversized
  content to an artifact (`web/fetch.rs:25`, `:225-260`, `:279-319`);
  `web_fetch_enhanced` reads the full body with `response.text()` and full
  images with `response.bytes()` — no Content-Length check, no byte cap, no
  artifact spill (`media/web_fetch_enhanced.rs:315-320,369-374`; image branch
  `:252-343`), truncating only the displayed text (`:392`); `image_fetch`
  checks size only AFTER downloading (`media/image_fetch.rs:243-279`,
  `max_size_mb` default 10 — the download itself is unbounded); `pdf_fetch`
  downloads the full body with no cap at all
  (`research/pdf_fetch.rs:85-91`). Duplicate-semantic marker:
  `web_fetch_enhanced.rs:37` `// TODO(v0.3): replace WebFetchTool with this
  enhanced version` — yet BOTH are registered and enabled (`registry.rs:299-302,
  316-320`; media feature CLI-enabled).
- Reachability: `web_fetch_enhanced`/`image_fetch` registered for every
  media-feature agent (registry.rs:317-320); `pdf_fetch` registered in both
  full and readonly registries (`registry.rs:172`); all CLI-enabled.
- Expected invariant: one semantic (fetch a URL body) with one bounded
  implementation; every network-download tool enforces a body-size cap and
  spills oversized output to the artifact writer (F-EXT-01 contract).
- Observed behavior: three of the four download tools can pull unbounded
  bodies into memory (a huge page, a 2 GB image → base64 of ~2.7 GB in RAM),
  and `web_fetch_enhanced` loses content instead of spilling it. The
  "enhanced" duplicate is LESS safe than the tool it is marked to replace,
  while both remain live.
- Impact: memory exhaustion / OOM from a network response (framework-bug class
  per AGENTS.md), inconsistent large-content handling, and two LLM-visible
  tools (`web_fetch` vs `web_fetch_enhanced`) with divergent semantics that
  the model cannot distinguish.
- Root cause: parallel implementation history (media pipeline TODO) never
  converged; size limits were added to `web_fetch` only and never propagated
  to its successors.
- Direction: converge on ONE web-fetch tool (per AGENTS.md duplicate rule):
  keep the bounded `web_fetch` semantics (body cap + spill), delete or
  fold `web_fetch_enhanced`; add byte caps to `image_fetch` (stream-limited
  download) and `pdf_fetch` (Content-Length + streamed cap), and spill
  oversized extracted text to artifacts like `sql_query`/`web_fetch`.
- Regression validation: mocked-response tests asserting body-cap rejection
  for each download tool; a large-image fixture asserting no unbounded
  allocation; a registry test asserting only one `web_fetch*` name is
  registered.
- Validation reports: [V01-01](validations/F-EXT-03/V01-01.md),
  [V04-01](validations/F-EXT-03/V04-01.md)

### F-EXT-03-P2-02: Schema-declared enum parameters are not enforced — tools silently fall back and then echo the requested value as the computed one

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `correlate_data.method` — schema enum `["pearson","spearman"]`
  (`data.rs:3221`), execute `unwrap_or("pearson")` with only `"spearman"`
  special-cased (`data.rs:3240-3243,3270`), result reports the REQUESTED
  method (`data.rs:3362-3364`) even when pearson was actually computed;
  `pivot_data.agg_function` — schema enum
  `["sum","mean","count","min","max","first","last"]` (`data.rs:3406-3408`),
  execute `match … { … _ => sum() }` (`data.rs:3494-3502`) and reports the
  requested function (`data.rs:3624`); `outlier_detection.method` — schema
  enum (`data_quality.rs:162`), execute falls back to zscore for anything not
  `"iqr"` (`data_quality.rs:177-180,236-240`) and echoes the requested method
  (`data_quality.rs:243`); `generate_chart.chart_type`/`output_format` — schema
  enums (`chart.rs:35-39,69-73`), execute defaults unknown types to line
  (`chart.rs:169-221`) and any non-"html" format to json (`chart.rs:117-121,
  133-137`); `web_fetch_enhanced.mode` — schema enum (`web_fetch_enhanced.rs:
  204-208`), any non-"image"/"json" value treated as text.
- Reachability: all tools registered and CLI-enabled; an LLM passing
  `"kendall"`, `"median"`, or `"zscore+"` gets a result labeled with that
  requested value computed by a different method.
- Expected invariant: schema enum = enforced contract; the result's method/
  function field states what was actually computed.
- Observed behavior: unknown values are silently mapped to a default while
  the result object labels the requested (never-executed) value.
- Impact: statistically wrong analyses presented with false labels — an agent
  can report "Kendall τ = 0.82" when pearson was computed; violates the
  honesty contract this task targets (validation + result schema).
- Root cause: hand-written `match`/`unwrap_or` fallbacks without parameter
  validation; no shared validation helper between the macro-schema
  (`#[tool]`) and execute bodies.
- Direction: validate enum params against the schema at execute entry
  (return `ToolError::InvalidParameter`), or use the framework's
  schema-driven parameter deserialization (F-EXT-01 contract) so declared
  enums are enforced once; never echo an unexecuted value as the executed one.
- Regression validation: per-tool tests passing an out-of-enum value and
  asserting `InvalidParameter` (or at minimum that the result reports the
  fallback actually used).
- Validation reports: [V03-02](validations/F-EXT-03/V03-02.md),
  [V04-02](validations/F-EXT-03/V04-02.md)

### F-EXT-03-P2-03: `analyze_image` description promises "detailed description and analysis of the image" but the tool performs no analysis

- Priority: P2
- Confidence: high (description vs code)
- Layer: framework
- Evidence: description `image.rs:32-34` ("Analyze image content, describing
  the information in the image … Returns a detailed description and analysis
  of the image"); execute `image.rs:57-106` only loads/encodes the image and
  returns metadata (MIME type, base64 size, estimated raw size) plus a note
  that the image "cannot be displayed as text to the LLM" (`:101-105`); no
  vision/LLM call exists anywhere in the tool.
- Reachability: registered for all media-feature agents (`registry.rs:316`),
  CLI-enabled; an LLM selecting the tool from its description gets a success
  result with no analysis.
- Expected invariant: a tool's description (the LLM-facing contract) must
  describe the capability the tool actually delivers.
- Observed behavior: tool reports success ("Image successfully loaded") while
  delivering no analysis, contradicting its description.
- Impact: wasted turns and misleading agent reasoning (agent believes the
  image was analyzed); the model may surface the note as an answer that
  analysis is impossible, but the tool selected itself with a false premise.
- Root cause: description written for an aspirational multimodal pipeline;
  the actual implementation was reduced to a loader and the description was
  never updated.
- Direction: rewrite the description to state the true capability (load/verify
  image and report metadata), or implement real multimodal analysis via the
  framework's vision-capable LLM path; add a description-to-behavior test.
- Regression validation: none needed beyond a doc/description update and a
  unit test asserting the output contains no "analysis" fields while the
  description claims none.
- Validation reports: [V04-02](validations/F-EXT-03/V04-02.md)

### F-EXT-03-P3-01: Non-finite statistics silently serialize as `null` — overflowed mean/std/skew/correlations are indistinguishable from missing data

- Priority: P3
- Confidence: high (behavior reproduced)
- Layer: framework
- Evidence: `df_to_json`/`any_value_to_json` map non-finite floats to
  `Value::Null` (`data.rs:2840-2845`); `DataStatsTool` computes
  mean/variance/stddev via `(x - mean).powi(2)` sums that overflow to `inf`
  (`data.rs:668-674`) and serializes with `json!` — reproduced: serde_json
  renders NaN/Inf as `null` ([V03-02](validations/F-EXT-03/V03-02.md));
  `statistics.rs:98-115` same pattern (values are finite-filtered, but
  moment arithmetic `powi(3)/powi(4)` overflows); `CorrelateTool` explicitly
  maps NaN to `null` (`data.rs:3348-3353`) while `format_smart_float`
  (`data.rs:2324-2344`, ratio tool) handles NaN/Inf as strings — inconsistent
  handling inside the same module family. `statistics.rs:86-90` labels the
  count of NaN entries as `missing_or_non_finite_count`, conflating
  non-finite with missing.
- Reachability: any data file with extreme magnitudes (e.g., 1e200 values,
  or `inf` in a CSV) through `data_stats`, `exploratory_statistics`,
  `correlate_data`, `ratio_data`.
- Expected invariant: a computed statistic must never silently vanish; NaN/
  Inf results are flagged or reported as errors.
- Observed behavior: `"mean": null`, `"stddev": null` in JSON output with no
  marker, warning, or error.
- Impact: downstream agents treat `null` as "no data"; analysis results are
  silently corrupted at the extremes. Cosmetic for typical data, hence P3.
- Root cause: f64 arithmetic without finite-result guards, combined with
  serde_json's silent NaN→null serialization; no post-computation
  `is_finite` audit.
- Direction: after computing each statistic, check `is_finite()` and either
  emit an explicit `null`-with-reason (e.g., `"stddev": null,
  "stddev_error": "non-finite result"`) or a marker field; unify via a shared
  stats-output helper used by all data tools.
- Regression validation: unit tests with `[1e200, 1e200, 1e200]`-style
  fixtures asserting the output contains an explicit non-finite marker for
  mean/stddev/skewness.
- Validation reports: [V03-02](validations/F-EXT-03/V03-02.md),
  [V04-02](validations/F-EXT-03/V04-02.md)

### F-EXT-03-P3-02: `export_data` silently truncates exports to `max_preview_rows` and misreports truncation with an off-by-one

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `data.rs:996-999` truncates the frame to
  `security.limits.max_preview_rows` BEFORE writing; result reports
  `"truncated": df.shape().0 >= max_export_rows` (`data.rs:1062`) — the `>=`
  comparison flags `truncated=true` even when nothing was cut (source rows
  exactly equal to the limit), and the truncation itself is silent beyond
  that boolean (no warning in the returned text, no `original_rows` field).
- Reachability: `export_data` with a CSV/Parquet larger than
  `max_preview_rows` (default 10 000) — CLI-enabled.
- Expected invariant: an export tool must either export everything it is
  asked to or clearly report partial export with exact counts.
- Observed behavior: files are written with rows silently dropped; the only
  signal is a JSON boolean that is also wrong at the boundary.
- Impact: data loss in exported artifacts that the agent may treat as
  complete; misleading `truncated` flag.
- Root cause: preview-oriented row cap reused for a write path without
  adapting the reporting (and a `>=` instead of `>` after the head()).
- Direction: report `original_rows`, `exported_rows`, and
  `truncated = original_rows > exported_rows`; consider refusing truncation
  unless the caller passes an explicit `max_rows`.
- Regression validation: fixture with exactly `max_preview_rows` rows
  asserting `truncated=false`; fixture larger asserting `truncated=true` with
  correct counts.
- Validation reports: [V04-02](validations/F-EXT-03/V04-02.md)

### F-EXT-03-P3-03: RAG index is an ephemeral global store with silent eviction — `rag_index` success hides data loss and indexes vanish on restart

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `rag.rs:71-76` — `global_vector_store` is a process-global
  `OnceLock<Arc<RwLock<VectorStore>>>` (no persistence, no path, no
  durability); `add_chunks` silently `drain(0..excess)` the oldest chunks
  beyond MAX_CHUNKS=10 000 (`rag.rs:46-53`) while `rag_index` reports
  "Successfully indexed N document chunks" (`rag.rs:308-314`) without any
  eviction notice; `chunk_size` is documented as "characters"
  (`rag.rs:237`) but measured with byte `len()` (`rag.rs:105,141`); search
  offers no way to list what is indexed (no provenance of index contents);
  shared global means pooled agents contaminate one index (cf.
  F-EXT-01-P1-02 shared-registry pattern).
- Reachability: `rag_index`/`rag_search` registered (registry.rs:58-60,
  277-279), CLI-enabled; every indexed corpus beyond 10k chunks loses its
  oldest chunks silently.
- Expected invariant: the index either persists or is explicitly ephemeral;
  eviction is reported; chunk sizes honor the documented unit.
- Observed behavior: silent eviction, session-scoped data, byte-vs-char
  mismatch in chunking, no index inventory.
- Impact: within-session data loss without notice and false "indexed"
  claims; cross-agent index contamination; chunk sizes that surprise on
  multilingual text.
- Root cause: placeholder in-memory store shipped with success-oriented
  reporting and no lifecycle/limits honesty.
- Direction: report evicted counts and total store size in `rag_index`
  results; document (and test) the ephemeral, per-process nature; either
  persist via a framework store or gate the tool off in production
  registries; switch chunk accounting to `chars().count()`.
- Regression validation: fixture indexing >MAX_CHUNKS chunks asserting the
  eviction count is reported; a Unicode fixture asserting chunk size in
  characters.
- Validation reports: [V04-02](validations/F-EXT-03/V04-02.md)

### F-EXT-03-P3-04: `generate_chart` HTML output interpolates title and data values unescaped — script injection in the generated local artifact

- Priority: P3
- Confidence: medium (code fact; impact depends on artifact usage)
- Layer: framework
- Evidence: `chart.rs:225-299` — `display_title` is interpolated into the
  HTML title/body (`:239,297`) and `spec_json` (containing the caller's
  `data` values) is embedded raw into a `<script>` block (`:285-286`); a
  title or data value containing `</script>` or `<script>` escapes the
  context. No HTML/JS escaping anywhere in `generate_html_page`.
- Reachability: `generate_chart(…, output_format="html")` with
  model-controlled title/data, artifact written to disk and opened in a
  browser by the user (local personal-assistant threat model — the LLM is the
  user's own assistant, so impact is low-severity, but the artifact is
  user-opened).
- Expected invariant: generated HTML artifacts must not allow script
  injection from their inputs (escape-on-output).
- Observed behavior: unescaped interpolation produces an artifact whose
  content can execute script when opened.
- Impact: local script execution in the user's browser context from a
  generated artifact — low likelihood given the local model, but a
  well-known artifact-hygiene defect.
- Root cause: string-format HTML generation without escaping.
- Direction: HTML-escape `display_title` and JSON-string-escape the spec
  embedding (`<`/`&`/`>` in text; embed `spec_json` via
  `serde_json::to_string` inside a JSON literal — which `to_string` already
  escapes `</` only if `escape_slash` … at minimum escape `<` in the
  interpolated script text).
- Regression validation: unit test generating HTML with a title and data
  values containing `<script>`/`</script>` and asserting the output contains
  no raw `<script>` from the inputs.
- Validation reports: [V04-02](validations/F-EXT-03/V04-02.md)

### F-EXT-03-P3-05: `bibtex_generate` cite-key disambiguation overflows — debug panic and non-letter suffixes beyond 26 duplicates

- Priority: P3
- Confidence: high (code fact); low (trigger likelihood)
- Layer: framework
- Evidence: `research/bibtex.rs:165-173` — `char::from(b'a' + (*count - 1)
  as u8)`: for a paper batch where >256 entries share one base cite key
  (same first-author last name + same year), `*count - 1` reaches 255+ and
  `b'a' + 255` overflows u8 → panic in debug builds (release wraps to a
  control character); for 27..=256 duplicates the suffix silently becomes a
  non-letter (`{`, `|`, `}`, …).
- Reachability: `bibtex_generate(papers=[…])` with a large batch sharing
  author+year — CLI-enabled, registered in readonly subset too (P1-02).
- Expected invariant: cite keys must be deterministic, printable, and
  collision-free without arithmetic overflow.
- Observed behavior: overflow and control-character suffixes for extreme
  duplicate counts.
- Impact: debug panics in tests/development; malformed citation keys in
  release — both only for pathological input.
- Root cause: unchecked `u8` arithmetic on an unbounded counter.
- Direction: use `checked_add`/`saturating` and switch to a suffix scheme
  that is well-defined beyond 'z' (e.g., `a..z, aa, ab…` or a numeric
  disambiguator).
- Regression validation: unit test generating 300 same-key papers asserting
  no panic and distinct printable keys.
- Validation reports: [V04-02](validations/F-EXT-03/V04-02.md)

### F-EXT-03-P3-06: Duplicate `parse_page_range` implementations with divergent limits; `extract_pdf` result header reports the requested range, not the extracted one

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: two independent page-range parsers —
  `pdf.rs:234-315` (enforces `limits.max_preview_pages`, validates
  start<=end and bounds) and `research/pdf_fetch.rs:194-248` (no page-count
  cap — `"all"` enumerates every page of the document, no start<=end check
  but `start..=end` handles it safely). `pdf.rs:98-108` formats the result
  header with the RAW `pages` parameter ("Text Content (pages 1-100 …)")
  even when `parse_page_range` limited extraction to `max_preview_pages`
  pages — the header lies about what was extracted.
- Reachability: `extract_pdf`/`pdf_fetch` with multi-page documents
  (CLI-enabled media + research features).
- Expected invariant: one parser for one semantic (AGENTS.md), and result
  headers describe the actual extracted pages.
- Observed behavior: two parsers with different safety limits (a read-only
  tool can force full-document page enumeration via pdf_fetch "all"), and
  extract_pdf's header reports pages that were never extracted.
- Impact: divergent behavior for the same user intent; misleading page
  summary; unbounded per-call work in pdf_fetch.
- Root cause: parser written twice during module history without
  consolidation.
- Direction: consolidate into one shared page-range parser (with the
  max-preview cap), and have `extract_pdf` report the actual page list it
  extracted.
- Regression validation: shared-parser unit tests for both call sites;
  extract_pdf fixture with `pages="1-100"` on a 30-page document asserting
  the header lists the pages actually returned.
- Validation reports: [V01-01](validations/F-EXT-03/V01-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search (web-fetch family, page-range parsers, research memory, name collisions) | yes | passed | [V01-01](validations/F-EXT-03/V01-01.md) |
| V02 | Registration and reachability (feature gates, CLI features, readonly surface) | yes | passed | [V02-01](validations/F-EXT-03/V02-01.md) |
| V03 | IQR n=4 panic — index arithmetic replication | yes | passed (OOB confirmed for n=4) | [V03-01](validations/F-EXT-03/V03-01.md) |
| V03 | serde_json NaN/Inf → null reproduction | yes | passed (silent null confirmed) | [V03-02](validations/F-EXT-03/V03-02.md) |
| V03 | Readonly-subset Write-permission registration trace | yes | passed | [V03-03](validations/F-EXT-03/V03-03.md) |
| V03 | research_remember/recall stub behavior trace | yes | passed | [V03-04](validations/F-EXT-03/V03-04.md) |
| V04 | `cargo check -p echo_tools --no-default-features --features "data,statistics,chart,research,media,rag,web,database" --locked` | yes | passed (exit 0) | [V04-01](validations/F-EXT-03/V04-01.md) |
| V04 | `cargo test -p echo_tools` (same features) | yes | passed (exit 0; 92 passed, 2 ignored) | [V04-02](validations/F-EXT-03/V04-02.md) |
| V04 | Live-network provider tests (arxiv/openalex/zotero/tavily…) | conditional | not_run — opt-in `#[ignore]` tests require external API keys / `EKO_PROVIDER_SMOKE=1`; review is read-only and the task invariants (validation/pagination/limits/schema honesty) do not require live network behavior | [V04-02](validations/F-EXT-03/V04-02.md) |
| V05 | Historical-document drift check | yes | passed | [V05-01](validations/F-EXT-03/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN:109/382/840 — statistics split (exploratory only; formal inference via reviewable scripts through `run_code`) | current | `statistics.rs:118-129`, registry test `registry.rs:467-495` |
| echo-tools/README.md feature table | current (naming drift only) | `lib.rs` doc table; tool names are generic in README |
| research/memory.rs:90 "In production, this would persist to SQLite" | stale (contradicts AGENTS.md no-SQLite and describes nonexistent persistence) | P1-01, [V03-04](validations/F-EXT-03/V03-04.md) |

## Coverage And Uncertainty

- Not inspected deeply: `data.rs` filter/aggregate/transform/contribution/bin
  tool bodies were outlined and their shared helpers (`parse_filter_expression`,
  `data_tool_response`) read, but per-tool edge cases beyond the shared
  envelope were not exhaustively traced; `clinical_trials.rs` parsing details
  beyond the search path; `excel.rs` read/profile/csv tool bodies (the write
  and load contracts were the focus); `security.rs` internals (F-SEC-01).
- Live-network provider behavior is explicitly not validated (recorded
  `not_run` above); all network-failure classifications are static.
- `SqlQueryTool` in the readonly subset is a documented risk trade-off
  (registry.rs:77) with keyword filter + READ ONLY transaction for non-SQLite;
  recorded here, not raised as a finding.
- `web_fetch_enhanced`/`ImageFetchTool` carry `#[allow(dead_code)]` markers on
  fields/methods but the tools themselves ARE registered and live (V02-01);
  the `#[allow(dead_code)]` on `ExploratoryStatisticsTool`
  (`statistics.rs:26`) is spurious like F-EXT-02's `git_status` note.
- P1-01/P1-02/P1-03 are statically verified; no end-to-end dynamic run was
  executed (read-only task, no fixtures in source). All carry explicit
  regression validations.
- F-EXT-01-P2-01 (WRITE_TOOLS drift) is reinforced by two new instances from
  this task: `excel_load` (writes Parquet/CSV files, declares `Read`
  permission, `excel.rs:1041-1043,1380-1410`, absent from WRITE_TOOLS) and
  `rag_index`; the fix directions are compatible (single write-classification
  authority).
- The `extract_pdf` header honesty issue (P3-06) and `export_data` truncation
  reporting (P3-02) are reporting-honesty defects, not data-corruption in the
  stored artifacts themselves.

## Handoff

- Downstream tasks may rely on: tool-by-feature registration map (V02-01);
  readonly-surface composition defect (P1-02) — relevant to any task using
  read-only Subagents (A-TSK-*, A-TOOL-01, X-AUT-01); the false-success
  memory stubs (P1-01) — relevant to A-DOM-01 (research workflows) and
  A-MEM-01 (memory policy must not rely on these tools); the enum-fallback
  family (P2-02) and NaN→null family (P3-01) for X-TOL-01 (tool error/schema
  conformance) and A-OUT-01 (export/artifact delivery).
- `F-EXT-01`: P1-02/P2-01 directions intersect — the readonly subset and
  WRITE_TOOLS must derive from one write-classification authority;
  `excel_load` and `rag_index` are two more drift instances.
- `A-DOM-01` (EKO analysis/research): must not depend on
  `research_remember`/`research_recall` persistence; bibtex/rag readonly
  exposure must be reviewed app-side.
- `X-BND-01`: record the four-tool web-fetch duplication (P2-01), the
  research-memory semantic overlap (P1-01), and the readonly-subset
  classification decision (P1-02).
- This report becomes stale if: the tool registry composition changes
  (readonly subsets, feature gates), research memory tools gain persistence,
  the web-fetch family converges, or `data_quality.rs` quantile logic is
  reworked.
