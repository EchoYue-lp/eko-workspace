# A-OUT-01: Output formats, export, and file delivery

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa (cross-referenced for framework artifact contract; not modified)
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only review)

## Question

Do EKO output profiles and Markdown/document/data export paths retain
complete content, artifact lineage, error causes, and consistent availability
across surfaces?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent-cli/echo-agent-app-core/src/output/{mod,format,markdown,syntax,table,theme,spinner}.rs`
  (full) — REPL/TUI terminal renderer facade (`OutputRenderer`), the
  `OutputFormat` enum, the Markdown-to-terminal renderer, and friends.
- `echo-agent-cli/echo-agent-app-core/src/export/{mod,latex}.rs` (full) —
  `LatexExporter::markdown_to_latex` and BibTeX generator.
- `echo-agent-cli/echo-agent-app-core/src/research.rs` (490-560, 1120-1345,
  1360-1600, 1988-2007) — `ReviewExportFormat`, `ReviewExportArtifact`,
  `export_review` / `export_all_review_formats`, pandoc/quarto
  `DocumentRenderer`, `render_review_markdown` / CSV / BibTeX / RIS renderers,
  `atomic_write`.
- `echo-agent-cli/echo-agent-app-core/src/research_tool.rs` (full) —
  `ResearchLibraryTool` agent tool exposing `export_review` action and its
  `parse_export_format`.
- `echo-agent-cli/echo-agent-app-core/src/analysis.rs` (125-215, 410-485,
  770-906) — `AnalysisOutputArtifact` (path/bytes/sha256), `output_kind`,
  `bounded_text` (UTF-8 truncation), `MAX_CAPTURE_CHARS`/`MAX_OUTPUT_FILES`.
- `echo-agent-cli/echo-agent-app-core/src/tasks/background.rs` (1-160) —
  `ResearchOutputFormat` (Markdown/Latex), `default_format`.
- `echo-agent-cli/echo-agent-app-core/src/profiles/types.rs` (1-80) —
  `Profile.output_format` field.
- `echo-agent-cli/src/cli/args.rs` (full) and `src/cli/repl.rs` (1-100,
  480-560, 660-860) — CLI flag surface, `OutputRenderer` consumer.
- `echo-agent-cli/src/tui/events.rs` (slash-command enum, 2627-3612) and
  `src/tui/markdown.rs` (1-75) — TUI export surface, TUI Markdown renderer.
- `echo-agent-cli/src/tauri/commands/conversations.rs` (640-725) — Tauri
  `export_conversation` Markdown exporter and `restore_conversation`.
- `echo-agent-cli/src/tauri/commands/research.rs` (230-270) — Tauri
  `export_systematic_review` command and its `parse_export_format`.
- `echo-agent-cli/src/tauri/commands/analysis.rs` (full) — analysis workbench
  IPC (list/get/save/run/cancel), no agent-tool mirror.
- `echo-agent-cli/web-frontend/src/api/endpoints.ts` (425-455, 1255-1276,
  1360-1380) — frontend `conversations.export` and `systematicReviewsApi.export`
  contracts, `ReviewExportFormat` / `ReviewExportArtifact` TypeScript types.
- `echo-agent-cli/web-frontend/src/components/papers/ReviewWorkbench.tsx`
  (140-170, 335-395) — `exportReview` flow, export-format `<select>`.
- `echo-agent/echo-core/src/tools/artifact.rs` (85-145, 145-235) — framework
  `ToolOutputArtifactRef` (path/bytes/sha256/retention),
  `ToolOutputArtifactWriter`, metadata round-trip.
- `echo-agent/echo-core/src/memory/conversation.rs` (60-90) — `StoredMessage`
  fields (`content`, `attachments_json`, `tool_calls_json`,
  `tool_result_json`).

## Out Of Scope

Deferred to downstream/other tasks:

- **A-STATE-01**: owns `Persistence::export_conversation_markdown` (the dead
  application exporter) and the conversation store authority; this task only
  cross-references it for content completeness.
- **A-TOOL-01**: owns the framework `process_tool_output_for_call`
  truncation + spill path and `ToolExecutionRepository` projection. This task
  consumes its conclusion that the model-facing truncation is UTF-8 safe
  (`chars().take()`).
- **F-EXT-01 / F-EXT-03**: own the framework `ToolOutputArtifactRef` /
  `persist_tool_output` primitives and the data/research tool artifact
  surface. This task treats them as the lineage reference.
- **A-FE-02**: frontend projection of tool-execution artifacts (reducer
  identity, pagination). This task audits only the Rust export/delivery
  contract and the TypeScript DTO shapes.
- **A-SRF-01..04 / X-SRF-01**: surface feature parity synthesis. This task
  files per-surface parity gaps but does not synthesize.
- **A-INP-01 / A-INT-01**: MCP/LSP/browser attachment delivery and
  prepared-input artifacts.
- **F-MEM-01**: framework atomic-write recipe. This task only cross-references
  the recurring missing-parent-fsync defect.

## Inputs

- Required repository documents read:
  - Repository root `AGENTS.md` — UTF-8-safe string rule (`chars().take()`,
    never byte slicing), no-panic rule, framework/application layering gate,
    dead-code cleanup rule, multi-mode parity rule, "动手前先查是不是已经有了"
    duplicate-search gate.
  - `docs/comprehensive-review/REPORTING.md`,
    `docs/comprehensive-review/templates/{task-report,validation-report}.md`,
    `docs/comprehensive-review/TASKS.md` (A-OUT-01 card and the
    A-STATE-01 / A-TOOL-01 / F-EXT-03 dependencies).
- Dependency task reports read:
  - **A-STATE-01** (complete) — established that `Persistence` and
    `SessionSearchEngine` are dead application authorities, that the framework
    `FileConversationStore` is the live conversation authority, and that
    `export_conversation` (Tauri) builds Markdown directly off
    `store.get_messages` without using `Persistence::export_conversation_markdown`.
    P2-01 is the directly applicable prior for this task's exporter-dead-code
    audit.
  - **A-TOOL-01** (complete) — established the framework
    `process_tool_output_for_call` truncation path is UTF-8 safe
    (`chars().take()`), the artifact spill threshold (32 KiB) and
    `DEFAULT_MAX_TOOL_OUTPUT_TOKENS = 4000`, and that `ToolExecutionRepository`
    paginates output at 64 KiB. This task relies on those conclusions for V02.
  - **F-EXT-03** (complete) — established that `web_search` / `sql_query`
    spill via `persist_tool_output` and emit `ToolOutputArtifactRef` with
    `truncated = true`; the framework artifact contract is the lineage
    reference for this task's V04 audit.
- Historical documents treated as hypotheses:
  - `output/format.rs:8` docstring "输出格式 (用于 --output / -o 标志)" —
    treated as a claim to re-verify (it is stale; see V01-01 and Historical
    Claim Status).
  - `output/mod.rs:1-8` docstring "`OutputRenderer` 是整个 CLI 输出的唯一外观
    (Facade), REPL 模式和 TUI 模式都通过它输出" — treated as a claim
    (half-stale: TUI does not use it).
  - `export/mod.rs:1-4` "提供多种格式的导出功能：Markdown、LaTeX、JSON 等"
    — treated as a claim (stale: only LaTeX is implemented, and it has no
    production caller).

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism | The framework `ToolOutputArtifactRef` / `ToolOutputArtifactWriter` / `persist_tool_output` (`echo-core/src/tools/artifact.rs`) is the single spill primitive — used by `shell`, `web_fetch`, `sql_query`, and the agent runtime's `process_tool_output_for_call`. Any framework consumer may use it; correctly lives in `echo-core`. This task does not propose to move it. |
| EKO product policy | All terminal rendering (`OutputRenderer`, TUI `markdown.rs`, `output/markdown.rs`, `table.rs`, `theme.rs`, `syntax.rs`, `spinner.rs`), all research/analysis document export (`research.rs` Markdown/PDF/DOCX/JSON/CSV/BibTeX/RIS, `analysis.rs` lineage), the pandoc/quarto adapter, the `Profile.output_format` field, and the `tasks::ResearchOutputFormat` enum are EKO product policy — they depend on EKO's product decisions (workspace-rooted reports, systematic-review domain, pandoc discovery). Correct layer. |
| Adapter boundary | The Tauri command `export_systematic_review` (`src/tauri/commands/research.rs:241`) and the agent tool `ResearchLibraryTool` action `export_review` (`research_tool.rs:164`) are two thin adapters over the same application-core `research::export_review` / `export_all_review_formats`. Conversion is lossless (both pass `review_id` + parsed `ReviewExportFormat` and receive `ReviewExportArtifact`). The Tauri `export_conversation` (`conversations.rs:643`) is also a thin adapter but its projection is lossy (see V01-01 / V04-01). |
| Duplicate search | Terms run across both repositories: `OutputFormat`, `FormatContext`, `FormattedOutput`, `format_response`, `default_format`, `ReviewExportFormat`, `ResearchOutputFormat`, `LatexExporter`, `markdown_to_latex`, `parse_export_format`, `export_conversation`, `export_review`, `export_conversation_markdown`, `document_renderer_available`, `OutputRenderer`, `render_markdown`. Results: (a) **three distinct format enums** (`output::OutputFormat`, `research::ReviewExportFormat`, `tasks::ResearchOutputFormat`) — different concepts but overlapping names; (b) **two `parse_export_format` helpers** (`research_tool.rs:220` and `tauri/commands/research.rs:257`) — true duplicate logic; (c) **two Markdown renderers** in the application (`output/markdown.rs` for REPL, `tui/markdown.rs` for TUI) plus the framework's spill-time truncation — different surfaces, not a duplicate; (d) **dead exporters**: `LatexExporter`, `Persistence::export_conversation_markdown`, `output::format::OutputFormat::format_response`. |
| Migration deletion | Recommended deletion targets: (1) `output/format.rs` `OutputFormat`, `FormatContext`, `FormattedOutput`, `format_response`, `OutputConfig.default_format`, `set_default_format` (A-OUT-01-P2-01); (2) `export/latex.rs` `LatexExporter` + `export/mod.rs` re-export (A-OUT-01-P2-02); (3) `tasks::ResearchOutputFormat::Latex` variant + the `output_format` field on `BackgroundTaskKind::Research` if confirmed unused (A-OUT-01-P3-02); (4) `Profile.output_format` field (A-OUT-01-P3-03); (5) consolidate the two `parse_export_format` helpers (A-OUT-01-P3-01). |

## Current Path

### Output format surface — three enums, one live registry

The application carries three format enums plus several ad-hoc format
strings. Verified at `echo-agent-cli` `b3b2e81`:

1. **`output::format::OutputFormat`** (`output/format.rs:10-24`):
   `Text | Json | Markdown | Table`, derived `clap::ValueEnum` + `Default =
   Text`. The module docstring (`format.rs:8`) claims it is "用于 --output /
   -o 标志". **There is no `--output` / `-o` flag** (`src/cli/args.rs:12-68`
   enumerates all CLI flags; none match). `format_response`
   (`format.rs:58-99`) and `FormatContext` / `FormattedOutput`
   (`format.rs:28-54`) have **zero production callers** — only the four
   in-module tests at `format.rs:118-138`. The entire `output/` module is
   annotated `#![allow(dead_code)]` at `output/mod.rs:9`, confirming the
   maintainers already know. `OutputConfig.default_format: OutputFormat`
   (`output/mod.rs:42`) is set by `set_default_format` (`output/mod.rs:93`)
   which itself has no caller. See V01-01.

