# X-TOL-01: Tool error, artifact, and schema conformance

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63 (baseline 9b0e0fa)
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5 (baseline b3b2e81)
> Worktree state: clean in both repositories before and after every step
> (`git status --porcelain` empty; no command that regenerates
> `web-frontend/src/generated/*.ts` was executed; generated-dir untouched)

## Question

Does one tool invocation retain the same schema, classification, output
integrity, artifact metadata, and terminal reason across all layers —
framework Tool schema/result (`echo-agent`), EKO execution projection
(`ToolExecutionRepository`), Rust/Tauri surface, and TypeScript rendering?

**Answer: yes on the normal success/`ToolError` path — every field, the
7-way failure taxonomy, the SHA-256 artifact checksum, and the byte-cursor
paging survive all four layers 1:1 (V01-01, V02-01, V03-01); no on the kill
paths and in two projection spots** — when the framework abandons in-flight
tools (batch timeout / batch cancel / subagent timeout / turn-end cleanup),
the EKO boundary can only record a bare `Cancelled` with no failure
classification and no partial-side-effect warning (P2-01, root cause
canonically F-RCT-04-P1-02/P2-02); the GUI byte-count label for
artifact-spilled outputs is wrong (P3-01); and a failed tool's error message
text is dropped from the persisted detail when the tool streamed output
(P3-02). The other conformance breaks found are already filed with canonical
IDs (see V05-01 matrix): F-RCT-04-P1-01 (result ordering), F-EXT-01-P1-01 /
A-TOOL-01-P1-01 (writer subagent schema), F-EXT-02-P1-01 and F-EXT-03-P1-03
(invalid-input panics), F-EXT-02-P2-02 and F-EXT-03-P1-01 (fabricated
success), F-EXT-03-P2-02 (schema enum not enforced), A-SRF-02-P2-01 /
A-FE-02-P2-01 (double producer / detail_ref identity key).

## Scope

Cross-layer trace of one tool invocation, limited to the four layers of the
task card:

- Framework (echo-agent): `echo-core/src/tools/mod.rs` (ToolFailure taxonomy
  :17-254, ToolResult :288-317), `echo-core/src/llm/types.rs:584-614`
  (ToolDefinition), `echo-core/src/agent/mod.rs:194-230` (tool event family),
  `echo-core/src/tools/artifact.rs` (writer, checksum, cursor-free spill),
  `src/agent/react/run/phases/tools.rs:142-431` (batch arms, ToolError/ToolResult
  emission), `src/agent/react/run/pipeline.rs:599-616` (failure construction),
  `src/agent/react/run/stream_channel.rs` (driver), `src/agent/snapshot.rs:926-1060`
  (spill/preview), `echo-execution/src/tools.rs:618-728` (per-tool timeout/retry).
- EKO execution: `echo-agent-cli/echo-agent-app-core/src/tool_execution.rs`
  (full: repository, summary/manifest/page types, cursor paging),
  `chat_driver.rs` (error normalization :153,:530-555, task-mode compensation
  :340-383), `src/tauri/commands/chat.rs` (TauriChatSink :1148-1331,
  TauriExecutionProjector :957-1114, emit_tool_execution_summary :185-208),
  `src/tauri/mod.rs:353-768` (bridge -> ExecEvent),
  `src/tauri/commands/tool_executions.rs` (detail commands),
  `src/tui/events.rs:2054-2132` + `src/tui/mod.rs:425-452,720-740` (TUI path).
- Frontend: `web-frontend/src/types/api.ts:49-116`, `stores/toolExecutionStore.ts`,
  `hooks/useTauriChat.ts:109-150`, `api/endpoints.ts:456-464`,
  `components/chat/InlineToolCall.tsx` (full).
- Tests: framework (stream_channel, tools, echo-core tools),
  EKO (tool_execution.rs tests, chat.rs projector test), frontend
  (toolExecutionStore, InlineToolCall, chatStore.toolExecution), TUI
  (events.rs tests).

## Out Of Scope

- Tool-per-domain correctness beyond failure classification and artifact
  conformance (shell process handling, git semantics, data analysis
  correctness) -> F-EXT-02/F-EXT-03 (complete).
