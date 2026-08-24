# A-PROJ-01: Project indexing, diff, and coding workspace services

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: not-applicable (read-only inspection of framework surface only)
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only review)

## Question

Are project indexing, diff, coding commands, and workspace state derived
from current files without stale caches or a second worktree/file authority?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent-cli/echo-agent-app-core/src/project/index.rs` (488 lines,
  read in full) — `ProjectIndex`, `FileInfo`, `SymbolMatch`, `build`,
  `walk`, `rebuild_maps`, `search_symbols`, `recently_modified`,
  `related_files`, `load`, `save`, `detect_language`,
  `extract_symbols_and_imports`, and the 3-test suite.
- `echo-agent-cli/echo-agent-app-core/src/project/context.rs` (200 lines,
  read in full) — `ProjectContext`, `discover_project_root`,
  `load_project_context`, `should_ignore_path`,
  `build_system_prompt_with_context`, `load_git_context`,
  `generate_file_tree_summary`, `collect_dir_entries`, `SKIP_DIRS`.
- `echo-agent-cli/echo-agent-app-core/src/project/gitignore.rs` (249 lines,
  read in full) — `GitIgnore`, `IgnorePattern`, `load`, `parse`,
  `is_ignored`, `glob_matches`, `simple_glob`, `globstar_match`, and the
  5-test suite.
- `echo-agent-cli/echo-agent-app-core/src/project/file_tracker.rs`
  (107 lines, read in full) — `FileChangeTracker`, `FileChange`,
  `ChangeType`, `record_change`/`record_write`/`record_delete`,
  `list_changes`, `diff_summary`.
- `echo-agent-cli/echo-agent-app-core/src/project/coding_loop.rs`
  (87 lines, read in full) — `CodingLoop`, `record_file_write`,
  `record_file_delete`, `diff_summary`, `status_summary`.
- `echo-agent-cli/echo-agent-app-core/src/project/detector.rs` (82 lines,
  read in full) — `ProjectType::detect`/`test_command`/`lint_command`.
- `echo-agent-cli/echo-agent-app-core/src/project/test_runner.rs`
  (204 lines, read in full) — `run_test_command`, `run_lint_command`,
  `parse_test_output`, `parse_rust_test_output`,
  `parse_generic_test_output`, `format_failures_as_prompt`.
- `echo-agent-cli/echo-agent-app-core/src/project/prompt.rs` (465 lines,
  read in full) — `PromptAssembler`, `refresh_project_context_projection`,
  `add_project_context`, `truncate_to_estimated_tokens`.
- `echo-agent-cli/echo-agent-app-core/src/diff.rs` (529 lines, read in
  full) — `FileDiff`, `DiffHunk`, `DiffLine`, `generate_unified_diff`,
  `parse_unified_diff`, `parse_hunk_header`, `parse_range`,
  `render_diff_ansi`, `render_diff_html`, `colorize_unified_diff`,
  `render_edit_diff`, and the 12-test suite.
- `echo-agent-cli/echo-agent-app-core/src/state.rs:409-450, 575-596,
  844-1050` — `WorkspaceState`, `AppState::from_shared` workspace init,
  `switch_workspace`, `exit_workspace`, `apply_workspace_routing`.
- `echo-agent-cli/echo-agent-app-core/src/workspace/registry.rs`
  (header + `RegistryIndex::load/save`, `WorkspaceRegistry` write paths
  at `:45`, `:418`) — workspace index persistence.
- `echo-agent-cli/echo-agent-app-core/src/infra.rs:240, 444, 528-531`
  — `refresh_dynamic_context` → `refresh_project_context_projection`
  wiring (the live project-context refresh path on workspace switch).
- `echo-agent-cli/src/cli/repl.rs:100-205` — REPL bootstrap: builds
  `ProjectContext` fresh, constructs `CodingLoop`, registers coding/diff
  commands.
- `echo-agent-cli/src/cli/cmd_impls/diff_cmd.rs` (304 lines, read in full)
  — the `/diff` command, all four modes (git / backup / two-file / html).
- `echo-agent-cli/src/cli/cmd_impls/coding.rs:280-420` — `/test`, `/review`
  commands and their diff source (`git diff HEAD` vs `CodingLoop.diff_summary`
  fallback).
- `echo-agent-cli/src/cli/cmd_impls/workspace.rs` (329 lines, read in full)
  — `/workspace new|list|switch|link|migrate|info`.
- `echo-agent-cli/src/tauri/commands/workspace.rs:131-180` — the Tauri
  `switch_workspace` IPC command (the only caller of
  `AppState::switch_workspace`).
- `echo-agent-cli/web-frontend/src/api/endpoints.ts:1422` and
  `web-frontend/src/components/file-browser/FileBrowser.tsx:12,103` —
  frontend workspace IPC and the (icon) `FileDiff` reference; verified no
  second Rust-side diff authority.
- Cross-repo duplicate search (V01) for `ProjectIndex`, `ProjectContext`,
  `generate_unified_diff`, `FileChangeTracker`, `switch_workspace`,
  `GitIgnore`, `is_ignored` across the whole `echo-agent-cli` repository.

## Out Of Scope

Deferred to downstream tasks:

- **A-CFG-01** (complete, read) — workspace switch's effect on config /
  hooks / watcher targets. This task only covers the project-index / diff /
  coding-loop consequence of the switch (which is: project context refresh
  via `refresh_dynamic_context`).
- **A-TSK-05** (complete, read) — the worktree / file-ownership / merge
  authority for the formal task runtime. That is a *separate* file-write
  authority (per-task fork worktrees) and is not in the project/diff/coding-
  command surface audited here. A-TSK-05 established that the worktree
  authority is single and well-defended.
- **F-EXT-02** (complete, read) — framework file/git/worktree tool
  internals. This task only consumes their conclusion that the framework
  file tools do NOT call back into `CodingLoop.record_file_write`.
- **A-TOOL-01** — application adapter wiring of tools. Whether the agent's
  file tools could/should report writes to the change tracker is an
  A-TOOL-01 question; this task only records that they currently do not.
- **B-PATH-01** — full IPC handler inventory; only the workspace-switch IPC
  surface is touched here.

## Inputs

Required repository documents read:

- Repository root `AGENTS.md` — "动手前先查是不是已经有了" / no-duplicate
  rule; "代码清理:无需兼容,过时代码可直接删"; UTF-8 safety
  (`chars().take()`, no byte slicing); panic-safety (no `unwrap`/`expect`);
  framework-vs-application layering gate; TUI/GUI feature-parity mandate;
  local-personal-assistant threat model.
- `docs/comprehensive-review/REPORTING.md`,
  `docs/comprehensive-review/templates/task-report.md`,
  `docs/comprehensive-review/templates/validation-report.md`.

Dependency task reports read:

- **A-CFG-01** (complete) — established that workspace switch replaces
  storage/memory/skills but NOT config, and that `AppState.app_config` is a
  stale snapshot after the first edit. This task extends the switch
  analysis to the project-context / index / diff consequence.
- **A-TSK-05** (complete) — established that the formal task runtime's
  worktree/file-ownership/merge is a single, well-defended authority. This
  task confirms the *coding-loop* surface (an older, parallel "what
  changed" tracker) does NOT overlap with it and is in fact inert.
- **F-EXT-02** (complete) — established the framework file tools
  (`EditFileTool` / `WriteFileTool` / `DeleteFileTool`) do not call back
  into any application-layer change tracker. This task relies on that to
  explain why `FileChangeTracker` is never populated.

Historical documents treated as hypotheses: the `project/index.rs` module
doc (`:1-6, 46`) which advertises a metadata cache persisted to
`~/.eko/cache/{project_hash}.json` (verified stale, see P3-01).

## Layering Decision

This is an **application-layer** task with no framework touchpoints in the
inspected surface.

| Classification | Required answer |
|---|---|
| Generic mechanism | None of the inspected code is framework. `echo-agent` provides the `Tool` / git / file primitives (F-EXT-02); EKO layers project discovery, diff rendering, the coding loop, and workspace state on top. |
| EKO product policy | Correctly placed in `echo-agent-app-core` (`project::*`, `diff.rs`, `workspace::*`, `state.rs`): the very concept of a "project index", a "coding mode", a slash-command `/diff`, and per-workspace memory isolation are EKO desktop-assistant concerns. |
| Adapter boundary | The CLI `/diff`, `/test`, `/review`, `/workspace` commands and the Tauri `switch_workspace` IPC are thin adapters over `app-core` functions. The CLI `/workspace switch` adapter is *too* thin (P1-02): it does not call `AppState::switch_workspace`. |
| Duplicate search | Searched names (whole `echo-agent-cli` repo): `ProjectIndex`, `FileInfo`, `SymbolMatch`, `ProjectContext`, `generate_unified_diff`, `render_diff_*`, `FileDiff`, `FileChangeTracker`, `CodingLoop`, `switch_workspace`, `GitIgnore`, `is_ignored`. Results: (a) **`ProjectIndex` has ZERO callers outside its own `#[cfg(test)]` block** — dead public API (P1-01). (b) **`FileChangeTracker` / `CodingLoop.record_file_write` / `record_file_delete` have ZERO callers outside the `project/` module** — an empty second "what changed" authority (P2-01). (c) `generate_unified_diff` / `render_diff_*` have a single consumer surface (`src/cli/cmd_impls/diff_cmd.rs`); no second diff renderer in the GUI (the frontend `FileDiff` is a React icon, confirmed). (d) `AppState::switch_workspace` has a single caller (Tauri IPC `src/tauri/commands/workspace.rs:137`); the CLI `/workspace switch` path does not call it (P1-02). |
| Migration deletion | Three deletion candidates identified: `project/index.rs` (entire module, P1-01), `FileChangeTracker` + its `CodingLoop` wrapper methods (P2-01), and the stale `~/.eko/cache/{project_hash}.json` doc (P3-01). Per AGENTS.md "无需兼容,过时代码可直接删". |

