# F-EXT-02: Shell, file, code, and Git tools

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean

## Question

Are the common local developer tools (shell, run_code, file read/write/edit,
grep/glob/search, git, worktree) correct for paths, UTF-8, atomic writes,
diff application, cancellation, process cleanup, and isolation?

## Scope

- `echo-tools/src/shell.rs` (full read), `code.rs` (full read),
  `files/mod.rs` resolve_path (full), `files/files.rs` (create/read/write/
  append/update/move/list), `files/edit.rs`, `files/diff.rs`, `files/grep.rs`,
  `files/glob.rs`, `files/code_search.rs`, `files/repo_map.rs` (partial),
  `files/artifact.rs` (ReadArtifactTool cursor/path resolution),
  `git.rs` (all tools + run_git), `git_worktree.rs`, `worktree_tool.rs`,
  `git_checkpoint.rs`, `diff_pagination.rs`, `security.rs` (PathValidator
  section), `registry.rs` (registration/reachability).
- Behavior verification: scratch program in /tmp replicating
  `find_occurrence_lines` and `glob_with_doublestar`; unit test runs for
  shell/files/git.

## Out Of Scope

- Data/research/media/database/network/web tools → F-EXT-03 (path validator
  usage by data tools only cross-referenced via F-SEC-01).
- Guard/sandbox internals and secret redaction → F-SEC-01 (re-read, no
  re-review).
- Tool contract / pagination / artifact-writer infrastructure → F-EXT-01
  (re-read; only tool-side compliance checked here).
- Retry math → F-REL-01 (re-read; no new retry sites found in these tools).
- EKO application wiring of tools beyond registration/context binding.

## Inputs

- Root `AGENTS.md` (UTF-8 rule, panic rule, local threat model), shared
  `README.md`, `REPORTING.md`, `TASKS.md` (F-EXT-02 card),
  `zcode-ds/README.md`.
- Dependency reports read: zcode-ds `F-EXT-01` (complete), `F-SEC-01`
  (complete), `F-REL-01` (complete).
- Historical documents treated as hypotheses: AGENTS.md UTF-8/panic rules;
  F-EXT-01 P2-01 WRITE_TOOLS drift; F-SEC-01 P3-06 validator convergence.

## Layering Decision

- Generic mechanism (framework, `echo_tools`): shell/run_code/files/git/
  worktree tools, resolve_path, git_checkpoint — all correctly placed; the
  framework crate is the right home for these reusable domain tools.
- EKO product policy (application): no new policy found inside scope; EKO
  consumes the tools via `register_all_tools` with no `with_base_dir`
  (confinement comes from `ctx.working_dir` per call,
  `echo-agent-cli/echo-agent-app-core/src/infra.rs:901`,
  `echo-agent/src/agent/react/run/pipeline.rs:495-512`).
- Adapter boundary: none new inside scope.
- Duplicate search terms (both repositories): `validate_within_base`,
  `validate_output_file`, `resolve_path`, `apply_patch|patch`, `create_
  checkpoint|git_checkpoint`, `effective_repo_path`, `find_git_root`,
  `kill_on_drop`, `process_group`, `catch_unwind`, `enter_worktree`/
  `exit_worktree`, `glob_with_doublestar|glob_match_advanced`. Results:
  no second path resolver, no patch-apply tool, no catch_unwind barrier;
  PathValidator (`validate_within_base`) unused by these tools (F-SEC-01
  P3-06 handoff confirmed); `git_status`'s `#[allow(dead_code)]` is spurious
  — the tool IS registered (registry.rs:263).

## Current Path

Verified data flow per tool family:

1. **Shell** (`shell` feature, registry.rs:202): `execute_stream_with_context`
   → `check_command_safety` (metacharacter rejection incl. `\n`, DANGEROUS
   blocklist, REQUIRE_APPROVAL, strict whitelist, git/cargo subcommand
   rules) → direct argv path (`tokio::process::Command`, kill_on_drop,
   `process_group(0)`, current_dir=ctx.working_dir, 60s default/300s cap
   timeout) with incremental UTF-8 decoding, retained-output cap 1 MiB,
   artifact spill via `ToolOutputArtifactWriter`; or sandbox path
   (`sh -c` when metacharacters present, cleanup delegated to executor).
   Cancellation = stream drop (`tx.closed()` → `kill -KILL -<pid>` on the
   process group); timeout → Timeout category + Possible side effect.
