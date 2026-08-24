# X-TOL-01: Tool error, artifact, and schema conformance

> Status: complete
> Reviewer: Codex review subagent
> Executor: Codex review subagent
> Accepted by: Codex primary reviewer
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: CLI was clean during source inspection; an external `Cargo.lock` modification appeared at the final gate and was not read or changed. Framework was externally dirty, with all adopted framework anchors reconstructed from committed `HEAD` blobs. One accidental dirty-body search is disclosed and excluded in V00. Only Codex reports were added.

## Question

Does one Tool invocation retain the same schema, classification, output
integrity, artifact metadata, and terminal reason across framework, EKO
persistence, frontend, and interactive surfaces?

## Scope

- One live ReAct Tool invocation from model ToolCall through policy stages,
  ToolManager, string/rich results, events, TaskRuntime projection, Tauri
  persistence, frontend store, detail paging, and `InlineToolCall`.
- Requested versus effective Tool name/arguments after intervention, hooks, and
  approval modification.
- `ToolFailure`, terminal classification, parent/turn terminal inference, and
  restart recovery.
- Complete text output, spill metadata/SHA-256, cursor identity, application
  paging precedence, and existing tests.

## Out Of Scope

- Generic schema validator, collision, cancellation, result-kind, pre-execution
  classification, and pagination-to-model defects owned by `F-EXT-01`.
- Batch call-ID/order/timeout/checkpoint defects owned by `F-RCT-04`.
- Individual shell/file/Git/data/research Tool defects owned by `F-EXT-02` and
  `F-EXT-03`.
- Channel event/artifact dropping owned by `A-TOOL-01`.
- Task/Subagent current-attempt, acceptance, artifact rendering, and lazy-output
  findings owned by `A-FE-02`.
- Dynamic fixtures, Cargo/rustc/tests/builds, browser, and network execution.

## Inputs

- Root `AGENTS.md`; shared `README.md`, `TASKS.md`, `REPORTING.md`; Codex
  `README.md` and report templates.
- Authorized accepted Codex dependencies: `F-RCT-04`, `F-EXT-01`,
  `F-EXT-02`, `F-EXT-03`, `A-TOOL-01`, and `A-FE-02`.
- Committed framework blobs and clean CLI source at the fixed commits above.
  No other reviewer directory was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Schema validation, requested/effective invocation identity, rich ToolResult/ToolFailure, typed terminal reason, spill descriptor, checksum, and snapshot-bound cursor are framework contracts useful to every consumer. |
| EKO product policy | Local retention root, conversation/TaskRun ownership, lazy GUI presentation, and copy/open UX belong to EKO. |
| Adapter boundary | EKO should persist a lossless canonical invocation/result envelope and project it into files/frontend. It must not infer per-call terminals from parent status or implement a second artifact reader with weaker identity rules. |
| Duplicate search | Searched ToolCall/ToolResult/ToolError/ToolStream/Complete, effective args/updated input, ToolFailure/status, artifact/SHA/cursor/read output, Tauri sink/repository, TaskRuntime producer/store, and frontend store/component across both repositories. Framework has one rich result and canonical artifact reader; EKO adds a second weaker reader and a projection that expects a Complete event the manager consumes. |
| Migration deletion | Extend the canonical framework event/result envelope, then delete EKO's unreachable PendingToolCompletion protocol and raw-path artifact reader. Keep one application repository as a lazy projection, not a Tool-result authority. |

## Current Path

```text
model ToolCall(name, args, call_id)
  -> run_tools emits early AgentEvent::ToolCall(requested name/args)
  -> pipeline
       intervention redirect/modify
       name-table validation
       PreToolUse modify
       approval modify params
       ExecuteStage -> ToolManager -> rich ToolResult/ToolFailure
       output guard -> spill/truncate -> SHA/path/bytes metadata -> trace
  -> execute_tool_with_policy collapses rich result to String
  -> run_tools emits AgentEvent::ToolResult(String) or ToolError(ToolFailure)
  -> shared drive_chat
       GUI Tauri sink -> ToolExecutionRepository -> detail manifest/JSONL
       TaskRuntime ExecEvent -> same repository + durable event store
       TUI/CLI/channel adapters (channel loss remains A-TOOL-01)
  -> frontend toolExecutionStore -> InlineToolCall -> detail/readOutput IPC
```

