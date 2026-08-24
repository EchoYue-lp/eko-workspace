# A-DOM-01: Data analysis and research workflows

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63 (cross-referenced for framework research clients; no framework modification)
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only review)

## Question

Are EKO-specific analysis/research policies, provenance, formal inference,
connectors, workbench state, and artifact export correctly placed and reliable?

## Scope

Primary source paths and behaviors inspected (all under
`echo-agent-cli/` unless noted):

- `echo-agent-app-core/src/analysis.rs` (full, 1214 lines) — file-backed
  analysis workspaces, manifest, run records, stale detection, output
  fingerprinting, `run_code` delegation, atomic writes, path-escape guards.
- `echo-agent-app-core/src/research.rs` (full, 2207 lines) — file-backed
  research library: sources, evidence, reviews, PRISMA flow, citation audit,
  multi-format export, Pandoc/Quarto renderer discovery.
- `echo-agent-app-core/src/research_connectors.rs` (full, 688 lines) —
  OpenAlex/Crossref/EuropePMC search-and-ingest, Zotero import/export, Europe
  PMC enrichment, `AutoIngestResearchTool` wrapper.
- `echo-agent-app-core/src/research_tool.rs` (full, 269 lines) — agent-facing
  `research_library` tool action surface.
- `echo-agent-app-core/src/tasks/task_runtime/profiles.rs:202-232, 344-361` —
  `DATA_ANALYSIS` profile (exploratory/formal boundary), boundary test.
- `echo-agent-app-core/src/subagents/data/analyst.md` (full) — subagent
  instructions that enforce the formal-inference script contract.
- `echo-agent-app-core/src/runtime.rs:283-289` — research library wiring at
  runtime composition step 11.
- `echo-agent-cli/src/tauri/commands/analysis.rs` (full, 146 lines) — analysis
  IPC commands, per-analysis cancellation via `session.cancel_token` DashMap.
- `echo-agent-cli/src/tauri/commands/research.rs:1-229` — research IPC
  commands, IPC error mapping.
- `echo-agent-cli/src/cli/cmd_impls/analysis.rs` (full) — `/analysis` slash
  command (CLI/TUI/channels surface).
- `echo-agent-cli/web-frontend/src/components/analysis/AnalysisPanel.tsx`
  (full, 630 lines) — analysis workbench GUI, lineage rendering, draft
  persistence, cancel.
- `echo-agent-cli/web-frontend/src/components/papers/PaperPanel.tsx` (full,
  483 lines) — paper library and connector panel.
- `echo-agent-cli/web-frontend/src/components/papers/ReviewWorkbench.tsx`
  (`computePrismaFlow` and structure, lines 35-120) — systematic-review
  workbench PRISMA derivation and rendering entry points.
- `echo-agent-cli/web-frontend/src/components/papers/ReviewMatrix.tsx` (full,
  290 lines) — evidence matrix and client-side CSV export.
- `echo-agent/echo-tools/src/research/clients.rs:47-105, 700-768` — framework
  research client failure classification (`http_error`/`invalid`/`json_error`
  all map to `ToolError::ExecutionFailed` / `InvalidParameter`, never
  `ToolFailure::Transient`).
- `echo-agent/echo-tools/src/research/arxiv.rs:110-145` — arxiv transport
  failure classification (also plain `ExecutionFailed`).

Document consulted:

- `echo-agent-cli/docs/2026-07-18-statistical-inference-correctness.md` — M12
  spec that fixes the exploratory/formal boundary.
- `echo-agent-cli/docs/MASTER-PLAN.md:109, 382, 840` — milestone notes on the
  statistics split.

## Out Of Scope

Deferred to downstream/other tasks:

- **F-EXT-03**: framework data/research/web/database tool numerical limits,
  SSRF, pagination primitives, SQL injection. This task consumes F-EXT-03's
  conclusions (e.g., research HTTP failures are `ToolError::ExecutionFailed`,
  not `ToolFailure::Transient`) and does not re-audit the framework internals.
- **A-OUT-01**: end-to-end output profiles, Markdown/document export across
  surfaces. This task only audits the research export renderer correctness
  inside `research.rs`, not the general output/export pipeline.
- **A-FE-02**: frontend reducer identity, out-of-order event handling for
  tool/task projections. This task reads the analysis/research GUI components
  only for "does the rendered state match the backend contract"; it does not
  audit the React state machine.
- **A-TSK-04 / A-PROJ-01**: TaskRun integration of the analysis subagent and
  project workspace rooting. This task treats `workspace_root` resolution as a
  given.
- Framework `ExploratoryStatisticsTool` numerical correctness — owned by
  F-EXT-03 (which already filed the IQR P1 and the n==1 variance P3). This
  task only verifies the **boundary** between exploratory and formal analysis
  in EKO, not the exploratory tool's arithmetic.

## Inputs

Required repository documents read in full:

- Repository root `AGENTS.md` — framework-vs-application layering gate,
  no-duplicate / single-authority rule, no-panic / UTF-8-safe hard rules,
  multi-mode parity rule, "local assistant, no online threat model" product
  positioning.
- `docs/comprehensive-review/templates/task-report.md`,
  `templates/validation-report.md`, `docs/comprehensive-review/REPORTING.md`.