2. **`research::ReviewExportFormat`** (`research.rs:491-501`):
   `Markdown | Pdf | Docx | Json | Csv | Bibtex | Ris`, serde
   `rename_all = "snake_case"`. This is the **live registry**. It is parsed
   from the wire string by two duplicate helpers:
   `research_tool.rs:220` (`parse_export_format`, agent path) and
   `tauri/commands/research.rs:257` (`parse_export_format`, GUI path). Both
   feed the same `export_review` / `export_all_review_formats`
   (`research.rs:1123, 1171`). See V01-01.

3. **`tasks::ResearchOutputFormat`** (`tasks/background.rs:116-122`):
   `Markdown | Latex`, `rename_all = "lowercase"`, `#[default] = Markdown`.
   Carried by `BackgroundTaskKind::Research { output_format, .. }`
   (`background.rs:21-27`). Every constructor site
   (`background.rs:251`, `tasks/service.rs:899`,
   `tauri/commands/tasks.rs:109`, `cli/cmd_impls/{research,coding,pipeline}.rs`)
   passes `ResearchOutputFormat::Markdown` or `Default::default()`. The
   `Latex` variant has **zero construction sites**. See V01-01.

4. **`Profile.output_format: String`** (`profiles/types.rs:22-23`) with
   `default_output_format() -> "text"` (`profiles/types.rs:46-48`). Written
   by `Profile::new` (`profiles/types.rs:63`). Grep for `.output_format`
   across the application returns only the writers inside `profiles/types.rs`
   and the unrelated `tasks/background.rs` field; **zero read sites**.

