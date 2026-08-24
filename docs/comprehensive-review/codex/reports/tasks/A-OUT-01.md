# A-OUT-01: Output formats, export, and file delivery

> Status: complete
> Reviewer: Codex review subagent
> Executor: Codex review subagent
> Accepted by: Codex primary reviewer
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: `echo-agent` dirty and excluded; `echo-agent-cli` clean; only Codex reports written

## Question

Do EKO output profiles and Markdown/document/data export paths retain complete content, artifact lineage, error causes, and consistent availability across surfaces?

## Scope

- App-core output facade, format/profile definitions and research background format selection.
- Canonical-conversation Markdown export and its Tauri/frontend delivery path.
- Systematic-review Markdown/PDF/DOCX/JSON/CSV/BibTeX/RIS renderers, artifacts and external converter boundary.
- File-backed analysis output capture, run history and GUI/shared command projection.
- Concrete GUI, TUI, CLI and channel registration/caller parity for these workflows.
- Adjacent tests inspected statically; no test/build/subprocess was executed.

## Out Of Scope

- Canonical transcript persistence defects owned by A-STATE-01.
- General Tool event/artifact delivery, especially the channel Tool-event drop, owned by A-TOOL-01.
- Framework Web/media/data/research Tool output and provenance defects owned by F-EXT-03.
- Prepared attachment ingestion owned by A-INP-01 and shared chat terminal semantics owned by A-CHAT-01.
- SQLite, framework public-API deletion, source fixes, shared indexes and dynamic quality gates.
- All dirty `echo-agent` source files; no new framework-source conclusion is made.

## Inputs

- Root `AGENTS.md`; shared review README, REPORTING, TASKS exact A-OUT-01 card; Codex README.
- Authorized dependencies [A-STATE-01](A-STATE-01.md), [A-TOOL-01](A-TOOL-01.md) and [F-EXT-03](F-EXT-03.md), read only for contract ownership and de-duplication.
- Current clean `echo-agent-cli` source and adjacent tests. No other reviewer directory was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Framework Tool artifacts, complete-output continuation and generic external-process cancellation remain framework mechanisms; reasonable framework export APIs are not dead merely because EKO does not call them. |
| EKO product policy | Which EKO response/review/conversation/analysis formats exist, where exports are stored, immutable product lineage, user-facing batch outcomes and surface parity belong to the application. |
| Adapter boundary | Tauri/CLI/TUI/channel adapters select a typed application export request, invoke one service and project the same artifact IDs, hashes, failures and delivery actions without re-rendering content. |
| Duplicate search | Searched `OutputFormat`, `OutputConfig`, `output_format`, `ResearchOutputFormat`, `format_response`, `LatexExporter`, review/conversation/analysis export names, renderer discovery, artifact/hash/revision fields, `/papers`, `/analysis`, Tauri commands and frontend consumers across both repositories. |
| Migration deletion | Establish one live application export request/result contract, then delete the inert generic output facade, free-form profile field, ignored research enum and duplicate dormant conversation renderer that remain unowned after caller cutover. Preserve unrelated reasonable framework APIs. |

No SQLite dependency or online-service permission gate is proposed.

## Current Path

```text
conversation
  canonical ConversationStore -> Tauri export_conversation
  -> role + text-only Markdown JSON response -> endpoints.ts wrapper -> no component caller

systematic review
  GUI / CLI / channel / research Tool -> export_review[_all]
  -> load ReviewDocument + sources + evidence + audit
  -> Markdown / JSON / CSV / BibTeX / RIS
     or synchronous Pandoc/Quarto -> fixed reports/systematic-review.<ext>
  -> {review_id, format, path, bytes, citation_audit}
  -> GUI path text / CLI-channel JSON text
  TUI: no direct /papers workflow

analysis
  GUI or shared CLI/TUI/channel /analysis -> run_code
  -> delete shared latest outputs -> execute -> collect at most 200 paths
  -> unique runs/<run-id>.json containing hashes but shared mutable paths
  -> GUI openArtifact or text path listing on other surfaces

format/profile declarations
  OutputFormat/OutputConfig -> no production format_response caller
  Profile.output_format -> no production read
  ResearchOutputFormat::Latex -> ignored by to_prompt
```

