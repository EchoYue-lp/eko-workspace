# F-EXT-02: Shell, file, code, and Git tools

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: not-applicable
> Worktree state: clean

## Question

Are common local developer tools correct for paths, UTF-8, atomic writes,
diff application, cancellation, process cleanup, and isolation?

## Scope

Primary source paths and behaviors inspected (all under `echo-agent/`):

- `echo-tools/src/shell.rs` — `ShellTool`, streaming execution,
  `IncrementalUtf8Decoder`, `cleanup_direct_child`, command-safety gate.
- `echo-tools/src/code.rs` — `RunCodeTool`, `validate_script_path`.
- `echo-tools/src/files/mod.rs` — `resolve_path` confinement helper,
  `normalize_path`.
- `echo-tools/src/files/edit.rs` — `EditFileTool` (search/replace + diff).
- `echo-tools/src/files/diff.rs` — `DiffTool`.
- `echo-tools/src/files/files.rs` — `CreateFileTool`, `DeleteFileTool`,
  `ReadFileTool`, `WriteFileTool`, `AppendFileTool`, `UpdateFileTool`,
  `MoveFileTool`, `ListDirTool`.
- `echo-tools/src/git.rs` — six git CLI tools and `effective_repo_path`.
- `echo-tools/src/git_worktree.rs` — `create_worktree`,
  `remove_worktree`, `list_worktrees`, `merge_worktree`.
- `echo-tools/src/git_checkpoint.rs` — checkpoint tag create/rollback/cleanup.
- `echo-tools/src/worktree_tool.rs` — `EnterWorktreeTool`,
  `ExitWorktreeTool`, `ListWorktreesTool`.

## Out Of Scope

- The `SandboxExecutor` trait and its concrete implementations — owned by
  F-SEC-01. This task only verifies that `ShellTool` and `RunCodeTool`
  delegate correctly to the sandbox when one is configured.