5. **Ad-hoc Markdown** (`tauri/commands/conversations.rs:643-677`): the
   `export_conversation` command hand-rolls a Markdown string from
   `store.get_messages`. It is not part of any registry and does not consult
   any of the above enums.

### Terminal renderers — two parallel implementations

- **REPL** uses `OutputRenderer` (`output/mod.rs:29-349`), instantiated only
  at `src/cli/repl.rs:96`. Methods used: `print_banner`,
  `print_session_info`, `print_user_message`, `print_assistant_prefix`,
  `print_token`, `print_tool_call`, `print_tool_result`, `print_error`,
  `print_warning`, `print_info`, `print_success`, `start_spinner`,
  `print_separator` (`repl.rs:98-856`). The REPL itself is a **hidden**
  entry: `args.rs:26` `--cli` (`hide = true, default_value_t = false`).
- **TUI** uses `crate::tui::markdown::render_markdown`
  (`src/tui/markdown.rs:53`) which builds `ratatui::text::Line` via a
  separate `MarkdownRenderer` state machine. It does **not** go through
  `OutputRenderer`. The `output/mod.rs:1-8` doc claim "REPL 模式和 TUI 模式都
  通过它输出" is half-stale.
- **GUI** renders Markdown in the frontend (React); the backend only ships
  content strings.

So three surfaces have three renderers; the supposed "唯一外观 (Facade)" is
bypassed by TUI and GUI.

### Document export pipeline (research)

Single application-core authority at `research.rs`:

`export_review(workspace_root, review_id, format)`
(`research.rs:1123-1169`) → `get_review` + `list_sources` + `list_evidence`
+ `audit_review` → `render_review_markdown` (`research.rs:1361-1472`) →
format-specific encoding:

- `Markdown` → `markdown.into_bytes()`.
- `Json` → `serde_json::to_vec_pretty` of `{review, sources, evidence,
  citation_audit}`.
- `Csv` → `render_evidence_csv` (`research.rs:1474-1527`).
- `Bibtex` → `render_bibtex` (`research.rs:1529-1546`).
- `Ris` → `render_ris` (`research.rs:1548-1576`).
- `Pdf` / `Docx` → `render_review_document` (`research.rs:1251-1324`),
  which writes the markdown to a temp file and shells out to pandoc or
  quarto.

Output is written via `atomic_write(path, &bytes)`
(`research.rs:1156` → `research.rs:1988-1999`) to
`<workspace>/.eko/research/reviews/<review_id>/reports/systematic-review.<ext>`.
`ReviewExportArtifact` (`research.rs:517-524`) returns `review_id`,
`format`, workspace-relative `path`, `bytes`, and the `citation_audit`. It
**does not** include a content hash (see V04-01).

`export_all_review_formats` (`research.rs:1171-1192`) builds the format list
conditionally: Markdown/JSON/CSV/BibTeX/RIS unconditionally, DOCX only when
`document_renderer_available()`, PDF only when `pdf_renderer_available()`.
This is the single source of truth for which formats are available.

### External converter discovery (V03)

`resolve_document_renderer` (`research.rs:1215-1233`) returns
`Option<DocumentRenderer>`:

1. `EKO_PANDOC` env var if it points to an executable.
2. `pandoc` on PATH.
3. `EKO_QUARTO` env var if it points to an executable.
4. `quarto` on PATH.
5. `None` otherwise.

`executable_available` (`research.rs:1241-1249`) spawns `<bin> --version`
with `Stdio::null` on stdout/stderr and checks `status.success()`.
`preferred_pdf_engine` (`research.rs:1326-1343`) honors `EKO_PDF_ENGINE`,
else probes `[typst, weasyprint, wkhtmltopdf, xelatex, lualatex, pdflatex]`.

When no renderer is present:

- `render_review_document` (`research.rs:1251-1257`) returns
  `ResearchError::External("PDF/DOCX export requires Pandoc or Quarto on
  PATH; EKO_PANDOC/EKO_QUARTO may point to a custom executable")`.
- When pandoc is present but no PDF engine: `render_review_document_with_renderer`
  (`research.rs:1285-1289`) returns `ResearchError::External("Pandoc PDF
  export requires typst, weasyprint, wkhtmltopdf, xelatex, lualatex, or
  pdflatex; EKO_PDF_ENGINE may select another supported engine")`.
- Renderer stderr is captured via `String::from_utf8_lossy(&output.stderr)
  .chars().take(2_000).collect()` (`research.rs:1313-1316`) — UTF-8 safe,
  bounded.

These are structured errors with actionable messages; the agent sees them
via `research_tool.rs:189` `Ok(result.unwrap_or_else(|error|
ToolResult::error(error.to_string())))` and the GUI sees them via the
`IpcError` mapping. No panic, no silent fallback to a degraded format.

### Conversation Markdown export (Tauri)

`export_conversation` (`tauri/commands/conversations.rs:643-677`) is the
**only** live conversation exporter. It calls `store.get_conversation` +
`store.get_messages` and writes:

```
# <title>

## <role>

<content>

```

Only `role` and `content` from each `StoredMessage` are emitted. The
remaining `StoredMessage` fields — `attachments_json`,
`tool_calls_json`, `tool_result_json`, `created_at`
(`echo-core/src/memory/conversation.rs:65-82`) — are **dropped**. The dead
`Persistence::export_conversation_markdown` (`persistence.rs:286-318`, per
A-STATE-01 P2-01) is richer (it includes title, timestamps, model, and
`tool_calls`), but it is dead and reads a different in-memory
`ConversationRecord` shape. See V01-01.

### Artifact lineage

Two artifact shapes are produced by EKO exports:

- **`AnalysisOutputArtifact`** (`analysis.rs:160-167`): `path`,
  `absolute_path`, `kind`, `bytes`, **`sha256`**. Computed by `hash_file`
  (`analysis.rs:877-894`) over the file content. Strong lineage.
- **`ReviewExportArtifact`** (`research.rs:517-524`): `review_id`, `format`,
  `path`, `bytes`, `citation_audit`. **No content hash.**

The framework reference is `ToolOutputArtifactRef`
(`echo-core/src/tools/artifact.rs:91-98`): `path`, `artifact_bytes`,
`payload_bytes`, **`sha256`**, `retention` — the strongest lineage contract,
round-tripped through `extend_metadata` / `from_metadata`
(`artifact.rs:100-142`). See V04-01.

### Cross-surface delivery

