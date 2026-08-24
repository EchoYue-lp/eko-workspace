# F-EXT-03: Data, research, media, database, and Web tools

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: not-applicable (framework-only task)
> Worktree state: one untracked probe file `echo-agent/tests/f_rct_01_probe.rs` outside `echo-tools/`; all `echo-tools` sources clean.

## Question

Are the domain tool contracts honest about validation, provenance, pagination,
numerical limits, network failures, and artifact output?

## Scope

Primary source paths and behaviors inspected (all under
`echo-agent/echo-tools/src/`):

- `Cargo.toml` `[features]` table, `lib.rs` module cfg-gates, `registry.rs`
  read-only vs writer registration.
- `data.rs`, `data_quality.rs`, `statistics.rs` — numerical limits, overflow,
  empty input, single-row inputs.
- `research/{arxiv.rs, bibtex.rs, clients.rs, clinical_trials.rs, memory.rs,
  pdf_fetch.rs, pubmed.rs, semantic_scholar.rs, mod.rs}` — provenance,
  pagination, network failure classification.
- `web/{fetch.rs, search.rs, extract.rs, providers/}`,
  `media/{image_fetch.rs, web_fetch_enhanced.rs, mod.rs}` — provenance,
  pagination, SSRF, body-size caps.
- `database.rs`, `rag.rs` — connection handling, SQL injection (audit
  1.1/1.12), schema validation, cursor binding, in-memory bounds.
- `security.rs:520-650` — SSRF validator and pinned-IP connect primitives.
- `echo-agent/echo-core/src/tools/pagination.rs` — cursor envelope +
  fingerprint binding (consumed by `web_search`, `sql_query`).

Document-format tools (`pdf.rs`, `excel.rs`, `word.rs`, `text.rs`, `image.rs`,
`chart.rs`) were inspected for artifact-output and numerical-limit surface
only; their detailed format-parsing correctness is out of scope.

## Out Of Scope

- Shell / file / code / git tools — deferred to F-EXT-02.
- Generic `Tool` contract, registry mechanics, pagination/artifact
  primitives — covered by F-EXT-01 (this task consumes them as-is).
- Provider-specific search quality (DuckDuckGo HTML scraping, Tavily/Brave
  wire format) — not a contract-honesty concern.
- Permission / risk-gating runtime behavior (who may call
  `ToolPermission::Network`) — deferred to the permission task.
- Application adapter wiring of these tools into the agent runtime —
  deferred to B-PATH-01 / A-TOOL-01.

## Inputs

- Required documents read:
  - `AGENTS.md` (root) — no-panic rule, checked/saturating arithmetic,
    UTF-8-safe string handling, framework-vs-application layering gate,
    dead-code cleanup rule.
  - `docs/comprehensive-review/REPORTING.md`.
  - `docs/comprehensive-review/templates/task-report.md`,
    `docs/comprehensive-review/templates/validation-report.md`.
- Dependency task reports read:
  - `F-EXT-01` (this reviewer) — relied on its conclusion that the generic
    `Tool` / `ToolResult` / `ToolFailure` taxonomy is the single typed
    contract, that cursor pagination lives in `echo-core::tools::pagination`,
    and that `ToolOutputArtifactWriter` is the single spill primitive. This
    task checks whether the domain tools use those primitives correctly and
    honestly.
