# A-PROJ-01: Project indexing, diff, and coding workspace services

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: `echo-agent` clean; `echo-agent-cli` clean except the documented ts-rs codegen side effect of `cargo test` (`web-frontend/src/generated/*.ts`, 79 modified, same pattern as A-TSK-05). Scratch replicas live in /tmp only.

## Question

Are project indexing, diff, coding commands, and workspace state derived from current files without stale caches or a second worktree/file authority?

**Answer: No — three separate defect classes. (1) There is no live project index at all: the only `ProjectIndex` type is dead code whose doc advertises a persistence/refresh cache that does not exist, and the only live "index" (TUI `@`-completion `project_files`) is a boot-time snapshot that is never invalidated, so it silently misses files created during a session. (2) Diff has three live, divergent implementations (framework `DiffTool`, app-core `diff.rs` for the REPL, and an inline algorithm in the Tauri `diff_file` command for the GUI) with duplicate `DiffHunk`/`DiffLine` type families and a dead frontend `DiffViewer` twin; the GUI engine emits one giant hunk per file with incorrect hunk headers. (3) The REPL coding-mode change tracker (`FileChangeTracker`) is write-only, so `/code-review`'s fallback always reports "No file changes", and its root is captured once at boot. Workspace-switch behavior for project context is compliant — the `eko:project-context` projection is refreshed with the new workspace root on switch (state.rs:883) and cleared on exit (state.rs:1072), and the GUI resolves its file-browser root from `current_workspace()` per call — but the A-CFG-01 CWD defects (P1-01/P1-02) keep cwd-derived discovery bound to the exited workspace after exit. No second worktree/file authority exists in this task's paths: the git-worktree authority remains A-TSK-05's single `task_runtime/worktree.rs`.**

## Scope

Primary source paths inspected (deep read):

- `echo-agent-cli/echo-agent-app-core/src/project/` (full): `index.rs` (ProjectIndex), `context.rs` (ProjectContext, discover_project_root, load_git_context, file-tree summary), `prompt.rs` (PromptAssembler, `refresh_project_context_projection`), `gitignore.rs` (GitIgnore), `file_tracker.rs` (FileChangeTracker), `coding_loop.rs` (CodingLoop), `detector.rs` (ProjectType), `test_runner.rs`, `mod.rs`.
- `echo-agent-cli/echo-agent-app-core/src/diff.rs` (full, 528 lines): generate/parse/render unified diff.
- `echo-agent-cli/echo-agent-app-core/src/workspace/` (full): `registry.rs` (WorkspaceRegistry, registry.json index, detect_from_cwd/manifest), `mod.rs` (Workspace/WorkspaceId/Kind), `layout.rs` (cross-read).
- `echo-agent-cli/echo-agent-app-core/src/state.rs` (:844-1032 switch_workspace, :1053-1185 exit_workspace, :413 registry), `infra.rs` (:180-260 agent build, :440-445/:528-531 refresh_dynamic_context, :901 tool registration, :1873 project discovery), `agent_pool.rs:553` (pool refresh).
- `echo-agent-cli/src/cli/cmd_impls/coding.rs` (full), `diff_cmd.rs` (full), `git.rs` (diff/status), `cli/repl.rs` (:118-204), `src/tui/mod.rs` (:1675-1709 collect_project_files, :1966), `src/tui/events.rs` (:1287, :1632-1662 file-reference completion), `src/tauri/commands/files.rs` (diff_file :267-380, workspace_changes :211-231, get_workspace_root :451-457, write guard :180-208), `src/tauri/mod.rs` (:119-125 registration).
- `echo-agent-cli/web-frontend/src/stores/fileStore.ts` (full), `components/file-browser/FileBrowser.tsx` + `DiffViewer.tsx`, `components/coding/DiffViewer.tsx` (+ TestRunnerPanel/GitLogPanel usage search), `api/endpoints.ts` (:772-826 diff/changes endpoints).
- Framework (V01 cross-search only): `echo-tools/src/files/diff.rs` (DiffTool), `files/repo_map.rs`, `registry.rs` (:27-50, :214-271), `git.rs` (git diff/status tools).

## Out Of Scope