- `docs/comprehensive-review/TASKS.md` (A-DOM-01 card and F-EXT-03 + A-TOOL-01
  dependencies).
- `echo-agent-cli/docs/2026-07-18-statistical-inference-correctness.md` — the
  M12 spec that fixes the exploratory/formal contract.
- `echo-agent-cli/docs/MASTER-PLAN.md` M12 milestone entries.

Dependency task reports read:

- `zcode-glm/tasks/F-EXT-03.md` (complete) — established that the four research
  search tools do not classify transient HTTP failures as `ToolFailure::Transient`
  (F-EXT-03-P3-02) and do not paginate (F-EXT-03-P2-01). This task relies on
  both conclusions when reasoning about connector failure behavior.
- `zcode-glm/tasks/A-TOOL-01.md` (complete) — established that `run_code` is
  gated by `configure_run_code_capability` against `has_local_os_sandbox` and
  is removed rather than degraded when the OS sandbox is unavailable, and that
  the agent sandbox is built once via `SandboxManager::local_sandbox()`. This
  task relies on that conclusion for the analysis execution path.

Historical documents treated as hypotheses:

- M12 spec claims (a) `exploratory_statistics` is descriptive-only with
  `inference=false`, (b) formal inference uses reviewable `.py`/`.R` scripts
  through `run_code.script_path`, (c) run records capture input/script hashes,
  package versions, seed, parameters. Each is re-verified under Historical
  Claim Status.

## Layering Decision

This is an **application-layer** task. All analysis/research policy
(file-backed layout, manifest contract, PRISMA/GRADE/risk-of-bias domain
models, citation audit, connector orchestration) lives in
`echo-agent-cli/echo-agent-app-core`. The framework contributes only generic
mechanisms that any consumer needs:

| Classification | Required answer |
|---|---|
| Generic mechanism (framework, retained) | `run_code` `script_path` execution with sandbox/timeout/cancel/path-escape — the single code-execution authority (verified by A-TOOL-01). `ExploratoryStatisticsTool` (descriptive-only, `inference=false`) — the only in-process statistics surface (verified by F-EXT-03). Research provider clients `OpenAlexClient`/`CrossrefClient`/`EuropePmcClient`/`ZoteroClient` + normalized `ScholarlyWork` — generic HTTP clients with SSRF-safe redirect policy and shared `http_error` classification. All are correctly placed and reused. |
| EKO product policy (application) | `analysis/<id>/` file layout with manifest + run records + stale detection; `research/{sources,evidence,reviews,fulltext,reports}/` file layout; `DATA_ANALYSIS` profile prompt that mandates the exploratory/formal boundary; `analyst.md` subagent instructions; `research_library` tool action surface; `install_auto_ingest_tools` wrapping; PRISMA/GRADE/RoB domain models; Pandoc/Quarto renderer discovery; per-analysis cancel via `session.cancel_token`. All are EKO product policy and correctly live in `echo-agent-app-core`. |
| Adapter boundary | The IPC shims in `tauri/commands/{analysis,research}.rs` and the `ResearchLibraryTool` action dispatcher are thin adapters: they translate Tauri/Tool parameters into `analysis::*`/`research::*` free-function calls and map `AnalysisError`/`ResearchError` into `IpcError`. They hold no scheduling authority, no second execution path, no separate state owner. The `AutoIngestResearchTool` wrapper (`research_connectors.rs:283-357`) is a delegating decorator: it forwards `name`/`description`/`parameters`/`permissions`/`risk_level`/`validate_parameters` to the inner tool and only post-processes a successful result. |
| Duplicate search | Searched names across both repos: `run_analysis`/`run_analysis_with_agent`, `create_analysis`, `save_analysis`, `load_analysis`, `list_analyses`, `create_source`/`ingest_source`/`find_matching_source`, `create_review`/`save_review`/`get_review`, `audit_review`, `export_review`/`export_all_review_formats`, `search_and_ingest`, `import_zotero`/`export_zotero`, `enrich_from_europe_pmc`, `install_auto_ingest_tools`, `ResearchLibraryTool`, `AutoIngestResearchTool`, `AnalysisRunRecord`, `SourceRecord`, `ReviewRecord`. Result: no parallel implementation. Each free function in `analysis.rs`/`research.rs` is the single authority; IPC commands, CLI commands, the `research_library` tool, and the auto-ingest wrapper are all thin callers. The frontend `computePrismaFlow` (`ReviewWorkbench.tsx:35-49`) duplicates the backend `prisma_flow` derivation (`research.rs:965-1008`), but it is a pure read-only projection for display and never feeds back into persisted state; flagged under Coverage And Uncertainty. |
| Migration deletion | No migration proposed. No deletion candidate identified in this task. |

## Current Path

Verified data flow at `echo-agent-cli` commit `b3b2e81`:

### Exploratory vs formal analysis boundary (V01)

The M12 spec (`docs/2026-07-18-statistical-inference-correctness.md:5-10`)
fixes two layers and EKO enforces both:

1. **Framework descriptive-only tool.** `ExploratoryStatisticsTool` is the only
   in-process statistics surface (F-EXT-03 verified it returns counts, mean,
   sample stddev, min, quartiles, max, skewness, excess kurtosis and asserts
   `inference=false`; the `hypothesis_test`/`regression`/`descriptive_advanced`
   tools were deleted and a registry test pins their absence). EKO does not
   add a second statistics tool.

