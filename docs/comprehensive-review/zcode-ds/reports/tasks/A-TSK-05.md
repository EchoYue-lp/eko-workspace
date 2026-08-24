# A-TSK-05: Worktree, file ownership, and merge policy

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: `echo-agent` has pre-existing untracked `check_*` scratch files; `echo-agent-cli` has modified `web-frontend/src/generated/*.ts` (ts-rs codegen side effect of cargo test, pre-existing pattern). No review-written source changes.

## Question

Does EKO safely isolate concurrent writers, reuse logical-task worktrees, protect user changes, and finalize/merge deterministically?

**Answer: Yes for the core pipeline — concurrent writers are isolated per logical task (`eko-fork-*` worktrees), logical-task worktrees are reused across retries/fix tasks, user changes in the main checkout are protected at integration (staged-index refusal, dirty-overlap refusal, merge-tree preflight, failure-preserving abort), and finalize/merge is deterministic (repo-level merge lock, execution-id trailer idempotency, conflict → preserve-and-Fail). Four gaps remain, all outside the happy-path merge logic: (P2) a process crash leaves the fork worktree's `git worktree lock` in place with no repair path — the logical task is permanently blocked; (P2) there is no lifecycle sweep for leaked `eko-fork-*` worktrees/branches (only `eko-unattended-*` has cleanup); (P2) writer isolation is enforced only by the routed Subagent definition — a writer task routed to a non-isolated role (builtin `general-purpose`, or any user role without `worktree: true`) silently runs in the main checkout and integration reports `no_changes`, bypassing the ownership/dirty-tree/merge protections; (P2) the legacy GUI worktree helpers in `src/tauri/commands/panels.rs` duplicate the app-core authority and have already diverged (locked state unparsed). Plus two P3 items (comment-vs-behavior drift on non-git workspaces, blocking git subprocesses on the async runtime). No P0/P1 data-loss vector was found in the isolated-writer pipeline.**

## Scope

Primary source paths inspected (deep read):

- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/worktree.rs` (full, 2231 lines): branch identity (`safe_branch_id`/`fork_branch_name`), `RunWorktree::acquire_fork/create_fork` reuse/lock semantics (:375-457), `diff_summary`/`has_changes` (:462-499), `integrate_fork_worktree` + `integrate_existing_worktree` (:569-807), ownership validation (:809-829), dirty-tree/staged-index/preflight checks (:705-754), `cleanup_managed_worktree` (:1001-1017), unattended list/merge/discard/cleanup (:1093-1276), `EkoWorktreeFactory`/`EkoDataWorkspaceFactory` (:1291-1489), full test module (:1491-2231).
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/planner.rs` (full): `FileOwnership` classification, `normalize_owned_file`, `analyze_file_ownership`, tests.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_shadow.rs` (full): per-run write locks, unique seq, atomic snapshot writes, tests.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs` (sections): `task_worktree_label` (:196-198), `UNATTENDED_DIRECT_MUTATION_TOOLS` + prompts (:227-293), `TaskDispatcher`/`RealTaskDispatcher::integrate` (:784-1030), `select_ownership_safe_wave` (:1127-1145), review-then-integrate in `resolve_dispatch` (:1457-1560), `integrate_reviewed_task` (:1686-1752), `execute_task` ownership/file-lock/dispatch (:1843-2508), `run_writer_subagent` + isolation context (:2889-2978), unattended preflight (:3936-4010), ownership wave tests (:4156-4185).
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs` (:490-623 run/task cancel tokens, `is_run_active`; boot recovery cross-referenced at :1631-1776 via A-TSK-03).
- `echo-agent-cli/echo-agent-app-core/src/infra.rs` (:387-426 worktree/data-workspace factory injection).
- `echo-agent-cli/echo-agent-app-core/src/subagents/coding/implementer.md`, `general-purpose.md`, `data/analyst.md` (isolation frontmatter); `src/subagent_loader.rs` (:440-460 isolation flag derivation).
- `echo-agent/src/agent/subagent/worktree.rs` (full): framework trait, `NoWorktreeFactory`.
- `echo-agent/src/agent/subagent/executor.rs` (:1590-1888): isolation resolution, hard-fail guard, factory create, finalize on all exit paths.
- `echo-agent/src/agent/config.rs` (:80-81, 413-417), `echo-agent/src/agent/react/mod.rs` (:428) factory propagation.
- `echo-agent-cli/src/tauri/commands/panels.rs` (:1828-2163) + `src/tauri/mod.rs` (:295-301) + `src/tui/events.rs` (:2300-2410): GUI/TUI surfaces.
- `echo-agent-cli/web-frontend/src/api/endpoints.ts` (:1972-2001): frontend consumers.
- Docs: `echo-agent-cli/docs/MASTER-PLAN.md` (:59-60, :122-131, :459-465), `docs/2026-07-22-unattended-worktree-lifecycle.md`, `docs/2026-07-25-logical-task-worktree-reuse.md`.

## Out Of Scope

- File-authority round-trip, plan.json/events.jsonl authority, torn-tail handling — A-TSK-01 (its P1-01 is consumed here only as a store-fault cross-reference; the file shadow's in-process concurrency safety is reviewed here, its authority semantics there).
- Claims/revisions/recovery state machine, terminal monotonicity, event replay — A-TSK-04.
- Executor boundary outcome handling (pause-during-wave, orphan claims, per-task cancel) — A-TSK-03.
- Framework `echo-tools` worktree tools (`enter_worktree`/`exit_worktree`, `git_worktree.rs`) — F-EXT-02 (their findings P2-02/P2-03 are cross-referenced, not re-reviewed).
- Frontend rendering of worktree panels beyond endpoint consumption — A-FE-01/02, A-SRF-03.
- Project diff/index services — A-PROJ-01 (depends on this task's conclusions).

## Inputs

- Root `AGENTS.md` (worktree development rules; UTF-8/panic safety; framework-vs-app layering; "EKO worktree mechanism is application product policy, git-operation correctness is generic"; data-loss P1 criterion for worktrees).
- Shared `README.md`, `REPORTING.md`, `TASKS.md` (A-TSK-05 card), `zcode-ds/README.md`, templates.
- Dependency task reports read: `A-TSK-03` (complete — controller boundary, review-then-integrate flow, per-task cancel dead code, write semaphores per run) and `F-EXT-02` (complete — framework worktree tools, dirty-tree/conflict handling of the tool layer, P2-02/P2-03 containment findings).
- Historical documents treated as hypotheses: `MASTER-PLAN.md` worktree rows, `2026-07-22-unattended-worktree-lifecycle.md`, `2026-07-25-logical-task-worktree-reuse.md` (classified in V05-01).

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism (framework, correctly placed) | `WorktreeFactory`/`WorktreeHandle`/`NoWorktreeFactory` + `DataWorkspaceFactory` traits (`echo-agent/src/agent/subagent/worktree.rs`, `workspace.rs`) and the `ExternalRunContext.isolation_id` transport (framework executor.rs:1655-1670). The framework hard-fail when isolation is declared but no factory exists (executor.rs:1609-1619) is a generic multi-writer data-loss guard, correctly generic. One impl per trait, both in the app. |
| EKO product policy (application) | `RunWorktree` lifecycle and `eko-fork-`/`eko-unattended-` branch policy, ownership wave filter and per-file locks (`planner.rs`, `executor.rs:1127-1145, 2003-2052`), the merge/integration boundary incl. dirty-tree protection and execution-id idempotency (`worktree.rs:569-1017`), `repo_merge_lock`, unattended management, `UnattendedWriteMode`, `EkoWorktreeFactory`/`EkoDataWorkspaceFactory` — all correctly placed in app-core. The file shadow's concurrency machinery (`file_shadow.rs`) is the app's persistence layer. |
| Adapter boundary | `EkoWorktreeFactory::create`/`finalize` map the framework contract to `RunWorktree` (thin, lossless); `RealTaskDispatcher::integrate` (executor.rs:889-1029) is the app-side merge adapter (lock acquisition + spawn_blocking + outcome events); `integrate_fork_worktree`'s trailer-based dedup is the boundary's idempotency key. Label identity (framework `"{role}-{isolation_id}"` vs EKO `task_worktree_label`) matches (V02-01). |
| Duplicate search | Terms (both repos, V01-01): `worktree`, `work_tree`, `WorktreeFactory`, `DataWorkspaceFactory`, `fork_branch_name`, `FORK_BRANCH_PREFIX`, `BRANCH_PREFIX`, `default_worktree_path`, `parse_worktree_list`, `validate_worktree_target`, `validate_branch_name`, `repo_merge_lock`, `FileOwnership`, `analyze_file_ownership`, `has_writer_file_overlap`, `FileTaskShadow`, `shadow`, `isolation_id`, `isolate_worktree`, `isolate_workspace`, `cleanup_unattended_worktrees`, `merge_unattended_worktree`, `UnattendedWriteMode`, `enter_worktree`, `exit_worktree`. Result: one live EKO authority per concept; `panels.rs` legacy GUI helpers are a live duplicate (P2-04); framework `echo-tools` worktree tools are a distinct model-facing surface (F-EXT-02), not used by the TaskRuntime; `has_writer_file_overlap` has zero production callers. |
| Migration deletion | When P2-04 is fixed, delete `panels.rs:1828-1909` helpers (parser, default path, target validation, branch validation) and route `list_worktrees`/`create_worktree`/`remove_worktree` through the shared module; when P2-03 is fixed, either hard-fail non-isolated writer routing (consistent with the framework guard) or report an explicit `in_place` integration status instead of `no_changes`. |

## Current Path

Verified call graph (V01-01/V02-01; detailed in those reports):

1. **Dispatch**: `task_execute`/resume → `execute_run` → `execute_runtime_plan` (executor.rs:1622-1683) builds `EkoRuntimeDagController` with per-run write/shell/llm semaphores (:1637-1639). Writer tasks (Implementation/Debugging) are dispatched by `run_writer_subagent` (executor.rs:2204-2227) with `isolation_id = {run_id}:{task_id}`; the framework resolves isolation from the routed `SubagentDefinition.isolate_worktree` (executor.rs:1603-1618), calls `EkoWorktreeFactory::create` → `RunWorktree::acquire_fork` (worktree.rs:381-457; locked → error; unlocked → relock-and-reuse; pruned → materialize; absent → create), locks the checkout, binds `working_dir`, and runs the subagent.
2. **Finalize**: on every exit path of the dispatch task (success/failure/cancel/timeout), `handle.finalize()` runs (framework executor.rs:1836-1855): clean checkout → `cleanup_managed_worktree` removes checkout+branch; dirty → unlock and retain with a diff summary appended to the result (worktree.rs:1324-1352).
3. **Review + integrate**: after the review gate passes, `resolve_dispatch` (executor.rs:1457-1560) calls `integrate_reviewed_task` → `RealTaskDispatcher::integrate` (executor.rs:889-1029): cancel checks at entry and before the merge lock; `repo_merge_lock` (process-wide, per repo root); `spawn_blocking(integrate_fork_worktree)`; events MergeStarted/MergeCompleted/MergeFailed.
4. **Merge** (`integrate_existing_worktree`, worktree.rs:622-807): stage all worktree changes → validate changed files ⊆ declared ownership → commit writer changes (gpg-sign off, EKO identity, execution-id trailer) → refuse if main index non-empty → refuse if main tree dirty in writer-owned paths → `git merge-tree` preflight → `git merge --no-ff` → cleanup worktree+branch → outcome `Merged`/`NoChanges`/`AlreadyIntegrated` (trailer idempotency, worktree.rs:579-592). Failure at any step → `preserve_error` unlocks and retains the worktree, task Failed (executor.rs:1546-1557); merge failure → `abort_own_merge`.
5. **Unattended legacy**: TUI (events.rs:2300-2410) and GUI (panels.rs:2049-2163, frontend endpoints.ts:1987-2001) list/merge/discard/cleanup `eko-unattended-*` resources under `repo_merge_lock`; `cleanup_unattended_worktrees` never touches active runs (`is_run_active`, store.rs:554-559) and keeps changed worktrees.
6. **Persistence**: every store write appends to `events.jsonl` under a per-run in-process write lock with unique seq, then rewrites `plan.json`/`run-state.json` atomically with per-call unique tmp names (file_shadow.rs:118-280).

## Findings

### A-TSK-05-P2-01: A process crash leaves the fork worktree locked with no repair path — the logical writer task is permanently blocked

- Priority: P2
- Confidence: high (mechanism is deterministic; crash trigger frequency medium)
- Layer: application
- Evidence: `acquire_fork` treats any locked existing worktree as active and hard-errors (`worktree.rs:385-391`, "worktree for logical task … is already active"); the lock is `git worktree lock` (`lock_worktree`, worktree.rs:911-926), which persists across process death; unlock happens only in `finalize` (framework executor.rs:1836-1855, runs on all graceful exit paths incl. cancel/timeout), `preserve_error` (worktree.rs:630-636) and `cleanup_managed_worktree` (worktree.rs:1001-1017). Boot recovery (`store.rs:1631-1776`) resets orphaned claims but never touches git locks; `cleanup_unattended_worktrees` covers only `eko-unattended-*` (worktree.rs:1046-1070, 1238-1276). After a crash mid-writer, the run resumes, the task is redispatched, `acquire_fork` errors, and the dispatch hard-fails (framework executor.rs:1706-1717) → task Failed forever; only manual `git worktree unlock` unblocks it.
- Reachability: any process crash (kill -9, power loss) while a writer Subagent holds a fork worktree; every later retry/resume of that run in-process.
- Expected invariant: MASTER-PLAN:465 claims "worktree repair … complete"; design doc 2026-07-25 states locks represent "active ownership, not completed history" — a lock whose owner process is gone is stale and must be repairable, exactly as `cleanup_unattended_worktrees` does for the legacy prefix.
- Observed behavior: stale fork lock → permanent dispatch failure with a generic "already active" error; no repair path in-process, no surface, no auto-detection (lock reason vs `is_run_active`).
- Impact: the exact crash the boot-recovery machinery was built for leaves the run permanently stuck on that task; user must hand-edit git state.
- Root cause: the fork-worktree lifecycle has no stale-lock detection; the "lock = active" invariant is enforced without checking whether the owner is still alive (unlike the unattended path which keys on `is_run_active`).
- Direction: in `acquire_fork`, when the checkout is locked and the store reports the owning run is not active (`is_run_active`), auto-unlock (with a logged reason) before reuse; or expose an EKO command that unlocks stale fork worktrees; mirror `cleanup_unattended_worktrees`'s active-run check. Add a fixture: crash simulation (locked worktree + inactive run) → resume dispatches successfully.
- Regression validation: new unit test in worktree.rs "stale locked fork worktree with inactive run is unlocked on acquire"; existing reuse tests stay green; Q-FLT-02 crash fixture.
- Validation reports: [V03-01](../validations/A-TSK-05/V03-01.md), [V05-01](../validations/A-TSK-05/V05-01.md)

### A-TSK-05-P2-02: No lifecycle sweep exists for leaked `eko-fork-*` worktrees/branches — failed/cancelled/crashed leftovers accumulate indefinitely

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `cleanup_unattended_worktrees` lists only `BRANCH_PREFIX` (`eko-unattended-`) branches (`list_prefixed_branches`, worktree.rs:1046-1070; cleanup at :1238-1276); there is no counterpart for `FORK_BRANCH_PREFIX` (V01-01: zero references outside worktree.rs except the event payload at executor.rs:937). Leak sources: failed merge → worktree preserved with committed changes, consumable only by the same task's retry/integration (worktree.rs:630-636); run cancelled/abandoned after finalize → dirty worktree retained (worktree.rs:1334-1351); crash → locked worktree (P2-01). Run deletion/cleanup cascade does not exist (no `delete_run` in store.rs/service.rs, V03-01 grep).
- Reachability: any cancelled run with a dirty writer, any failed merge, any crash; all subsequent runs on the same repo.
- Expected invariant: managed resources have a bounded lifecycle (2026-07-25 doc: "automatic cleanup for empty environments, retention only when reviewable work exists"); AGENTS.md worktree section requires a cleanup path.
- Observed behavior: `eko-fork-*` checkouts (full-size worktrees) and branches are never swept; only manual per-task integration consumes them; the GUI/TUI cleanup commands ignore them.
- Impact: unbounded disk growth (each retained checkout ~ repo size) and stale `eko-fork-*` refs permanently visible in the user's repository, invisible to the product's own cleanup UI.
- Root cause: the fork namespace predates the unattended cleanup and was never given the same lifecycle management; cleanup exists for legacy resources only.
- Direction: extend the cleanup/queue surface to `eko-fork-*` with the same safety rules (skip active runs via `is_run_active`; keep changed worktrees; remove provably clean ones without `--force`), or attach fork-worktree cleanup to run terminal states. Also make `remove_unattended_worktree` (`--force`, worktree.rs:1203-1217) consistent with the fork path's clean-removal semantics to close the late-content race (the fork path refuses, worktree.rs:1001-1017, test `automatic_cleanup_refuses_new_dirty_content`).
- Regression validation: fixture "cancelled run with retained dirty fork worktree → sweep lists it, keeps it; user discard removes it"; "clean fork worktree after crash-unlock → sweep removes it"; existing unattended cleanup tests stay green.
- Validation reports: [V01-01](../validations/A-TSK-05/V01-01.md), [V03-01](../validations/A-TSK-05/V03-01.md)

### A-TSK-05-P2-03: Writer tasks routed to a non-isolated role silently run in the main checkout and report `no_changes` — isolation, ownership and dirty-tree protections are bypassed

- Priority: P2
- Confidence: high (mechanism) / medium (reachability depends on LLM or user role choice)
- Layer: application
- Evidence: `execute_task` routes every Implementation/Debugging task to `run_writer_subagent` by kind only (executor.rs:2204-2227); `run_writer_subagent` passes no isolation flag and performs no role check (executor.rs:2889-2978); the framework isolates only when the routed definition declares `isolate_worktree` (executor.rs:1603-1619), derived from frontmatter `worktree: true` (subagent_loader.rs:446, infra.rs:698-701). The builtin `general-purpose` role is `readonly: false, worktree: false` ("可读写当前工作区，不提供 worktree 隔离"); user-defined roles may likewise omit it. Integration then finds no fork branch and returns `NoChanges` (`integrate_fork_worktree`, worktree.rs:594-605), and the dirty-tree/staged-index/ownership-validation/merge-preflight checks (worktree.rs:705-754, 809-829) never run; the subagent's in-place edits remain uncommitted in the user's working tree.
- Reachability: LLM-chosen `agent_role` in a plan (planner allows any registered project/user/builtin role, profiles.rs:118-119) or explicit task role assignment selecting `general-purpose`/custom non-isolated role for a writer kind; also every `InPlace`-mode unattended run by design, with no runtime distinction.
- Expected invariant: writer isolation is the product invariant (AI_CODING prompt_suffix: "writer tasks should … use a writer-capable role so the runtime can isolate their changes"; MASTER-PLAN: "a writer Subagent receives an `eko-fork-*` worktree only when that writer is dispatched"); a writer must never silently mutate the main checkout, and an integration result must never claim `no_changes` when the writer modified the main tree.
- Observed behavior: the writer runs with write tools against the user's working directory; ownership conflict checks, dirty-overlap protection and the merge pipeline are skipped; the task completes with `worktree integration=no_changes`; per-run semaphores/locks give no cross-run mutual exclusion for such in-place writers.
- Impact: user's uncommitted changes in the main checkout can be overwritten by the writer (only `write_file`'s expected_hash guards it), subagent edits are silently mixed into the user's tree, and the model/user is misled by the `no_changes` integration report.
- Root cause: isolation enforcement lives only in the framework's definition flag; EKO's writer routing never validates or compensates for the routed role's isolation capability, and the integration boundary cannot distinguish "writer was isolated, no changes" from "writer ran in place".
- Direction: (a) enforce at dispatch — reject or auto-downgrade a writer task whose role lacks worktree isolation (mirroring the framework's hard-fail, or fall back with an explicit `in_place` integration status instead of `no_changes`); (b) surface a warning in the plan/review when a writer role is non-isolated; (c) gate `general-purpose` write capability per run write-mode. Delete nothing; the `NoChanges` status stays for genuinely clean isolated writers.
- Regression validation: fixture "writer task routed to non-isolated role → task fails with explicit isolation error (or reports `in_place` status), main checkout untouched"; "in-place writer with dirty overlap → refused"; Q-FLT-02 candidate.
- Validation reports: [V02-01](../validations/A-TSK-05/V02-01.md), [V03-01](../validations/A-TSK-05/V03-01.md)

### A-TSK-05-P2-04: Legacy GUI worktree helpers/commands in `panels.rs` duplicate the app-core authority and have already diverged

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `panels.rs` re-implements `parse_worktree_list` (:1828-1897, no `locked` line handling — the shared parser at worktree.rs:272-355 parses it), `validate_branch_name` (:1883), `default_worktree_path` (:1908-1927), `validate_worktree_target` (:1929-1955) and commands `list_worktrees`/`create_worktree`/`remove_worktree` (:1957-2047), registered at `src/tauri/mod.rs:295-297` and consumed by the frontend (`endpoints.ts:1972-1984`; `WorktreeInfo` has no `locked` field). The shared module doc claims centralisation of "operations that were previously scattered across Tauri panels.rs commands" (worktree.rs:3-9) but the scattered copies remain live; the legacy create/remove commands do not take `repo_merge_lock` and have no lock/merge integration.
- Reachability: GUI worktree panel (list/create/remove) on every Tauri session.
- Expected invariant: one parser/validator authority per git surface; GUI and TUI/task-runtime views of the same worktrees agree (AGENTS.md: no parallel authorities; F-EXT-02-P2-02/P2-03 containment fixes must apply once).
- Observed behavior: two parsers for the same `git worktree list --porcelain` output — the GUI panel cannot show/act on lock state while the unattended surface can; path-validation and containment fixes must be applied twice; the legacy commands manage worktrees outside the merge-lock discipline.
- Impact: divergence risk (GUI shows different facts than the task runtime), duplicated maintenance surface, and GUI-created worktrees can race EKO merges.
- Root cause: the shared module consolidated only the unattended/task paths; the three legacy panel commands were left on their own helpers.
- Direction: migrate `list_worktrees`/`create_worktree`/`remove_worktree` to the shared `task_runtime::worktree` API (extended with lock-aware create/remove), delete `panels.rs:1828-1909` helpers and the frontend's locked-less `WorktreeInfo` divergence, and take `repo_merge_lock` for mutations.
- Regression validation: frontend WorktreePanel smoke test (list/create/remove) against a fixture repo; parser round-trip test shared between GUI and app-core; existing unattended command tests stay green.
- Validation reports: [V01-01](../validations/A-TSK-05/V01-01.md), [V02-01](../validations/A-TSK-05/V02-01.md)

### A-TSK-05-P3-01: `infra.rs` comment claims non-git projects run unisolated with a warning, but every writer task hard-fails there instead

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: infra.rs:396-400 ("if it's not a git repo, no factory is injected (subagents declaring isolation log a warning and run unisolated — the framework's default)") vs framework hard-fail `echo-agent/src/agent/subagent/executor.rs:1609-1619` ("Subagent … declares isolate_worktree but no WorktreeFactory is configured; refusing to run without isolation") and `RealTaskDispatcher::integrate` requiring a git root (executor.rs:916-919).
- Reachability: any Implementation/Debugging task in a non-git working directory (docs/plain directories).
- Expected invariant: comment describes code behavior (AGENTS.md: no misleading API/comments); writer dispatch either fails with a product-level explanation or falls back explicitly.
- Observed behavior: in non-git workspaces the writer task always fails at dispatch with a framework-internal error mentioning `NoWorktreeFactory`; the documented graceful fallback does not exist.
- Impact: misleading comment (a future implementer may "fix" the fallback or misattribute the failure); poor user-facing error in a common workspace shape.
- Root cause: the comment predates the framework's hard-fail guard and was never reconciled.
- Direction: either implement the documented fallback (run unisolated only under explicit user consent) or correct the comment and surface a clear product error at run/plan level ("workspace is not a git repository; writer tasks require worktree isolation").
- Regression validation: unit fixture "writer task in non-git dir → dispatch fails with the product-level message"; comment update.
- Validation reports: [V02-01](../validations/A-TSK-05/V02-01.md), [V05-01](../validations/A-TSK-05/V05-01.md)

### A-TSK-05-P3-02: `EkoWorktreeFactory::create`/`finalize` run blocking git subprocesses inline on the async runtime

- Priority: P3
- Confidence: high
- Layer: adapter
- Evidence: the framework calls `factory.create()` synchronously inside the spawned dispatch task (`executor.rs:1706-1717`) and `finalize` synchronously (:1836-1855); `EkoWorktreeFactory` explicitly accepts blocking git ops ("the git ops themselves are short and synchronous — acceptable", worktree.rs:1306-1309), while the framework contract requires the application factory to offload ("`create` may block on a git subprocess; the application's factory is responsible for offloading that to spawn_blocking", echo-agent worktree.rs:17-18) and the integration path already does offload (executor.rs:961). Same defect class as F-EXT-02-P3-01 (framework git tools).
- Reachability: every writer dispatch (create: worktree list/prune/merge-base/add/lock subprocesses) and every finalize (status/rev-list/diff) on the async runtime.
- Expected invariant: async tooling must not block runtime workers; cancellation can interrupt the work.
- Observed behavior: on large repos a writer dispatch stalls a runtime worker for the git subprocess duration; cancellation cannot interrupt the git calls.
- Impact: latency/concurrency degradation; minor relative to the shell/git tool family defect already tracked.
- Root cause: the factory's comment chose convenience over the framework's documented offload contract.
- Direction: wrap create/finalize bodies in `tokio::task::spawn_blocking` inside `EkoWorktreeFactory` (matching `RealTaskDispatcher::integrate`).
- Regression validation: existing worktree unit tests stay green (they call the factory directly, so the wrapper must be in the trait-impl boundary); optional timing test.
- Validation reports: [V02-01](../validations/A-TSK-05/V02-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition + duplicate search (worktree/file-shadow/ownership concepts; panels.rs duplication; fork sweep absence) | yes | passed | [V01-01](../validations/A-TSK-05/V01-01.md) |
| V02 | Registration + runtime reachability (factory injection → framework dispatch → finalize; review → integrate; TUI/GUI surfaces; label identity) | yes | passed | [V02-01](../validations/A-TSK-05/V02-01.md) |
| V03 | Invariant/edge cases (ownership conflict; dirty-tree protection; reuse/repair/cleanup; merge failure + cancellation) | yes | failed (P2-01/P2-02/P2-03; P3-01/P3-02 cross-checked) | [V03-01](../validations/A-TSK-05/V03-01.md) |
| V04 | `cargo test -p echo-agent-app-core --locked tasks::task_runtime::worktree` | yes | passed (exit 0, 25 ok) | [V04-01](../validations/A-TSK-05/V04-01.md) |
| V04 | `cargo test -p echo-agent-app-core --locked tasks::task_runtime::planner` | yes | passed (exit 0, 11 ok) | [V04-02](../validations/A-TSK-05/V04-02.md) |
| V04 | `cargo test -p echo-agent-app-core --locked tasks::task_runtime::file_shadow` | yes | passed (exit 0, 9 ok) | [V04-03](../validations/A-TSK-05/V04-03.md) |
| V04 | `cargo test -p echo-agent-app-core --locked tasks::task_runtime::executor::tests::ownership` | yes | passed (exit 0, 3 ok) | [V04-04](../validations/A-TSK-05/V04-04.md) |
| V04 | `cargo test -p echo_agent --features subagent --locked agent::subagent::worktree` | yes | passed (exit 0, 4 ok) | [V04-05](../validations/A-TSK-05/V04-05.md) |
| V05 | Historical-document drift (MASTER-PLAN worktree rows; 2026-07-22/25 design docs; infra comment) | yes | passed (drift classified) | [V05-01](../validations/A-TSK-05/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN: "Unattended worktree lifecycle and review parity — Complete" | current | worktree.rs:1093-1276 + TUI/GUI surfaces (V05-01) |
| MASTER-PLAN: "Logical-task worktree reuse and content-aware cleanup — Complete" | current for reuse; incomplete for cleanup | reuse verified (V04-01); no fork sweep (P2-02), no stale-lock repair (P2-01) |
| MASTER-PLAN: "worktree repair … complete" | current only for legacy `eko-unattended-*` surface | 2026-07-22 doc + cleanup_unattended_worktrees; fork namespace lacks repair (P2-01) |
| 2026-07-22 doc: "unlocks inactive stale worktrees; reports active process ownership separately" | current | `cleanup_unattended_worktrees` + `is_run_active` (store.rs:554-559); cross-process `--force` caveat in P2-02 |
| 2026-07-25 doc: at-most-one worktree per logical task; lock = active ownership; reuse/retention/clean-removal rules | current | worktree.rs:375-457, 1001-1017, 1324-1352; tests V04-01 |
| infra.rs: "subagents declaring isolation log a warning and run unisolated" (non-git dirs) | regressed (comment) | framework hard-fail executor.rs:1609-1619 (P3-01) |

## Coverage And Uncertainty

- All conclusions are static except the V04 test runs; no live LLM writer dispatch was executed (read-only review). P2-03's reachability depends on LLM/user role choice — mechanism verified, trigger not empirically reproduced. P2-01's trigger (process crash mid-writer) is rare; mechanism deterministic.
- Cross-process concerns (two EKO processes sharing one repo/store): fork worktree merges are fail-safe via git's own index.lock (one merge fails cleanly); `cleanup_unattended_worktrees` force-unlocks based on this process's in-memory `is_run_active`, which is unsafe against a live run in another process (recorded in P2-02 direction, not a standalone finding); FileTaskShadow same-run appends are per-process only (single-process product assumption, matches A-TSK-01 file-authority scope).
- `has_writer_file_overlap` (planner.rs:197) has zero production callers — the advertised non-blocking overlap warning is unwired (recorded in V03, not a finding; A-TSK-02/planner surfaces own it).
- The pause/cancel integration interplay during an in-flight blocking merge (merge completes after cancel; trailer makes resume idempotent) was verified statically, not dynamically (Q-FLT-02 candidate).
- Framework `dispatch_team`/teammate worktree behavior (F-SUB-02 scope) and the `echo-tools` worktree tools (F-EXT-02) were not re-reviewed.
- Run deletion cascade does not exist (no `delete_run`); conversation-deletion cleanup of task/worktree artifacts belongs to A-STATE-01.

## Handoff

- Downstream tasks may rely on: EKO's isolated-writer pipeline is sound at the happy path — ownership-safe waves (V04-04), per-file locks, per-logical-task worktree reuse with attempt-scoped events (V04-01), thorough dirty-tree/staged-index/merge-preflight protection of the user's main checkout (V04-01), deterministic merge with execution-id idempotency (V04-01), in-process file-shadow concurrency (V04-03), and one generic framework trait seam (V04-05). No P0/P1 data-loss vector in the isolated path.
- Findings to fold into roadmap: P2-01 (stale fork-lock repair), P2-02 (fork-resource sweep + `--force` consistency), P2-03 (writer isolation enforcement for non-isolated roles), P2-04 (panels.rs duplicate authority migration), P3-01 (comment/behavior drift), P3-02 (spawn_blocking offload).
- Reports to read: the 9 validation reports above; dependency reports A-TSK-03 (integration ordering, per-run semaphores) and F-EXT-02 (framework worktree tool defects P2-02/P2-03 and git blocking P3-01, same defect classes).
- Stale conditions: this report becomes stale if `worktree.rs` lock/cleanup/integration logic, `planner.rs` ownership classification, `executor.rs` writer routing or `RealTaskDispatcher::integrate`, the framework `dispatch_fork` isolation resolution, the subagent frontmatter isolation flags, or `panels.rs` worktree commands change; also if a fork-worktree sweep or stale-lock repair appears (P2-01/P2-02 weakened).
- Follow-up task IDs: A-TSK-04 (recovery interplay with claims), A-PROJ-01 (diff/worktree authority consumers), X-TSK-01 (adapter conformance of the merge boundary), Q-FLT-02 (crash-stale-lock, cancel-during-integration, non-isolated-role fixtures), S-RDM-01 (roadmap ordering of P2-01..P2-04).
