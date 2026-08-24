# A-DOM-01: Data analysis and research workflows

> Status: complete
> Reviewer: Codex review subagent
> Executor: Codex review subagent
> Accepted by: Codex primary reviewer
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: `echo-agent` had extensive concurrent changes, all excluded by reading framework files only from `HEAD`; `echo-agent-cli` was clean. Review-report isolation breach is disclosed in [V00-02](../validations/A-DOM-01/V00-02.md).

## Question

Are EKO-specific analysis/research policies, provenance, formal inference, connectors, workbench state, and artifact export correctly placed and reliable under a local personal-assistant threat model?

## Scope

- EKO analysis manifest, input lineage, execution, run records, output artifacts, CLI/channel/Tauri commands, and GUI workbench.
- EKO research source/evidence/review records, scholarly/Zotero/Europe PMC connectors, automatic ingestion, citation audit, PRISMA/GRADE state, report exports, and paper/review GUI.
- Framework HEAD only where required to establish the generic `exploratory_statistics`, `run_code`, scholarly client, Tool, and isolated data-workspace boundaries.
- Data/research TaskRuntime routing, analyst/data-shaper prompt contracts, and durable artifact mapping.

## Out Of Scope

- Generic framework Tool correctness findings owned by `F-EXT-03`.
- Tool allowlist/permission defects owned by `A-TOOL-01`; its analyst impact is referenced, not duplicated.
- General frontend reducer/event parity, Task runtime scheduler correctness, and provider implementation internals.
- Cargo, rustc, tests, builds, dynamic fixtures, and network provider calls, all explicitly prohibited for this task.
- Current uncommitted `echo-agent` contents and diffs.

## Inputs

- `AGENTS.md`, `docs/comprehensive-review/{README.md,REPORTING.md,TASKS.md}`, and `docs/comprehensive-review/codex/README.md`.
- Authorized dependency reports: `F-EXT-03` and `A-TOOL-01`.
- Historical hypotheses: `echo-agent-cli/docs/MASTER-PLAN.md`, `2026-07-18-statistical-inference-correctness.md`, and `2026-07-18-file-backed-analysis-workbench.md`.
- Three non-dependency Codex files were accidentally opened for formatting reference. Their content was discarded and was not adopted; see [V00-02](../validations/A-DOM-01/V00-02.md). This task therefore remains `needs_evidence` until primary independently samples source anchors.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Provider-neutral scholarly clients, exploratory descriptive statistics, canonical `run_code`, Tool contracts, and the `DataWorkspaceFactory` isolation primitive belong in `echo-agent`. These are reasonable framework capabilities regardless of EKO usage. |
| EKO product policy | `analysis/` and `research/` file layouts, formal inference policy, search/review schemas, source lineage, citation audit, PRISMA/GRADE, local workbench state, export formats, and surface behavior belong in `echo-agent-cli`. |
| Adapter boundary | `run_analysis_with_agent` obtains the canonical ToolManager and calls `run_code`; `ResearchLibraryTool` maps one Agent Tool to EKO file services; EKO supplies the generic data-workspace factory. These adapters should convert context/results without owning a second executor. |
| Duplicate search | Searched both repositories by type/field/behavior and callers: analysis/run record/lineage, source/evidence/review/provenance, scholarly/search/ingest, inference/statistics/run_code, workspace/finalize, Tool registration, commands, and frontend consumers. No second file-backed analysis/research store or application code runner was found. Backend and frontend do duplicate PRISMA derivation. |
| Migration deletion | Keep framework public capability menus. Fixes should delete frontend `computePrismaFlow` after it consumes the backend authority, and delete the best-effort auto-ingest wrapper if a single transactional search-and-ingest Tool becomes authoritative. |

## Current Path

Analysis follows `AnalysisPanel` or CLI/channel commands -> Tauri/direct command -> `analysis::{create,save,load,run_analysis_with_agent}` -> primary `ToolManager` -> registered `run_code`. EKO writes `analysis/<id>/manifest.json`, saved Python/R script, shared `outputs/`, immutable-looking `runs/<run-id>.json`, and `latest-run.json`; the GUI renders `last_run` and its output paths.