- **Review export**: agent (`research_library` action `export_review`) and
  GUI (`export_systematic_review` Tauri command) both call
  `research::export_review` / `export_all_review_formats`. Identical
  artifacts. The frontend (`ReviewWorkbench.tsx:155-168`,
  `endpoints.ts:1372-1375`) shows `artifact.path` and
  `artifact.citation_audit`.
- **Conversation export**: GUI-only. No agent tool, no TUI slash command
  (the TUI `SlashCommand` enum at `src/tui/events.rs:2627-3612` has no
  `Export` variant). The agent cannot trigger conversation export.
- **Analysis workbench**: GUI-only (`src/tauri/commands/analysis.rs`). The
  agent reaches analysis scripts through `run_code` (writes to the analysis
  `outputs/` dir) but has no typed `analysis_library` tool to list runs or
  fetch `AnalysisOutputArtifact` lineage.
- **Spilled tool artifacts**: framework-owned. `read_artifact`
  (`echo-tools/src/files/artifact.rs:90`) reads only from
  `ctx.output_artifacts` (the spill root). Review/analysis export files live
  under `<workspace>/.eko/{research,analyses}/` — **outside** the spill
  root — so `read_artifact` cannot page through them; the agent must use
  `read_file` (no pagination, no manifest).

## Findings

### A-OUT-01-P2-01: `output::OutputFormat` / `FormatContext` / `format_response` are dead; the `--output / -o` flag does not exist

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/output/format.rs:8` — docstring
    "输出格式 (用于 --output / -o 标志)".
  - `echo-agent-cli/echo-agent-app-core/src/output/format.rs:10-24` — the
    `OutputFormat` enum with `clap::ValueEnum` derive.
  - `echo-agent-cli/src/cli/args.rs:12-68` — the `Args` struct; no `--output`
    or `-o` short.
  - `echo-agent-cli/echo-agent-app-core/src/output/mod.rs:9` — module-level
    `#![allow(dead_code)]`.
  - `echo-agent-cli/echo-agent-app-core/src/output/mod.rs:42, 60, 93-96` —
    `OutputConfig.default_format`, its `Text` default, and the unused
    `set_default_format` setter.
- Reachability: definition (`output/format.rs:10`) → registration
  (`output/mod.rs:23` `pub use format::{FormatContext, OutputFormat}`) →
  **zero live callers**. The only callers of `format_response` /
  `FormatContext::new` / `FormattedOutput` are the four in-module tests at
  `format.rs:118-138`. Grep for `default_format` outside `output/mod.rs`
  returns only the unrelated `tasks/background.rs::default_format` helper and
  `profiles/types.rs::default_output_format` (both string-typed, different
  concept).
- Expected invariant: per AGENTS.md "动手前先查是不是已经有了" and "代码清理:
  无需兼容, 过时代码可直接删", a public format enum that advertises a CLI flag
  must either be wired to that flag or deleted; a stale docstring claiming a
  flag exists is a falsifiable contract that must be honored or removed.
- Observed behavior: the `OutputFormat` enum and its `format_response`
  machinery compile (via the module-level `#![allow(dead_code)]`) but are
  never invoked. The `--output` flag the docstring promises does not exist in
  `Args`. The REPL (`repl.rs`) uses `OutputRenderer` for terminal styling
  only, never for format selection; it ignores `default_format`.
- Impact:
  - **Misleading API/doc surface.** A new contributor reading `format.rs`
    will reasonably assume EKO has a `--output json|markdown|text|table` flag
    and build features on top of `format_response` instead of the live
    paths. The `clap::ValueEnum` derive reinforces the illusion.
  - **Naming collision.** `OutputFormat` (`output::`) vs `ReviewExportFormat`
    (`research::`) vs `ResearchOutputFormat` (`tasks::`) — three
    format-flavored enums in one application. The dead one muddies search.
  - **No correctness risk** to current users — the live paths
    (`export_conversation`, `export_review`) do not consult it.
- Root cause: an earlier iteration planned a `--output` flag and built the
  enum + `clap::ValueEnum` + `format_response`, but the flag was never added
  to `Args`; the REPL and TUI took over rendering via `OutputRenderer` /
  `tui::markdown` instead. `#![allow(dead_code)]` suppressed the warning that
  would have reminded someone to delete it.
- Direction: delete `output/format.rs` entirely (the enum, `FormatContext`,
  `FormattedOutput`, `format_response`, and the four tests); drop
  `pub mod format;` from `output/mod.rs`; drop the `default_format` field
  from `OutputConfig` and the `set_default_format` method. The remaining
  `output/` submodules (`markdown`, `syntax`, `table`, `theme`, `spinner`)
  stay — they back the live `OutputRenderer`. After deletion, revisit
  `#![allow(dead_code)]` on `output/mod.rs` and tighten it to the specific
  still-dead helpers (if any) so future drift is surfaced.
- Regression validation: `cargo check --workspace`; `cargo check
  -p echo-agent-app-core --no-default-features`; `cargo test -p
  echo-agent-app-core output::` (the surviving renderer tests). The REPL
  smoke (`echo-agent-cli --cli` then one turn) must still render.
- Validation reports: [V01-01](../validations/A-OUT-01/V01-01.md)

