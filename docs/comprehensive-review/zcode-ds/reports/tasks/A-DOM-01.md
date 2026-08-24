# A-DOM-01: Data analysis and research workflows

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean (both repositories)

## Question

Are EKO-specific analysis/research policies, provenance, formal inference, connectors, workbench state, and artifact export correctly placed and reliable?

## Scope

- `echo-agent-cli/echo-agent-app-core/src/analysis.rs` (full read; 1214 lines)
- `echo-agent-cli/echo-agent-app-core/src/research.rs` (full read; 2207 lines)
- `echo-agent-cli/echo-agent-app-core/src/research_connectors.rs` (full read; 688 lines)
- `echo-agent-cli/echo-agent-app-core/src/research_tool.rs` (full read; 269 lines)
- Registration: `runtime.rs:283-289`; Tauri commands `src/tauri/commands/analysis.rs`, `src/tauri/commands/research.rs`; CLI `src/cli/cmd_impls/analysis.rs` (and research command surface); `tasks/service.rs` + `tasks/background.rs` research-task prompts.
- Frontend: `web-frontend/src/components/analysis/AnalysisPanel.tsx`, `components/papers/{PaperPanel,PaperList,PaperDetail,ReviewMatrix,ReviewWorkbench}.tsx`, `api/endpoints.ts` (analysisApi/papersApi/evidenceApi/systematicReviewsApi types).
- Framework anchors: `echo-tools/src/code.rs` (run_code script_path/sandbox/cancel), `echo-core/src/sandbox.rs` (ExecutionResult semantics), `echo-execution/src/tools.rs:606-625` (execute_tool_with_context), `echo-tools/src/research/*` output schemas, `statistics.rs` (F-EXT-03 re-read only for the boundary).
- Subagent policy: `subagents/data/analyst.md`.

## Out Of Scope

- Framework domain tool correctness (data/statistics/chart/web/database/rag) -> F-EXT-03 (complete; re-read for cross-references only).
- Tool exposure/execution pipeline/sandbox/terminal -> A-TOOL-01 (complete; its analysis-path note is adopted here).
- TaskRuntime execution controller, background pipeline scheduling -> A-TSK-03/A-TSK-04; worktree semantics -> A-TSK-05.
- Export/output module (`output/`), conversation persistence, artifact delivery -> A-OUT-01, A-STATE-01.
- Frontend stores/reducers architecture -> A-FE-01..03 (only the six workbench components inspected).
- Live-network provider behavior — `not_run` (opt-in `#[ignore]` tests, V03-03).

## Inputs

- Root `AGENTS.md` (full), shared `README.md`, `REPORTING.md`, `TASKS.md` (A-DOM-01 card), `zcode-ds/README.md`, report templates.
- Dependency reports read: zcode-ds `F-EXT-03` (complete) and `A-TOOL-01` (complete).
- Historical documents treated as hypotheses: `docs/MASTER-PLAN.md` (M12/M13 claims), `echo-agent-cli/docs/2026-07-18-file-backed-analysis-workbench.md`, `2026-07-18-statistical-inference-correctness.md`.

## Layering Decision