Formal policy is prompt-driven and correctly separated: framework HEAD `echo-tools/src/statistics.rs:118-128` labels its result exploratory/non-inferential, while `echo-agent-cli/echo-agent-app-core/src/subagents/data/analyst.md:12-17` requires saved SciPy/statsmodels/R code through `run_code`. The inherited `A-TOOL-01-P1-01` blocks this writer Subagent path today, but the primary/workbench service remains reachable.

Research follows paper/review GUI, Tauri commands, or `research_library` (installed at `runtime.rs:283-289`) -> EKO source/evidence/review services -> file records. `research_library.search_sources` couples framework scholarly clients to EKO ingestion; older provider tools are wrapped for automatic ingestion. Reviews derive PRISMA, run a citation audit, and export Markdown/PDF/DOCX/JSON/CSV/BibTeX/RIS.

TaskRuntime data implementation routes to `analyst` (`profiles.rs:121-135`). EKO injects a framework `DataWorkspaceFactory` (`infra.rs:418-426`) that creates and keeps an OS temp directory (`worktree.rs:1391-1464`); framework HEAD appends its non-recursive file listing to Subagent text output (`executor.rs:1857-1880`).

## Findings

### A-DOM-01-P0-01: A rerun deletes the artifact bytes referenced by immutable historical run records

- Priority: P0
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/analysis.rs:160`, `:386`, `:455`, `:478`, `:750`, `:772`
- Reachability: GUI/CLI/channel run -> `run_analysis_with_agent` -> `run_analysis` -> `clear_generated_outputs` -> `run_code` -> per-run JSON.
- Expected invariant: an immutable run record continues to resolve the exact output bytes whose size/hash it records.
- Observed behavior: every run records shared paths under `analysis/<id>/outputs` plus shared `result.json`/`environment.json`; the next run deletes those paths before execution. The old JSON remains and now points to missing or replaced bytes. No run-history list/get/open surface was found.
- Impact: rerunning an analysis irreversibly removes the prior run's reproducibility evidence while leaving metadata that appears auditable.
- Root cause: the implementation snapshots hashes, not artifacts, and conflates a mutable latest-output workspace with immutable evidence storage.
- Direction: promote outputs into `runs/<run-id>/artifacts/` or a content-addressed store before publishing the run record; make latest a reference, expose history, and remove the shared-path claims from historical records.
- Regression validation: execute two runs with different outputs, then open/hash every artifact from both run IDs and restart the service.
- Validation reports: [V04](../validations/A-DOM-01/V04-01.md)

### A-DOM-01-P0-02: Partial Europe PMC refresh overwrites prior enrichment with empty values

- Priority: P0
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent-cli/echo-agent-app-core/src/research_connectors.rs:188`, `:204`, `:211`, `:218`, `:232`, `:243`
- Reachability: paper GUI/Tauri or `research_library enrich_europe_pmc` -> four provider calls -> new supplement -> save over the source.
- Expected invariant: a transient failure in one endpoint preserves the last known data for that dimension.
- Observed behavior: each failed request becomes an empty vector/`None`; the complete fresh supplement replaces the previous supplement and returns success with warnings.
- Impact: retrying enrichment can silently erase stored citation/reference/entity/full-text metadata; a full-text file may remain but its record link is lost.
- Root cause: endpoint failure and a legitimate empty provider response share the same value, followed by replace-all persistence.
- Direction: model per-dimension success/failure and merge only successful dimensions; retain previous values on failure. Provide an explicit reset action if replacement with empty is desired.
- Regression validation: seed all four dimensions, fail each endpoint independently, and assert the failed dimension is retained while successful dimensions update.
- Validation reports: [V08](../validations/A-DOM-01/V08-01.md)

