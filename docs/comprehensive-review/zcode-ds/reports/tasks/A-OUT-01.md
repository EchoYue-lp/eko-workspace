# A-OUT-01: Output formats, export, and file delivery

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean (both repositories; scratch files only under
> /tmp/eko-out-check, outside the repos)

## Question

Do EKO output profiles and Markdown/document/data export paths retain complete
content, artifact lineage, error causes, and consistent availability across
surfaces?

## Scope

- Output-format/profile machinery: `echo-agent-app-core/src/output/`
  (`mod.rs` OutputRenderer facade, `format.rs` OutputFormat,
  `markdown.rs` terminal renderer, `syntax.rs`, `table.rs`, `theme.rs`,
  `spinner.rs`), `echo-agent-app-core/src/export/` (LatexExporter),
  `echo-agent-app-core/src/profiles/` (ProfileManager, `output_format` field),
  `echo-agent-app-core/src/tasks/background.rs` (ResearchOutputFormat),
  `echo-agent-cli/src/cli/cmd_impls/advanced.rs` (`/output`, `/export`,
  `/profile` commands).
- Export/file-delivery paths: `echo-agent-app-core/src/research.rs`
  (systematic-review export incl. Pandoc/Quarto PDF/DOCX conversion),
  `echo-agent-app-core/src/research_tool.rs` (agent tool `research_library`),
  `echo-agent-app-core/src/analysis.rs` (file-backed analysis artifacts),
  `echo-agent-cli/src/tauri/commands/research.rs`, `commands/conversations.rs`
  (`export_conversation`), `commands/analysis.rs`.
- Frontend consumers: `web-frontend/src/components/papers/*` (ReviewWorkbench
  export), `web-frontend/src/components/layout/RightWorkspace.tsx` (research
  tab), `web-frontend/src/api/endpoints.ts` (conversationApi.export,
  systematicReviewsApi.export, analysisApi, web-mode fallbacks).
- Live renderers: `echo-agent-cli/src/tui/markdown.rs` (TUI markdown),
  `echo-agent-cli/src/tui/widgets/chat.rs`, REPL streaming path in
  `src/cli/repl.rs`.

## Out Of Scope

- Framework domain tools that produce data artifacts (`export_data`,
  `generate_chart`, `web_fetch` spill, `bibtex_generate`) — F-EXT-03
  (complete); its P3-02 (`export_data` truncation) is cross-referenced.
- Conversation persistence/restore/store roots — A-STATE-01 (complete);
  its P3-03 (TUI `/fork` unfiltered transcript) is the same defect class as
  one finding here.
- Tool exposure/permission/sandbox — A-TOOL-01 (complete).
- Research/analysis workbench domain logic (connectors, provenance, formal
  inference) — A-DOM-01 (primary owner of the research pipeline semantics;
  this task covers only the export/file-delivery projection).
- Frontend rendering quality of large outputs (collapse/expand) — A-FE-02.
- Live-network provider behavior — F-EXT-03 (recorded `not_run` there).

## Inputs

- Root `AGENTS.md` (full), shared `README.md`, `REPORTING.md`, `TASKS.md`
  (A-OUT-01 card), `zcode-ds/README.md`, report templates.
- Dependency reports read: zcode-ds `A-STATE-01` (complete), `A-TOOL-01`
  (complete), `F-EXT-03` (complete).
- Historical documents treated as hypotheses: `echo-agent-cli/docs/MASTER-PLAN.md`
  (lines 46-54, 245-274), `echo-agent-cli/docs/architecture.md:120`,
  `echo-agent-cli/README.md:385-388`, `output/mod.rs` and `output/format.rs`
  module docs.

## Layering Decision

- Generic mechanism (framework, correct): none new — the framework has no
  output/export concept (V01-01); all machinery below is application-side.
  The one framework asset the export paths should consume is
  `filter_user_visible_transcript` (`echo-agent/src/agent/snapshot.rs:65-71`),
  currently private; the CLI session export bypasses it (P2-01).
