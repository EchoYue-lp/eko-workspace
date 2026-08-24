# A-TSK-06: Task review, artifacts, and parent context

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only review)

## Question

Are complete Subagent results, checks, acceptance, artifacts, and
bounded parent summaries preserved without leaking thinking protocol?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/review.rs`
  (553 lines) — read in full: the review gate entry (`requires_review`,
  lines 49-55), the LLM verdict shape (`ReviewVerdict`,
  `ReviewIssueDraft`, 58-76), the gate itself (`review_task`, 82-143),
  the circuit breaker (`circuit_breaker_action`, 153-222), the
  fix-task builder (`build_fix_task`, 241-276), the prompt builders
  (`review_preamble`, `build_review_prompt`, 278-337), the strict
  outcome/severity parsers (339-354), the 6-test suite (365-552).
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/compact_context.rs`
  (728 lines) — read in full: the runtime-recovery capsule builder
  (`build_runtime_recovery_capsule`, 163-255), the per-task group
  renderer (`push_task_group`, 258-326), the structured-summary
  formatter (`format_summary`, 336-350), the protected-marker
  installer (`install_task_context_protection`, 157-160), the
  `PreModelContextProjector` impl (133-154), the run-scoped
  registry (`TaskRuntimeProjectionRegistry`, 53-124), and the 5-test
  suite (404-727).
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/memory_bridge.rs`
  (556 lines) — read in full: the `MemoryPolicy` enum
  (`None`/`FireAndForget`/`Blocking`, 39-58), the `MemoryEvent`
  variants (`RunCompleted`, `RepeatedTaskFailure`,
  `RunCancelledByUser`, `ReviewFoundIssue`, 64-83), the three
  dispatch helpers (89-151), the candidate builder
  (`build_candidates`, 198-286), the 8-test suite (289-555).
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/ledger.rs`
  (292 lines) — read in full: the progress markdown renderer
  (`render_progress`, 17-120), the export-path helper (122-128), the
  todos JSON exporter (130-134), the trace archiver
  (`archive_trace`, 138-154), the progress writer
  (`write_progress`, 164-185), the 3-test suite (187-291).
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs`
  (3496 lines) — read in relevant slices:
  - `SubagentReleaseRecord` (135-147) — the durable release payload
    including `result` and `full_output`;
  - `RecoverableSubagentResult` (110-114) — the read-side struct;
  - `add_review` (1403-1430), `add_artifact` (1432-1453),
    `put_summary` (1460-1478) — the three collection-write paths;
  - `list_artifacts` (1800-1804), `list_reviews` (1806-1817),
    `get_summary` (1819-1827) — the three collection-read paths;
  - `record_subagent_assigned` (1902-1930),
    `record_subagent_released` (1933-1970) — the durable boundary
    pair;
  - `recoverable_subagent_result` (2039-2078) — the fold that
    re-reads result + full_output from `SubagentReleased`;
  - `bounded_event_text` (2194-2200) — the 2 000-char summary cap;
  - `artifact_round_trip_preserves_path_and_metadata` (2236-2274)
    and `boot_recovery_reuses_completed_subagent_without_redispatch`
    (2738), `patched_spec_uses_new_execution_identity_without_retry_bump`
    (3148) — the three tests that prove the durability claims.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_store.rs`
  (1-327 + relevant tests) — read in full the public read API:
  - `list_artifacts` (225-255) — filter-fold on `ArtifactProduced`;
  - `list_reviews` (258-305) — filter-fold on
    `ReviewPassed`/`NeedsFix`/`Blocked`;
  - `get_summary` (308-326) — `rfind`-fold on
    `Note{kind=summary_persisted}`;
  - `fold_task_runtime` (345-400) — the per-task metadata fold
    (owner/started/completed/summary).
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_shadow.rs`
  (1-200, 360-436) — read in relevant slices:
  - `run_dir` / `events_path` / `plan_path` / `run_state_path`
    (87-101) — the on-disk layout `~/.eko/tasks/{run_id}/…`;
  - `default_root` (83-85) — `~/.eko/tasks/`;
  - `append_event_line` (118-178) — the append-only write primitive
    (audited in A-TSK-04 V03);
  - `append_line` (427-436) — `O_APPEND + sync_all`;
  - `atomic_write` (405-422) — uuid-tmp + fsync + rename.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs`
  (985-1031, 1415-1486, 1565-1699, 1700-1816) — read in relevant
  slices:
  - `PlanTask` schema (985-1031), specifically the
    `execution_checks` (1007) vs `acceptance_criteria` (1011)
    fields with their distinct doc comments (1003-1010);
  - `Artifact` (1415-1426) and `ArtifactKind` (563-605);
  - `SubagentTaskResult` (1572-1626) — `from_framework_outcome`,
    `terminal`, the `summary ≤ 1 200 chars` and `remaining_work ≤
    64 × 500 chars` bounds;
  - `SubagentRun` (1656-1697) — the in-memory + durable record
    whose `result` field carries the structured outcome;
  - `ReviewResult` / `ReviewIssue` / `IssueSeverity` (1700-1756);
  - `TaskExecutionSummary` (1761-1815) — the structured summary
    with `to_runtime_summary` framework-neutral projection.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs`
  (247 KB) — read in relevant slices:
  - the dispatch contract and `SubagentReleaseRecord` invocation
    (2300-2470) — both success and failure terminal branches;
  - `assess_task_execution` (665-765) — the hard-evidence gate;
  - the `Executed` arm that runs `run_review_gate` (1457-1530);
  - `run_review_gate` (1773-1836) — the LLM gate driver with
    retry and circuit-breaker integration;
  - `collect_dependency_summaries` (2598-2646) — the structured
    summary preference over truncated todo summary;
  - the main-agent streaming token/thinking split (3140-3260) —
    the `in_thinking` flag that excludes thinking from `output`;
  - the `review_gate_receives_complete_output_instead_of_bounded_summary`
    test (5288-5344) — the executable proof.
- `echo-agent-cli/echo-agent-app-core/src/subagent_prompt.rs`
  (741 lines) — read in full: the `EkoSubagentPromptCompiler`, the
  `PlannedTaskPrompt` schema, the `compile_planned_invocation`
  builder that renders the `[task_context]` block with separate
  sections for execution_checks / acceptance_criteria /
  required_artifacts / dependencies, and the 7-test suite.

## Out Of Scope

Deferred to downstream tasks:

- **A-TSK-05** (already complete): worktree, file ownership, merge
  policy. The worktree integration step (`integrate_reviewed_task`)
  is the seam this task stops at.
- **A-TSK-04** (already complete): claim identity, revision CAS,
  crash recovery. This task relies on the durable-event-fold
  invariant established there.
- **A-STATE-01** (already complete): conversation-deletion cascade.
  V03 of this task references the cascade's reach but does not
  re-audit the cascade itself.
- **A-FE-02**: frontend projection of task/subagent state — Rust
 -side persistence only is in scope here.
- **F-RCT-05**: runtime-state checkpoint resume — that is the
  full-agent-trajectory resume; this task only audits the
  task-runtime review/result path.

## Inputs

- Required repository documents read:
  - `AGENTS.md` (root) — "implementation 门禁 vs. acceptance checks"
    boundary; the framework-vs-application layering gate;
    "代码清理: 无需兼容, 过时代码可直接删" (relevant to V03's dead
    `retention` field); "本地个人助理" threat model (the basis for
    the retention finding's priority).
  - `docs/comprehensive-review/REPORTING.md`,
    `docs/comprehensive-review/templates/{task-report,validation-report}.md`,
    `docs/comprehensive-review/TASKS.md` (A-TSK-06 spec).
- Dependency task reports read:
  - **A-TSK-04** (complete) — established that `SubagentReleased`
    is the durable terminal fact, that
    `recoverable_subagent_result` is the read-side fold keyed on
    `(task_id, execution_id)`, and that recovery reuses the
    durable result without re-dispatch when the execution_id
    matches. This task relies on those invariants and answers the
    handoff item: "A-TSK-06 → may rely on the typed-status event
    mapping and the durable-result fold; the review gate consumes
    the `RuntimeTaskResolution::Blocked` outcome that the
    claim-guarded status write produces."
  - **A-STATE-01** (complete) — established that the
    `delete_conversation` cascade reaches the framework
    ConversationStore, the ToolExecutionRepository, the
    framework tool-output scope, and the user-input spill dir, but
    does NOT reach the task-runtime artifact tree. This task's V03
    confirms and extends that gap.
  - **A-TSK-03** (complete) — established that the controller is
    thin and that `run_review_gate` is one of the eight product-
    policy callbacks. This task confirms the review gate is the
    sole entry to the reviewer LLM.
  - **F-TSK-01** (complete) — the canonical framework `TaskSpec`
    model; this task confirms EKO's `execution_checks` /
    `acceptance_criteria` fields project losslessly to the
    framework's identically-named fields.
- Historical documents treated as hypotheses: the module docstrings
  on `review.rs:1-20`, `compact_context.rs:1-8`,
  `memory_bridge.rs:1-16`, `ledger.rs:1-8`,
  `types.rs:1003-1010` (the field-level docs that separate
  execution_checks from acceptance_criteria). All verified below.

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | The framework `TaskSpec` (echo-orchestration) defines `execution_checks`, `acceptance_criteria`, and `required_artifacts` as separate `Vec<String>` fields. EKO's `PlanTask` projects them losslessly (types.rs:1146-1148). The framework `SubagentResult`/`SubagentArtifact`/`SubagentVerification` are the structured-output contracts. EKO's `SubagentTaskResult::from_framework_outcome` is a lossless adapter (types.rs:1589-1602). The framework `MemoryLayerManager::write_memory` is the single memory-write chokepoint; EKO's `memory_bridge` is a thin candidate-builder over it. |
| EKO product policy | Confirmed app-owned: the file-backed review/summary/artifact/subagent event authorities on `events.jsonl`; the 6-state review-gate driver (`run_review_gate`, `requires_review`, `circuit_breaker_action`); the `MemoryPolicy` (None/FireAndForget/Blocking); the `MemoryEvent` taxonomy; the structured summary's extra fields (decisions / next_implications / suggested_tasks); the recovery capsule (`RUNTIME_RECOVERY_MARKER`); the ledger progress/trace export; the `ProfileTemplate`-driven review checklist; the bounded summary caps (2 000 chars on SubagentReleased, 1 200 chars on SubagentTaskResult.terminal, 260 chars on the capsule, 80 chars on each summary field). |
| Adapter boundary | `TaskExecutionSummary::to_runtime_summary` (types.rs:1779-1814) is a thin adapter to the framework's product-neutral `TaskExecutionSummary`. `SubagentTaskResult::from_framework_outcome` (types.rs:1589-1602) is a thin adapter from the framework's `SubagentOutcome`. `EkoRevisionedTaskStore` (revisioned_adapter.rs, audited in A-TSK-04) delegates persistence to the file store with no patch/validation logic. None of these adapters own authority over review/summary/result semantics. |
| Duplicate search | Searched names (whole `echo-agent-cli` repo): `add_review`, `list_reviews`, `add_artifact`, `list_artifacts`, `put_summary`, `get_summary`, `record_subagent_released`, `record_subagent_assigned`, `recoverable_subagent_result`, `requires_review`, `review_task`, `circuit_breaker_action`, `build_fix_task`, `build_review_prompt`, `RUNTIME_RECOVERY_MARKER`, `TASK_CONTEXT_MARKER`, `TaskRuntimeContextProjector`, `MemoryPolicy`, `MemoryEvent`, `write_memory_candidate`, `build_candidates`, `render_progress`, `archive_trace`, `write_progress`. Result: ONE definition per name; ZERO parallel review/summary/result/artifact authorities in `echo-agent-cli`. The framework `echo-orchestration::tasks` defines `TaskExecutionSummary` separately and EKO projects to it via `to_runtime_summary` (one-way). |
| Migration deletion | V03 identifies the artifact `metadata.retention` string field as effectively write-only (no reader). The `retention: "conversation_or_30d"` example in the round-trip test (store.rs:2259) has no implementation. Either implement the policy or remove the field from the test/example. |

## Current Path

Verified review/result/artifact/parent-context flow at commits
`echo-agent` 9b0e0fa / `echo-agent-cli` b3b2e81.

### Two-gate completion assessment

```text
Task dispatch completes
  → assess_task_execution(task, result)              [executor.rs:695]
      hard-evidence gate, NEVER judges acceptance_criteria
      status/summary/remaining_work/verification → ExecutionFailed (retryable)
      execution_checks lacking Observed+Passed      → AcceptancePending (block)
      required_artifacts lacking sha256+producer    → AcceptancePending (block)
      otherwise                                     → Executed

  → run_review_gate(store, reviewer_llm, run_id, task, subagent_output)
                                                     [executor.rs:1773]
      if !requires_review(task)                     → Pass
      if reviewer_llm is None                       → Skipped (block, no auto-pass)
      else review_task(llm, store, run_id, task, subagent_output).await
                                                     [review.rs:82]
        build_review_prompt(task, subagent_output, template)
          → Message::system(review_preamble(template))
          → Message::user(prompt with execution_checks AND acceptance_criteria)
        llm.chat(request) → ReviewVerdict JSON
        parse_outcome (strict: unknown → Blocked, never Pass)
        store.add_review(review)                     [store.rs:1403]
                                                     → ReviewPassed/NeedsFix/Blocked event
      match outcome:
        Pass          → integrate_reviewed_task → Completed
        NeedsFix      → circuit_breaker_action → CreateFix or Suspend
        Blocked       → Suspend
        Skipped       → block (no reviewer LLM)
