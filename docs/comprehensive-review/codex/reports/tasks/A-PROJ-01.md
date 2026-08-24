# A-PROJ-01: Project indexing, diff, and coding workspace services

> Status: complete
> Reviewer: Codex review subagent
> Executor: Codex review subagent
> Accepted by: Codex primary reviewer
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: CLI was clean during source inspection, then externally dirty
> `Cargo.lock` appeared during closeout and was excluded; framework worktree was
> externally dirty and excluded except committed HEAD metadata

## Question

Are project indexing, diff, coding commands, and workspace state derived from
current files without stale caches or a second worktree/file authority?

## Scope

- App-core `project/{index,context,gitignore,coding_loop,file_tracker}.rs`,
  `diff.rs`, workspace registry and project prompt projection.
- CLI REPL registration plus workspace, coding and diff command paths.
- GUI Tauri workspace/file commands, IPC registration, frontend endpoint,
  WorkspaceStore/FileStore, and the live FileBrowser/DiffViewer.
- Definition/export/registration/real reachability, index invalidation, file and
  workspace identity, switch/refresh/conflict behavior, diff provenance,
  UTF-8/panic/overflow, large-input bounds/cancellation, and test inventory.

## Out Of Scope

- Source changes and Cargo/rustc/test/build/dynamic fixture/network execution.
- Generic framework file/git/worktree tool safety, mutation, artifacts and
  cancellation owned by F-EXT-02. No current dirty framework source was read.
- TaskRuntime worktree claim/integration/cleanup and retained data-workspace
  ownership owned by A-TSK-05.
- GUI `AppState::switch_workspace` atomic store/cwd rollback owned by A-CFG-01;
  this report covers absent/fake surface transitions and GUI file-cache identity.
- Broad frontend architecture/accessibility, task artifacts, conversation-store
  behavior, and configuration discovery except where required to trace one
  project-root transition.

## Inputs

- Root `AGENTS.md`; shared `README.md`, `REPORTING.md`, `TASKS.md`; Codex README
  and report templates.
- Exact Codex dependencies A-CFG-01, A-TSK-05, and F-EXT-02. No other reviewer
  directory or non-dependency task report was read.
- CLI source at the pinned clean commit. Framework HEAD hash/status was recorded;
  current externally dirty framework bodies and diffs were excluded.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Reusable diff primitives, safe path/file primitives and generic Git/worktree tools may live in the framework. Existing framework APIs remain independent of EKO usage; F-EXT-02 owns their review. |
| EKO product policy | Workspace registry/current selection, GUI tabs/drafts, CLI/TUI commands, project prompt projection and coding-workspace UX are EKO application responsibilities. |
| Adapter boundary | Tauri/CLI/TUI adapters should carry stable workspace plus relative-file identity, call one application workspace transition, and losslessly adapt one diff authority; they must not infer current state from recency or own a second diff algorithm. |
| Duplicate search | Searched both repositories for project/index/context/ignore, file trackers, diff result/hunk/generation, workspace registry/switch/link, coding loops, file API/store, registrations, constructors, mutation callers, tests and equivalent Git/file behaviors. Framework search was committed-HEAD only. |
| Migration deletion | Keep app-core's context-bounded tested diff authority; delete the Tauri diff algorithm after a lossless adapter exists. Delete inert ProjectIndex/tracker/`.bak` claims unless a real bounded lifecycle is deliberately wired. Do not add another workspace store, index, diff engine, or worktree owner. |

## Current Path

```text
GUI workspace switch
  -> workspaceStore.switchTo(id)
  -> Tauri switch_workspace -> WorkspaceRegistry.open -> AppState switch
  -> frontend resets chat/conversations only
  -> module-global FileStore remains keyed by relative path
  -> saveSelected(path, draft, old revision)
  -> backend resolves path against current workspace root

GUI files/diff
  Tauri registered commands -> filesApi -> FileStore -> FileBrowser
  workspace_changes -> ad hoc porcelain text parser -> relative path
  diff_file -> git show or empty base -> second TextDiff algorithm -> IPC/UI

CLI
  REPL builds ProjectContext and CodingLoop once from startup root
  /workspace switch -> registry open/touch only -> prints success
  /test and /code-review -> frozen CodingLoop root
  /code-review fallback -> never-populated FileChangeTracker
  /diff <file> -> nonexistent .bak producer

Indexing
  ProjectIndex build/load/save -> tests only, no runtime owner/invalidation
  ProjectContext -> live prompt projection
  GitIgnore field -> loaded, but should_ignore_path has no production caller
```