- EKO product policy (application, correct): `ResearchOutputFormat`/
  `OutputFormat`/`OutputRenderer`/`LatexExporter`/`profiles` module,
  `/export`/`/output` commands, review-export authority `research::export_review`,
  `export_conversation`, TUI markdown renderer, analysis artifact layout.
- Adapter boundary: CLI/Tauri commands are thin adapters over the app-core
  export authority — except `/export` (`cmd_impls/advanced.rs:166-217`) which
  reads the live runtime context directly instead of the canonical projection,
  and `export_conversation` which reads the persisted store — two divergent
  sources for one semantic (P2-01, duplicate authority per AGENTS.md).
- Duplicate search (V01-01, both repositories): `OutputFormat`,
  `OutputRenderer`, `render_markdown_to_terminal`, `render_markdown`,
  `LatexExporter`, `ResearchOutputFormat`/`output_format`, `ProfileManager`/
  `get_active`, `export_review`/`export_conversation`/`cmd_export`,
  `--output`/`-o`, `pandoc`/`quarto`/`EKO_PDF_ENGINE`, `axum`/`Router::new`/
  `TcpListener`, `ApiError`/`StreamingEvent`, `/export` HTTP route.
  Result: one review-export authority; two session-export implementations with
  different sources and delivery; four dead/aspirational format surfaces
  (`OutputFormat`, `/output` stub, `ResearchOutputFormat` field,
  `profiles.output_format`); no framework duplicate. No `worker` terminology.

## Current Path

Verified data flows (V02-01):

1. **Review export (single authority, healthy)**: `research::export_review`
   (`research.rs:1123-1169`) renders Markdown/JSON/CSV/BibTeX/RIS in-process
   and PDF/DOCX via Pandoc/Quarto (`render_review_document`, `:1251-1324`;
   missing converter/engine -> `ResearchError::External` with actionable
   message, `:1252-1257,1285-1290`), writes atomically under
   `{ws}/research/reviews/<id>/reports/systematic-review.<ext>`, returns
   `ReviewExportArtifact {review_id, format, path (workspace-relative), bytes,
   citation_audit}`. Three surfaces call it: Tauri `export_systematic_review`
   (`src/tauri/commands/research.rs:240-255`, registered mod.rs:284, consumed
   by `ReviewWorkbench.tsx:155-169`), CLI `/papers export`
   (`cmd_impls/research.rs:265-277`), agent tool `research_library.export_review`
   (`research_tool.rs:164-182`, registered `runtime.rs:287`).
2. **Session export (two divergent implementations)**: CLI/TUI `/export`
   (`cmd_impls/advanced.rs:166-217`) dumps the raw runtime context
   (`agent.get_messages()`, `react/mod.rs:1260-1263`) to
   `~/.eko/exports/<name>.json|md`; Tauri `export_conversation`
   (`conversations.rs:642-662`) projects the persisted store record to a
   markdown string and is registered (mod.rs:217) but has **no frontend
   consumer** (`conversationApi.export`, endpoints.ts:437-441, is unreferenced;
   its HTTP fallback route has no server — web mode is dev-only).
3. **Output-format machinery (decorative)**: `OutputFormat`
   (`output/format.rs:9-24`, doc claims `--output`/`-o` flag that does not
   exist) — zero production callers; `/output` (`advanced.rs:265-279`) is a
   no-op stub; `BackgroundTaskKind::Research.output_format`
   (`tasks/background.rs:26`) is never read (`to_prompt` drops it with `..`);
   `Profile.output_format` (`profiles/types.rs:22-23`) sits in a module with
   zero callers; `LatexExporter` (`export/latex.rs`) has zero callers. All
   live call sites hardcode Markdown (`research.rs:45`, `coding.rs:235`,
   `pipeline.rs:116`, `service.rs:899`).
4. **Rendering**: TUI (default surface) renders markdown through
   `tui/markdown.rs` (streaming-safe, `tui/mod.rs:1207,1426`); its table
   renderer mixes byte and char width units (`:388,414-419`); app-core
   `OutputRenderer` serves only the REPL subset (`repl.rs`), whose
   truncations are char-safe (`output/mod.rs:207-212,224-226`); the app-core
   terminal markdown/code/table renderers are unreachable.