For streaming Tools, ToolManager forwards Output/Progress but consumes Complete
as its return value. The later central spill metadata therefore cannot reach
EKO's `PendingToolCompletion` listener. The framework's own `read_artifact` is
the positive integrity authority: it constrains the root, verifies SHA-256,
binds cursor to file size/mtime/hash, and rejects mutation. EKO's detail reader
does not reuse that contract.

## Findings

### X-TOL-01-P1-01: GUI persists the requested invocation while policy executes a different effective invocation

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent/src/agent/react/run/phases/tools.rs:63`; `echo-agent/src/agent/react/run/pipeline.rs:90`; `:112`; `:237`; `:258`; `:328`; `:340`; `:478`; `:509`; `echo-agent-cli/src/tauri/commands/chat.rs:1193`; `echo-agent-cli/echo-agent-app-core/src/tool_execution.rs:191`; `:394`
- Reachability: live ReAct batch emits requested ToolCall -> intervention/hook/approval can rewrite -> ExecuteStage uses effective name/params -> Tauri persists only the earlier event -> frontend displays/copies `args_full`.
- Expected invariant: one invocation records the actual executed name and arguments, while any requested values are explicitly labeled as provenance.
- Observed behavior: the public ToolCall and GUI detail are created before rewriting. Intervention and PreToolUse update the pipeline context; approval updates only `params`, leaving even internal `input` consumers stale. No later event carries the effective invocation.
- Impact: a user reviewing or copying GUI details can see a harmless/different command, path, or Tool than the one that performed the side effect; audit/post-hook/trace fields can also disagree after approval modification.
- Root cause: ToolCall is treated simultaneously as an LLM request and execution-start fact, with no canonical post-policy invocation envelope.
- Direction: canonicalize name/schema/arguments once after all allowed rewrites and before execution; emit requested plus effective values and rewrite provenance in one typed execution-start event. Update `ctx.input` and `params` atomically. Delete the early event as the durable execution authority rather than teaching each adapter to reconstruct rewrites.
- Regression validation: redirect plus Unicode argument rewrite plus approval edit; actual Tool, trace, hooks, persisted detail, and all surfaces must agree on effective values while retaining requested provenance.
- Validation reports: [V01](../validations/X-TOL-01/V01-01.md)

### X-TOL-01-P1-02: Rich terminal results and complete artifacts collapse before EKO persistence

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent/echo-execution/src/tools.rs:823`; `echo-agent/src/agent/react/run/pipeline.rs:729`; `echo-agent/src/agent/snapshot.rs:1229`; `echo-agent/src/agent/react/run/phases/tools.rs:273`; `echo-agent-cli/src/tauri/commands/chat.rs:1249`; `:1264`; `echo-agent-cli/echo-agent-app-core/src/tool_execution.rs:260`; `:292`; `:413`; `echo-agent-cli/web-frontend/src/components/chat/InlineToolCall.tsx:61`
- Reachability: any streaming or non-streaming Tool -> rich ToolResult -> pipeline spill/metadata -> string AgentEvent -> Tauri repository -> frontend detail paging.
- Expected invariant: the terminal result is the complete authoritative source or carries a verifiable artifact descriptor; streaming chunks are incremental observations and cannot replace it merely by existing.
- Observed behavior: ToolManager consumes stream Complete; central metadata is added later; then the snapshot returns only a String. EKO's Complete handler is therefore unreachable on this path. A non-streaming spilled result is stored as a normal preview with `truncated=false`, no artifact descriptor, and preview-sized byte count. If any streamed Output exists, repository `has_output` suppresses the terminal String and `read_output` always chooses JSONL over the artifact fallback.
- Impact: GUI detail can label a 500-character preview or partial streamed observation as complete, hide the recovery artifact/hash, under-report bytes, and copy content different from the Tool's final result.
- Root cause: the framework rich result is collapsed at the event boundary while EKO independently assumes Complete metadata will arrive; repository precedence then confuses “some output observed” with “complete output stored.”
- Direction: define one canonical typed terminal Tool envelope carrying result kind, success/failure, output source, artifact descriptor, digest/bytes, truncation, and finality. Persist it losslessly. Treat stream chunks as a separate channel and explicitly select/deduplicate terminal content. Delete `PendingToolCompletion` after the typed terminal owns metadata.
- Regression validation: non-stream spill and streaming chunk plus distinct large final result; GUI detail/copy must return the complete source, correct bytes/digest/truncation, and one terminal.
- Validation reports: [V02-02](../validations/X-TOL-01/V02-02.md), [V04](../validations/X-TOL-01/V04-01.md)