## Current Path

Verified project / diff / workspace data flow at `echo-agent-cli` commit
`b3b2e81`:

```text
Project context (LIVE — the one production actually uses)
   REPL bootstrap (src/cli/repl.rs:119-128) or
   AppState::switch_workspace (state.rs:883) or
   subagent build (infra.rs:444)
     │
     ▼
   discover_project_root(path)        [context.rs:22 → utils.rs:12]
     ancestors().find(VCS_MARKERS)     # .git/.hg/.svn wins
     else ancestors().find(FALLBACK)   # .eko/Cargo.toml/package.json/...
     │
     ▼
   load_project_context(root)         [context.rs:32-47]
     ├─ name = root.file_name()
     ├─ file_tree_summary = generate_file_tree_summary(root)
     │     collect_dir_entries (max_depth=3, caps at 80 entries,
     │     skips SKIP_DIRS, hides dotfiles except .env.example)
     │     # FRESH READ EVERY CALL — no cache
     └─ gitignore = GitIgnore::load(root)   [.gitignore parse]
     │
     ▼
   PromptAssembler::project_context   [prompt.rs:285-320]
     ├─ P6: project_structure (token_budget = total/8, cap 4_000)
     └─ P7: git_context = load_git_context(root)   [context.rs:91-122]
         ├─ git status --short (sync subprocess)
         └─ git diff --stat   (sync subprocess)
         # FRESH READ EVERY CALL — no cache
     │
     ▼
   refresh_project_context_projection [prompt.rs:352-367]
     agent.context().replace_projection("eko:project-context", msg)
     # Called on bootstrap AND on every workspace switch


Project context (DEAD — never reached)
   ProjectIndex::build(root)          [index.rs:81-129]
     walk() recursively reads dir, extracts symbols/imports,
     caps files at 1 MB, skips target/node_modules/.next/...
     # ZERO production callers (only #[cfg(test)] at index.rs:459,482)
   ProjectIndex::save / load          [index.rs:66-78]
     ~/.eko/cache/{project_hash}.json
     # ZERO callers; cache dir never created


Diff rendering (single consumer: /diff CLI command)
   /diff                  → git diff --color=always      [diff_cmd.rs:57]
   /diff --staged         → git diff --cached --color=always
   /diff <file>           → generate_unified_diff(file, .bak, disk)  [diff_cmd.rs:131]
   /diff <f1> <f2>        → generate_unified_diff(label, disk, disk) [diff_cmd.rs:177]
   /diff --html [<file>]  → render_diff_html(parse_unified_diff(git diff))
   # AUTHORITY: git CLI + on-disk file reads. No in-memory cache.


Diff rendering (empty second authority — never populated)
   CodingLoop.file_tracker: FileChangeTracker  [coding_loop.rs:14]
     record_file_write / record_file_delete    [coding_loop.rs:47-54]
     # ZERO callers outside project/ module
     → diff_summary() always returns "No file changes"
   /review falls back to this ONLY if `git diff HEAD` fails  [coding.rs:352-361]
     → in practice always empty when reached


Workspace switch (GUI path — the one that actually switches state)
   Tauri switch_workspace(id)           [tauri/commands/workspace.rs:131-180]
     → AppState::switch_workspace(ws)   [state.rs:844-1032]
       1. workspace.current = Some(ws)
       2. std::env::set_current_dir(ws.root)   # process CWD
       3. agent.set_working_dir(Some(ws.root))
       4. refresh_dynamic_context(agent, Some(ws.root))   [infra.rs:528]
            → refresh_project_context_projection  # REBUILDS ProjectContext
            → refresh_memory_projections
       5. pool.apply_working_dir(...)
       6. persistence / conversation_store / runtime_state_store re-init
       7. memory store + layer manager rebind
       8. workspace-curated skills reload
       9. apply_workspace_routing (Skills + prompt by WorkspaceKind)
     # NO ProjectIndex to rebuild (it's dead).
     # Diff has no cache, so nothing to refresh.
     # ProjectContext IS rebuilt fresh from the new root (step 4).

Workspace switch (CLI path — does NOT switch state)
   /workspace switch <name>            [cmd_impls/workspace.rs:114-146]
     → registry.open_by_name(name)
     → println!("Switched to workspace: ...")
     # NEVER calls AppState::switch_workspace.
     # No CWD change, no agent working_dir update, no store swap,
     # no memory reload, no project-context refresh.
```