Positive facts:

- Analysis artifacts record path, byte count and SHA-256, and its GUI offers a real open-artifact action.
- Individual PDF/DOCX absence and non-zero converter failures retain useful causes; stderr bounding is UTF-8-safe.
- CSV quoting and Markdown table pipe/newline handling are explicit; existing analysis Unicode path tests cover a useful narrow case.

## Findings

### A-OUT-01-P1-01: Conversation export drops canonical structured messages and has no live delivery consumer

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent-cli/src/tauri/commands/conversations.rs:643-676`; `echo-agent-cli/web-frontend/src/api/endpoints.ts:437-440`; `echo-agent-cli/src/tauri/mod.rs:217`; `echo-agent-cli/echo-agent-app-core/src/persistence.rs:286-320`.
- Reachability: the Tauri command is registered and reads the canonical conversation store, but the frontend only wraps it and never calls it; there is no CLI/TUI/channel counterpart. A second dormant persistence helper renders a different richer shape.
- Expected invariant: exporting a conversation retains the canonical message sequence, IDs/timestamps, multimodal attachments, Tool calls/results and complete-artifact references, then provides an actual save/download/open result.
- Observed behavior: the registered command emits only role and optional text. Tool-only assistant messages become empty headings; Tool result metadata and attachment/artifact facts disappear. It returns in-memory content with no user-facing consumer.
- Impact: even after canonical persistence is correct, an exported conversation is incomplete and EKO users cannot invoke the declared GUI export workflow.
- Root cause: export was implemented as an ad hoc text projection and IPC stub instead of a consumer of the canonical message projection plus file-delivery service.
- Direction: create one lossless versioned conversation export projection and one typed artifact result; wire it to every surface. Remove the dormant parallel renderer after its unique useful fields are migrated.
- Regression validation: multimodal Unicode conversation with message identity, Tool pair and large-output artifact; field-level export/import inspection and GUI/TUI/CLI/channel availability.
- Validation reports: [V02](../validations/A-OUT-01/V02-01.md), [V03](../validations/A-OUT-01/V03-01.md), [V09](../validations/A-OUT-01/V09-01.md)

### A-OUT-01-P1-02: Review artifacts overwrite one path without revision, hash or renderer lineage

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/research.rs:426-433,518-524,1123-1168`.
- Reachability: GUI, `/papers export` on CLI/channel and the registered research Tool all call the same live export functions.
- Expected invariant: an exported artifact is bound to the exact `ReviewDocument.revision`, output bytes and renderer identity, and an older artifact identity never resolves to newer bytes.
- Observed behavior: every export overwrites `reports/systematic-review.<ext>` and returns only review ID, format, path, bytes and audit. Revision, SHA-256, generation time and converter/version are absent.
- Impact: after review edits or another export, stored responses and audit trails cannot prove which review state produced a PDF/DOCX/Markdown file; the same path silently changes identity.
- Root cause: latest-output filenames are being used as durable artifact identity while the revision already present in the source document is discarded at the boundary.
- Direction: store revision-addressed immutable artifacts and return source revision, content hash, generated time and renderer metadata; retain an explicit latest pointer only as a projection.
- Regression validation: export two revisions and two converter versions, reopen every old artifact, verify bytes/hash/revision, and test retention/deletion policy.
- Validation reports: [V04](../validations/A-OUT-01/V04-01.md), [V09](../validations/A-OUT-01/V09-01.md)