2. **Application formal-inference contract.** Formal inference is routed
   through file-backed Python/R scripts executed via the framework `run_code`
   tool. Three independent enforcement points exist:

   - **Subagent instructions** (`subagents/data/analyst.md:15-17`): "Use
     `exploratory_statistics` only for descriptive distribution summaries; it
     is not an inference engine. For hypothesis tests, regression, modeling,
     or custom visualization, first write a reviewable `.py` or `.R` script in
     the assigned `working_dir`, use mature libraries such as
     SciPy/statsmodels or established R packages, then execute that same saved
     file through `run_code` with `script_path`."
   - **Domain profile** (`profiles.rs:213-218`): `DATA_ANALYSIS.prompt_suffix`
     pins "Treat `exploratory_statistics` as descriptive only; formal inference
     must use a persisted SciPy/statsmodels/R script executed through
     `run_code.script_path`." A regression test at `profiles.rs:344-361`
     (`data_profile_separates_exploration_from_formal_inference`) asserts the
     prompt contains `exploratory_statistics`, `SciPy/statsmodels/R`, the
     guidance mentions input hashes and rejects hand-written p-value
     approximations, and the review checklist includes "mature-library script".
   - **File-backed execution path** (`analysis.rs:386-483`): `run_analysis`
     loads the on-disk script, builds a `ToolContext` with
     `working_dir = analysis_dir`, and calls
     `tool_manager.execute_tool_with_context("run_code", parameters, &context)`
     with `script_path = document.manifest.script_path`. There is no second
     Python executor, no statistics DSL, no hand-rolled p-value path.

The boundary is therefore enforced by prompt + profile + tool routing, not by
a runtime state machine — consistent with how Claude Code and Codex enforce
behavioral contracts (per AGENTS.md "关键决策:先调研业界优秀实现").

### Provenance and lineage (V02)

**Analysis side.** `AnalysisRunRecord` (`analysis.rs:170-190`) captures every
input needed for reproducibility:

- `script: AnalysisFileFingerprint` with `sha256` of the executed script
  (`analysis.rs:737-748`, `fingerprint_required` after `load_analysis`).
- `inputs: Vec<AnalysisFileFingerprint>` with per-file `sha256` and
  `available` flag (`analysis.rs:710-735`, `fingerprint_inputs`). Missing
  inputs are recorded as `available: false, sha256: None`, not silently
  dropped.
- `parameters_sha256` (`analysis.rs:900-902`) — canonical JSON byte hash.
- `random_seed`, `environment` (read from `environment.json` the script
  writes), `exit_code`, `sandbox_type`, `output` (bounded to 200 KiB via
  `chars().take`), `output_truncated`.

Stale detection (`analysis.rs:668-708`) re-derives current script/input/
parameters/seed fingerprints and reports the exact reason when any differ
from the last run. Frontend `Lineage` tab (`AnalysisPanel.tsx:541-580`)
renders every field. Run records are written both to
`runs/<run_id>.json` and to `latest-run.json` (`analysis.rs:478-481`), so the
history is durable and the latest is O(1).

**Research side.** `SourceRecord.provenance: Vec<SourceProvenance>`
(`research.rs:91-93, 60-65`) records `provider`, `query`, `retrieved_at`, and
`record_url` for every ingestion path:

- `search_and_ingest` (`research_connectors.rs:367-395`) stamps the query on
  every work via `source_request_from_work`.
- `ingest_tool_output` (`research_connectors.rs:430-483`) stamps the agent
  tool name and the query extracted from the tool's output JSON.
- Zotero import stamps provider `"zotero"` (`research_connectors.rs:135`).
- Manual `create_source` accepts a caller-supplied provenance vector.

Each `EvidenceRecord` carries `source_id` (validated by `upsert_evidence` at
`research.rs:751`) and optional `review_id` (validated at `research.rs:753`).
`audit_review` (`research.rs:1010-1121`) cross-checks every
`review.source_ids` against the sources directory, every evidence's
`source_id`, and flags missing locators, empty claims, included-without-
evidence, and exclusions-without-reason. The audit report is embedded in
every export artifact (`research.rs:1136`), so provenance gaps surface in the
delivered document.

The two systems are independent: analysis provenance is content-addressed
(SHA-256 of files and parameters), research provenance is source-attributed
(provider + query + URL). Both are durable, file-backed, and rendered in
their respective UIs.

### Connector failure handling (V03)

Three connector surfaces, three different policies:

1. **`search_and_ingest`** (`research_connectors.rs:83-112`): a single
   provider is selected per call; its `.search()` is the framework client.
   Any transport, HTTP-non-2xx, or JSON-parse failure returns `Err(...)` via
   `external()` (`research_connectors.rs:539-541`) and propagates as
   `ResearchError::External`, mapped to `IpcError::Internal` at the Tauri
   boundary (`research.rs:34-37`). The agent/user sees a structured error.
   No partial batch — but each call only hits one provider, so the blast
   radius is one query. **Consistent with F-EXT-03-P3-02**: the framework
   clients classify transport errors as plain `ToolError::ExecutionFailed`,
   not `ToolFailure::Transient`, so there is no automatic retry. This is a
   known framework-level gap, not new to this task.