Positive boundaries to retain: GUI write uses SHA-256 expected revision and a
same-directory temporary rename; current external edits are not blindly
overwritten within one workspace. App-core diff has context-bounded grouping and
tests equal/add/remove/replace. AppState is the GUI runtime workspace authority.

## Findings

### A-PROJ-01-P0-01: GUI drafts can overwrite the same relative file in a different workspace

- Priority: P0; confidence: high; layer: application.
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/web-frontend/src/stores/workspaceStore.ts:49`,
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/web-frontend/src/stores/fileStore.ts:20`,
  `:179`, `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/files.rs:154`,
  `:164`, `:451`.
- Reachability: live GUI workspace switch leaves the live module-global
  FileStore intact; FileBrowser's save shortcut invokes `saveSelected`, whose
  backend binds the retained relative path to the newly current workspace.
- Expected invariant: draft identity is `(workspace/project root, relative path,
  base revision)` and cannot be rebound implicitly by a workspace transition.
- Observed behavior: WorkspaceStore clears chat and conversations but never
  clears/rekeys FileStore. Documents, tabs, selected path and draft are keyed
  only by relative path. If workspaces A and B contain the same relative file
  with identical base bytes, their content-hash revisions match; after editing A
  and switching to B, save passes B's revision check and writes A's draft to B.
- Impact: an ordinary workspace switch followed by save can corrupt the user's
  file in a different project; in-flight reads/diffs can also populate stale
  workspace data.
- Root cause: frontend file identity omits workspace/root and the switch
  transition has no dirty-document or request-generation boundary.
- Direction: make workspace identity part of every document/request/artifact
  key; atomically block, retain with explicit origin, or clear dirty documents
  during switch; generation-fence/cancel prior requests. Keep backend revision
  and atomic-write protections.
- Regression validation: two workspaces with same relative path and equal/different
  base revisions, dirty/clean tabs, in-flight read/diff/tree/status, switch/exit/
  delete, and save after switch.
- Validation reports: [V04](../validations/A-PROJ-01/V04-01.md),
  [V10](../validations/A-PROJ-01/V10-01.md)

### A-PROJ-01-P1-02: CLI reports a workspace switch without switching runtime state, while TUI has no switch path

- Priority: P1; confidence: high; layer: application.
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/cli/cmd_impls/workspace.rs:98`,
  `:113`, `:165`, `:281`, `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/workspace/registry.rs:185`,
  `:217`, `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/cli/repl.rs:118`,
  `:182`, `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tui/commands.rs:53`.
- Reachability: `/workspace` is registered in the production REPL. Its switch
  calls only registry `open_by_name` then prints success; `/test`, review,
  prompts and Agent state continue using values constructed at REPL startup.
- Expected invariant: GUI, TUI and CLI expose equivalent one-owner workspace
  transitions and report success only after live project services are rebound.
- Observed behavior: CLI open only updates `last_active`; info/link then call the
  most-recent entry current, list hardcodes ID `default` as active, and no live
  service changes. TUI's SlashCommand enum has no workspace operation and source
  comments state it has no workspace concept. GUI alone calls AppState.
- Impact: CLI users believe subsequent tests/reviews/Agent actions target the
  selected workspace when they still target the startup project; TUI users
  cannot perform the equivalent operation.
- Root cause: registry recency is treated as runtime current state and surface
  adapters do not share one application transition service.
- Direction: expose one application workspace-transition service used by all
  surfaces and pass the runtime current workspace explicitly to list/info/link.
  Delete recency-as-current and hardcoded-default labels. Coordinate the GUI
  atomicity repair with A-CFG-01-P1-02 rather than creating another switch path.
- Regression validation: switch each surface, then assert prompt, file, coding,
  task, memory/conversation and Agent roots all agree; inject partial failures
  and require truthful terminal output.
- Validation reports: [V05](../validations/A-PROJ-01/V05-01.md)

### A-PROJ-01-P1-03: Git change parsing corrupts quoted paths and treats command failure as a clean tree

- Priority: P1; confidence: high; layer: adapter.
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/files.rs:211`,
  `:233`, `:675`, `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/web-frontend/src/stores/fileStore.ts:68`,
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/web-frontend/src/components/file-browser/FileBrowser.tsx:72`.
- Reachability: FileBrowser polls the registered workspace-change command and
  passes every returned path to live diff loading.
- Expected invariant: Git path bytes are decoded losslessly, including rename/
  copy and Unicode/special names; failure is a typed failure, not no changes.
- Observed behavior: newline porcelain is lossy-decoded, split on textual ` -> `
  and only stripped of quotes. Default quoted/octal Git paths therefore become
  nonexistent display/diff paths. Any nonzero Git status returns an empty list.
  The Unicode test bypasses the wire behavior by supplying decoded Unicode.
- Impact: changed Chinese/emoji/special-name files can be unopenable from the
  changes panel, while Git/repository failures misleadingly appear clean.
- Root cause: an ad hoc string parser replaces Git's unambiguous NUL-delimited
  pathname protocol and erases command status.
- Direction: parse `--porcelain=v1 -z` bytes (or a structured library), retain
  rename source/destination identity, and propagate typed exit/stderr errors.
- Regression validation: spaces, quotes, tabs, newlines, Chinese, emoji,
  rename/copy pairs, deletions, non-repository, missing Git and nonzero status.
- Validation reports: [V06](../validations/A-PROJ-01/V06-01.md),
  [V10](../validations/A-PROJ-01/V10-01.md)

### A-PROJ-01-P1-04: The live GUI uses a second diff engine that misreports equal content and Git errors

- Priority: P1; confidence: high; layer: adapter.
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/diff.rs:65`,
  `:438`, `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/files.rs:41`,
  `:267`, `:292`, `:310`, `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/web-frontend/src/components/file-browser/DiffViewer.tsx:8`.
