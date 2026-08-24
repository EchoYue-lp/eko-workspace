# A-TSK-06: Task review, artifacts, and parent context

> Status: complete
> Reviewer: Codex review subagent
> Executor: Codex review subagent
> Review date: 2026-08-13
> `echo-agent` commit: 3aa7929928442aab91e4dce9c426d909a5f0a1ab
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: CLI source clean at inspection; framework externally dirty, so inspected framework files came from committed `git show HEAD:<path>` blobs and no external diff was read
> Accepted by: Codex primary reviewer after independent committed-source,
> reachability, finding-count, link, executor, commit, and isolation sampling.

## Question

Are complete Subagent results, checks, acceptance, artifacts, and bounded parent summaries preserved without leaking thinking protocol?

## Scope

- Framework Subagent full-output and structured-result contract, artifact hydration, observed verification, and thinking-event separation.
- EKO TaskRuntime result conversion, release/summary/review persistence, review gate, explicit retry, artifact projection, worktree integration rewrite, dependency prompt, recovery capsule, memory bridge, and task tool final output.
- Definition, registration, real reachability, restart identity, UTF-8 bounds, and existing static tests.

## Out Of Scope

- Generic Subagent prompting/streaming/compression behavior beyond the exact adapter boundary; retained as F-SUB/F-CMP catalog ownership without reading those reports.
- Claim/revision/recovery fencing already owned by A-TSK-04 and conversation persistence owned by A-STATE-01.
- Worktree creation, dirty-tree protection, merge ownership, and cleanup policy as a whole; only the effect of successful cleanup on persisted artifact locators is in scope.
- Source fixes, Cargo/rustc/tests/builds, dynamic fixtures, and network activity.

## Inputs

- Root AGENTS.md; shared README, REPORTING, exact A-TSK-06 card in TASKS, templates, and Codex README.
- Exact Codex dependencies [A-TSK-04](A-TSK-04.md) and [A-STATE-01](A-STATE-01.md), both complete.
- Current clean CLI source at the stated commit and committed framework blobs at the stated commit. No other reviewer directory or non-dependency atomic report was read.

## Layering Decision

| Classification | Current answer |
|---|---|
| Generic mechanism | A provider-neutral Subagent result must separate complete user-facing output, bounded structured outcome, runtime-observed evidence, execution identity, and thinking events; this correctly belongs in `echo-agent`. |
| EKO product policy | PlanTask acceptance, review policy, worktree integration, durable artifact presentation, dependency summaries, recovery capsules, and final task tool projection belong in `echo-agent-cli`. |
| Adapter boundary | EKO should losslessly convert the framework outcome, persist one execution-scoped terminal fact, review the complete final output plus authoritative evidence, and derive bounded typed projections without owning a second lifecycle. |
| Duplicate search | Searched both repositories for result/outcome/summary/review/artifact/verification/full-output/thinking/recovery definitions, stores, formatters, callers, and tests. One DAG and one file-backed TaskRuntime authority remain; result artifacts and `ArtifactProduced` are currently disconnected projections. |
| Migration deletion | Extend the existing `TaskExecutionSummary`/release event and one same-ID PlanTask retry path. Delete the unused `PlanTask` payload from `ReviewGateOutcome::NeedsFix`, stale fix-task comments/fields if no longer part of the model, and ad hoc dependency string assembly after typed projection replaces it. Do not create another task, store, or artifact authority. |

No SQLite or public-service permission boundary is involved.

## Current Path

```text
typed AgentEvent stream
  -> thinking events (transient only) / final output buffer
  -> SubagentResult { complete output, bounded SubagentOutcome }
  -> EKO SubagentTaskResult (lossless structured mapping)
  -> TaskExecutionSummary + execution-scoped SubagentReleased(full_output, result)
  -> hard evidence assessment(checks/artifacts) -> semantic review(full_output)
  -> reviewed writer integration -> Completed

restart -> exact execution-id SubagentReleased -> same result/full_output -> review
downstream -> get_summary -> ad hoc (summary + written files + decisions) string -> prompt
artifact UI -> ArtifactProduced events only (no production bridge from result artifacts)
```

The immediate result and restart paths retain complete output, and typed thinking events do not enter it. The defects start in EKO's later review-retry, artifact, and parent projections.

## Findings

### A-TSK-06-P1-01: The downstream context projection is both lossy and unbounded