- Chat-turn lifecycle/status derivation and interrupt behavior -> A-SRF-03.
- Task/Subagent projection identity beyond the tool card -> A-FE-02
  (complete), X-TSK-01, X-EVT-01.
- Permission/HITL gating -> F-HITL-01, A-HITL-01, X-AUT-01.
- Live provider behavior and live network tools -> Q-FLT-01/Q-FLT-02,
  F-EXT-03 (not_run recorded there).

## Inputs

- Root `AGENTS.md` (full, in context), shared `README.md`, `REPORTING.md`,
  `TASKS.md` (X-TOL-01 card), `zcode-ds/README.md`, report templates.
- Dependency task reports read (all zcode-ds, complete): `F-RCT-04`,
  `F-EXT-01`, `F-EXT-02`, `F-EXT-03`, `A-TOOL-01`, `A-FE-02`; canonical IDs
  cross-referenced as cited therein (A-SRF-02-P2-01, A-SRF-03-P2-01,
  A-TSK-06-P2-01, F-REL-01).
- Historical documents treated as hypotheses: `echo-agent-cli/docs/
  MASTER-PLAN.md:98,:196-203`, root `docs/MASTER-PLAN.md:115/:275/:472/:602`
  (classified in the Historical Claim Status section).

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism (framework, correct) | Tool trait/schema, `ToolFailureCategory` 7-way taxonomy, `ToolResult`, artifact writer with SHA-256, per-tool timeout/retry classification, the event family (ToolCall/ToolStream/ToolResult/ToolError), cursor-free spill + `read_artifact`. All correctly placed in `echo-agent`; no movement proposed. |
| EKO product policy (application, correct) | `ToolExecutionRepository` (summary 4-way status + detail manifest), 160-char `args_preview`, 64 KiB page cap, JSONL journal + crash repair, `cancel()` kill-path normalization, TUI `failure_category` metadata strings, frontend collapsed-vs-expanded rendering. The repository's `cancel()` signature (no failure parameter) is the EKO-side defect P2-01. |
| Adapter boundary | chat.rs sink/projector and the TUI event mapper are thin, lossless adapters on the `ToolError` path (V01-01); the Tauri commands serialize structs unchanged. No second execution authority exists anywhere (V01-01; consistent with A-TOOL-01). |
| Duplicate search | Terms searched across both repositories: `ToolFailureCategory` (one definition), `ToolExecutionStatus` (EKO 4-way vs TUI local 4-way — two independent enums with identical variants; both derived from the framework event, not an authority split), `failure_category` metadata key (TUI only), `detail_ref`/`execution_key`, `artifact_sha256`/`output_bytes`, `cancel_active_tools` (chat.rs only; TUI uses its own in-memory cancel), `PartialSideEffect` (producers: files/files.rs:97 + from_error conversion only), `read_output`/`read_artifact_page` (one paging authority in EKO). Results: no duplicate schema, no second taxonomy, no second paging authority. |
| Migration deletion | P2-01 fix extends `cancel()`/adds a kill-record path — the old behavior is the current `cancel()`; deletion target is the failure-less Cancelled projection, replaced by a failure-carrying kill record. P3-01/P3-02 fix `finish()` bookkeeping only. |

## Current Path

