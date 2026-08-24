# A-TSK-05: Worktree, file ownership, and merge policy

> Status: complete
> Reviewer: Codex review subagent
> Executor: Codex review subagent
> Review date: 2026-08-13
> `echo-agent` commit: 3aa7929928442aab91e4dce9c426d909a5f0a1ab
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: CLI clean; framework had external changes including `echo-tools` worktree files and unrelated core/integration/orchestration/ReAct/state/eval/evolution/trace files; all inspected framework source was reconstructed from committed `HEAD`, no external diff was read
> Accepted by: Codex primary reviewer after independent source-anchor,
> reachability, finding-count, link, executor, commit, and isolation sampling.

## Question

Does EKO safely isolate concurrent writers, reuse logical-task worktrees, protect user changes, and finalize/merge deterministically across cancellation and restart?

## Scope

- EKO TaskRuntime Git fork identity, acquisition, locking, finalization, integration, cleanup, and merge serialization.
- PlanTask file-ownership normalization, ready-wave conflict policy, per-file guard, and actual-diff enforcement.
- Claim/cancel ordering at the review-to-integration boundary.
- EKO concrete data-workspace creation, retained ownership, locator/lineage handoff, and cleanup policy.
- Registration/real reachability and static existing-test inventory.

## Out Of Scope

- Generic shell/file/Git/checkpoint/worktree Tool correctness: F-EXT-02.
- General ready-frontier/retry/completion-cancel/run settlement: A-TSK-03.
- General claim identity, revision fencing, retry/recovery, and event replay: A-TSK-04/F-TSK-03.
- Framework Subagent mode/team behavior and model-reported artifact provenance: F-SUB-01/F-SUB-02.
- Review acceptance, required artifact semantics, complete result/parent projection: A-TSK-06.
- Source fixes, Cargo/rustc/tests/builds, dynamic fixtures, and network.

## Inputs

- Root AGENTS.md; shared README/REPORTING/TASKS; Codex README and templates.
- Exact Codex dependencies A-TSK-03 and F-EXT-02; A-TSK-04 for explicit claim/recovery deduplication.
- Current CLI source and committed framework source at the hashes above.
- Narrow F-SUB report boundaries only for requested artifact/workspace deduplication; no other reviewer directory was read.

## Layering Decision

| Classification | Current answer |
|---|---|
| Generic mechanism | Invocation-scoped worktree/workspace handles, stable runtime isolation identity, cancellation propagation, and claim-aware terminal contracts are reusable framework concerns. |
| EKO product policy | Git target selection, exact-file ownership, local dirty-tree preservation, review-before-merge, retained workspace duration, and TaskRuntime cleanup are local-assistant application decisions. |
| Adapter boundary | EKO injects factories, maps PlanTask identity/ownership, integrates reviewed output, and settles the one framework claim. It must not merge before the claim-bound side effect is authorized or maintain a second task graph. |
| Duplicate search | Searched worktree/workspace factories, branch/lock/base/target/execution identities, ownership selection/validation, merge/cancel/cleanup/recovery, data-workspace manifests, and all registrations/callers across both repositories. |
| Migration deletion | Extend existing `WorktreeFactory`/`DataWorkspaceFactory` and EKO TaskRuntime records. Delete role-derived fork identity and implicit-current-HEAD integration after migration; replace `TempDir::keep()` without an owner/sweep. Do not add another executor, store, or artifact authority. |

No SQLite or public-service permission boundary is involved. User dirty protection is a valid local data-loss guard.

## Current Path

```text
infra -> EkoWorktreeFactory / EkoDataWorkspaceFactory
  -> Subagent definition isolation flag
  -> RuntimeDagExecutor claim -> EKO writer dispatch
  -> framework SubagentExecutor
       -> factory.create(label = agent_role + run_id:task_id)
       -> invocation working_dir -> Subagent execution
       -> factory.finalize(diff or top-level filename listing)
  -> EKO review gate
  -> RealTaskDispatcher::integrate
       -> current-HEAD merge lock/cancel prechecks
       -> blocking integrate_fork_worktree
       -> actual diff ownership + dirty/index guards + merge-tree + git merge
  -> claimed Completed CAS
```