2. **`enrich_from_europe_pmc`** (`research_connectors.rs:188-255`): graceful
   degradation. Each of the four sub-requests (citations, references,
   text-mined terms, full-text XML) is attempted independently; failure
   pushes a human-readable warning and continues with `Vec::new()` / `None`.
   The supplement is persisted with whatever was retrieved plus the warning
   list, and `EuropePmcEnrichmentResult.warnings` is surfaced to the frontend
   (`PaperPanel.tsx:313-317` shows "Enriched with N warning(s)"). See finding
   A-DOM-01-P3-01 for a timestamp-honesty defect in this path.

3. **`AutoIngestResearchTool` wrapper** (`research_connectors.rs:283-357`):
   the wrapper calls the inner tool first; on success it attempts
   `ingest_tool_output`. If ingestion itself fails (file I/O, JSON parse,
   schema mismatch), the wrapper **logs a warning and returns the original
   successful search result unchanged** (`research_connectors.rs:330-332`).
   See finding A-DOM-01-P2-01 for the contract-honesty gap this creates.

The Zotero client paths (`import_zotero`, `export_zotero`) follow the
`search_and_ingest` policy: any framework client failure returns `Err` and
aborts the batch. `import_zotero` does skip malformed items individually
(`research_connectors.rs:127-132`), so a single bad item does not poison the
whole import — graceful at the item level.

### Artifact preservation and rendering (V04)

**Analysis.** `collect_outputs` (`analysis.rs:772-805`) walks the
`outputs/` directory (capped at `MAX_OUTPUT_FILES = 200`, symlinks skipped
to avoid cycles, `sha256` computed for every artifact), classifies by
extension (`chart`/`table`/`report`/`result`/`file`), and persists the list
on the run record. `clear_generated_outputs` (`analysis.rs:750-758`) wipes
`environment.json`, `result.json`, and `outputs/` before each run, so a
failed run cannot inherit a previous successful run's artifacts (verified by
`failed_rerun_does_not_inherit_previous_outputs` at `analysis.rs:1145-1179`).
The frontend renders artifacts with kind + size and offers
`fileSystem.openArtifact(absolute_path)` (`AnalysisPanel.tsx:516-531`).

**Research.** `export_review` (`research.rs:1123-1169`) renders the review
to seven formats (Markdown, JSON, CSV, BibTeX, RIS, plus PDF/DOCX when a
renderer is available). Each format is written atomically to
`research/reviews/<id>/reports/systematic-review.<ext>` and the
`ReviewExportArtifact` records the relative path, byte size, and the full
`CitationAuditReport`. `export_all_review_formats` (`research.rs:1171-1192`)
iterates the available formats and never short-circuits on a single failure
(it `collect()`s, so any error aborts the batch — see finding
A-DOM-01-P3-02 for the silent-skip issue inside `export_review` itself).

Renderer discovery (`research.rs:1194-1343`) probes `EKO_PANDOC` /
`EKO_QUARTO` env vars first, then PATH; `pdf_renderer_available` correctly
reports `false` when Pandoc is present but no PDF engine is. The
`pandoc_renderer_produces_pdf_and_docx_bytes` test at `research.rs:2181-2206`
exercises the renderer with a fixture shell script. See finding
A-DOM-01-P3-03 for BibTeX export correctness.

## Findings

