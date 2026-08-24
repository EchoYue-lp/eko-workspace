# A-TSK-05: Worktree, file ownership, and merge policy

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only review)

## Question

Does EKO safely isolate concurrent writers, reuse logical-task worktrees,
protect user changes, and finalize/merge deterministically?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/worktree.rs`
  (2231 lines) — read in full:
  - module doc + branch-name conventions (1-37) — `BRANCH_PREFIX` and
    `FORK_BRANCH_PREFIX` namespaces;
  - process-wide per-repo integration mutex `repo_merge_lock` (39-57);
  - branch-id sanitiser `safe_branch_id` (61-89) and
    `fork_branch_name` (91-93);
  - `WorktreeInfo` parser `parse_worktree_list` (272-355);
  - `RunWorktree::acquire_fork` (381-425) — the reuse/repair entry point;
  - `RunWorktree::create_fork` (431-457) — new-worktree creation;
  - `RunWorktree::diff_summary` / `has_changes` / `unlock` (462-518);
  - `integrate_fork_worktree` (569-620) and `integrate_existing_worktree`
    (622-807) — the merge authority with preflight, dirty-overlap and
    staged-files guards, merge-abort, and `preserve_error` cleanup;
  - `validate_changed_files` ownership enforcement (809-829);
  - `main_dirty_paths` / `reject_active_git_operation` /
    `abort_own_merge` (873-987);
  - `cleanup_managed_worktree` (1001-1017) — finalize-time cleanup;
  - legacy `list_unattended_worktrees` / `merge_unattended_worktree` /
    `discard_unattended_worktree` / `cleanup_unattended_worktrees`
    (1093-1276);
  - `EkoWorktreeFactory` (1290-1364) — the framework
    `WorktreeFactory` impl injected into `AgentConfig`;
  - `EkoDataWorkspaceFactory` (1383-1489) — the framework
    `DataWorkspaceFactory` impl (data/research subagents);
  - the 25-test `tests` module (1491-2231), with particular attention to:
    `fork_acquire_reuses_unlocked_dirty_worktree` (1557),
    `clean_factory_finalize_removes_checkout_and_branch` (1580),
    `dirty_factory_finalize_retains_and_unlocks_checkout` (1604),
    `automatic_cleanup_refuses_new_dirty_content` (1631),
    `automatic_cleanup_retains_new_unique_commit` (1651),
    `ownership_violation_does_not_touch_main_checkout` (1767),
    `conflicting_worktree_fails_without_dirtying_main_index` (1795),
    `local_dirty_owned_path_blocks_integration` (1844),
    `staged_user_change_is_never_captured_by_merge_commit` (1873),
    `repeated_integration_detects_completed_execution` (1939),
    `unattended_cleanup_skips_active_run` (2014),
    `unattended_merge_materializes_and_integrates_orphan_branch` (2067).
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/planner.rs`
  (337 lines) — read in full: `FileOwnership` enum,
  `FileOwnership::conflicts_with`, `normalize_owned_file`,
  `file_ownership`, `analyze_file_ownership`, `has_writer_file_overlap`
  and the 11-test suite.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/profiles.rs`
  (362 lines) — read in full: confirmed it is prompt-template surface
  only, not runtime ownership policy.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs`
  — relevant slices:
  - `task_worktree_label` (196-198) — the label format
    `"{agent_role}-{run_id}:{task_id}"`;
  - `UNATTENDED_DIRECT_MUTATION_TOOLS` (234-254) — `enter_worktree`/
    `exit_worktree` are DISABLED in unattended-Fork mode;
  - `select_ownership_safe_wave` (1127-1145) — wave narrowing;
  - `EkoRuntimeDagController::resolve_dispatch` (1348-1562) — the
    integration trigger and completion CAS;
  - `integrate_reviewed_task` (1686-1752) — integration summary
    persistence;
  - `RealTaskDispatcher::integrate` (906-1028) — the per-repo merge-lock
    acquisition, cancellation checks, and `spawn_blocking` driver for
    `integrate_fork_worktree`;
  - `execute_task` per-file lock acquisition (1995-2052) — the
    sorted-order deadlock-prevention guard;
  - `finalize_cancelled_run_state` (643-661) and the Paused-state
    sweep (546-559);
  - the cancellation test pair `runtime_plan_cancellation_propagates_to_cancelled_outcome`
    (5727) and `runtime_plan_cancellation_preserves_explicit_pause` (5758).
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_shadow.rs`
  (1090 lines, header + write path) — confirmed it is the
  events.jsonl-based persistence store, NOT a file-ownership tracker;
  the per-run write lock protects event-append atomicity, not writer
  isolation.
- Framework contract:
  `echo-agent/src/agent/subagent/worktree.rs` (170 lines) — read in
  full: the `WorktreeFactory` trait, the `WorktreeHandle` with
  `finalize: Box<dyn FnOnce>`, and the hard-fail safety gate when a
  subagent declares `isolate_worktree` but no factory is configured.
- Framework dispatch:
  `echo-agent/src/agent/subagent/executor.rs:1700-1889` — the
  `working_dir` binding, `tokio::select!` cancellation during dispatch,
  and the post-run `finalize` invocation.
- Cross-repo duplicate search (V01) for `acquire_fork`, `create_fork`,
  `integrate_fork_worktree`, `repo_merge_lock`, `file_write_locks`,
  `select_ownership_safe_wave`, `enter_worktree`, `ExitWorktreeTool`,
  `merge_worktree` across the whole `echo-agent-cli` repository.

## Out Of Scope

Deferred to downstream tasks:

- **A-TSK-03** already established the controller/executor boundary
  (the kernel drives waves; EKO injects only product policy). This
  task consumes its conclusion that `integrate_reviewed_task` is the
  single post-review merge site.
- **A-TSK-04** owns claim/revision recovery under concurrent plan
  patches; the worktree-merge site is downstream of the claim CAS.
- **F-EXT-02** (complete) owns the framework-level findings
  (`echo-tools/src/git_worktree.rs::create_worktree` path traversal,
  non-atomic writes, MERGE-state leak in `merge_worktree`). This task
  only answers whether EKO's *application-layer* policy mitigates them.
- **A-TSK-01** owns the file-authority persistence boundary; the
  `file_shadow` inspection here is only to confirm it does not also
  own writer isolation.
- The internal correctness of the framework kernel's cancellation
  drain — F-TSK-03-P2-01 / F-TSK-03-P2-02.

## Inputs

- Required documents read:
  - `AGENTS.md` (root) — rule 6 (single task-relationship authority);
    the framework-vs-application layering gate; the "adapter must stay
    thin" rule; the worktree-merge / parallel-writer expectation
    (B-REF-01); the "防止用户无意中的数据丢失" safety category; the
    YAGNI / delete-over-retain cleanup rule; UTF-8 / panic safety.
  - `docs/comprehensive-review/REPORTING.md`,
    `docs/comprehensive-review/templates/{task-report,validation-report}.md`.
- Dependency task reports read:
  - **A-TSK-03** (complete) — established the executor's
    `RuntimeDagExecutor` boundary, the per-file write locks
    (`file_write_locks` at executor.rs:1154), the file-ownership wave
    narrowing (`select_ownership_safe_wave`), and the orphan
    reconciliation sweeps. This task drills into the worktree-merge
    step that A-TSK-03 only sketched.
  - **F-EXT-02** (complete) — found five framework-level worktree
    defects (P1-01 path traversal, P1-02 non-atomic writes, P2-02
    ignored `working_dir`, P2-03 namesake-branch force-delete, P2-04
    MERGE-state leak on conflict, P2-05 missing `--` separator). Its
    handoff explicitly warns: "A-TSK-05 must not assume the framework's
    worktree tools are safe." This task answers the converse question:
    does EKO's own policy route around them?
  - **F-TSK-03** (complete) — established the framework executor's
    cancellation-grace drain and abort-orphan gap (P2-02). EKO's
    `finalize_cancelled_run_state` mitigates the latter at the
    application layer.
- Historical documents treated as hypotheses: the `worktree.rs` module
  doc (1-22), the `RunWorktree::acquire_fork` doc (376-381), the
  `EkoWorktreeFactory` doc (1280-1289), and the
  `echo-agent/src/agent/subagent/worktree.rs` safety-gate doc
  (20-26) — all verified below.

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | The framework's `WorktreeFactory` trait and `WorktreeHandle` (echo-agent `subagent/worktree.rs:85-107`) are correctly generic: they describe the contract "create an isolated path + a finalize hook" without depending on git. The framework's hard-fail gate when `isolate_worktree` is declared but no factory is configured (`subagent/executor.rs:1609-1620`) is the correct multi-writer data-loss prevention. The framework's per-repo `repo_merge_lock` *equivalent* — the kernel does not own one; this is correctly an application concern. |
| EKO product policy | Confirmed app-owned: the `BRANCH_PREFIX` / `FORK_BRANCH_PREFIX` namespace, the `<repo_parent>/<repo_name>-<sanitized-branch>` path convention, the `eko-unattended-*` legacy cleanup surface, the `merge --no-ff -m "Merge EKO task ..."` trailer convention (`EKO-Execution-Id:`), the dirty-overlap and staged-files guards, the file-ownership `FileOwnership` classification (`ReadOnly | Known | Unknown`), the wave-narrowing `select_ownership_safe_wave`, the per-file `TokioMutex` table (`file_write_locks`), the dirty-tree rejection list (`reject_active_git_operation`), and the legacy `merge_unattended_worktree` / `discard_unattended_worktree` operations. None of these is a generic-framework concern; all assume EKO's "local desktop assistant doing parallel coding/research tasks" product shape. |
| Adapter boundary | `EkoWorktreeFactory` (worktree.rs:1290-1354) is a thin adapter: it implements the framework `WorktreeFactory::create` by delegating to `RunWorktree::acquire_fork` and closes over the `RunWorktree` handle in the `finalize` closure. No framework-owned concept (frontier, retry, cancellation drain) is re-implemented here. `EkoDataWorkspaceFactory` (worktree.rs:1391-1467) is similarly a thin adapter over `tempfile::Builder`. The adapter is lossless: the framework's `WorktreeHandle::finalize: Box<dyn FnOnce -> Result<String, WorktreeError>>` carries exactly the diff summary the application produces. |
| Duplicate search | Searched names (whole `echo-agent-cli` repo): `acquire_fork`, `create_fork`, `integrate_fork_worktree`, `integrate_existing_worktree`, `repo_merge_lock`, `cleanup_managed_worktree`, `select_ownership_safe_wave`, `file_write_locks`, `enter_worktree`, `ExitWorktreeTool`, `merge_worktree`, `WorktreeFactory`, `WorktreeHandle`, `RunWorktree`. Result: ONE `integrate_fork_worktree` (worktree.rs:569); ONE `acquire_fork` (worktree.rs:381); ONE `repo_merge_lock` (worktree.rs:46); ONE `EkoWorktreeFactory` (worktree.rs:1290); ONE `select_ownership_safe_wave` (executor.rs:1127); ONE `file_write_locks` table (executor.rs:1154). The framework's `EnterWorktreeTool` / `ExitWorktreeTool` / `merge_worktree` (the F-EXT-02-broken surface) are referenced ONLY inside `UNATTENDED_DIRECT_MUTATION_TOOLS` (executor.rs:247-248) where they are listed precisely so they can be DISABLED in unattended-Fork mode. ZERO call sites of `enter_worktree` / `exit_worktree` / `merge_worktree` from the framework tool surface reach the EKO task runtime. V01. |
| Migration deletion | No migration proposed. Two cleanup recommendations: A-TSK-05-P3-01 (a stale doc reference) and A-TSK-05-P3-02 (a test-coverage gap). No code deletion. |

## Current Path

Verified EKO writer-isolation data flow at commits
`echo-agent` 9b0e0fa / `echo-agent-cli` b3b2e81:

```text
Plan tasks declared by the model (task_create / task_update)
   │
   ▼