### A-DOM-01-P1-01: Missing declared inputs can execute and later be labeled current

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/analysis.rs:386`, `:668`, `:710`; `echo-agent-cli/web-frontend/src/components/analysis/AnalysisPanel.tsx:443`, `:541`
- Reachability: save manifest with an input path -> any analysis run surface -> fingerprint -> `run_code` -> reload/stale computation -> GUI lineage.
- Expected invariant: formal analysis cannot succeed/current without every required declared input, unless an explicit optional-input contract says otherwise.
- Observed behavior: missing inputs are recorded as `available=false` but execution proceeds. If the same input remains unavailable, current and prior fingerprints match and no stale reason is added. The UI can show `Missing` and `current` simultaneously.
- Impact: a succeeded/current formal analysis may not have consumed its declared dataset, invalidating its lineage and user-facing conclusion.
- Root cause: input fingerprints are treated as observations instead of preconditions; stale compares only change, not validity.
- Direction: require available declared inputs for formal runs, or add explicit required/optional semantics; publish a typed failed run on preflight failure and keep required-unavailable state stale.
- Regression validation: missing-before-run, removed-after-run, optional-missing, symlink, Unicode path, and restore-after-missing cases.
- Validation reports: [V03](../validations/A-DOM-01/V03-01.md)

### A-DOM-01-P1-02: Isolated data Subagent outputs never become durable workbench artifacts

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent-cli/echo-agent-app-core/src/subagents/data/analyst.md:15`, `:16`; `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/worktree.rs:1369`, `:1425`, `:1437`; `echo-agent-cli/echo-agent-app-core/src/infra.rs:418`; framework HEAD `src/agent/subagent/executor.rs:1857`
- Reachability: DataAnalysis implementation PlanTask -> default `analyst` -> workspace-isolated Fork dispatch -> OS temp output -> finalize listing.
- Expected invariant: durable data work is promoted to the selected workspace's canonical `analysis/<id>` artifact authority and downstream consumers receive resolvable artifact identities.
- Observed behavior: each Subagent writes under a distinct kept OS temp directory. Finalization returns only top-level basenames appended to free text; later Subagents get another directory and the main workbench cannot discover temp-local `analysis/<id>`. Kept directories have no cleanup authority.
- Impact: TaskRuntime can claim analysis artifacts that neither a later Subagent nor the GUI workbench can reliably locate, while abandoned data accumulates in the OS temp directory.
- Root cause: collision isolation was implemented without an application promotion/manifest protocol; text listing is being used as artifact transport.
- Direction: keep the generic factory, but add an EKO promotion manifest containing source temp root, artifact IDs/hashes, canonical destination, ownership, and retention. Promote atomically to workspace analysis records, then clean temp state. Delete basename-only downstream assumptions.
- Regression validation: data-shaper -> analyst -> GUI open across distinct Subagent workspaces, including cancellation/crash, recursive outputs, Unicode names, and cleanup.
- Validation reports: [V05](../validations/A-DOM-01/V05-01.md)