### A-OUT-01-P1-03: External conversion is unbounded and `all` cannot report skipped or partially written formats

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/research.rs:1171-1191,1215-1248,1251-1323`; `echo-agent-cli/src/tauri/commands/research.rs:241-253`; `echo-agent-cli/web-frontend/src/components/papers/ReviewWorkbench.tsx:155-167,350-376`.
- Reachability: selecting PDF, DOCX or All in the GUI and equivalent CLI/channel export commands synchronously enter renderer discovery and execution.
- Expected invariant: converter probe/render is cancellable and time-bounded; batch output identifies every requested format as produced, skipped or failed and retains already-produced artifact results on partial failure.
- Observed behavior: `status()`/`output()` have no timeout/cancel and execute synchronously. All silently omits DOCX/PDF when unavailable, while a later converter failure returns one error after earlier fixed files were already overwritten, losing the partial success set.
- Impact: one hung converter can stall an interactive export indefinitely; “All formats” can report success while omitting document formats or report only failure despite persistent partial side effects.
- Root cause: raw subprocess discovery/execution and sequential file mutation sit behind a `Result<Vec<Artifact>>` contract that has no requested/skipped/failed states.
- Direction: use the bounded cancellable process primitive, preflight a typed capability result, render into a batch staging area, and return a per-format outcome before atomically publishing successful artifacts.
- Regression validation: missing binary/engine, spawn error, non-zero, missing output, hang, cancel and late-format failure with exact batch manifest and cleanup checks.
- Validation reports: [V05](../validations/A-OUT-01/V05-01.md), [V09](../validations/A-OUT-01/V09-01.md)

### A-OUT-01-P1-04: Human-readable review and bibliography renderers lose evidence and citation identity

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/research.rs:1128-1151,1361-1428,1529-1595`.
- Reachability: Markdown is the source for Markdown/PDF/DOCX exports; BibTeX and RIS are live selectable formats on GUI, CLI and channel.
- Expected invariant: review documents retain material evidence and references, and bibliography entries have unique stable Unicode-safe keys and escaped values.
- Observed behavior: the human-readable report omits evidence excerpt/effect/limitations and other fields retained in CSV and has no reference list. Linked source load failures are silently filtered from render input. BibTeX keys are ASCII-filtered first-author surname plus year without disambiguation, so same-author/year records collide and non-ASCII names collapse; values are unescaped. RIS accepts embedded newlines as new control-looking lines.
- Impact: exported reports can omit the evidence needed to audit a claim, while valid multilingual libraries can produce ambiguous or syntactically corrupted citation files.
- Root cause: each renderer hand-picks fields and invents identity/escaping rather than consuming one complete normalized export document and format-aware serializer.
- Direction: build a versioned complete review export document first, make human formats intentional projections with completeness metadata, use stable source-ID-derived disambiguated keys, and use proven format serializers/escaping.
- Regression validation: all EvidenceRecord fields, missing linked source, same-author/year, CJK-only authors, braces/backslashes/newlines, and round-trip parse through independent Markdown/BibTeX/RIS consumers.
- Validation reports: [V06](../validations/A-OUT-01/V06-01.md), [V09](../validations/A-OUT-01/V09-01.md)

### A-OUT-01-P1-05: A new analysis run destroys the artifacts referenced by immutable old run records

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/analysis.rs:160-190,395-481,750-769`.
- Reachability: every GUI, CLI, TUI or channel analysis run enters the same `run_analysis` path and clears outputs before Tool execution.
- Expected invariant: `runs/<run-id>.json` permanently resolves the exact output bytes whose hashes it records, including after later success, failure or cancellation.
- Observed behavior: unique run records point to shared `environment.json`, `result.json` and `outputs/` paths. The next run deletes these before execution, so old paths become missing or later identify different bytes; only hashes remain.
- Impact: users cannot open, compare or reproduce prior analysis outputs despite an apparently immutable run history and SHA-256 lineage contract.
- Root cause: immutable metadata and mutable latest-output storage have different identity scopes.
- Direction: execute/publish under `runs/<run-id>/artifacts`, record those immutable paths, and project/copy a latest view separately. Migrate or explicitly classify existing dangling records.
- Regression validation: two successes plus failed/cancelled reruns; resolve and hash every artifact from every historical record before and after retention cleanup.
- Validation reports: [V07](../validations/A-OUT-01/V07-01.md), [V09](../validations/A-OUT-01/V09-01.md)

### A-OUT-01-P1-06: Analysis capture silently omits output files and can falsely claim complete console text

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/analysis.rs:32,426-443,772-827,904-905`.
- Reachability: every completed analysis run collects files and stores bounded console text in its durable record; all surfaces render that record.
- Expected invariant: every completeness boundary reports total/retained counts and truncation, and deterministic selection makes the manifest reproducible.
- Observed behavior: filesystem traversal stops at 200 before sorting, records no omitted count/flag, and depends on `read_dir` order. A second application `chars().take` bound can remove console characters while `output_truncated` remains the upstream Tool flag.
- Impact: a succeeded run can present a complete-looking but non-repeatable subset of files and console evidence, undermining analysis reproducibility across surfaces.
- Root cause: caps are implementation details rather than typed manifest facts, and nested output bounds do not compose their completeness flags.
- Direction: collect/sort deterministically, expose total/retained/omitted metadata or a complete manifest artifact, and OR every local reduction into one typed truncation reason with continuation reference.
- Regression validation: 201+ multilingual files in varied creation order and upstream-untruncated output above the application cap; assert stable identity, counts, reason and complete continuation.
- Validation reports: [V08](../validations/A-OUT-01/V08-01.md), [V09](../validations/A-OUT-01/V09-01.md)