- Historical documents treated as hypotheses:
  - Audit findings 1.1, 1.12 (SQL injection) and 1.7 (image-fetch SSRF)
    referenced via the task prompt. Classified under Historical Claim Status
    below.

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | Yes. The data/research/database/web/rag/chart domain tools are generic agent capabilities that any `echo-agent` consumer (CLI, third-party headless, future reuse) may want. They correctly live in the `echo-tools` framework crate and are each feature-gated so a consumer pays only for what it uses. The pagination cursor, artifact spill, and SSRF primitives they consume are themselves in `echo-core` (verified by F-EXT-01). |
| EKO product policy | None at this layer. `SqlQueryTool` being kept in `register_readonly_tools` despite write capability (`registry.rs:74-80`) is the only product-flavored choice; the comment justifies it ("local analysis context, risk is low") and the actual protection is framework-level (`SET TRANSACTION READ ONLY` + keyword filter). No EKO-specific field or policy is embedded in these tools. |
| Adapter boundary | The framework exposes the tools and their `parameters()` JSON Schema; the application adapter feeds `ToolContext` (`output_artifacts`, working dir) at execution time. `database::SqlQueryTool::execute_with_context` and `web::WebFetchTool::execute_with_context` are the only seams; both read `ctx.output_artifacts` for spill configuration and nothing else. Thin, lossless, no scheduling authority. |
| Duplicate search | Searched names: `SqlQueryTool`, `ListTablesTool`, `DescribeTableTool`, `WebFetchTool`, `WebFetchToolEnhanced`, `WebSearchTool`, `WebExtractTool`, `ImageFetchTool`, `DataReadTool` (+13 sibling data tools), `MissingValueAnalysisTool`, `OutlierDetectionTool`, `ConsistencyCheckTool`, `ExploratoryStatisticsTool`, `RagIndexTool`, `RagSearchTool`, `RagChunkDocumentTool`, `ArxivSearchTool`, `PubMedSearchTool`, `SemanticScholarSearchTool`, `ClinicalTrialsSearchTool`, `PdfFetchTool`, `BibtexGenerateTool`, `GenerateChartTool`. Result: no duplicate authority. `WebFetchTool` (`web/`) vs `WebFetchToolEnhanced` (`media/`) are distinct tools with distinct names (`web_fetch` vs `web_fetch_enhanced`), registered together when both `web` and `media` are enabled. |
| Migration deletion | No migration proposed in this task. No deletion candidate identified beyond the dead `ImageFetchTool::is_image_url` helper noted in F-EXT-03-P3-03. |

## Current Path

Verified data flow at commit `9b0e0fa`:

1. **Feature gating.** `Cargo.toml:15-37` defines 11 user features plus `full`
   and empty `default`. `statistics = ["data"]` is the only intra-`echo-tools`
   feature dependency. `lib.rs:22-86` cfg-gates each module exactly once.
   `registry.rs` constructs each tool under the matching `#[cfg(feature =
   "...")]` block.

2. **Read-only vs writer split.** `register_readonly_tools`
   (`registry.rs:21-193`) excludes `run_code`, all write/edit/delete file
   tools, git commit/branch/worktree-enter, `ExcelWriteTool`, and
   `DataExportTool`. Two `#[test]`s at `registry.rs:426-463` assert `run_code`
   presence in `register_all_tools` and absence in the read-only subset.

3. **Pagination.** `web_search` (`web/search.rs:141-167`) and `sql_query`
   (`database.rs:107-114, 178-189`) consume `PageRequest::from_parameters`
   and `paginate(items, &query_identity)`. The cursor envelope
   (`pagination.rs:65-70`) binds `offset` to a SHA-256 fingerprint of
   `{query, limit, items snapshot}` so a cursor is rejected
   (`CursorQueryMismatch`, `pagination.rs:135-137`) when the query, limit, or
   underlying result set changes. The four research search tools
   (`arxiv/pubmed/semantic_scholar/clinical_trials`) do **not** use this
   primitive.

4. **SSRF.** `WebFetchTool::execute_with_context` connects via
   `crate::security::ssrf_safe_get(url, timeout, 5)` (`web/fetch.rs:201`).
   `ImageFetchTool::execute` uses `ssrf_safe_request` for HEAD and
   `ssrf_safe_get` for GET (`media/image_fetch.rs:198-203, 226-227`).
   `ssrf_safe_get` resolves once, rejects any private/link-local IP
   (`security.rs:615-664`), and pins the connection to the validated public
   addresses, closing the DNS-rebinding TOCTOU window.

5. **Artifact output.** `WebFetchTool` (`web/fetch.rs:281-318`) and
   `SqlQueryTool` (`database.rs:494-537`) spill oversized payloads to
   `persist_tool_output`, set `ToolResult::truncated`, and emit a
   `ToolOutputArtifactRef`. Inline cells in SQL results are truncated by
   `chars().take(MAX_INLINE_CELL_CHARS)` (`database.rs:586-592`), UTF-8 safe.