Invariants verified by this graph (full evidence in V01–V04):

- **The live project context is cache-less.** Every call to
  `load_project_context` / `generate_file_tree_summary` /
  `load_git_context` reads fresh from disk (V01). The projection is
  replaced on every workspace switch (V03). There is therefore no stale
  index to invalidate in the live path.
- **The advertised project index is dead.** `ProjectIndex` (488 lines,
  including a designed-but-unused JSON cache) has zero production callers
  (V01). The question "how is the index invalidated?" resolves to: there
  is no live index, so there is no stale-cache risk from it — only dead
  code (P1-01) and a misleading doc (P3-01).
- **The diff authority is single and on-disk.** `/diff` reads from `git
  diff` or compares files on disk via `similar::TextDiff` (V02). There is
  no second diff cache. The `FileChangeTracker` is a *nominal* second
  authority but is never populated (P2-01).
- **Workspace switch rebuilds project context on the GUI path.**
  `refresh_dynamic_context` (state.rs:883) re-runs
  `refresh_project_context_projection` against the new root (V03). The CLI
  `/workspace switch` does none of this (P1-02).

## Findings

The headline result is mixed-but-clean on the live path: the project
context and diff that production actually uses are derived from current
files with no stale cache and no second authority. However, the codebase
carries a **488-line dead `ProjectIndex`** (with a designed-but-unused
disk cache), an **empty `FileChangeTracker`** that poses as a second diff
authority, and a **CLI workspace-switch command that does not actually
switch state**. No P0 issues; one P1 (CLI parity gap), three P2, two P3.