- Worktree lifecycle/merge/ownership (EKO `task_runtime/worktree.rs`) — A-TSK-05 (its P2-01..P2-04 consumed as cross-references; only the diff-summary consumer boundary is rechecked).
- Framework worktree/file/git tool correctness (F-EXT-02 P1-01..P3-04, incl. the doublestar defect class) — cross-referenced; the EKO `GitIgnore` `**` defect is this task's own instance (P3-01).
- Config/workspace-switch CWD semantics — A-CFG-01 (P1-01/P1-02 consumed, not re-reviewed; this task adds only the project-context refresh verification).
- Instruction/rules projection (unified_memory / InstructionProvider) — A-MEM-01/A-CFG-01.
- Frontend rendering details of chat/file panels beyond diff endpoints — A-FE-01/02, A-SRF-03.
- `workspace_routing.rs` prompt content — A-MEM-01/A-SUB-01.
- Framework `repo_map`/grep/glob tool internals (F-EXT-02/F-EXT-03 scope); only registration and role vs EKO ProjectIndex checked here.

## Inputs

- Root `AGENTS.md` (UTF-8/panic rules, no-duplicate-authority, framework-vs-app layering, worktree conventions, dead-code cleanup), shared `README.md`, `REPORTING.md`, `TASKS.md` (A-PROJ-01 card), `zcode-ds/README.md`, templates.
- Dependency task reports read in full: `A-CFG-01` (complete — workspace switch/exit CWD, config scopes; P1-01/P1-02/P2-02 consumed), `A-TSK-05` (complete — worktree/file authority; P2-04 panels.rs duplication consumed), `F-EXT-02` (complete — framework tool defects incl. UTF-8 panic P1-01 and glob `**` P2-01; consumed as reference classes).
- Historical documents treated as hypotheses: `echo-agent-cli/docs/MASTER-PLAN.md` (Coding section :112-120, :43), `docs/2026-07-11-tui-parity-design.md` (:50 file-reference completion), `docs/2026-07-28-app-core-full-audit.md` (:297 diff.rs zero-framework-dependency), `docs/configuration.md` (workspace mentions), `workspace/mod.rs:15` (SQLite wording), `project/index.rs:44-46` (module self-doc).

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism (framework, correctly placed) | `DiffTool` (`echo-tools/src/files/diff.rs`) and `RepoMapTool` (`files/repo_map.rs`) — model-facing read-only tools, registered (`registry.rs:38/236`, `:31/222`); `git_diff`/`git_status` etc. (`registry.rs:263-271`). Framework `similar`-based diff engine is the natural single diff authority. |
| EKO product policy (application) | `ProjectIndex` (dead), TUI `collect_project_files` snapshot, app-core `diff.rs` (REPL rendering), `diff_file`/`workspace_changes` GUI commands, `FileChangeTracker`/`CodingLoop`, `GitIgnore`, `WorkspaceRegistry`, `ProjectContext` structural summary + git-context projection. |
| Adapter boundary | `diff_file` (files.rs:267-380) is an adapter surface that re-implements diff hunking inline instead of delegating to a shared engine — an application-side duplicate of a generic mechanism (P2-03). `get_workspace_root` is a thin, lossless adapter over `current_workspace()` — compliant. |
| Duplicate search | Terms (both repos, V01-01): `ProjectIndex`, `collect_project_files`/`project_files`, `generate_unified_diff`/`parse_unified_diff`/`render_diff_ansi`/`render_diff_html`, `FileDiff`/`DiffHunk`/`DiffLine`/`DiffStats`, `TextDiff`/`udiff`, `DiffViewer`, `FileChangeTracker`/`record_file_write`/`record_file_delete`/`clear_changes`, `with_type`, `GitIgnore`/`is_ignored`/`should_ignore_path`, `WorkspaceRegistry`/`detect_from_cwd`/`detect_from_manifest`, `git status`/`--porcelain`/`diff --stat` parsers, `run_lint_command`/`format_failures_as_prompt`/`status_summary`. Results: one workspace registry; three diff engines; two EKO file-index implementations (one dead, one boot-snapshot); one write-only tracker; no second worktree/file authority in these paths (A-TSK-05 remains the single worktree authority). |
| Migration deletion | When P2-03 is fixed: delete the inline engine in `files.rs:310-372`, the duplicate `DiffHunk`/`DiffLine` types (files.rs:40-63), the dead `components/coding/DiffViewer.tsx`, and re-point the GUI at the shared engine. When P2-01 is fixed: delete `project/index.rs` (or wire it). When P2-04 is fixed: delete `with_type`, `status_summary`, `run_lint_command`, `format_failures_as_prompt`, and either wire or delete `FileChangeTracker`. |