6. **SQL injection defense.** `SqlQueryTool` layers (a) statement-prefix
   allowlist (`database.rs:119-132`), (b) dangerous-keyword denylist
   (`database.rs:134-170`), (c) `SET TRANSACTION READ ONLY` for non-SQLite
   (`database.rs:432-458`), (d) sqlx single-statement execution. Table names
   in `describe_table` are validated to `[A-Za-z0-9_.]` and single-quote
   escaped (`database.rs:306-334`).

7. **Numerical computation.** `statistics.rs` uses saturating/`unwrap_or`
   throughout and returns `None` for std (`len() < 2`) and moments
   (`len() < 3`). `data_quality.rs` IQR path has an index-arithmetic defect
   (F-EXT-03-P1-01). `data.rs` profile path divides variance by `n - 1`
   without checking `n == 1` (F-EXT-03-P3-01). `rag.rs::cosine_similarity`
   guards zero-norm and mismatched dims.

## Findings

### F-EXT-03-P1-01: `outlier_detection` (IQR method) panics on any numeric column with exactly four finite values

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-tools/src/data_quality.rs:232-235` — the
    `values.len() < 4` guard, so `n == 4` is the first accepted length.
  - `echo-agent/echo-tools/src/data_quality.rs:253-254` — the buggy index
    arithmetic:
    ```rust
    let q1 = sorted[n / 4.min(n - 1)];
    let q3 = sorted[3 * n / 4.min(n - 1)];
    ```
- Reachability: live. `OutlierDetectionTool::execute` is registered under
  `feature = "data"` (`registry.rs:147-152`). The default `method` is
  `"iqr"` (`data_quality.rs:180`). Any CSV/JSON/Parquet file with a numeric
  column containing exactly four finite values triggers the panic on the
  default code path. Confirmed by an isolated `rustc` reproduction
  (`/tmp/iqr_repro.rs`) that emits the exact indexing expression: exit code
  101 with `index out of bounds: the len is 4 but the index is 4`.
- Expected invariant: AGENTS.md "禁止任何会导致系统 panic 的 API" — never use
  direct indexing where the index is not provably in range; the tool must
  tolerate any input that passes its own documented minimum (`len() >= 4`).
- Observed behavior: the intended clamp was
  `sorted[(n / 4).min(n - 1)]` / `sorted[(3 * n / 4).min(n - 1)]`, but due
  to method-call precedence `4.min(n - 1)` binds to the **divisor**, not the
  index. For `n == 4`: divisor becomes `min(4, 3) = 3`, so
  `q3_idx = (3 * 4) / 3 = 4`, and `sorted[4]` is out of bounds (valid
  indices `0..=3`). For every `n >= 5` the divisor is `4` and the index lands
  in range, which is why the existing test (`data_quality.rs:563-579`) with
  `n == 9` passes and the bug stayed latent.
- Impact: a single-row-short dataset crashes the tool call instead of
  returning a structured result. For a local analysis assistant this is a
  capability failure on a common small-input edge case, not data loss.
  Categorized P1 (major capability failure on valid input) rather than P0
  (no data corruption / secret exposure).
- Root cause: operator-precedence mistake — `.min(n - 1)` was meant to clamp
  the quotient index but parsed as clamping the divisor.
- Direction: replace both lines with the clamped-index form:
  ```rust
  let q1 = sorted[(n / 4).min(n - 1)];
  let q3 = sorted[(3 * n / 4).min(n - 1)];
  ```
  Better, switch to `.get(...)` per AGENTS.md. Optionally raise the
  `values.len() < 4` guard to `< 5` if quartile semantics are deemed
  meaningless at `n == 4`. Add a regression test for `n == 4`.
- Regression validation: a unit test feeding a four-value numeric CSV to
  `OutlierDetectionTool` with `method = "iqr"` must return a structured
  result rather than panicking; the existing `n == 9` test must still pass.
- Validation reports: [V02](../validations/F-EXT-03/V02-01.md)

### F-EXT-03-P2-01: Research search tools return collections but expose no cursor pagination, unlike `web_search` / `sql_query`

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-tools/src/research/arxiv.rs:111-114` — URL hardcodes
    `start=0&max_results={max_results}`; the tool has no `cursor` parameter
    in its schema (`arxiv.rs:42-69`).
  - `echo-agent/echo-tools/src/research/pubmed.rs:95-116` — same shape,
    `retmax` only, no offset/cursor.
  - `echo-agent/echo-tools/src/research/semantic_scholar.rs:101-104` —
    `limit` only, no offset/cursor.
  - Contrast: `echo-agent/echo-tools/src/web/search.rs:141-167` consumes
    `PageRequest::from_parameters` and emits `page.next_cursor`;
    `database.rs:107-114, 178-189` does the same for `sql_query`.
