# A-FE-02: Task, Subagent, and tool projections

> Status: complete
> Reviewer: Codex primary reviewer
> Executor: Codex primary reviewer
> Review date: 2026-08-13
> `echo-agent` commit: 3aa7929928442aab91e4dce9c426d909a5f0a1ab
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: CLI clean; framework worktree externally dirty and excluded except committed `git show HEAD:<path>` evidence

## Question

Do frontend projections preserve attempt identity, terminal monotonicity, lazy
output, complete results, artifacts, and Task acceptance distinctions?

## Scope

- `taskRuntimeStore`, `subagentRunStore`, `toolExecutionStore`, their reducers,
  hydration adapters, and static test inventory.
- TaskRuntime, Subagent stream/detail/result, message execution, and tool
  rendering components.
- The committed EKO durable release/review/artifact event shape needed to prove
  producer-to-consumer reachability.

## Out Of Scope

- Missing Rust/TypeScript variants and command DTO drift owned by `A-FE-01`.
- Backend result/review retry/artifact production defects owned by `A-TSK-06`.
- The live tool reducer's terminal-reopen behavior owned by `A-SRF-03`.
- General frontend architecture, cross-conversation retention, accessibility,
  and responsive behavior owned by `A-FE-03`.
- Source fixes, build, test, fixture, browser, or network execution.

## Inputs

- Root `AGENTS.md`, `docs/comprehensive-review/TASKS.md`, reporting protocol,
  and Codex review rules.
- Accepted Codex dependency reports `A-FE-01` and `A-TSK-06`.
- Clean CLI source at the fixed commit and committed framework event/type blobs.

## Layering Decision

Generic Subagent execution identity and terminal result shape remain framework
contracts. EKO owns Task acceptance/review/artifact policy. React stores are thin
projections: they must retain the framework execution ID intact, select the
current `(plan_revision, attempt)`, monotonically merge richer observations, and
render EKO's authoritative Task distinctions without inventing a second task or
tool state machine. The existing stores already provide the correct authority
locations; the remediation should extend their reducers/selectors rather than
add sibling stores.

## Positive Conclusions

- `subagentRunStore` keys records by `(run_id, subagent_run_id)` and keeps every
  retry record separate. A duplicate `started` cannot reopen a terminal run.
- `toolExecutionStore` durable hydration merges by owner/run/call identity,
  ranks terminal over running, preserves the earliest start, and keeps separate
  TaskRuns distinct. The remaining live-ingest overwrite is already owned by
  `A-SRF-03-P2-04` and is not duplicated here.
- Persisted Task statuses remain authoritative over Subagent trace status, so a
  completed execution does not itself mark executor-owned acceptance complete.
- TaskRuntime conversation loading has generation guards and resets the event
  cursor when changing runs.

## Findings

### A-FE-02-P1-01: Current-attempt selection ignores plan revision

- Priority: P1
- Confidence: high
- Layer: frontend projection
- Evidence: `echo-agent-cli/web-frontend/src/stores/subagentRunStore.ts:407`; `echo-agent-cli/web-frontend/src/stores/subagentRunStore.ts:417`; `echo-agent-cli/web-frontend/src/components/chat/ParallelExecutionBlock.tsx:62`; `echo-agent-cli/web-frontend/src/components/chat/ParallelExecutionBlock.test.ts:80`
- Reachability: one TaskRun commits a new plan revision for the same Task ID -> execution IDs from both revisions remain in the store -> every message-level Subagent list calls `latestSubagentRunsByTask`.
- Expected invariant: the current projection compares the complete execution identity `(run, task, plan_revision, attempt)`; a newer revision always supersedes every attempt from an older revision.
- Observed behavior: `executionAttempt` parses only the suffix after the last colon. Selection groups by `(run, task)` and prefers the numerically larger attempt, so revision 3 attempt 5 hides revision 4 attempt 1 even when the latter is newer. Tests cover only two attempts in one revision.
- Impact: after plan editing or same-node retry reset, the inline card can display an obsolete terminal output while the current revised Subagent is running, and current controls/details point at the wrong execution.
- Root cause: a structured identity is encoded as an opaque string, then partially reparsed by the presentation selector.
- Direction: carry generated/typed `plan_revision` and `attempt` fields on `SubagentRunState`, compare revision before attempt, and retain the opaque execution ID solely as identity. Delete suffix parsing once all producers populate the fields.
- Regression validation: feed revision 3 attempt 5 completed before revision 4 attempt 1 running, in both event orders, and assert revision 4 is displayed while both records remain inspectable.
- Validation reports: [V01](../validations/A-FE-02/V01-01.md), [V07](../validations/A-FE-02/V07-01.md)

### A-FE-02-P1-02: A first terminal event permanently blocks richer durable terminal data