5. **Availability matrix** (V02-01): review export = GUI + CLI + agent tool,
   **absent on TUI** (`SlashCommand` enum `tui/commands.rs:53-140` has no
   research/export commands; unknown commands rejected `events.rs:4683-4688`);
   session export = CLI only (GUI endpoint dead); analysis = GUI + TUI + CLI
   (`/analysis` shares `run_analysis_with_agent`); web/HTTP API mode has no
   server (frontend `isTauri()` branches fall back to dev-only `/api`, and
   analysis/research explicitly reject web mode with
   `analysisRequiresDesktop`/`researchRequiresDesktop`).

## Findings

### A-OUT-01-P2-01: Session export `/export` writes the unfiltered runtime transcript (system prompt included) and silently drops non-text content; a second, unused `export_conversation` implements the same semantic with a different source

- Priority: P2
- Confidence: high
- Layer: adapter (CLI command) + application (divergence)
- Evidence: `echo-agent-cli/src/cli/cmd_impls/advanced.rs:166-217` —
  `handle.read_async(... a.context().lock().await ...)` over
  `ctx.messages()` (`:171-176`); JSON maps `m.content.as_text().unwrap_or_default()`
  (`:182-186`), markdown uses `msg.content.as_text_ref()` (`:192-197`) which
  only matches `MessageContent::Text` (`echo-core/src/llm/types.rs:120-127`);
  file written to `~/.eko/exports/<name>.<ext>` (`:206-214`). `get_messages`
  returns the full runtime context incl. `Role::System` messages
  (`echo-agent/src/agent/react/mod.rs:1260-1263`; `types.rs:152-172`). The
  canonical projection `filter_user_visible_transcript`
  (`echo-agent/src/agent/snapshot.rs:65-71`) is not applied. Duplicate:
  `export_conversation` (`echo-agent-cli/src/tauri/commands/conversations.rs:642-662`,
  store-projected markdown) registered at `src/tauri/mod.rs:217` with zero
  frontend callers (V01-01/V02-01).
- Reachability: `/export [json|markdown] [name]` in the REPL (default CLI
  surface) — README-documented feature (`README.md:386` "导出会话");
  `export_conversation` reachable only via unused `conversationApi.export`
  (endpoints.ts:437-441).
- Expected invariant: exported session files contain the same user-visible
  transcript the per-turn save persists (one projection policy), and never
  leak internal context (system prompt/instructions); one export
  implementation per semantic.
- Observed behavior: the exported markdown starts with `### system` and the
  full system prompt; tool-call/multimodal parts with no text are silently
  omitted from markdown (JSON writes `content: ""`); the exported file
  diverges from what the user sees in the session; the GUI-side export
  command exists but nothing calls it.
- Impact: exported/shared session files leak internal instructions/context and
  misrepresent the conversation (tool activity missing); two implementations
  with different sources (live context vs persisted projection) will drift;
  the README-documented feature is the only CLI session file export.
- Root cause: the command predates the framework's user-visible projection and
  never reused it; `export_conversation` was written in parallel for a GUI
  consumer that was never built; no shared export authority was introduced.
- Direction: route `/export` through the same projection as the canonical
  per-turn save (export the framework's `filter_user_visible_transcript`, or
  add a `project_user_visible_messages` helper, and reuse it for fork,
  /export and the GUI export); make `export_conversation` the shared
  authority and wire or delete it; add an EKO test asserting the exported
  markdown contains no `### system` section and includes tool activity text.
- Regression validation: fixture — session with a system message + a tool call
  turn; `/export markdown` output must contain neither the system text nor
  unhandled gaps; JSON export must serialize tool messages; a second fixture
  comparing `/export` output with the persisted conversation projection.
- Validation reports: [V01-01](../validations/A-OUT-01/V01-01.md),
  [V02-01](../validations/A-OUT-01/V02-01.md),
  [V03-01](../validations/A-OUT-01/V03-01.md),
  [V03-03](../validations/A-OUT-01/V03-03.md)