Verified data flow (V01-01, V02-01, V03-01): LLM tool call -> framework
`ToolCall {call_id, name, args}` -> `run_tools` batch phase
(phases/tools.rs:142-431) -> per tool: `ToolManager` (timeout/retry gating,
echo-execution/src/tools.rs:618-728) -> `ToolResult` (spill + preview,
snapshot.rs:926-1060) -> `ToolStream::Complete` (metadata/truncated captured)
then `ToolResult`/`ToolError {error, failure}` events
(phases/tools.rs:207-264, :395-410). EKO GUI sink:
`ToolCall` -> `start()` (fresh detail_ref, tool_execution.rs:191-258);
`ToolStream::Output` -> `append_output` (channel-mapped JSONL chunks,
chat.rs:1231-1248); `Complete` -> pending metadata/truncated
(chat.rs:1249-1262); `ToolResult` -> `finish(success=true, output)`
(chat.rs:1264-1292); `ToolError` -> `finish(success=false, error_text,
Some(failure))` (chat.rs:1293-1316); `Cancelled|Error` -> `cancel_active_tools`
(chat.rs:1325-1328). Subagent path: bridge (mod.rs:353-768) -> ExecEvent ->
`TauriExecutionProjector::project_tool_event` (chat.rs:957-1114) -> same
repository. Tauri commands serve `get_tool_execution_detail` /
`read_tool_execution_output` (commands/tool_executions.rs:16-56). Frontend:
`execution://event` (kind=tool) -> `toolExecutionStore.ingest`
(useTauriChat.ts:132-137); `InlineToolCall` lazy-loads manifest + paged
output via byte cursor (InlineToolCall.tsx:61-118, 244-258); failure rendered
as `category · recovery` in the expanded view (:214-218). TUI: framework
events mapped at events.rs:2054-2132, failure -> `failure_category`/
`recovery_action` metadata strings (events.rs:721-755), rendered by
`tool_metadata_label` (mod.rs:728-740). Terminal/kill exits: batch timeout
(phases/tools.rs:284-292) and batch cancel (:295-300) abandon futures without
any `ToolError`; EKO compensates by `cancel()` (status Cancelled only) on
`AgentEvent::Error`/`Cancelled` (chat.rs:1325-1328), on non-running
`TurnStatus` (chat.rs:1365-1368), on subagent
`Completed|Failed|Cancelled|TimedOut` (chat.rs:1106-1110), and at boot
recovery for Running records (tool_execution.rs:527-545).

## Findings

### X-TOL-01-P2-01: Tool terminal-reason taxonomy collapses to a bare "cancelled" at the EKO boundary on every kill path — `cancel()` cannot record a timeout, a failure, or a partial side effect, so the 7-way framework classification is unreachable for aborted tools

- Priority: P2
- Confidence: high (full static chain in both repositories; no dynamic
  kill-path run — read-only review)
- Layer: application (EKO repository/projection boundary); root cause shared
  with framework F-RCT-04-P1-02/P2-02
- Evidence: `echo-agent-cli/echo-agent-app-core/src/tool_execution.rs:360-392`
  — `cancel(owner, call_id)` sets `ToolExecutionStatus::Cancelled` only; no
  `failure`, no metadata, no reason parameter; `tool_execution.rs:527-545`
  — boot recovery also converts Running -> Cancelled without a failure;
  `echo-agent-cli/src/tauri/commands/chat.rs:1106-1110` —
  `RuntimeEventKind::Completed | Failed | Cancelled | TimedOut =>
  cancel_active_tools` (a subagent TIMEOUT therefore marks its in-flight
  tools "cancelled"); `chat.rs:1325-1328` — `AgentEvent::Cancelled |
  AgentEvent::Error` (the only signal a batch timeout produces, via the
  envelope adapter, echo-core/src/agent/event_envelope.rs:136-139) ->
  `cancel_active_tools`; `chat.rs:1365-1368` — any non-running `TurnStatus`
  -> `cancel_active_tools`; framework side produces no `ToolError` for
  batch-killed tools (phases/tools.rs:284-300) — canonical F-RCT-04-P1-02 /
  P2-02; the per-call `finish` path that WOULD carry the taxonomy is only
  reachable from `ToolError` events (chat.rs:1293-1316).
- Reachability: definition -> registration -> live callers: every batch
  timeout (EKO default batch budget ~360.9 s, run/retry.rs:69-107), every
  user cancellation mid-batch, every subagent timeout/cancel with in-flight
  tools, and every turn-end cleanup with still-running tools on the EKO main
  GUI path; plus process-restart recovery of any Running record.
- Expected invariant: MASTER-PLAN:98 "tools have explicit success/failure/
  cancelled terminal states" and the task's conformance question — the
  terminal reason must be recoverable and distinguishable at every layer;
  a timeout or a possibly-partially-executed tool is never silently
  re-labeled as a plain user cancel.