### A-OUT-01-P1-07: Export actions and file delivery are not equivalent across GUI, TUI, CLI and channels

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent-cli/src/cli/cmd_impls/research.rs:135-144,343-346`; `echo-agent-cli/src/cli/channels.rs:162-171,389-398`; `echo-agent-cli/src/tui/commands.rs:53-115`; `echo-agent-cli/web-frontend/src/components/papers/ReviewWorkbench.tsx:155-167,363-376`; `echo-agent-cli/web-frontend/src/components/analysis/AnalysisPanel.tsx:261-271`.
- Reachability: CLI and channel register `/papers export`; GUI ReviewWorkbench invokes the same service; TUI registers `/analysis` but not `/papers`. GUI analysis artifacts are openable, whereas review artifacts render as path text only.
- Expected invariant: each full-Agent surface can request the same exports and receives the same typed artifact identity plus a surface-appropriate open/download/attachment action.
- Observed behavior: direct research export is absent from TUI; review file delivery in GUI stops at inert comma-separated paths; conversation export is unavailable everywhere despite a registered IPC stub. Analysis is the only inspected workflow with a GUI open action.
- Impact: users must switch interaction modes or manually locate files for core output workflows, contradicting the required surface parity.
- Root cause: commands/components were added independently instead of deriving availability and delivery from one capability/result contract.
- Direction: register one shared application export command/service across surfaces and keep only rendering differences; do not duplicate Tool execution or channel artifact authority owned by A-TOOL-01.
- Regression validation: one conversation, review and analysis export scenario per GUI/TUI/CLI/channel with equivalent IDs, hashes, failures and usable delivery action.
- Validation reports: [V02](../validations/A-OUT-01/V02-01.md), [V09](../validations/A-OUT-01/V09-01.md)

### A-OUT-01-P2-01: Three output-format models are inert or disconnected from execution

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/output/format.rs:8-95`; `echo-agent-cli/echo-agent-app-core/src/output/mod.rs:30-119`; `echo-agent-cli/echo-agent-app-core/src/profiles/types.rs:21-47`; `echo-agent-cli/echo-agent-app-core/src/tasks/background.rs:118-128,160-174`; `echo-agent-cli/echo-agent-app-core/src/export/latex.rs:1-220`.
- Reachability: repository-wide search finds `OutputFormat::format_response` only in its own tests, no CLI `--output` consumer, no production read of profile output format, and no use of the Research `Latex` variant when building its prompt.
- Expected invariant: every public application configuration/type either controls one runtime path or is absent; format selection has one typed authority.
- Observed behavior: a generic output facade, free-form profile string and research Markdown/LaTeX enum coexist without a joined production caller; the standalone LaTeX exporter does not implement the research selection.
- Impact: callers/configuration imply formats that EKO silently ignores, while future changes can wire a fourth authority instead of repairing one existing path.
- Root cause: successive output designs were declared without switching a real caller and deleting the replaced application API.
- Direction: select the canonical export request/result contract, wire real entries first, then delete the unused app-only facade/fields/exporter or implement them through that authority. Do not delete reasonable framework APIs based on EKO usage.
- Regression validation: definition-registration-caller matrix and every supported format from selection through byte signature; zero unused format fields/types after cutover.
- Validation reports: [V01](../validations/A-OUT-01/V01-01.md), [V02](../validations/A-OUT-01/V02-01.md), [V09](../validations/A-OUT-01/V09-01.md)