### A-PROJ-01-P1-01: CLI `/workspace switch` does not switch state (TUI/GUI parity gap)

- Priority: P1
- Confidence: high
- Layer: application (adapter)
- Evidence:
  - `echo-agent-cli/src/cli/cmd_impls/workspace.rs:114-146` — `ws_switch`
    opens the registry (`registry.open_by_name(name)`) and prints the
    workspace info, then returns. It never obtains an `AppState` handle
    and never calls `AppState::switch_workspace`.
  - `echo-agent-cli/src/tauri/commands/workspace.rs:131-137` — the Tauri
    `switch_workspace` IPC command is the **only** caller of
    `AppState::switch_workspace` in the whole repo (V01 grep,
    `grep -rn "switch_workspace(" --include="*.rs"`).
  - `echo-agent-cli/src/cli/repl.rs:182-204` — the REPL constructs
    `CodingLoop` once from the bootstrap project root and never rebinds
    it; there is no plumbing to re-bind it on a CLI workspace switch.
- Reachability: any user running `/workspace switch <name>` in the CLI/TUI.
  The command reports success ("Switched to workspace: …") but nothing
  about the live session changes: process CWD, agent `working_dir`,
  persistence, conversation store, memory store, runtime state store, and
  the `eko:project-context` projection all keep pointing at the bootstrap
  workspace. Subsequent `/diff`, `/test`, `/review`, and file-tool calls
  operate on the old root.
- Expected invariant (AGENTS.md "TUI 与 GUI 是功能完全一样的 Agent 完全体"):
  `/workspace switch` in the CLI must perform the same state replacement
  the GUI performs via `AppState::switch_workspace`.
- Observed behavior: the CLI command is a read-only registry viewer
  wearing a switch command's clothes. This both violates TUI/GUI parity
  and directly answers the V03 sub-question ("what happens on workspace
  switch?") with "in the CLI, nothing".
- Impact: a CLI/TUI user who switches workspaces is silently left in the
  bootstrap workspace for all subsequent coding/diff/file operations.
  Combined with A-CFG-01-P1-01 (config watcher not refreshed on switch),
  the CLI workspace model is substantially non-functional. Confusing and
  hard to diagnose because the command explicitly reports success.
- Root cause: `ws_switch` was written against the registry-only API
  (`WorkspaceRegistry::open_by_name`) and predates the richer
  `AppState::switch_workspace` orchestration; the CLI command registry
  does not thread an `AppState` (or even a shared `CommandContext`
  workspace handle) into the workspace subcommands.
- Direction: thread a handle to `AppState` (or at minimum a
  `switch_workspace` capability) into the CLI workspace command, and have
  `ws_switch` invoke the same `AppState::switch_workspace` the Tauri
  command uses. If a full `Arc<AppState>` is not available in the CLI
  process shape, extract the switch orchestration into a function that
  both surfaces can call. Also update the CLI `CodingLoop`'s
  `project_root` after the switch so `/diff`, `/test`, `/review` see the
  new root.
- Regression validation: `/workspace switch B` in the CLI, then `/diff`
  shows B's diff, `/test` runs B's test command, `read_file` reads B's
  files, and `eko:project-context` projection reflects B's tree.
- Validation reports: [V03-01](../validations/A-PROJ-01/V03-01.md)

### A-PROJ-01-P2-01: `ProjectIndex` is 488 lines of dead code with a designed-but-unused cache

- Priority: P2
- Confidence: high
- Layer: application (dead code)
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/project/index.rs:1-488` — the
    entire module: `ProjectIndex`, `FileInfo`, `SymbolMatch`, `build`,
    `walk`, `rebuild_maps`, `search_symbols`, `recently_modified`,
    `related_files`, `by_language`, `load`, `save`,
    `extract_symbols_and_imports`, `detect_language`.
  - Whole-repo grep for `ProjectIndex` returns matches ONLY inside
    `project/index.rs` (the struct definition + two `#[cfg(test)]`
    call sites at `:459`, `:482`). No production caller.
  - `echo-agent-cli/echo-agent-app-core/src/lib.rs:29` — `pub mod project;`
    exposes the module, and `project/mod.rs:6` re-exports `pub mod index;`,
    so it compiles into the crate, but nothing reads it.
- Reachability: compile-time only. `ProjectIndex::build` / `load` / `save`
  / `search_symbols` are never invoked from any production path (REPL
  bootstrap, agent build, workspace switch, IPC, tools, prompt assembly).
  The live project-context path goes through `ProjectContext`
  (`context.rs`) and `PromptAssembler` (`prompt.rs`), not `ProjectIndex`.
- Expected invariant (AGENTS.md "动手前先查是不是已经有了" +
  "代码清理:无需兼容,过时代码可直接删"): a 488-line module advertising
  a metadata cache should either be wired into the context pipeline or
  deleted. Living in the tree as-is, it misleads readers (and any future
  agent) into thinking there is a live index that needs invalidation
  handling — which is exactly the framing of this review task.
- Observed behavior: the module exists, compiles, passes its 3 unit tests,
  and is otherwise inert. Its doc (`index.rs:46`) claims "Can be
  serialized to `~/.eko/cache/{project_hash}.json` for persistence" — a
  cache directory that is never created and a `save`/`load` pair with no
  caller.