- Priority: P1
- Confidence: high
- Layer: application/adapter
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/src/agent/subagent/types.rs:245`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs:1758`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:2073`, `:2595`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/subagent_prompt.rs:400`
- Reachability: completed upstream PlanTask -> persisted `TaskExecutionSummary` -> ready dependent dispatch -> `collect_dependency_summaries` -> `EkoPromptPayload::planned_task` -> `append_dependencies` -> Subagent model input.
- Expected invariant: the parent projection is aggregate-bounded yet retains the stable locators/evidence/next-step facts required for a dependent task to consume upstream work.
- Observed behavior: it includes only summary, written paths, and decisions. It drops artifact locators/identity, verification, remaining work, next implications, suggestions, and read lineage. Despite being described as compact/full context, up to 64 paths of 2,048 characters per dependency are joined without a per-dependency or aggregate prompt budget.
- Impact: a dependent Subagent can be unable to locate a required artifact or know which checks passed and must re-read/recompute work; sufficiently large path/dependency sets can consume model context and prevent the downstream node from executing reliably.
- Root cause: a typed structured summary is flattened by an ad hoc formatter with neither a field contract nor aggregate budget.
- Direction: introduce one typed parent projection with explicit required fields, stable artifact locators, per-field/per-dependency/aggregate Unicode-safe budgets, and omission markers. Delete `collect_dependency_summaries` string assembly after migration.
- Regression validation: build multiple completed dependencies containing long Unicode paths, artifacts, observed checks, and next implications; assert essential fields survive, omissions are explicit, and final prompt size stays within a fixed budget.
- Validation reports: [V02](../validations/A-TSK-06/V02-01.md), [V06](../validations/A-TSK-06/V06-01.md), [V09](../validations/A-TSK-06/V09-01.md), [V10](../validations/A-TSK-06/V10-01.md)