## Validation Matrix

| ID | Claim or execution | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Commit and dirty-source isolation | yes | passed | [report](../validations/A-OUT-01/V00-01.md) |
| V01 | Format/renderer definition and duplicate search | yes | failed | [report](../validations/A-OUT-01/V01-01.md) |
| V02 | Registration and cross-surface reachability | yes | failed | [report](../validations/A-OUT-01/V02-01.md) |
| V03 | Canonical conversation export completeness | yes | failed | [report](../validations/A-OUT-01/V03-01.md) |
| V04 | Review artifact identity and lineage | yes | failed | [report](../validations/A-OUT-01/V04-01.md) |
| V05 | External converter/batch failure state table | yes | failed | [report](../validations/A-OUT-01/V05-01.md) |
| V06 | Human-readable/Unicode bibliography invariants | yes | failed | [report](../validations/A-OUT-01/V06-01.md) |
| V07 | Historical analysis artifact resolution | yes | failed | [report](../validations/A-OUT-01/V07-01.md) |
| V08 | Large output cap and truncation semantics | yes | failed | [report](../validations/A-OUT-01/V08-01.md) |
| V09 | Existing test coverage inventory | yes | passed inventory | [report](../validations/A-OUT-01/V09-01.md) |
| V10 | Dynamic large/Unicode/converter/surface matrix | future | not run per instruction | [report](../validations/A-OUT-01/V10-01.md) |
| V11 | Framework/application output boundary gate | yes | passed | [report](../validations/A-OUT-01/V11-01.md) |
| V99 | Exact-ID/header/link/source-boundary integrity | yes | passed | [report](../validations/A-OUT-01/V99-01.md) |
| V30 | Primary source-anchor sampling and acceptance | yes | passed | [report](../validations/A-OUT-01/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| A-STATE-01: canonical conversation store retains structured framework transcript | current | The exporter reads that store but discards its structured fields; [V03](../validations/A-OUT-01/V03-01.md). |
| A-TOOL-01: channel drops ordinary Tool lifecycle/artifact events | current | A-OUT-01 owns export workflow availability/result delivery only; general Tool-event repair remains A-TOOL-01. |
| F-EXT-03: specialized framework data/research/media Tool output can be incomplete | current | This report covers EKO conversation/review/analysis adapters; framework Tool corrections remain F-EXT-03. |
| Analysis run records provide durable lineage and artifacts | regressed | Inputs and current outputs have hashes, but historical bytes are deleted and output caps are not represented; [V07](../validations/A-OUT-01/V07-01.md), [V08](../validations/A-OUT-01/V08-01.md). |

## Coverage And Uncertainty

- No Cargo, rustc, test, build, dynamic fixture, converter process, UI session, channel session or network operation ran. [V10](../validations/A-OUT-01/V10-01.md) is future evidence, not a passing result.
- Static call graphs are conclusive for discarded fields, deterministic path overwrite, absence of timeout/cancel, pre-sort cap, missing TUI command and unused format fields. Exact converter starvation and third-party parser diagnostics remain unmeasured.
- Dirty framework source was excluded. Framework artifact/process conclusions come only from authorized dependency ownership, not fresh source inspection.
- This task does not claim that natural-language Tool selection can never export from TUI; it finds the missing direct shared workflow and unequal delivery result.
- Changes to conversation export, review artifact/result schema, analysis output directories/caps, surface command registration or output format types stale this report.

## Handoff

- Preserve one canonical export request/result contract in EKO: source revision, artifact ID/path/hash/bytes/time/renderer, completeness and per-format outcome.
- Fix identity before presentation: immutable review and analysis artifacts make GUI/TUI/CLI/channel delivery straightforward and auditable.
- Treat conversation export as a lossless projection of A-STATE-01's canonical store, not another transcript authority.
- Reuse bounded cancellation/artifact primitives without copying framework Tool execution into EKO adapters; leave A-TOOL-01 and F-EXT-03 findings canonical in their scopes.
- Primary independently reconstructed representative conversation, review,
  analysis, and format-authority findings in V30. Future dynamic validation
  belongs to implementation regressions, Q-E2E-01, and X-SRF-01.