- Impact: maintenance burden + reviewer/agent confusion. The task's own
  V01 question ("how is the index invalidated?") is a trap under the
  current code: the honest answer is "there is no live index", but the
  code strongly implies otherwise.
- Root cause: `ProjectIndex` was authored as an intended cache layer
  (the doc + `load`/`save` API make the design intent clear) but was
  never wired into the context pipeline, and `ProjectContext` /
  `PromptAssembler` were built (or migrated) to read fresh from disk
  instead. The old module was never retired.
- Direction: delete `project/index.rs` and its `pub mod index;` export in
  `project/mod.rs`. Per AGENTS.md "无需兼容,过时代码可直接删". If a
  future product need requires a persistent project cache, it should be
  re-added against the current `ProjectContext` shape (the on-demand
  fresh-read model already delivers correctness without invalidation
  complexity).
- Regression validation: `cargo check -p echo-agent-app-core` after
  deletion; the 3 tests in `index.rs::tests` are deleted along with the
  module.
- Validation reports: [V01-01](../validations/A-PROJ-01/V01-01.md)

### A-PROJ-01-P2-02: `FileChangeTracker` / `CodingLoop` change tracking is an empty second "what changed" authority

- Priority: P2
- Confidence: high
- Layer: application (duplicate / inert authority)
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/project/file_tracker.rs:28-106`
    — `FileChangeTracker` with `record_change`/`record_write`/`record_delete`
    + `diff_summary`.
  - `echo-agent-cli/echo-agent-app-core/src/project/coding_loop.rs:11-86`
    — `CodingLoop` wraps `FileChangeTracker`; exposes
    `record_file_write`/`record_file_delete`/`diff_summary`.
  - Whole-repo grep for `record_file_write` / `record_file_delete` /
    `record_write` / `record_delete` / `record_change` / `FileChangeTracker`
    outside `src/project/` returns ZERO matches. The framework file tools
    (`EditFileTool` / `WriteFileTool` / `DeleteFileTool`, per F-EXT-02)
    never call back into the tracker; the application adapter (A-TOOL-01
    territory) does not wire them to do so either.
  - `echo-agent-cli/src/cli/cmd_impls/coding.rs:346-361` — `/review`'s
    only use of the tracker is the fallback branch when `git diff HEAD`
    fails (`g.diff_summary()`), which is always empty in practice.
- Reachability: any `/review` run in a non-git project (where `git diff
  HEAD` fails) hits the fallback and prints "No file changes" regardless
  of what was actually edited. In a git project the tracker is simply
  never consulted.
- Expected invariant (AGENTS.md rule 6: "任务关系只有一个权威 API";
  generalizes to "what changed" having one authority): either the tracker
  is the authority and file tools report into it, or it is removed and
  `git diff` / on-disk comparison remain the sole authority. The current
  state — a tracker that exists, is exposed via `CodingLoop`, and is
  silently always empty — is the worst of both: a second "authority"
  that contradicts the single-authority rule while delivering no value.
- Observed behavior: `CodingLoop.diff_summary()` always returns
  `"No file changes"` because nothing populates it. `/review` in a
  non-git project therefore always reports zero changes.
- Impact: (1) latent silent-failure: if a future change starts trusting
  `diff_summary()` for anything nontrivial, it will silently report
  emptiness. (2) Reviewer/agent confusion: the presence of a
  `FileChangeTracker` suggests writes are tracked, when they are not.
  (3) Contradicts AGENTS.md single-authority rule.
- Root cause: `CodingLoop` + `FileChangeTracker` predate the framework
  tool layer and were never wired into the actual file-tool call path.
  When `/review` was rewritten to shell out to `git diff HEAD`, the
  tracker became redundant but was not removed.
- Direction: delete `FileChangeTracker` (file_tracker.rs) and its users in
  `CodingLoop` (`record_file_write`, `record_file_delete`, `diff_summary`,
  `change_count`, `clear_changes`, `status_summary`'s tracker-dependent
  branch). The `/review` fallback should fall back to an explicit
  `"Not a git repository; /review requires git to inspect changes."`
  message rather than an always-empty tracker summary. Alternatively, if
  tracker-backed review for non-git projects is desired, wire the
  framework file tools to report writes (A-TOOL-01 scope) — but that is a
  product decision, not a bug fix, and the simpler path is deletion.
- Regression validation: `/review` in a non-git tempdir reports the new
  explicit message; existing git-backed `/review` behavior unchanged.
- Validation reports: [V02-01](../validations/A-PROJ-01/V02-01.md)

### A-PROJ-01-P2-03: `gitignore::globstar_match` byte-slices `&str` mid-UTF-8 (latent panic)

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/project/gitignore.rs:178-181`
    — `for j in 0..=remaining.len() { let candidate = &remaining[j..]; …
    remaining = &remaining[j + part.len()..]; }`. `remaining.len()` is the
    byte length; `&remaining[j..]` is a byte slice. When `remaining`
    contains a multibyte UTF-8 char (e.g. a Chinese directory name) and
    `j` is not a char boundary, this panics.
  - `echo-agent-cli/echo-agent-app-core/src/project/gitignore.rs:125-156`
    — `simple_glob` is byte-indexed (`path_bytes[pi]`, `pat_bytes[pp]`)
    and will happily match a `?`/`*` against a single byte of a multibyte
    char, producing semantically wrong matches (correctness, not panic).
  - `gitignore.rs:57,61` — `raw[1..]` after `starts_with('!')` /
    `starts_with('/')` is safe (both are single-byte ASCII prefixes, so
    offset 1 is a boundary); included for completeness, not a finding.