### A-DOM-01-P1-03: Systematic-review export lacks a canonical semantic validity gate

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/research.rs:907`, `:965`, `:1010`, `:1123`, `:1748`; `echo-agent-cli/web-frontend/src/components/papers/ReviewWorkbench.tsx:35`
- Reachability: GUI or Agent `save_review` accepts a full record -> raw decisions/assessments persist -> audit/export writes a formal report.
- Expected invariant: formal review state has referential integrity, unique/stage-consistent decisions, possible PRISMA counts, and cannot export as complete when errors remain.
- Observed behavior: validation checks identity/title and a narrow medical requirement. Duplicate/foreign decisions, invalid stage progressions, foreign bias assessments, duplicate/empty GRADE entries, and impossible PRISMA counts pass. Raw rows are counted; frontend duplicates the authority; export proceeds regardless of audit errors.
- Impact: EKO can produce a polished “systematic review” whose study counts and evidence assessments contradict its own records.
- Root cause: serde shape validation, persistence validation, audit, PRISMA derivation, and export policy are separate incomplete authorities.
- Direction: create one backend semantic validator used by save/audit/export, distinguish draft/exploratory from formal/complete, block or unmistakably label invalid formal exports, and delete frontend `computePrismaFlow` after consuming the backend value.
- Regression validation: a table of duplicate, foreign-ID, invalid-transition, impossible-count, empty-GRADE, audit-error export, and valid draft/formal cases.
- Validation reports: [V06](../validations/A-DOM-01/V06-01.md)

### A-DOM-01-P1-04: Automatic scholarly ingestion hides persistence failure behind Tool success

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent-cli/echo-agent-app-core/src/runtime.rs:283`; `echo-agent-cli/echo-agent-app-core/src/research_connectors.rs:283`, `:306`, `:319`; `echo-agent-cli/echo-agent-app-core/src/research_tool.rs:125`
- Reachability: primary runtime wraps live scholarly provider Tools -> successful provider ToolResult -> best-effort EKO ingestion -> unchanged result to ReAct.
- Expected invariant: if a feature promises automatic persistence, the Agent receives typed persisted/failed/partial status and never mistakes retrieval success for saved evidence.
- Observed behavior: malformed output, local I/O error, or record conflict is only logged; the original successful ToolResult is returned unchanged. The explicit `research_library.search_sources` path does propagate errors, leaving two ingestion semantics.
- Impact: background research can cite or report search results as persisted evidence even though the local provenance record was never written.
- Root cause: a best-effort side effect was layered around an otherwise authoritative Tool result without projecting its outcome.
- Direction: converge on one transactional search-and-ingest Tool with structured persistence results. If that replaces automatic wrapping, remove the wrapper and its tests; otherwise make partial failure visible in ToolResult metadata/output.
- Regression validation: malformed provider output, read-only workspace, write failure after N records, duplicate conflict, cancellation, and retry/idempotency.
- Validation reports: [V07](../validations/A-DOM-01/V07-01.md)