- Cross-reference: same defect class as A-STATE-01-P3-03 (TUI `/fork`);
  A-STATE-01's store-root findings (P1-01/P2-01) matter for which store the
  GUI export would read.

### A-OUT-01-P2-02: TUI table renderer mixes byte and char width units — misaligned columns and wrong truncation thresholds for CJK/emoji content on the default surface

- Priority: P2
- Confidence: high (dynamically confirmed in V03-04)
- Layer: application
- Evidence: `echo-agent-cli/src/tui/markdown.rs:388` (`col_widths[i] =
  (*width).max(cell.len().min(40))` — bytes), `:394-400` (total/suppression
  math), `:414-415` (`if cell.len() > width { format!("{:.width$}...", cell,
  width = width.saturating_sub(3)) }` — byte condition, char truncation),
  `:419` (`format!(" {:<width$} │", truncated, width = width)` — char
  padding). Same bug class in the dead app-core renderer
  (`output/markdown.rs:311` `header.len()`).
- Reachability: any table in a markdown answer rendered by the TUI chat
  (`tui/widgets/chat.rs:209` -> `render_markdown`, `tui/mod.rs:1207/1426`);
  confirmed numerically: a 6-char CJK cell (18 bytes) is padded to 18
  characters (12 extra display columns); truncation threshold compares bytes
  while the truncation cuts characters (V03-04, `rustc` run in /tmp).
- Expected invariant: column geometry and truncation use one unit (characters
  or display width) so CJK/emoji content renders aligned and truncates at the
  declared limit (AGENTS.md UTF-8 rules; unicode-width is already a `tui`
  feature dependency, `Cargo.toml:25`).
- Observed behavior: tables with any multibyte cell misalign (over-padding),
  and the 40-unit truncation cap effectively applies to bytes for ASCII cells
  and to characters for CJK cells — inconsistent, invisible to the existing
  tests (which contain no non-ASCII table fixtures; V04-05).
- Impact: degraded rendering of the most common non-English content on the
  default TUI surface; wrong truncation thresholds make tables misleading for
  long CJK cells.
- Root cause: byte length (`str::len`) reused as a display width; the
  renderer was written against ASCII assumptions while the rest of the module
  is otherwise char-aware.
- Direction: compute widths with `cell.chars().count()` (or
  `unicode_width::UnicodeWidthStr::width`, already a dependency) and keep the
  truncation/padding in the same unit; add table fixtures with CJK and emoji
  cells asserting column alignment and truncation boundaries; mirror the fix
  in the dead `output/markdown.rs` renderer before deciding to delete it
  (P3-01).
- Regression validation: unit test — markdown table with a 6-char CJK cell and
  a 40+ char CJK cell, asserting the rendered lines align and the long cell is
  truncated to the declared width.
- Validation reports: [V03-01](../validations/A-OUT-01/V03-01.md),
  [V03-04](../validations/A-OUT-01/V03-04.md),
  [V04-05](../validations/A-OUT-01/V04-05.md)

### A-OUT-01-P2-03: Output-format machinery is decorative — `OutputFormat` is dead, `/output` is a no-op stub, `ResearchOutputFormat` is inert, and every real export path hardcodes Markdown

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `OutputFormat` (`echo-agent-app-core/src/output/format.rs:9-24`)
  with module doc claiming the `--output`/`-o` flag (`:8`) — zero production
  callers, no such flag in `src/cli/args.rs` (V01-01); `/output`
  (`src/cli/cmd_impls/advanced.rs:265-279`) prints "Output format: {f}" and
  returns without changing anything; `BackgroundTaskKind::Research.output_format`
  (`tasks/background.rs:26,115-122`) is never read — `to_prompt` matches
  `Self::Research { topic, max_papers, .. }` (`:161-174`); all creation sites
  pass `Markdown` (`cmd_impls/research.rs:45`, `cmd_impls/coding.rs:235`,
  `cmd_impls/pipeline.rs:116`, `tasks/service.rs:899`, `src/tauri/commands/tasks.rs:109`);
  `LatexExporter` (`export/latex.rs`) zero callers; `Profile.output_format`
  (`profiles/types.rs:22-23,46-48`) in a module with zero callers.
