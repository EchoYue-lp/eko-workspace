# A-TSK-06: Task review, artifacts, and parent context

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean (both source repositories)

## Question

Are complete Subagent results, checks, acceptance, artifacts, and bounded
parent summaries preserved without leaking thinking protocol?

**Answer: Yes in the core invariants, with three dead/unbounded gaps. The
complete Subagent output (not the bounded summary) is what the review gate
consumes, the same output is persisted on the `SubagentReleased` terminal
boundary and reused verbatim after restart, execution checks and required
artifacts are gated on hard observed evidence while acceptance criteria are
judged only by the reviewer LLM, and the thinking trace is excluded from
`output` at both dispatch paths and never persisted — no thinking content
reaches the reviewer, parent summaries, capsule, memory, or hooks. However:
(P2) the runtime `Artifact` / `ArtifactProduced` / `list_artifacts`
projection has zero production writers, so the GUI run-level artifact list is
permanently empty; (P3) `MemoryEvent::ReviewFoundIssue`/`RepeatedTaskFailure`
are dead arms so the documented review-issue / repeated-failure memories are
never written; (P3) `full_output` is persisted unbounded in `events.jsonl`
and trace archives are written CWD-relative; (P3) summary-chain and todo
summaries have loose aggregate bounds (per-item bounds only).**

## Scope

Primary source paths inspected (deep read):

- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/review.rs`
  (full): `requires_review` (:49-55), `review_task` (:82-143),
  `circuit_breaker_action` (:153-222), `build_fix_task` (:241-276),
  `build_review_prompt` (:298-337).
- `compact_context.rs` (full): registry (:53-131), `PreModelContextProjector`
  impl (:133-154), `build_runtime_recovery_capsule` (:163-255),
  `push_task_group`/`format_summary`/`truncate_chars` (:257-402).
- `memory_bridge.rs` (full): `MemoryEvent` (:64-83), dispatch helpers
  (:89-151), `build_candidates` (:198-286).
- `types.rs` (:530-786 event/review/artifact enums; :960-1160 `PlanTask`
  spec/execution projection; :1415-1426 `Artifact`; :1476-1626
  `SubagentArtifactResult`/`SubagentVerificationResult`/`SubagentTaskResult`;
  :1654-1815 `SubagentRun`/`ReviewResult`/`TaskExecutionSummary`).
- `executor.rs` slices: `run_completion_blockers` (:587-641),
  `assess_task_execution` (:663-790), `recoverable_subagent_result` reuse
  (:1293-1313), `resolve_dispatch` review/acceptance branches (:1348-1562),
  `run_review_gate` (:1773-1836), `execute_task` summary/result/release
  (:2073-2440), `collect_dependency_summaries` (:2598-2646),
  `run_readonly_subagent`/`run_writer_subagent` (:2798-2978),
  `run_main_agent_task` thinking/verification/artifact observation
  (:3049-3494).
- `store.rs` slices: `add_artifact` (:1432-1447), `put_summary` (:1460),
  `record_subagent_released` (:1933-1970), `recoverable_subagent_result`
  (:2039-2078), `set_claimed_task_status` (:1032-1065), `bounded_event_text`
  (:2194-2200).
- `file_store.rs` (:225-260 `list_artifacts`/`list_reviews` event derivation),
  `task_execute_tool.rs` (:440-555 outcome text + spill, :598-700
  `format_execution_summary`), `hook_event_dispatcher.rs` (:317-352
  `SubagentReleased` translation), `ledger.rs` (:15-154 `render_progress`/
  `archive_trace`), `subagent_prompt.rs` (:123-210, :306-380
  `compile_planned_invocation`), `task_tools.rs` (:333 task-mode brief),
  `run_driver.rs` (:110-150), `infra.rs` (:440-470, :675-690, :810-825),
  `chat_driver.rs` (:240-265, :895-935).
- Tauri/前端投影: `src/tauri/commands/task_runtime.rs` (:115-140),
  `web-frontend/src/stores/taskRuntimeStore.ts` (:40-70),
  `web-frontend/src/api/endpoints.ts` (:553).
- Framework: `echo-agent/src/agent/subagent/types.rs` (:245-250 constants,
  :300-345, :386-547 `parse_subagent_outcome`/bounds, :600-713
  `SubagentResult`), `subagent/executor.rs` (:229-254
  `merge_observed_evidence`, :1197-1252 thinking handling),
  `subagent/prompt.rs` (:161-163 reasoning drop), `subagent/events.rs`
  (:114-143), `echo-orchestration/src/tasks/runtime.rs` (:189-190,
  :309-340 `Summarize`), `runtime_executor.rs` (:82 controller contract).

## Out Of Scope

- Claims/revisions/recovery/replay — A-TSK-04 (complete); the `SubagentReleased`
  persistence is consumed here only for the review-input equivalence fixture.
- Conversation persistence/restore — A-STATE-01 (complete).
- File authorities / adapter losslessness / event rebuild — A-TSK-01.
- Controller boundary policy (pause/cancel ownership) — A-TSK-03.
- Frontend projections of task/Subagent results — A-FE-02; tool-output
  artifact infrastructure (spill/read_artifact) — F-EXT-01/A-TOOL-01.
- Framework Subagent execution modes/teams — F-SUB-02; compression semantics
  of the protected-marker machinery — F-CMP-01.

## Inputs

- Root `AGENTS.md` (UTF-8/panic safety; framework-vs-app layering; review
  strategy is EKO product policy; one task-relation authority).
- Shared `README.md`, `REPORTING.md`, `TASKS.md` (A-TSK-06 card),
  `zcode-ds/README.md`, report templates.
- Dependency task reports read: `A-TSK-04` (complete; claim/ledger facts,
  `recoverable_subagent_result` anchors, torn-tail blocker), `A-STATE-01`
  (complete; conversation/artifact retention context).
- Historical documents treated as hypotheses: `echo-agent-cli/docs/
  MASTER-PLAN.md:55-70,155-215`, `echo-agent-cli/docs/2026-07-17-subagent-
  results-and-completion.md:20-45,90-160`, `echo-agent-cli/docs/2026-07-27-
  runtime-dag-kernel-convergence.md:80-120,148-165`, `docs/MASTER-PLAN.md:
  80-115,190-250` (classified in V05-01).

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism (framework, correct) | `SubagentResult`/`SubagentOutcome` (output, summary, verification with Observed/Reported source, artifacts with hash+producer id, touched_files); `SubagentVerification`/`SubagentArtifact`; `PreModelContextProjector`/`ContextProjection`/protected markers; `TaskSpec` carries `execution_checks`/`acceptance_criteria`/`required_artifacts` as data; framework `Summarize` trait (unused by EKO — retained generic API, not a duplicate authority). `was_truncated` is always `false` in production construction (subagent/executor.rs:1102,:1437), so review input is never silently truncated. |
| EKO product policy (application, correct) | Review gate (`review.rs`) incl. circuit breaker and fix-task policy; acceptance/check/artifact gate (`assess_task_execution`) and `run_completion_blockers`; full-output persistence on `SubagentReleased` + restart reuse; compact-context capsule and registry; summary chain (`collect_dependency_summaries` + prompt payload); memory bridge candidates/policies; spill of the final `task_execute` result to a complete artifact with bounded pointer. |
| Adapter boundary | `SubagentTaskResult::from_framework(outcome)` (types.rs:1584-1626) and `TaskExecutionSummary::to_runtime_summary` (:1777-1814) are thin lossless projections; `record_subagent_released`/`recoverable_subagent_result` persist/restore the framework output verbatim at the terminal boundary; the framework executor routes thinking tokens to the event bus only (never into `output`). No second ready-frontier, validator, or review loop in the adapter. |
| Duplicate search (V01-01) | Terms searched: `review_task`, `requires_review`, `circuit_breaker_action`, `build_fix_task`, `run_review_gate`, `ReviewResult`, `reviewer_llm`, `list_reviews`, `LlmCritic`, `set_critic`, `verifier`, `acceptance_criteria`, `execution_checks`, `required_artifacts`, `assess_task_execution`, `SubagentVerification`, `Observed`, `TaskExecutionSummary`, `put_summary`, `get_summary`, `collect_dependency_summaries`, `format_execution_summary`, `Summarize`, `summary_chain`, `TaskRuntimeContextProjector`, `PreModelContextProjector`, `RUNTIME_RECOVERY_MARKER`, `TASK_CONTEXT_MARKER`, `install_task_context_protection`, `write_memory_candidate`, `MemoryEvent`, `MemoryPolicy`, `MemoryLayerManager`, `Artifact`, `ArtifactProduced`, `add_artifact`, `list_artifacts`, `SubagentArtifact`, `observed_artifacts`, `persist_tool_output`, `ThinkingDelta`, `reasoning_content`, `in_thinking`, `worker`. Result: one live authority per concept; two dead arms found (`add_artifact`/`ArtifactProduced` chain, `ReviewFoundIssue`/`RepeatedTaskFailure`); `LlmCritic` and `ReviewIntegration` are distinct mechanisms, not review duplicates; zero `worker` terms. |
| Migration deletion | If the run-level artifact list is not a required surface, delete `Artifact`/`ArtifactProduced`/`add_artifact`/`list_artifacts`/`list_task_artifacts` + frontend fetch (P2-01); if kept, wire one writer at task completion. Dead memory-event arms (`ReviewFoundIssue`/`RepeatedTaskFailure`) either get production emitters at the circuit-breaker/review boundary or are deleted with their candidate builders (P3-01). |

## Current Path

Verified data flow (V01-01/V02-01/V03-01):

1. **Dispatch**: `execute_task` builds the typed `planned_task` payload
   (goal, deps summaries, files, checks, acceptance, artifacts) with the
   `[task_context]` marker (subagent_prompt.rs:318-380) and dispatches via
   `run_readonly_subagent`/`run_writer_subagent` (framework delegation) or
   `run_main_agent_task` (primary-agent stream, executor.rs:3049).
2. **Thinking isolation**: in both stream loops thinking tokens are routed to
   `ThinkingDelta`/`DispatchThinkingDelta` realtime events; only non-thinking
   tokens enter `output` (framework subagent/executor.rs:1200-1222; EKO
   executor.rs:3155-3182). Thinking is never persisted in `events.jsonl`
   (realtime sinks only) and `filter_history` drops reasoning turns
   (subagent/prompt.rs:161-163).
3. **Result projection**: framework `SubagentOutcome` (summary <= 1200 chars,
   artifacts hydrated with bytes/sha256/producer id, verification with
   Observed/Reported source, bounded remaining_work/touched_files) is mapped
   losslessly to `SubagentTaskResult` (types.rs:1584-1626); observed tool
   evidence (shell commands, tool-log artifacts) is merged in
   (executor.rs:3145-3486; framework executor.rs:229-254).
4. **Persistence**: on success, `put_summary` (TaskExecutionSummary) +
   `record_subagent_released` (`SubagentReleased` event carrying
   `result` + full `full_output`, store.rs:1950-1966) + `archive_trace`
   (ledger.rs:138-154) + `TaskCompleted` event with the terminal payload.
5. **Review**: `resolve_dispatch` -> `assess_task_execution` (hard evidence
   for checks/artifacts; missing -> Blocked with `AcceptancePending`,
   executor.rs:1433-1456) -> `run_review_gate` (acceptance criteria judged by
   reviewer LLM over the COMPLETE output, executor.rs:1461-1474) -> circuit
   breaker / fix task / suspend (review.rs:153-235) -> `integrate_reviewed_task`
   (worktree integration) -> Completed only on claim-guarded pass.
6. **Restart**: `recoverable_subagent_result` folds the persisted
   result+full_output back (store.rs:2039-2078) and `resolve_dispatch`
   re-enters the review boundary with identical evidence (executor.rs:1293-1313);
   `run_completion_blockers` re-assesses every Completed task before the run
   completes (executor.rs:386/:587-641).
7. **Parent projection**: parent consumes bounded summaries (task_execute
   outcome text, spill to complete artifact when large, task_execute_tool.rs:
   505-555), the capsule at model boundaries (compact_context.rs:163-255),
   memory candidates (memory_bridge.rs), and hooks (bounded summary only,
   hook_event_dispatcher.rs:317-345).

## Findings

### A-TSK-06-P2-01: Runtime `Artifact` records are a dead projection — `ArtifactProduced` is never emitted, so the GUI run-level artifact list is permanently empty

- Priority: P2
- Confidence: high (static proof; zero non-test writers)
- Layer: application
- Evidence: `store.add_artifact` (`echo-agent-cli/echo-agent-app-core/src/
  tasks/task_runtime/store.rs:1432-1447`) is the only writer of
  `RuntimeEventKind::ArtifactProduced`; its only caller in the whole
  repository is its own test (`store.rs:2263`); `file_store.rs:225-249`
  derives `list_artifacts` exclusively from `ArtifactProduced` events; the
  live artifact observation path (`observed_artifacts`, executor.rs:3145,
  :3422-3431) merges artifacts into `SubagentTaskResult.artifacts` via
  `merge_observed_evidence` (executor.rs:3481-3486) and never calls
  `add_artifact`; the Tauri command `list_task_artifacts`
  (`src/tauri/commands/task_runtime.rs:115-119`) and the frontend run
  snapshot fetch (`web-frontend/src/stores/taskRuntimeStore.ts:46-57`,
  `web-frontend/src/api/endpoints.ts:553`) read the dead projection.
- Reachability: GUI run panel -> `loadRunSnapshot` -> `listArtifacts` ->
  `list_task_artifacts` -> `list_artifacts` -> event scan; always `[]` in
  production. The `Artifact` struct (types.rs:1415-1426) and
  `ArtifactKind`/`ArtifactProduced` (types.rs:560-598, :649) are otherwise
  unreferenced in production.
- Expected invariant: artifacts produced by a task are retained as runtime
  records (`ArtifactProduced` event + `list_artifacts`) so the run-level
  surface and event stream expose them; artifact retention must survive
  restart (event replay).
- Observed behavior: no runtime artifact record is ever written; the
  run-level artifact list is always empty; the event stream contains no
  `artifact_produced` entries; artifacts exist only inside
  `SubagentTaskResult.artifacts` (SubagentReleased events / summaries) and as
  trace/spill files.
- Impact: the run-level artifact panel is a permanently non-functional
  surface; runtime artifact discovery/retention (a documented event kind and
  Tauri API) silently never works; downstream consumers of
  `ArtifactProduced` (hooks, frontend, recovery) never fire. Artifacts are
  NOT lost from the durable result path (files + hashes remain in
  `SubagentTaskResult`), so this is a surface/projection defect, not data
  loss.
- Root cause: the `Artifact`/`ArtifactProduced` projection predates the
  `SubagentTaskResult.artifacts` integration and was never wired — the
  executor's observed-artifact collection merges into the result instead of
  also persisting runtime records.
- Direction: either (a) call `store.add_artifact` for each merged artifact at
  the task terminal boundary (dedup by path per attempt) so
  `ArtifactProduced` fires and `list_artifacts` works, or (b) if the
  per-subagent result view is the intended surface, delete the dead chain:
  `Artifact` struct, `ArtifactProduced` event kind, `add_artifact`,
  `list_artifacts`, `list_task_artifacts` command, and the frontend
  `artifacts` fetch in `taskRuntimeStore.ts` (and the `RuntimeArtifact` type).
  Option (a) is recommended — it restores the documented run-level surface.
- Regression validation: fixture "task producing N artifacts -> N
  `ArtifactProduced` events; `list_artifacts` returns them after restart
  (event replay)"; GUI snapshot shows the run-level artifact list.
- Validation reports: [V01-01](../validations/A-TSK-06/V01-01.md),
  [V02-01](../validations/A-TSK-06/V02-01.md), [V03-01](../validations/A-TSK-06/V03-01.md)

### A-TSK-06-P3-01: `MemoryEvent::ReviewFoundIssue` and `RepeatedTaskFailure` are dead arms — the documented review-issue and repeated-failure memories are never written

- Priority: P3
- Confidence: high (grep proof; zero production constructors)
- Layer: application
- Evidence: the module contract documents these memories
  (`memory_bridge.rs:9-14`: "task failed review repeatedly -> repeated-bug-
  pattern memory", "review found an issue -> record the issue class" and
  `:77-82` plan §991) and `build_candidates` implements them
  (`memory_bridge.rs:231-250`, `:266-285`), but the only production dispatch
  sites construct `RunCompleted`/`RunCancelledByUser`
  (`executor.rs:456-465`, `:516-525`; `run_driver.rs:134-145`); the circuit
  breaker (`review.rs:153-235`) returns `Suspend`/`CreateFix` and emits no
  memory event; `MemoryEvent::ReviewFoundIssue`/`RepeatedTaskFailure` have
  zero non-test constructors (grep-verified).
- Reachability: none — the enum arms are reachable only from tests
  (`memory_bridge.rs:356-372`).
- Expected invariant: when a review records an issue class or a task fails
  review repeatedly, the memory bridge writes the corresponding candidate
  through the single `MemoryLayerManager` chokepoint (plan §991).
- Observed behavior: the breaker path (the only place those conditions are
  detected) never constructs the events; the candidates can never be written
  in production.
- Impact: the documented "known pitfall for similar future work" and
  "watch for this issue class" memories silently never land; no data
  corruption (memory is best-effort by design), but a stated product memory
  capability is unimplemented.
- Root cause: the memory-event enum was written ahead of its wiring; the
  review/breaker integration was never extended to emit the events.
- Direction: in `circuit_breaker_action`'s `Suspend` path (or the caller
  `run_review_gate`, executor.rs:1825-1833) dispatch
  `MemoryEvent::ReviewFoundIssue` (per distinct issue category) and, on
  fingerprint-repeat suspension, `RepeatedTaskFailure`; or delete the dead
  variants and their candidate builders if the memory is not wanted.
- Regression validation: unit test — a NeedsFix review with an issue
  category followed by a repeated fingerprint suspends AND produces the two
  memory candidates; recall finds them (mirror the B5.5 recall-closure test).
- Validation reports: [V01-01](../validations/A-TSK-06/V01-01.md),
  [V02-01](../validations/A-TSK-06/V02-01.md), [V05-01](../validations/A-TSK-06/V05-01.md)

### A-TSK-06-P3-02: `full_output` is persisted unbounded in `events.jsonl` and trace archives are written CWD-relative — ledger growth and scattered archive files

- Priority: P3
- Confidence: high (mechanism) / medium (impact magnitude)
- Layer: application
- Evidence: `record_subagent_released` embeds the complete `full_output`
  verbatim in the `SubagentReleased` event payload
  (`store.rs:1964-1966`; only `summary` is bounded at :1950); the append-only
  `events.jsonl` therefore grows by every full subagent output per attempt
  with no cap or retention; `archive_trace` is called with `base = None`
  (`executor.rs:2317`) and writes to `./.eko/runtime/{run_id}/artifacts/
  traces/{task_id}.txt` relative to the process CWD (`ledger.rs:138-154`),
  so the archive location depends on how the app was launched (GUI vs CLI
  diverge) and is not rooted at the canonical store root.
- Reachability: every completed/failed task dispatch; output size is
  unbounded by the framework (only `was_truncated` exists and is always
  false — subagent/executor.rs:1102,:1437).
- Expected invariant: the event ledger stores bounded event payloads while
  large outputs live in separate artifact files with stable paths; trace
  archives are written to a canonical per-run location.
- Observed behavior: full outputs are duplicated into the ledger (needed for
  restart-equivalence, but unbounded) and a second full copy is written to a
  CWD-dependent trace path with no retention policy.
- Impact: `events.jsonl` and the trace directory grow without bound across
  retries and runs (disk pressure; `read_events` re-reads and rewrites
  projections of the full stream at every write — amplification);
  restart-equivalence is preserved but at the cost of unbounded ledger size.
- Root cause: the terminal-boundary persistence requirement (V02-01) was
  implemented as raw full-text in the event payload with no size strategy,
  and the trace archive default base was left as the process CWD.
- Direction: cap `full_output` in the event payload (e.g. bounded preview +
  a trace-artifact reference, mirroring `record_tool_finished`'s
  `result_preview` pattern) while keeping a durable full-output store for
  review reuse — e.g. write the trace file under the store root
  (`WorkspaceLayout`/shadow root) and store its path in the event; add a
  retention policy for trace archives.
- Regression validation: fixture "large subagent output -> events.jsonl
  bounded, review after restart still receives the full output via the
  referenced trace artifact"; disk-growth test across N retries.
- Validation reports: [V03-01](../validations/A-TSK-06/V03-01.md),
  [V02-01](../validations/A-TSK-06/V02-01.md)

### A-TSK-06-P3-03: Parent summary chains have per-item bounds but no aggregate bound — dependency summaries and `TodoItem.summary` can grow large without an enforced cap

- Priority: P3
- Confidence: medium (bounds are loose, worst case requires many files/
  decisions)
- Layer: application
- Evidence: `collect_dependency_summaries` joins `written` files and
  `decisions` without truncation (`executor.rs:2623-2636`); the framework
  parse bounds each item (MAX_RESULT_ITEMS=64, MAX_PATH_CHARS=2048,
  MAX_DETAIL_CHARS=500 — `echo-agent/src/agent/subagent/types.rs:247-250`),
  so a single dependency summary can reach ~128 KB and the joined brief is
  unbounded across items; `format_execution_summary`
  (`task_execute_tool.rs:640-700`) joins the same lists into the
  `task_execute` outcome text; the capsule is the only projection that
  re-truncates (MAX_SUMMARY_CHARS=260, 80 chars/item, 3 items/field —
  compact_context.rs:30-34, :336-363); `completion_summary` is the
  unbounded concatenation `summary | integration_summary`
  (`executor.rs:1751`) stored into `TodoItem.summary` via
  `set_claimed_task_status` with no bound (`store.rs:1032-1065`), and it
  feeds `run_completion_blockers`, memory candidates, and `progress.md`.
- Reachability: every completed task with declared files/decisions flows into
  downstream subagent briefs and the todo summary; worst case requires a
  subagent to touch many files or report many decisions.
- Expected invariant: parent-facing summaries are bounded in aggregate as
  well as per field, so repeated chains and compaction cannot inflate the
  parent context or todo rows without limit.
- Observed behavior: only the capsule and the final task_execute spill
  enforce aggregate bounds; the summary chain and todo summaries rely on the
  loose per-item framework bounds.
- Impact: large dependency summaries can bloat subagent briefs and
  `TodoItem.summary` (memory writes, progress.md, capsules all re-carry
  them); pathological runs can approach or exceed context budgets.
- Root cause: bounds were applied at the source (framework parse) and the
  final projection (capsule/spill) but not at the summary-chain joins.
- Direction: apply a shared char cap when building `dep_summaries`
  (`collect_dependency_summaries`) and when concatenating
  `completion_summary` (e.g. reuse `bounded_event_text`), and cap
  `TodoItem.summary` at the `set_claimed_*` write path.
- Regression validation: fixture "subagent reports 64 files x 2048 chars ->
  dependency summary and todo summary are capped at N chars (chars().count()
  assertion)"; repeated-compression stability test with the capped summaries.
- Validation reports: [V03-01](../validations/A-TSK-06/V03-01.md),
  [V01-01](../validations/A-TSK-06/V01-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition + duplicate search (both repos; review/acceptance/check/artifact/summary/capsule/memory/thinking terms) | yes | passed (2 dead arms found) | [V01-01](../validations/A-TSK-06/V01-01.md) |
| V02 | Registration + runtime reachability (reviewer LLM threading, capsule registry, memory dispatch, full-output persistence/reuse, completion re-check) | yes | passed (P2-01/P3-01 reachability gaps) | [V02-01](../validations/A-TSK-06/V02-01.md) |
| V03 | Invariants/edge cases (acceptance/check separation; artifact retention; bounded parent summaries; thinking isolation; UTF-8 safety) | yes | passed (P2-01, P3-01, P3-02, P3-03 evidence) | [V03-01](../validations/A-TSK-06/V03-01.md) |
| V04 | `cargo test -p echo-agent-app-core --locked task_runtime::review` | yes | passed (exit 0, 6 ok) | [V04-01](../validations/A-TSK-06/V04-01.md) |
| V04 | `cargo test -p echo-agent-app-core --locked task_runtime::compact_context` | yes | passed (exit 0, 5 ok) | [V04-02](../validations/A-TSK-06/V04-02.md) |
| V04 | `cargo test -p echo-agent-app-core --locked task_runtime::memory_bridge` | yes | passed (exit 0, 8 ok) | [V04-03](../validations/A-TSK-06/V04-03.md) |
| V04 | `cargo test -p echo-agent-app-core --locked review_gate_receives_complete_output` | yes | passed (exit 0, 1 ok) | [V04-04](../validations/A-TSK-06/V04-04.md) |
| V04 | `cargo test -p echo-agent-app-core --locked task_runtime::types` | yes | passed (exit 0, 51 ok) | [V04-05](../validations/A-TSK-06/V04-05.md) |
| V04 | `cargo test -p echo-agent-app-core --locked execution_check_requires_observed_evidence_and_integrity` | yes | passed (exit 0, 1 ok) | [V04-06](../validations/A-TSK-06/V04-06.md) |
| V04 | `cargo test -p echo-agent-app-core --locked plain_text_summary_passes` | yes | passed (exit 0, 1 ok) | [V04-07](../validations/A-TSK-06/V04-07.md) |
| V04 | `cargo test -p echo_agent --lib --features subagent --locked subagent::types` | yes | passed (exit 0, 11 ok) | [V04-08](../validations/A-TSK-06/V04-08.md) |
| V05 | Historical-document drift (MASTER-PLAN review/thinking/acceptance claims; results-and-completion doc; convergence doc; root MASTER-PLAN) | conditional | passed (1 regressed claim -> P3-01) | [V05-01](../validations/A-TSK-06/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN (app): final answer/summary cannot refer to reasoning; "never promotes a thinking trace into the result" | current | framework subagent/executor.rs:1200-1222; EKO executor.rs:3155-3182; prompt.rs:161-163 (V01-01/V03-01) |
| MASTER-PLAN (app): "TaskRuntime review consumes the complete output rather than the bounded parent summary, and persists that output on the terminal boundary so restart recovery receives identical evidence" | current | executor.rs:1461-1474; store.rs:1950-1966, 2039-2078; V04-04 (V02-01) |
| 2026-07-17 results-and-completion: `SubagentReleased` saves structured result + complete output; resume continues from review boundary; review prompt must include output beyond the summary boundary | current | store.rs:1933-1970; executor.rs:1293-1313; V04-04 (V02-01) |
| 2026-07-17 results-and-completion: completion gate = completed status, non-empty summary, no remaining work, observed-pass verification, required artifact with hash/producer id | current | executor.rs:695-765 (V03-01) |
| 2026-07-27 convergence: `required_artifacts`/`execution_checks`/`acceptance_criteria` kept distinct; no verification-list flattening | current | types.rs round-trip test (V04-05) |
| Root MASTER-PLAN: "完成判定同时检查 task node、required artifacts、verification 和 unresolved failures" | current | `run_completion_blockers` executor.rs:587-641 (V03-01) |
| Root MASTER-PLAN: "超长工具结果已统一为完整 artifact + 有界模型/会话投影" | current (task_execute path) | task_execute_tool.rs:505-555 spill (V03-01) |
| memory_bridge plan §991: review issue class recorded for future avoidance | regressed (unimplemented) | dead `ReviewFoundIssue`/`RepeatedTaskFailure` arms -> P3-01 (V05-01) |
| No doc claims runtime `Artifact`/`ArtifactProduced` records are produced | consistent with code | dead `add_artifact` -> P2-01 is an implementation gap, not doc drift (V05-01) |

## Coverage And Uncertainty

- All conclusions are static traces plus the V04 unit-test runs; no live LLM
  run, no fault injection, and no GUI process was launched (read-only review).
  P2-01's GUI-emptiness claim rests on the static call chain
  (frontend -> Tauri -> file_store event scan) plus the zero-writer grep; a
  Q-E2E-01 smoke (run a task that produces an artifact, open the run panel)
  would confirm visually.
- P3-03's worst-case sizes are computed from framework constants
  (64 x 2048 chars), not measured; a pathological-fixture test is proposed.
- `ReviewIntegration` (evolution evidence review) and `LlmCritic`
  (final-answer self-verification) were classified as distinct mechanisms,
  not task-review duplicates; their own semantics belong to A-MEM-01/A-EVO-01.
- The frontend rendering of the (empty) artifact list was not inspected
  component-by-component (A-FE-02 scope); only the store/API fetch was
  verified.
- The `TaskExecutionSummary.to_runtime_summary` mapping and the framework
  `Summarize` trait were checked structurally; the trait has no EKO
  implementor, so no double summary authority exists.

## Handoff

- Downstream tasks may rely on: review consumes the COMPLETE Subagent output
  (not the bounded summary) and that output is persisted in
  `SubagentReleased` and reused after restart — restart-equivalent review
  input holds (V02-01, V04-04); acceptance/check separation is enforced —
  execution checks and required artifacts require observed/hard evidence,
  acceptance criteria are reviewer-judged, and the run cannot complete while
  any Completed task lacks evidence (V03-01); thinking content never reaches
  `output`, review prompts, summaries, capsule, memory, or hooks (V03-01);
  the capsule/summaries are UTF-8 safe (chars-based truncation) (V03-01).
- Findings for the roadmap: P2-01 (wire or delete the runtime artifact
  projection — GUI run-level artifact list currently always empty),
  P3-01 (wire review-issue/repeated-failure memory events at the circuit
  breaker, or delete the dead arms), P3-02 (bound `full_output` in
  `events.jsonl`; root trace archives at the canonical store location),
  P3-03 (cap aggregate summary-chain and `TodoItem.summary` sizes).
- Reports to read: the 12 validation reports above; dependency reports
  A-TSK-04 (claim/ledger authority, recovery) and A-STATE-01 (conversation/
  artifact retention); A-TSK-01 (file authority) for P3-02's ledger-growth
  interplay.
- Stale conditions: this report becomes stale if `review.rs`, the executor
  review/acceptance branches (`resolve_dispatch`/`run_review_gate`/
  `assess_task_execution`), `record_subagent_released`/
  `recoverable_subagent_result`, `compact_context.rs`, `memory_bridge.rs`,
  `add_artifact`/`list_artifacts`, the summary-chain joins, or the
  thinking-token routing in either executor change; also if a production
  caller of `add_artifact` or the dead memory-event arms appears (findings
  fixed).
- Follow-up task IDs: A-FE-02 (frontend artifact/result projections),
  X-TSK-01 (cross-repo conformance of result/artifact projection),
  X-STA-01 (artifact identity across restart), Q-FLT-02 (fault fixtures:
  artifact-producing task, review-issue memory recall, large-output ledger
  bound), Q-E2E-01 (GUI run-panel artifact smoke), S-RDM-01 (roadmap items
  for the four findings above).