- Observed behavior: all kill paths converge on `cancel()`; the persisted
  summary says `cancelled` and the detail manifest has `failure: None`.
  A user cannot distinguish "I cancelled" from "the batch timed out" or
  "the subagent timed out"; no partial-side-effect or postcondition warning
  ever reaches the GUI/TUI for aborted tools. The GUI collapsed row shows
  "cancelled" (InlineToolCall.tsx:139-146), the expanded view shows no
  failure line (:214-218 renders nothing when `failure` is null).
- Impact: mislabeled terminal reasons on the flagship surface for a routine
  class of exits; for a local coding agent the missing partial-side-effect
  warning is a correctness hazard (a timed-out `edit_file`/`shell` may have
  applied part of its work and neither the model nor the user is told —
  F-RCT-04-P2-02 consequence that the EKO boundary cannot express even if
  the framework fixed it); observability (A-OBS-01, X-STA-01) and any
  replay/analysis see the same collapse.
- Root cause: `cancel()` was written as a pure status flip when the only
  kill path was user cancellation; the framework's later batch/subagent kill
  paths (which produce no `ToolError`) were absorbed into the same method
  with no classification parameter, and the EKO boundary never grew a
  "record an aborted execution with a reason" API.
- Direction: extend the kill record path to accept an optional reason —
  add `failure: Option<ToolFailure>` (and optionally `metadata`) to
  `cancel()`/a new `abandon()` and map the sources: user cancel ->
  `Cancelled`; batch timeout -> `Timeout` with `side_effect: Possible` for
  write tools; subagent `TimedOut` -> `Timeout`; boot recovery keeps
  `Cancelled`. Keep the 4-way `ToolExecutionStatus` (summary) but always
  populate `ToolExecutionDetailManifest.failure` from the reason. The
  framework-side terminal emission remains F-RCT-04-P1-02/P2-02's deletion
  target.
- Regression validation: projector fixture — subagent `TimedOut` with an
  in-flight write tool -> summary `Failed` (or Cancelled) with
  `failure.category == timeout` and `side_effect == possible`; chat fixture —
  batch timeout -> in-flight tool manifest carries the timeout failure;
  boot-recovery fixture unchanged (Running -> Cancelled); TUI/GUI rendering
  fixture asserting the failure line appears for the aborted tool.
- Validation reports: [V02-01](../validations/X-TOL-01/V02-01.md),
  [V05-01](../validations/X-TOL-01/V05-01.md)

### X-TOL-01-P3-01: `ToolExecutionDetailManifest.output_bytes` never reflects artifact-spilled output — the GUI shows "输出 · 0 B" for a multi-megabyte artifact

- Priority: P3
- Confidence: high (code fact; exact sites verified)
- Layer: application (EKO projection)
- Evidence: `echo-agent-cli/echo-agent-app-core/src/tool_execution.rs:312-330`
  — `finish()` skips appending the result and never updates `output_bytes`
  when `artifact_available` (metadata `artifact_path` present and file
  exists); `tool_execution.rs:394-411` — `detail_manifest` serves
  `output_bytes: manifest.output_bytes`; the artifact byte counts live only
  in `metadata` (`artifact_bytes`/`artifact_payload_bytes`, inserted by
  `extend_metadata`, echo-core/src/tools/artifact.rs:100-119); GUI label
  `web-frontend/src/components/chat/InlineToolCall.tsx:204-207`
  (`输出 · {formatBytes(manifest.output_bytes)}{manifest.truncated ? '
  · Agent 上下文已截断' : ''}`).
- Reachability: any tool whose output crosses the 1 MiB spill threshold
  (artifact.rs:16) on the GUI path — routine for shell/log/output tools.
- Expected invariant: the manifest's `output_bytes` equals the complete
  persisted output size (the artifact payload) so the size label is honest.
- Observed behavior: `output_bytes` stays 0 (or the streamed-chunk count)
  while the artifact holds the full output; the GUI shows "输出 · 0 B ·
  Agent 上下文已截断" and the true size is only visible inside the raw
  metadata `<details>` block.
- Impact: misleading output-size reporting on the tool detail surface; the
  complete content itself is intact and paged correctly (V03-01), so this
  is an honesty defect, not data loss.
- Root cause: `finish()` tracks `output_bytes` only on the JSONL-append
  branch; the artifact branch never merges `artifact_payload_bytes`.