- Priority: P1
- Confidence: high
- Layer: frontend reducer
- Evidence: `echo-agent-cli/web-frontend/src/stores/subagentRunStore.ts:376`; `echo-agent-cli/web-frontend/src/stores/subagentRunStore.ts:458`; `echo-agent-cli/web-frontend/src/stores/taskRuntimeStore.ts:196`; `echo-agent-cli/src/tauri/mod.rs:617`
- Reachability: live `DispatchFailed`/`DispatchCancelled` reaches the store without full output -> TaskRuntime polling later projects durable `subagent_released(full_output, result)` for the same execution -> `ingest` sees an already-terminal record.
- Expected invariant: terminal status is monotonic, but duplicate/later observations may fill missing output, artifacts, verification, duration, usage, or provenance without changing the terminal reason.
- Observed behavior: the reducer returns immediately for every event after any terminal state. It therefore prevents reopening, but also discards the durable full output and any richer terminal fields. The terminal test checks only duplicate `started`, not enrichment.
- Impact: the same failed/cancelled execution can show only a bounded summary during its live session but show a fuller result after reload; diagnostics and artifacts depend on observation timing.
- Root cause: lifecycle rank and field enrichment are conflated into one early-return guard.
- Direction: implement a monotonic terminal merge: reject status regression/conflicting terminal replacement while filling absent fields from the same execution identity. Define a deterministic precedence for conflicting terminal facts and surface corruption rather than silently selecting arrival order.
- Regression validation: ingest live failed/cancelled first and a richer durable release second, then reverse the order; assert one unchanged terminal status and identical complete fields.
- Validation reports: [V02](../validations/A-FE-02/V02-01.md), [V07](../validations/A-FE-02/V07-01.md)

### A-FE-02-P1-03: Task acceptance causes and recovery actions are flattened into one generic label

- Priority: P1
- Confidence: high
- Layer: frontend projection
- Evidence: `echo-agent-cli/web-frontend/src/generated/PlanTask.ts:44`; `echo-agent-cli/web-frontend/src/generated/RuntimeEventKind.ts:47`; `echo-agent-cli/web-frontend/src/components/task/TaskRuntimePanel.tsx:450`; `echo-agent-cli/web-frontend/src/components/task/TaskRuntimePanel.tsx:496`; `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:1403`
- Reachability: hard checks, required artifacts, reviewer NeedsFix/Blocked, dependency failure, and recovery can all persist a blocked/failed Task plus status/review payload -> `loadRunSnapshot` loads Plan/Todos/events -> mounted TaskRuntimePanel.
- Expected invariant: the GUI distinguishes execution failure, missing observed check, missing artifact, semantic NeedsFix, external Blocked, dependency blocking, and recovery uncertainty, preserving their reason and valid next action.
- Observed behavior: the generated PlanTask already carries `status_detail`/`failure_fingerprint`, and events include `review_needs_fix`, `review_blocked`, issues, and fingerprints. The panel ignores those fields and does not render events; a completed Subagent plus blocked Task becomes only `评审未通过`, with the same generic retry/edit controls.
- Impact: users cannot tell whether to rerun a check, produce an artifact, edit the task, fix a semantic defect, or resolve an external blocker. The UI can encourage a retry that cannot address the actual acceptance failure.
- Root cause: Todo status is treated as the whole acceptance contract even though authoritative reason/evidence already exists in PlanTask and review events.
- Direction: derive one typed acceptance projection from canonical PlanTask plus latest review/hard-gate event, render cause/evidence and only legal recovery actions, and delete prose-only inference from Subagent status. Preserve `A-TSK-06` ownership of carrying reviewer feedback into the next attempt.
- Regression validation: render separate fixtures for missing check, missing artifact, NeedsFix issues, ReviewBlocked, dependency block, and recovery blocker; assert distinct reason and action sets.
- Validation reports: [V03](../validations/A-FE-02/V03-01.md), [V07](../validations/A-FE-02/V07-01.md)

### A-FE-02-P1-04: Authoritative TaskRuntime artifacts are fetched but never rendered or opened

- Priority: P1
- Confidence: high
- Layer: frontend projection
- Evidence: `echo-agent-cli/web-frontend/src/stores/taskRuntimeStore.ts:50`; `echo-agent-cli/web-frontend/src/stores/taskRuntimeStore.ts:83`; `echo-agent-cli/web-frontend/src/components/task/TaskRuntimePanel.tsx:517`; `echo-agent-cli/web-frontend/src/components/subagent/SubagentResultView.tsx:41`
- Reachability: every TaskRuntime load/refresh calls `listArtifacts` and stores `RuntimeArtifact[]` -> TaskRuntimePanel is mounted in the right rail.
- Expected invariant: a durable accepted artifact remains discoverable by run/task, exposes its stable ID/title/kind/provenance, and has a read/open/export action independent of transient Subagent cards.
- Observed behavior: no mounted component reads `taskRuntimeStore.artifacts`. Subagent results separately render only `available|missing` plus a raw path; they expose no ID, digest, producer, byte count, metadata, or open/read action.
- Impact: even once the backend projection defect in `A-TSK-06-P1-03` is fixed, the GUI will silently hide the authoritative artifact list and users cannot recover full outputs or inspect provenance.
- Root cause: artifact acquisition was added to store hydration without a presentation/interaction consumer, while a second bounded Subagent list became the only visible approximation.
- Direction: render the canonical RuntimeArtifact list in TaskRuntime UI and resolve it through one safe open/read/export adapter; join Subagent artifact summaries to it by stable identity/provenance. Remove duplicate raw-path presentation after cutover.
- Regression validation: hydrate multiple task artifacts including unavailable/large entries, collapse and reopen the task panel, and assert stable metadata plus read/open actions without embedding full bytes.
- Validation reports: [V04](../validations/A-FE-02/V04-01.md), [V08](../validations/A-FE-02/V08-01.md)