### A-OUT-01-P2-02: `LatexExporter` (`export/latex.rs`) is dead; `tasks::ResearchOutputFormat::Latex` is never constructed

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/export/latex.rs:6-155` —
    `LatexExporter` with `markdown_to_latex`, `convert_line`,
    `escape_latex`, `generate_bibtex`.
  - `echo-agent-cli/echo-agent-app-core/src/export/mod.rs:1-7` — module
    docstring "提供多种格式的导出功能：Markdown、LaTeX、JSON 等" and
    `pub use latex::LatexExporter`.
  - Grep for `LatexExporter` / `markdown_to_latex` across `echo-agent-cli`:
    zero production callers; the only references are inside `export/latex.rs`
    (the impl + four tests).
  - `echo-agent-cli/echo-agent-app-core/src/tasks/background.rs:118-122` —
    `ResearchOutputFormat::{Markdown, Latex}`. Grep for
    `ResearchOutputFormat::Latex` across both repos: zero hits. Every
    constructor (`background.rs:251`, `tasks/service.rs:899`,
    `src/tauri/commands/tasks.rs:109`,
    `src/cli/cmd_impls/{research,coding,pipeline}.rs`) passes `Markdown` or
    `Default::default()`.
  - `echo-agent-cli/echo-agent-app-core/src/profiles/types.rs:22-23, 46-48,
    63` — `Profile.output_format: String` (default `"text"`), written by
    `Profile::new`; zero `.output_format` read sites outside the struct.
- Reachability: `export/latex.rs` is registered (`pub mod export` at
  `lib.rs:16`, `pub use latex::LatexExporter` at `export/mod.rs:7`) but has
  **no live caller**. The `Latex` variant of `ResearchOutputFormat` is
  defined and serialized but never constructed. `Profile.output_format` is
  persisted to `~/.eko/profiles/<name>.json` but never read.
- Expected invariant: per AGENTS.md "不要为'可能将来还会用'的代码留死路径.
  YAGNI —— 删了, 将来真需要再写", a whole export module, an enum variant,
  and a profile field that no caller exercises should be deleted, not
  retained as speculative API.
- Observed behavior: `LatexExporter::markdown_to_latex` is a credible
  Markdown-to-LaTeX converter (it handles headers, bold/italic/code,
  `[[N]]` citations, special-char escaping) — but it is unreachable. The
  `research::ReviewExportFormat::Pdf` path that *would* naturally produce
  LaTeX (via pandoc + `--pdf-engine=xelatex`) does not go through
  `LatexExporter`; it shells out to pandoc directly
  (`research.rs:1276-1309`). So even if LaTeX output were desired,
  `LatexExporter` is not in the loop.
- Impact:
  - **Maintenance burden + misleading surface.** The `export/mod.rs`
    docstring claims the application supports "Markdown、LaTeX、JSON 等"
    exports; only the research path actually does, and it does not use this
    module. A contributor investigating LaTeX export will find two parallel
    paths (`LatexExporter` and the pandoc `DocumentRenderer::Pandoc` with
    xelatex engine) and have to discover which is live.
  - **Profile field** silently persists a value the runtime ignores — a
    "config that doesn't do what it says" defect akin to A-TOOL-01-P2-01.
  - **No correctness risk** for current users.
- Root cause: `LatexExporter` predates the pandoc-based research document
  pipeline; when the research path gained real PDF/DOCX export via pandoc,
  the standalone LaTeX converter was not removed. The
  `ResearchOutputFormat::Latex` variant and `Profile.output_format` field
  are residue from the same era.
- Direction:
  1. Delete `echo-agent-app-core/src/export/latex.rs` and
     `echo-agent-app-core/src/export/mod.rs`; drop `pub mod export;` from
     `lib.rs:16`.
  2. Either delete `ResearchOutputFormat::Latex` outright, or — if the
     research-pipeline background task should actually produce LaTeX — wire
     it to call `LatexExporter` (after resurrecting it from step 1, or by
     inlining the conversion). Given YAGNI and that the live systematic
     review path uses pandoc for LaTeX-flavored PDF, deletion is recommended.
  3. Remove `Profile.output_format` and its `default_output_format` helper.
- Regression validation: `cargo check --workspace`; `cargo test -p
  echo-agent-app-core --tasks`; `cargo test -p echo-agent-app-core
  --profiles`. If `ResearchOutputFormat::Latex` is deleted, the
  `BackgroundTaskKind::Research` serde round-trip test must still pass
  (existing records serialize `markdown` only).
- Validation reports: [V01-01](../validations/A-OUT-01/V01-01.md)

### A-OUT-01-P2-03: Live `export_conversation` Markdown drops tool calls, tool results, attachments, and reasoning; the dead `Persistence::export_conversation_markdown` was richer

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/tauri/commands/conversations.rs:663-670` — the live
    Markdown builder:
    ```rust
    let mut content = format!("# {}\n\n", conv.title.as_deref().unwrap_or("Conversation"));
    for msg in &stored {
        content.push_str(&format!(
            "## {}\n\n{}\n\n",
            msg.role,
            msg.content.as_deref().unwrap_or("")
        ));
    }
    ```
    Only `role` and `content` are emitted.
  - `echo-agent/echo-core/src/memory/conversation.rs:65-82` — `StoredMessage`
    also carries `attachments_json`, `tool_calls_json`,
    `tool_result_json`, `created_at`. None are read by the live exporter.
  - Contrast with `echo-agent-cli/echo-agent-app-core/src/persistence.rs:286-318`
    (dead per A-STATE-01 P2-01): it emits title, `created_at`, `model`,
    role labels, `content`, and `tool_calls` (`tc.name`, `tc.arguments`).
- Reachability: every Tauri `export_conversation` invocation. The command
  is registered at `src/tauri/mod.rs:217` and consumed by the frontend at
  `web-frontend/src/api/endpoints.ts:437-440`.
- Expected invariant: per the task question — "retain complete content,
  artifact lineage, error causes". A Markdown export of a tool-using
  conversation must at minimum preserve the tool calls the assistant made
  and the tool results the tools returned; otherwise the export is a
  narrative summary, not a record of what happened.
- Observed behavior: exporting a conversation that contains an assistant
  message with `tool_calls_json = "[{\"name\":\"run_code\",...}]"` and a
  following `tool` role message with `tool_result_json` produces a Markdown
  file that lists "## assistant" (with only the final textual answer, which
  may be empty during a tool-only turn) and "## tool" (with empty content,
  because tool-role messages put their payload in `tool_result_json`, not
  `content`). The reader sees a gap where the tool interaction happened.
  Reasoning traces (`attachments_json` → `thinking_segments`,
  `execution_rounds`) and timestamps are likewise dropped.
- Impact:
  - **Content-completeness defect.** The exported Markdown misrepresents
    tool-heavy conversations. A user who exports to archive or share loses
    the provenance of how the answer was produced — the opposite of the
    "artifact lineage" the task asks about.
  - **Asymmetry.** The dead `Persistence::export_conversation_markdown`
  did this better; when it is deleted per A-STATE-01 P2-01 option 1, the
  application loses the richer recipe unless the live exporter is upgraded
  first.
- Root cause: `export_conversation` was written as a minimal
  proof-of-concept Markdown dumper and was never extended when
  `StoredMessage` gained `tool_calls_json` / `tool_result_json` /
  `attachments_json` (the `_echo_message_version` envelope from A-STATE-01).
  The richer `Persistence::export_conversation_markdown` was orphaned at
  the same time.
- Direction: extend `export_conversation` to walk the full
  `StoredMessage`:
  - Emit `content` (as today).
  - Decode `tool_calls_json` (a JSON array of `{name, arguments}`) and
    append a fenced block per call.
  - Decode `tool_result_json` (for `tool`-role messages) and append it
    under the tool section.
  - Decode `attachments_json` `_echo_message_version` envelope and emit
    `reasoning_content` (if present) as a blockquote.
  - Emit `created_at` timestamps.
  Reuse the canonical decode helpers (`restore_projection_meta` /
  `AttachmentsPayload::parse` — A-STATE-01 V03-01) so the round-trip stays
  aligned with the framework. Then it is safe to delete the dead
  `Persistence::export_conversation_markdown`.