- Direction: in `finish()`, when the artifact is available, set
  `output_bytes = artifact_payload_bytes` (or `artifact_bytes` for the
  label-with-channels variant) from the metadata; add a spill fixture
  asserting the served manifest `output_bytes` matches the artifact size.
- Regression validation: EKO unit test — `finish()` with metadata carrying
  `artifact_path` (existing file) + `artifact_payload_bytes=10485760` ->
  `detail_manifest(...).output_bytes == 10485760`; GUI render fixture
  asserting "10.0 MiB" for that manifest.
- Validation reports: [V03-01](../validations/X-TOL-01/V03-01.md)

### X-TOL-01-P3-02: A failed tool's error message is not persisted when the tool streamed output — the GUI detail shows stdout/stderr and the failure category but never the error text

- Priority: P3
- Confidence: high (code fact)
- Layer: application (EKO projection)
- Evidence: `echo-agent-cli/echo-agent-app-core/src/tool_execution.rs:316-330`
  — `finish()` appends the `result` argument (for failures: the
  `ToolError.error` text, passed from `chat.rs:1293-1316`; the framework
  emits it at phases/tools.rs:244-248) only when `!execution.has_output &&
  !artifact_available`; `has_output` is set by any non-empty streamed chunk
  (`append_output`, tool_execution.rs:288); GUI renders only chunks +
  `failure.category · recovery` (InlineToolCall.tsx:214-239).
- Reachability: any tool that streams stdout/stderr and then fails — common
  for `shell` (stderr), `git`-family, and long-running tools on the GUI path.
- Expected invariant: the persisted detail of a failed tool contains the
  error message (the complete-execution promise of
  `tool_execution.rs:1-5`); the failure reason must not depend on whether
  the tool happened to stream.
- Observed behavior: the detail file holds the streamed chunks and the
  manifest failure category/recovery, but the error text is absent; the
  model-facing context does carry "[Error] ..." (phases/tools.rs:252-256),
  so the loss is confined to the GUI detail view.
- Impact: the user cannot see why a streamed tool failed without reading the
  model's context; the GUI failure card is incomplete.
- Root cause: the "don't duplicate streamed output" optimization was applied
  to the error text too — the error message is not part of the stream.
- Direction: in `finish()`, always persist a failure result — append the
  `result` argument as a `Result`-channel chunk on the failure branch
  regardless of `has_output` (streamed chunks and the error message are
  distinct facts); keep the success-branch optimization. Add a fixture:
  tool with streamed stdout then `ToolError` -> detail page contains both
  the stdout chunk and the error text.
- Regression validation: EKO unit test — `append_output(stdout)` then
  `finish(false, "permission denied", Some(failure))` ->
  `read_output` returns the stdout chunk AND a `Result` chunk containing
  "permission denied"; frontend render fixture for the same.
- Validation reports: [V03-01](../validations/X-TOL-01/V03-01.md),
  [V04-01](../validations/X-TOL-01/V04-01.md)

No further findings: the normal-path field mapping, the 7-way taxonomy
propagation, the SHA-256/artifact-metadata survival, and the cursor paging
are conformant (V01-01, V02-01, V03-01); all other conformance breaks are
canonically filed elsewhere (V05-01).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Field mapping of one invocation across framework schema -> EKO execution -> Rust surface -> TS rendering (field-by-field, identity, serde tags) | yes | passed | [V01-01](../validations/X-TOL-01/V01-01.md) |
| V02 | Error taxonomy: `ToolFailureCategory` (7-way) -> EKO errors -> GUI/TUI rendering, including kill-path audit | yes | passed (P2-01 evidence) | [V02-01](../validations/X-TOL-01/V02-01.md) |
| V03 | Long-output checksum/cursor: SHA-256 survival, artifact metadata, byte-cursor paging, UTF-8 boundaries, size reporting | yes | passed (P3-01/P3-02 evidence) | [V03-01](../validations/X-TOL-01/V03-01.md) |
| V04 | Invalid/timeout/cancel/partial-side-effect fixture inventory + executed frontend suites (`npx vitest run src/stores/toolExecutionStore.test.ts src/components/chat/InlineToolCall.test.tsx`) | yes | passed (exit 0; 12 passed; gap inventory recorded) | [V04-01](../validations/X-TOL-01/V04-01.md) |
| V05 | Cross-reference with existing findings (canonical ID consistency matrix) + historical-document drift | yes | passed | [V05-01](../validations/X-TOL-01/V05-01.md) |