PlanValidator (framework) — structural DAG / cycle check
   │
   ▼
RuntimeDagExecutor::execute (framework kernel)
   │
   ├─ controller.load_snapshot            [executor.rs:1227]
   │     plan_tasks cache populated
   │
   ├─ state.ready_task_ids                [framework]
   │
   ├─ controller.select_ready_wave(ready) [executor.rs:1265]
   │     select_ownership_safe_wave(ready)  [executor.rs:1127-1145]
   │       for each task in ready (kernel-supplied order):
   │         ownership = planner::file_ownership(&task)
   │         ReadOnly        → always included
   │         Known(files)    → included iff no conflict with selected_writers
   │         Unknown{..}     → included iff no other writer is selected
   │                           (Unknown conflicts with every writer)
   │
   ├─ validate_selected_wave              [framework]
   │
   ├─ join_set.spawn {                    [framework]
   │     semaphore.acquire_owned().await  [framework]
   │     controller.claim_task(...)       [executor.rs:1254]
   │     controller.dispatch_task(...)    [executor.rs:1284]
   │       → execute_task(...)            [executor.rs:1843-2509]
   │           │
   │           ├─ acquire write_sem / shell_sem / llm_sem (EkoRuntimeDagController)
   │           │
   │           ├─ per-file TokioMutex acquisition (executor.rs:1995-2052)
   │           │   sorted_files = ownership.known_files().sorted()
   │           │   # sorted to prevent lock-ordering deadlock
   │           │   for f in sorted_files:
   │           │     file_write_locks.entry(f).or_default()
   │           │   for mtx in per_file_mutexes:
   │           │     tokio::select! {
   │           │       cancel.cancelled() → return Err(Cancelled)
   │           │       g = mtx.lock_owned() → guards.push(g)
   │           │     }
   │           │   # Held until dispatch returns (FileLockGuard RAII)
   │           │
   │           ├─ if writer (Implementation | Debugging):
   │           │     run_writer_subagent(...) → framework delegate_to_agent
   │           │       [executor.rs:2889-2978]
   │           │       framework SubagentExecutor (echo-agent
   │           │       subagent/executor.rs:1603-1888):
   │           │         if definition.isolate_worktree && factory.is_none():
   │           │           hard-fail ("no WorktreeFactory configured")
   │           │         factory.create(label)         [echo-agent :1707]
   │           │           → EkoWorktreeFactory::create [worktree.rs:1297]
   │           │             → RunWorktree::acquire_fork(label, repo_root)
   │           │                              [worktree.rs:381-425]
   │           │               1. branch = fork_branch_name(label)
   │           │                  = "eko-fork-" + safe_branch_id(label)
   │           │               2. validate_branch_name (git check-ref-format)
   │           │               3. find existing worktree for branch:
   │           │                  - locked  → Err (active elsewhere)
   │           │                  - unlocked+exists → re-lock, reuse
   │           │                  - pruned  → git worktree prune, fall through
   │           │               4. branch_exists?
   │           │                  default_worktree_path + validate_worktree_target
   │           │                  + git worktree add <path> <branch>
   │           │               5. else create_fork:
   │           │                  default_worktree_path + validate_worktree_target
   │           │                  + git worktree add -b <branch> <path> <base>
   │           │                  + lock_worktree
   │           │         invocation.working_dir = handle.path
   │           │         [echo-agent executor.rs:1748-1751]
   │           │         # subagent's shell/file/git tools now run inside wt
   │           │         run subagent (cancellation-aware via tokio::select!)
   │           │         handle.finalize()    [echo-agent executor.rs:1837]
   │           │           → EkoWorktreeFactory finalize closure [worktree.rs:1324]
   │           │             if !has_changes:
   │           │               cleanup_managed_worktree (remove dir + branch)
   │           │             else:
   │           │               diff_summary + unlock (retain for review)
   │           │
   │           └─ else (read-only / verification):
   │                 run_readonly_subagent OR run_main_agent_task
   │                 (no worktree, no file lock)
   │   }
   │
   ├─ join_set drain + cancellation grace [framework]
   │
   └─ controller.resolve_dispatch(...)    [executor.rs:1348-1562]
        │
        ├─ CompletionAssessment::Executed + ReviewGate::Pass:
        │     integrate_reviewed_task(...) [executor.rs:1522, 1686-1752]
        │       → RealTaskDispatcher::integrate [executor.rs:906-1028]
        │         1. git_repo_root(working_dir)
        │         2. merge_lock = repo_merge_lock(repo_root)  [process-wide]
        │         3. tokio::select! {
        │              cancel.cancelled() → Err("cancelled while waiting")
        │              g = merge_lock.lock_owned() → g
        │            }
        │         4. if cancel.is_cancelled() → Err (second check)
        │         5. spawn_blocking { integrate_fork_worktree(...) }
        │              [worktree.rs:569-620 → integrate_existing_worktree 622-807]
        │                a. find_integration_commit(trailer) → short-circuit
        │                   (AlreadyIntegrated, idempotent retries)
        │                b. git add -A in worktree
        │                c. merge-base HEAD branch
        │                d. changed_files = git diff --cached --name-only -z
        │                e. validate_changed_files(ownership, changed_files)
        │                   → Err if writer touched files outside its declared
        │                     ownership
        │                f. if staged-against-HEAD: commit writer changes with
        │                   "EKO-Execution-Id: <id>" trailer
        │                g. if changed_files empty → NoChanges + cleanup
        │                h. if ancestor(branch, HEAD) → AlreadyIntegrated
        │                i. reject_active_git_operation (MERGE_HEAD, rebase, etc.)
        │                j. staged-files check (refuse if main index not clean)
        │                k. dirty-overlap check (refuse if main has uncommitted
        │                   changes in writer-owned paths)
        │                l. merge-tree --write-tree --name-only --messages
        │                   HEAD branch    [NON-MUTATING PREFLIGHT]
        │                   exit 1 → Err("merge conflict"), no mutation
        │                m. git -c commit.gpgsign=false merge --no-ff --no-edit
        │                   -m "Merge EKO task ...\n\nEKO-Execution-Id: ..." branch
        │                   !success → git merge --abort, then Err
        │                n. cleanup_managed_worktree (remove now-merged wt+branch)
        │         6. outcome persisted to summary store + RuntimeEventKind::Merge*
        │
        └─ on Err: task → Failed; on Ok: task → Completed