- Regression validation: a test that exports a conversation containing one
  user message, one assistant message with a `run_code` tool call, one
  `tool` message with the result, and one final assistant answer — assert
  the Markdown contains the tool name, the arguments, the result snippet,
  and the final answer. Cross-check against `get_conversation` so the
  export and the live view agree.
- Validation reports: [V01-01](../validations/A-OUT-01/V01-01.md),
  [V04-01](../validations/A-OUT-01/V04-01.md)

### A-OUT-01-P3-01: `parse_export_format` is duplicated in `research_tool.rs` and `tauri/commands/research.rs`

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/research_tool.rs:220-233` —
    `fn parse_export_format(value: &str) -> ResearchResult<ReviewExportFormat>`
    returning the application `ResearchError::Invalid`.
  - `echo-agent-cli/src/tauri/commands/research.rs:257-270` — a second
    `fn parse_export_format(value: &str) -> Result<ReviewExportFormat,
    IpcError>` returning `IpcError::Validation`.
  - Both match the same seven wire strings (`markdown`, `pdf`, `docx`,
    `json`, `csv`, `bibtex`, `ris`) to the same enum.
- Reachability: the agent path (`research_tool.rs`) is reached via
  `ResearchLibraryTool::execute_with_context` action `export_review`
  (`research_tool.rs:164-181`). The GUI path is reached via the Tauri
  command `export_systematic_review` (`research.rs:251`). Both are live.
- Expected invariant: per AGENTS.md "严禁平行实现同一语义" — a single wire
  string → enum mapping must have one authoritative parser. Two parsers
  invite drift: adding a new format to one and forgetting the other yields
  an inconsistency where the agent accepts a format the GUI rejects (or vice
  versa).
- Observed behavior: the two parsers currently agree, but they are not
  wired to a shared definition. The error types differ
  (`ResearchError::Invalid` vs `IpcError::Validation`), which is the only
  legitimate layer-specific difference; the matching arms are pure
  duplication.
- Impact: low today (they agree); the risk is future drift. Also a small
  layering smell: the Tauri adapter re-implements application-core parsing
  instead of calling into `research::`.
- Root cause: the Tauri command was written as a thin mirror of the agent
  tool and copied the parser verbatim rather than re-exporting it.
- Direction: move `parse_export_format` into `echo-agent-app-core::research`
  (e.g. `pub fn parse_review_export_format(value: &str) ->
  ResearchResult<ReviewExportFormat>`), re-export it, and have both
  `research_tool.rs` and `tauri/commands/research.rs` call it. The Tauri
  side keeps its own `ResearchError → IpcError` mapping (that is the legit
  adapter concern).
- Regression validation: a parameterized test over the seven formats plus
  one invalid string, run against the shared helper.
- Validation reports: [V01-01](../validations/A-OUT-01/V01-01.md)

### A-OUT-01-P3-02: `ReviewExportArtifact` lacks a content hash; lineage is weaker than the framework `ToolOutputArtifactRef` and `AnalysisOutputArtifact`

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/research.rs:517-524` —
    `ReviewExportArtifact { review_id, format, path, bytes,
    citation_audit }`. No `sha256`.
  - Contrast: `echo-agent/echo-core/src/tools/artifact.rs:91-98` —
    `ToolOutputArtifactRef { path, artifact_bytes, payload_bytes, sha256,
    retention }`. And `echo-agent-cli/echo-agent-app-core/src/analysis.rs:160-167`
    — `AnalysisOutputArtifact { path, absolute_path, kind, bytes, sha256 }`.
  - `echo-agent-cli/echo-agent-app-core/src/research.rs:1166` —
    `bytes: u64::try_from(bytes.len()).unwrap_or(u64::MAX)` is computed but
    no hash is.
- Reachability: every successful `export_review` / `export_all_review_formats`
  → surfaced to both the agent (`research_tool.rs:178-179` via
  `success_json`) and the GUI (`endpoints.ts:1372-1375`,
  `ReviewWorkbench.tsx:159-161`).
- Expected invariant: per the task question — "artifact lineage". A
  reproducible-build document export should carry a content hash so the
  caller can detect re-run drift (the systematic-review contract already
  tracks revisions elsewhere). The framework already sets this expectation
  with `ToolOutputArtifactRef.sha256`; the application `AnalysisOutputArtifact`
  follows it; `ReviewExportArtifact` is the outlier.
- Observed behavior: the agent and GUI receive a `path` and a `bytes` count
  but no fingerprint. Re-running `export_review` after a tiny content
  change produces a new file at the same path; nothing in the artifact
  metadata distinguishes the two runs except `bytes` (which is fragile for
  same-size edits).
- Impact: low. No correctness or security defect. The gap is contract
  inconsistency: a downstream consumer that assumes "EKO artifact metadata
  includes a hash" (a reasonable assumption given the framework and analysis
  shapes) will fall through on review artifacts.
- Root cause: `ReviewExportArtifact` was specified before the
  `sha256`-everywhere convention was consolidated; `export_review` already
  has `hash_bytes` (`research.rs:2001-2003`) available but does not call it
  for the artifact.
- Direction: compute `hash_bytes(&bytes)` inside `export_review`
  (`research.rs:1156` after `atomic_write`) and add `pub sha256: String` to
  `ReviewExportArtifact`. Mirror the field in the TypeScript type
  (`endpoints.ts:1266-1272`). This is a small, additive change.
- Regression validation: a test that exports a review in two formats and
  asserts the two `sha256` values differ; a serialization smoke on the
  TypeScript type.
- Validation reports: [V04-01](../validations/A-OUT-01/V04-01.md)

### A-OUT-01-P3-03: `output/mod.rs::print_tool_result` mixes `chars().take(300)` (preview) with `output.len() > 300` (suffix decision) — UTF-8 inconsistent

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/output/mod.rs:224-225`:
    ```rust
    let preview: String = output.chars().take(300).collect();
    let suffix = if output.len() > 300 { "..." } else { "" };
    ```
- Reachability: every REPL `OutputRenderer::print_tool_result` call
  (`repl.rs:787`). REPL is a hidden entry (`args.rs:26` `--cli`), so the
  blast radius is small, but the function is part of the public
  `OutputRenderer` API.
- Expected invariant: per AGENTS.md "字符串处理: UTF-8 安全" — "判断长度用
    字符数" and the explicit bad-example `s.len() > 100 // ❌ 这是字节数`.
  The suffix decision must use the same unit (chars) as the preview
  truncation.