Positive controls are meaningful. Known file owners are normalized and conflicting owners do not share a wave; unknown writers serialize. Integration verifies the actual staged diff, refuses any staged user index, rejects dirty overlap, preflights conflicts, preserves failed worktrees, and deletes only demonstrably clean/no-unique-work branches ([V03](../validations/A-TSK-05/V03-01.md), [V04](../validations/A-TSK-05/V04-01.md)). Four lifecycle gaps remain.

## Findings

### A-TSK-05-P1-01: An obsolete claimed attempt can merge after cancellation or supersession

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:912`, `:931`, `:961`, `:1518`, `:1535`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/worktree.rs:756`
- Reachability: reviewed implementation/debugging result -> current-claim check -> dispatcher integration -> blocking Git merge -> claimed Completed CAS.
- Expected invariant: cancellation/supersession must prevent an obsolete attempt from mutating the authoritative checkout, or the exact side effect must be durably settled under the same claim before another state wins.
- Observed behavior: cancellation is checked only before the blocking integration starts. The merge performs no cancel/claim check, and the next claim CAS occurs after the Git commit; it may return Superseded even though main already changed.
- Impact: cancellation can report/settle without stopping the mutation, or a revised/retried PlanTask can run after an obsolete attempt's code has already entered the authoritative branch.
- Root cause: integration is an irreversible side effect placed between a preflight claim read and the final claim CAS, with no claim-bound commit/settlement protocol.
- Direction: make integration one idempotent claim-bound application operation: persist Prepared with target/base/execution, revalidate claim immediately before the merge commitment, then durably record commit and terminal claim outcome. If cancellation cannot interrupt a started Git merge, define that point as a commit barrier and settle its result before honoring cancel. Delete the current free-standing merge-before-CAS path; retain the single TaskRuntime controller.
- Regression validation: barrier at merge start; race cancel, pause, plan revision, and claim replacement; assert either no merge or one matching durable terminal record, never Superseded plus an unowned commit.
- Validation reports: [V07](../validations/A-TSK-05/V07-01.md), [V09](../validations/A-TSK-05/V09-01.md), [V10](../validations/A-TSK-05/V10-01.md)

