# F-EXT-02: Shell, file, code, and Git tools

> Status: complete
> Reviewer: Codex primary reviewer (delegated static evidence independently sampled)
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: both source repositories clean; echo-agent transitioned from
> `9b0e0fa` with six externally owned dirty files to clean `3aa7929` during review

## Question

Are reusable shell, file, code, Git, and worktree tools correct for path/UTF-8
handling, atomic/conflict-safe mutation, cancellation, process/resource cleanup,
bounded output, and partial-side-effect reporting?

## Scope

- `echo-tools/src/shell.rs`, `code.rs`, `git.rs`, `git_checkpoint.rs`,
  `git_worktree.rs`, `worktree_tool.rs`, `registry.rs`.
- All implementations under `echo-tools/src/files`.
- Sandbox contract and Local/Docker/Kubernetes adapters used by shell/run_code.
- Root feature forwarding, ReactAgent registration, sandbox injection, and
  limited EKO searches needed to distinguish product commands from framework
  Tool duplication.
- Static test and historical-document inventory. No executable validation.

## Out Of Scope

- Generic Tool schema, registry collision, ToolManager cancellation, result
  envelope, cursor visibility, and artifact protocol owned by
  [F-EXT-01](F-EXT-01.md).
- General permission/HITL policy and security synthesis owned by F-SEC-01. Its
  report was unavailable at this handoff; later synthesis must backlink and
  deduplicate, not overwrite this task's concrete tool-contract evidence.
- EKO interactive terminal/file/worktree command correctness, TaskRuntime
  worktrees, provider/network tools, database/SQLite, source fixes, and roadmap
  design.
- Cargo, rustc, tests, builds, dynamic fixtures, Docker/Kubernetes access, and
  network activity, all prohibited for this review phase.

## Inputs

- Root `AGENTS.md`; shared `README.md`, `REPORTING.md`, `TASKS.md`; Codex review
  protocol and report templates.
- Dependency [F-EXT-01](F-EXT-01.md), used only to separate generic Tool
  contracts from domain implementation ownership.
- Current clean source at the commits above and scoped `echo-agent` Git history.
- F-SEC-01 was not read because no Codex task report existed at handoff. No
  other reviewer directory was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Path-safe filesystem access, atomic replacement, revision/conflict checks, cancellable bounded child processes, sandbox cleanup, Git/worktree state transitions, and truthful Tool action metadata are reusable framework mechanisms. |
| EKO product policy | Which tools a mode exposes, user approval UI, local artifact retention, explicit interactive `!command`, repository selection, and TaskRuntime worktree policy belong to EKO. The findings do not require cloud-style permissions or SQLite. |
| Adapter boundary | EKO may select/render framework Tools and separately offer explicit user commands. Those handlers are not a second framework executor. Adapters must not recreate mutation, Git, worktree, cancellation, or recovery authority. |
| Duplicate search | Searched both repositories by Tool/type/function names and by behaviors: read/write/edit/replace, checkpoint/rollback, Git/worktree create/merge/remove, process spawn/cancel/timeout, and registration. Framework ownership is concentrated in `echo-tools`; `UpdateFileTool` and `EditFileTool` are the material same-layer duplicate. |
| Migration deletion | Establish one atomic mutation primitive and make one edit Tool canonical; remove `UpdateFileTool` and its divergent tests/registration after migration. Replace the current HEAD-tag rollback and destructive worktree cleanup rather than preserving them as parallel safety paths. |

Public framework capabilities remain valid even when EKO does not invoke them.
The direction is to correct the reusable API, not delete Git, sandbox, or
worktree features because one application path has another implementation.

## Current Path

```text
Cargo shell/files/git features
  -> echo_tools::register_all_tools
  -> ReactAgent ToolManager
  -> ToolContext { working_dir, cancel, sandbox, artifacts }
       shell -> sandbox stream OR direct process-group stream
       run_code -> OsSandbox-or-better -> controlled sandbox execute
       files -> resolve_path -> direct tokio fs operations
       Git -> synchronous run_git(std::process::Command::output)
       worktree -> synchronous create/list/merge/remove helpers

file mutation
  -> optional HEAD tag checkpoint
  -> direct write/remove/rename
  -> metadata may expose checkpoint tag
rollback(tag)
  -> git checkout tag -- .
```