- Observed behavior: for a Chinese string of 100 characters (300 bytes),
  `output.len() > 300` is false at exactly 100 chars but true at 101 chars
  (303 bytes). The preview `chars().take(300)` would show up to 300 chars,
  so for Chinese the suffix `"..."` is added while the preview is still far
  from its 300-char cap — the ellipsis misleads the reader into thinking
  the preview is truncated when it is not. Conversely, a 400-char Chinese
  string (1200 bytes) gets both a 300-char preview and the suffix, which is
  correct.
- Impact: cosmetic only. No panic (no byte slicing is done); the only
  consequence is a misleading `"..."` for multi-byte content in the narrow
  band where byte length crosses 300 but char count is below 300.
- Root cause: the preview was migrated to `chars().take()` but the suffix
  guard was not.
- Direction: replace `output.len() > 300` with
  `output.chars().count() > 300`. Note `print_tool_call` at `mod.rs:207`
  already does this correctly (`args_str.chars().count() > 200`) — mirror
  it.
- Regression validation: a unit test feeding a 100-char Chinese string
  (300 bytes) and asserting no suffix; feeding a 400-char Chinese string
  and asserting both a 300-char preview and a suffix.
- Validation reports: [V02-01](../validations/A-OUT-01/V02-01.md)

### A-OUT-01-P3-04: `research::atomic_write` omits parent-directory fsync (recurring atomic-write defect)

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/research.rs:1988-1999`:
    ```rust
    fn atomic_write(path: &Path, bytes: &[u8]) -> ResearchResult<()> {
        let parent = path.parent().ok_or_else(/* ... */)?;
        fs::create_dir_all(parent)?;
        let temp_path = parent.join(format!(".{}.tmp", new_record_id("write")));
        let mut file = fs::File::create(&temp_path)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::rename(&temp_path, path)?;
        Ok(())
    }
    ```
  - No `sync_parent_directory` call after the rename; no cleanup of
    `temp_path` on error (the `?` operators early-return, leaving the temp
    behind).
  - Contrast with the framework `FileConversationStore::atomic_write`
    (`echo-state/src/memory/file_conversation.rs:494-528`, per A-STATE-01
    Current Path) which does fsync the temp, rename, **then** fsync the
    parent directory, and removes the temp on any I/O error after creation.
- Reachability: every review-export write (`research.rs:1156`) and every
  other research write that routes through `atomic_write` (source/evidence/
  review JSON via `write_json` at `research.rs:1985`).
- Expected invariant: per A-STATE-01 V02-01 and F-MEM-01 P2-01/P2-02 — an
  atomic-write recipe that omits parent-dir fsync can lose the rename
  across a crash; the framework has already consolidated the correct
  recipe. The application must match it for any durability-sensitive write.
- Observed behavior: on a crash between `fs::rename` and the directory
  entry being flushed, the renamed file may not appear after recovery
  (classic rename-durability gap). The temp file leaks on error.
- Impact: low for the local-assistant threat model (single user, rare
  crashes mid-export), and review exports are re-runnable. But the defect
  is recurring: A-STATE-01 V02-01 called out the same pattern in
  `Persistence::write_json`; F-MEM-01 P2-01/P2-02 called it out in
  `FileStore` and `EmbeddingStore`. `research::atomic_write` is the fourth
  instance.
- Root cause: copy-pasted atomic-write recipe that predates the framework's
  consolidated version; never migrated.
- Direction: either (a) call the framework's `echo_state` / `echo_agent`
  atomic-write helper if one is exposed for application use, or (b) inline
  the full recipe (fsync temp → rename → fsync parent dir, with temp
  cleanup on error) into `research::atomic_write`. Option (a) is preferred
  if it removes the application's fourth copy of the recipe.
- Regression validation: a test that triggers an error after temp creation
  and asserts the temp is removed; a documentation note that the recipe
  matches the framework.
- Validation reports: [V04-01](../validations/A-OUT-01/V04-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Format/renderer registry — formats present, renderers selected, content complete | yes | failed | [V01-01](../validations/A-OUT-01/V01-01.md) |
| V02 | Large/Unicode content — UTF-8-safe truncation across all output paths | yes | failed | [V02-01](../validations/A-OUT-01/V02-01.md) |
| V03 | Missing external converter — graceful fallback when pandoc/quarto absent | yes | passed (with UX finding) | [V03-01](../validations/A-OUT-01/V03-01.md) |
| V04 | Artifact identity and cross-surface delivery — consistent artifacts and lineage across TUI/GUI/CLI | yes | failed | [V04-01](../validations/A-OUT-01/V04-01.md) |
| V05 | Historical-document drift | conditional | passed | See Historical Claim Status below; each claim classified inline. |

V01, V02, V04 are recorded as **failed** because they each host at least
one finding (A-OUT-01-P2-01 / P2-03 for V01; P3-03 for V02; P2-03 / P3-02 /
P3-04 plus the conversation-export parity gap for V04). V03 passed at the
Rust layer (structured errors, conditional format list, UTF-8-safe stderr
capture); the only gap is a frontend UX pre-check, noted inside the
report.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `output/format.rs:8` "输出格式 (用于 --output / -o 标志)" | stale | No `--output` / `-o` flag exists in `src/cli/args.rs:12-68`; the enum and `format_response` have zero live callers (V01-01). |
| `output/mod.rs:1-8` "OutputRenderer 是整个 CLI 输出的唯一外观 (Facade), REPL 模式和 TUI 模式都通过它输出" | half-stale | REPL uses it (`repl.rs:96`); TUI bypasses it and uses `tui::markdown::render_markdown`. The "唯一外观" claim is false. |
| `output/mod.rs:9` `#![allow(dead_code)]` | current (and diagnostic of P2-01) | The suppress confirms the maintainers know parts are dead; this task recommends tightening it after deleting `format.rs`. |
| `export/mod.rs:1-4` "提供多种格式的导出功能：Markdown、LaTeX、JSON 等" | stale | Only `LatexExporter` is implemented in `export/`, and it has zero production callers (V01-01). The live multi-format export lives in `research.rs`. |
| `research.rs:1216-1233` "PDF/DOCX export requires Pandoc or Quarto on PATH" | current | `resolve_document_renderer` probes PATH + `EKO_PANDOC`/`EKO_QUARTO` exactly as documented; structured errors at `research.rs:1254, 1287` (V03-01). |
| `research.rs:491-501` `ReviewExportFormat` (7 formats) | current | All seven are honored by `export_review`; `export_all_review_formats` conditionally includes DOCX/PDF (V03-01). |
| A-STATE-01 P2-01 "`Persistence` and `SessionSearchEngine` are constructed but never read" | current | This task relies on it: the richer `Persistence::export_conversation_markdown` is dead; the live `export_conversation` is lossy (A-OUT-01-P2-03). |
| A-TOOL-01 V04 "model-facing truncation uses `chars().take()` (UTF-8 safe)" | current | Re-confirmed; the only UTF-8 outlier in the output paths is `print_tool_result` at the application layer (A-OUT-01-P3-03). |
| F-EXT-03 Current Path 5 "`web_fetch` / `sql_query` spill via `persist_tool_output` and emit `ToolOutputArtifactRef` with `truncated=true`" | current | Used here as the lineage reference; `ReviewExportArtifact` is the outlier with no `sha256` (A-OUT-01-P3-02). |