```

Invariants verified by this graph (full evidence in V01-V04):

- **Single ownership authority.** `planner::FileOwnership` is the only
  classification. `select_ownership_safe_wave` (executor.rs:1127) is
  the only wave-narrowing site. The per-file `file_write_locks` table
  (executor.rs:1154, 2020-2029) is the only physical mutex. There is
  no second writer-isolation mechanism in EKO. V01.
- **Single worktree integration authority.** `integrate_fork_worktree`
  (worktree.rs:569) is the only production merge site;
  `integrate_existing_worktree` (worktree.rs:622) is its single
  helper. The legacy `merge_unattended_worktree` routes through the
  same `integrate_existing_worktree`. ZERO call sites of the
  framework's broken `merge_worktree` / `enter_worktree` /
  `exit_worktree` surface in EKO's task runtime. V01.
- **Deterministic, sanitised paths.** Every worktree path is computed
  by `default_worktree_path` (worktree.rs:218) from a
  `safe_branch_id`-sanitised label, and verified by
  `validate_worktree_target` (worktree.rs:245) which canonicalises
  both the repo parent and the target parent and rejects any target
  whose parent is not under the repo parent. V01, V02.
- **Dirty-tree protection is layered.** Three independent guards
  (reject_active_git_operation, staged-files check, dirty-overlap
  check) run after the preflight but before the mutating merge. V02.
- **Reuse preserves uncommitted retry work.** A retained, unlocked
  worktree for the same logical task is re-locked and reused across
  retries; the test `fork_acquire_reuses_unlocked_dirty_worktree`
  proves the previous attempt's `retry.txt` survives. V03.
- **Cleanup is conservative.** `cleanup_managed_worktree` removes the
  directory only if `git worktree remove` succeeds, and removes the
  branch only if `branch_ahead_of_head == 0`. Late-arriving dirty
  content or unique commits abort the cleanup and retain the
  worktree. V03.
- **Merge failure never leaves the main checkout dirty.** The
  `conflicting_worktree_fails_without_dirtying_main_index` test
  asserts an empty `git status --porcelain` after a preflight
  conflict; the `preserve_error` helper unlocks the failed worktree
  and embeds its path/branch in the error message so it can be
  reviewed or retried. V04.
- **Cancellation mid-merge runs to completion.** The
  `spawn_blocking` at executor.rs:961 is uninterruptible; this is
  correct (a half-finished `git merge` would leave `MERGE_HEAD` set
  and conflict markers in working files). The cancel token is
  checked twice before the spawn (select on lock acquire +
  explicit `is_cancelled()` check) so cancel-before-merge-start
  aborts cleanly with no side effects. V04.

## Findings

The headline result is strongly positive: EKO's application-layer
worktree policy is well-defended and **fully mitigates** the
framework-level defects F-EXT-02 found (P1-01 path traversal, P2-04
MERGE-state leak). The framework's broken `enter_worktree` /
`exit_worktree` / `merge_worktree` surface is not reachable from
EKO's task runtime: those tool names are listed in
`UNATTENDED_DIRECT_MUTATION_TOOLS` (executor.rs:247-248) precisely so
they can be disabled in unattended-Fork mode, and the production
merge authority is `integrate_fork_worktree` instead. Three minor
(P3) cleanup items are recorded; no P0/P1/P2 issues found.

### A-TSK-05-P3-01: `worktree.rs` module doc still references the removed `panels.rs` shared surface

- Priority: P3
- Confidence: high
- Layer: application (documentation)
- Evidence: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/worktree.rs:3-9`
  — the module doc says "centralises the git-worktree operations that
  were previously scattered across Tauri `panels.rs` commands, so both
  the CLI and the GUI can share them." `panels.rs` no longer exists in
  `echo-agent-cli`; the GUI/Tauri commands live under
  `src/tauri/commands/task_runtime.rs` (A-TSK-03 V01). The reference
  is historical and slightly misleading for a new reader.