- Reachability: live. All four research tools are registered under
  `feature = "research"` (`registry.rs:162-176, 386-400`). An agent that
  asks for "more results" after the first page has no mechanism to advance.
- Expected invariant: a collection-returning tool family should expose a
  consistent pagination contract so the agent can page through results
  uniformly; the framework already provides `PageRequest` / `PageInfo` for
  exactly this purpose (established by F-EXT-01).
- Observed behavior: each research tool fetches its first `limit`-sized page
  and stops. The descriptions are technically honest (arxiv says "Max 100
  results per query", no promise of a next page), but the family is
  internally inconsistent — two of the seven collection tools paginate, four
  do not.
- Impact: an agent doing literature review cannot retrieve results beyond the
  first page without re-issuing a narrower query. Workable but a real
  capability gap for a research-focused toolset.
- Root cause: the research tools predate the framework's cursor-pagination
  primitive and were never migrated to it.
- Direction: thread the upstream API's offset/`start`/page token through
  `PageRequest` and emit a `PageInfo` cursor (as `web_search` does), or — if
  pagination is deliberately out of scope for these tools — state so
  explicitly in each description. Prefer the former for parity.
- Regression validation: a test that fetches page 1, then replays the
  returned cursor to fetch page 2 and asserts disjoint results.
- Validation reports: [V03](../validations/F-EXT-03/V03-01.md)

### F-EXT-03-P3-01: `DataProfileTool` sample variance divides by zero for single-row numeric columns, silently emitting `null` stddev/variance

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-tools/src/data.rs:660-673`:
    ```rust
    if !values.is_empty() {                     // n >= 1
        ...
        let n = sorted.len();
        ...
        let variance: f64 =
            sorted.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>()
                / (n - 1) as f64;               // n == 1 -> divide by 0
        let stddev = variance.sqrt();
    ```
- Reachability: live. `DataProfileTool` is registered under
  `feature = "data"` (`registry.rs:137`). Any numeric column with exactly one
  non-null finite value reaches this branch.
- Expected invariant: descriptive statistics should either compute a
  meaningful value or return a documented `None`/error for undefined cases
  (sample variance is undefined for `n == 1`).
- Observed behavior: `n - 1 == 0` produces `variance = inf` and
  `stddev = inf`; `serde_json::json!` then serializes non-finite f64 as
  `null`, so the agent receives `"stddev": null, "variance": null` with no
  diagnostic explaining why. Compare `statistics.rs:156`, which returns
  `None` for `len() < 2` explicitly.
- Impact: low. No panic, no data loss; the agent gets a confusing `null`
  instead of a clear "not enough data" message. Mild contract-honesty gap.
- Root cause: the guard at `data.rs:660` checks non-emptiness but not the
  `n >= 2` required for sample variance.
- Direction: guard with `if n >= 2` for the variance/stddev block (mirroring
  `statistics.rs:155-164`), or emit `"variance": null` with an
  `"error": "need >= 2 values for sample variance"` field as the neighboring
  code does for non-numeric columns.
- Regression validation: a unit test with a one-row numeric CSV asserting
  the result either omits variance or returns a structured "insufficient
  data" message rather than `null`.
- Validation reports: [V02](../validations/F-EXT-03/V02-01.md)

### F-EXT-03-P3-02: Research HTTP failures returned as raw `ToolError::ExecutionFailed`, not classified as retryable `ToolFailure`

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-tools/src/research/arxiv.rs:118-126` —
    `client.get(&url).send().await` error mapped to plain
    `ToolError::ExecutionFailed`.
  - `echo-agent/echo-tools/src/research/clients.rs:79,99` — shared
    `http_error(...)` helper returns a plain error; research clients
    (`OpenAlexClient`, `CrossrefClient`, etc.) inherit it.
  - Contrast: `echo-agent/echo-tools/src/web/search.rs:177-186` wraps the
    provider failure in
    `ToolFailure::new(ToolFailureCategory::Transient).retryable().with_retry_after(1_000)`,
    handing retry policy to the central runtime (verified by two tests at
    `search.rs:291-323`).
- Reachability: live. Every `arxiv_search` / `pubmed` /
  `semantic_scholar_search` / `clinical_trials_search` invocation that hits a
  transient network failure returns a non-retryable error.
- Expected invariant: transient transport failures (DNS hiccup, connection
  reset, 5xx) should be classified as `ToolFailureCategory::Transient` so the
  F-EXT-01 retry policy can recover automatically, consistent with how
  `web_search` already behaves.
- Observed behavior: a single connection-reset on the arxiv API fails the
  whole tool call with no retry, even though the framework already has the
  machinery to retry transient failures.
- Impact: low for one-shot queries, material for batched literature scans
  where a single transient blip drops a query that the agent then has to
  re-issue manually.
- Root cause: the research tools predate the `ToolFailure` taxonomy and were
  not migrated when `web_search` was.
- Direction: wrap the HTTP send + parse error path in
  `ToolResult::error(...).with_failure(ToolFailure::new(Transient).retryable().with_retry_after(ms))`
  for retriable status codes / transport errors, mirroring `web_search`.
- Regression validation: a mock-client test that simulates one transport
  failure then a success, asserting the structured `failure.category ==
  Transient` (mirror of `search.rs:310-323`).
- Validation reports: [V03](../validations/F-EXT-03/V03-01.md)

### F-EXT-03-P3-03: `ImageFetchTool::is_image_url` (dead helper) bypasses the SSRF-safe connect path

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-tools/src/media/image_fetch.rs:47-67` — `is_image_url`
    issues `self.client.head(url).send().await` directly on the raw
    `reqwest::Client`, with no `crate::security::ssrf_safe_*` call.
  - `echo-agent/echo-tools/src/media/image_fetch.rs:70-119` —
    `download_image_as_base64` (also dead) does use `ssrf_safe_get`
    (`image_fetch.rs:74`).
  - The live `execute` path (`image_fetch.rs:162-299`) uses
    `ssrf_safe_request` (HEAD, `image_fetch.rs:198-203`) and `ssrf_safe_get`
    (GET, `image_fetch.rs:226-227`).
- Reachability: **not currently reachable**. Both helpers are tagged
  `#[allow(dead_code)]` and are not called from `execute`. The live image
  fetch path is SSRF-safe — audit finding 1.7 (image-fetch SSRF) is
  **fixed**.
- Expected invariant: AGENTS.md "随手清理是强制要求" — dead code with an
  unsafe pattern should not linger, because revival without migration would
  silently reintroduce the SSRF hole that audit 1.7 closed.
- Observed behavior: the dead `is_image_url` keeps a non-SSRF-safe HEAD
  pattern on the type. Anyone who later wires it back into `execute` (or a
  new tool) reopens the DNS-rebinding window the live path already closed.
- Impact: none at runtime today. Risk is future-regression only.
- Root cause: the helper was written before the SSRF migration and not
  cleaned up; `#[allow(dead_code)]` suppressed the warning that would have
  reminded someone.
- Direction: either delete `is_image_url` (preferred under AGENTS.md
  dead-code rule — the live `execute` already does its own image detection
  via `ssrf_safe_request` HEAD), or migrate it to `ssrf_safe_request` if a
  use case appears.
- Regression validation: `cargo check -p echo_tools --features media` after
  deletion; the live `execute` path must remain unchanged.
- Validation reports: [V03](../validations/F-EXT-03/V03-01.md)

### F-EXT-03-P3-04: `rag_index` advertises an `overlap` parameter but never applies it

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/echo-tools/src/rag.rs:238-241` — schema advertises `overlap`
    as "Overlap characters between chunks (default 100)".
  - `echo-agent/echo-tools/src/rag.rs:264-267, 468-471` — `overlap` is read
    from parameters and passed into `chunk_text`.
  - `echo-agent/echo-tools/src/rag.rs:94-131` — `chunk_text` accepts
    `overlap` but never reads it.
  - `echo-agent/echo-tools/src/rag.rs:134` — `chunk_by_sentences` binds the
    argument to `_overlap` (underscore-discarded).
- Reachability: live. Every `rag_index` and `rag_chunk_document` call accepts
  `overlap` and silently ignores it.
- Expected invariant: a tool's parameter schema is a contract; if a
  parameter is documented, the tool must either honor it or reject
  unsupported values.
- Observed behavior: chunks are produced without any overlap regardless of
  the `overlap` value, but the tool reports success and the description
  promises overlap behavior.
- Impact: low. An agent relying on overlapping chunks for cross-boundary
  retrieval coverage gets silently worse results than the contract implies.
- Root cause: the overlap feature was stubbed (parameter + plumbing) but
  never implemented in the chunking logic.
- Direction: either implement sentence/paragraph overlap in `chunk_text`
  (carry the last `overlap` chars into the next chunk), or remove the
  `overlap` parameter from the schema and the description until it is
  implemented.
- Regression validation: a test asserting that two adjacent chunks share
  exactly `overlap` trailing/leading characters; or, if the parameter is
  removed, a test asserting the schema no longer mentions `overlap`.
- Validation reports: [V04](../validations/F-EXT-03/V04-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Feature/capability map — every tool gated by exactly one feature, read-only/writer split correct | yes | passed | [V01-01](../validations/F-EXT-03/V01-01.md) |
| V02 | Data/statistics numerical safety — no panic / no unguarded division on empty/single-row/n=4 inputs | yes | failed | [V02-01](../validations/F-EXT-03/V02-01.md) |
| V03 | Research/media/web provenance, pagination, network failure, SSRF (audit 1.7) | yes | passed (with findings) | [V03-01](../validations/F-EXT-03/V03-01.md) |
| V04 | Database/rag connection handling, SQL injection (audit 1.1/1.12), schema validation | yes | passed | [V04-01](../validations/F-EXT-03/V04-01.md) |
| V05 | Historical-document drift check | yes | passed | See Historical Claim Status below; each audit claim classified inline. |

V02 is recorded as **failed** because the IQR panic (F-EXT-03-P1-01) refutes
its claim. The other three validations pass; their associated findings
(F-EXT-03-P2-01, -P3-02, -P3-03, -P3-04) are contract-honesty gaps observed
during passing inspections, not validation failures.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| Audit 1.1 / 1.12 — SQL injection in `SqlQueryTool` / `describe_table` | fixed | `database.rs:119-170` (prefix allowlist + keyword denylist), `database.rs:306-334` (table-name validation + quote escaping), `database.rs:432-458` (`SET TRANSACTION READ ONLY`). V04-01 confirms three independent layers. |
| Audit 1.7 — image-fetch SSRF (raw `client.get` on user URL) | fixed (on live path) | `media/image_fetch.rs:198-203, 226-227` route HEAD/GET through `ssrf_safe_*`. Stale residue: dead `is_image_url` helper still uses raw `client.head` (F-EXT-03-P3-03). |

## Coverage And Uncertainty

- Code not inspected in depth:
  - `excel.rs` (1400+ lines of calamine/rust_xlsxwriter wiring) — scanned for
    `as usize`/`as u32` casts and `unwrap`s; the `get_value((row as u32, col
    as u32))` pattern recurs throughout but row/col come from
    already-bounded calamine ranges, so no panic surface was identified.
    Deep format-parsing correctness is out of scope.
  - `pdf.rs`, `word.rs`, `text.rs`, `image.rs` — inspected only for size/page
    limits (`pdf.rs:234-310` honors `ResourceLimits::max_preview_pages`) and
    artifact output. No numerical-limit defect found.
  - `research/{bibtex.rs, memory.rs, pdf_fetch.rs}` — `bibtex` is a pure
    formatter; `memory` is a local KV; `pdf_fetch` reuses `ssrf_safe_get`.
    Not exhaustively line-read.
  - `web/providers/{duckduckgo,brave,tavily}` — provider wire-format
    correctness out of scope.
- Validations not executed at runtime beyond the IQR repro: V01, V03, V04 are
  static inspections. The IQR panic was confirmed with an isolated `rustc`
  program because compiling echo-tools with the `data` feature + polars for a
  single test is disproportionately expensive and the bug is in pure index
  arithmetic (no polars involvement); the standalone repro is a faithful
  witness (V02-01 explains the equivalence).
- Environmental limits: none. `echo-tools` sources are clean at `9b0e0fa`.
- Claims that remain uncertain:
  - Whether any downstream consumer actually relies on the four-research-tool
    absence of pagination (i.e., is the gap observed-in-practice or only
    observed-in-code?). The capability gap is certain; its user impact is
    inferred.
  - Whether the `n == 1` variance `null` has confused any agent in practice;
    the silent-wrong-output behavior is certain, the user-visible impact is
    inferred.

## Handoff

- Conclusions downstream tasks may rely on:
  - The generic primitives audited in F-EXT-01 (`ToolResult` /
    `ToolFailure` / `PageRequest` / `ToolOutputArtifactWriter`) are used
    correctly by `web_search`, `web_fetch`, and `sql_query`. Any tool-runtime
    retry/pagination/artifact task can treat these three as the reference
    implementations.
  - The research tool family is the inconsistent one: it does not paginate and
    does not classify transient failures. A tool-runtime / agent-recovery
    task (F-RCT-04 and similar) should not assume uniform failure handling
    across all `echo-tools` collection tools.
  - SSRF is uniformly enforced on every live outbound HTTP path in
    `echo-tools` (`web_fetch`, `image_fetch`, `web_fetch_enhanced`, research
    clients via `ssrf_safe_redirect_policy` + hardcoded hosts). The security
    task (F-SEC-01) can rely on this.
  - SQL injection (audit 1.1/1.12) is closed by layered defense; the
    database task / A-TSK-* / permission tasks do not need to re-audit the
    SQL surface.
- Reports they must read:
  - [V02-01](../validations/F-EXT-03/V02-01.md) for the IQR panic repro
    command and exit code.
  - [V03-01](../validations/F-EXT-03/V03-01.md) for the per-tool
    provenance/pagination/SSRF matrix.
  - [V04-01](../validations/F-EXT-03/V04-01.md) for the three-layer SQL
    injection defense and the cursor-fingerprint binding.
- Conditions that make this report stale:
  - Any change to `data_quality.rs:253-254` invalidates F-EXT-03-P1-01 and
    V02.
  - Adding cursor pagination to any of the four research tools invalidates
    F-EXT-03-P2-01 (partially or fully).
  - Wrapping research HTTP errors in `ToolFailure::Transient` invalidates
    F-EXT-03-P3-02.
  - Deleting or SSRF-migrating `ImageFetchTool::is_image_url` invalidates
    F-EXT-03-P3-03.
  - Implementing or removing the `rag_index` `overlap` parameter invalidates
    F-EXT-03-P3-04.
  - Loosening `validate_db_url` or the `describe_table` table-name validator
    invalidates V04.
- Follow-up task IDs (no fixes implemented in this review):
  - A future tool-correctness fix task should land the one-line IQR index
    clamp (F-EXT-03-P1-01) plus a `n == 4` regression test; this is the only
    P1 and is cheap to fix.
  - A research-tools parity task should add cursor pagination (F-EXT-03-P2-01)
    and `Transient` failure classification (F-EXT-03-P3-02) to arxiv /
    pubmed / semantic_scholar / clinical_trials, reusing `PageRequest` and
    the `web_search` failure pattern.
  - A cleanup task should delete the dead `is_image_url` helper
    (F-EXT-03-P3-03) and resolve the `rag_index` `overlap` contract
    (F-EXT-03-P3-04) — either implement or remove.