### X-TOL-01-P2-03: EKO duplicates artifact paging without checksum or snapshot-bound cursors

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `echo-agent/echo-tools/src/files/artifact.rs:16`; `:86`; `:280`; `:310`; `:375`; `echo-agent-cli/echo-agent-app-core/src/tool_execution.rs:394`; `:413`; `:668`; `:720`
- Reachability: any EKO detail manifest that receives `artifact_path` -> `read_tool_execution_output` -> repository `read_artifact_page` -> frontend repeated cursor loads.
- Expected invariant: complete artifact paging uses the canonical root confinement, expected digest, immutable identity, UTF-8 boundary, and opaque cursor contract.
- Observed behavior: EKO keeps the raw path privately but implements another reader accepting a decimal byte offset. It does not validate configured root, expected `artifact_sha256`, file size/mtime across pages, or bind the cursor to `detail_ref`. The framework reader already implements all those checks.
- Impact: replaced/mutated files can yield mixed pages that the UI labels complete under the original Tool record; corruption is not detected and provenance cannot be trusted.
- Root cause: GUI lazy reading reimplemented the framework artifact authority instead of adapting its verified descriptor/reader.
- Direction: expose one framework artifact read service or content-addressed descriptor to EKO and bind cursor to artifact ID/hash/snapshot. Delete application `parse_cursor`/`read_artifact_page` after cutover; keep EKO only as the authorization-free local IPC adapter and renderer.
- Regression validation: mutate/replace artifact between pages, replay cursor against another detail, wrong expected digest, UTF-8 boundary, retention deletion, and ordinary multi-page completion.
- Validation reports: [V02-02](../validations/X-TOL-01/V02-02.md), [V04](../validations/X-TOL-01/V04-01.md)