- Reachability: Tauri registers `diff_file`; filesApi/FileStore/FileBrowser use
  it directly. App-core's tested generator is used by CLI but not GUI.
- Expected invariant: one diff authority returns no hunks for equal content,
  correct context/counts for changes, and distinguishes missing-at-ref/untracked
  from Git failure.
- Observed behavior: GUI duplicates the types/algorithm and appends every equal
  line into one hunk, so an unchanged file renders a full-file `0/0` diff.
  Context is excluded from hunk counts. Any git-show spawn/nonzero/join failure
  becomes empty old content and can render a false all-insert diff.
- Impact: GUI diff can falsely show changes, emit invalid hunk headers, conceal
  repository/ref failures, and disagree with CLI for the same file.
- Root cause: the Tauri adapter owns semantic diff generation and erases base
  lookup provenance instead of adapting app-core's canonical diff.
- Direction: adapt app-core's diff model losslessly and delete Tauri's
  DiffResult/Hunk/Line algorithm; use a typed base lookup that distinguishes
  untracked/missing/deleted from execution failure.
- Regression validation: equal, insert/delete/replace, distant hunks, context
  counts, binary/deleted/untracked, missing ref/path, invalid repo and Unicode;
  assert CLI/GUI/TUI result equivalence.
- Validation reports: [V01](../validations/A-PROJ-01/V01-01.md),
  [V07](../validations/A-PROJ-01/V07-01.md),
  [V10](../validations/A-PROJ-01/V10-01.md)

### A-PROJ-01-P2-05: ProjectIndex advertises a lifecycle that is neither reachable nor implemented

- Priority: P2; confidence: high; layer: application.
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/project/index.rs:43`,
  `:63`, `:80`, `:131`, `:429`,
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/project/context.rs:32`.
- Reachability: repository-wide references to ProjectIndex outside its
  definition are its two unit tests; no startup, prompt, Agent, CLI, GUI or
  workspace transition owns it.
- Expected invariant: a public persisted index advertised as startup/on-demand
  has a live owner, root/revision identity, invalidation and bounded traversal,
  or is removed as obsolete.
- Observed behavior: load trusts serialized root/files/time, no update/remove/
  refresh exists, walk has no symlink visited set or file/depth/time/cancel bound,
  silently drops errors and ignores the separate live GitIgnore policy. Save is
  a non-atomic overwrite. None affects current runtime because no caller exists.
- Impact: the public app-core API is a misleading maintenance target and, if
  wired as documented, immediately permits stale results and unbounded/cyclic
  traversal rather than providing the promised current index.
- Root cause: an aspirational implementation was exported and tested locally
  without a lifecycle owner or integration test.
- Direction: delete ProjectIndex if current ProjectContext/on-demand tools are
  the intended product; otherwise first define one application lifecycle and
  implement identity/invalidation/bounds using one ignore policy. Do not add a
  second worktree/file authority.
- Regression validation: production reachability, changed/deleted/renamed files,
  corrupt/wrong-root cache, ignores, symlink cycle/outside root, I/O errors,
  large repository, cancellation and atomic save.