- Reachability: `/output` is registered (advanced.rs:279, repl.rs:168) and
  user-invocable in the REPL; the README documents it (`README.md:387`).
- Expected invariant: a documented format switch actually selects a render
  path; an `output_format` parameter influences the produced artifact; a
  "Latex" option produces LaTeX.
- Observed behavior: `/output markdown` prints a line and leaves all rendering
  untouched; LaTeX research output is unreachable (the only enum value ever
  used is Markdown; `LatexExporter` is orphaned); no artifact is ever produced
  in anything but Markdown by the pipelines.
- Impact: users and agents are told a capability exists (format profiles,
  LaTeX export) that has no effect; the CLI surface presents a control that
  lies; `ResearchOutputFormat::Latex` is a trap for downstream code that
  believes it is honored.
- Root cause: format selection was scaffolded (enum + stub command + field)
  before any render/export path consumed it; later implementations hardcoded
  Markdown and the scaffolding was never wired or removed.
- Direction: either wire one format switch end-to-end (REPL render path +
  research pipeline prompt/artifact) or delete the dead pieces: `OutputFormat`
  + `format_response`, the `/output` command, the `output_format` field and
  `ResearchOutputFormat::Latex` (keep the enum only if a consumer exists),
  `LatexExporter`, `Profile.output_format` (with the whole `profiles` module —
  see P3-01); update README.md:387 and architecture.md:120.
- Regression validation: after deletion — grep `OutputFormat`/`LatexExporter`/
  `output_format` returns only tests or intentional call sites; `/output` no
  longer listed in `/help`; `cargo test -p echo-agent-app-core --lib` green.
- Validation reports: [V01-01](../validations/A-OUT-01/V01-01.md),
  [V02-01](../validations/A-OUT-01/V02-01.md),
  [V05-01](../validations/A-OUT-01/V05-01.md)

### A-OUT-01-P2-04: TUI — the default surface — exposes no research-library or export commands, while GUI has a full workbench and CLI has `/papers` (availability asymmetry)

- Priority: P2
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent-cli/src/tui/commands.rs:53-140` — the `SlashCommand`
  enum contains no research/papers/export/save/load variants; unknown commands
  are rejected with "Unknown command: {command}" (`src/tui/events.rs:4683-4688`);
  the TUI does not route to the CLI registry. GUI: `PaperPanel` mounted on the
  Research tab (`web-frontend/src/components/layout/RightWorkspace.tsx:113-119`),
  `ReviewWorkbench.tsx:155-169` export. CLI: `PapersCommand` with `/papers
  export` (`cmd_impls/research.rs:265-277,341-355`). TUI does expose
  `/analysis` (enum, Coding section) sharing `run_analysis_with_agent`.
- Reachability: a TUI user (default `echo-agent-cli` entry, main.rs TUI
  default) cannot list sources, create/audit a review, or export anything
  from the research library; the systematic-review workbench is GUI-only and
  the slash interface is REPL-only.
- Expected invariant: surface parity (AGENTS.md) — any capability on one
  surface (research library management, export) exists on the others,
  differing only in rendering.
- Observed behavior: review creation/export exists on GUI + CLI but is absent
  on TUI; session export exists on CLI only; there is no single capability
  that is uniformly absent/present, so this is not a deliberate TUI policy.
- Impact: the primary entry point of the product silently lacks the entire
  research/export domain; a TUI user must switch surfaces to export a review.
- Root cause: the TUI slash-command enum grew by surface-local additions while
  domain commands were added to the CLI registry and the GUI panels without a
  cross-surface capability checklist.
- Direction: add research-library commands (list/show/evidence/reviews/audit/
  export) to the TUI `SlashCommand` enum routing to the same app-core
  functions, or mount a minimal research panel; reuse the CLI registry or a
  shared app-core service so one implementation serves REPL and TUI; add a
  surface-vs-command matrix test (X-SRF-01 style).
- Regression validation: TUI integration — run `/papers export <id> markdown`
  in the TUI and assert the artifact file exists and the path is printed;
  `/help` lists the new commands.
- Validation reports: [V02-01](../validations/A-OUT-01/V02-01.md),
  [V03-03](../validations/A-OUT-01/V03-03.md)

### A-OUT-01-P3-01: Dead output/export code cluster with false facade claims — `LatexExporter`, `profiles` module, app-core terminal renderers, `export_conversation` and the web-mode API types are unreachable, masked by `#![allow(dead_code)]`

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `output/mod.rs:9` `#![allow(dead_code)]` over a module whose doc
  claims `OutputRenderer` is the single facade for "REPL and TUI" (`:6-7`) —
  TUI uses `tui/markdown.rs` and the REPL uses only 13 of ~28 methods
  (V01-01); `output/markdown.rs` (`render_markdown_to_terminal`, zero
  callers), `output/syntax.rs`, `output/table.rs` renderers unreachable;
  `export/latex.rs` `LatexExporter` zero callers (tests green, V04-02);
  `profiles/` (`ProfileManager`) zero callers (tests green, V04-03);
  `export_conversation` (`src/tauri/commands/conversations.rs:642-662`)
  registered (mod.rs:217) with zero frontend consumers; the frontend HTTP
  fallback (`conversationApi.export` HTTP branch, endpoints.ts:441) and the
  web-mode API types `ApiError`/`StreamingEvent` (`app-core/error.rs:3-40`)
  have no server implementation (no axum Router/TcpListener in the CLI tree,
  V01-01).