### X-TOL-01-P1-04: EKO records failure, timeout, parent completion, and crash interruption as cancellation

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent/echo-core/src/tools/mod.rs:76`; `echo-agent-cli/echo-agent-app-core/src/tool_execution.rs:54`; `:360`; `:508`; `echo-agent-cli/src/tauri/commands/chat.rs:1106`; `:1116`; `:1171`; `:1325`; `:1365`; `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:1999`
- Reachability: active direct-chat or TaskRuntime Tool -> missing per-call terminal followed by Agent/turn/parent failure, timeout, normal completion, cancellation, or process restart -> repository cancel/recovery -> frontend status.
- Expected invariant: a per-call terminal preserves failed/timed_out/cancelled/interrupted/unknown distinctions and typed recovery/side-effect facts; parent status cannot invent a Tool outcome.
- Observed behavior: the repository supports only running/succeeded/failed/cancelled. Direct-chat Error and every non-running turn terminal cancel all active calls. TaskRuntime Completed, Failed, Cancelled, and TimedOut share the same cancellation method. Boot recovery also changes every Running record to Cancelled. The durable TaskRuntime boundary separately supports ToolFailed and failure payloads, demonstrating the projection loses rather than lacks this distinction.
- Impact: users and recovery logic cannot tell a user stop from timeout, parent failure, normal parent completion with an orphan, or crash after a possible side effect; “cancelled” can invite an unsafe blind retry or hide an incomplete side effect.
- Root cause: the application fills missing per-call framework terminals by inferring cancellation from unrelated parent/process transitions.
- Direction: first fix per-call terminal ownership under `F-RCT-04-P1-05`; then persist distinct typed terminal reasons including interrupted/unknown and ToolFailure. Parent terminal should close orphan records as unknown/interrupted with evidence, not cancel. Delete generic `cancel_active_tools` use for failed/timed-out/completed paths.
- Regression validation: one active possible-side-effect Tool under user cancel, batch timeout, parent failure, normal parent completion, and process restart; require distinct outcomes, one terminal, and verify-before-retry facts.
- Validation reports: [V03](../validations/X-TOL-01/V03-01.md), [V04](../validations/X-TOL-01/V04-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Commits, dirty isolation, and accidental search disclosure | yes | inconclusive; excluded command | [V00](../validations/X-TOL-01/V00-01.md) |
| V01 | Requested/effective schema and field mapping | yes | failed/finding | [V01](../validations/X-TOL-01/V01-01.md) |
| V02 | Long-output, rich result, checksum, cursor, and artifact path | yes | V02-01 superseded; failed/finding | [V02-02](../validations/X-TOL-01/V02-02.md) |
| V03 | Error taxonomy and terminal-reason mapping | yes | failed/finding | [V03](../validations/X-TOL-01/V03-01.md) |
| V04 | Existing test coverage and edge-case inventory | yes | failed/gaps | [V04](../validations/X-TOL-01/V04-01.md) |
| V05 | Dependency ownership and finding deduplication | yes | passed | [V05](../validations/X-TOL-01/V05-01.md) |
| V06 | Invalid/timeout/cancel/partial-side-effect dynamic fixtures | future | not_run | [V06](../validations/X-TOL-01/V06-01.md) |
| V99 | Final report integrity and source-boundary gate | yes | V99-01 failed regex; V99-02 self-matched; V99-03 passed | [V99-03](../validations/X-TOL-01/V99-03.md) |
| V30 | Primary committed-source sampling and acceptance | yes | passed | [V30-01](../validations/X-TOL-01/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| F-EXT-01: rich ToolResult becomes string-only at live ReAct boundary | current | V02-02; EKO persistence impact is now traced end to end. |
| F-EXT-01: text spill/read_artifact is complete and hash-bound | current in framework | V02-02; EKO does not reuse that reader. |
| F-RCT-04: batch timeout lacks per-call terminal outcomes | current | V03; EKO further classifies unresolved calls as cancelled. |
| A-TOOL-01: channel drops ordinary Tool lifecycle/artifact facts | current, separate owner | V05; not duplicated here. |
| A-FE-02: durable Tool hydration otherwise merges terminal over running | current positive boundary | V05; terminal truth is wrong before hydration for X-TOL-01-P1-04. |

## Coverage And Uncertainty

- No Cargo/rustc/test/build/dynamic fixture/browser/network action ran. V06 is
  future regression specification, not proof of runtime timing.
- One `rg` accidentally read dirty framework bodies. V00 preserves the exact
  command and excludes its output; every cited framework anchor was separately
  reconstructed from commit `3aa7929`.
- CLI `Cargo.lock` became externally modified only at the final gate. It was not
  opened, modified, or used as evidence; all adopted CLI anchors are source
  files read while the repository was clean.
- Static reachability is conclusive for event field loss and repository branch
  selection. It does not measure how often users configure rewriting hooks or
  how often artifacts mutate after creation.
- Root framework defects remain under `F-EXT-01`/`F-RCT-04`; X-TOL findings
  describe cross-layer adapter/application behavior and must be fixed without
  adding a second authority.
- Changes to ToolManager Complete forwarding, AgentEvent Tool variants,
  `ToolExecutionRepository`, TaskRuntime Tool projection, or frontend detail IPC
  make this report stale.

## Handoff

- Define one rich canonical Tool invocation and terminal envelope before
  repairing individual surfaces. It must carry requested/effective input,
  typed failure/terminal, complete-output source, artifact hash/bytes, and
  cursor identity.
- Make EKO persistence a lossless projection of that envelope; remove the
  unreachable Complete side channel, raw-path reader, output-precedence guess,
  and parent-to-cancel inference after cutover.
- Preserve the existing framework artifact writer/read_artifact integrity
  checks and application lazy-loading UX.
- `Q-FLT-01` should execute V06 after remediation. The primary independently
  reconstructed all four findings from committed source in V30; this task is complete.