### A-DOM-01-P2-01: `AutoIngestResearchTool` swallows ingestion failures; the agent sees a successful search while sources silently fail to persist

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/research_connectors.rs:306-337` —
    `AutoIngestResearchTool::execute_with_context`:
    ```rust
    let result = self.inner.execute_with_context(parameters, context).await?;
    if result.success {
        match ingest_tool_output(&workspace_root, self.name(), &result.output) {
            Ok(records) if !records.is_empty() => { /* log info */ }
            Ok(_) => {}
            Err(error) => {
                tracing::warn!(tool = self.name(), %error, "research result ingestion failed")
            }
        }
    }
    Ok(result)
    ```
  - `echo-agent-cli/echo-agent-app-core/src/research_connectors.rs:257-281` —
    `ingest_tool_output` itself also silently skips records whose
    `source_request_from_tool_record` returns `None` (e.g., missing title),
    via `.filter_map(...)`.
- Reachability: live. `install_auto_ingest_tools` is called from
  `runtime.rs:286` during agent composition and wraps `arxiv_search`,
  `semantic_scholar_search`, `pubmed_search`, `clinical_trials_search`.
  Every agent-driven research search hits this wrapper.
- Expected invariant: when a tool's documented side effect is "results are
  persisted to the research library" (the wrapper exists precisely to do
  this), a failure to persist should be visible to the caller — either as a
  non-fatal annotation on the returned `ToolResult` or as a structured
  warning. Otherwise the agent proceeds on the false belief that the
  literature is captured.
- Observed behavior: any I/O error, JSON schema mismatch, or
  record-normalization failure inside `ingest_tool_output` is logged at
  `warn!` and the original successful search `ToolResult` is returned
  unchanged. The model receives the full search output (so it can answer the
  immediate query) but has zero signal that the library was not updated.
  Records that fail `source_request_from_tool_record` (e.g., a provider
  payload without a `title` field) are dropped without even a log line.
- Impact: in batch literature-review workflows the agent will not revisit
  failed ingestions and the user will discover missing sources only by
  manually diffing the library against the search output. For a research
  workbench whose explicit value proposition is "results are inspectable,
  versionable, and usable" (`research.rs:3-4`), a silent ingest failure
  undermines the provenance guarantee that V02 verifies at the record level.
- Root cause: the wrapper treats persistence as best-effort and optimizes for
  "never break the agent's flow". That trade-off is defensible, but the
  result must carry the failure signal — the `ToolResult` type has
  `warnings`/metadata channels precisely for this.
- Direction: append a non-fatal warning to the returned `ToolResult` when
  ingestion partially or fully fails (e.g.,
  `result = result.with_warning(format!("ingested {created}/{total}; N failures: {err}"))`
  or set `result.metadata["ingest_failures"]`). Optionally also count
  filter-mapped drops in `ingest_tool_output` and include them. The agent
  then sees "search succeeded, but 3 of 25 results failed to persist: ..."
  and can decide whether to retry.
- Regression validation: a unit test that injects an `ingest_tool_output`
  failure (e.g., point `workspace_root` at a read-only directory) and
  asserts the returned `ToolResult` carries the warning while still
  reporting `success == true`.
- Validation reports: [V03](../validations/A-DOM-01/V03-01.md)

### A-DOM-01-P3-01: `enrich_from_europe_pmc` stamps `enriched_at = now()` even when every Europe PMC sub-request fails

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/research_connectors.rs:188-255` —
    the function unconditionally constructs `EuropePmcSupplement` with
    `enriched_at: Some(Utc::now())` (line 251) and calls
    `save_europe_pmc_supplement`, regardless of how many sub-requests failed.
  - `echo-agent-cli/echo-agent-app-core/src/research.rs:629-649` —
    `save_europe_pmc_supplement` overwrites `source.europe_pmc` with the
    supplement and sets `source.updated_at = Utc::now()`.
  - `echo-agent-cli/echo-agent-app-core/src/research.rs:135` —
    `EuropePmcSupplement.enriched_at: Option<DateTime<Utc>>`.
- Reachability: live. Triggered by the `enrich_europe_pmc` action of the
  `research_library` tool, the `enrich_paper_europe_pmc` Tauri command, and
  the GUI "Enrich selected" button (`PaperPanel.tsx:308-323`).
- Expected invariant: an `enriched_at` timestamp should mean "the enrichment
  succeeded at this time" or be `None`/an explicit failure marker. Stamping
  a fresh timestamp on an empty supplement makes the source appear
  freshly enriched.
- Observed behavior: if Europe PMC is fully unreachable (network down,
  rate-limited, source removed), all four sub-requests push warnings and
  return empty data, the supplement is persisted with empty `citation_ids`,
  `reference_ids`, `biomedical_entities`, `full_text_path = None`, plus
  `enriched_at = Some(<now>)`. The frontend reports "Enriched with 4
  warning(s)" — the user sees a success-flavored message while no data was
  actually retrieved.
- Impact: low for immediate use (the warnings are surfaced and the data is
  visibly empty in the detail view). The defect is timestamp honesty: a
  future audit comparing `enriched_at` against the warning list will see a
  "recent enrichment" with no content and may treat the source as enriched.
- Root cause: the function treats warnings as a soft signal but commits the
  supplement unconditionally; `enriched_at` was not coupled to "did any
  sub-request succeed".
- Direction: gate `enriched_at` on at least one successful sub-request
  (`Some(Utc::now())` only if at least one of citations/references/entities/
  full_text returned data; otherwise `None`). Or, if all four failed, skip
  `save_europe_pmc_supplement` entirely and return only the warnings.
- Regression validation: a test that mocks all four Europe PMC sub-requests
  to fail and asserts the persisted source has `europe_pmc.enriched_at ==
  None` (or the supplement is absent), and that the result still carries
  the four warnings.
- Validation reports: [V03](../validations/A-DOM-01/V03-01.md)