- Reachability: none for the listed code; `export_conversation` reachable only
  via IPC with no caller.
- Expected invariant: docs describe actual behavior; dead code is removed
  (AGENTS.md cleanup rule).
- Observed behavior: ~700 lines of dead facade/renderer/export/profile code
  with green tests, a stale facade claim, and a "web mode" HTTP API surface
  that cannot be reached from any binary.
- Impact: maintainers/LLMs may believe the REPL renders markdown and the GUI
  can export conversations; the `#![allow(dead_code)]` hides future rot; the
  web-mode endpoints look supported while no server exists.
- Root cause: scaffolding and migrated modules left behind without a caller
  audit; the module-level allow suppresses the warning that would have
  surfaced it.
- Direction: delete `LatexExporter` (or wire it to `ResearchOutputFormat::Latex`
  if P2-03 keeps LaTeX), the `profiles` module, the unreachable
  `OutputRenderer` methods + `output/markdown.rs`/`syntax.rs`/`table.rs`
  renderers, `export_conversation` (or give it the sole session-export
  authority per P2-01), and the web-mode API types if web mode stays
  unsupported; remove `#![allow(dead_code)]`; fix the module doc.
- Regression validation: `cargo test -p echo-agent-app-core --lib` and
  `cargo test -p echo-agent-cli --lib` green after removal; grep of each
  deleted symbol returns zero non-test hits; `cargo check
  --no-default-features --features gui --bin echo-agent-tauri` green.
- Validation reports: [V01-01](../validations/A-OUT-01/V01-01.md),
  [V02-01](../validations/A-OUT-01/V02-01.md),
  [V04-02](../validations/A-OUT-01/V04-02.md),
  [V04-03](../validations/A-OUT-01/V04-03.md),
  [V05-01](../validations/A-OUT-01/V05-01.md)

### A-OUT-01-P3-02: `export_all_review_formats` short-circuits on the first failing format — already-written artifacts are discarded from the response