### A-DOM-01-P2-01: Research provenance is captured but collapsed and absent from formal human-facing artifacts

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/research.rs:59`, `:1694`; `echo-agent-cli/echo-agent-app-core/src/research_connectors.rs:397`; `echo-agent-cli/echo-agent-app-core/src/research.rs:1010`, `:1361`, `:1474`; `echo-agent-cli/web-frontend/src/components/papers/PaperDetail.tsx:110`
- Reachability: connector search -> `SourceProvenance` -> source merge -> paper workbench/audit/export.
- Expected invariant: every formal search discovery remains linked to its query/search run and is visible/audited in ordinary review artifacts; manual exploratory sources remain permitted.
- Observed behavior: merge deduplicates provenance by provider+URL only, so later distinct query/time discoveries disappear. Paper UI and Markdown/CSV review artifacts do not render provenance; audit does not require or link it to search strategies. Only JSON exposes the raw field.
- Impact: a user cannot reconstruct which query retrieved a study or verify that a reported search strategy produced the included source, despite prompts promising a transparent trail.
- Root cause: provenance is an optional source attribute rather than a retrieval/search-run relation with formal-review requirements and projections.
- Direction: introduce retrieval/search-run identity, retain distinct query/time observations, require it only at the formal review boundary, and render the trail in workbench and reports.
- Regression validation: one source found by multiple queries/providers, manual exploratory source, formal missing-provenance audit, merge round-trip, and Markdown/CSV/GUI projection.
- Validation reports: [V09](../validations/A-DOM-01/V09-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V00-01 | Commit and dirty boundary | yes | passed | [report](../validations/A-DOM-01/V00-01.md) |
| V00-02 | Review-report isolation disclosure | yes | inconclusive | [report](../validations/A-DOM-01/V00-02.md) |
| V01 | Definition, duplicate search, and layering | yes | passed | [report](../validations/A-DOM-01/V01-01.md) |
| V02 | Exploratory/formal boundary and execution reachability | yes | passed | [report](../validations/A-DOM-01/V02-01.md) |
| V03 | Required-input and stale-lineage invariant | yes | failed | [report](../validations/A-DOM-01/V03-01.md) |
| V04 | Immutable run/artifact retention | yes | failed | [report](../validations/A-DOM-01/V04-01.md) |
| V05 | TaskRuntime artifact handoff/promotion | yes | failed | [report](../validations/A-DOM-01/V05-01.md) |
| V06 | Review semantic validity/audit/export | yes | failed | [report](../validations/A-DOM-01/V06-01.md) |
| V07 | Automatic-ingest persistence failure | yes | failed | [report](../validations/A-DOM-01/V07-01.md) |
| V08 | Europe PMC partial failure | yes | failed | [report](../validations/A-DOM-01/V08-01.md) |
| V09 | Provenance capture, merge, audit, and projection | yes | failed | [report](../validations/A-DOM-01/V09-01.md) |
| V10 | Registration and runtime reachability | yes | passed | [report](../validations/A-DOM-01/V10-01.md) |
| V11 | Historical-document drift | yes | passed | [report](../validations/A-DOM-01/V11-01.md) |
| V12 | Existing test-source inventory | yes | passed | [report](../validations/A-DOM-01/V12-01.md) |
| V90 | Targeted executable fixtures | future | not_run | [report](../validations/A-DOM-01/V90-01.md) |
| V99 | Report integrity and source-state gate | yes | passed | [report](../validations/A-DOM-01/V99-01.md) |
| V30 | Independent primary reconstruction after isolation disclosure | yes | passed | [report](../validations/A-DOM-01/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `2026-07-18-statistical-inference-correctness`: framework statistics are exploratory and formal inference uses persisted mature-library scripts | current | [V02](../validations/A-DOM-01/V02-01.md) |
| `MASTER-PLAN`: file-backed analysis workbench and surface registration complete | current | [V10](../validations/A-DOM-01/V10-01.md) |
| `MASTER-PLAN:83`: analysis has immutable `runs/` | regressed | JSON remains immutable, but output bytes are deleted by rerun; [V04](../validations/A-DOM-01/V04-01.md) |
| File-backed workbench claim that inputs/outputs establish reproducibility | regressed | missing input may be current; TaskRuntime output is not promoted; [V03](../validations/A-DOM-01/V03-01.md), [V05](../validations/A-DOM-01/V05-01.md) |
| `MASTER-PLAN`: automatic scholarly ingestion, deterministic citation audit, and report export | current for reachability, partial for reliability | [V07](../validations/A-DOM-01/V07-01.md), [V06](../validations/A-DOM-01/V06-01.md) |
| Workspace prompt requires a transparent research trail | regressed at projection/merge boundary | [V09](../validations/A-DOM-01/V09-01.md) |

## Coverage And Uncertainty

- No executable test, build, provider request, render fixture, PDF/DOCX inspection, or cancellation scenario was run. V90 records this explicit limit; fixes need the cases listed per finding.
- Framework evidence is limited to clean HEAD objects. Current dirty code is neither endorsed nor contradicted.
- Provider clients' internal HTTP/pagination correctness remains owned by `F-EXT-03`; this report covers EKO adapter semantics after a provider result/failure.
- The exact output an LLM may write into its free-text Subagent result is nondeterministic. Finding P1-02 rests on the absence of a structured/canonical artifact mapping, not on a claim that an LLM can never print an absolute path.
- PDF/DOCX generator formatting was inspected only at call topology. The more severe prerequisite issue is that invalid review state reaches every renderer.
- V00-02 remains immutable disclosure. The primary independently reconstructed
  every finding family from current clean CLI source and committed framework
  anchors in V30 before changing status to `complete`.

## Handoff

- Primary independently rebuilt P0-01/P0-02 and sampled P1-01 through P2-01 in
  V30, starting from current source rather than the discarded non-dependency
  reports.
- Downstream synthesis may rely on the layering decision and V01/V02/V10 only after the same source-boundary sampling.
- Do not duplicate `A-TOOL-01-P1-01`; it is a prerequisite for the analyst path, not the durable-artifact defect in P1-02.
- Fixes belong in EKO application/adapters. Keep framework public scholarly, statistics, Tool, and workspace primitives unless a framework-wide review proves replacement.
- This report becomes stale if either reviewed commit changes analysis records, research schemas/connectors/audit/export, Tool registration, TaskRuntime data workspace wiring, or paper/analysis workbench projections.