- The generic `Tool` / `ToolResult` contract — owned by F-EXT-01; this
  task assumes F-EXT-01's conclusion that `ToolFailure::side_effect =
  Possible` + `VerifyThenRetry` is the safety-conservative recovery path.
- Other `echo-tools` domain families (data/research/media/web/database/rag)
  — owned by F-EXT-03.
- MCP-integrated external tools — owned by F-INT-01.
- EKO application adapters that wire these tools into the agent — owned by
  A-TOOL-01.

## Inputs

- Required documents read:
  - `AGENTS.md` (root) — UTF-8 rule (`chars().take()`, no byte slicing),
    panic-safety rule (no `unwrap`/`expect`/`panic!` in production),
    layering gate (framework vs. application), worktree-isolation
    expectation (B-REF-01 parallel-writer pattern), no-overreach security
    boundary (local personal assistant).
  - `docs/comprehensive-review/REPORTING.md`,
    `docs/comprehensive-review/templates/task-report.md`,
    `docs/comprehensive-review/templates/validation-report.md`.
- Dependency task reports read:
  - [F-EXT-01](F-EXT-01.md) (this reviewer) — relied on its conclusion
    that the `Tool` / `ToolResult` / `ToolFailure` contract is the single
    typed surface and that `PartialSideEffect` / `Timeout` route to
    `VerifyThenRetry`. This task extends that to the concrete builtin
    implementations.
- Historical documents treated as hypotheses:
  - `echo-agent/docs/{zh,en}/34-git-isolation.md` and
    `30-react-safety.md` describe the intended checkpoint/worktree
    workflow. Their claims about `rollback_to_checkpoint` are referenced
    as historical context (see Historical Claim Status).

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | Yes for shell/file/git operations: any `echo-agent` consumer needs path-confined file IO, a Unicode-safe shell, a typed run_code primitive, and git CLI wrappers. These are correctly placed in `echo-tools` (a framework crate) at the right layer of abstraction (concrete `Tool` impls over `echo-core`'s `Tool` trait). |
| EKO product policy | The `git_checkpoint` integration in `EditFileTool` / `WriteFileTool` / `DeleteFileTool` injects `echo-checkpoint/*` tags into any git repository the tool touches. For a generic framework consumer that is not EKO, this is product policy leaking into framework code: there is no opt-out, and the tag name is hardcoded. Borderline — acceptable as a "safe-by-default framework opinion" but should be opt-out or feature-gated. The `WorktreeConfig.path_suffix` parameter and `enter_worktree`/`exit_worktree` tool surface are correctly generic; only their implementations are unsafe (see findings). |
| Adapter boundary | The framework `Tool` contract is honored: each builtin returns `ToolResult` with correct `kind`, `failure.category`, `failure.side_effect`, and `postcondition`. `WriteFileTool` emits `PartialSideEffect + idempotency_key + postcondition` exactly as F-EXT-01 specified. The tool → sandbox seam (`set_sandbox`, `execute_with_limits_and_cancel`) is thin and lossless. |
| Duplicate search | Searched names: `EditFileTool` vs. `UpdateFileTool` (file edit), `create_checkpoint` / `rollback_to_checkpoint` (git checkpoint), `create_worktree` / `WorktreeConfig` (worktree). Result: (a) `EditFileTool` and `UpdateFileTool` are duplicate authorities for file content replacement — see F-EXT-02-P2-01; (b) `rollback_to_checkpoint` has zero non-test, non-doc callers (dead public API per AGENTS.md "dead code" rule — see Coverage And Uncertainty); (c) `WorktreeConfig` is the single worktree authority, no duplicate. |
| Migration deletion | If F-EXT-02-P2-01 is accepted, `UpdateFileTool` and its registration in `registry.rs` should be deleted (EditFileTool strictly dominates). If `rollback_to_checkpoint` is confirmed unused, it should be deleted from `git_checkpoint.rs`. |

## Current Path

Verified shell/file/code/git data flow at commit `9b0e0fa`:

1. **Shell execution path.** `ShellTool::execute_stream_with_context`
   (shell.rs:432) parses `command`, runs `check_command_safety`
   (shell.rs:207), and either (a) delegates to `SandboxExecutor` via
   `start_sandbox_stream` (shell.rs:607) when configured, or (b) spawns a
   direct child via `start_direct_stream` (shell.rs:564). The direct child
   is configured with `kill_on_drop(true)` + `process_group(0)` (unix) at
   shell.rs:576-578. `run_direct_child` (shell.rs:674) selects over
   `tx.closed()` (consumer drop), `deadline` (timeout), stdout read,
   stderr read, and `child.wait()`. Cancellation and timeout both route
   to `cleanup_direct_child` (shell.rs:845), which sends
   `kill -KILL -<pgid>` to the child's whole process group, then reaps.

2. **Streaming UTF-8 path.** `IncrementalUtf8Decoder` (shell.rs:968)
   buffers trailing incomplete bytes from each pipe read and only emits
   valid UTF-8 prefixes; `split_stream_chunks` (shell.rs:825) chunks by
   `chars()` + `len_utf8()`, never byte-slicing a multibyte sequence. The
   existing test `shell_stream_preserves_unicode_split_across_pipe_reads`
   (shell.rs:1511) reassembles a single `€` split across three reads.

3. **Path confinement path.** Every file-tool entry point routes through
   `resolve_path` (files/mod.rs:35), which normalizes `.`/`..` textually,
   checks `starts_with(base)`, then `std::fs::canonicalize`s to defend
   against symlinks. For not-yet-existing write targets, the *parent*
   directory is canonicalized and checked (files/mod.rs:96-113).
   `validate_script_path` (code.rs:89) implements equivalent defense for
   `run_code`'s `script_path` parameter, including a Unix-only
   symlink-outside-working-dir rejection test (code.rs:783).

4. **Edit application path.** `EditFileTool` (edit.rs:91) reads the file,
   validates `old_content` is present, rejects if `count > 1 &&
   !replace_all` (edit.rs:169-178), creates a `.bak` copy (edit.rs:222)
   and a git checkpoint (edit.rs:216), then `tokio::fs::write`s in place
   (edit.rs:231) and returns a unified diff. `UpdateFileTool`
   (files.rs:909) does the same read-modify-write but with no `.bak`, no
   checkpoint, no diff, no multi-occurrence gate, and no `expected_hash`.

5. **Write path.** `WriteFileTool` (files.rs:617) optionally verifies
   `expected_hash` (files.rs:641-670), creates parent dirs and a git
   checkpoint, then `tokio::fs::write`s in place. Returns
   `PartialSideEffect + idempotency_key + postcondition` on failure
   (files.rs:678-685, 700-707), exactly matching the F-EXT-01 contract
   for partial side-effect reporting.

6. **Worktree path.** `EnterWorktreeTool::execute` (worktree_tool.rs:78)
   reads `branch`, `base`, `path_suffix`, `repo_path` from LLM JSON and
   calls `create_worktree` (git_worktree.rs:37). `create_worktree`
   computes `git_root.join(".worktrees").join(suffix)` (git_worktree.rs:44-51)
   and runs `git worktree add -b <branch> <path>`. `remove_worktree`
   (git_worktree.rs:115) runs `git worktree remove --force` and
   `git branch -D`.

## Findings

### F-EXT-02-P1-01: Worktree `path_suffix` traversal writes outside `.worktrees/`

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-tools/src/git_worktree.rs:44-51` — worktree path computation.
  - `echo-tools/src/worktree_tool.rs:88-91, 97-101` — `path_suffix`
    forwarded from LLM JSON to `WorktreeConfig` without sanitization.
  - `echo-tools/src/worktree_tool.rs:53-76` — JSON Schema declares
    `path_suffix` as a free-form string with no pattern constraint.
- Reachability: `EnterWorktreeTool` is a registered `Tool` exposed to the
  LLM (the schema declares `branch`, `base`, `path_suffix`, `repo_path`).
  Any agent invocation can supply `path_suffix = "../evil-target/wt"`.
  V04 empirically confirmed `git worktree add` accepts the literal `..`
  path and creates the worktree at the escaped location.
- Expected invariant: a `path_suffix` is a directory *name* under
  `.worktrees/`; traversal attempts are rejected or normalized to stay
  inside the worktrees root.
- Observed behavior: `Path::join(".worktrees").join("../evil-target/wt")`
  produces the literal path `.worktrees/../evil-target/wt`, and git
  creates the worktree at `evil-target/wt` outside `.worktrees/`. Combined
  with `remove_worktree`'s unconditional `--force` and `git branch -D`
  (F-EXT-02-P2-03), the escaped worktree can later be force-removed,
  discarding user files.
- Impact: data loss / integrity. A confused or malicious prompt can write
  agent working files into arbitrary locations outside the intended
  `.worktrees/` root, then have them force-deleted on cleanup. This is
  exactly the "防止用户无意中的数据丢失" category AGENTS.md explicitly
  endorses as a legitimate local safety concern.
- Root cause: `path_suffix` is interpolated into the worktree path
  without normalization or containment check. The branch field *is*
  sanitized (git_worktree.rs:47-50) for path purposes; `path_suffix` is
  not, despite being equally attacker-controlled.
- Direction: apply the same confinement pattern `resolve_path` uses
  (files/mod.rs:35): normalize the suffix, reject if it contains
  `ParentDir` / `RootDir` / absolute components, then `canonicalize` the
  resulting `.worktrees/<suffix>` path and re-check it `starts_with` the
  `.worktrees` root. Add a JSON Schema `pattern` constraint as
  defense-in-depth. Or simpler: only allow `path_suffix` to be a single
  path component (no `/`, no `..`).
- Regression validation: a fixture that calls `create_worktree` with
  `path_suffix = "../escape"` and asserts an `Err`; a fixture that calls
  `EnterWorktreeTool` with a flag-shaped `path_suffix` and asserts
  rejection. See V04 follow-up.
- Validation reports: [V04](../validations/F-EXT-02/V04-01.md)

### F-EXT-02-P1-02: No file tool performs atomic (crash-safe) writes

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-tools/src/files/files.rs:699` — `WriteFileTool`:
    `tokio::fs::write(&path, content).await`.
  - `echo-tools/src/files/files.rs:963` — `UpdateFileTool`:
    `tokio::fs::write(&path, &updated)`.
  - `echo-tools/src/files/edit.rs:231` — `EditFileTool`:
    `tokio::fs::write(&path, &updated)`.
  - `echo-tools/src/files/files.rs:190` — `CreateFileTool`:
    `tokio::fs::write(&path, "")`.
- Reachability: every mutating file operation in the framework.
- Expected invariant: a process or power loss mid-write must not corrupt
  the target file; either the old content or the new content is fully
  present after recovery.
- Observed behavior: `tokio::fs::write` opens with `O_WRONLY|O_CREAT|O_TRUNC`
  and writes the payload; if interrupted between `truncate` and the final
  `fsync`, the file is left truncated/partial. `EditFileTool` mitigates
  by creating a `{path}.bak` (edit.rs:222-228) but the live file is still
  corrupted on crash and the `.bak` is never cleaned up; `WriteFileTool`
  does not even create a `.bak`. `UpdateFileTool` does neither.
- Impact: silent data corruption on crash / power loss. AGENTS.md
  explicitly endorses "防止框架自身 bug 造成破坏" and "防止用户无意中
  的数据丢失" as legitimate local safety concerns.
- Root cause: convenience over correctness — `tokio::fs::write` is the
  one-liner; the atomic pattern (write to `<path>.tmp`, `fsync`,
  `tokio::fs::rename`) is more code.
- Direction: introduce a single `atomic_write(path, bytes)` helper in
  `echo-tools/src/files/` (or `echo-core`) that performs the
  temp-file-then-rename dance, and migrate `WriteFileTool`,
  `UpdateFileTool`, `EditFileTool` to it. The `.bak` sidecar in
  `EditFileTool` can be removed once atomic write lands (it becomes
  redundant). `AppendFileTool` (true append semantics) can stay as-is.
- Regression validation: a fixture that simulates a crash mid-write (e.g.
  by panicking between `write` and `rename` via an injected hook) and
  asserts the original file is intact; a fixture that asserts no `.bak`
  files accumulate after atomic-write migration.
- Validation reports: [V03](../validations/F-EXT-02/V03-01.md)

### F-EXT-02-P2-01: `UpdateFileTool` duplicates `EditFileTool` with strictly fewer safety features

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-tools/src/files/files.rs:853-976` — `UpdateFileTool` (read,
    `contains` check, `replacen(..., 1)`, `tokio::fs::write`).
  - `echo-tools/src/files/edit.rs:49-257` — `EditFileTool` (read,
    `contains` check, multi-occurrence gate, dry_run, `.bak`, git
    checkpoint, diff output).
- Reachability: both tools are registered (see `registry.rs`) and exposed
  to the LLM. An LLM may pick either with similar prompts.
- Expected invariant: there is one canonical file-content-replacement
  tool; if a second exists, it must be a strict superset, not a strict
  subset.
- Observed behavior: `UpdateFileTool` and `EditFileTool` accept identical
  parameters (`path`, `old_content`, `new_content`) and perform identical
  semantics, but `UpdateFileTool` lacks every safety feature:

  | Capability | `EditFileTool` | `UpdateFileTool` |
  |---|---|---|
  | Multi-occurrence rejection | yes | no — silently replaces first |
  | `replace_all` mode | yes | no |
  | `dry_run` preview | yes | no |
  | Unified diff output | yes | no |
  | `.bak` backup | yes | no |
  | Git checkpoint | yes | no |

- Impact: an LLM that uses `UpdateFileTool` instead of `EditFileTool` for
  a multi-occurrence `old_content` silently edits only the first
  occurrence and reports success — confusing agent behavior and silent
  data drift. Per AGENTS.md "能复用就不新建,能扩展就不另起" and the
  framework's no-duplicate-authority rule (REPORTING.md P2).
- Root cause: `UpdateFileTool` likely predates `EditFileTool` and was not
  retired when `EditFileTool` was added.
- Direction: delete `UpdateFileTool` and its registration in
  `registry.rs`. Any external caller depending on `update_file` should
  migrate to `edit_file` (which is a strict superset). Per AGENTS.md
  "无需兼容,过时代码可直接删".
- Regression validation: `cargo check --workspace --all-features` after
  deletion; update any tests in `registry.rs` or downstream crates.
- Validation reports: [V03](../validations/F-EXT-02/V03-01.md)

### F-EXT-02-P2-02: Worktree tools ignore the agent's `working_dir`

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-tools/src/worktree_tool.rs:78` — `EnterWorktreeTool::execute`
    signature (context-free).
  - `echo-tools/src/worktree_tool.rs:175` — `ExitWorktreeTool::execute`.
  - `echo-tools/src/worktree_tool.rs:278` — `ListWorktreesTool::execute`.
  - `echo-tools/src/worktree_tool.rs:92-95, 182-185, 280-283` — `repo_path`
    defaults to `"."`, interpreted relative to the *process* cwd.
  - Compare `echo-tools/src/git.rs:552-569` — `effective_repo_path`
    correctly threads `ctx.working_dir` for the sibling git tools.
- Reachability: every worktree tool invocation in a session that has
  `ctx.working_dir` set (the default for subagent / worktree-isolated
  sessions).
- Expected invariant: tools that operate on a repository respect the
  session's working directory binding, like the file and git tools do.
- Observed behavior: the three worktree tools implement `fn execute`
  rather than `fn execute_with_context`, so `ctx.working_dir` is dropped
  on the floor. An agent inside a worktree calling `enter_worktree` would
  target the *process* cwd instead of its session worktree.
- Impact: breaks worktree isolation for nested worktree creation,
  confuses `list_worktrees` output, and can cause `exit_worktree` to
  operate on the wrong repository.
- Root cause: the worktree tools predate the `ctx.working_dir`
  convention; they were not migrated when `git.rs` added
  `effective_repo_path`.
- Direction: switch all three to `execute_with_context` and route
  `repo_path` through a helper equivalent to `effective_repo_path`
  (ideally extract `effective_repo_path` to a shared module so both
  `git.rs` and `worktree_tool.rs` use it).
- Regression validation: a test analogous to
  `test_shell_honors_context_working_dir` (shell.rs:1914) and
  `test_create_file_lands_in_context_working_dir` (files.rs:1290) for
  `EnterWorktreeTool`.
- Validation reports: [V04](../validations/F-EXT-02/V04-01.md)

### F-EXT-02-P2-03: `exit_worktree` missing-worktree fallback force-deletes a namesake branch

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-tools/src/worktree_tool.rs:195-210` — fallback that synthesizes
    `ManagedWorktree { managed: true, branch: <path-basename> }` when
    `worktree_path` is not in `git worktree list`.
  - `echo-tools/src/git_worktree.rs:115-147` — `remove_worktree`
    unconditionally runs `git worktree remove --force` and
    `git branch -D -- <branch>`.
- Reachability: any `exit_worktree` call where the path is not currently
  registered as a worktree (e.g. it was already removed, or the path was
  typo'd, or the worktree was created outside this session).
- Expected invariant: if the requested worktree is not in the registry,
  the tool should report an error, not synthesize a fake "managed" entry
  and force-delete whatever branch matches its basename.
- Observed behavior: the synthesized `ManagedWorktree` has `managed: true`
  and `branch: <basename-of-path>`, so `remove_worktree` proceeds to
  `git branch -D -- <basename>`. If a real branch happens to share the
  name (e.g. path ends in `/main`), it is force-deleted with no confirmation.
- Impact: silent branch deletion. AGENTS.md "防止用户无意中的数据丢失".
- Root cause: the fallback was added to "be lenient" but ignores the
  destructive consequence of `managed: true` + `git branch -D`.
- Direction: when the path is not in `git worktree list`, return an error
  ("worktree not found, refusing to clean up") rather than synthesizing.
  If lenient cleanup is desired, the synthesized entry should have
  `managed: false` so `remove_worktree` skips the `git branch -D` step.
- Regression validation: a test calling `exit_worktree` with an
  unregistered path containing a basename that matches an existing branch,
  asserting the branch is preserved.
- Validation reports: [V04](../validations/F-EXT-02/V04-01.md)

### F-EXT-02-P2-04: `merge_worktree` leaves the repo in MERGE state on conflict and disrupts concurrent workers

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-tools/src/git_worktree.rs:202-232` — `merge_worktree` runs
    `git checkout <target_branch>` then `git merge --no-edit <branch>`;
    on merge failure it returns `Err` without rollback.
- Reachability: any `exit_worktree { merge_to: ... }` invocation where
  the merge has conflicts (a routine case for parallel subagent work).
- Expected invariant: on merge failure, the repository is returned to
  the pre-merge state (or at least to a discoverable MERGE state with
  actionable guidance); concurrent workers in the main worktree are not
  disrupted.
- Observed behavior:
  1. `git checkout <target_branch>` (line 202) operates on the main
     worktree's HEAD. If another subagent or the user is working there,
     the checkout silently switches their branch out from under them.
  2. On merge conflict, the function returns `Err("Merge conflict or
     error: <stderr>")` with no `git merge --abort`; the repo is left
     with conflict markers in working files and `MERGE_HEAD` set.
- Impact: a failed merge breaks the main worktree for all other workers
  and requires manual recovery. Concurrent-writer correctness violated.
- Root cause: the function assumes the caller has exclusive access to the
  main worktree and that conflicts are someone else's problem.
- Direction:
  - Run the merge in a fresh short-lived worktree on `target_branch`
    rather than checking out the main worktree (the same pattern
    `create_worktree` uses).
  - On merge failure, `git merge --abort` before returning `Err`, or
    document explicitly that the caller must abort.
  - Return a structured error (with `ToolFailure::category = Permanent`
    and a postcondition) rather than a string error.
- Regression validation: a fixture that creates a conflict and asserts
  `git status` shows no `MERGE_HEAD` after `merge_worktree` returns Err;
  a fixture that a concurrent worker's branch is unchanged.
- Validation reports: [V04](../validations/F-EXT-02/V04-01.md)

### F-EXT-02-P2-05: `create_worktree` lacks `--` separator after `-b <branch>`, enabling flag-shaped branch names

- Priority: P2
- Confidence: medium
- Layer: framework
- Evidence:
  - `echo-tools/src/git_worktree.rs:65-74` — `cmd.args(["worktree", "add"])`
    then either `["-b", &config.branch, &worktree_dir, "--", base]` or
    `["-b", &config.branch, &worktree_dir]`. The `--` is placed after the
    path, before `base`; it never protects `-b`'s argument.
  - Compare `echo-tools/src/git.rs:438, 441, 449, 528` — `git_branch` and
    `git_commit` correctly insert `--` immediately after `branch` /
    `add` to defend against flag-shaped names.
- Reachability: any `EnterWorktreeTool` invocation where the LLM supplies
  `branch = "--detach"` (or similar). V04 empirically confirmed
  `git worktree add -b --force <path>` parses `--force` as the branch
  name.
- Expected invariant: a branch name is never parsed as a git option.
- Observed behavior: git's own argv parser decides; `git worktree add -b
  --force <path>` was accepted and printed "Preparing worktree (new
  branch '--force')". A branch named `--detach` / `-B` / `--no-checkout`
  could silently change the worktree-add semantics rather than creating
  the named branch.
- Impact: confused-deputy / silent-semantic-shift. An LLM-supplied
  flag-shaped branch name could create a worktree in an unexpected state
  (e.g. detached HEAD instead of a new branch).
- Root cause: missing `--` separator; inconsistent with the sibling
  `git_branch` tool.
- Direction: insert `--` immediately after `&config.branch` to terminate
  option parsing, mirroring the `git_branch` pattern.
- Regression validation: a test that calls `create_worktree` with
  `branch = "--force"` and asserts either rejection or a literal
  branch named `--force` (caller's choice), not silently altered
  semantics.
- Validation reports: [V04](../validations/F-EXT-02/V04-01.md)

### F-EXT-02-P3-01: `cleanup_direct_child` invokes synchronous `kill` in an async context

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-tools/src/shell.rs:847-853` —
  `std::process::Command::new("kill").args(...).status()` inside an
  `async fn`.
- Reachability: every direct (non-sandbox) shell cancellation.
- Expected invariant: async code does not block the tokio worker thread.
- Observed behavior: the synchronous `kill` invocation blocks the worker
  for the syscall's duration (typically <1ms, but can be longer under
  load or with signal-handler latency).
- Impact: very low in practice; technically a blocking call in async.
- Root cause: convenience; `tokio::process::Command` would be the
  async-correct choice but adds an await.
- Direction: switch to `tokio::process::Command::new("kill")...status().await`
  or, better, use the `nix` crate's `killpg`/`kill` syscalls directly
  (no subprocess at all). Alternative: send signals via the
  `libc::kill` syscall through a `tokio::task::spawn_blocking` boundary.
- Regression validation: existing `dropping_shell_stream_kills_and_reaps_child`
  test still passes.
- Validation reports: [V02](../validations/F-EXT-02/V02-01.md)

### F-EXT-02-P3-02: `EditFileTool::find_occurrence_lines` uses byte slicing on `&str` (safe but violates AGENTS.md guidance)

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-tools/src/files/edit.rs:263` — `haystack[search_from..].find(needle)`.
  - `echo-tools/src/files/edit.rs:265` — `haystack[..abs_pos].lines().count() + 1`.
- Reachability: only on the error path (when `old_content` matches
  multiple locations).
- Expected invariant: per AGENTS.md, "全部使用 `take`,禁止字节截断".
- Observed behavior: the slices are safe by construction (both
  `search_from` and `abs_pos` are derived from `str::find()`, which
  returns UTF-8-boundary byte offsets; `needle.len()` is the byte length
  of a valid `&str`, also boundary-aligned). They cannot panic and cannot
  split a multibyte character.
- Impact: none functionally; the rule violation is cosmetic.
- Root cause: the find-based byte-offset idiom is standard Rust but
  technically violates the AGENTS.md preferred form.
- Direction: either (a) leave as-is with a comment documenting why the
  slices are UTF-8-safe (cheapest), or (b) refactor to a char-iterator
  scan that avoids byte slicing entirely (matches the rule).
- Regression validation: existing `test_find_occurrence_lines` and
  `test_find_occurrence_lines_multiline` tests still pass.
- Validation reports: [V01](../validations/F-EXT-02/V01-01.md)

### F-EXT-02-P3-03: `git_checkpoint` runs synchronous git subprocesses inside async tool bodies

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-tools/src/git_checkpoint.rs:13-17, 31-35, 66-69, 79-82, 93-97`
    — all `std::process::Command::new("git")`.
  - Called from `EditFileTool::execute_with_context` (edit.rs:216-219),
    `DeleteFileTool::execute_with_context` (files.rs:289-292),
    `WriteFileTool::execute_with_context` (files.rs:688-693) — all
    `async fn`.
- Reachability: every write/edit/delete on a file inside a git
  repository.
- Expected invariant: async tool bodies do not block the tokio worker
  thread on subprocess invocation.
- Observed behavior: each mutation triggers 1-3 synchronous `git
  rev-parse` / `git tag` / `git tag -l` subprocess invocations on the
  async worker thread. Under concurrent subagent writes this serializes
  on the worker.
- Impact: latency / throughput; not a correctness defect.
- Root cause: `git_checkpoint` predates the async migration of the file
  tools.
- Direction: move the git invocations behind
  `tokio::task::spawn_blocking`, or rewrite with `tokio::process::Command`.
- Regression validation: existing tests still pass.
- Validation reports: [V03](../validations/F-EXT-02/V03-01.md)

### F-EXT-02-P3-04: `git_checkpoint` tag name uses second-resolution timestamps, colliding under concurrent writers

- Priority: P3
- Confidence: medium
- Layer: framework
- Evidence: `echo-tools/src/git_checkpoint.rs:24-28` —
  `format!("echo-checkpoint/{}", timestamp.as_secs())`.
- Reachability: any two file mutations within the same wall-clock second
  (likely under parallel subagent execution).
- Expected invariant: each checkpoint gets a unique tag name.
- Observed behavior: two mutations in the same second produce the same
  tag name; the second `git tag` invocation fails silently (tag exists),
  `create_checkpoint` returns `None`, and the caller proceeds *without*
  a checkpoint. The cleanup (`cleanup_old_checkpoints`) also keeps only
  the last 10 globally, so concurrent writers delete each other's tags.
- Impact: some mutations silently skip checkpoint creation, weakening the
  rollback safety net. Not data loss directly, but a recovery gap.
- Root cause: timestamp resolution too coarse; no per-process /
  per-call uniquification.
- Direction: include a counter / pid / nanosecond timestamp in the tag
  name, or use a content-derived hash.
- Regression validation: a fixture that fires two `create_checkpoint`
  calls in the same second and asserts distinct tags.
- Validation reports: [V03](../validations/F-EXT-02/V03-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Path/Unicode safety across shell, file, code, git tools | yes | passed | [V01-01](../validations/F-EXT-02/V01-01.md) |
| V02 | Process-tree cancellation kills descendants and reaps | yes | passed | [V02-01](../validations/F-EXT-02/V02-01.md) |
| V03 | Atomic writes, conflict detection, edit application safety | yes | failed | [V03-01](../validations/F-EXT-02/V03-01.md) |
| V04 | Worktree creation, cleanup, conflict, isolation safety | yes | failed | [V04-01](../validations/F-EXT-02/V04-01.md) |
| V05 | Historical-document drift check | conditional | n/a | See Historical Claim Status; historical docs are referenced only as context, not as evidence for any current claim. |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `echo-agent/docs/{zh,en}/34-git-isolation.md` "Use `rollback_to_checkpoint()` to restore worktree state on error" | stale | `rollback_to_checkpoint` (git_checkpoint.rs:45) has zero non-test, non-doc callers across `echo-agent` and `echo-agent-cli`. The recovery flow it documents is not wired into any tool or runtime path. The function exists as dead public API; per AGENTS.md dead-code rule it is a deletion candidate. |
| `echo-agent/docs/{zh,en}/34-git-isolation.md` "Worktrees isolate parallel subagent work" | regressed | The *intent* is correct, but the current `EnterWorktreeTool`/`ExitWorktreeTool` do not honor `ctx.working_dir` (F-EXT-02-P2-02), `path_suffix` admits traversal outside `.worktrees/` (F-EXT-02-P1-01), and `merge_worktree` disrupts concurrent workers (F-EXT-02-P2-04). The documented isolation is not actually delivered by current code. |
| `echo-agent/docs/{zh,en}/30-react-safety.md` "checkpoint + rollback is the safety net for file mutations" | partially current | `create_checkpoint` is wired into `EditFileTool` / `WriteFileTool` / `DeleteFileTool` (current), but `rollback_to_checkpoint` is not (stale). The forward path is current; the recovery path is aspirational. |
| F-EXT-01 "tools route `PartialSideEffect` / `Timeout` to `VerifyThenRetry`" | current | `WriteFileTool` (files.rs:678-685, 700-707) emits `PartialSideEffect + idempotency_key + postcondition` exactly as F-EXT-01 specified; `EditFileTool` and `UpdateFileTool` do not emit partial-side-effect metadata on failure (a minor gap, not contradicting F-EXT-01 which is about the contract, not the builtin). |

## Coverage And Uncertainty

- **Code not inspected:**
  - `echo-tools/src/files/glob.rs`, `grep.rs`, `code_search.rs`,
    `repo_map.rs`, `artifact.rs` — read-priority tools; spot-checked for
    byte slicing and panic APIs (none found in production code outside
    `glob.rs`'s own comments documenting its char-iterator refactor at
    line 305, 439). A full audit of these is the boundary between this
    task and F-EXT-03; their risk profile is low (read-only).
  - `echo-tools/src/files/files.rs::MoveFileTool` — read but not deeply
    audited for cross-filesystem rename fallback (line 1087 uses
    `tokio::fs::rename`, which fails across filesystems; no fallback).
    Note: low impact.
- **Validations not executed at runtime:**
  - V01 is mostly static inspection; the path-confine and UTF-8 claims
    are corroborated by the existing `test_shell_honors_context_working_dir`,
    `test_create_file_lands_in_context_working_dir`, and
    `read_file_is_utf8_safe_and_rejects_oversized_single_lines` tests
    that passed in the V03 run, but V01 itself did not run those tests
    directly.
  - V04's path-traversal reproduction was executed manually with raw
    `git` commands rather than through a Rust test, because no test
    exercises this scenario today.
- **Environmental limits:**
  - The first build of `echo_tools` (default features) took ~2.5 minutes
    due to a cold dependency build and a package-cache file lock. The
    cancellation test only runs with `--features shell`; it is filtered
    out of default-feature runs (echo_tools has `default = []`), which is
    itself a feature-isolation gap but belongs to F-FEAT-01, not here.
- **Claims that remain uncertain:**
  - Whether the `git_checkpoint` tags are observed by any EKO or
    downstream consumer. The `git_checkpoint` metadata is returned in the
    `ToolResult::metadata` field, but no audited caller reads it. If no
    caller uses it, the entire `git_checkpoint` subsystem is dead safety
    weight (a recovery net with no recovery consumer) and a deletion
    candidate pending confirmation in A-TOOL-01.
  - Whether `MoveFileTool` cross-filesystem fallback matters in
    practice. None of the audited callers move files across filesystems.

## Handoff

- Conclusions downstream tasks may rely on:
  - The `Tool` / `ToolResult` / `ToolFailure` contract established by
    F-EXT-01 is correctly consumed by `WriteFileTool` and `RunCodeTool`
    (full `PartialSideEffect + idempotency_key + postcondition`
    reporting). `EditFileTool`, `UpdateFileTool`, and `AppendFileTool`
    have gaps in partial-side-effect reporting — A-TOOL-01 should
    verify this is intentional or flag it.
  - Process-tree cancellation in `ShellTool` is correct and tested; the
    runtime cancellation token propagates correctly. F-RCT-04 (tool batch
    execution) can rely on this.
  - Path confinement is uniformly correct via `resolve_path` for the file
    family and via `validate_script_path` for `run_code`. F-SEC-01 can
    rely on this for the file/code surface.
  - Worktree isolation is *not* safe in its current form. A-TSK-05
    (worktree/file-ownership policy) must not assume the framework's
    worktree tools are correct — they need to be fixed first (P1-01) or
    worked around at the application layer.
  - `git_checkpoint` may be entirely dead weight if no caller consumes
    the `git_checkpoint` metadata. A-TSK-05 / A-TOOL-01 should confirm.
- Reports they must read:
  - [V01-01](../validations/F-EXT-02/V01-01.md) for the path/Unicode
    evidence (no findings; P3-02 only).
  - [V02-01](../validations/F-EXT-02/V02-01.md) for the cancellation
    evidence (P3-01 only).
  - [V03-01](../validations/F-EXT-02/V03-01.md) for the atomic-write /
    duplicate-tool evidence (covers P1-02, P2-01, P3-03, P3-04).
  - [V04-01](../validations/F-EXT-02/V04-01.md) for the worktree evidence
    (covers P1-01, P2-02, P2-03, P2-04, P2-05).
- Conditions that make this report stale:
  - Any change to `resolve_path` (files/mod.rs) invalidates V01 and the
    path-confinement claim.
  - Any change to `start_direct_stream` / `cleanup_direct_child`
    (shell.rs) invalidates V02 and the cancellation claim.
  - Introduction of an `atomic_write` helper, deletion of
    `UpdateFileTool`, or migration of `git_checkpoint` to spawn_blocking
    invalidates V03 and the corresponding findings.
  - Any change to `create_worktree` / `EnterWorktreeTool` /
  `merge_worktree` invalidates V04 and the corresponding findings.
- Follow-up task IDs (no fixes implemented in this review):
  - A-TOOL-01 should verify the application adapter correctly consumes
    `ToolResult::failure.side_effect` and `postcondition` for the file
    family (relates to the partial-side-effect gaps in `EditFileTool` /
    `UpdateFileTool` / `AppendFileTool`).
  - A-TSK-05 (worktree, file ownership, merge policy) must not assume
    the framework's worktree tools are safe; P1-01 and P2-02/03/04/05
    should be fixed first or worked around at the application layer.
  - X-BND-01 / X-INV-01 should fold the duplicate `UpdateFileTool` /
    `EditFileTool` authority and the dead `rollback_to_checkpoint` API
    into their cross-repository duplicate/dead-code inventory.
  - Q-FLT-01 / Q-FLT-02 should add crash-mid-write, concurrent-write
    race, and worktree-traversal fixtures as called out in V03 / V04
    follow-ups.