2. **run_code** (registry.rs:206): validates language, mutually-exclusive
   code/script_path; script path canonicalized against working_dir
   (symlink-safe); hardcoded `IsolationLevel::OsSandbox` floor + executor
   pre-checks (fail closed, F-SEC-01 V03); pre-checks `ctx.cancel`, passes
   it into `execute_with_limits_and_cancel`; Cancelled/Timeout/Permanent
   categories with Possible side effect + postconditions; output capped at
   1 MiB via `enforce_output_limit` (UTF-8-safe, tested).
3. **Files** (registry.rs:224-238): every tool resolves via `resolve_path`
   (canonicalize + parent-canonicalize for not-yet-existing outputs; lexical
   `..` rejection; confinement to base_dir else working_dir). write_file
   supports expected_hash optimistic concurrency and returns
   PartialSideEffect + idempotency key on write failure; edit_file requires
   an exact anchor, rejects ambiguous occurrences, checkpoints + `.bak`
   before write; delete_file checkpoints before removal; read_file is
   UTF-8/GBK-aware with token-bounded paging (saturating arithmetic);
   grep/glob/code_search walk with ignore lists and root confinement (grep
   additionally allows the artifact root); diff computes unified diffs
   paginated by `diff_pagination::split_unified_diff` with fingerprint-bound
   cursors (F-EXT-01 V03).
4. **Git** (registry.rs:263-271): all tools run synchronous
   `std::process::Command::new("git")` in `run_git` (argv, `--` separators,
   `-` prefix guard on diff target, `..` rejection on repo_path); git_status
   .. git_commit are read/parametrized; worktree tools create/merge/list/
   remove worktrees under `git_root/.worktrees/`; file tools call
   `git_checkpoint::create_checkpoint` (tag `echo-checkpoint/<unix-ts>`)
   before mutations on existing files.

## Findings

### F-EXT-02-P1-01: `edit_file` with empty `old_content` panics — byte-slice out of bounds / non-char-boundary in `find_occurrence_lines`

- Priority: P1
- Confidence: high (code fact + empirically reproduced with the exact logic)
- Layer: framework
- Evidence: `echo-tools/src/files/edit.rs:260-270`
  (`haystack[search_from..]` where `search_from = abs_pos + needle.len()
  .max(1)`); entry chain `edit.rs:152` (`original.contains("")` is always
  true → no early rejection), `edit.rs:168-177` (`matches("").count() ≥ 2`
  for any non-empty file → `find_occurrence_lines` called); `old_content`
  is a plain optional string parameter (`edit.rs:102-105`), empty string is
  valid JSON.
- Reachability: `edit_file` registered at `registry.rs:235` (feature
  `files`); EKO default toolset via `register_all_tools`
  (`echo-agent-cli/echo-agent-app-core/src/infra.rs:901`); no `catch_unwind`
  anywhere in echo-execution/agent tool execution
  (`echo-execution/src/tools.rs:686-698` awaits the tool future inline) — a
  panic propagates into the agent run task and aborts the run.
- Expected invariant: AGENTS.md hard rule — no API that panics on
  abnormal input; LLM-supplied parameters must never crash the tool.
- Observed behavior: `edit_file(path, old_content="", new_content=...)` on
  a non-empty file panics: ASCII content → "start byte index 4 is out of
  bounds"; Chinese content → "byte index 1 is not a char boundary" (verified
  by scratch run replicating the function verbatim, V01-01).
- Impact: a plausible LLM mistake (empty replacement anchor) crashes the
  tool call and the whole agent run; violates the project's strictest
  invariant.
- Root cause: `find_occurrence_lines` advances `search_from` by byte length
  and slices `haystack[search_from..]`; the `needle.len().max(1)` hack was
  meant to guard the empty-needle case but produces a non-char-boundary /
  out-of-bounds offset instead of terminating.
- Direction: reject empty `old_content` with InvalidArguments before the
  occurrence scan (or use `match_indices`/char-boundary-safe iteration and
  terminate cleanly on empty needle); add a regression test with
  `old_content=""` on both ASCII and multibyte files asserting a tool error
  (not a panic).
- Regression validation: the new test above; existing edit tests stay green;
  optionally `cargo test -p echo_tools --features files`.
- Validation reports: [V01-01](../validations/F-EXT-02/V01-01.md)

### F-EXT-02-P2-01: `glob` tool `**` patterns never match real files — documented capability silently broken