- Validation reports: [V02](../validations/A-PROJ-01/V02-01.md),
  [V03](../validations/A-PROJ-01/V03-01.md),
  [V10](../validations/A-PROJ-01/V10-01.md)

### A-PROJ-01-P2-06: Coding review fallback and single-file diff depend on producers that do not exist

- Priority: P2; confidence: high; layer: application.
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/project/coding_loop.rs:46`,
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/project/file_tracker.rs:39`,
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/cli/cmd_impls/coding.rs:337`,
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/cli/cmd_impls/diff_cmd.rs:133`.
- Reachability: `/code-review` uses the tracker only when Git diff fails;
  `/diff <file>` always attempts `<file>.bak`. Both commands are registered.
- Expected invariant: every fallback or advertised snapshot has a live producer
  and reflects complete current edits.
- Observed behavior: no code calls record-write/delete, so fallback always says
  no file changes. No repository code creates `.bak`, despite help text claiming
  Agent edits do; the single-file comparison is normally unavailable.
- Impact: precisely when Git diff fails, review produces a false empty summary;
  users are directed to a single-file diff workflow the application never
  creates data for.
- Root cause: old parallel change/snapshot mechanisms survived without write-
  path integration while Git/app-core diff became the live authority.
- Direction: use one current Git/file authority and delete FileChangeTracker,
  CodingLoop mutation/fallback, `.bak` branch/help/tests unless a durable,
  identity-bound snapshot is deliberately introduced.
- Regression validation: Git failure, untracked files, non-Git projects,
  successful two-file/Git diff, and any retained snapshot's producer/locator.
- Validation reports: [V02](../validations/A-PROJ-01/V02-01.md),
  [V09](../validations/A-PROJ-01/V09-01.md)

### A-PROJ-01-P2-07: GUI project reads can materialize unbounded work and accept stale async results

- Priority: P2; confidence: high; layer: application.
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/files.rs:211`,
  `:267`, `:383`, `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/web-frontend/src/stores/fileStore.ts:58`,
  `:116`, `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/web-frontend/src/components/file-browser/FileBrowser.tsx:33`.
- Reachability: the mounted FileBrowser loads tree/changes, polls every 2.5s and
  invokes live full diff requests.
- Expected invariant: repository-scale operations have bounds/pagination,
  cancellation/timeout and request/workspace generations; complete oversized
  data is retained by artifact locator rather than discarded.
- Observed behavior: diff has no input/output cap and returns old, new and every
  line in one IPC object; status materializes every untracked path; tree has only
  a depth bound. Blocking tasks have no cancellation/timeout. Frontend requests
  have no abort/generation check, so late results overwrite newer state.
- Impact: large repositories/files can consume disproportionate memory and UI
  time, while rapid file/workspace changes display stale tree/change/diff facts.
- Root cause: preview endpoints and frontend stores have no shared bounded
  operation contract or stable request identity.
- Direction: add bounded paging/streaming and cancellation/generation tokens;
  return complete oversized results by durable artifact locator and delete
  redundant old/new IPC bodies when unused.
- Regression validation: large files/repositories, burst polling, rapid file and
  workspace switches, cancellation/shutdown, stale completion and artifact full
  content/lineage.
- Validation reports: [V08](../validations/A-PROJ-01/V08-01.md),
  [V10](../validations/A-PROJ-01/V10-01.md)

### A-PROJ-01-P2-08: The public project ignore matcher panics on multibyte globstar paths

- Priority: P2; confidence: high; layer: application.
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/project/gitignore.rs:158`,
  `:178`, `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/project/context.rs:49`.
- Reachability: ProjectContext is live, and `should_ignore_path` exposes the
  matcher as app-core API; no current internal production caller invokes it, so
  this is not classified as a core-path P1.
- Expected invariant: arbitrary Chinese/emoji paths never reach byte-index str
  slicing or panic.
- Observed behavior: globstar_match iterates all byte offsets and evaluates
  `&remaining[j..]`, then slices at `j + part.len()` without boundary checks.
- Impact: an app-core consumer checking a multibyte path against a middle `**`
  pattern can panic the process; current internal paths do not trigger it.
- Root cause: an ad hoc glob matcher mixes byte scanning with unchecked Rust str
  slicing instead of using a proven ignore/path library.
- Direction: use a mature Git-ignore matcher or char-safe boundaries and delete
  the custom byte-slicing matcher when replaced.
- Regression validation: Chinese/emoji before/inside/after wildcard, negation,
  root/directory patterns and malformed patterns under no-panic assertions.