All required validations executed; every reported command has a known exit
code; no validation is pending.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `echo-agent-cli/docs/MASTER-PLAN.md:98` — tools have explicit success/failure/cancelled terminal states | regressed for kill paths | kill paths end as bare Cancelled with no failure (tool_execution.rs:360-392; chat.rs:1106-1110,1325-1328); P2-01; V02-01 |
| Root `docs/MASTER-PLAN.md:115/:275/:602` — oversized results as complete artifact + bounded preview, shared path/size/SHA-256/retention | current | artifact.rs writer + `extend_metadata` + manifest preservation (V03-01); size label defect P3-01 |
| Root `docs/MASTER-PLAN.md:472` (M4) — unified failure classification with partial-side-effect category | current at manager level / bypassed on kill paths | F-RCT-04-P2-02 + P2-01 (V05-01) |
| `echo-agent-cli/docs/MASTER-PLAN.md:196-203` — frontend terminal states monotonic | regressed only on the live ingest path | A-FE-02-P2-01 canonical (V05-01) |

## Coverage And Uncertainty

- All conclusions are static traces plus one executed frontend suite (V04-01,
  exit 0, 12 passed); no live tool run, no GUI process, no provider call was
  executed (read-only review). Framework test green state at these commits is
  carried from the dependency reports (F-RCT-04 V04-01/02, F-EXT-01 V04-01/02,
  A-TOOL-01 V04-01..04).
- P2-01's reachability includes batch timeout and subagent timeout paths
  whose live frequency depends on EKO configuration (per-tool 120 s default
  bounds most tools before the ~360.9 s batch timer; subagent timeouts are a
  documented lifecycle path); the collapse is code-fact regardless of
  frequency.
- The CLI/channel surfaces were not re-traced beyond the shared
  `drive_chat`/sink contract (A-SRF-04 owns them); their tool rendering
  inherits the same framework events and the same repository.
- The `SqlQueryTool` readonly-surface trade-off and `web_fetch` family
  duplication are F-EXT-03-scope and were not re-audited.
- P3-01/P3-02 were verified statically at exact sites; no end-to-end spill
  run was executed (no source modification allowed).

## Handoff

- Downstream tasks may rely on: the normal-path four-layer conformance
  (V01-01/V02-01/V03-01) — field mapping, taxonomy propagation, checksum
  survival, cursor paging are intact; the kill-path classification collapse
  (P2-01) is the EKO-side complement of F-RCT-04-P1-02/P2-02; the two
  projection defects (P3-01 output_bytes, P3-02 error-text loss); the
  fixture gap inventory (V04-01) — cancel is covered everywhere, timeout
  only at the framework manager level, invalid-arguments and partial side
  effects have no cross-layer fixture; the canonical consistency matrix
  (V05-01).
- Reports to read: this report + V01-01..V05-01; dependency reports
  F-RCT-04, F-EXT-01..03, A-TOOL-01, A-FE-02 (all complete).
- Stale triggers: changes to `tool_execution.rs` (cancel/finish/
  detail_manifest/read_output), `chat.rs` sink/projector arms, the bridge
  (mod.rs) payload fields, `phases/tools.rs` kill arms, `artifact.rs`
  metadata keys, `InlineToolCall` rendering, or the TUI event mapper
  invalidate the corresponding claims.
- Follow-up task IDs (fixes are not implemented in this review):
  Q-FLT-01 (build the missing fixture families from F-RCT-04-P2-01 +
  V04-01's list), X-EVT-01 (terminal-reason conformance across consumers),
  S-RDM-01 (roadmap items for P2-01, P3-01, P3-02 with the canonical merge
  note F-RCT-04-P1-02/P2-02), Q-TST-01 (frontend failure-category and
  artifact-size rendering tests).