- Priority: P2
- Confidence: high (logic + empirical verification)
- Layer: framework
- Evidence: `echo-tools/src/files/glob.rs:240-284` (`glob_with_doublestar`
  compares the raw pattern text: `path_str.ends_with("*.rs")` for
  `**/*.rs`, `rest.ends_with("*.rs")` for `src/**/*.rs`) — verified:
  `**/*.rs` on `/a/b/main.rs` → false; only a file literally named `*.rs`
  matches; description advertises `**` (`glob.rs:47-48`, `glob.rs:213`).
- Reachability: `GlobTool` registered at `registry.rs:234`; every framework
  consumer with the `files` feature (EKO included) exposes it to the model.
- Expected invariant: documented glob semantics — `**` matches any path
  prefix, `*.{rs,ts}` braces, `?` single char (tests pin `*`/`?`/braces
  only; no test covers `**`).
- Observed behavior: `glob(pattern="**/*.rs", path=...)` returns
  "No files matching '**/*.rs' found" for any real tree.
- Impact: the model silently concludes no files match — wrong exploration
  results in code tasks; silent wrong answer is worse than an error.
- Root cause: doublestar implemented as literal `ends_with`/`contains` on
  the raw pattern instead of a segment-wise glob match; the fallback
  `glob_match_advanced` is never reached for `**` patterns.
- Direction: implement `**` by splitting the path into relative segments and
  matching prefix/suffix with `glob_match_advanced`, or convert the pattern
  to a regex with proper segment semantics; add tests with a real temp tree
  (`**/*.rs`, `src/**/*.rs`, `**/a/**/b.rs`).
- Regression validation: new glob integration tests; existing
  `test_glob_match_*` stay green.
- Validation reports: [V01-01](../validations/F-EXT-02/V01-01.md)

### F-EXT-02-P2-02: `exit_worktree`/`remove_worktree` swallow every error and can delete an unverified branch — false success + silent partial side effects

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-tools/src/git_worktree.rs:130-146` — `git worktree remove
  --force` failure falls through to `let _ = git worktree prune`, prune and
  `git branch -D` errors ignored, function returns `Ok(())` unconditionally;
  `worktree_tool.rs:195-210` — when the supplied `worktree_path` does not
  string-match a listed worktree (trailing slash, normalization difference),
  the fallback constructs `ManagedWorktree { managed: true, branch:
  file_name }` from the raw user path; `--force` on remove discards
  uncommitted/untracked worktree content without any warning message;
  `git branch -D` (git_worktree.rs:140-144) force-deletes.
- Reachability: `ExitWorktreeTool` registered at `registry.rs:270`;
  LLM-callable in EKO.
- Expected invariant: destructive cleanup only targets worktrees verified to
  belong to this repo; every failed step is reported; success implies the
  worktree was actually removed.
- Observed behavior: success is reported even when removal failed (worktree
  left on disk with its branch deleted, or vice versa); a worktree path with
  any normalization mismatch triggers the fallback and force-deletes a
  branch named after the arbitrary path's file_name.
- Impact: silent partial side effects and potential loss of branch refs /
  uncommitted worktree content; violates the tool's own "removed and branch
  deleted" success message.
- Root cause: error-channel `Result` ignored in favor of `let _ =`; the
  "treat as managed" fallback was added for convenience without verifying
  ownership.
- Direction: verify the target worktree exists in `list_worktrees` (canonical
  path comparison) before any destructive action; return the first failure
  instead of `Ok(())`; drop the arbitrary-path fallback or confine it to
  paths under `git_root/.worktrees/`; warn explicitly that uncommitted
  changes are discarded (`--force`), or require `merge_to`.
- Regression validation: fixture repo: exit_worktree with a non-listed path
  must error and must NOT delete any branch; a removal failure must surface
  as a failed ToolResult.
- Validation reports: [V04-01](../validations/F-EXT-02/V04-01.md)

### F-EXT-02-P2-03: `enter_worktree` `path_suffix` is not confined to `.worktrees/` — absolute or `..` suffixes create worktrees (full checkouts) outside the repo

- Priority: P2
- Confidence: high (mechanism), medium (severity judgment under local model)
- Layer: framework
- Evidence: `echo-tools/src/git_worktree.rs:44-51` —
  `git_root.join(".worktrees").join(suffix)`; `PathBuf::join` replaces the
  base for absolute suffixes and `..` escapes upward; only the branch-derived
  default name is sanitized (47-50); `path_suffix` is a documented parameter
  (`worktree_tool.rs:65-67`).
- Reachability: `EnterWorktreeTool` registered at `registry.rs:269`;
  LLM-callable in EKO.
- Expected invariant: worktrees are created under `git_root/.worktrees/`
  (module doc, AGENTS.md worktree conventions).
- Observed behavior: `path_suffix="../../other-repo"` or an absolute path
  creates the worktree (a full checkout, plus branch creation) at an
  arbitrary location; no containment check anywhere in
  `create_worktree`.
- Impact: the isolation feature itself can write outside the declared scope
  — files created in unrelated directories; in a nested-repo scenario the
  checkout lands inside another repository.
- Root cause: suffix treated as an opaque directory name with no
  normalization/validation, unlike the sanitized branch-derived default.
- Direction: validate `path_suffix` (reject empty, absolute, `.`/`..`,
  path separators) or resolve-and-confine canonically under
  `git_root/.worktrees/`; add a regression test with `path_suffix="../x"`
  and an absolute suffix expecting rejection.
- Regression validation: new test in git_worktree.rs; existing find_git_root
  tests stay green.
- Validation reports: [V04-01](../validations/F-EXT-02/V04-01.md)

### F-EXT-02-P3-01: git-family tools block the async runtime and ignore `ctx.cancel` (synchronous `std::process::Command::output()` in async tools)

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-tools/src/git.rs:598-608` (`run_git` sync `.output()`),
  `git_worktree.rs:60-105,119-147,153-157,202-231,235-248` (all sync),
  `git_checkpoint.rs:13-42,51-56,66-83` — invoked synchronously inside async
  file-tool paths (`files/files.rs:289`, `files/edit.rs:216`); zero
  `ctx.cancel` consultation in git.rs/git_worktree.rs/worktree_tool.rs
  (grep).