### A-TSK-05-P1-02: Integration silently targets whichever branch is current at settlement time

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/worktree.rs:364`, `:431`, `:579`, `:646`, `:681`, `:728`, `:756`, `:928`
- Reachability: writer fork from branch A -> user/process switches authoritative checkout to branch B -> reviewed integration or restart retry.
- Expected invariant: fork creation records an explicit target ref/base, and integration/retry verifies and uses that target independent of ambient checkout state.
- Observed behavior: creation bases from current `HEAD` but stores base only in the in-memory handle. Integration recomputes against current `HEAD` and runs `git merge` there. Idempotency searches the execution trailer only in current `HEAD` history.
- Impact: a valid task can be merged into the wrong branch. After a prior merge on A and switch to B, retry may not find the trailer; with the fork already cleaned it returns `NoChanges` and can mark the task completed on B without its changes.
- Root cause: ambient checkout state is used as durable target identity.
- Direction: persist target ref plus immutable creation base with the TaskRun/PlanTask integration record; reject target drift or integrate through an explicit ref operation with user-visible policy. Search idempotency by the persisted integration record/commit and verify reachability from that target. Delete implicit-current-HEAD target inference.
- Regression validation: create on A, switch to B before integrate and after merge-before-CAS, restart, then assert deterministic target behavior and idempotent recovery.
- Validation reports: [V06](../validations/A-TSK-05/V06-01.md), [V09](../validations/A-TSK-05/V09-01.md)

### A-TSK-05-P1-03: Fork worktree identity is not claim-bound or restart-recoverable

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:192`, `:196`, `:935`, `:2213`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/worktree.rs:381`, `:911`, `:1093`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/echo-orchestration/src/tasks/revisioned.rs:521`, `:640`
- Reachability: process stops after fork lock but before finalize; or a failed/Blocked task updates `agent_role` before retry.
- Expected invariant: one logical PlanTask has one immutable fork identity; locks carry a recoverable live-owner lease and boot recovery can safely reclaim or surface retained dirty work.
- Observed behavior: identity includes mutable `agent_role`. Any locked checkout is rejected as active, while the lock reason has no claim/process/expiry identity. Cleanup/recovery enumerates only legacy `eko-unattended-*`, not fork branches. A Blocked task may change role and therefore derive a new branch.
- Impact: process interruption can permanently block retry; role edits can orphan preserved changes and create a second fork for the same logical task, so review/integration no longer sees the first attempt's work.
- Root cause: a Git advisory lock and mutable display/role label substitute for a durable TaskRun/PlanTask ownership lease.
- Direction: key the fork by immutable run/task ID, persist branch/path/base/claim owner, and reconcile locks on boot using live driver/claim evidence while preserving dirty work. Provide one fork listing/reclaim/cleanup path. Delete role-derived branch identity and bare-label lock reasons after migration.
- Regression validation: stop after create/dirty write/finalize boundaries; restart twice; edit role on a Blocked task; assert one preserved fork, safe lease reclaim, and no orphan branch.
- Validation reports: [V05](../validations/A-TSK-05/V05-01.md), [V09](../validations/A-TSK-05/V09-01.md)