Positive controls are substantial: direct shell execution uses a process group,
kill-on-drop, stream receiver cleanup, incremental UTF-8 decoding, bounded
retained output, and optional full artifacts (`shell.rs:576`, `shell.rs:825`).
RunCode requires OS-level sandbox isolation, confines persisted script paths,
passes cancellation and resource limits, and preserves timeout/cancel result
categories (`code.rs:280`, `code.rs:332`, `code.rs:358`). These do not repair
the specialized Git/worktree or remote sandbox control-plane gaps below.

## Findings

### F-EXT-02-P0-01: Git checkpoints lose the state they claim to preserve and rollback overwrites unrelated edits

- Priority: P0
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-tools/src/git_checkpoint.rs:6`,
  `echo-agent/echo-tools/src/git_checkpoint.rs:23`,
  `echo-agent/echo-tools/src/git_checkpoint.rs:45`,
  `echo-agent/echo-tools/src/git_checkpoint.rs:51`,
  `echo-agent/echo-tools/src/files/files.rs:288`,
  `echo-agent/echo-tools/src/files/files.rs:687`,
  `echo-agent/echo-tools/src/files/edit.rs:215`
- Reachability: delete_file, write_file, and edit_file call create_checkpoint
  before live mutations; rollback_to_checkpoint is public and documented.
- Expected invariant: the returned recovery handle preserves exact pre-mutation
  tracked/staged/untracked target bytes and restores only that operation.
- Observed behavior: the checkpoint is a lightweight tag on HEAD, so it captures
  no index or working-tree changes. A rollback checks out the tag over `.` for
  the whole repository, overwriting unrelated tracked edits, while untracked
  content is not recoverable. Second-resolution tag names also collide.
- Impact: a user relying on the advertised safety net can permanently lose the
  target's actual prior content and unrelated local work.
- Root cause: commit identity is treated as file-mutation snapshot identity,
  and rollback scope is repository-wide despite accepting a file path.
- Direction: replace this mechanism with a collision-free per-operation snapshot
  that stores exact prior bytes/state and restores only owned paths. Delete the
  HEAD-tag path and repo-wide checkout once migrated; expose partial recovery
  facts explicitly.
- Regression validation: dirty tracked/staged/untracked/deleted/renamed files,
  two unrelated files, two checkpoints in one second, scoped rollback, failure
  mid-restore, and exact byte identity.
- Validation reports: [V05](../validations/F-EXT-02/V05-01.md),
  [V13](../validations/F-EXT-02/V13-01.md)

### F-EXT-02-P0-02: Worktree exit can force-delete unverified work and report success after failure

- Priority: P0
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-tools/src/git_worktree.rs:43`,
  `echo-agent/echo-tools/src/git_worktree.rs:115`,
  `echo-agent/echo-tools/src/git_worktree.rs:119`,
  `echo-agent/echo-tools/src/git_worktree.rs:130`,
  `echo-agent/echo-tools/src/git_worktree.rs:138`,
  `echo-agent/echo-tools/src/git_worktree.rs:193`,
  `echo-agent/echo-tools/src/worktree_tool.rs:187`
- Reachability: full registration exposes enter_worktree and exit_worktree;
  exit directly invokes merge/remove helpers from an async Tool.
- Expected invariant: only exactly identified owned worktrees are cleaned;
  dirty/unmerged work is preserved by default; partial Git failures are terminal
  and reported with current repository state.
- Observed behavior: a custom path suffix is not confined under `.worktrees`;
  an unknown exit path is synthesized as managed. Removal uses `--force`, then
  ignores removal failure, force-deletes the inferred branch with ignored
  status, returns Ok, and the Tool reports success. Merge failure can leave the
  main checkout switched/conflicted with no restoration.