- Reachability: every git tool call, and every write_file/delete_file/
  edit_file on an existing file inside a git repo (2 extra git invocations
  per mutation: rev-parse + tag).
- Expected invariant: async tools must not block runtime workers; cancelled
  calls stop their processes (F-EXT-01 cancel contract; shell.rs is the
  reference implementation).
- Observed behavior: a slow `git log`/`git diff` on a large repo blocks one
  runtime worker for its duration; a cancelled run cannot stop the git
  process; checkpoints add blocking git calls to the hot write path.
- Impact: latency and concurrency degradation; cancellation contract
  deviation limited to the git family (run_code/shell are compliant —
  V02-01).
- Root cause: git tooling predates the async process pattern used by
  shell.rs; checkpoint helper written as plain std code.
- Direction: migrate `run_git`/worktree commands to `tokio::process::Command`
  with kill_on_drop + process_group, or `tokio::task::spawn_blocking`;
  defer checkpoint creation or make it best-effort async; consider batching
  per-mutation checkpoints.
- Regression validation: git tool unit tests with a fixture repo; a cancel
  test asserting the git child is killed on ctx.cancel.
- Validation reports: [V02-01](../validations/F-EXT-02/V02-01.md),
  [V05-03](../validations/F-EXT-02/V05-03.md)