```

The two gates are **strictly ordered** (hard evidence first,
reviewer second) and consult **disjoint fields**
(execution_checks + required_artifacts vs. acceptance_criteria).

### Result preservation (two projections, written together)

```text
execute_task terminal branch (success or failure)
  store.put_summary(TaskExecutionSummary{                  [executor.rs:2319 / 2413]
      run_id, task_id, subagent_name,
      result: SubagentTaskResult,
      decisions: vec![],                                   // populated by worktree integration
      next_implications: vec![],
      suggested_tasks: extracted_from_subagent_output,
      created_at,
  })                                                       [store.rs:1460]
    → append Note{kind: summary_persisted, summary}
    → rewrite_plan

  store.record_subagent_released(SubagentReleaseRecord{    [executor.rs:2331 / 2425]
      run_id, task_id, execution_id,
      agent_name, task_subject, plan_revision, attempt,
      status: result.status,
      result: Some(&result),
      full_output: Some(&full_output),                     // the post-thinking output
      dispatch_hook,
  })                                                       [store.rs:1933]
    → append SubagentReleased{ summary: bounded(summary, 2_000),
                               result, full_output, … }
```

The two writes are intentionally separate because they serve
different consumers:

- `recoverable_subagent_result` (store.rs:2039) reads
  `SubagentReleased` for resume reuse — needs `result` +
  `full_output`.
- `FileTaskStore::get_summary` (file_store.rs:308) reads
  `Note{summary_persisted}` for cross-task context — needs
  `decisions` / `next_implications` / `suggested_tasks`, which the
  SubagentReleased payload does not carry.

### Read-side folds (deterministic over the event stream)

```text
recoverable_subagent_result(run_id, task_id, execution_id)  [store.rs:2039]
  iterate list_events(run_id) in seq order:
    skip events whose task_id != task_id || step_id != execution_id
    SubagentAssigned → result = None (clear)
    SubagentReleased{status=completed} →
        result = event.payload.result + event.payload.full_output
  return Option<RecoverableSubagentResult>