- Impact: local uncommitted files and unmerged commits can be destroyed, an
  unrelated branch/path can be targeted, or the repository can remain partially
  mutated while the model/user is told cleanup succeeded.
- Root cause: path/branch strings stand in for stable ownership identity and a
  destructive multi-command sequence has no checked state machine.
- Direction: require a persisted ownership token and canonical in-root path;
  reject unknown worktrees; refuse dirty/unmerged cleanup unless a distinct
  explicit destructive action is approved; stop on every command failure and
  return precise partial state. Remove the synthetic-managed fallback and
  unconditional `--force`/`-D` path.
- Regression validation: escaping suffix, unknown/external path, dirty/untracked
  worktree, unmerged branch, remove/delete failure, conflict, original-branch
  restoration, and truthful partial result.
- Validation reports: [V06](../validations/F-EXT-02/V06-01.md),
  [V13](../validations/F-EXT-02/V13-01.md)

### F-EXT-02-P1-01: Missing descendants bypass symlink confinement

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-tools/src/files/mod.rs:72`,
  `echo-agent/echo-tools/src/files/mod.rs:94`,
  `echo-agent/echo-tools/src/files/mod.rs:115`,
  `echo-agent/echo-tools/src/files/files.rs:674`,
  `echo-agent/echo-tools/src/files/files.rs:1077`
- Reachability: all registered file Tools use resolve_path; write/move then
  create missing parent directories and mutate the returned path.
- Expected invariant: any target under a configured/runtime root remains under
  the canonical root after symlink resolution, including targets not yet born.
- Observed behavior: if target and immediate parent are missing, resolve_path
  returns a text-normalized path without canonicalizing the deepest existing
  ancestor. An existing in-root symlink followed by a missing descendant is
  therefore followed outside the root during create/write/rename.
- Impact: an agent operation can write or move outside its intended workspace,
  causing unintended local data changes.
- Root cause: nonexistent-target validation checks only the immediate parent and
  treats inability to canonicalize as permission to continue.
- Direction: resolve the deepest existing ancestor relative to a canonical root
  and perform mutation handle-relative without following substituted symlinks;
  fail closed when proof is unavailable.
- Regression validation: nested missing descendants through symlinks for every
  mutator, absolute/relative paths, Unicode components, and symlink-swap races.
- Validation reports: [V03](../validations/F-EXT-02/V03-01.md)

### F-EXT-02-P1-02: File mutations are non-atomic and conflict checks are check-then-truncate

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-tools/src/files/files.rs:641`,
  `echo-agent/echo-tools/src/files/files.rs:698`,
  `echo-agent/echo-tools/src/files/edit.rs:143`,
  `echo-agent/echo-tools/src/files/edit.rs:221`,
  `echo-agent/echo-tools/src/files/edit.rs:230`,
  `echo-agent/echo-tools/src/files/files.rs:963`,
  `echo-agent/echo-tools/src/files/files.rs:1071`
- Reachability: write_file, edit_file, update_file, create_file, and move_file
  are all registered in the full Tool set.
- Expected invariant: overwrites commit atomically from a verified revision;
  create/move cannot overwrite a target that appears after validation.
- Observed behavior: checks and reads are separated from direct truncating writes
  or rename; expected_hash is optional and not held/revalidated at commit.
  edit/update independently overwrite after read. Fixed `.bak` copy is not an
  atomic transaction. Existence checks race later create/truncate/rename.
- Impact: concurrent user/tool edits can be silently lost, and crash/write error
  can leave a truncated or partially updated file.
- Root cause: each Tool owns ad hoc mutation sequencing instead of a common
  atomic revisioned filesystem primitive.
- Direction: use same-directory temp + flush/sync + atomic replacement with a
  required revision for read-modify-write, create_new for creation, and
  no-replace rename semantics. Preserve permissions and return exact side-effect
  facts on failure.
- Regression validation: concurrent writer, crash/short-write injection,
  permission preservation, create/move races, and recovery artifact integrity.