### A-DOM-01-P3-02: `export_review` silently drops missing sources from the rendered output while `audit_review` flags them in the same artifact

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/research.rs:1129-1134`:
    ```rust
    let sources = document
        .record
        .source_ids
        .iter()
        .filter_map(|source_id| get_source(workspace_root, source_id).ok())
        .collect::<Vec<_>>();
    ```
  - `echo-agent-cli/echo-agent-app-core/src/research.rs:1136` — the same
    function then calls `audit_review`, which emits a `missing_source` error
    for every dropped source (`research.rs:1020-1030`).
  - The rendered Markdown/CSV/BibTeX/RIS iterate only the surviving
    `sources` (`research.rs:1416-1430, 1474-1527, 1529-1576`).
- Reachability: live. Every `export_review` and `export_all_review_formats`
  invocation (Tauri command, `research_library` tool, GUI "Export" button).
- Expected invariant: an exported review artifact should be internally
  consistent — the evidence/PRISMA counts and the citation-audit section
  should describe the same set of sources that appear in the rendered
  tables/bibliography.
- Observed behavior: a review with `source_ids = ["src-a", "src-deleted"]`
  produces a Markdown export whose Evidence table and BibTeX omit
  `src-deleted`, but whose Citation Audit section reports
  `Error: Review references missing source src-deleted`. The PRISMA flow
  still counts it in `records_identified` (derived from `source_ids.len()`,
  `research.rs:977`). A reader of the export sees N-1 rows where the audit
  promises N.
- Impact: low — the audit does flag the problem. But the rendered body is
  misleading in isolation (e.g., a BibTeX file with a missing entry that
  the surrounding document still cites).
- Root cause: `export_review` uses `filter_map(...ok())` to be resilient to
  partial library state; the rendered output and the audit were written
  independently and never reconciled.
- Direction: either (a) render a placeholder row/entry for missing sources
  (e.g., a Markdown row `[missing source src-deleted]`, a BibTeX comment
  `% missing source src-deleted`), keeping the artifact consistent with the
  audit; or (b) fail `export_review` with `ResearchError::Invalid` when any
  source is missing (forcing the user to run `audit_review` and fix first).
  Option (a) preserves graceful export; option (b) is stricter.
- Regression validation: a test that exports a review referencing a missing
  source and asserts the rendered output either contains a placeholder for
  the missing id or the call returns `Invalid`.
- Validation reports: [V04](../validations/A-DOM-01/V04-01.md)

### A-DOM-01-P3-03: BibTeX export produces duplicate citation keys for same-author same-year sources and does not escape special characters in field values

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/research.rs:1578-1592` —
    `citation_key`:
    ```rust
    let author = source.authors.first()
        .and_then(|name| name.split_whitespace().last())
        .unwrap_or("source");
    let year = source.year.map(|y| y.to_string()).unwrap_or_else(|| "nd".to_string());
    let raw = format!("{author}{year}");
    raw.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '_').collect()
    ```
    No disambiguation suffix; two "Smith 2025" papers both produce
    `Smith2025`.
  - `echo-agent-cli/echo-agent-app-core/src/research.rs:1529-1546` —
    `render_bibtex` interpolates `source.title`, `source.authors`,
    `source.venue`, `source.doi`, `source.url` directly into `{...}`-delimited
    fields with no escaping. A title containing `}` (e.g.,
    `"Effect of {drug} on ..."` as a literal title) terminates the field
    early and corrupts the `.bib` parse.
- Reachability: live. `export_review(_, Bibtex)` and
  `export_all_review_formats` (which always includes BibTeX,
  `research.rs:1175-1181`).
- Expected invariant: a generated `.bib` file should parse cleanly with a
  standard BibTeX/BibLaTeX parser and have unique keys.
- Observed behavior: duplicate keys silently collide (most parsers keep only
  the last entry); unescaped `{`, `}`, `\`, `&`, `%`, `$`, `#`, `_`, `^`
  inside fields break or subtly corrupt the parser.
- Impact: low for casual single-paper exports; material for systematic
  reviews that export dozens of sources and feed the `.bib` into LaTeX. The
  citation audit (`audit_review`) does not check BibTeX correctness, so the
  user gets no warning.
- Root cause: `render_bibtex` was written as a minimal formatter; neither
  key disambiguation nor LaTeX-special-character escaping was implemented.
- Direction: (a) disambiguate keys with a counter or short hash on collision
  (`Smith2025a`, `Smith2025b`, or `Smith2025-<source.id.suffix()>`); (b)
  escape `\` `{` `}` `&` `%` `$` `#` `_` `^` in field values (replace with
  the LaTeX backslash form, or wrap titles in extra braces for
  case-preservation). The existing `citation_audit_and_all_report_formats`
  test only asserts the file is created, not that it parses.
- Regression validation: a test that exports two same-author same-year
  sources and asserts distinct keys; and a test that exports a source whose
  title contains `}` and asserts the resulting bytes round-trip through a
  BibTeX parser (or at minimum contain `\}`).
- Validation reports: [V04](../validations/A-DOM-01/V04-01.md)