FileTaskStore::get_summary(run_id, task_id)                [file_store.rs:308]
  iterate read_events(run_id):
    rfind event where event_type == Note
                    && task_id matches
                    && payload.kind == "summary_persisted"
  return event.payload.summary as TaskExecutionSummary

FileTaskStore::list_artifacts(run_id)                      [file_store.rs:225]
  filter events where event_type == ArtifactProduced
  map each payload → Artifact

FileTaskStore::list_reviews(run_id)                        [file_store.rs:258]
  filter events where event_type in {ReviewPassed, ReviewNeedsFix, ReviewBlocked}
  map each payload → ReviewResult
```

### Parent-context bounded summary

```text
collect_dependency_summaries(store, run_id, task)          [executor.rs:2598]
  for each dep_id in task.depends_on:
    find todo where task_id == dep_id && status == Completed
    structured = store.get_summary(run_id, &todo.task_id)
    if structured:
      parts = [summary, modified_files, decisions] joined as "label: value"
    else fallback to todo.summary (truncated text)
  return Vec<(title, summary_text)> bounded by depends_on.len()

EkoPromptPayload::planned_task(task, dep_summaries, …)     [subagent_prompt.rs:123]
  → PlannedTaskPrompt{ dependency_summaries: Vec<DependencyPrompt>, … }