## Coverage And Uncertainty

- **Frontend deep-dive** was scoped to the export consumers
  (`endpoints.ts`, `ReviewWorkbench.tsx`). Full reducer/preview rendering
  belongs to A-FE-02; this task only audited the DTO contracts
  (`ReviewExportFormat`, `ReviewExportArtifact`, `conversations.export`)
  for parity with the Rust side.
- **TUI Markdown renderer** (`tui/markdown.rs`) was inspected for
  completeness of format coverage, not for rendering correctness. It
  handles headings/code/lists/tables/blockquotes; whether it renders every
  GFM extension the backend emits is out of scope.
- **Analysis workbench** (`analysis.rs`) was inspected for artifact
  lineage (`AnalysisOutputArtifact.sha256`) and UTF-8 truncation
  (`bounded_text`); the deeper question of whether the agent should have a
  typed `analysis_library` tool (mirroring `research_library`) is filed as
  a coverage observation, not a finding, because the task scope is "output
  formats and export" and the analysis pipeline does produce output
  artifacts accessible via `read_file`.
- **Channels mode** (`--channels`) was not inspected; it is a hidden
  experimental entry and its output paths, if any, are out of scope.
- **No executable test was run** in this review (read-only). All V-series
  reports are static inspections against `echo-agent-cli` `b3b2e81` and
  `echo-agent` `9b0e0fa`.
- **`export_review` PDF/DOCX happy path** was not executed because it
  requires pandoc + a PDF engine on PATH; the inspection covers the
  command construction (`research.rs:1276-1309`) and the structured-error
  branches. A live run would strengthen V03-01 but is not necessary to
  classify the fallback behavior.
- **Claims that remain uncertain**:
  - Whether any consumer actually depends on `ReviewExportArtifact`
    carrying a hash today (it does not, so the addition is strictly
    additive; the impact of *not* adding it is the contract inconsistency,
    not a live bug).
  - Whether `tasks::ResearchOutputFormat::Latex` was ever intended to be
    wired to `LatexExporter`; the absence of any construction site
    suggests not, but product intent could not be confirmed from code
    alone.

## Handoff

- Downstream tasks may rely on:
  - The **live** document-export authority is `research::export_review` /
    `export_all_review_formats`, reached by two thin adapters
    (`ResearchLibraryTool` action `export_review` for the agent,
    `export_systematic_review` Tauri command for the GUI). Both produce
    identical artifacts (V04-01).
  - The **live** conversation-export authority is the Tauri
    `export_conversation` command, and it is lossy (drops tool calls /
    results / attachments / reasoning). Any downstream task that promises
    "export the conversation" must extend this command (A-OUT-01-P2-03)
    rather than revive `Persistence::export_conversation_markdown`.
  - The **dead** output/export surface (`output/format.rs`, `export/latex.rs`,
    `tasks::ResearchOutputFormat::Latex`, `Profile.output_format`) is safe
    to delete; no caller relies on it (V01-01).
  - The framework `ToolOutputArtifactRef` (with `sha256`) is the lineage
    reference; `AnalysisOutputArtifact` follows it, `ReviewExportArtifact`
    does not (yet) — A-OUT-01-P3-02.
  - External-converter fallback is structurally graceful at the Rust layer
    (V03-01); the only gap is a frontend UX pre-check.
- Reports downstream tasks must read:
  - [V01-01](../validations/A-OUT-01/V01-01.md) for the format-registry and
    dead-exporter matrix.
  - [V04-01](../validations/A-OUT-01/V04-01.md) for the cross-surface
    delivery map (agent vs GUI vs TUI).
  - A-STATE-01 V01-01 / V04-01 for the `Persistence` / `SessionSearchEngine`
    dead-code context that this task extends.
  - A-TOOL-01 V04 for the framework truncation + spill contract that this
    task treats as the UTF-8 and lineage reference.
- Conditions that make this report stale:
  - Adding a `--output` flag to `Args` that wires `output::OutputFormat`
    (would resolve A-OUT-01-P2-01).
  - Extending `export_conversation` to emit tool calls / results /
    attachments / reasoning (would resolve A-OUT-01-P2-03).
  - Deleting `output/format.rs`, `export/latex.rs`,
    `tasks::ResearchOutputFormat::Latex`, or `Profile.output_format`
    (would resolve A-OUT-01-P2-01 / P2-02).
  - Adding a `sha256` field to `ReviewExportArtifact` (would resolve
    A-OUT-01-P3-02).
  - Adding an agent-side conversation-export tool or a TUI `/export` slash
    command (would resolve the V04-01 parity gap).
- Follow-up task IDs (no fixes implemented in this review):
  - **A-SRF-01 / X-SRF-01** should pick up the conversation-export surface
    parity gap (GUI-only; agent and TUI cannot export conversations).
  - **A-FE-02** should pick up the ReviewWorkbench UX pre-check for PDF
    /DOCX availability before the user clicks Export.
  - **X-BND-01** should consolidate the three format enums
    (`output::OutputFormat`, `research::ReviewExportFormat`,
    `tasks::ResearchOutputFormat`) once the dead `OutputFormat` is removed,
    and confirm whether the surviving two should be unified or kept
    distinct (they serve different domains: systematic-review export vs
    research-pipeline background-task output).
  - A dedicated cleanup task should land the dead-exporter deletions
    (A-OUT-01-P2-01, P2-02, P3-01, P3-03) and the `export_conversation`
    content extension (A-OUT-01-P2-03) in one PR, since they all touch the
    same `output/` / `export/` / `research.rs` / `conversations.rs` area.