- Reachability: confirmed empirically (V03). A pattern with a `**` whose
  first segment does not match at byte offset 0 panics on a Chinese path:
  `globstar_match("z**", "中文模块")` panics at
  `start byte index 1 is not a char boundary; it is inside '中'`. In
  production, reachability is gated by `should_ignore_path`
  (context.rs:55-63), which has **zero callers outside the `project/`
  module** today, so the panic is currently latent. It becomes live the
  moment any caller (e.g. a future file-browser filter, a tree-summary
  pruner, or a workspace `link`) consults `ProjectContext.should_ignore_path`.
- Expected invariant (AGENTS.md UTF-8 rule: "全部使用 `take`,禁止字节截断"):
  path/pattern matching on developer-supplied text must use char iterators
  or boundary-safe indexing, never raw byte offsets.
- Observed behavior: `&remaining[j..]` panics for non-boundary `j`; the
  byte-level `simple_glob` produces wrong matches across multibyte chars.
- Impact: latent panic + correctness bug in a pub API on a
  Chinese-developer-friendly project. No live caller today, but the API
  is exported on `ProjectContext` and is a footgun for the next consumer.
- Root cause: the glob matcher was written with a byte-oriented algorithm
  (classic backtracking glob) and never adapted to UTF-8.
- Direction: rewrite `simple_glob` and `globstar_match` over `Vec<char>`
  (or `chars().collect::<Vec<_>>()`) so all indexing is char-based. This
  also fixes the byte-level mis-match in `simple_glob`. Add a regression
  test with `**` + Chinese paths.
- Regression validation: `globstar_match("z**", "中文模块")` returns
  `false` (no match) without panicking; existing 5 gitignore tests still
  pass; add a Chinese-path `**` test.
- Validation reports: [V03-01](../validations/A-PROJ-01/V03-01.md)

### A-PROJ-01-P3-01: `ProjectIndex` module doc advertises a cache that never exists

- Priority: P3
- Confidence: high
- Layer: application (documentation)
- Evidence: `echo-agent-cli/echo-agent-app-core/src/project/index.rs:43-47`
  — "Built once at startup and refreshed on demand. Can be serialized to
  `~/.eko/cache/{project_hash}.json` for persistence."
- Reachability: documentation only. `save`/`load` have zero callers;
  `~/.eko/cache/` is never created by any audited path.
- Expected invariant: module docs describe behavior that exists or is
  explicitly labelled aspirational.
- Observed behavior: the doc states persistence as a fact; the reality is
  the module is dead (P2-01) and the cache never exists.
- Impact: low; readability/onboarding. Reinforces the "is there a live
  index?" confusion that complicates this very review task.
- Root cause: the doc was written against the original design intent and
  never reconciled with the actual wiring (or lack of it).
- Direction: folded into P2-01's deletion. If `ProjectIndex` is kept for
  some reason, the doc must be rewritten to mark the cache as
  not-yet-implemented.
- Regression validation: doc-only; no test.
- Validation reports: [V01-01](../validations/A-PROJ-01/V01-01.md)

### A-PROJ-01-P3-02: Workspace registry index/manifest writes are non-atomic