compile_planned_invocation(task, history)                  [subagent_prompt.rs:306]
  renders [task_context] block:
    Domain profile: {key} ({label})
    Execution standard: {execution_guidance}
    User goal: {user_goal}                                  // bounded by run.goal
    Task: {title}\n\n{description}
    Context from completed upstream tasks:                  // bounded by 3 items
      - {dep.title}: {dep.summary}
    Declared write targets: {files}
    Execution checks: {execution_checks}                    // distinct section
    Acceptance criteria: {acceptance_criteria}              // distinct section
    Required artifacts: {required_artifacts}                // distinct section
    Execution boundary: {task_boundary}
    Delegation: {enabled|disabled}
  no parent reasoning, no parent thinking, no parent message history
  (transfer_policy == Fresh; history is empty)
```

### Runtime recovery capsule (compression-safe)

```text
TaskRuntimeContextProjector::project(context)              [compact_context.rs:133]
  derive run_id from context.run_id or context.turn_id
  look up store via TaskRuntimeProjectionRegistry (process-stable)
  if store and run_id present:
    build_runtime_recovery_capsule(store, run_id)          [compact_context.rs:163]
      → ContextProjection{ marker: RUNTIME_RECOVERY_MARKER,
                            message: Message::user(capsule_text) }
  else ContextProjection{ marker, message: None }
```

The capsule is bounded by:

- Goal: 420 chars (`MAX_GOAL_CHARS`);
- Task title: 96 chars;
- Task description: 220 chars;
- Summary: 260 chars;
- Items per field: 3 (`MAX_ITEMS_PER_FIELD`);
- Per-group task limits: 4 running, 4 blocked/failed, 6 pending,
  5 completed.

`install_task_context_protection` (compact_context.rs:157-160)
adds `TASK_CONTEXT_MARKER` as a `replaceable_protected_marker`,
so the latest task brief survives mid-task compression but stale
briefs are replaced.

### Thinking-protocol exclusion

```text
Main-agent streaming dispatch                          [executor.rs:3140-3260]
  let mut output = String::new();
  let mut in_thinking = false;
  while let Some(event) = stream.next().await {
      match event {
          AgentEvent::Token(content) =>
              if in_thinking {
                  emit ThinkingDelta realtime trace event ONLY
              } else {
                  output.push_str(&content);              // ← only here
                  emit TokenDelta realtime trace event
              }
          AgentEvent::ThinkStart => in_thinking = true
          AgentEvent::ThinkEnd { .. } => in_thinking = false
          …
      }
  }
  // output → full_output → record_subagent_released(full_output = Some(&output))
```

The thinking stream is routed only to the realtime trace sink
(`ExecEvent::subagent(... ThinkingDelta ...)`), which is not
persisted by `record_subagent_released`. The `output` buffer that
becomes the durable `full_output` therefore excludes every thinking
token by construction.

### Memory bridge (best-effort, never blocks)

```text
Terminal run event                                       [memory_bridge.rs:89-151]
  → MemoryEvent::{RunCompleted|RepeatedTaskFailure|RunCancelledByUser|ReviewFoundIssue}
  → write_memory_candidate_dispatch(policy, layer_manager, store, event)
      None          → no write
      FireAndForget → tokio::spawn(write_memory_candidate_inner)
      Blocking      → await write_memory_candidate_inner
  build_candidates(store, event)                          [memory_bridge.rs:198]
    RunCompleted → fold completed todos (title + summary) into content
                   key: taskrun:completed:{run_id}
                   type: ArchitectureDecision, source: AutoExtracted
    RepeatedTaskFailure → key: taskrun:failure:{fingerprint}
                          type: DebuggingLesson, source: ErrorResolution
    RunCancelledByUser → key: taskrun:cancelled:{run_id}
                         type: UserPreference, source: UserCorrection
    ReviewFoundIssue → key: taskrun:review_issue:{category}
                       type: DebuggingLesson, source: AutoExtracted
  MemoryLayerManager::write_memory(key, content, meta)    [framework]