- **Generic mechanism (framework)**: `run_code` `script_path` mode (sandbox/timeout/cancel/path-escape), `ExploratoryStatisticsTool` (descriptive only, `inference=false`), research provider clients (`OpenAlexClient`/`CrossrefClient`/`EuropePmcClient`/`ZoteroClient` + normalized `ScholarlyWork`), search tool output schemas. All correctly placed and reused.
- **EKO product policy (application)**: the file-backed analysis workbench (`analysis/`), the file-backed research library + systematic-review workbench (`research/`), provenance model (`SourceProvenance`), connector orchestration (`search_and_ingest`/`import_zotero`/`enrich_from_europe_pmc`), auto-ingest wrapping, the analyst role boundary policy, export renderers. Correctly placed — these are EKO-local decisions (file-based, workspace-rooted, GUI/TUI/CLI parity).
- **Adapter boundary**: `source_request_from_work`/`source_request_from_tool_record`/`source_to_scholarly_work` convert framework `ScholarlyWork`/tool JSON to `SourceRecord` requests — thin, lossless, no scheduling/state authority; `AutoIngestResearchTool` is an execution-side adapter but carries an undisclosed side effect (P2-01) and its persistence root is the agent working dir (potential divergence from the app workspace root).
- **Duplicate search terms (both repos, V01-01)**: `research_library`; `SourceProvenance`/`provenance`; `research_remember`/`research_recall`; `install_auto_ingest_tools`/`AUTO_INGEST_TOOLS`; `run_analysis`/`create_analysis`/`save_analysis`; `search_and_ingest`/`import_zotero`/`export_zotero`/`enrich_from_europe_pmc`; `PrismaFlow`/`computePrismaFlow`; `export_review`/`ReviewExportFormat`. Results: one definition per concept; no second research store or connector authority; `research_remember`/`research_recall` unused by EKO; frontend `computePrismaFlow` duplicates the backend `prisma_flow` derivation (P3-03).

## Current Path

1. **Analysis workbench**: GUI `AnalysisPanel` -> `analysisApi` -> Tauri `commands/analysis.rs` -> `analysis.rs` (create/save/list/load/run) over `analysis/<id>/` (manifest.json, script, latest-run.json, runs/<run-id>.json, outputs/). `run_analysis` (`analysis.rs:386-483`) fingerprints script/inputs, hashes parameters, clears generated outputs, executes the persisted script through the framework `run_code` tool via the primary agent's ToolManager with `working_dir = analysis/<id>` and a cancel token, then writes an immutable run record and refreshes stale status. Status mapping (`:866-875`) matches `ExecutionResult::success` semantics (exit_code==0 && !timed_out && !cancelled, `echo-core/src/sandbox.rs:277-288`). Fail-closed when `run_code` is absent (`ToolError::NotFound`, `echo-execution/src/tools.rs:612-614`). Note: this path executes the tool directly through the ToolManager, bypassing the 16-stage pipeline (no hooks/audit/visibility stages) — recorded by A-TOOL-01 as a product decision; output truncation is handled locally (MAX_CAPTURE_CHARS).
2. **Research library**: `research/` with `sources/`, `evidence/`, `reviews/<id>/`, `fulltext/`, `reports/`. Source ingestion paths: GUI connectors (`search_scholarly_sources`, `import_zotero_library`, `export_zotero_library`, `enrich_paper_europe_pmc`), agent tool `research_library` (`research_tool.rs`, actions incl. search_sources/import_zotero/export_zotero/enrich_europe_pmc/audit_review/export_review), and the auto-ingest wrapper around `arxiv_search`/`semantic_scholar_search`/`pubmed_search`/`clinical_trials_search` on the primary agent (`runtime.rs:283-289`). Sources carry `SourceProvenance {provider, query, retrieved_at, record_url}`; evidence links to sources (and optionally reviews); reviews carry protocol/screening/RoB/GRADE/PRISMA/medical context; `audit_review` produces a citation audit embedded in every export artifact.
3. **Formal inference boundary**: `analyst.md` restricts `exploratory_statistics` to descriptive summaries and mandates reviewable `.py`/`.R` scripts executed via `run_code(script_path)` under `analysis/<id>/` for tests/models; framework registry pins the statistics split (`echo-tools/src/registry.rs:467-495`).
4. **Export**: `export_review` writes md/json/csv/bibtex/ris (pdf/docx via pandoc/quarto when available) atomically into `reviews/<id>/reports/systematic-review.<ext>` with the citation audit; `export_all_review_formats` probes renderer availability; analysis artifacts are collected with kind/size/SHA-256 (`analysis.rs:772-843`).

## Findings

### A-DOM-01-P2-01: Auto-ingest wraps four framework search tools with an undisclosed write side effect — searches silently persist into the research library and ingest failures are invisible