### A-DOM-01-P3-04: `find_matching_source` rescans the entire library on every `ingest_source` call; ingesting a 100-result page does 100 full directory reads

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/research.rs:1602-1636` —
    `find_matching_source` calls `list_sources(workspace_root, None, None)?`,
    which reads and deserializes every `sources/*.json`.
  - `echo-agent-cli/echo-agent-app-core/src/research.rs:376-394` —
    `ingest_page` calls `ingest_source` per work in the page.
  - `echo-agent-cli/echo-agent-app-core/src/research_connectors.rs:257-281`
    — `ingest_tool_output` calls `ingest_source` per record in the tool
    output.
  - `echo-agent-cli/echo-agent-app-core/src/research.rs:1711-1746` —
    `ensure_source_is_unique` does the same full scan, so `create_source`
    (the non-duplicate path) scans twice.
- Reachability: live. Every batch search/import path. Limit is clamped to
  100 (`research_connectors.rs:93`), so the worst case is 100 scans of M
  files = O(N·M) directory reads per `search_and_ingest`.
- Expected invariant: a batch ingestion should not have superlinear cost in
  the existing library size. For a research workbench designed for
  systematic reviews (hundreds to thousands of sources), this becomes
  noticeable.
- Observed behavior: ingesting 50 new sources into a 1000-source library
  performs ~50,000 JSON deserializations on the library, each reading from
  disk. The functions are correct, just slow.
- Impact: low for small libraries; becomes a real latency hit for serious
  systematic-review use. Not a correctness or safety defect.
- Root cause: the dedup and uniqueness checks were written for correctness
  with a simple full-scan, then batch callers were added on top without
  memoizing the scan.
- Direction: build the in-memory lookup once per batch
  (`search_and_ingest`/`ingest_tool_output`/`import_zotero`) — load
  `list_sources` once into a `BTreeMap` keyed by DOI/PMID/arXiv/etc., then
  check membership against the map, inserting as you go. The free-function
  `find_matching_source`/`ensure_source_is_unique` can stay for the
  single-record path; the batch path should use a new
  `find_matching_source_in_index` helper.
- Regression validation: the existing
  `tool_results_are_ingested_idempotently` test covers correctness; add a
  bench or a test that ingests a 50-record page into a 500-source fixture
  library and asserts the number of `sources/*.json` reads is O(M), not
  O(N·M).
- Validation reports: [V02](../validations/A-DOM-01/V02-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Exploratory/formal-analysis boundary: descriptive-only framework tool + script-via-`run_code` formal path enforced by subagent prompt, profile, and tool routing | yes | passed | [V01-01](../validations/A-DOM-01/V01-01.md) |
| V02 | Provenance/lineage: analysis run records capture script/input/parameter/seed hashes; research sources carry provider/query/URL provenance and are cross-checked by `audit_review` | yes | passed | [V02-01](../validations/A-DOM-01/V02-01.md) |
| V03 | Connector failure: `search_and_ingest` propagates errors; `enrich_from_europe_pmc` degrades gracefully; auto-ingest wrapper swallows failures (P2 finding) | yes | passed (with findings) | [V03-01](../validations/A-DOM-01/V03-01.md) |
| V04 | Artifact/rendering: analysis outputs fingerprinted and cleared per run; review export produces seven formats with embedded citation audit (P3 findings on silent drop and BibTeX) | yes | passed (with findings) | [V04-01](../validations/A-DOM-01/V04-01.md) |

No `V05` (historical-document drift) report: the M12 spec claims are
re-verified inside V01 and V04 and classified in the table below, not as a
separate execution.

V03 and V04 are recorded as **passed (with findings)** because the validations
confirm the connector/export paths are reachable and functionally correct;
the associated findings (A-DOM-01-P2-01, -P3-01, -P3-02, -P3-03) are
contract-honesty and scaling gaps observed during the passing inspection, not
validation refutations.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| M12 spec "`exploratory_statistics` is descriptive only, `inference=false`" | current | Boundary enforced at three layers: `analyst.md:15`, `profiles.rs:213-218` + test `profiles.rs:344-361`, and `analysis.rs:420-422` routing through `run_code`. Framework tool verified by F-EXT-03. See V01. |
| M12 spec "formal inference uses reviewable `.py`/`.R` scripts through `run_code.script_path`; no second Python executor" | current | `analysis.rs:386-483` is the single execution path; `run_analysis_with_agent` reuses the agent's `ToolManager`. No DSL, no parallel statistics executor. |
| M12 spec "run records capture input SHA-256, script SHA-256, package versions, seed, parameters" | current | `AnalysisRunRecord` (`analysis.rs:170-190`) captures all; frontend renders them (`AnalysisPanel.tsx:541-580`). |
| M12 spec "delete `hypothesis_test`/`regression`/`descriptive_advanced`; no two inference implementations" | current (framework) | Verified by F-EXT-03 / B-DOC-01; registry pins their absence. |
| MASTER-PLAN:109/382/840 statistics split | current | Same evidence as above. |
| F-EXT-03-P3-02 "research HTTP failures are plain `ToolError::ExecutionFailed`, not `ToolFailure::Transient`" | current (framework) | Re-confirmed in `echo-tools/src/research/clients.rs:754-760` and `arxiv.rs:123-126`. EKO's `search_and_ingest` inherits this behavior via the framework clients. |
| A-TOOL-01 "`run_code` is removed when OS sandbox unavailable; no bare fallback" | current | `analysis.rs:445-453` handles the resulting `Err` from `execute_tool_with_context` and records `AnalysisRunStatus::Failed` with the error message — analysis does not crash when `run_code` is absent. |

## Coverage And Uncertainty

- Code not inspected in depth:
  - The full `ReviewWorkbench.tsx` (1339 lines) — read the
    `computePrismaFlow` derivation (lines 35-49) and the section structure
    (protocol/screening/evidence/quality/prisma/audit). The interactive
    screening/quality form state machines were skimmed, not exhaustively
    line-read. They belong to A-FE-02's frontend-projection scope.
  - The framework research clients' parsing paths
    (`parse_openalex_page`, `parse_crossref_page`, Europe PMC detail
    parsers) — relied on F-EXT-03's coverage. This task only re-verified
    the error-classification paths (`clients.rs:700-768`).
  - The `output/` and `export/` modules — owned by A-OUT-01. This task
    audited only the research-localized `render_review_*` functions.
  - The `PaperDetail.tsx` and `PaperList.tsx` rendering details — read for
    API contract only.
- Validations not executed at runtime beyond the module test suites:
  - `cargo test --lib -p echo-agent-app-core --locked analysis::` — 6/6
    pass (analysis), 0.19s.
  - `cargo test --lib -p echo-agent-app-core --locked research::` — 4/4
    pass (research), 9.55s.
  - No live provider smoke tests run (they are `#[ignore]`-gated behind
    `EKO_PROVIDER_SMOKE` / `ZOTERO_API_KEY` and require network).
- Environmental limits: none. `echo-agent-cli` worktree clean at `b3b2e81`.
- Claims that remain uncertain:
  - The user-visible impact of A-DOM-01-P2-01 (auto-ingest swallow) depends
    on how often ingestion actually fails in practice; the silent-failure
    path is certain, the in-practice frequency is inferred.
  - The BibTeX parser-breakage in A-DOM-01-P3-03 is certain for inputs
    containing `{`/`}`/`\`; whether any real source in the test fixtures
    triggers it is not exercised by the existing test (which only checks
    file creation).
  - Frontend `computePrismaFlow` (`ReviewWorkbench.tsx:35-49`) duplicates
    the backend `prisma_flow` (`research.rs:965-1008`). They agree today on
    the fields they both compute, but `computePrismaFlow` is a pure display
    helper that does not feed back into persisted state, so it is not a
    second authority — just a parallel projection. A divergence would
    produce a UI number that disagrees with the exported artifact; not
    re-audited here.

## Handoff

Conclusions downstream tasks may rely on:

- The exploratory/formal-analysis boundary is **correctly placed and
  enforced** at three layers (subagent prompt, domain profile, tool
  routing). Any downstream task auditing statistics correctness can treat
  `analysis.rs:386-483` as the single formal-inference execution path and
  `ExploratoryStatisticsTool` as the single in-process statistics surface.
- Provenance is **strong and durable** on both sides: analysis run records
  are content-addressed (SHA-256 of script/inputs/parameters), research
  sources are source-attributed (provider/query/URL). The citation audit
  is the single consistency checker for research and is embedded in every
  export. Downstream export/output tasks (A-OUT-01) can rely on the
  `ReviewExportArtifact` shape.
- Connector failures degrade gracefully at the batch level
  (`enrich_from_europe_pmc` collects warnings, `import_zotero` skips bad
  items) with one caveat: the `AutoIngestResearchTool` wrapper swallows
  ingestion errors (A-DOM-01-P2-01). Agent/recovery tasks (F-RCT-*) should
  not assume the agent always knows whether its research search was
  persisted.
- Multi-mode parity for analysis/research is satisfied at the function
  layer (CLI `/analysis` slash command, TUI/channels via the same command,
  GUI Tauri commands, agent `research_library` tool). The CLI path lacks
  per-run cancellation (passes `cancel: None` at
  `cli/cmd_impls/analysis.rs:50`) — acceptable for a foreground slash
  command, but worth noting if a future long-running CLI analysis is added.
- The research library's file layout (`research/{sources,evidence,reviews,
  fulltext,reports}/`) and contract version 1 are stable; downstream tasks
  can rely on them.

Reports downstream tasks must read:

- `zcode-glm/tasks/F-EXT-03.md` (framework research tool contract, including
  the F-EXT-03-P3-02 non-retryable HTTP failure classification that this
  task inherits).
- `zcode-glm/tasks/A-TOOL-01.md` (`run_code` sandbox selection and
  no-bare-fallback, which this task relies on for the analysis execution
  path).
- `echo-agent-cli/docs/2026-07-18-statistical-inference-correctness.md`
  (the M12 spec that fixes the exploratory/formal boundary).

Conditions that make this report stale:

- Any change to `analysis.rs:386-483` (the `run_analysis` execution path),
  `analysis.rs:668-708` (stale detection), or `analysis.rs:170-190`
  (`AnalysisRunRecord` fields).
- Any change to `research_connectors.rs:283-357` (the auto-ingest wrapper)
  or `research_connectors.rs:188-255` (Europe PMC enrichment).
- Any change to `research.rs:1123-1169` (`export_review`), `research.rs:1529-1600`
  (BibTeX/CSV/citation_key renderers), or `research.rs:1602-1746`
  (`find_matching_source`/`ensure_source_is_unique`).
- Wiring `enriched_at` to success state (would resolve A-DOM-01-P3-01).
- Adding a warning channel to the auto-ingest wrapper (would resolve
  A-DOM-01-P2-01).
- Migrating the framework research clients to `ToolFailure::Transient`
  (per F-EXT-03-P3-02) — would not invalidate this report's conclusions but
  would change the V03 reasoning.

Follow-up task IDs (no fixes implemented in this review):

- A tool-correctness / contract-honesty task should land the auto-ingest
  warning (A-DOM-01-P2-01) and the `enriched_at` gating (A-DOM-01-P3-01).
- A research-export hardening task should resolve the silent-source-drop
  (A-DOM-01-P3-02) and the BibTeX key-collision / escaping defects
  (A-DOM-01-P3-03).
- A scaling task should memoize the library scan in batch ingestion paths
  (A-DOM-01-P3-04).
- Cross-reference to F-EXT-03-P3-02: the framework-level "research HTTP
  failures are not retryable" gap, which EKO inherits; a framework fix
  there would automatically improve `search_and_ingest` resilience.