## Current Path

Verified call graph (V01-01/V02-01; details in those reports):

1. **Project context (live, fresh each time)**: `discover_project_root` (utils.rs:12-40, VCS marker walk from cwd or explicit root) → `load_project_context` (context.rs:32-47: fresh 3-level file-tree walk + `GitIgnore::load`) → injected as the `eko:project-context` projection via `refresh_project_context_projection` (prompt.rs:352-367, live `git status --short` + `git diff --stat` per call, context.rs:91-122). Refresh sites: agent creation (infra.rs:444), workspace switch (state.rs:883, new workspace root), exit (state.rs:1072, cleared), pool refresh (agent_pool.rs:553).
2. **Workspace switch**: GUI switch IPC → `AppState::switch_workspace` (state.rs:844-1032) chdirs to the workspace root, sets agent working_dir + artifact dir, refreshes the projection with the new root; `get_workspace_root` (files.rs:451-457) resolves per IPC call from `current_workspace()`. `exit_workspace` (state.rs:1053-1185) resets stores and clears the projection but never restores CWD (A-CFG-01-P1-02, cross-ref).
3. **GUI file diff (live)**: FileBrowser → `filesApi.diff` (endpoints.ts:815-818) → `diff_file` (files.rs:267-380): old content `git show {ref}:{path}` in `spawn_blocking`, new content fresh `read_to_string`, inline `TextDiff::from_lines` + `iter_all_changes`, single hunk; changes list via `workspace_changes` (fresh `git status --porcelain`), frontend polls every 2.5 s and after saves (fileStore.ts:38-45, :205).
4. **REPL coding commands**: `/diff` (diff_cmd.rs) → app-core `diff.rs` engine or live `git diff`; `/test`/`/fix` → `run_test_command` (`sh -c`, test_runner.rs:27-54); `/code-review` → live `git diff HEAD` with a dead `diff_summary()` fallback (coding.rs:346-361); all rooted at the boot-captured `CodingLoop.project_root` (repl.rs:183-189).
5. **TUI `@`-completion**: `project_files = collect_project_files(".", 10_000)` once at boot (mod.rs:1966), consumed by `complete_file_reference` on Tab (events.rs:1287, :1632-1662); never rebuilt.
6. **Framework model tools**: `diff`, `repo_map`, `git_diff`/`git_status` registered for every EKO agent via `register_all_tools` (infra.rs:901; registry.rs:38/236, :31/222, :263-271) — the agent-facing exploration/diff authority, separate from both UI engines.
7. **Dead paths**: `ProjectIndex` (no constructor anywhere), `FileChangeTracker` writers (none), `should_ignore_path`/`is_ignored` (none), `detect_from_cwd`/`detect_from_manifest` (none), `run_lint_command`/`format_failures_as_prompt` (none), `components/coding/DiffViewer.tsx` (none).

## Findings