```

The bridge consumes the structured `TaskExecutionSummary` (via
`store.list_todos` + `todo.summary`) — not raw thinking. Best-effort
errors are logged and swallowed.

Invariants verified by this graph (full evidence in V01-V04):

- **Two-gate separation holds.** `assess_task_execution` never reads
  `acceptance_criteria`; `review_task` reads them via the prompt.
  V02.
- **Full result + full_output preserved on SubagentReleased.**
  V01.
- **Structured summary preserved on Note{summary_persisted}.**
  V01.
- **Thinking excluded by construction.** V01.
- **Review prompt deterministic in (task, output, template).**
  V04.
- **Recovery reuses durable result without re-dispatch.** V04,
  cross-references A-TSK-04 V01/V04.
- **No second authority for review/summary/result/artifact.** V01
  duplicate search.
- **No cleanup path for the task-runtime tree.** V03 (finding).
- **`metadata.retention` is a write-only string.** V03 (finding).
- **`archive_trace` duplicates `full_output` and writes to an
  unreliable CWD-derived path.** V03 (finding).

## Findings

The headline result is mixed: the review/result/parent-context
design is sound and well-bounded, but the retention side is
unfinished. One P2 retention gap and two P3 hardening items are
recorded; none affect correctness on the happy path.

### A-TSK-06-P2-01: No cleanup cascade for the task-runtime artifact tree

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_shadow.rs:82-101`
    — the on-disk layout is `~/.eko/tasks/{run_id}/{events.jsonl,
    plan.json, run-state.json}`. `default_root` returns
    `echo_agent::paths::user_data_path("tasks")` =
    `~/.eko/tasks/`.
  - Whole-repo grep for `delete_run|purge_run|remove_run|cleanup_runtime|cleanup_task_runtime|task_runtime.*delete`
    in `echo-agent-cli/echo-agent-app-core/src`,
    `echo-agent-cli/src/tauri`, `echo-agent-cli/src/tui` returns
    ZERO matches (excluding test fixtures and unrelated uses).
  - `echo-agent-cli/src/tauri/commands/conversations.rs:586-640`
    (`delete_conversation`) calls
    `echo_agent::tools::artifact::cleanup_tool_output_scope`,
    `tool_executions.remove_conversation`, and
    `prepared_turn::cleanup_user_input_scope` — none of which
    touch `~/.eko/tasks/`.
  - `echo-agent-cli/src/tui/events.rs:3067-3102` (TUI
    `/delete-session`) reaches the same three cleanup helpers and
    is missing `tool_executions.remove_conversation` (A-STATE-01
    P2-02); neither surface reaches the task-runtime tree.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs`
    has no `delete_run`, no `delete_runs_for_conversation`, no
    `purge_runtime_scope`. The `TaskRuntimeStore` API is
    write/read-only.
- Reachability: every complex run produces a `~/.eko/tasks/{run_id}/`
  directory that is never removed. The ledger's
  `{base}/.eko/runtime/{run_id}/` is likewise never removed. Both
  grow unbounded with usage.
- Expected invariant: per AGENTS.md "代码清理" and the
  local-assistant threat model, deleted conversations should not
  leave orphaned multi-MB artifact trees. The conversation-deletion
  cascade should reach every artifact scoped to that conversation.
- Observed behavior: a deleted conversation leaves the
  `~/.eko/tasks/{run_id}/` events/plan/run-state files and the
  `~/.eko/runtime/{run_id}/` progress/trace files indefinitely.
  Repeated create/delete cycles accumulate ghost runs the user
  cannot see or clean from any surface.
- Impact: medium. Disk growth is the main concern; for a user who
  runs many complex tasks and periodically deletes conversations,
  the orphaned artifacts can accumulate to GB-scale over months.
  There is also a privacy angle (sensitive content persists after
  the user believes they deleted the conversation) — for the
  local-assistant threat model this is a robustness gap, not a
  multi-user leak.
- Root cause: the task-runtime artifact tree was added (U1c
  migration, see file_shadow.rs module doc) without a corresponding
  cleanup hook. The conversation-deletion cascade predates it and
  was never extended.
- Direction: add a `TaskRuntimeStore::delete_runs_for_conversation(conv_id)`
  that enumerates runs by `conversation_id` (the field exists on
  `TaskRun`) and removes their `~/.eko/tasks/{run_id}/` directories;
  wire it into both `delete_conversation` (Tauri) and
  `/delete-session` (TUI). The same cascade should also remove the
  ledger's `{base}/.eko/runtime/{run_id}/` (currently the ledger
  has no delete API either — add one or fold it into the same
  helper). Coordinate with A-STATE-01 P2-02's recommendation to
  unify the Tauri/TUI cascade.
- Regression validation: a test that creates a run, completes a
  task, deletes the owning conversation, and asserts both
  `~/.eko/tasks/{run_id}/` and `~/.eko/runtime/{run_id}/` are gone,
  and `store.get_run(run_id)` returns `None`.
- Validation reports: [V03-01](../validations/A-TSK-06/V03-01.md)

### A-TSK-06-P3-01: `archive_trace` duplicates `full_output` and writes to an unreliable path

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/ledger.rs:138-154`
    — `archive_trace` writes the full subagent output via
    `std::fs::write(&path, output)` to
    `{base}/.eko/runtime/{run_id}/artifacts/traces/{task_id}.txt`.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:2317`
    — the production call site passes `base = None`, which
    `archive_trace` interprets as "use CWD" (ledger.rs:140-141).
    For Tauri the CWD is not reliable (A-STATE-01 made the same
    observation about `write_progress`'s `base` parameter).
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:2341`
    — the same `full_output` is also persisted as the
    `SubagentReleased.full_output` field on `events.jsonl`
    (store.rs:1965). The trace archive is therefore a duplicate
    store of the same content.
- Reachability: every successful task dispatch writes the trace
  archive once. Writer tasks with multi-MB outputs double their
  disk footprint.