- Priority: P2
- Confidence: high (mechanism fully traced)
- Layer: application (adapter boundary)
- Evidence: `echo-agent-cli/echo-agent-app-core/src/research_connectors.rs:283-357` (`AutoIngestResearchTool::description` returns the inner framework description verbatim at `:298-300`); side effect at `:311-336` (on `result.success`, parse output and `ingest_tool_output`, returning the original result unchanged); `AUTO_INGEST_TOOLS` at `:23-28`; `install_auto_ingest_tools` at `:359-365`; registration `runtime.rs:283-289` (primary agent only); persistence root `context.working_dir` fallback `env::current_dir()` at `:314-318` vs the GUI/CLI workspace root `state.app_state.workspace.current` (`src/tauri/commands/analysis.rs:17-25`); ingest failures warn-only (`:330-333`).
- Reachability: primary agent in Chat/Task/Auto modes with the research feature (CLI-enabled) -> any model call of `arxiv_search`/`semantic_scholar_search`/`pubmed_search`/`clinical_trials_search` that succeeds persists `research/sources/*.json` records.
- Expected invariant: a tool's LLM-visible description describes its complete behavior including side effects (F-EXT-01/F-EXT-03 honesty contract); the EKO research root is unambiguous and matches the app workspace root.
- Observed behavior: the wrapped tools are described as pure search; a successful search also writes sources into the library with no disclosure; if ingestion fails (including a per-record conflict aborting the batch, since `ingest_tool_output` collects into `Result<Vec<_>>` failing on the first Err, `research_connectors.rs:276-280`), only a `tracing::warn` is emitted and the caller believes nothing happened. The persistence root is the agent working dir, which can diverge from the workspace root used by the GUI, silently splitting research into two locations.
- Impact: the user's research library accumulates records without awareness; the model cannot reason about what is stored (and cannot tell the user); a diverging root (launch cwd, future task-scoped working dirs) makes GUI and agent see different libraries; combined with F-EXT-03-P1-01 (framework memory stubs) the research-persistence story is only as honest as this wrapper.
- Root cause: side-effect wrapping at the tool boundary without updating the description and without a single EKO workspace-root authority; batch ingestion is all-or-nothing.
- Direction: make the side effect explicit — either return the ingest summary in the tool result and mention persistence in the wrapped descriptions, or move auto-ingestion behind an explicit action (e.g., `research_library` action or a documented GUI connector), and resolve the persistence root from the EKO workspace service (same root as `commands/analysis.rs:17-25`) instead of `context.working_dir`; on ingest failure, downgrade per-record instead of all-or-nothing and surface the count in the result.
- Regression validation: unit test wrapping a mock search tool whose output parses, asserting the source file exists AND the returned result text discloses the ingest; a fixture with a conflicting record asserting the remaining records still ingest and the result reports partial success; a test asserting the research root equals the workspace root.
- Validation reports: [V02-01](validations/A-DOM-01/V02-01.md), [V03-05](validations/A-DOM-01/V03-05.md)

### A-DOM-01-P2-02: Analysis rerun deletes the previous run's artifacts before executing — immutable run records then reference files that no longer exist and cannot be regenerated