- Validation reports: [V04](../validations/F-EXT-02/V04-01.md)

### F-EXT-02-P1-03: Git and worktree Tools block async runtimes without child-process cancellation

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-tools/src/git.rs:421`,
  `echo-agent/echo-tools/src/git.rs:505`,
  `echo-agent/echo-tools/src/git.rs:571`,
  `echo-agent/echo-tools/src/git_worktree.rs:60`,
  `echo-agent/echo-tools/src/worktree_tool.rs:78`
- Reachability: every registered Git implementation ultimately calls run_git;
  registered worktree Tools directly invoke synchronous helpers.
- Expected invariant: external commands run asynchronously under ToolContext
  cancellation/deadline with process-tree reaping and bounded output.
- Observed behavior: `std::process::Command::output` executes synchronously
  inside async Tool futures. Cancellation, timeout, process-group handling, and
  output caps are absent.
- Impact: Git hooks, signing, credential helpers, or large output can block an
  executor thread indefinitely, outlive cancellation, or exhaust memory.
- Root cause: Git/worktree predate and bypass the controlled process primitive
  already used by shell/code paths.
- Direction: route all Git control commands through one noninteractive,
  cancellable, timeout-bounded process service with streamed bounded output and
  typed side effects; remove synchronous helpers from async Tools.
- Regression validation: hanging hook/signing/helper, queued and running cancel,
  timeout race, descendants, huge log/diff, and one terminal event.
- Validation reports: [V07](../validations/F-EXT-02/V07-01.md)

### F-EXT-02-P1-04: Remote sandbox cancellation excludes availability, creation, and cleanup

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-execution/src/sandbox/docker.rs:108`,
  `echo-agent/echo-execution/src/sandbox/docker.rs:318`,
  `echo-agent/echo-execution/src/sandbox/docker.rs:345`,
  `echo-agent/echo-execution/src/sandbox/docker.rs:403`,
  `echo-agent/echo-execution/src/sandbox/k8s.rs:93`,
  `echo-agent/echo-execution/src/sandbox/k8s.rs:136`,
  `echo-agent/echo-execution/src/sandbox/k8s.rs:305`
- Reachability: RunCode passes its token to the selected SandboxExecutor;
  Docker/Kubernetes are public selectable framework adapters.
- Expected invariant: one deadline/token bounds the entire resource lifecycle
  and terminal facts distinguish removed, orphaned, and cleanup-failed resources.
- Observed behavior: only Docker start/Kubernetes attached run is selected
  against timeout/cancel. Availability/create and all cleanup commands are
  unbounded; cleanup status is ignored.
- Impact: cancelled/timed-out code can hang during control operations, leave a
  container/pod, or delay its terminal indefinitely without actionable facts.
- Root cause: control-plane setup/cleanup sit outside the controlled execution
  state machine.
- Direction: budget all phases from one deadline; use cancellation-aware child
  commands; give cleanup its own bounded recovery budget and return resource ID
  plus verified/orphaned state for later cleanup.
- Regression validation: hang/cancel/fail every control phase, verify orphan
  metadata and idempotent later cleanup.
- Validation reports: [V08](../validations/F-EXT-02/V08-01.md)

### F-EXT-02-P1-05: GitBranchTool declares read-only permissions for mutating actions

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-tools/src/git.rs:380`,
  `echo-agent/echo-tools/src/git.rs:389`,
  `echo-agent/echo-tools/src/git.rs:396`,
  `echo-agent/echo-tools/src/git.rs:436`
- Reachability: the full registry exposes GitBranchTool; one schema selects
  list, create, checkout, or delete.
- Expected invariant: declared permission/risk covers every action that can
  execute through the Tool.
- Observed behavior: the Tool declares Read/ReadOnly but can create a branch,
  switch the working tree, or delete a merged branch.
- Impact: framework consumers may route mutating operations through read-only
  policy, telemetry, or approval assumptions.
- Root cause: action-polymorphic behavior is described by one static metadata
  value chosen for only the list variant.
- Direction: split read listing from mutating commands or make risk/permission
  parameter-aware before approval; do not rely on registry placement as a
  substitute for truthful Tool metadata.
- Regression validation: all four variants through the real policy path with
  action-specific permissions and audit facts.
- Validation reports: [V09](../validations/F-EXT-02/V09-01.md)

### F-EXT-02-P2-01: Two registered file-replacement authorities have incompatible safety semantics

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-tools/src/files/files.rs:853`,
  `echo-agent/echo-tools/src/files/files.rs:943`,
  `echo-agent/echo-tools/src/files/edit.rs:27`,
  `echo-agent/echo-tools/src/files/edit.rs:167`,
  `echo-agent/echo-tools/src/registry.rs:231`,
  `echo-agent/echo-tools/src/registry.rs:235`