- Expected invariant: derived/debug artifacts should not duplicate
  canonical state, and their write location should be deterministic.
- Observed behavior: the trace archive duplicates
  `SubagentReleased.full_output` and lands in a CWD-dependent path
  for Tauri runs. The ledger doc (ledger.rs:1-8) acknowledges the
  progress.md is "a derived recovery view", but the trace archive
  is a separate artifact with a separate (unreliable) location.
- Impact: low for correctness (the canonical `full_output` on
  `events.jsonl` is intact), medium for disk growth on writer-heavy
  runs and medium for debuggability (the trace file location is
  unpredictable in Tauri).
- Root cause: `archive_trace` predates the durable
  `SubagentReleased.full_output` field. When the latter was added,
  the former was not removed.
- Direction: either (a) delete `archive_trace` and rely on
  `SubagentReleased.full_output` as the single source (preferred —
  AGENTS.md "delete over retain"), or (b) keep it but pass an
  explicit `base` from the caller (the workspace root) and document
  that it is a debug-only duplicate. Option (a) also removes the
  CWD-dependency problem.
- Regression validation: a test that confirms
  `recoverable_subagent_result(run_id, task_id, execution_id).full_output`
  is non-empty and matches the original output (already covered by
  `boot_recovery_reuses_completed_subagent_without_redispatch`).
- Validation reports: [V03-01](../validations/A-TSK-06/V03-01.md)

### A-TSK-06-P3-02: Artifact `metadata.retention` field is write-only (no enforcer)

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:2257-2261`
    — the `artifact_round_trip_preserves_path_and_metadata` test
    writes `"retention": "conversation_or_30d"` in `metadata`.
  - Whole-repo grep for `conversation_or_30d` returns exactly one
    match (the test). Whole-repo grep for `retention` returns hits
    in this test, the `Artifact` metadata field doc, and unrelated
    contexts. There is no reader that interprets the `retention`
    key, no TTL enforcer, no scheduler that consumes it.
- Reachability: none — the field is written but never read.
- Expected invariant: a documented retention policy should be
  enforced, or the field should be removed (AGENTS.md "code
  cleanup").
- Observed behavior: the `retention` metadata field is a
  stringly-typed wish with no implementation. A new contributor
  reading the test will reasonably assume retention is enforced.
- Impact: low (no correctness risk), but the dead field is
  misleading documentation.
- Root cause: the retention concept was sketched (the test fixture
  names a policy) but never implemented, and the cleanup cascade
  that would have enforced it (P2-01) is also missing.
- Direction: either (a) remove `retention` from the test fixture
  and document that artifact retention is currently "live until the
  owning run is deleted" (which is itself unimplemented — P2-01),
  or (b) implement the policy as part of P2-01's cascade (e.g.
  `delete_runs_for_conversation` removes the artifacts; a future
  TTL sweeper can extend this). Option (a) is the smaller change
  and avoids implying a contract that does not exist.
- Regression validation: the `artifact_round_trip_preserves_path_and_metadata`
  test still passes after the fixture is updated.
- Validation reports: [V03-01](../validations/A-TSK-06/V03-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Two-projection result preservation (full-result on SubagentReleased; structured summary on Note{summary_persisted}); thinking protocol excluded by construction; no parallel authority | yes | passed | [V01-01](../validations/A-TSK-06/V01-01.md) |
| V02 | Acceptance/check separation: distinct fields, distinct gates (hard evidence first, reviewer second), distinct prompt sections | yes | passed | [V02-01](../validations/A-TSK-06/V02-01.md) |
| V03 | Artifact retention: documented policy + reachable cleanup path | yes | failed | [V03-01](../validations/A-TSK-06/V03-01.md) |
| V04 | Restart-equivalent review input: prompt deterministic, durable full_output reused on resume | yes | passed | [V04-01](../validations/A-TSK-06/V04-01.md) |
| V05 | Historical-document drift | conditional (applicable — five code/module comments treated as hypotheses; classifications inline in Historical Claim Status) | passed | classified inline |

Executed cargo commands (all exit 0):

```text
cd echo-agent-cli && cargo test -p echo-agent-app-core --lib tasks::task_runtime::review
  → 6 passed; 0 failed; 0 ignored (0.06s)
cd echo-agent-cli && cargo test -p echo-agent-app-core --lib tasks::task_runtime::compact_context
  → 5 passed; 0 failed; 0 ignored (0.15s)
cd echo-agent-cli && cargo test -p echo-agent-app-core --lib tasks::task_runtime::memory_bridge
  → 8 passed; 0 failed; 0 ignored (0.26s)
cd echo-agent-cli && cargo test -p echo-agent-app-core --lib tasks::task_runtime::ledger
  → 3 passed; 0 failed; 0 ignored (0.10s)
cd echo-agent-cli && cargo test -p echo-agent-app-core --lib tasks::task_runtime::file_store
  → 5 passed; 0 failed; 0 ignored (0.16s)
cd echo-agent-cli && cargo test -p echo-agent-app-core --lib subagent_prompt
  → 7 passed; 0 failed; 0 ignored (0.01s)
cd echo-agent-cli && cargo test -p echo-agent-app-core --lib \
  tasks::task_runtime::executor::tests::review_gate_receives_complete_output_instead_of_bounded_summary
  → 1 passed; 0 failed; 0 ignored (0.09s)