- Priority: P2
- Confidence: high (code + design doc confirm)
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/analysis.rs:403` (`clear_generated_outputs` deletes `outputs/`, `environment.json`, `result.json` before each run); `:750-770` (removal helpers); run record stores only fingerprints (path/available/bytes/sha256, `:178-190`, `:394-399`) — not the executed script content; runs are written at `:478-481`; the design doc explicitly specifies rebuild semantics (`echo-agent-cli/docs/2026-07-18-file-backed-analysis-workbench.md`, "每次运行会重建契约内的 environment.json、result.json 和 outputs/") while the module doc claims "immutable per-run records" (`analysis.rs:1-6`).
- Reachability: any rerun of an analysis with prior outputs (GUI run button, `/analysis run`); a failed or cancelled rerun leaves `latest-run.json` with an empty outputs list while the prior artifacts were already deleted.
- Expected invariant: immutable per-run lineage records remain actionable — the artifacts and script content they reference are retrievable; a failed rerun must not destroy the last successful run's artifacts.
- Observed behavior: after any rerun, the previous run's `outputs/` files are gone from disk; the runs/<run-id>.json record (the lineage evidence shown in the GUI Lineage tab) references artifact paths and SHA-256s that no longer exist, and the executed script content was never retained — the old result cannot be reproduced or even inspected.
- Impact: silent loss of generated analysis artifacts (charts, tables, reports) on the common edit-script-then-rerun flow; the "immutable run record" is misleading; failed/cancelled reruns destroy the last good outputs, leaving the workbench with no artifacts at all.
- Root cause: the anti-staleness invariant ("failed rerun must not inherit previous outputs") was implemented by deleting files instead of isolating the new run's workspace or archiving the old outputs; fingerprint-only records cannot restore content.
- Direction: archive the prior run's generated outputs (e.g., move `outputs/` to `runs/<run-id>/outputs/` before clearing, or write per-run artifact copies) and/or store the executed script bytes in the run record; keep the no-stale-inheritance invariant but make it non-destructive; update the module doc to describe actual retention.
- Regression validation: fixture running analysis A (produces outputs), modifying the script, rerunning and asserting the first run's artifacts remain accessible via its run record; fixture with a cancelled second run asserting the first run's outputs still exist.
- Validation reports: [V03-02](validations/A-DOM-01/V03-02.md), [V04-01](validations/A-DOM-01/V04-01.md)

### A-DOM-01-P3-01: `delete_source` cascade misses medical guideline references and the full-text file; `audit_review` does not check guideline references

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/research.rs:696-730` (`delete_source` removes the source, its evidence, and review `source_ids`/`screening`/`risk_of_bias` entries, but not `medical.guideline_source_ids` in other reviews and not the orphan `research/fulltext/<source_id>.xml`); `audit_review` (`research.rs:1010-1121`) checks source_ids membership, evidence orphans, empty claims, missing locators, included-without-evidence, exclusion-without-reason — but never `guideline_source_ids`; `save_review` re-validates guideline refs at `research.rs:936-943`.
- Reachability: delete a guideline source that another medical review references via `medical.guideline_source_ids`, then run the citation audit on that review.
- Expected invariant: deletion cascades are complete; the citation audit reports every dangling reference; the review audit is the reliable signal for review integrity.
- Observed behavior: the dangling guideline reference survives deletion; the audit reports 0 errors for it; the next `save_review` fails with a confusing "source not marked as a guideline" error (the source no longer exists); the full-text XML file stays on disk as an orphan.
- Impact: audit-based review workflows can miss a broken medical-context reference; storage of orphaned full-text files; surface behavior that violates the "audit catches dangling references" product promise.
- Root cause: the cascade and the audit were written around `source_ids`/evidence only; the newer `guideline_source_ids` field was not added to either.
- Direction: extend `delete_source` to strip `guideline_source_ids` and remove the full-text file; extend `audit_review` with a `missing_guideline_source` check; add a fixture to the audit test.
- Regression validation: fixture deleting a guideline source referenced by a medical review, asserting the review's `guideline_source_ids` are cleaned, the audit reports the gap before the fix, and no fulltext file remains.
- Validation reports: [V03-02](validations/A-DOM-01/V03-02.md), [V04-02](validations/A-DOM-01/V04-02.md)

### A-DOM-01-P3-02: BibTeX/RIS export rendering defects — citation-key collisions and unescaped special characters break `.bib`/`.ris` artifacts