- Reachability: documentation only; no behaviour impact.
- Expected invariant: module docs describe the current architecture,
  not a stale migration narrative.
- Observed behavior: the doc reads as if `panels.rs` is still the
  alternative surface.
- Impact: very low; readability / onboarding only.
- Root cause: the doc was written during the original extraction and
  was not updated when the Tauri command layout was reorganised.
- Direction: rewrite the module doc to drop the `panels.rs` reference
  and describe the current sharing pattern (app-core consumed by both
  CLI `task_execute` and Tauri `task_runtime` commands).
- Regression validation: doc-only; no test.
- Validation reports: [V01-01](../validations/A-TSK-05/V01-01.md)

### A-TSK-05-P3-02: No executor-level test exercises cancellation arriving during `spawn_blocking { integrate_fork_worktree }`

- Priority: P3
- Confidence: medium
- Layer: application (test coverage)
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:961-971`
    — the production cancellation-during-merge site. `spawn_blocking`
    is uninterruptible; the result is `.await`ed unconditionally.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:5375-5755`
    — the cancellation tests. All three
    (`cancellation_preserves_completed_tasks_and_finalizes_the_run`,
    `runtime_plan_cancellation_propagates_to_cancelled_outcome`,
    `runtime_plan_cancellation_preserves_explicit_pause`) use
    `ScriptedDispatcher`, whose `integrate` is a stub. None drives
    cancellation through the real `RealTaskDispatcher::integrate`
    path.
  - The worktree-level tests in `worktree.rs::tests` exercise merge
    conflicts and dirty-tree rejection directly, but never cancel a
    merge in flight (they are blocking, not async).