### A-TSK-05-P1-04: Retained data workspaces have no durable owner, usable handoff locator, or cleanup

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/worktree.rs:1368`, `:1414`, `:1425`, `:1437`, `:1442`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/src/agent/subagent/workspace.rs:53`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/src/agent/subagent/executor.rs:1857`
- Reachability: any registered data/research Subagent with `workspace: true` -> injected EKO factory -> generated files -> finalize -> later analyst/terminal cleanup.
- Expected invariant: retained outputs are durably tied to TaskRun/PlanTask/SubagentRun, handed off as an unambiguous complete manifest/locator, and cleaned only after collection or terminal ownership transfer.
- Observed behavior: `TempDir::keep()` discards automatic cleanup and returns a random path with no ledger/sweep. Finalize emits only lossy, non-recursive top-level names; the framework appends these names without the absolute workspace locator to ordinary model output. A downstream isolated Subagent receives another random directory.
- Impact: the intended downstream analyst cannot deterministically open prior shards from the structured result, nested outputs are omitted, and every run leaks unowned temporary data indefinitely. This makes the advertised parallel collect/synthesize path unreliable.
- Root cause: a text listing is used as both artifact handoff and lifetime ownership, but it carries neither a handle nor provenance.
- Direction: let the existing TaskRuntime own a structured workspace manifest/lease containing run/task/execution, root locator, recursive entries and hashes/sizes as appropriate; hand references to downstream collection and clean at explicit terminal/release. Replace unowned `keep()` and delete the contradictory drop-cleanup comment. A-TSK-06 should consume this manifest rather than create another artifact store.
- Regression validation: nested/non-UTF8-name/empty/failed/cancelled/restarted workspaces, downstream collection, and terminal sweep; assert complete stable lineage and no deletion before release or leak after release.
- Validation reports: [V08](../validations/A-TSK-05/V08-01.md), [V09](../validations/A-TSK-05/V09-01.md), [V10](../validations/A-TSK-05/V10-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition/duplicate/layer authority search | yes | passed | [V01](../validations/A-TSK-05/V01-01.md) |
| V02 | Registration and runtime reachability | yes | passed | [V02](../validations/A-TSK-05/V02-01.md) |
| V03 | Ownership conflict and actual-diff enforcement | yes | passed | [V03](../validations/A-TSK-05/V03-01.md) |
| V04 | Dirty-tree, conflict, and clean-only cleanup inspection | yes | passed | [V04](../validations/A-TSK-05/V04-01.md) |
| V05 | Logical fork identity, reuse, and restart repair | yes | failed | [V05](../validations/A-TSK-05/V05-01.md) |
| V06 | Target ref/base and integration idempotency | yes | failed | [V06](../validations/A-TSK-05/V06-01.md) |
| V07 | Cancellation/claim/integration ordering | yes | failed | [V07](../validations/A-TSK-05/V07-01.md) |
| V08 | Data-workspace ownership, lineage locator, and cleanup | yes | failed | [V08](../validations/A-TSK-05/V08-01.md) |
| V09 | Existing tests and missing edge cases | yes | failed | [V09](../validations/A-TSK-05/V09-01.md) |
| V10 | Dependency/historical finding deduplication | yes | passed | [V10](../validations/A-TSK-05/V10-01.md) |
| V11 | Targeted executable fixtures | policy-deferred | not_run | [V11](../validations/A-TSK-05/V11-01.md) |
| V12 | Report/link/executor/source integrity gate | yes | attempt 1 inconclusive; attempt 2 passed | [A1](../validations/A-TSK-05/V12-01.md), [A2](../validations/A-TSK-05/V12-02.md) |
| V30 | Primary acceptance sampling | yes | passed | [V30](../validations/A-TSK-05/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| F-EXT-02 generic worktree cleanup can destructively force-remove unverified work | current; canonical in dependency | [V10](../validations/A-TSK-05/V10-01.md) |
| A-TSK-03 completion/cancel and run settlement are split | current; canonical in dependency | [V07](../validations/A-TSK-05/V07-01.md), [V10](../validations/A-TSK-05/V10-01.md) |
| A-TSK-04 stale claim/recovery writes need stronger fencing | current; canonical in dependency | [V05](../validations/A-TSK-05/V05-01.md), [V07](../validations/A-TSK-05/V07-01.md), [V10](../validations/A-TSK-05/V10-01.md) |
| EKO comments: retained data workspace is cleaned when handle drops and downstream can find shards | contradicted by current code | [V08](../validations/A-TSK-05/V08-01.md) |

## Coverage And Uncertainty

- Pure static review only. No Cargo/rustc/test/build/fixture/network process ran; V11 is explicitly `not_run`.
- Framework was externally dirty throughout finalization. All framework anchors were read from `git show HEAD:path`; none of the dirty working copies/diffs was read. CLI source remained clean before report creation.
- A-TSK-05-P1-01 does not duplicate A-TSK-03's general completion/cancel race: it identifies the EKO-specific irreversible Git side effect occurring before final claim CAS.
- A-TSK-05-P1-04 covers concrete workspace lifetime and locator. A-TSK-06 must separately assess acceptance, complete artifact projection, and bounded parent context.
- Cross-process Git activity remains visible through Git errors but was not dynamically exercised. The proposed target/claim ledger must preserve local single-user behavior without adding online-service permissions.

## Handoff

- Preserve the existing positive file-ownership, dirty-overlap, preflight, and clean-only cleanup guards.
- Fix as one coherent application lifecycle: immutable run/task fork identity plus persisted target/base/claim; claim-bound idempotent integration; restart reconciliation; structured workspace manifest and release cleanup.
- Extend existing framework isolation handles only where reusable identity/manifest fields are needed. Keep Git target/dirty/review policy in EKO and retain one framework `RuntimeDagExecutor` plus one TaskRuntime graph.
- A-TSK-06 should read P1-04's workspace manifest boundary and own only artifact acceptance/parent projection. Cross-repository synthesis should merge P1-01 with A-TSK-03/A-TSK-04 lifecycle work while retaining this Git-side-effect evidence.
- This report becomes stale if EKO `worktree.rs`, planner ownership, integration dispatcher/claim settlement, infra factory injection, framework Subagent isolation handles/executor, or task role patch rules change.