### F-EXT-02-P3-02: grep/glob/code_search directory walks follow symlinks without cycle detection or re-confinement

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-tools/src/files/grep.rs:381-393` (`walk_and_search`
  recurses on `path.is_dir()` — symlinks followed, no canonical re-check,
  no visited set), `glob.rs:199-207` (same), `code_search.rs:289-291`
  (stack-based, same).
- Reachability: any grep/glob/code_search over a directory containing a
  symlink (agent-created or pre-existing) — the initial root is confined,
  the walk is not.
- Expected invariant: search results stay within the resolved root; walks
  terminate on any directory topology.
- Observed behavior: a symlinked dir inside the scope pulls files from
  outside the scope into results (read-only leak of paths outside the
  working dir); a symlink cycle → unbounded async recursion (heap growth)
  or infinite stack loop — tool hang / memory exhaustion.
- Impact: read-only, local model — but a hang/OOM in a search tool stalls
  the agent run; wrong results leak out-of-scope paths to the model.
- Root cause: walkers rely on `is_dir()` (follows symlinks) without
  canonicalization/cycle tracking, unlike `resolve_path` which canonicalizes
  its single entry.
- Direction: resolve each entry with `fs::canonicalize` (or
  `symlink_metadata` + explicit skip of symlinked dirs) and re-check
  containment against the canonical root; track visited canonical dirs;
  add a symlink-cycle test.
- Regression validation: fixture with `scope/link -> /etc` asserting no
  out-of-scope files in results; fixture with a self-referential symlink
  asserting termination.
- Validation reports: [V01-01](../validations/F-EXT-02/V01-01.md)

### F-EXT-02-P3-03: `update_file`/`append_file` lack the git-checkpoint recovery aid and all file writes are non-atomic

- Priority: P3
- Confidence: high (fact); medium (impact)
- Layer: framework
- Evidence: `files/files.rs:688-696,715-717` (write_file checkpoint),
  `files/edit.rs:216-236` (edit_file checkpoint + `.bak` backup),
  `files/files.rs:961-968` (update_file write — no checkpoint),
  `files/files.rs:824-839` (append_file — no checkpoint); all writes are
  direct `tokio::fs::write` (truncate+write, no temp+rename).
- Reachability: every update_file/append_file call on a tracked file.
- Expected invariant: mutation tools offer comparable recovery aid;
  crash/cancel mid-write must not silently corrupt content without recourse.
- Observed behavior: a crash or cancelled write mid-truncate leaves a
  truncated/partial file; update_file/append_file provide no checkpoint
  (and no hash postcondition on failure) to restore from, while
  write_file/edit_file/delete_file do.
- Impact: local, agent-driven edits — a lost edit with no recovery path;
  inconsistent protection across the write surface.
- Root cause: checkpoint/backup aid was added per-tool rather than in a
  shared write primitive; direct fs::write chosen for simplicity.
- Direction: route all mutations through a shared helper that checkpoints
  (or documents the choice) and writes via temp-file + rename for
  atomicity; add a failure-mode test asserting the on-disk state after a
  simulated mid-write abort.
- Regression validation: existing write/edit/delete tests stay green;
  new update_file checkpoint test.
- Validation reports: [V01-01](../validations/F-EXT-02/V01-01.md)

### F-EXT-02-P3-04: `files/mod.rs` `resolve_path` doc claims working_dir mode has "no confinement check" but the code confines to it

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-tools/src/files/mod.rs:24-27` (doc: "When only
  `working_dir` is set ... there is no confinement check") vs
  `mod.rs:45-70` (`effective_base = base_dir.or(working_dir)`; confinement
  + canonicalize checks run in both cases).
- Reachability: any framework consumer reading the doc to decide whether
  absolute paths are allowed when only working_dir is set.
- Expected invariant: doc describes code behavior (AGENTS.md: no misleading
  public API).
- Observed behavior: with only `ctx.working_dir` set, absolute paths outside
  it are rejected (the safer behavior, consistent with worktree isolation),
  but the doc says they are allowed; EKO's file-tool confinement therefore
  comes from working_dir (V06-01) — stricter than documented.
- Impact: framework consumers may design around the wrong contract; the
  stricter behavior itself is desirable.
- Root cause: doc written when working_dir was a pure CWD override; the
  confinement behavior landed later without doc update.
- Direction: fix the doc comment to state confinement applies whenever an
  effective base exists; add a test pinning working_dir confinement for
  absolute paths.
- Regression validation: new resolve_path test; existing worktree-cwd tests
  stay green.
- Validation reports: [V01-01](../validations/F-EXT-02/V01-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Path/Unicode/UTF-8/panic scan incl. empirical verification of find_occurrence_lines + glob_with_doublestar | yes | passed | [V01-01](../validations/F-EXT-02/V01-01.md) |
| V02 | Process-tree cancellation: shell direct/sandbox, run_code cancel, git family, panic barrier | yes | passed | [V02-01](../validations/F-EXT-02/V02-01.md) |
| V03 | Conflicting edit/diff checks, git argument injection guards, WRITE_TOOLS drift | yes | passed | [V03-01](../validations/F-EXT-02/V03-01.md) |
| V04 | Worktree create/reuse/merge/cleanup and partial-side-effect scenarios | yes | passed | [V04-01](../validations/F-EXT-02/V04-01.md) |
| V05 | `cargo test -p echo_tools --lib --locked --features "shell,files,git" shell` | conditional | passed (exit 0, 22 passed) | [V05-01](../validations/F-EXT-02/V05-01.md) |
| V05 | same command, filter `files` | conditional | passed (exit 0, 41 passed) | [V05-02](../validations/F-EXT-02/V05-02.md) |
| V05 | same command, filter `git` | conditional | passed (exit 0, 5 passed) | [V05-03](../validations/F-EXT-02/V05-03.md) |
| V06 | Cross-reference F-EXT-01/F-SEC-01 (cancel, artifact writer, validators, fallback clients) | conditional | passed | [V06-01](../validations/F-EXT-02/V06-01.md) |

Note: `echo_tools` defaults to `default = []`, so the tests require
`--features "shell,files,git"`; this is recorded in each V05 report.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| AGENTS.md: UTF-8-safe truncation mandatory (`chars().take`, no byte slicing) | regressed in one site | `files/edit.rs:260-270` empty-needle byte slicing panics (P1-01); all other sites compliant (V01-01) |
| AGENTS.md: no panicking APIs | regressed in one site | P1-01; no catch_unwind barrier contains it (V02-01) |
| F-EXT-01-P2-01: WRITE_TOOLS static list drifted from registered writers | current | registry.rs:206,267-270 mutating tools absent from `echo-agent/src/tools/mod.rs:118-139` (V03-01) |
| F-SEC-01-P3-06: `validate_within_base` zero callers; data-tool output validation lexical-only | current (unchanged) | security.rs:272-343 zero callers; file/git tools use resolve_path, not PathValidator (V01-01, V06-01) |
| F-SEC-01-P3-05: fallback `Client::new()` sites (5) | current (no new sites in scope) | V06-01 |
| F-SEC-01-P3-07: dead duplicate output guard | current (outside scope, unchanged) | V06-01 |
| MASTER-PLAN worktree isolation claims (AGENTS.md worktree section) | current with containment gaps | P2-03 (path_suffix escape), P2-02 (cleanup) |

## Coverage And Uncertainty

- Empirical verification of P1-01 and P2-01 used a scratch replica of two
  helper functions (/tmp/f-ext-02-check/check.rs), not the compiled tool
  end-to-end; the replicated code is verbatim from the reviewed files.
- No end-to-end `enter_worktree`/`exit_worktree` run in a real fixture repo
  was executed (no source modification allowed; git.rs/worktree_tool.rs
  have no unit tests — V05-03).
- `repo_map.rs` read only partially (root resolution, size cap, max_depth);
  `artifact.rs` re-verified for cursor/path only (F-EXT-01 V03 owns the
  writer contract).
- `git_commit` mid-loop staging partial effect (V03-01) recorded, not a
  finding (reversible, error returned).
- `git_log` unbounded `count` (V03-01) recorded, not a finding.
- read_file loads the whole file into memory before line-slicing (no size
  cap) — noted, not raised (token budget bounds the output).
- F-EXT-02-P2-03 severity is a judgment call under the local threat model
  (agent-driven writes outside `.worktrees/`); no end-to-end exploit path
  verified beyond the code fact.
- All V05 runs passed under the shared workspace file lock (parallel review
  builds expected); exit codes recorded per report.

## Handoff

- Downstream tasks may rely on: shell and run_code are UTF-8-safe and fully
  cancellable/cleanup-complete (V01/V02); file tools' resolve_path is the
  de-facto validator (canonical + parent-canonical) while
  `validate_within_base` remains unused (V01, V06); WRITE_TOOLS drift
  confirmed (V03); the git family has zero unit tests (V05-03) and is
  blocking/non-cancellable (P3-01).
- Reports to read: 8 validation reports in `validations/F-EXT-02/`.
- Conditions that make this report stale: changes to shell.rs process
  handling, code.rs isolation/cancel, resolve_path, edit_file
  occurrence logic, glob doublestar, worktree create/remove, or git
  command execution.
- Follow-up task IDs: F-EXT-01 (WRITE_TOOLS authority — P2-01 upstream of
  this task's registration evidence); F-SEC-01 (validate_within_base
  convergence for data tools; P3-06 fix should also cover the deep-
  nonexistent-parent fallback in resolve_path); Q-STA-01 (dead-code sweep
  candidates: spurious `#[allow(dead_code)]` on GitStatusTool, fetch.rs
  field); A-TOOL-* (EKO tool wiring — checkpoint policy per mutation is an
  application-side decision); F-RCT-02/03 (run abort on tool panic — P1-01
  interaction).
- Fixes are deferred to the final iteration roadmap; this review is
  read-only.