### A-PROJ-01-P2-01: `ProjectIndex` is dead code whose module doc advertises a startup build, on-demand refresh, and `~/.eko/cache` persistence that no code performs — EKO has no live project index, and the type offers no invalidation API at all

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/project/index.rs:43-46` (doc: "Built once at startup and refreshed on demand. Can be serialized to `~/.eko/cache/{project_hash}.json`"), `:47-61` (struct with `built_at`/`modified`), `:64-78` (`load`/`save`), `:81-129` (`build`), `:197-207` (`rebuild_maps`); zero production references in either repository — the only constructors are the module's own tests (`index.rs:459, :482`); no code writes or reads `~/.eko/cache/` (V01-01).
- Reachability: none — no definition-site caller; compiled into app-core but never constructed or registered anywhere.
- Expected invariant: a public module whose doc describes runtime behavior must be live, or must not exist; an index must have a defined build/refresh/invalidation lifecycle (AGENTS.md: no misleading APIs, dead code deleted).
- Observed behavior: the type has no `refresh`/`invalidate` method, no mtime comparison (V03-01); `built_at` is written but never read; `modified` values would go stale immediately if the index were ever wired; the documented cache file is never written.
- Impact: answers the task's "index lifecycle and invalidation" question in the negative — the only EKO project-index code is a dead, misleading duplicate of the live framework exploration tools (`repo_map`, `grep`, `glob`); a future implementer wiring it up would inherit a staleness-by-design structure with no invalidation path.
- Root cause: the index was written for an early "context assembly" design that was superseded by the projection/`PromptAssembler` path (project/prompt.rs), and the module was never deleted or rewired.
- Direction: delete `project/index.rs` per AGENTS.md dead-code cleanup, or — if context assembly is reintroduced — implement `refresh` comparing `FileInfo.modified` against disk mtimes and a cache keyed by project identity; update or remove the module doc accordingly.
- Regression validation: after deletion, `grep -rn "ProjectIndex" echo-agent echo-agent-cli` returns zero hits; existing context-assembly tests stay green (V04-01).
- Validation reports: [V01-01](../validations/A-PROJ-01/V01-01.md), [V02-01](../validations/A-PROJ-01/V02-01.md), [V03-01](../validations/A-PROJ-01/V03-01.md)

### A-PROJ-01-P2-02: The only live project-file index — TUI `@`-completion `project_files` — is a boot-time snapshot that is never invalidated or rebuilt, so files created during a session never appear and deleted files linger

- Priority: P2
- Confidence: high (mechanism); medium (impact frequency)
- Layer: application
- Evidence: `echo-agent-cli/src/tui/mod.rs:1966` — `app.project_files = collect_project_files(std::path::Path::new("."), 10_000);` is the single assignment in the codebase (V01-01); walker `mod.rs:1675-1709` (sync `std::fs::read_dir` recursion, skips `.git`/`.worktrees`/`target`/`node_modules`, 10 000-entry cap, symlinks not followed via `file_type()`); sole consumer `complete_file_reference` (`tui/events.rs:1287` Tab dispatch, `:1632-1662`).
- Reachability: every TUI session — Tab on an `@` token (TUI is a default headless surface); exercised by a unit test (`events.rs:5630`).
- Expected invariant: the completion index reflects current files — a file the agent just created is completable, a deleted file is not (tui-parity-design.md:50 claims this feature as "完成").
- Observed behavior: the list is fixed for the session; files written by the agent (writer subagents, `write_file`/`edit_file` calls) during a long coding session are absent from `@`-completion; deleted/renamed files stay listed; the 10k-entry sync walk also blocks TUI startup on large repositories.
- Impact: silent capability degradation on the primary headless surface — the model/user cannot reference newly created files by name and is offered stale paths; startup latency on large repos. This is the task's "stale cache" question answered in the affirmative for the TUI surface.
- Root cause: completion was implemented as a boot-time snapshot with no refresh trigger; no write-event or on-demand rebuild exists, and the TUI has no workspace-switch surface (A-CFG-01-P1-03) that could have motivated a rebind.
- Direction: rebuild `project_files` on demand (on `@` prefix, bounded re-walk or dir-mtime keyed cache), or subscribe to the file-write events EKO already emits; when a TUI workspace surface is added (A-CFG-01-P1-03), rebind on switch.
- Regression validation: TUI fixture — session starts with file A; a tool writes file B; `@`-completion must offer B and must drop a file deleted mid-session; large-repo fixture asserts bounded walk time.
- Validation reports: [V01-01](../validations/A-PROJ-01/V01-01.md), [V02-01](../validations/A-PROJ-01/V02-01.md), [V03-01](../validations/A-PROJ-01/V03-01.md)

### A-PROJ-01-P2-03: Diff has three live, divergent implementations — framework `DiffTool`, app-core `diff.rs` (REPL), and an inline engine in the GUI `diff_file` — with duplicate type families and a dead frontend `DiffViewer` twin; the GUI engine emits one giant hunk per file with incorrect hunk headers

- Priority: P2
- Confidence: high (code facts; GUI hunk structure replica-verified in V04-04)
- Layer: application (GUI/REPL surfaces) with a framework participant (`DiffTool`)
- Evidence:
  - Engine 1 (framework, model-facing): `echo-tools/src/files/diff.rs` (`DiffTool`, `similar::udiff::unified_diff`), registered `registry.rs:38/236`.
  - Engine 2 (app-core, REPL): `echo-agent-cli/echo-agent-app-core/src/diff.rs:70-163` (`generate_unified_diff` with `TextDiff::grouped_ops(context)`, trims trailing newlines at :102/:112/:121; `parse_unified_diff` :168-270), sole consumer `src/cli/cmd_impls/diff_cmd.rs:14-16`.
  - Engine 3 (GUI): `src/tauri/commands/files.rs:310-372` — inline `TextDiff::from_lines` + `iter_all_changes` with no hunk grouping (`current_hunk_lines` never reset, one `hunks.push` at the end :364-371), hunk counts tracking only changed lines (:331-338), raw `change.value()` including trailing newline (:360), plus duplicate `DiffHunk`/`DiffLine` types (files.rs:40-63 vs diff.rs:14-61).
  - Frontend: live hunks-based `components/file-browser/DiffViewer.tsx` (FileBrowser.tsx:242) vs dead string-based `components/coding/DiffViewer.tsx` (zero usages, V01-01).
  - Replica (V04-04): 10-line file with one replaced line → GUI engine emits one hunk `@@ -1,1 +1,1 @@` containing all 11 lines — header counts reflect only changed lines, not the hunk's actual content.
- Reachability: GUI file-browser diff view (every `diff_file` call, registered tauri/mod.rs:122, consumed endpoints.ts:815-818 → fileStore.ts:116-143); REPL `/diff` (diff_cmd.rs); agent-facing `diff` tool for every EKO agent.
- Expected invariant: one diff authority with standard unified-diff hunk structure; the same file renders equivalently on every surface (AGENTS.md: no parallel authorities; REPORTING: duplicate-authority class).
- Observed behavior: three engines with divergent semantics (context grouping, newline trimming, hunk structure); GUI diffs of large files render the whole file as one hunk; GUI hunk headers misstate counts; fixes (e.g. F-EXT-02-P2-01 class, binary handling) must be applied three times; the model-facing tool and the GUI can disagree about the same change.
- Impact: inconsistent user-visible diffs across surfaces, misleading hunk headers, quadratic rendering for large files, and a duplicated maintenance surface — the exact "second authority" defect AGENTS.md forbids.
- Root cause: each surface grew its own diff convenience code; the shared app-core engine predates the GUI command, and the GUI command was written against its own DTO types instead of reusing it.
- Direction: make app-core `diff.rs` the single engine — have `diff_file` (files.rs) call `generate_unified_diff`/`parse_unified_diff` and map to its DTO, or reuse the framework `DiffTool` output; add hunk-context grouping and correct header counts to the GUI path; delete the duplicate `DiffHunk`/`DiffLine` types and the dead `components/coding/DiffViewer.tsx`.
- Regression validation: fixture repo — the same file diffed via `/diff`, GUI `diff_file`, and framework `diff` must produce identical hunks and header counts; a large-file fixture (10k lines, one change) must render a bounded hunk.
- Validation reports: [V01-01](../validations/A-PROJ-01/V01-01.md), [V03-01](../validations/A-PROJ-01/V03-01.md), [V04-04](../validations/A-PROJ-01/V04-04.md)

### A-PROJ-01-P2-04: The REPL coding-mode change tracker is write-only — `/code-review`'s fallback always reports "No file changes", `CodingLoop`'s root is captured once at boot and never rebound, and most of the tracker API is dead

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `FileChangeTracker::record_change/record_write/record_delete/clear_changes` (`file_tracker.rs:39-87`) and `CodingLoop::record_file_write/record_file_delete/clear_changes/change_count/status_summary/with_type` (`coding_loop.rs:28-85`) have zero production callers (V01-01); the only consumer of the tracker is `diff_summary()` in the `/code-review` git-failure fallback (`cmd_impls/coding.rs:358-360`), which therefore always returns "No file changes"; `CodingLoop::new(&project_root)` captures the root once at session boot (repl.rs:183-189); `run_lint_command`/`format_failures_as_prompt` (`test_runner.rs:57-67, :184-203`) also have zero callers.
- Reachability: `/code-review` in any REPL session where `git diff HEAD` fails (fresh repository without commits, git unavailable, non-git dir) — the fallback prints a false "No file changes"; `/test`/`/fix` are live but root-bound to the boot directory.
- Expected invariant: a command named "review accumulated changes" reflects the changes actually made; a tracking API must be fed or removed (AGENTS.md: no dead code, no misleading surfaces).
- Observed behavior: in the fallback case the command claims there is nothing to review while the user's edits exist on disk; the tracker can never accumulate anything because nothing records into it; `status_summary` and `with_type` are unreachable.
- Impact: misleading REPL output in a common early-repo scenario; a dead API family that invites future miswiring (the same trap class as A-CFG-01-P2-01's orphan store).
- Root cause: the tracker was built for an early "coding loop" design; the write-hook into the agent's file tools was never implemented, and `/code-review` was later rewritten around `git diff HEAD`, leaving the fallback as the only (empty) consumer.
- Direction: either wire `record_file_write/delete` into the agent's file-tool result path (or shell hooks) so the tracker is real, or delete the tracker + `with_type` + `status_summary` and make `/code-review`'s fallback an explicit error ("not a git repository / no HEAD"); delete `run_lint_command`/`format_failures_as_prompt` if unused.
- Regression validation: fixture — non-git directory, `/code-review` after a file edit must state explicitly that there is no git HEAD (not "No changes to review"); after wiring, the tracker count reflects the edit.
- Validation reports: [V01-01](../validations/A-PROJ-01/V01-01.md), [V02-01](../validations/A-PROJ-01/V02-01.md), [V03-01](../validations/A-PROJ-01/V03-01.md)

### A-PROJ-01-P3-01: `GitIgnore::globstar_match` panics on UTF-8 (multibyte) paths via `&remaining[j..]` byte slicing and mis-matches `**` patterns even on ASCII paths — latent, but one wiring step from violating AGENTS.md's strictest invariant

- Priority: P3
- Confidence: high (panic replicated verbatim in V04-04); reachability is currently zero
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/project/gitignore.rs:178-183` — `for j in 0..=remaining.len() { let candidate = &remaining[j..]; ... }` byte-slices a `&str` at every offset; `:97-121` (`glob_matches`: `**` handled by `starts_with`/`ends_with` text matching); `should_ignore_path` (`project/context.rs:55-63`), the only production-facing entry, has zero callers (V01-01) — `GitIgnore::load` itself is reached via `load_project_context` (context.rs:39), so the type is constructed in production while the matching path is not.
- Reachability: none today; any future wiring of `should_ignore_path` (e.g. file-tool filtering — the documented purpose, context.rs:49-54) exposes the panic to every multibyte-path operation with a `**` rule.
- Expected invariant: AGENTS.md — no API that panics on abnormal input; UTF-8-safe iteration mandatory; gitignore `**` matches zero-or-more directory segments.
- Observed behavior: replica run — `globstar_match("src/**/*.rs", "src/中文.rs")` panics ("start byte index 5 is not a char boundary; it is inside '中'"); `globstar_match("src/**/*.rs", "src/main.rs")` returns `false` (real files never match), so both the panic and the `**` semantics are wrong (same defect class as F-EXT-02-P2-01's doublestar).
- Impact: latent run-abort (panic propagates into the agent run — no `catch_unwind` barrier, F-EXT-02 V02) the moment the ignore check is wired into file tooling; silently wrong ignore decisions in the meantime.
- Root cause: `globstar_match` reuses byte offsets for `&str` slicing (AGENTS.md rule violation), and `**` was implemented by text prefix/suffix rather than segment matching.
- Direction: rewrite matching with char-boundary-safe iteration (`char_indices`/`match_indices`, or compile to a regex) and correct segment semantics; add multibyte + `src/**/*.rs` fixtures; or delete `GitIgnore` until a consumer exists.
- Regression validation: unit tests — `is_ignored("src/中文.rs", false)` with a `**` rule must return without panicking; `src/**/*.rs` must match `src/main.rs` and `src/a/b/main.rs`; existing gitignore tests stay green (V04-01).
- Validation reports: [V01-01](../validations/A-PROJ-01/V01-01.md), [V03-01](../validations/A-PROJ-01/V03-01.md), [V04-04](../validations/A-PROJ-01/V04-04.md)

### A-PROJ-01-P3-02: Documentation drift — the workspace layout doc still says "后台任务 SQLite DB", the tui-parity "完成" claim hides a boot-snapshot design, and `project/index.rs` self-doc describes behavior that does not exist

- Priority: P3
- Confidence: high
- Layer: application (docs)
- Evidence: `workspace/mod.rs:15` — "`tasks/` # 后台任务 SQLite DB" (no SQLite in EKO; same family as A-CFG-01-P3-01, cross-referenced, not duplicated); `docs/2026-07-11-tui-parity-design.md:50` — "@query + Tab 在项目文件索引中补全 | 完成" — mechanism exists (events.rs:1632) but is a never-refreshed boot snapshot (P2-02); `project/index.rs:44-46` — "Built once at startup and refreshed on demand ... serialized to `~/.eko/cache/{project_hash}.json`" — no such behavior exists (P2-01).
- Reachability: documentation-only.
- Expected invariant: docs and module docs describe real behavior (AGENTS.md: no misleading public API/comments).
- Observed behavior: three doc sites misrepresent current code; the workspace doc repeats the removed SQLite story; the completion doc overstates the feature.
- Impact: maintenance confusion and wrong expectations for implementers; minor relative to the code findings.
- Root cause: docs predate the projection-based context design, the SQLite removal, and the snapshot-based completion implementation.
- Direction: fix `workspace/mod.rs:15` wording, mark the tui-parity completion row as "boot snapshot, refresh pending (P2-02)", and remove/replace the `index.rs` self-doc with the code (or delete the file, P2-01).
- Regression validation: grep for `SQLite DB` in EKO workspace docs returns zero hits; `index.rs` doc matches code after P2-01.
- Validation reports: [V05-01](../validations/A-PROJ-01/V05-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition + duplicate search (index/diff/tracker/registry authorities, both repos; framework git tools cross-check) | yes | passed (duplicates found and classified) | [V01-01](../validations/A-PROJ-01/V01-01.md) |
| V02 | Registration + runtime reachability (GUI diff/changes, REPL commands, TUI completion, framework tools, switch/exit refresh) | yes | passed | [V02-01](../validations/A-PROJ-01/V02-01.md) |
| V03 | Invariant/edge cases: index lifecycle & invalidation; diff single source of truth; workspace switch; large-repo and conflicting-file fixtures | yes | failed (violations → P2-01..P2-04, P3-01; switch refresh and write guards verified compliant) | [V03-01](../validations/A-PROJ-01/V03-01.md) |
| V04 | `cargo test -p echo-agent-app-core --locked project` | yes | passed (exit 0, 25 ok) | [V04-01](../validations/A-PROJ-01/V04-01.md) |
| V04 | `cargo test -p echo-agent-app-core --locked diff` | yes | passed (exit 0, 14 ok) | [V04-02](../validations/A-PROJ-01/V04-02.md) |
| V04 | `cargo test -p echo-agent-app-core --locked workspace::registry` | yes | passed (exit 0, 7 ok) | [V04-03](../validations/A-PROJ-01/V04-03.md) |
| V04 | Scratch replicas: `globstar_match` UTF-8 panic + `**` mismatch; `diff_file` single-hunk structure | yes | passed (panic exit 101 reproduced; single-hunk/wrong-count reproduced) | [V04-04](../validations/A-PROJ-01/V04-04.md) |
| V05 | Historical-document drift (MASTER-PLAN Coding, tui-parity, app-core audit, module self-docs) | yes | passed (drift classified) | [V05-01](../validations/A-PROJ-01/V05-01.md) |

All required validations executed; every command has a known exit code; no validation is pending.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `MASTER-PLAN.md:112-120` — LSP discovery + Tree-sitter repo map, UTF-8-safe text fallback | current | framework `repo_map` (`echo-tools/src/files/repo_map.rs`) registered (`registry.rs:31/222`) (V05-01) |
| `MASTER-PLAN.md:43` — "LSP automatic discovery and AST-aware repo map | Complete" | current | repo_map reachable via `register_all_tools` (infra.rs:901) (V05-01) |
| `docs/2026-07-11-tui-parity-design.md:50` — "@query + Tab 项目文件索引补全 \| 完成" | current (mechanism) / incomplete by design | completion live (events.rs:1287/:1632) but index is a boot snapshot, never invalidated → P2-02 |
| `docs/2026-07-28-app-core-full-audit.md:297` — "`output/`, `diff.rs`, `context_window.rs` — zero framework dependency" | current | app-core `diff.rs` uses `similar` directly, no `echo-agent` imports (V05-01) |
| `project/index.rs:44-46` — "Built once at startup and refreshed on demand ... `~/.eko/cache`" | stale (self-doc) | type is dead; no build/refresh/persistence anywhere → P2-01 |
| `workspace/mod.rs:15` — "`tasks/` # 后台任务 SQLite DB" | stale | no SQLite in EKO (A-CFG-01-P3-01 family, cross-ref) → P3-02 |
| `A-CFG-01-P1-01/P1-02` — switch/exit CWD misalignment | current (consumed, not duplicated) | switch chdirs + refreshes projection (state.rs:854/:883); exit never restores CWD (state.rs:1053-1185) → project-context arm verified compliant on switch, cwd-derived discovery stale on exit (V02-01/V03-01) |
| `A-TSK-05-P2-04` — panels.rs duplicate worktree authority | current (consumed) | this task found no additional worktree/file authority in project/diff paths (V01-01) |

## Coverage And Uncertainty

- No process was launched; every dynamic claim rests on verbatim scratch replicas (V04-04) or traced code paths. Q-* dynamic suites (Q-FLT-02, Q-STA-01) should add: TUI `@`-completion after an agent writes a file; GUI diff of a 10k-line file; `git show` failure path in `diff_file`; switch/exit with cwd assertions.
- `switch_workspace`/`exit_workspace` have zero unit tests (state.rs has no test module) — the switch-refresh compliance conclusion is static (V03-01).
- `diff_file`/`diff_cmd` have no Rust unit tests; the GUI hunk structure was verified by replica, not end-to-end rendering.
- `GitIgnore.is_ignored` is unreachable in production — the panic is latent; priority reflects that.
- Framework `repo_map` internals were not re-reviewed (F-EXT-02/F-EXT-03 scope); only registration and the distinct-purpose classification are claimed here.
- Frontend diff rendering (colors, whitespace) was not visually verified; only the data contract (hunks) was checked.
- The frontend 2.5 s `loadChanges`/`refreshSelectedFromDisk` poll makes the GUI changes list effectively live — no GUI stale-cache finding.

## Handoff

- Downstream tasks may rely on: project context is derived from current files at every refresh site and follows the workspace switch (state.rs:883) / is cleared on exit (state.rs:1072); the GUI file browser reads git state live (per-call + 2.5 s poll) with a revision-guarded, atomic write path (files.rs:182-205); the workspace registry is a single authority (state.rs:413/577) — no stale registry cache; EKO has **no** live project index (P2-01 dead, P2-02 boot snapshot); diff has three divergent engines (P2-03); the REPL change tracker is write-only (P2-04); `GitIgnore` matching panics on multibyte paths if ever wired (P3-01).
- Findings to fold into the roadmap: P2-01 (delete or wire `ProjectIndex`), P2-02 (on-demand TUI index rebuild), P2-03 (single diff authority; delete inline GUI engine + twin types + dead DiffViewer), P2-04 (wire or delete the tracker), P3-01 (gitignore matching rewrite), P3-02 (doc fixes).
- Reports to read: the 8 validation reports above; dependency reports A-CFG-01 (P1-01/P1-02/P2-02), A-TSK-05 (P2-01..P2-04), F-EXT-02 (P1-01, P2-01/P2-02/P2-03 defect classes).
- Conditions that make this report stale: changes to `project/index.rs`, `project/gitignore.rs`, `project/context.rs`/`prompt.rs` refresh logic, `tui/mod.rs` `collect_project_files`, `tui/events.rs` completion, `src/tauri/commands/files.rs` `diff_file`/`workspace_changes`, app-core `diff.rs`, `coding_loop.rs`/`file_tracker.rs`/`test_runner.rs`, `repl.rs` CodingLoop construction, or `state.rs` switch/exit refresh sites; also if a second project index or diff authority appears.
- Follow-up task IDs: A-FE-01/02 (DiffHunk DTO parity between `diff_file` and app-core `diff.rs` types), A-SRF-01 (TUI `@`-completion refresh + parity with A-CFG-01-P1-03), X-TOL-01 (diff tool conformance across model/REPL/GUI surfaces), Q-FLT-02/Q-STA-01 (globstar panic fixture, single-hunk large-file fixture, switch/exit cwd fixtures), S-RDM-01 (roadmap ordering of P2-01..P2-04). Fixes are deferred to the iteration roadmap; this review is read-only.