### A-TSK-06-P1-02: Review feedback is discarded before the same PlanTask retries

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/review.rs:122`, `:241`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:1475`, `:1822`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:1211`, `:1261`
- Reachability: reviewable task completes -> reviewer returns NeedsFix -> `build_fix_task` builds same-ID issue-enriched brief -> executor matches `NeedsFix(_fix_task)` -> blocks -> user invokes real retry -> store increments retry while retaining original description -> redispatch.
- Expected invariant: a review failure stays on the canonical PlanTask and its concrete issues become immutable input to the next attempt.
- Observed behavior: the issue-enriched task value is discarded. The stored review remains visible but no prompt path consumes it, while explicit retry deliberately leaves task title/description unchanged.
- Impact: the next Subagent is asked to repeat the original assignment without the defects it must correct; retries can reproduce the same output and consume the review circuit-breaker budget without a convergence signal.
- Root cause: comments and helper still implement an abandoned generated-fix-task design, while the runtime switched to explicit same-node retry without adding a feedback projection.
- Direction: persist review feedback against the reviewed execution/attempt and inject a bounded issue brief into the same PlanTask's next invocation. Delete the unused `PlanTask` payload/helper and stale `created_fix_task_id` contract if no remaining product path uses generated fix nodes.
- Regression validation: force NeedsFix with two concrete issues, retry through the production command, and assert the next execution has the same task ID, new execution ID, and both review issues exactly once in its prompt.
- Validation reports: [V04](../validations/A-TSK-06/V04-01.md), [V10](../validations/A-TSK-06/V10-01.md), [V11](../validations/A-TSK-06/V11-01.md)

### A-TSK-06-P1-03: Accepted writer artifacts lose their durable locator and never enter the public artifact projection

- Priority: P1
- Confidence: high
- Layer: application/adapter
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/src/agent/subagent/types.rs:600`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:1685`, `:1734`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/worktree.rs:648`, `:786`, `:1001`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:1432`, `:1800`; `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/task_runtime.rs:119`
- Reachability: writer Subagent reports relative artifact -> framework resolves it to the isolated worktree and hashes it -> EKO persists result -> review passes -> integration merges changed files and removes worktree -> result summary remains -> GUI/Tauri asks `list_artifacts`.
- Expected invariant: every accepted artifact has one stable ID, current locator, digest, producer execution, and public projection after integration/cleanup.
- Observed behavior: successful integration rewrites only touched files/decision. The artifact still claims `available=true` at the deleted worktree path. `list_artifacts` reads only `ArtifactProduced` events, but `add_artifact` is called only by its isolated store test, not task completion.
- Impact: after a successful reviewed task, downstream/UI consumers can receive an empty artifact list or a locator that no longer exists, defeating required-artifact reuse and provenance despite acceptance having passed.
- Root cause: result artifacts and TaskRuntime artifacts are parallel representations with no integration/rebase projection owner.
- Direction: make one artifact record authoritative. At integration, rebase writer paths to the authoritative checkout or copy content to retained artifact storage, revalidate hash/availability, emit the durable projection with execution and merge lineage, and derive summaries/UI from it. Delete the disconnected projection path after cutover.
- Regression validation: produce a required file in a writer worktree, pass review, integrate and clean the worktree, restart, then list/read by artifact ID and verify bytes/hash/provenance from the durable locator.
- Validation reports: [V05](../validations/A-TSK-06/V05-01.md), [V10](../validations/A-TSK-06/V10-01.md)

### A-TSK-06-P2-04: Basename matching can satisfy an artifact requirement with the wrong locator

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:737`, `:773`
- Reachability: a PlanTask declares a basename-only `required_artifacts` entry -> current execution reports/hydrates a same-named file under any directory -> hard acceptance assessment.
- Expected invariant: a required artifact refers to one normalized logical locator or explicit artifact ID; path relaxation is intentional and collision-free.
- Observed behavior: `artifact_matches` treats any actual file with the same basename as a match when the requirement itself is a basename. Digest and producer bind the file to the attempt, but not to the intended directory or artifact role.
- Impact: a task can pass hard artifact acceptance after producing a different same-named file, leaving the actual requested deliverable absent.
- Root cause: artifact requirements are free-form strings and path matching mixes exact, suffix, and basename semantics without declaring which identity the plan requested.
- Direction: normalize requirements into typed IDs/locators at plan validation. Permit basename matching only through an explicit selector with uniqueness enforcement; delete the implicit basename fallback.
- Regression validation: create two current-attempt files with the same basename under different roots and assert only the declared locator/ID can satisfy acceptance.
- Validation reports: [V03](../validations/A-TSK-06/V03-01.md), [V10](../validations/A-TSK-06/V10-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Commit and dirty-source isolation | yes | passed | [V00](../validations/A-TSK-06/V00-01.md) |
| V01 | Definition, duplicate, and authority search | yes | passed | [V01](../validations/A-TSK-06/V01-01.md) |
| V02 | Complete output versus structured result map | yes | passed | [V02](../validations/A-TSK-06/V02-01.md) |
| V03 | Acceptance/check/artifact identity separation | yes | failed -> finding | [V03](../validations/A-TSK-06/V03-01.md) |
| V04 | Review outcome to retry context | yes | failed -> finding | [V04](../validations/A-TSK-06/V04-01.md) |
| V05 | Artifact retention and public projection | yes | failed -> finding | [V05](../validations/A-TSK-06/V05-01.md) |
| V06 | Bounded parent dependency projection | yes | failed -> finding | [V06](../validations/A-TSK-06/V06-01.md) |
| V07 | Thinking-protocol non-leak | yes | passed | [V07](../validations/A-TSK-06/V07-01.md) |
| V08 | Restart-equivalent result/review route | yes | passed | [V08](../validations/A-TSK-06/V08-01.md) |
| V09 | Compact context, memory, and final parent output | yes | passed | [V09](../validations/A-TSK-06/V09-01.md) |
| V10 | Existing test inventory and gaps | yes | failed -> missing regressions | [V10](../validations/A-TSK-06/V10-01.md) |
| V11 | Dependency/historical ownership and dedup | yes | passed | [V11](../validations/A-TSK-06/V11-01.md) |
| V12 | Dynamic restart/artifact/retry/budget fixtures | future | not_run by instruction | [V12](../validations/A-TSK-06/V12-01.md) |
| V13 | Report integrity, source isolation, and owned-session gate | yes | passed | [V13](../validations/A-TSK-06/V13-01.md) |
| V30 | Primary acceptance sampling | yes | passed | [V30](../validations/A-TSK-06/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| A-TSK-04: exact execution identity prevents stale result reuse | current | V08 confirms exact release/recovery identity; this task does not duplicate claim/recovery findings. |
| A-STATE-01: conversation transcript/store authority defects | current but out of scope | TaskRuntime result events are a separate file-backed projection; no conversation-store conclusion is reused. |
| Review module: NeedsFix creates and schedules a fix task | stale/regressed | V04 shows the helper constructs a same-ID task but the live executor discards it and only explicit unchanged retry is reachable. |
| Executor comment: dependency summaries provide full context including remaining work | regressed | V06 shows remaining work and several other structured fields are omitted and the aggregate is not bounded. |

## Coverage And Uncertainty

- Source-conclusive paths were reviewed statically; no build, test, fixture, or network command was run.
- Framework evidence is from committed blobs because its live worktree changed concurrently; those external edits were neither read nor classified.
- No provider-malformed thinking event fixture was run. Correctly typed thinking events are separated; a provider that mislabels reasoning as final content remains outside this task.
- Tool-output artifacts stored outside managed writer worktrees may remain durable; P1-03 is specifically the writer-result/public-projection path.
- `put_summary` failure is warning-only at dispatch, but the outer run-completion gate detects a missing structured result. Degraded same-run dependency context before that terminal failure remains covered by P1-01; no separate duplicate finding is created.

## Handoff

- Downstream work may rely on the positive immediate result map, typed thinking separation, and exact-execution restart reuse in V02/V07/V08.
- Fix order: define the single typed artifact and parent-projection contracts; preserve review issues on same-node retries; then remove obsolete fix-task/projection code.
- Preserve one `TaskRun -> PlanTask -> SubagentRun` authority. Do not create generated sibling fix tasks, a second artifact store, or another summary chain.
- This report becomes stale if SubagentOutcome fields, TaskExecutionSummary, review retry, worktree integration cleanup, artifact events, dependency prompt compilation, or execution identity changes.
- Status remains `needs_evidence` until Codex primary reviewer samples anchors and accepts the static evidence.