- Validation reports: [V12](../validations/A-PROJ-01/V12-01.md),
  [V10](../validations/A-PROJ-01/V10-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search | yes | passed | [V01-01](../validations/A-PROJ-01/V01-01.md) |
| V02 | Registration and runtime reachability | yes | failed | [V02-01](../validations/A-PROJ-01/V02-01.md) |
| V03 | ProjectIndex lifecycle/invalidation/bounds | yes | failed | [V03-01](../validations/A-PROJ-01/V03-01.md) |
| V04 | Workspace-keyed file/draft identity trace | yes | failed | [V04-01](../validations/A-PROJ-01/V04-01.md) |
| V05 | CLI/TUI/GUI workspace switch trace | yes | failed | [V05-01](../validations/A-PROJ-01/V05-01.md) |
| V06 | Git change path/error protocol | yes | failed | [V06-01](../validations/A-PROJ-01/V06-01.md) |
| V07 | Canonical versus GUI diff authority | yes | failed | [V07-01](../validations/A-PROJ-01/V07-01.md) |
| V08 | Large-input bounds/cancellation/stale results | yes | failed | [V08-01](../validations/A-PROJ-01/V08-01.md) |
| V09 | Coding tracker and backup producer trace | yes | failed | [V09-01](../validations/A-PROJ-01/V09-01.md) |
| V10 | Existing test inventory | yes | failed | [V10-01](../validations/A-PROJ-01/V10-01.md) |
| V11 | Executable conflict/large/Unicode fixtures | future | not_run | [V11-01](../validations/A-PROJ-01/V11-01.md) |
| V12 | UTF-8/panic inspection of ignore matching | yes | failed | [V12-01](../validations/A-PROJ-01/V12-01.md) |
| V13 | Evidence-chain and isolation integrity gate | yes | passed | [V13-02](../validations/A-PROJ-01/V13-02.md) |
| V30 | Primary source-anchor sampling and acceptance | yes | passed | [V30-01](../validations/A-PROJ-01/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| A-CFG-01-P1-02: GUI switch can split live roots on failure | current, dependency-owned | The same AppState transition remains; this task does not repeat its atomicity finding and isolates frontend file-cache identity in V04. |
| A-TSK-05 worktree/integration ownership findings | current, dependency-owned | No TaskRuntime worktree implementation is reclassified here; A-PROJ findings cover ordinary project files, GUI/CLI workspace adapters and diff. |
| F-EXT-02 generic file/git/worktree safety findings | current, dependency-owned | Framework contracts remain independent; no finding here treats CLI non-use as grounds to delete framework APIs. |
| ProjectIndex doc: built at startup/refreshed on demand | stale | V02/V03 find only test constructors and no refresh API or owner. |
| CLI `.bak` help: backups are created when Agent edits | stale | V09 finds no producer in either repository. |

## Coverage And Uncertainty

- All conclusions are static. V11 records the required future executable
  conflict, Unicode Git, index-cycle and large-repository/file fixtures.
- No Cargo/rustc/test/build/network/fixture process was started. Existing tests
  were inventoried but not executed.
- Current framework source is heavily externally dirty; this task did not read
  its bodies or diffs. CLI `Cargo.lock` also became externally dirty during
  closeout and was not read, modified or reverted. Only reviewed CLI source,
  accepted dependency reports and framework HEAD metadata were used.
- Link-project UI reachability and broad web fallback server endpoints were not
  needed to establish the live GUI/Tauri paths and are not claimed complete.
- P0-01 is derived from explicit state keys and SHA-256 equality. Dynamic timing
  is unnecessary for the equal-base sequential switch/save path, but fix-stage
  reproduction remains mandatory.

## Handoff

- Treat `(workspace/project identity, relative path, revision)` as the minimum
  file identity for downstream frontend/workspace work; relative path plus hash
  is insufficient across roots.
- Preserve one application workspace transition owner and app-core's tested diff
  authority. Delete replaced Tauri/recency/tracker/backup mechanisms rather than
  layering new adapters beside them.
- Read V04, V05, V06 and V07 before fixing workspace/file/diff flows; read V03
  before deciding whether ProjectIndex should be deleted or deliberately wired.
- This report becomes stale if WorkspaceStore/FileStore identity, Tauri files
  commands, app-core diff/project services, CLI workspace/coding commands, or
  WorkspaceRegistry behavior changes.
- The primary reviewer independently sampled the cross-workspace corruption,
  workspace-switch, Git-path, duplicate-diff, inert-index/tracker and UTF-8
  anchors. V30 records acceptance; this task is complete.