### A-FE-02-P2-05: Durable Subagent full output is eagerly transported and retained before expansion

- Priority: P2
- Confidence: high
- Layer: application/frontend projection
- Evidence: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:1950`; `echo-agent-cli/web-frontend/src/stores/subagentRunStore.ts:380`; `echo-agent-cli/web-frontend/src/stores/subagentRunStore.ts:507`; `echo-agent-cli/web-frontend/src/components/chat/SubagentStreamBlock.tsx:103`; `echo-agent-cli/web-frontend/src/components/task/SubagentDetailView.tsx:153`
- Reachability: every durable Subagent release serializes `full_output` into JSONL -> conversation load fetches all run events -> adapter copies complete output into global Zustand before any card is expanded -> expansion renders it as Markdown.
- Expected invariant: collapsed lists hydrate bounded summary/status/identity only; complete large output is retained as an artifact/detail locator and loaded on explicit expansion with cancellation and size handling.
- Observed behavior: full output is embedded in the event, parsed, copied into every terminal run, and retained regardless of collapsed state. Expansion controls only React rendering; it does not defer transport, parsing, or memory. No output locator/detail endpoint exists.
- Impact: long multi-Subagent runs make conversation restore and memory proportional to all historical final outputs, cause repeated large object copies, and can stall the GUI before the user opens any result.
- Root cause: durable event replay is used simultaneously as lifecycle index and full-content store.
- Direction: keep a bounded terminal summary plus content length/hash/artifact reference in lifecycle events; add an explicit lazy detail read using the existing artifact model, cache with a bounded policy, and delete embedded `full_output` after migration.
- Regression validation: restore a run with several multi-megabyte results and assert collapsed hydration does not fetch/retain bodies; expanding one result fetches exactly that body and preserves UTF-8/content hash.
- Validation reports: [V05](../validations/A-FE-02/V05-01.md), [V09](../validations/A-FE-02/V09-01.md)

## Validation Matrix

| ID | Claim | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Inputs, commits, source isolation, scope | yes | passed | [V00](../validations/A-FE-02/V00-01.md) |
| V01 | Subagent execution identity and current-attempt selector | yes | failed/finding | [V01](../validations/A-FE-02/V01-01.md) |
| V02 | Duplicate/out-of-order terminal merge | yes | failed/finding | [V02](../validations/A-FE-02/V02-01.md) |
| V03 | Task acceptance distinction projection | yes | failed/finding | [V03](../validations/A-FE-02/V03-01.md) |
| V04 | Artifact acquisition and rendering reachability | yes | failed/finding | [V04](../validations/A-FE-02/V04-01.md) |
| V05 | Collapsed/expanded large-output data flow | yes | failed/finding | [V05](../validations/A-FE-02/V05-01.md) |
| V06 | Tool identity/hydration and duplicate ownership | yes | passed/deduplicated | [V06](../validations/A-FE-02/V06-01.md) |
| V07 | Existing reducer/component test inventory | yes | failed/gaps | [V07](../validations/A-FE-02/V07-01.md) |
| V08 | Dependency ownership and semantic deduplication | yes | passed | [V08](../validations/A-FE-02/V08-01.md) |
| V09 | Dynamic reducer/render/large-output fixtures | future | not_run | [V09](../validations/A-FE-02/V09-01.md) |
| V10 | Exact-link/header/source-isolation integrity | yes | V10-01 inaccurate; V10-02 passed | [V10](../validations/A-FE-02/V10-02.md) |

## Coverage And Uncertainty

- This review is source-conclusive but purely static. No Vitest, browser render,
  build, or synthetic fixture was executed per user instruction.
- Framework source was referenced only through committed blobs or already
  accepted dependency evidence because its live worktree changed concurrently.
- `RuntimeArtifact` currently lacks a read/open endpoint in the inspected Task
  component path; exact UX belongs to the roadmap, not this atomic review.
- The store can retain Subagent runs across conversation switches. That broader
  retention/performance policy belongs to `A-FE-03` and is not counted here.
- The live tool reducer defect remains solely `A-SRF-03-P2-04`; V06 records the
  otherwise sound durable merge so synthesis can avoid a duplicate repair.

## Handoff

Fix order: define typed revision/attempt fields and monotonic enrichment; expose
typed acceptance causes; connect canonical artifacts and lazy result details;
then remove opaque-ID parsing, raw-path-only artifact UI, and embedded full
output. Preserve one TaskRuntime/Subagent/tool authority and reuse the generated
contracts rather than creating frontend-only domain types.