- Reachability: any production cancellation that lands in the
  microsecond window between `merge_lock.lock_owned()` returning and
  `spawn_blocking` completing. Realistically reachable only on a
  user-initiated `cancel_run` or a parent-process shutdown while a
  writer is being integrated.
- Expected invariant: the cancellation-during-merge path is exercised
  by at least one test, asserting (a) the merge runs to completion
  (because `spawn_blocking` is uninterruptible), (b) the result is
  applied correctly (Completed if merge succeeded, even if cancel
  arrived), and (c) the run terminates cleanly (Cancelled outcome
  via the kernel's drain).
- Observed behavior: no test covers this; the property is enforced
  only by code inspection.
- Impact: low for correctness (the design is sound — see Current
  Path), but the test gap means a future refactor could accidentally
  break the "merge runs to completion" invariant without any test
  catching it.
- Root cause: the executor test suite was built around
  `ScriptedDispatcher` for determinism; no `RealTaskDispatcher`-grade
  fixture exists that exercises a real git merge under cancellation.
- Direction: add a `tokio::test` that (1) creates a real temp git
  repo, (2) dispatches a writer that produces a mergeable change, (3)
  cancels the run after `MergeStarted` is emitted, (4) asserts the
  merge commit is present (or absent, if the cancel beat the
  spawn_blocking start) and the run terminates within a bounded time.
  This may require a small test-only hook to deterministically win
  the race.
- Regression validation: the new test itself.
- Validation reports: [V04-01](../validations/A-TSK-05/V04-01.md)

### A-TSK-05-P3-03: `acquire_fork` reuses an existing worktree path without re-running `validate_worktree_target`

- Priority: P3
- Confidence: medium
- Layer: application (defense-in-depth)
- Evidence: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/worktree.rs:385-401`
  — when `find_worktree_info` returns an existing unlocked worktree
  whose path exists, the code reuses `PathBuf::from(existing.path)`
  directly. The `validate_worktree_target` confinement check is only
  run on the *create* paths (worktree.rs:408, 435, 611, 1183).
- Reachability: requires an attacker to first create a git worktree
  with branch name `eko-fork-<safe_branch_id(label)>` pointing at an
  escaped location, then trigger EKO's `acquire_fork` for that label.
  Because the label is built from internal IDs
  (`{agent_role}-{run_id}:{task_id}`) and `safe_branch_id` appends a
  12-char sha256 digest, an attacker would need to control the
  `run_id` / `task_id` and predict the resulting branch name, AND
  have write access to the user's git repo. At that point the
  attacker already owns the user's machine.
- Expected invariant: every worktree path EKO uses is verified to
  live under the repo parent, regardless of how it was discovered.
- Observed behavior: the reuse path trusts `git worktree list
  --porcelain` output. If a worktree with the right branch somehow
  exists at an out-of-tree location, the reuse would honour that
  location.
- Impact: very low. The attack requires prior write access to the
  repo, which is outside EKO's threat model (local personal
  assistant; if the attacker can write to the repo, they can already
  replace files directly). The create paths are all validated, so
  this is purely a defense-in-depth observation.
- Root cause: the reuse path predates the explicit
  `validate_worktree_target` helper and was written to trust git's
  own list.
- Direction: optionally, re-run `validate_worktree_target` on the
  reused path before returning, OR add a one-line comment documenting
  why the reuse path is trusted (the branch was created by EKO itself
  via the validated create path, so its worktree location is
  transitively trusted). The comment option is cheaper and matches
  the threat model.
- Regression validation: existing `fork_acquire_reuses_unlocked_dirty_worktree`
  test still passes.
- Validation reports: [V03-01](../validations/A-TSK-05/V03-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Ownership-conflict detection: plan-time wave narrowing + dispatch-time per-file locks; F-EXT-02-P1-01 path traversal is mitigated by `safe_branch_id` + `default_worktree_path` + `validate_worktree_target`; framework worktree tools not reachable from EKO task runtime | yes | passed | [V01-01](../validations/A-TSK-05/V01-01.md) |
| V02 | Dirty-tree protection: `reject_active_git_operation` + staged-files guard + dirty-overlap guard; preflight via `merge-tree --write-tree`; tests `local_dirty_owned_path_blocks_integration`, `staged_user_change_is_never_captured_by_merge_commit`, `conflicting_worktree_fails_without_dirtying_main_index` | yes | passed | [V02-01](../validations/A-TSK-05/V02-01.md) |
| V03 | Reuse/repair/cleanup: `acquire_fork` reuses unlocked worktrees; pruned worktrees are materialised; cleanup refuses dirty content and unique-commit branches; tests `fork_acquire_reuses_unlocked_dirty_worktree`, `automatic_cleanup_*`, `unattended_cleanup_skips_active_run` | yes | passed | [V03-01](../validations/A-TSK-05/V03-01.md) |
| V04 | Merge failure and cancellation: preflight catches conflicts without mutating; failed merge triggers `git merge --abort`; cancellation before merge is checked twice; mid-merge cancel runs to completion (correct); tests `conflicting_worktree_fails_without_dirtying_main_index`, `repeated_integration_detects_completed_execution`; coverage gap on mid-merge cancel (P3-02) | yes | passed | [V04-01](../validations/A-TSK-05/V04-01.md) |
| V05 | Historical-document drift | conditional (applicable — three code/module comments treated as hypotheses; classifications inline in Historical Claim Status) | passed | classified inline |

Executed cargo commands (all exit 0):

```text
cd echo-agent-cli && cargo test -p echo-agent-app-core --lib task_runtime::worktree::tests
  → 25 passed; 0 failed; 0 ignored; 0 measured; 624 filtered out (1.13s)

cd echo-agent-cli && cargo test -p echo-agent-app-core --lib task_runtime::planner::tests
  → 11 passed; 0 failed; 0 ignored; 0 measured; 638 filtered out (0.00s)
```

The full `echo-agent-cli` pre-commit gate was not re-run because this
review is read-only; the targeted worktree + planner subsets are the
directly relevant evidence and are the suites that exercise the
ownership and merge invariants audited here.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `worktree.rs:1-9` module doc: "centralises the git-worktree operations that were previously scattered across Tauri `panels.rs` commands, so both the CLI and the GUI can share them." | stale (cosmetic) | `panels.rs` no longer exists; the GUI surface lives in `src/tauri/commands/task_runtime.rs`. The *intent* (CLI + GUI share the same operations) is still true; only the historical reference is stale. See A-TSK-05-P3-01. |
| `worktree.rs:5-9` "It is product-layer (app-core) because worktree isolation is an EKO desktop-assistant concern, not a generic agent-framework one — the framework only exposes the `working_dir` propagation that lets shell/file/git tools chroot themselves into a worktree path." | current (corroborated and refined) | V01 confirms the framework's `WorktreeFactory` trait (echo-agent `subagent/worktree.rs:85-91`) is the contract; EKO supplies `EkoWorktreeFactory` (worktree.rs:1297). The framework also owns the `isolate_worktree` hard-fail gate (`subagent/executor.rs:1609-1620`), which is correctly generic (multi-writer data-loss prevention). The doc's claim that the framework only exposes `working_dir` propagation understates the framework's role: it also owns the trait + safety gate. |
| `worktree.rs:1280-1289` `EkoWorktreeFactory` doc: "Stored behind an `Arc` and injected into the framework's `AgentConfig.subagent_worktree_factory`, so the framework can create worktrees for subagents declaring `isolate_worktree: true` without the framework itself depending on git or the application." | current | V01 confirms: the framework `SubagentExecutor` invokes `factory.create(label)` at `subagent/executor.rs:1707` and binds `invocation.working_dir = handle.path` at `subagent/executor.rs:1748-1751`. |
| `worktree.rs:41-45` `repo_merge_lock` doc: "The lock protects the shared Git index/refs when different TaskRuntime runs finish isolated writers at the same time. Cross-process races still fail through Git's own lock files and are surfaced as integration failures." | current | V04 confirms: `RealTaskDispatcher::integrate` acquires `repo_merge_lock` at executor.rs:925-930 before invoking `integrate_fork_worktree`. The lock is process-wide; cross-process races fall back to git's `.git/index.lock` and `.git/MERGE_HEAD`. |
| `planner.rs:1-9` module doc: "`file_ownership` / `analyze_file_ownership` provide deterministic ownership classification for writer tasks. Exact workspace-relative files may run in parallel when disjoint; broad or invalid scopes are `Unknown` and must serialize with every writer." | current | V01 confirms: `select_ownership_safe_wave` (executor.rs:1127-1145) consumes `FileOwnership::conflicts_with`; `Unknown` conflicts with every writer. |
| F-EXT-02 handoff: "Worktree isolation is *not* safe in its current form. A-TSK-05 (worktree/file-ownership policy) must not assume the framework's worktree tools are correct — they need to be fixed first (P1-01) or worked around at the application layer." | current (worked around at application layer) | V01 confirms EKO does not route through the framework's `EnterWorktreeTool` / `ExitWorktreeTool` / `merge_worktree`; those tool names are listed in `UNATTENDED_DIRECT_MUTATION_TOOLS` (executor.rs:247-248) precisely so they can be disabled. The application-layer `integrate_fork_worktree` is the merge authority and is well-defended. The framework tools remain broken for any other consumer (F-EXT-02 findings stand). |
| F-EXT-02-P2-04 "merge_worktree leaves the repo in MERGE state on conflict and disrupts concurrent workers" | mitigated for EKO | V04 confirms `integrate_existing_worktree` (worktree.rs:728-784) does a non-mutating `merge-tree --write-tree` preflight and calls `git merge --abort` on actual merge failure. The framework's `merge_worktree` is not used. |
| A-TSK-03 handoff: "F-TSK-03-P2-02's orphan-claim hazard is mitigated at the EKO application layer: `finalize_cancelled_run_state` reconciles all non-terminal tasks to `Cancelled`." | current (relevant to V04) | V04 confirms: a mid-merge cancel that completes the merge will leave the task `Completed` (the CAS writes through); `finalize_cancelled_run_state` only touches `Pending | Running | Blocked` tasks, so the completed-and-merged task is not reverted. |

## Coverage And Uncertainty

- **Inspected in full:** the entire `worktree.rs` (2231 lines), the
  entire `planner.rs` (337 lines), the entire `profiles.rs` (362
  lines), the framework `subagent/worktree.rs` (170 lines), and the
  relevant slices of `subagent/executor.rs:1600-1890`. The
  `executor.rs` slices covering ownership/lock/integration/cancellation
  were read in full where they bear on the worktree policy
  (lines 192-270 for the disabled-tools list and label format;
  643-661 for the cancelled sweep; 906-1030 for the dispatcher
  integrate path; 1127-1145 for wave selection; 1265-1562 for
  dispatch/resolve; 1686-1752 for integrate_reviewed_task;
  1843-2052 for execute_task's lock acquisition; 5375-5825 for the
  cancellation/pause tests).
- **Inspected partially:** `file_shadow.rs` was read at the header
  and write-path level only (lines 1-150). It is the events.jsonl
  persistence store; its per-run write lock protects event-append
  atomicity, not writer isolation. Its internals are A-TSK-01 / A-TSK-04
  territory.
- **Not inspected (out of scope):**
  - `executor.rs` slices beyond the worktree/ownership/dispatch
    surface (the review-gate LLM wiring, the per-task prompt
    assembly, the unattended-vs-attended mode branching outside the
    disabled-tools list). These are A-TSK-02 / A-TSK-03 territory.
  - The full `echo-agent-cli` pre-commit matrix (fmt / clippy /
    all-features test). The review is read-only; the targeted
    worktree + planner subsets are the directly relevant evidence
    (36 tests pass).
- **Uncertain claims:**
  - The exact behaviour of `git merge-tree --write-tree
    --name-only --messages HEAD <branch>` across git versions. The
    code assumes exit code 1 means "conflict detected, no tree
    written"; this is correct for git >= 2.38 (where `--write-tree`
    was stabilised). Older git versions do not support this subcommand
    and would fail at the `git_output` level with a non-{0,1} status,
    routing to the "merge preflight failed" branch. This is acceptable
    but undocumented; if the project's minimum git version matters, it
    should be recorded.
  - Whether the cross-process race window noted in
    `repo_merge_lock`'s doc (line 45) is ever actually exercised in
    practice. EKO is a single-process application; the only way two
    EKO processes would race on the same repo is if the user runs two
    EKO instances simultaneously. Git's own `.git/index.lock` serialises
    them, and the loser surfaces as an integration failure (which
    preserves the worktree for retry per `preserve_error`).

## Handoff

- **Conclusions downstream tasks may rely on:**
  - EKO's writer isolation has a single ownership authority
    (`planner::FileOwnership`), a single wave-narrowing site
    (`select_ownership_safe_wave`), a single per-file mutex table
    (`file_write_locks`), a single worktree-integration authority
    (`integrate_fork_worktree`), and a single per-repo serialization
    lock (`repo_merge_lock`). There is no duplicate authority. (V01)
  - F-EXT-02's framework-level defects (P1-01 path traversal, P2-04
    MERGE-state leak) are **fully mitigated for EKO at the application
    layer**: the broken framework tools are not reachable from EKO's
    task runtime (they are listed in `UNATTENDED_DIRECT_MUTATION_TOOLS`
    to be disabled), and EKO's `integrate_fork_worktree` does its own
    validated path computation + preflight + merge-abort. The
    framework tools remain broken for any other consumer. (V01, V04)
  - Dirty-tree protection is layered and tested: an active git
    operation, ANY staged file, or ANY uncommitted change in a
    writer-owned path blocks the integration. The main checkout
    cannot be dirtied by a failed merge. (V02)
  - Worktree reuse across retries is sound: a retained, unlocked
    worktree for the same logical task is re-locked and reused; dirty
    content and unique commits survive cleanup. (V03)
  - Cancellation mid-merge runs to completion by design
    (`spawn_blocking` is uninterruptible); cancellation before the
    merge starts is checked twice and aborts cleanly. The
    cancellation-during-merge test gap is filed as A-TSK-05-P3-02
    (test coverage, not correctness). (V04)
  - The 25-test worktree suite and the 11-test planner suite pass;
    ownership enforcement, dirty-tree rejection, conflict preflight,
    cleanup conservatism, and reuse semantics are all covered. (V04)
- **Reports downstream tasks must read:**
  - [V01-01](../validations/A-TSK-05/V01-01.md) for the ownership
    matrix, path-traversal mitigation, and the no-duplicate-authority
    confirmation.
  - [V02-01](../validations/A-TSK-05/V02-01.md) for the dirty-tree
    protection evidence and the staged-files / dirty-overlap test
    mapping.
  - [V03-01](../validations/A-TSK-05/V03-01.md) for the reuse /
    repair / cleanup evidence and the conservation invariants.
  - [V04-01](../validations/A-TSK-05/V04-01.md) for the merge-failure
    and cancellation evidence, including the mid-merge cancel test
    gap.
- **Task-to-reference mapping:**
  - A-TSK-04 (claims/revisions/recovery/terminal monotonicity) → may
    rely on the merge site being downstream of the claim CAS
    (`set_claimed_status` runs after `integrate_reviewed_task`
    returns); should verify that a Completed task that the run later
    cancels (via `finalize_cancelled_run_state`) does not lose its
    already-merged work. The current code is correct
    (`finalize_cancelled_run_state` skips Completed tasks), but
    A-TSK-04 should add a regression test if not already present.
  - A-TSK-01 (file authority) → may rely on `file_shadow` being
    orthogonal to writer isolation (it is; `file_shadow` is the
    event store, `file_write_locks` is the writer mutex).
  - Q-FLT-01 / Q-FLT-02 (filter / lint tasks) → should add
    crash-mid-write, concurrent-write race, and mid-merge-cancel
    fixtures as called out in V04 follow-up.
- **Conditions that make this report stale:**
  - Any commit that adds a second `integrate_*` authority, a second
    `file_write_locks`-style table, or a second `select_*_wave`
    function invalidates V01.
  - Any commit that re-enables `enter_worktree` / `exit_worktree` /
    `merge_worktree` in EKO's task runtime (e.g. removing them from
    `UNATTENDED_DIRECT_MUTATION_TOOLS`) invalidates the F-EXT-02
    mitigation claim in V01.
  - Any change to `validate_worktree_target`, `safe_branch_id`, or
    `default_worktree_path` invalidates the path-traversal
    mitigation claim in V01.
  - Any change to `integrate_existing_worktree`'s preflight /
    merge-abort / dirty-tree guards invalidates V02 and V04.
  - Any change to `acquire_fork`'s reuse logic or
    `cleanup_managed_worktree`'s ahead-check invalidates V03.
- **Follow-up task IDs (no fixes implemented in this review):**
  - A documentation cleanup task should pick up A-TSK-05-P3-01
    (stale `panels.rs` reference in the module doc). One-line edit.
  - A test-coverage task should pick up A-TSK-05-P3-02 (mid-merge
    cancellation test). Requires a real-git fixture in the executor
    test suite.
  - A defense-in-depth task should optionally pick up A-TSK-05-P3-03
    (re-validate reused worktree paths). The comment-option is the
    cheap path and matches the threat model.
  - The framework-level F-EXT-02 follow-ups (P1-01, P1-02, P2-02,
    P2-03, P2-04, P2-05) remain the higher-priority items for the
    framework track; they are NOT blocking for EKO because EKO routes
    around them at the application layer (this task's V01).