- Reachability: register_all_tools exposes both update_file and edit_file to the
  same model/runtime.
- Expected invariant: exact-content replacement has one canonical conflict,
  ambiguity, diff, recovery, and result contract.
- Observed behavior: edit_file rejects ambiguous matches, provides dry-run/diff,
  checkpoint and `.bak`; update_file replaces first match with none of those.
  Both still direct-write and neither is conflict-safe.
- Impact: model behavior and recovery depend on which synonym it chooses;
  fixes/tests can land in one path while the other remains unsafe.
- Root cause: a later richer edit Tool was added without deleting or adapting
  the older replacement implementation.
- Direction: make one corrected atomic edit implementation authoritative and
  delete UpdateFileTool registration/type/tests after callers/descriptions
  migrate; do not preserve a second semantic path as an adapter.
- Regression validation: registry has one replacement authority; unique/multiple
  match, dry-run, revision conflict, Unicode, no-op, and result contract.
- Validation reports: [V01](../validations/F-EXT-02/V01-01.md),
  [V04](../validations/F-EXT-02/V04-01.md)

### F-EXT-02-P2-02: Pagination occurs after unbounded input and result materialization

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-tools/src/files/files.rs:445`,
  `echo-agent/echo-tools/src/files/diff.rs:144`,
  `echo-agent/echo-tools/src/files/diff.rs:182`,
  `echo-agent/echo-tools/src/files/grep.rs:218`,
  `echo-agent/echo-tools/src/files/code_search.rs:150`,
  `echo-agent/echo-tools/src/files/code_search.rs:324`,
  `echo-agent/echo-tools/src/files/repo_map.rs:124`,
  `echo-agent/echo-tools/src/git.rs:598`
- Reachability: registered read/diff/search/map/Git tools use these paths before
  PageRequest or result projection.
- Expected invariant: page/output limits bound peak work and memory, while a
  complete artifact/continuation remains available.
- Observed behavior: whole files, diffs, result vectors/maps, traversals, and Git
  output are materialized before pagination. Some individual-file caps exist,
  but no common aggregate work/memory bound does.
- Impact: large ordinary local repositories can cause long stalls, high memory,
  or OOM even when only a small result page is requested.
- Root cause: pagination was applied as final output formatting rather than a
  bounded query/execution contract.
- Direction: stream/short-circuit searches and diffs under aggregate byte/item/
  time budgets, spill full artifacts when needed, and resume from stable query
  identity rather than materializing all results.
- Regression validation: sparse huge files, many small files/matches, huge Git
  output, cancellation, bounded RSS, exact multi-page/artifact completion.
- Validation reports: [V10](../validations/F-EXT-02/V10-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition, duplicate, and layering search | yes | passed | [V01](../validations/F-EXT-02/V01-01.md) |
| V02 | Feature, registration, and runtime reachability | yes | passed | [V02](../validations/F-EXT-02/V02-01.md) |
| V03 | Path and nested-symlink confinement | yes | failed | [V03](../validations/F-EXT-02/V03-01.md) |
| V04 | Atomic/conflict-safe file mutation | yes | failed | [V04](../validations/F-EXT-02/V04-01.md) |
| V05 | Checkpoint/rollback exact-state invariant | yes | failed | [V05](../validations/F-EXT-02/V05-01.md) |
| V06 | Worktree ownership, merge, and partial failure | yes | failed | [V06](../validations/F-EXT-02/V06-01.md) |
| V07 | Git/worktree process cancellation and cleanup | yes | failed | [V07](../validations/F-EXT-02/V07-01.md) |
| V08 | Docker/Kubernetes lifecycle deadline | yes | failed | [V08](../validations/F-EXT-02/V08-01.md) |
| V09 | Git action permission/risk parity | yes | failed | [V09](../validations/F-EXT-02/V09-01.md) |
| V10 | Input/intermediate resource bounds | yes | failed | [V10](../validations/F-EXT-02/V10-01.md) |
| V11 | UTF-8, panic, and overflow static inspection | yes | passed | [V11](../validations/F-EXT-02/V11-01.md) |
| V12 | Existing test coverage inventory | yes | passed | [V12](../validations/F-EXT-02/V12-01.md) |
| V13 | Historical-document drift | yes | failed | [V13](../validations/F-EXT-02/V13-01.md) |
| V14 | Targeted executable regression matrix | conditional | not_run | [V14](../validations/F-EXT-02/V14-01.md) |
| V15 | Integrity gate, malformed link extractor | yes | failed | [V15](../validations/F-EXT-02/V15-01.md) |
| V16 | Corrected exact-link/header/source integrity gate | yes | passed | [V16](../validations/F-EXT-02/V16-01.md) |
| V30 | Primary current-commit source sampling and acceptance | yes | passed | [V30](../validations/F-EXT-02/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| Git checkpoint uses a lightweight tag on HEAD | current | `docs/en/34-git-isolation.md:210`; `git_checkpoint.rs:6` |
| Checkpoint captures exact repository state and is an automatic safety net | stale | [V13](../validations/F-EXT-02/V13-01.md) |
| Rollback restores checkpoint state | stale/misleading | it replaces all tracked paths from HEAD and omits dirty/untracked state; [V05](../validations/F-EXT-02/V05-01.md) |
| Worktree merge and cleanup compose as one successful flow | stale/misleading | failures/partial state in [V06](../validations/F-EXT-02/V06-01.md) |

## Coverage And Uncertainty

- No dynamic validation was executed. Platform-specific Git/filesystem/process
  behavior and peak RSS remain future evidence, recorded in V14.
- F-SEC-01 was unavailable; later synthesis must compare security/policy findings
  and retain this task as owner of concrete tool implementation behavior.
- Docker and Kubernetes adapters were inspected statically without contacting a
  daemon/cluster. Local sandbox and direct shell were sampled as positive
  references, not exhaustively re-reviewed.
- Source transitioned during review. Pre-transition accidental matching lines
  from six then-dirty ReAct files were discarded. All conclusions above were
  rebuilt against clean `3aa7929`; that commit only incorporated those six
  external test-credibility files and did not modify this task's primary paths.
- EKO's product-specific worktree/runtime and interactive command implementations
  were only searched to establish layer ownership; their behavior remains for
  application tasks.

## Handoff

- Primary reviewer should independently sample V03-V10 anchors and preserve
  P0 ownership for exact-state checkpoint and destructive worktree cleanup.
- F-SEC-01 should backlink P0/P1 implementation facts where relevant without
  converting them into cloud-style security gates; the local threat is
  accidental data loss/framework damage.
- F-EXT-01 remains canonical for generic Tool schema/manager cancellation/cursor
  projection. This task owns specialized process propagation and domain behavior.
- Iteration planning should sequence: exact recovery/worktree safety, path-safe
  atomic mutation and duplicate deletion, controlled Git/sandbox lifecycle,
  then resource-bounded read/search/diff.
- Primary current-commit sampling and acceptance are recorded in V30. This report
  becomes stale if `resolve_path`, file mutators, checkpoint/worktree helpers,
  Git child execution, sandbox lifecycle, Tool permission metadata, or their
  registration changes.