- Priority: P3
- Confidence: high
- Layer: application (crash safety)
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/workspace/registry.rs:43-47`
    — `RegistryIndex::save` calls `fs::write(path, json)` (truncate-then-
    write; crash mid-write corrupts `registry.json`).
  - `echo-agent-cli/echo-agent-app-core/src/workspace/registry.rs:32-41`
    — `RegistryIndex::load` falls back to `Self::default()` on any parse
    failure (`serde_json::from_str(&data).ok().unwrap_or_default()`),
    silently producing an empty registry.
  - `echo-agent-cli/echo-agent-app-core/src/workspace/registry.rs:418`
    — workspace manifest write uses the same `fs::write(&manifest_path,
    json)` pattern.
- Reachability: any crash / power loss / disk-full during a workspace
  create / delete / link. The local-personal-assistant threat model
  (AGENTS.md) places this in the "防止框架自身 bug 造成破坏 / 防止用户
  无意中的数据丢失" category — low probability but the impact (all
  workspaces silently invisible on next launch) is user-visible.
- Expected invariant: a metadata file that tracks user workspaces should
  survive a crash mid-write (write-temp-then-rename), or at least the
  load path should distinguish "absent" (normal) from "corrupt"
  (surface loudly).
- Observed behavior: corrupt `registry.json` loads as an empty registry;
  the user's workspace list disappears with no error surfaced.
- Impact: low under normal operation; confusing data-loss-like symptom on
  the rare crash-during-write. Same class of defect as F-EXT-02-P1-02
  (non-atomic file writes in the framework file tools).
- Root cause: convenience-over-correctness (`fs::write` is the one-liner);
  no distinction between absent and corrupt on load.
- Direction: introduce a small `atomic_write(path, bytes)` helper
  (write-to-temp + `fsync` + `rename`) — ideally the same helper
  F-EXT-02-P1-02 recommends for the framework file tools — and migrate
  `RegistryIndex::save` and the manifest write to it. Separately, have
  `RegistryIndex::load` log a warning and (optionally) surface via IPC
  when the file exists but fails to parse.
- Regression validation: a fixture that truncates `registry.json`
  mid-write asserts either the original content survives (after atomic
  fix) or a clear error is logged (after load-path fix).
- Validation reports: [V04-01](../validations/A-PROJ-01/V04-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Index lifecycle and invalidation: `ProjectIndex` is dead (zero callers); live project context (`ProjectContext` + `PromptAssembler`) reads fresh from disk with no cache; no stale-cache risk in production | yes | passed | [V01-01](../validations/A-PROJ-01/V01-01.md) |
| V02 | Diff source-of-truth: `/diff` and `/review` derive output from `git diff` / on-disk files; `FileChangeTracker` is a nominal second authority but is never populated | yes | passed | [V02-01](../validations/A-PROJ-01/V02-01.md) |
| V03 | Workspace switch: GUI path rebuilds `ProjectContext` via `refresh_dynamic_context`; CLI `/workspace switch` does not call `AppState::switch_workspace`; `gitignore::globstar_match` UTF-8 panic reproduced | yes | passed | [V03-01](../validations/A-PROJ-01/V03-01.md) |
| V04 | Large repository / conflicting fixtures: `generate_file_tree_summary` caps depth/entries; `ProjectIndex::walk` caps file size; `/diff` delegates large-repo work to git; registry writes are non-atomic (P3-02) | conditional (large-repo + crash-safety surface inspected) | passed | [V04-01](../validations/A-PROJ-01/V04-01.md) |
| V05 | Historical-document drift | conditional (applicable — `project/index.rs` module doc treated as a hypothesis; classified stale, see P3-01) | passed | classified inline in Historical Claim Status |

Executed cargo commands (all exit 0):

```text
cd echo-agent-cli && cargo test -p echo-agent-app-core --lib project::
  → 11 passed; 0 failed; 0 ignored; 638 filtered out (0.02s)

cd echo-agent-cli && cargo test -p echo-agent-app-core --lib diff::
  → 12 passed; 0 failed; 0 ignored; 637 filtered out (0.01s)

cd echo-agent-cli && cargo test -p echo-agent-app-core --lib workspace::
  → 18 passed; 0 failed; 0 ignored; 631 filtered out (0.03s)