- Priority: P3
- Confidence: high (code fact; impact depends on data)
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/research.rs:1529-1546` (`render_bibtex` interpolates title/author/journal/year/doi/url unescaped into `@article{...}` — `%`, `{`, `}`, `&` break BibTeX parsing); `:1548-1576` (`render_ris` same unescaped interpolation); `:1578-1592` (`citation_key` = first-author last name + year filtered to ASCII alphanumerics — non-ASCII authors (e.g., Chinese names) yield an empty stem, so keys become bare years; same-author+same-year sources collide).
- Reachability: `export_review(..., Bibtex|Ris)` (GUI export, `/research` CLI surface, `research_library` tool) on a library with non-ASCII authors, `%`/`{}` in titles, or same-author+same-year duplicates.
- Expected invariant: exported citation formats are parseable and keys are unique; rendering escapes payloads (per F-EXT-01 artifact contract).
- Observed behavior: `.bib` files with `%` titles get the remainder commented out; `{`/`}` in titles break entry braces; duplicate keys for colliding author+year; empty stems for non-ASCII authors (the adjacent CSV/Markdown renderers escape correctly — `csv_cell`/`markdown_cell` at `:1594-1600`).
- Impact: bibliography import fails or silently drops/duplicates entries in Zotero/LaTeX — a rendering-fixture defect in the systematic-review artifact pipeline.
- Root cause: string-format renderers without format-aware escaping; key derivation without collision handling.
- Direction: escape BibTeX/RIS fields (or emit via a parser-safe template); derive keys with a disambiguator suffix (e.g., `a`, `b`, or numeric) and a non-ASCII fallback stem (e.g., transliteration or `source` + id fragment).
- Regression validation: fixtures with a Chinese author, a `%`-containing title, and two same-author-same-year sources asserting parseable output and unique keys.
- Validation reports: [V03-04](validations/A-DOM-01/V03-04.md), [V04-02](validations/A-DOM-01/V04-02.md)

### A-DOM-01-P3-03: Frontend duplicates the backend PRISMA derivation — `computePrismaFlow` can drift from the authoritative export computation

- Priority: P3
- Confidence: high (both sides identical today)
- Layer: application (frontend)
- Evidence: `web-frontend/src/components/papers/ReviewWorkbench.tsx:35-52` (`computePrismaFlow` re-implements the formula) vs backend `echo-agent-cli/echo-agent-app-core/src/research.rs:965-1008` (`prisma_flow`, authoritative in `ReviewDocument.prisma_flow` and every export artifact). The frontend even returns its own `prisma_flow` in the document it receives (`get_review` -> `prisma_flow(&record)`, `research.rs:892-905`).
- Reachability: ReviewWorkbench PRISMA section (`ReviewWorkbench.tsx:303` uses `computePrismaFlow` instead of the server-computed `prisma_flow`).
- Expected invariant: frontend consumes backend facts; one derivation authority (A-SRF-03/A-FE-01 contract).
- Observed behavior: two independent implementations of the same derivation; they match today (the fixture `ReviewWorkbench.test.ts` restates the same formula), so any future change to one side silently diverges the displayed flow from the exported flow.
- Impact: display/export divergence risk with no test catching it; duplicated maintenance surface.
- Root cause: convenience re-derivation instead of using the backend-provided `prisma_flow` field.
- Direction: render `document.prisma_flow` directly and delete `computePrismaFlow` (and its fixture), or keep the helper only as a formatting layer over the backend value.
- Regression validation: after removal, a fixture asserting the PRISMA section renders the backend-provided values unchanged.
- Validation reports: [V03-04](validations/A-DOM-01/V03-04.md), [V04-03](validations/A-DOM-01/V04-03.md)

### A-DOM-01-P3-04: `write_full_text_xml` accepts unbounded XML — a large Europe PMC full-text payload is written without a size limit

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/research.rs:651-669` — `write_full_text_xml` only rejects empty input and calls `atomic_write` with the caller's `&str`; the 4 MiB cap (`MAX_JSON_BYTES`, `:24`) applies to JSON reads only; `enrich_from_europe_pmc` passes server-provided XML through (`research_connectors.rs:232-239`).
- Reachability: `enrich_paper_europe_pmc` on a source with a large PMCID full text (full-text XML of a large article or an abnormal server response).
- Expected invariant: every file write path has a size bound consistent with the module's read limits.
- Observed behavior: multi-MB (or larger) XML is written to `research/fulltext/<source_id>.xml` without cap or error.
- Impact: unbounded disk growth from a network response; inconsistent with the bounded read side.
- Root cause: the write path never adopted the read-side limit.
- Direction: enforce a cap (e.g., reject > MAX_JSON_BYTES or a dedicated fulltext limit) and return a warning on truncation, consistent with `write_json` (`research.rs:1978-1986`).
- Regression validation: fixture writing an oversized XML asserting a structured error (or explicit truncation warning) and no oversized file.
- Validation reports: [V03-02](validations/A-DOM-01/V03-02.md), [V03-03](validations/A-DOM-01/V03-03.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search (research_library/provenance/connectors/analysis across both repos) | yes | passed | [V01-01](validations/A-DOM-01/V01-01.md) |
| V02 | Registration and runtime reachability (primary-agent registration, GUI/CLI/TUI entry points, readonly-surface exclusion, run_code status/cancel semantics) | yes | passed | [V02-01](validations/A-DOM-01/V02-01.md) |
| V03 | Exploratory/formal-analysis boundary | yes | passed | [V03-01](validations/A-DOM-01/V03-01.md) |
| V03 | Provenance/lineage invariants (source provenance, evidence linkage, fingerprints, stale, deletion cascade, run-record retention) | yes | passed (gaps -> P2-02, P3-01, P3-04) | [V03-02](validations/A-DOM-01/V03-02.md) |
| V03 | Connector failure handling (search/ingest/enrich/zotero/renderer; live tests not_run) | yes | passed | [V03-03](validations/A-DOM-01/V03-03.md) |
| V03 | Artifact/rendering fixtures (exports, escaping, frontend rendering) | yes | passed (gaps -> P3-02, P3-03) | [V03-04](validations/A-DOM-01/V03-04.md) |
| V03 | Auto-ingest side-effect contract trace (description passthrough, workspace-root resolution) | yes | passed (-> P2-01) | [V03-05](validations/A-DOM-01/V03-05.md) |
| V04 | `cargo test -p echo-agent-app-core --lib --locked "analysis::tests"` | yes | passed (exit 0, 6 passed) | [V04-01](validations/A-DOM-01/V04-01.md) |
| V04 | `cargo test -p echo-agent-app-core --lib --locked "research"` | yes | passed (exit 0, 6 passed, 2 ignored live) | [V04-02](validations/A-DOM-01/V04-02.md) |
| V04 | `npx vitest run` AnalysisPanel + ReviewWorkbench fixtures | yes | passed (exit 0, 2 files/2 tests) | [V04-03](validations/A-DOM-01/V04-03.md) |
| V04 | Live-network provider/Zotero tests | conditional | not_run — opt-in `#[ignore]` with `EKO_PROVIDER_SMOKE=1` / API keys; review is read-only and the reviewed invariants (boundary, provenance, connector-failure classification, artifacts) are statically verifiable | [V03-03](validations/A-DOM-01/V03-03.md) |
| V05 | Historical-document drift (MASTER-PLAN M12/M13, workbench and inference design docs) | yes | passed | [V05-01](validations/A-DOM-01/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN:109/382/840 — statistics split (exploratory only; formal inference via reviewable scripts through run_code) | current | analyst.md boundary; statistics.rs descriptive-only; analysis.rs:420-422 executes run_code; echo-tools/src/registry.rs:467-495 pin test |
| `2026-07-18-file-backed-analysis-workbench.md` — directory contract, run-record contract, rebuild-outputs semantics, GUI+CLI entries, no-SQLite/no-Jupyter non-goals | current | analysis.rs constants :25-33, run record :457-481, clear_generated_outputs :403 (root of P2-02), commands/analysis.rs + cmd_impls/analysis.rs |
| `2026-07-18-statistical-inference-correctness.md` — exploratory_statistics inference=false contract; formal script artifact contract; EKO boundary | current | statistics.rs (F-EXT-03 verified); analyst.md; run records capture fingerprints/environment/seed (analysis.rs:457-477) |
| Framework `research_remember`/`research_recall` stubs (F-EXT-03-P1-01) | current (framework) but not applicable to EKO | zero CLI references (V01-01); EKO research persistence is the file-backed library |
| Framework readonly-surface write tools (F-EXT-03-P1-02) | current (framework) but not applicable to EKO research tools | research_library/auto-ingest are primary-agent-only (V02-01); research_library declares [Read, Write] honestly (research_tool.rs:69-71) |

## Coverage And Uncertainty

- All behavior claims are static; no end-to-end run with a real model or real sandbox was executed (read-only review). The analysis run path is covered by unit tests with a stub `ScriptTool`; the real `run_code` path relies on the framework contract verified in F-EXT-02/A-TOOL-01.
- Live provider behavior (OpenAlex/Crossref/Europe PMC/Zotero) is `not_run` with environmental reason (opt-in keys); connector-failure classification is static.
- `ReviewWorkbench.tsx` (1339 lines) was skimmed for export/audit/PRISMA sections, not exhaustively reviewed; `ReviewMatrix.tsx` skimmed; PaperDetail/PaperList reviewed for data flow only. Frontend store/reducer architecture is A-FE-* territory.
- The background Research/ResearchToWriting task prompts were read but the TaskRuntime execution of research tasks was not re-verified (A-TSK-03/04 scope); the prompts rely on `research_library` presence on the primary agent (verified registration).
- The `run_analysis` pipeline bypass (direct ToolManager execution, no hooks/audit stages) is recorded by A-TOOL-01 as a product decision; not re-raised here.
- `export_review` silently drops missing sources from exported lists; the embedded citation audit flags them, so artifacts stay honest — recorded, not a finding.
- CLI `/analysis run` lacks the GUI's concurrent-run guard (cancel-token map) — two concurrent runs would interleave output writes; recorded as residual (the atomic writes prevent JSON corruption; artifact interleaving falls under P2-02).

## Handoff

- Conclusions downstream tasks may rely on: the analysis/research flows are EKO product policy over framework primitives with correct layering; provenance/lineage design is strong (fingerprints, stale detection, revision conflicts, citation audit, immutable run records) with two integrity gaps (P2-02 artifact retention, P3-01 guideline cascade/audit); the auto-ingest wrapper is the only undisclosed side-effect surface (P2-01); exports are file-backed and atomic with localized BibTeX/RIS rendering defects (P3-02) and a duplicated frontend PRISMA derivation (P3-03).
- Reports to read: this report, its 11 validation reports, and dependency reports F-EXT-03 and A-TOOL-01.
- Cross-references: F-EXT-03-P1-01 does not affect the EKO research flow (unused stubs); F-EXT-03-P1-02 does not extend to EKO research tools (primary-agent-only); A-TOOL-01's analysis-path note (pipeline bypass) remains open for A-PLG-01 (hooks semantics); the PRISMA duplication feeds A-FE-01/02; the artifact-retention gap feeds A-OUT-01/A-STATE-01; the auto-ingest description-honesty issue feeds X-TOL-01 (tool schema/behavior conformance) and X-BND-01 (adapter-boundary classification).
- Conditions that make this report stale: changes to runtime.rs registration (auto-ingest scope), research.rs cascade/audit logic, analysis.rs clear/retention semantics, export renderers, ReviewWorkbench PRISMA usage, or the framework search-tool output schemas.
- Follow-up task IDs: X-TOL-01, X-BND-01, A-OUT-01, A-FE-02, A-PLG-01. Fixes are deferred to the iteration roadmap; this review is read-only.