- Priority: P3
- Confidence: medium (static; behavior is a direct consequence of
  `Iterator::collect` on `Result`)
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/research.rs:1171-1191` —
  `formats.into_iter().map(|format| export_review(...)).collect()`; each
  `export_review` writes its file before returning (`:1153-1156`), so an
  earlier success is on disk when a later format fails (e.g., pandoc present
  but exits non-zero); the `Err` propagates and the already-written artifacts
  never reach the caller.
- Reachability: `/papers export <id> all`, GUI "export all", or the agent
  tool with `format = "all"` on a machine where Docx/PDF rendering fails.
- Expected invariant: a multi-format export either reports all produced
  artifacts or reports exactly what failed and what was produced.
- Observed behavior: the user receives only "Unable to export review: …"
  while the workspace already contains `systematic-review.md/.json/.csv/…`;
  the files are invisible to the caller until a re-listing.
- Impact: misleading failure reporting and orphaned artifacts the user does
  not know about; a retry re-writes them.
- Root cause: `collect::<Result<Vec<_>, _>>()` semantics applied to a
  side-effecting iterator without partial-result handling.
- Direction: loop with explicit per-format error capture and return
  `(Vec<ReviewExportArtifact>, Vec<(format, error)>)` (or fold errors into a
  report field); GUI/CLI print both the produced paths and the failed formats
  with causes.
- Regression validation: fixture with a pandoc stub that fails for Docx only —
  assert the response still contains the Markdown/JSON/CSV artifacts and an
  error entry for Docx.
- Validation reports: [V03-03](../validations/A-OUT-01/V03-03.md),
  [V04-04](../validations/A-OUT-01/V04-04.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search (formats, renderers, export paths, web API, both repos) | yes | passed (inventory: 1 authority, 2 divergent session exports, 4 dead format surfaces) | [V01-01](../validations/A-OUT-01/V01-01.md) |
| V02 | Registration and runtime reachability (Tauri invoke_handler, CLI registry, TUI enum, agent tool, frontend consumers) | yes | passed (availability matrix; TUI gap, dead export_conversation) | [V02-01](../validations/A-OUT-01/V02-01.md) |
| V03 | UTF-8/large-content scan (table geometry, truncations, streaming, export content) | yes | failed (TUI byte/char mismatch; /export unfiltered) | [V03-01](../validations/A-OUT-01/V03-01.md) |
| V03 | Missing external converter behavior (pandoc/quarto/engine absent, stderr, surface mapping) | yes | passed (causes retained on all surfaces) | [V03-02](../validations/A-OUT-01/V03-02.md) |
| V03 | Artifact identity and cross-surface delivery (ReviewExportArtifact, single authority, partial failure) | yes | failed (partial-failure reporting; TUI absence; session-export divergence) | [V03-03](../validations/A-OUT-01/V03-03.md) |
| V03 | Dynamic format-semantics reproduction (byte vs char width, /tmp scratch, repo untouched) | yes | passed (mismatch confirmed numerically) | [V03-04](../validations/A-OUT-01/V03-04.md) |
| V04 | `cargo test -p echo-agent-app-core --lib --locked output` | yes | passed, exit 0 (26 passed) | [V04-01](../validations/A-OUT-01/V04-01.md) |
| V04 | `cargo test -p echo-agent-app-core --lib --locked export` | yes | passed, exit 0 (83 passed incl. latex) | [V04-02](../validations/A-OUT-01/V04-02.md) |
| V04 | `cargo test -p echo-agent-app-core --lib --locked profiles` | yes | passed, exit 0 (10 passed) | [V04-03](../validations/A-OUT-01/V04-03.md) |
| V04 | `cargo test -p echo-agent-app-core --lib --locked research` | yes | passed, exit 0 (6 passed, 2 ignored — live-provider smokes) | [V04-04](../validations/A-OUT-01/V04-04.md) |
| V04 | `cargo test -p echo-agent-cli --lib --locked -- tui::markdown` | yes | passed, exit 0 (7 passed) | [V04-05](../validations/A-OUT-01/V04-05.md) |
| V05 | Historical-document drift (MASTER-PLAN, architecture.md, README, module docs) | yes | passed (4 stale claims classified) | [V05-01](../validations/A-OUT-01/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN.md:47 — review export complete (md/json/csv/bib/ris under `reports/`) | current | research.rs:1123-1169; V04-04 |
| MASTER-PLAN.md:52 — Pandoc/Quarto discovery + selectable PDF engine + "portable-format fallback" | current (discovery) / partially stale (no automatic fallback; errors by design) | research.rs:1215-1233,1285-1290; V03-02 |
| MASTER-PLAN.md:54 — "conversation export remains canonical" | stale | export_conversation unused; live export is unfiltered `/export` (P2-01/P3-01) |
| MASTER-PLAN.md:273-274 — real pandoc+typst DOCX/PDF through the app export path | current-shaped | pandoc fixture test; V04-04 |
| architecture.md:120 — `Research.output_format` field | stale (inert) | P2-03; V05-01 |
| README.md:386-387 — `/export` 导出会话, `/output` 切换输出格式 | current (`/export` exists) / stale (`/output` no-op) | P2-01, P2-03; V05-01 |
| `output/mod.rs:6-7` — OutputRenderer is the single facade for REPL and TUI | stale | TUI has own renderer; REPL subset only (P3-01) |
| `output/format.rs:8` — OutputFormat for `--output`/`-o` flag | stale (no flag) | P2-03; V05-01 |

## Coverage And Uncertainty

- All behavior claims are static call-graph/format-semantics proofs; no EKO
  process was launched (read-only review). The V03-04 numeric check ran a
  standalone `rustc` binary in /tmp — outside both repositories.
- `/export` path-traversal via the export name (`~/.eko/exports/<name>` — a
  `..` in the name escapes the exports dir) is recorded as residual risk, not
  a finding: user-initiated, local-only, own-content overwrite (consistent
  with the local threat model; X-AUT-01 territory).
- `render_bibtex` does not escape `{`/`}`/`\` in titles/authors, so unusual
  titles can produce malformed `.bib` entries; low impact, folded into
  coverage (the five formats are regenerable; no panic).
- The TUI streaming re-render (`tui/mod.rs:1207` re-parses the whole
  accumulated message per chunk) is O(n^2) for long outputs — recorded, not
  promoted (bounded by message size; Q-PERF-01 may measure).
- The GUI session export path was not exercised end-to-end (no frontend
  consumer exists — that IS the finding); the research GUI export was
  verified at the IPC/component level, not by clicking the UI (Q-E2E-01).
- Live pandoc/quarto presence was not probed on this machine; the
  missing-converter and present-converter behaviors are code-traced, and the
  fixture test covers the present case.
- `ipc_error` mapping drops the `ResearchError` variant but keeps the message;
  error causes are preserved on all surfaces (V03-02).

## Handoff

- Conclusions downstream tasks may rely on: review export is a single,
  tested authority with consistent `ReviewExportArtifact` identity and
  correct missing-converter error behavior; session export is duplicated and
  unfiltered (P2-01); the TUI lacks the research/export domain (P2-04); the
  format/profile machinery is decorative (P2-03) and a dead-code cluster is
  masked by `#![allow(dead_code)]` (P3-01); `export_all_review_formats` has
  partial-failure reporting (P3-02); TUI table rendering breaks on CJK/emoji
  (P2-02, dynamically confirmed).
- Reports to read: this report, its 12 validation reports, and dependency
  reports A-STATE-01 (store roots / fork filter class), A-TOOL-01 (surface
  wiring), F-EXT-03 (data export honesty: `export_data` truncation P3-02 and
  enum-fallback P2-02 intersect with P2-03's honesty theme).
- Conditions that make this report stale: changes to `output/` or `export/`
  or `profiles/` modules, `cmd_impls/advanced.rs` (/export /output),
  `research.rs` export paths, `tui/commands.rs` or `tui/events.rs` slash
  handling, `tui/markdown.rs` table geometry, Tauri research/conversation
  commands, or the frontend papers/conversation API.
- Follow-up task IDs: A-DOM-01 (research pipeline/export policy — P2-04 and
  P2-03 research parts), X-SRF-01 (per-surface capability matrix — P2-04,
  P3-01), X-AUT-01 (/export name traversal residual), Q-PERF-01 (TUI
  streaming re-render), X-BND-01 (session-export duplicate authority, web-mode
  API types), A-FE-01 (web-mode fallback branches — which endpoints lack a
  server). Fixes are deferred to the iteration roadmap; this review is
  read-only.