```

The full `echo-agent-cli` pre-commit gate was not re-run because this
review is read-only; the targeted project + diff + workspace subsets are
the directly relevant evidence (41 tests pass) and exercise the
invariants audited here. An additional standalone Rust reproduction of
the `globstar_match` UTF-8 panic is recorded in V03-01.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `project/index.rs:43-47` "Built once at startup and refreshed on demand. Can be serialized to `~/.eko/cache/{project_hash}.json` for persistence." | stale | `ProjectIndex` has zero production callers (V01); the cache dir is never created; `save`/`load` are unused. See A-PROJ-01-P3-01 and the deletion recommendation in P2-01. |
| `project/index.rs:1-2` "metadata cache for fast context assembly" | stale | Production context assembly (`prompt.rs::add_project_context`) does not use `ProjectIndex`; it reads `file_tree_summary` and `git_context` fresh on every call. There is no fast-path cache. |
| `project/file_tracker.rs:5-6` "Accumulates file changes made during a coding session and generates unified diffs for review" | stale/inert | No code path records changes into the tracker (V02); `diff_summary()` always returns "No file changes". |
| `project/coding_loop.rs:1-2` "orchestrates the coding workflow … understand → explore → plan → edit → test → fix cycle" | aspirational | `CodingLoop` is a thin wrapper exposing `test_command` / `lint_command` / `record_file_*` / `diff_summary`; it does not orchestrate any loop. The "edit → test → fix" cycle is driven by the agent + tools, not by this struct. |
| `project/context.rs:7-12` "Instruction/rules content is NOT loaded here — that responsibility belongs solely to `InstructionProvider`" | current | Verified: `load_project_context` builds only `name` / `file_tree_summary` / `gitignore`; no instruction content. `PromptAssembler::add_project_context` (prompt.rs:291-320) injects only structure + git modules. |
| `state.rs:851-864` "切换进程工作目录到工作区根目录。这样所有工具（shell、文件读写、搜索等）都会自动在工作区目录下执行。" | current for GUI, not delivered for CLI | The GUI `switch_workspace` does `set_current_dir` (state.rs:854) and refreshes the agent; the CLI `/workspace switch` (workspace.rs:114) does neither. See A-PROJ-01-P1-01. |

## Coverage And Uncertainty

- **Inspected in full:** the entire `project/{index,context,gitignore,
  file_tracker,coding_loop,detector,test_runner,prompt}.rs` modules
  (collectively ~1760 lines), the entire `diff.rs` (529 lines), the
  `WorkspaceState` definition and the full `switch_workspace` /
  `exit_workspace` / `apply_workspace_routing` bodies (state.rs:409-450,
  844-1050), the CLI `/diff`, `/workspace`, and the relevant slices of
  `/test` and `/review` (coding.rs:280-420), and the Tauri
  `switch_workspace` IPC command.
- **Inspected partially:** `workspace/registry.rs` was read at the header
  and the `RegistryIndex::load`/`save` + manifest-write paths
  (`:32-61, 112, 418`); the full registry CRUD was sampled from the
  18-test `workspace::` suite (all green) but not line-by-line.
- **Not inspected (out of scope):**
  - The web-frontend React rendering of workspace / diff state (only the
    IPC call site at `endpoints.ts:1422` and the `FileDiff` icon import
    were checked to confirm there is no second Rust-side diff authority).
  - The framework file tools' write path (F-EXT-02 territory); this task
    only relies on F-EXT-02's conclusion that those tools do not call
    back into `CodingLoop`.
  - The full `echo-agent-cli` pre-commit matrix (fmt / clippy /
    all-features test). The review is read-only; the targeted project +
    diff + workspace test subsets (41 tests pass) are the directly
    relevant evidence.
- **Uncertain claims:**
  - The exact future intent for `ProjectIndex`. This task classifies it
    as dead and recommends deletion, but it is possible the authors
    intend to wire it in as a real cache. If so, the right response is
    still to delete it now (YAGNI) and re-add against the current
    `ProjectContext` shape when the need materializes.
  - Whether any out-of-tree consumer (an external `echo-agent` user)
    depends on `ProjectIndex`. `ProjectIndex` is `pub` in
    `echo-agent-app-core`, which is an EKO application crate (not the
    reusable `echo-agent` framework), so out-of-tree consumption is
    implausible. AGENTS.md's "echo-agent-cli 不需要兼容" stance applies.

## Handoff

- **Conclusions downstream tasks may rely on:**
  - The live project context (`ProjectContext` via `PromptAssembler`) and
    the diff rendered by `/diff` are **derived from current files with no
    stale cache and no second authority**. Any downstream task that
    assumes a cached index must be invalidated should reconsider: there
    is no live index; `refresh_dynamic_context` rebuilds the projection
    on every workspace switch. (V01, V03)
  - `ProjectIndex` (488 lines) and `FileChangeTracker` are dead/inert and
    are deletion candidates (P2-01, P2-02). Do not build new features on
    top of them.
  - The CLI `/workspace switch` is a no-op for live state (P1-01);
    downstream tasks that assume "the user switched workspaces in the
    CLI" changed the agent's `working_dir` are wrong. Only the GUI path
    switches state.
  - The diff authority is single: `git diff` (via the `/diff` and
    `/review` commands) plus on-disk file comparison via `similar`. The
    `FileChangeTracker` does not contribute. (V02)
- **Reports downstream tasks must read:**
  - [V01-01](../validations/A-PROJ-01/V01-01.md) for the
    duplicate-search evidence (which names are dead) and the live
    context-build path.
  - [V02-01](../validations/A-PROJ-01/V02-01.md) for the diff
    source-of-truth evidence and the empty-tracker confirmation.
  - [V03-01](../validations/A-PROJ-01/V03-01.md) for the workspace-switch
    GUI-vs-CLI gap and the `globstar_match` UTF-8 panic reproduction.
  - [V04-01](../validations/A-PROJ-01/V04-01.md) for the large-repo
    caps and the non-atomic registry-write evidence.
- **Task-to-reference mapping:**
  - A-CFG-01 may rely on this task's confirmation that the
    `eko:project-context` projection is rebuilt on workspace switch
    (state.rs:883 → infra.rs:530 → prompt.rs:352). A-CFG-01's "config
    watcher targets not refreshed on switch" (A-CFG-01-P1-01) is a
    separate gap and remains valid.
  - A-TOOL-01 should consume P2-02's conclusion when deciding whether
    to wire file-tool write callbacks into a change tracker: the current
    tracker is empty and should either be deleted or wired deliberately
    (not left half-alive).
  - X-INV-01 / B-REF-01 (cross-repo dead-code / duplicate inventory)
    should fold `ProjectIndex` and `FileChangeTracker` into their lists.
- **Conditions that make this report stale:**
  - Any commit that wires `ProjectIndex` into a live caller (consumes
    P2-01 / P3-01).
  - Any commit that adds a caller of `CodingLoop.record_file_write` /
    `record_delete` (consumes P2-02).
  - Any commit that changes `ws_switch` (CLI) to call
    `AppState::switch_workspace` (consumes P1-01).
  - Any commit that rewrites `gitignore::globstar_match` /
    `simple_glob` over char iterators (consumes P2-03).
  - Introduction of an `atomic_write` helper and migration of
    `RegistryIndex::save` / manifest write to it (consumes P3-02).
- **Follow-up task IDs (no fixes implemented in this review):**
  - A CLI parity task should pick up A-PROJ-01-P1-01 (CLI workspace
    switch is a no-op). This is a behavioral fix and may require threading
    an `AppState` handle into the CLI command registry.
  - A "project module cleanup" task should batch P2-01 (delete
    `ProjectIndex`), P2-02 (delete `FileChangeTracker` + `CodingLoop`
    tracker methods), and P3-01 (stale doc, folded into P2-01's
    deletion).
  - A UTF-8-safety task should pick up P2-03 (rewrite the gitignore glob
    matcher over char iterators) — and while in the neighbourhood,
    re-audit the byte-indexed `simple_glob` for the same class of bug.
  - A crash-safety task should pick up P3-02 (atomic registry writes),
 preferably by sharing the `atomic_write` helper recommended by
    F-EXT-02-P1-02 for the framework file tools.