```

The full `echo-agent-cli` pre-commit matrix was not re-run because
this review is read-only; the seven targeted subsets above are the
directly relevant evidence — they exercise the review-gate /
compact-context / memory-bridge / ledger / file-store /
subagent-prompt paths audited in this task.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `review.rs:1-20` module doc: "Every implementation/debugging task must pass a review before downstream tasks continue … decides one of Pass / NeedsFix / Blocked." | current | Verified by V02: `requires_review` triggers on `Implementation`/`Debugging` and on any non-empty `acceptance_criteria`; the three outcomes map to distinct event kinds and distinct executor arms. |
| `review.rs:14-19` "Circuit breaker (plan §810-840): if the same task hits `max_retries`, or the same `failure_fingerprint` repeats, or the same review issue class repeats …" | current | Verified by `circuit_breaker_action` (review.rs:153-222): all three rules implemented. The `same_failure_threshold = 2` default is hard-coded at the call site (executor.rs:1825). |
| `compact_context.rs:1-8` "Compression-safe TaskRuntime context capsules … When the main-agent context is prepared or compacted mid-task, however, the LLM still needs a concise recovery view of the active run." | current | Verified by V01/V04: the capsule is built deterministically from the file-backed store at every `PreModelContextProjector::project` call; the bounds (MAX_GOAL_CHARS=420, etc.) make it compression-safe. |
| `memory_bridge.rs:1-16` "all long-term memory writes must go through the single chokepoint `MemoryLayerManager::write_memory` … there is exactly one write path, not a parallel one." | current | Verified by V01 duplicate search: one `MemoryLayerManager::write_memory` call site (memory_bridge.rs:165); the bridge is the only consumer of `MemoryEvent`s. |
| `ledger.rs:1-8` "Generates a human-readable `progress.md` from canonical TaskRuntime files. The markdown export is a derived recovery view; run events and plan files remain authoritative." | current | Verified by V01/V03: `render_progress` is a pure function over the store; `write_progress` and `archive_trace` are best-effort writes whose failure is logged but does not fail the call. |
| `ledger.rs:136-137` "Archive raw subagent output as a trace artifact (plan §1057-1061). Writes to `{base}/.eko/runtime/{run_id}/artifacts/traces/{task_id}.txt`." | current-with-caveat | The function exists and is reachable (executor.rs:2317). The caveat is A-TSK-06-P3-01: it duplicates `SubagentReleased.full_output` and writes to a CWD-derived path. |
| `types.rs:1003-1010` field-level docs separating `execution_checks` (commands, observed+passed) from `acceptance_criteria` (prose, reviewer-judged) | current | Verified by V02: the two fields are separate at the schema, gate, and prompt layers. |
| `executor.rs:686-689` "Acceptance criteria are intentionally NOT judged here — they are reviewer-judged in the ReviewGate, never auto-passed." | current | Verified by V02: `assess_task_execution` reads only `execution_checks` and `required_artifacts`; the reviewer gate is the only consumer of `acceptance_criteria`. |
| `subagent_prompt.rs:18-23` `COMMON_ORCHESTRATION_POLICY`: "Do not create, modify, approve, or execute the parent TaskRuntime plan." | current | Verified by V01: the planned-task prompt payload carries only the assigned task's fields, never the parent plan or other tasks. The subagent cannot see or modify the plan. |
| `store.rs:1455-1459` `put_summary` doc: "Persist or overwrite the per-task execution summary. Primary key is `(run_id, task_id)` so a re-execution replaces the prior summary." | current | Verified by V01: `FileTaskStore::get_summary` uses `rfind`, so the last `Note{summary_persisted}` for a task wins; a re-execution cleanly replaces. |
| `store.rs:2036-2038` `recoverable_subagent_result` doc: "A later `SubagentAssigned` with the same id clears an older terminal fact, which is how an explicitly confirmed retry avoids reusing stale output." | current | Re-verified by V04 (cross-ref A-TSK-04 V01): the fold clears on `SubagentAssigned`. |
| A-TSK-04 handoff: "A-TSK-06 → may rely on the typed-status event mapping and the durable-result fold; the review gate consumes the `RuntimeTaskResolution::Blocked` outcome that the claim-guarded status write produces." | resolved (verified) | V01/V04 confirm the review gate produces `RuntimeTaskResolution::Blocked` via the claim-guarded `set_claimed_status` (executor.rs:1503-1515); the durable-result fold is sound (A-TSK-04 V01). |
| A-STATE-01 handoff: "the conversation-deletion cascade reaches the framework ConversationStore, the ToolExecutionRepository, the framework tool-output scope, and the user-input spill dir, but does NOT reach the task-runtime artifact tree." | resolved (verified, extended) | V03 confirms: zero cleanup paths under `~/.eko/tasks/` or `~/.eko/runtime/`. Filed as A-TSK-06-P2-01. |

## Coverage And Uncertainty

- **Inspected in full:** the entire review gate (`review.rs`), the
  entire compact-context module (`compact_context.rs`), the entire
  memory bridge (`memory_bridge.rs`), the entire ledger
  (`ledger.rs`), the entire file-store read API
  (`file_store.rs:1-327`), the entire subagent-prompt compiler
  (`subagent_prompt.rs`), the entire result/summary/artifact/review
  surface of `TaskRuntimeStore`, the entire result-preservation
  surface of `executor.rs` (the dispatch terminal branches, the
  hard-evidence gate, the reviewer gate, the streaming token/thinking
  split, the dependency-summary collector).
- **Inspected partially:** the 247-K `executor.rs` was read in the
  slices cited above; the dispatch-to-Subagent pipeline
  (`run_readonly_subagent`, `run_writer_subagent`,
  `run_main_agent_task`) was sampled only at the result-collection
  and terminal-persistence seams. Their internal mechanics are
  A-TSK-03 (controller boundary) and A-TSK-05 (worktree) territory.
- **Not inspected (out of scope):**
  - The full `echo-agent-cli` pre-commit matrix (fmt / clippy /
    all-features test). The review is read-only; the seven targeted
    subsets above are the directly relevant evidence.
  - The framework `echo-orchestration::tasks::TaskExecutionSummary`
    semantics (F-TSK-01 territory). This task only verified the
    adapter `to_runtime_summary` is lossless for the fields EKO
    uses.
  - The frontend rendering of review/summary/artifact state
    (A-FE-02 territory).
- **Uncertain claims:**
  - The exact disk-growth rate of P2-01 is hard to bound without
    measurement. It depends on user behavior (how many complex
    runs, how large the outputs, how often conversations are
    deleted). Filed as P2 because the gap is structural (no
    cleanup path exists at all), not because there is evidence of
    imminent disk exhaustion.
  - Whether any out-of-repo plugin consumes the `metadata.retention`
    field is unknowable from this repo. If it does, P3-02's
    "remove the field" recommendation would break it; the safer
    fallback is to leave the field and document that it is
    aspirational.

## Handoff

- **Conclusions downstream tasks may rely on:**
  - The two-gate completion assessment is sound: hard evidence
    (execution_checks + required_artifacts) is judged by
    `assess_task_execution`; acceptance_criteria are judged by the
    reviewer LLM. The two are never conflated. (V02)
  - Subagent results are preserved in two durable projections:
    full-result on `SubagentReleased`, structured summary on
    `Note{summary_persisted}`. Both are deterministic folds over
    the same `events.jsonl`. (V01)
  - Thinking protocol is excluded from durable artifacts by
    construction (the `in_thinking` flag in the streaming loop).
    (V01)
  - The review gate sees the FULL subagent output (not the bounded
    summary), and the prompt is deterministic in
    `(task, output, template)`. After a crash, the same inputs
    reproduce the same prompt. (V04)
  - The durable-result fold reuses output only for the exact
    `execution_id`; a plan revision invalidates it (intended).
    (V04, cross-refs A-TSK-04 V01)
  - Parent-context summaries are bounded (char limits + item
    limits) and compression-safe (replaceable protected marker
    for the latest task brief; deterministic capsule rebuild).
    (V01)
- **Reports downstream tasks must read:**
  - [V01-01](../validations/A-TSK-06/V01-01.md) for the two-projection
    result-preservation map and the thinking-exclusion proof.
  - [V02-01](../validations/A-TSK-06/V02-01.md) for the gate-by-gate
    separation of execution_checks vs. acceptance_criteria.
  - [V03-01](../validations/A-TSK-06/V03-01.md) for the retention
    gap and the duplicate-trace finding.
  - [V04-01](../validations/A-TSK-06/V04-01.md) for the
    restart-equivalent review-input reconstruction path.
- **Task-to-reference mapping:**
  - **X-TSK-01** (task graph and adapter conformance) → may rely on
    the review/summary/result preservation invariants established
    here; the field round-trip from EKO `PlanTask` to framework
    `TaskSpec` is lossless for `execution_checks` and
    `acceptance_criteria`.
  - **X-STA-01** (persistence/recovery/identity continuity) → must
    incorporate A-TSK-06-P2-01 (no cleanup cascade) into its
    retention-and-deletion-cascade audit.
  - **A-FE-02** (frontend projections) → may rely on the
    ReviewResult / TaskExecutionSummary / Artifact schemas being
    stable and round-trip-safe across the Tauri DTO boundary.
  - **A-OUT-01** (output formats / export) → may rely on the
    ledger's `render_progress` being a pure function over the
    store; the export path is deterministic.
  - **Q-PERF-01** (performance and resource lifecycle) → must
    incorporate A-TSK-06-P2-01 (unbounded task-runtime artifact
    growth) and A-TSK-06-P3-01 (duplicate trace archive) into its
    disk-growth analysis.
- **Conditions that make this report stale:**
  - Any commit that adds a `delete_run` / cleanup cascade for
    `~/.eko/tasks/` invalidates A-TSK-06-P2-01.
  - Any commit that removes `archive_trace` (P3-01 option a)
    invalidates that finding.
  - Any commit that adds a reader for `metadata.retention`
    invalidates A-TSK-06-P3-02.
  - Any commit that merges `execution_checks` and
    `acceptance_criteria` into a single field invalidates V02.
  - Any commit that routes thinking tokens into the `output`
    buffer (e.g. removing the `in_thinking` gate) invalidates V01.
- **Follow-up task IDs (no fixes implemented in this review):**
  - A cleanup-focused task should pick up A-TSK-06-P2-01 (extend
    the conversation-deletion cascade to the task-runtime tree)
    in coordination with A-STATE-01 P2-02 (TUI/GUI cascade parity).
  - A small cleanup task should pick up A-TSK-06-P3-01 (delete
    `archive_trace` or pass an explicit base) and A-TSK-06-P3-02
    (remove or implement `metadata.retention`).
