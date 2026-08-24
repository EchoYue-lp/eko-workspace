# X-TOL-01: Tool error, artifact, and schema conformance

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: read-only review. `echo-agent` has uncommitted modifications
> in `src/agent/react/run/{phases/tools.rs,pipeline.rs,stream_channel.rs}` and
> `src/testing/{mock_llm.rs,mock_tool.rs,mod.rs}` — a different agent's in-flight
> work on the concurrent-batch ordering fix (F-RCT-04-P3-01) and mock-harness
> expansion. Every framework citation below was taken from
> `git show 9b0e0fa:...`; the dirty paths do not affect this report.
> `echo-agent-cli` is clean.

## Question

Does one tool invocation retain the same schema, classification, output
integrity, artifact metadata, and terminal reason across all layers
(model → framework `Tool` → EKO projection → Tauri command → frontend
store)?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent/echo-core/src/tools/mod.rs:1-540` — `Tool` trait,
  `ToolResult`, `ToolResultKind`, `ToolFailure`, `ToolFailureCategory`,
  `ToolRecoveryAction`, `ToolSideEffect`, `ToolStreamEvent`,
  `ToolExecutionConfig`. The framework's typed tool contract.
- `echo-agent/echo-core/src/agent/mod.rs:135-310` — `AgentEvent` enum,
  especially `ToolCall` / `ToolResult` / `ToolError` / `ToolStream` /
  `ToolBatchStart` / `ToolBatchEnd`. The framework's event boundary.
- `echo-agent/echo-core/src/tools/artifact.rs:1-170` —
  `ToolOutputArtifactRef` (incl. `extend_metadata` / `from_metadata`),
  `ToolOutputArtifactWriter` (sha256 hasher), spill-file layout.
- `echo-agent/src/agent/snapshot.rs:920-1070, 1185-1280` —
  `process_tool_output_for_call` (spill + truncation policy),
  `execute_tool_with_policy` (the framework's per-call `String`-returning
  boundary; the load-bearing drop point for V01).
- `echo-agent/src/agent/react/run/pipeline.rs:460-756` — `ExecuteStage`
  (streaming vs non-streaming dispatch; `ctx.result = Some(result)`),
  `TruncationStage` (enriches `ctx.result.truncated` + `.metadata`).
- `echo-agent/src/agent/react/run/phases/tools.rs:115-440` (at `9b0e0fa`) —
  the batch orchestration that emits `AgentEvent::ToolResult|ToolError` and
  the `FuturesUnordered` completion-order push (F-RCT-04-P3-01).
- `echo-agent-cli/src/tauri/commands/chat.rs:957-1332` — the two EKO
  observers: `TauriExecutionProjector::project_tool_event` (Subagent /
  TaskRuntime path) and `TauriChatSink::handle_tool_event` (Chat path);
  the `pending_tool_completions` merge for streaming tools.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:3273-3456`
  — `AgentEvent` → `ExecEvent` payload conversion in the subagent path;
  the dual projection (trace-sink + store) for tool completion.
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:1975-2034, 2857-2895`
  — `record_tool_finished` (writes `ToolCompleted` / `ToolFailed` into
  `events.jsonl`) and the `tool_failure_boundary_persists_recovery_contract`
  regression test.
- `echo-agent-cli/echo-agent-app-core/src/tool_execution.rs:1-460, 579-997` —
  `ToolExecutionRepository` (`start` / `append_output` / `finish` /
  `cancel` / `read_output` / `detail_manifest`), `StoredManifest`, JSONL +
  artifact cursor pagination, UTF-8 boundary repair, the five unit tests.
- `echo-agent-cli/echo-agent-app-core/src/analysis.rs:866-875` —
  `run_status` (the single flattening consumer of `ToolFailureCategory`).
- `echo-agent-cli/web-frontend/src/types/api.ts:49-116` — hand-written TS
  mirror of `ToolExecution*` / `ToolFailure*`.
- `echo-agent-cli/web-frontend/src/api/endpoints.ts:456-467` —
  `toolExecutionApi` (`list` / `detail` / `readOutput`).
- `echo-agent-cli/web-frontend/src/stores/toolExecutionStore.ts:1-117` —
  dual identity scheme, merge policy.
- `echo-agent-cli/web-frontend/src/components/chat/InlineToolCall.tsx:23, 60-269`
  — the lazy reader, `LIVE_DETAIL_AUTOLOAD_CHARS = 256 KiB`, the
  `manifest.truncated` label, the `manifest.failure.{category,recovery}`
  render.
- `echo-agent-cli/src/tauri/commands/tool_executions.rs:1-58` — the three
  Tauri commands (`list_tool_executions`, `get_tool_execution_detail`,
  `read_tool_execution_output`) that bridge EKO → frontend.

## Out Of Scope

Deferred to named task IDs:

- **F-EXT-01 / F-EXT-02 / F-EXT-03**: the framework `Tool` / `ToolResult` /
  `ToolFailure` contract correctness and individual tool semantics. This
  task consumes their conclusions; it does not re-audit tool internals.
- **F-RCT-04**: the framework's per-batch tool orchestration (concurrency,
  timeout, cancellation, retry). This task treats F-RCT-04's
  `AgentEvent::ToolError` emission as the input to the cross-layer
  conformance audit.
- **A-TOOL-01**: EKO's per-mode tool exposure, sandbox selection, and the
  GUI PTY terminal. This task consumes A-TOOL-01's `ToolExecutionRepository`
  conclusions (pure projection, no scheduling authority).
- **A-FE-02**: frontend reducer identity, monotonicity, and lazy output.
  This task consumes A-FE-02's conclusions about `toolExecutionStore.ingest`
  (the live-overwrite defect A-SRF-03-P2-01) and `InlineToolCall`'s lazy
  reader; it does not re-audit reducer policy.
- **F-LLM-01..03**: per-provider tool-call wire serialization (how
  `parameters()` JSON Schema is mapped onto OpenAI / Anthropic tool-call
  wire formats). The model-side schema is touched only where it appears in
  the `ToolResult` → `AgentEvent` boundary.
- **A-BOOT-01 / B-PATH-01**: multi-mode parity gaps (TUI / channel
  surfaces). This task audits the GUI path; the same conformance questions
  apply to other surfaces but are noted, not audited.

## Inputs

Required repository documents read in full:

- Repository root `AGENTS.md` — framework-vs-application layering gate;
  "first prove no duplicate exists" implementation gate; the "Schema
  conformance / artifact metadata / terminal reason" expectation implied
  by the prompt-driven-over-state-machine rule; no-panic / UTF-8 safety;
  dead-code cleanup rule; multi-mode parity rule.
- `docs/comprehensive-review/REPORTING.md`.
- `docs/comprehensive-review/templates/task-report.md`,
  `templates/validation-report.md`.
- `docs/comprehensive-review/TASKS.md` (X-TOL-01 card and the
  `F-RCT-04`, `F-EXT-01..03`, `A-TOOL-01`, `A-FE-02` dependency list).

Dependency reports read:

- `zcode-glm/tasks/F-EXT-01.md` (complete) — the `Tool` / `ToolResult` /
  `ToolFailure` contract is the single typed tool surface; the default
  `category → recovery` mapping; `PartialSideEffect`/`Timeout` route to
  `VerifyThenRetry`; cursor pagination (`PageRequest` / `PageInfo`) and
  artifact spill (`ToolOutputArtifactWriter`) live in `echo-core`.
- `zcode-glm/tasks/F-EXT-02.md` (complete) — `ShellTool` and `RunCodeTool`
  are the streaming tools; `WriteFileTool` emits `PartialSideEffect` +
  `idempotency_key` + `postcondition` on failure; `EditFileTool` /
  `UpdateFileTool` / `AppendFileTool` have gaps in partial-side-effect
  reporting.
- `zcode-glm/tasks/F-EXT-03.md` (complete) — `web_search` / `sql_query`
  consume `PageRequest` and `ToolOutputArtifactWriter` correctly; research
  tools neither paginate nor classify transient failures; the SSRF/image
  paths are correct.
- `zcode-glm/tasks/F-RCT-04.md` (complete) — `phases::tools::run_tools` is
  the single batch orchestrator; pairing is by `tool_call_id`;
  `execute_tool_with_policy` is the per-call seam that reduces
  `Result<String, ToolCallFailure>`; the batch layer emits
  `AgentEvent::ToolResult|ToolError` from the per-call result.
- `zcode-glm/tasks/A-TOOL-01.md` (complete) — the runtime-event observer
  in `chat.rs:960-1180` is a thin projection adapter; the
  `ToolExecutionRepository` is a pure projection (no scheduling authority,
  no second cancellation path); framework `CancellationToken` is the
  single execution-cancellation authority.
- `zcode-glm/tasks/A-FE-02.md` (complete) — `InlineToolCall` is pull-based,
  cursor-paginated, 256 KiB-capped for live, with manual "load more";
  `toolExecutionStore.ingest` is NOT monotone (A-SRF-03-P2-01); the
  hydrate/merge path IS monotone; the dual-identity scheme (`tool.id` vs
  `executionIdentity`) is the structural root.

Historical documents treated as hypotheses:

- A-TOOL-01 V04's claim that "two-layer output bounding" keeps
  "no unbounded output in the frontend or model context". Treated as a
  claim to re-verify end-to-end (V03).
- A-FE-02's `InlineToolCall.tsx:23` comment treating
  `LIVE_DETAIL_AUTOLOAD_CHARS = 256 KiB` as the lazy-load bound. Verified
  current by V03.
- The AGENTS.md prompt-driven-over-state-machine rule (informed by the
  Claude Code / Codex research) — treated as the design intent behind
  `VerifyThenRetry` being advisory metadata, not a framework gate.

## Layering Decision

This is a **cross-layer conformance** task. It audits the seams between
three layers that are individually owned by framework and application
tasks, but whose joint behavior no single layer task verifies.

| Classification | Required answer |
|---|---|
| Generic mechanism | The framework `ToolResult` / `ToolFailure` taxonomy, `AgentEvent::ToolResult|ToolError|ToolStream`, `ToolOutputArtifactWriter`, and the `process_tool_output_for_call` spill policy are all correctly generic framework primitives (verified by F-EXT-01 / F-RCT-04). The framework is NOT the source of the cross-layer drop documented in X-TOL-01-P2-01 — it is a *consequence* of the `execute_tool_with_policy` return type being `Result<String, ToolCallFailure>`. The drop is therefore a design seam, not a defect local to one layer. |
| EKO product policy | The EKO `ToolExecutionRepository` projection (`tool_execution.rs`), the two observers in `chat.rs`, the `pending_tool_completions` merge, the `LIVE_DETAIL_AUTOLOAD_CHARS = 256 KiB` cap, the hand-written TS mirror in `types/api.ts:49-116`, and the `toolExecutionApi` endpoint shapes are all EKO product policy, correctly in `echo-agent-cli`. None belongs in the framework. |
| Adapter boundary | The framework's `execute_tool_with_policy` → `Result<String, ToolCallFailure>` is the single load-bearing seam: it is where the typed `ToolResult` becomes a `String` on success. The EKO `TauriChatSink::handle_tool_event` and `TauriExecutionProjector::project_tool_event` are thin adapters that forward what reaches them — they hold no scheduling authority (A-TOOL-01 verified). The `pending_tool_completions` merge is the adapter-side workaround that recovers streaming-tool metadata; it is correct but asymmetric. |
| Duplicate search | Searched both repos for: `execute_tool_with_policy` (1 live definition, called by `phases::tools::run_tools` only — F-RCT-04 confirmed), `ToolExecutionRepository` (1 definition, 1 field on `AppState.storage` — A-TOOL-01 confirmed), `TauriChatSink` / `TauriExecutionProjector` (1 each, in `chat.rs`), `toolExecutionApi` (1 definition, `endpoints.ts:456`), `ToolFailure` consumption in TS (1 hand-written type, 1 reader in `InlineToolCall.tsx:214`), `pending_tool_completions` / `PendingToolCompletion` (1 definition each, used identically by both observers). No parallel implementation of any conformance seam. |
| Migration deletion | No deletion proposed in this task. The findings identify seams and test gaps, not dead code; resolution is left to follow-up task IDs. |

## Current Path

Verified cross-layer data flow at the cited commits. The chain has five
hops; the load-bearing transformation happens at hop 2.

```text
MODEL TOOL CALL ─ provider tool_call{id, name, arguments} ────────────────────┐
                                                                               │
HOP 1: FRAMEWORK TOOL EXECUTES (F-EXT-01/02/03 owned)                          │
  Tool::execute_with_context(params, ctx) -> Result<ToolResult, ReactError>    │
  ToolResult { kind, success, output, error, failure, bytes, data,             │
               truncated, mime_type, metadata }          [echo-core/mod.rs:288]│
  (For streaming tools: ToolStreamEvent::Progress|Output|Complete(ToolResult)  │
   forwarded by ExecuteStage through stream_tx → batch stream_rx.)             │
                                                                               │
HOP 2: FRAMEWORK PIPELINE + PER-CALL BOUNDARY (F-RCT-04 owned)                 │
  ExecuteStage: ctx.result = Some(result.clone())        [pipeline.rs:620]     │
  TruncationStage: enriches ctx.result.truncated + .metadata + ctx.output      │
                                              [pipeline.rs:743-753]            │
  execute_tool_with_policy -> Result<String, ToolCallFailure>                  │
      on Ok:  ctx.output.unwrap_or(result.output)       [snapshot.rs:1242]     │
              ↑ drops ctx.result.kind/.data/.bytes/.mime_type;                  │
                for streaming tools, truncated+metadata recovered via           │
                ToolStream::Complete channel (separate AgentEvent)              │
      on Err: ToolCallFailure { error, failure }        [snapshot.rs:1248-1255] │
                                                                               │
HOP 3: FRAMEWORK AgentEvent (echo-core owned)                                  │
  ToolResult { call_id, name, output: String }          [echo-core/agent/mod.rs:203] │
  ToolError  { call_id, name, error: String, failure: ToolFailure }            │
                                              [echo-core/agent/mod.rs:212]     │
  ToolStream { call_id, name, event: ToolStreamEvent }  (parallel channel)     │
                                              [echo-core/agent/mod.rs:223]     │
  ToolBatchStart / ToolBatchEnd                                                  │
                                                                               │
HOP 4: EKO OBSERVER (application; A-TOOL-01 owned)                             │
  Chat path:     TauriChatSink::handle_tool_event       [chat.rs:1193-1331]    │
    ToolCall      -> tool_executions.start(...)        [chat.rs:1209-1223]      │
    ToolStream::Complete(result) -> pending_completions[id] =                  │
                                    {metadata: result.metadata,                │
                                     truncated: result.truncated}              │
                                              [chat.rs:1253-1260]              │
    ToolResult    -> finish(true, output, None, pending.metadata,              │
                              pending.truncated)       [chat.rs:1270-1278]      │
    ToolError     -> finish(false, error, Some(failure.clone()),               │
                              pending.metadata, pending.truncated)              │
                                              [chat.rs:1302-1310]              │
  Subagent path: TauriExecutionProjector::project_tool_event                   │
                                              [chat.rs:957-1114]               │
    Same pending_tool_completions pattern; payload is ExecEvent JSON.          │
    (Separately, store.record_tool_finished writes ToolCompleted/ToolFailed     │
     into events.jsonl with bounded 500-char result_preview + result_chars +    │
     failure. Dual projection; GUI observer reads trace-sink payload, not store.)│
                                                                               │
HOP 5: EKO REPOSITORY + TAURI COMMAND                                          │
  ToolExecutionRepository.finish(success, result, failure, metadata, truncated)│
      -> StoredManifest { failure, metadata, truncated, output_bytes, ... }     │
                                              [tool_execution.rs:293-343]      │
  detail_manifest -> ToolExecutionDetailManifest { failure, metadata,           │
      truncated, output_bytes, ... } (strips "artifact_path" from metadata)    │
                                              [tool_execution.rs:394-411]      │
  read_output(cursor, limit) -> ToolExecutionDetailPage { chunks,               │
      next_cursor, complete }   paginates JSONL OR artifact file                │
                                              [tool_execution.rs:413-450]      │
  Tauri commands: list_tool_executions / get_tool_execution_detail /            │
                  read_tool_execution_output                                   │
                                              [tool_executions.rs:15-57]       │
                                                                               │
HOP 6: FRONTEND STORE + COMPONENT (A-FE-02 owned)                              │
  toolExecutionStore.ingest (live, by tool.id)          [toolExecutionStore.ts:206]│
  toolExecutionStore.hydrateConversation (by executionIdentity)                 │
                                              [toolExecutionStore.ts:223]      │
  InlineToolCall.loadPage -> detail + readOutput(cursor)                       │
                                              [InlineToolCall.tsx:61-117]      │
  renders manifest.truncated label / manifest.failure.{category,recovery} /     │
          manifest.metadata / chunks                                          │
                                              [InlineToolCall.tsx:188-260]     │
```

### Cross-layer field preservation (full table)

The headline conformance question — does one tool invocation retain the
same schema, classification, output integrity, artifact metadata, and
terminal reason across all layers? — is answered field-by-field:

| ToolResult field | Hop 1 framework | Hop 2 per-call | Hop 3 AgentEvent | Hop 4-5 EKO | Hop 6 frontend | Verdict |
|---|---|---|---|---|---|---|
| `output` (text) | yes | yes | yes (ToolResult) | yes | yes | preserved |
| `error` (text) | yes | yes | yes (ToolError) | yes | yes | preserved |
| `failure.category` | yes | yes | yes (ToolError) | yes | yes | preserved |
| `failure.recovery` | yes | yes | yes | yes | yes | preserved |
| `failure.side_effect` | yes | yes | yes | yes | yes | preserved |
| `failure.retry_after_ms` / `idempotency_key` / `postcondition` | yes | yes | yes | yes | yes | preserved |
| `truncated` (model-side) | yes (TruncationStage) | **dropped at snapshot.rs:1242** | no | recovered for streaming tools only (via `pending_completions`) | asymmetric | **X-TOL-01-P2-01** |
| `metadata` (spill: sha256, artifact_path, output_handling, original_bytes) | yes (TruncationStage) | **dropped at snapshot.rs:1242** | no | recovered for streaming tools only | asymmetric | **X-TOL-01-P2-01** |
| `kind` | yes | dropped | no | no | no | dropped (F-EXT-01-P3-01) |
| `data` (structured JSON) | yes | dropped | no | no | no | dropped (F-EXT-01-P3-01) |
| `bytes` | yes | dropped | no | no | no | dropped (rarely used) |
| `mime_type` | yes | dropped | no | no | no | dropped |

**Net answer**: classification (category/recovery/side_effect), terminal
reason (success vs failure status), and the human-readable output/error
text survive end-to-end with full fidelity. The model-side output
integrity signals (`truncated`, `metadata`) and the rich payload fields
(`kind`, `data`, `bytes`, `mime_type`) do NOT survive the framework
boundary for non-streaming tools; streaming tools recover `truncated` +
`metadata` via the parallel `ToolStream::Complete` channel. The artifact
itself (file + sha256) is always written to disk by the framework; the
structured handle to it (`artifact_path` metadata) reaches the frontend
only for streaming tools.

### Spill / sha256 path (V03 detail)

The framework computes a real `sha256` of the spilled payload inside
`ToolOutputArtifactWriter` (a `sha2::Sha256` hasher fed every written
byte; `artifact.rs:8,152-156`). The same hash is then exposed on two
surfaces:

1. **Model-facing output text** (always): `snapshot.rs:970-974` embeds
   "...Full output artifact: {path} ({size} MiB, sha256 {hash}). Use
   read_artifact with this exact path and expected_sha256 to retrieve
   bounded pages until complete." The model can call `read_artifact`
   with the textual pointer to verify and retrieve bounded pages.
2. **Structured metadata** (for streaming tools only at the frontend):
   `artifact.extend_metadata` (`artifact.rs:101-119`) populates
   `artifact_sha256`, `artifact_path`, `artifact_bytes`,
   `artifact_payload_bytes`, `artifact_retention`, `artifact_kind`,
   `artifact_media_type`, `artifact_status`. These reach the EKO
   `manifest.metadata` via the `pending_tool_completions` merge
   (streaming only). The EKO `detail_manifest` strips `artifact_path`
   before returning to the frontend (`tool_execution.rs:401`) — the
   frontend sees `artifact_sha256` and the byte counts but not the path
   itself.

For non-streaming spilled tools (e.g. a large `web_fetch` body, a big
`sql_query` result, or `read_file` on a very large file), the spill file
is on disk and model-reachable via `read_artifact`, but the EKO
`read_output` API cannot reach it (no `artifact_path` in metadata), the
`manifest.truncated` flag is `false`, and the GUI lazy reader paginates
only the JSONL that holds the preview+pointer text.

### Cursor pagination layers

There are two independent cursor pagination mechanisms, not one:

1. **Framework in-tool pagination** (`echo-core/tools/pagination.rs`):
   `PageRequest::from_parameters` + `paginate(items, &query_identity)`.
   Used inside individual tool payloads (`web_search`, `sql_query`). The
   cursor binds `offset` to a SHA-256 fingerprint of `{query, limit,
   items snapshot}` and rejects stale cursors
   (`CursorQueryMismatch`). Exposed as JSON inside `ToolResult.data`.
   Correct and bounded (F-EXT-01 V03, F-EXT-03 V03).
2. **EKO projection pagination** (`tool_execution.rs:413-450`):
   `read_output(cursor, limit)` paginates the JSONL output OR the
   artifact file. Opaque byte-offset cursor; UTF-8 boundary repair
   (`utf8_page_prefix`). The frontend `InlineToolCall` consumes
   `next_cursor` and passes it to the next `readOutput` call.

The two mechanisms do not interact; a `web_search` call's
`PageInfo.next_cursor` lives inside the tool's structured `data`, while
the EKO `read_output` cursor pages through the tool's textual output
log. Both are correct; they solve different problems.

### Dual projection in the TaskRuntime path

The same subagent tool invocation produces two parallel records:

- `store.record_tool_finished` writes `events.jsonl` with
  `RuntimeEventKind::ToolCompleted` (success) or `ToolFailed` (failure),
  payload `{result_preview (bounded 500 chars), result_chars, failure}`
  (`store.rs:2002-2034`).
- `executor.emit_exec(...)` emits an `ExecEvent` with
  `RuntimeEventKind::ToolCompleted` (for both success and failure —
  success distinguished by `success: true`), payload `{result (full
  string), success, failure?}` (`executor.rs:3326-3341, 3375-3391`).

The GUI observer reads the **trace-sink** payload
(`chat.rs:1034-1105`), not the store payload. The two disagree on (a)
event kind for failures (`ToolFailed` vs `ToolCompleted`) and (b)
whether the result is bounded (500 chars vs full string). Not a
per-field corruption — both carry `failure` losslessly — but a
conformance complexity for any downstream consumer that joins the two
records by `call_id`.

## Findings

### X-TOL-01-P2-01: Non-streaming tools lose `truncated` + spill `metadata` at the framework `execute_tool_with_policy` boundary; the GUI lazy reader cannot reach the spilled artifact content

- Priority: P2
- Confidence: high
- Layer: framework (root cause) + application (asymmetric recovery)
- Evidence:
  - `echo-agent/src/agent/snapshot.rs:1242` (commit `9b0e0fa`) —
    `execute_tool_with_policy` returns `Ok(ctx.output.unwrap_or(result.output))`
    on success. The signature is `Result<String, ToolCallFailure>`.
    `ctx.result` (the full `ToolResult`, enriched with `truncated` +
    spill `metadata` by `TruncationStage` at `pipeline.rs:743-753`) is
    in scope but not returned.
  - `echo-agent/echo-core/src/agent/mod.rs:203-210` —
    `AgentEvent::ToolResult` carries only `{call_id, name, output:
    String}`. The drop is structural at the framework event boundary.
  - `echo-agent-cli/src/tauri/commands/chat.rs:1264-1278` — the GUI
    chat sink's `AgentEvent::ToolResult` handler calls `finish(true,
    output, None, completion.metadata, completion.truncated)`.
    `completion` comes from `pending_tool_completions.remove(call_id)`,
    which is populated ONLY by `AgentEvent::ToolStream::Complete(result)`
    at `chat.rs:1249-1262`. A non-streaming tool never emits that
    variant, so `completion.metadata` is `HashMap::new()` and
    `completion.truncated` is `false`.
  - `echo-agent-cli/echo-agent-app-core/src/tool_execution.rs:413-450`
    — `read_output` consults `manifest.metadata.get("artifact_path")`
    to paginate a spill file. With empty metadata, it falls through to
    JSONL output (which holds the preview+pointer text, not the full
    payload).
  - `echo-agent-cli/web-frontend/src/components/chat/InlineToolCall.tsx:206`
    — the `"Agent 上下文已截断"` label is gated on
    `manifest.truncated`, which is `false` for non-streaming spilled
    tools.
- Reachability: every non-streaming tool whose output exceeds
  `DEFAULT_MAX_TOOL_OUTPUT_TOKENS` (4000 tokens) OR the 32 KiB artifact
  threshold. The non-streaming families are: file tools (read_file,
  write_file, edit_file, list_dir), git tools, data/research/media/web
  tools (`web_fetch` spills at `web/fetch.rs:281-318`; `sql_query`
  spills at `database.rs:494-537`), and any MCP tool that does not opt
  into streaming. This is the majority of the tool surface.
- Expected invariant: a tool result whose payload was spilled to a
  sha256-tagged artifact should expose both (a) a `truncated` signal
  and (b) a structured handle to the artifact, to every consumer that
  reads the result — not just the model.
- Observed behavior: streaming tools (`shell`, `run_code`) recover
  `truncated` + spill `metadata` via the parallel
  `ToolStream::Complete` channel and the `pending_tool_completions`
  merge. Non-streaming tools lose both at the framework boundary; the
  EKO projection stores only the bare output String, with the spill
  pointer embedded textually but no structured `artifact_path`. The
  user sees the pointer text in the GUI but cannot page into the full
  payload from the lazy detail panel.
- Impact: medium. Not data loss in the strict sense — the artifact is
  on disk and the model can `read_artifact` it — but the GUI's lazy
  reader (the primary user-facing surface for tool output) is unable
  to reach the spilled content, and the `truncated` label silently
  mis-reports the output as complete. For a local assistant whose
  pitch is "supervise your agent" (AGENTS.md), this is a real
  supervision defect on the common non-streaming path. It is also a
  cross-layer conformance break: the same tool invocation presents
  different integrity signals depending on whether it streams.
- Root cause: the `execute_tool_with_policy` return type predates the
  `TruncationStage` enrichment of `ctx.result`. When the truncation
  policy was centralized in the pipeline, the per-call return type was
  not widened to carry the post-truncation metadata; only the model
  path (which reads the output string with the embedded pointer) was
  updated. The streaming recovery was added later via
  `pending_tool_completions` and works because streaming tools emit a
  separate `Complete(result)` event that the GUI sink can intercept.
- Direction: two options.
  (a) **Widen the framework seam** (cleaner): change
    `execute_tool_with_policy` to return
    `Result<ToolResult, ToolCallFailure>` (or
    `Result<ProcessedToolOutput, ToolCallFailure>`) and update
    `AgentEvent::ToolResult` to carry `truncated` + `metadata` (or the
    full `ToolResult`). The batch layer already has access to
    `ctx.result`; the change is mechanical. This makes streaming and
    non-streaming tools symmetric. Cost: touches the public
    `AgentEvent` enum (a breaking wire change for any consumer reading
    `output` as the only field), so it needs a coordinated framework +
    CLI bump.
  (b) **Recover at the EKO adapter** (smaller blast radius): have the
    EKO sink parse the spill pointer out of the output text (it is
    already there: "Full output artifact: {path} ... sha256 {hash}")
    and synthesize the `artifact_path` / `artifact_sha256` metadata +
    the `truncated` flag for non-streaming tools. This treats the
    framework seam as frozen and works around it at the projection
    layer. Cost: text-parsing in the adapter (brittle if the framework
    ever changes the pointer wording); but bounded and reversible.
  Prefer (a) if a framework minor-version bump is acceptable;
  otherwise (b) until the framework seam is widened.
- Regression validation: a cross-layer fixture (see X-TOL-01-P2-03)
  that drives a non-streaming spilled tool result through the full
  framework → EKO → frontend path and asserts both
  `manifest.truncated == true` and that `read_output` reaches the
  spilled artifact content via `manifest.metadata["artifact_path"]`
  (under option (a)) or via the parsed pointer (under option (b)).
- Validation reports: [V01](../validations/X-TOL-01/V01-01.md),
  [V03](../validations/X-TOL-01/V03-01.md).

### X-TOL-01-P2-02: Tool-execution wire types (`ToolFailure`, `ToolExecutionDetailManifest`, etc.) are hand-written TypeScript, not generated from the Rust source — adding a Rust enum variant would not break the TS build

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/web-frontend/src/types/api.ts:55-78` —
    `ToolFailureCategory`, `ToolRecoveryAction`, `ToolFailure` are
    defined as TS literal unions / interfaces with hand-written
    snake_case variants.
  - `echo-agent-cli/web-frontend/src/types/api.ts:80-116` —
    `ToolExecution`, `ToolExecutionDetailManifest`,
    `ToolExecutionDetailChannel`, `ToolExecutionDetailChunk`,
    `ToolExecutionDetailPage` are hand-written TS interfaces.
  - `echo-agent-cli/web-frontend/src/generated/` — directory listing
    contains `TaskExecutionSummary.ts`, `ToolCallInfo.ts`,
    `ToolInfo.ts`, `ToolSource.ts`, but NO `ToolFailure.ts`,
    `ToolFailureCategory.ts`, `ToolExecutionSummary.ts` (EKO's own
    summary, distinct from the task-runtime's
    `TaskExecutionSummary`), `ToolExecutionDetailManifest.ts`, etc.
  - `echo-agent-cli/echo-agent-app-core/src/tool_execution.rs:63-150`
    — `ToolExecutionSummary`, `ToolExecutionDetailManifest`,
    `ToolExecutionDetailChannel`, `ToolExecutionDetailChunk`,
    `ToolExecutionDetailPage` are `#[derive(Serialize, Deserialize)]`
    but do NOT have `#[derive(TS)]` with `#[ts(export, rename = ...)]`.
  - Contrast: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs:604-606`
    — `RuntimeEventKind` is `#[derive(Debug, Clone, Copy, PartialEq, Eq,
    Serialize, Deserialize, TS)]` with `#[ts(export, rename =
    "RuntimeEventKind")]`, producing `generated/RuntimeEventKind.ts`.
    Same pattern for `SubagentTaskResult`, `RecoveryBlocker`,
    `ReviewResult`. A-FE-01 established this as the gold-standard IPC
    contract.
- Reachability: every tool-execution IPC call
  (`list_tool_executions`, `get_tool_execution_detail`,
  `read_tool_execution_output`) — the Rust types serialize as JSON and
  the frontend hand-written types interpret them. Any drift between the
  Rust enum and the TS union is invisible to the compiler.
- Expected invariant: per AGENTS.md's "implementation gate"
  (first prove no duplicate / single authority), a wire contract
  between Rust and TypeScript should have ONE definition site, not
  two. The task-runtime family achieves this; the tool-execution
  family does not.
- Observed behavior: the hand-written TS mirror currently matches the
  Rust source variant-for-variant (verified V02). But adding a new
  `ToolFailureCategory` variant (e.g. `QuotaExceeded`) in Rust would
  serialize correctly at runtime (the frontend reads it as a string),
  while the TS union would silently lack the literal — no compile
  error, no type-narrowing arm. The same applies to new manifest
  fields.
- Impact: medium-low. The current state is correct; the risk is
  future drift. Combined with X-TOL-01-P2-01, the tool-execution
  family has both a structural drop and a non-generated contract —
  together they make the cross-layer conformance fragile.
- Root cause: the tool-execution projection was added before the
  `#[derive(TS)]` convention was standardized on the task-runtime
  family (per A-FE-01's gold-standard note). It was not retrofitted.
- Direction: add `#[derive(TS)]` with `#[ts(export, rename = ...)]` to
  `ToolFailure`, `ToolFailureCategory`, `ToolRecoveryAction`,
  `ToolSideEffect`, `ToolExecutionSummary`,
  `ToolExecutionDetailManifest`, `ToolExecutionDetailChannel`,
  `ToolExecutionDetailChunk`, `ToolExecutionDetailPage`, and
  `ToolExecutionOwner` in `tool_execution.rs` (and the corresponding
  framework types in `echo-core/tools/mod.rs` if they are re-exported
  through the IPC boundary). Delete the hand-written copies in
  `types/api.ts` and import from `generated/`. This makes drift a
  compile error.
- Regression validation: `npx tsc --noEmit` passes after the swap;
  the existing `toolExecutionStore.test.ts` (9 tests) and
  `InlineToolCall.test.tsx` still pass. The
  `ts-rs`-generated `generated/ToolFailure.ts` is diffed against the
  former hand-written copy to confirm variant-for-variant equality
  before deletion.
- Validation reports: [V02](../validations/X-TOL-01/V02-01.md).

### X-TOL-01-P2-03: No end-to-end fixture drives any of {invalid-args, timeout, cancel, partial-side-effect} through the framework → EKO → frontend path; the `PartialSideEffect` store test asserts only `failure.category`

- Priority: P2
- Confidence: high
- Layer: application (test gap)
- Evidence: full per-family coverage matrix in
  [V04-01](../validations/X-TOL-01/V04-01.md). Headline gaps:
  - `echo-agent-cli/echo-agent-app-core/src/tool_execution.rs:834-1075`
    — five unit tests; none calls `finish(false, ..., Some(failure),
    ...)` or `cancel(...)`. The `failure` / `truncated` / `metadata`
    code paths in `finish` are exercised in production but not by any
    repository-level test.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs:2857-2895`
    — `tool_failure_boundary_persists_recovery_contract` constructs a
    `PartialSideEffect` failure with `.with_postcondition("verify
    target hash")` but asserts ONLY `event.payload.failure.category
    == "partial_side_effect"`. The `recovery`, `side_effect`, and
    `postcondition` sub-fields are not asserted (the construction sets
    them; the test does not verify they survive).
  - `echo-agent-cli/src/tauri/commands/chat.rs:1193-1332` — the GUI
    sink has no test driving `AgentEvent::ToolError` through
    `handle_tool_event` into `tool_executions.finish(false, ..., Some
    (failure), ...)`. `grep -rn 'AgentEvent::ToolError'
    echo-agent-cli --include='*.rs'` finds the production sites
    (`chat.rs:1293`, `executor.rs:3343`) and the `surface_contract.rs`
    contract-sample (which constructs a `Timeout` `ToolError` but
    does not drive it through the sink).
  - `echo-agent-cli/web-frontend/src/components/chat/InlineToolCall.test.tsx`
    — the three tests ingest single tools and check the summary
    rendering; none ingests a `failed` tool with a `failure` manifest
    and asserts `manifest.failure.category` / `manifest.failure.recovery`
    rendering.
- Reachability: the cross-layer conformance that X-TOL-01 audits is
  precisely the path with no end-to-end test. A regression that, for
  example, drops `failure.postcondition` at the `ToolCallFailure` →
  `AgentEvent::ToolError` hop, or that mis-maps `partial_side_effect`
  to a wrong `recovery` in the TS mirror, would not be caught.
- Expected invariant: each safety-critical edge case
  (`PartialSideEffect` especially, because it carries the
  verify-before-retry semantic) should have at least one test that
  drives it through the full pipeline and asserts every field that
  the contract promises to preserve.
- Observed behavior: framework batch tests (F-RCT-04, 29 tests) cover
  the framework-internal behavior; EKO repository tests cover the
  success path; frontend store tests cover reducer monotonicity with
  synthetic records. The seam between them — where the framework's
  `ToolFailure` becomes the frontend's `manifest.failure` — has no
  fixture.
- Impact: medium. The static inspection (V01, V02) shows the fields
  ARE forwarded losslessly today; the defect is the absence of a
  regression-catching test. Combined with the asymmetric drop in
  X-TOL-01-P2-01 and the non-generated contract in X-TOL-01-P2-02,
  the tool-execution conformance has no automated guard.
- Root cause: each layer's tests were written by the layer's owning
  task (F-RCT-04 framework, A-TOOL-01 EKO repository, A-FE-02
  frontend store) without a cross-layer integration test. The X-TOL
  track exists precisely to fill this gap; this finding confirms it
  has not been filled yet.
- Direction: add a cross-layer fixture (likely in
  `echo-agent-cli/echo-agent-app-core/tests/` or as a `#[cfg(test)]`
  module in `chat.rs`) that constructs an `AgentEvent::ToolError`
  with each of the four categories (`InvalidArguments`, `Timeout`,
  `Cancelled`, `PartialSideEffect`) and drives it through a
  `TauriChatSink`-equivalent adapter into a real
  `ToolExecutionRepository`, asserting the resulting
  `detail_manifest.failure` carries `category`, `recovery`,
  `side_effect`, and (for `PartialSideEffect`)
  `postcondition`/`idempotency_key`. Pair with an `InlineToolCall`
  vitest that renders a failed tool with a manifest and asserts the
  `category · recovery` label.
- Regression validation: the four new tests must FAIL if any of the
  cited fields is dropped at any hop (mutation-testing-friendly).
- Validation reports: [V04](../validations/X-TOL-01/V04-01.md).

### X-TOL-01-P3-01: `analysis.rs::run_status` flattens `ToolFailureCategory` (7 variants) to `AnalysisRunStatus` (3 values); `recovery` / `side_effect` / `postcondition` discarded

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/analysis.rs:866-875`:
    ```rust
    fn run_status(result: &echo_agent::tools::ToolResult) -> AnalysisRunStatus {
        if result.success { return AnalysisRunStatus::Succeeded; }
        match result.failure.as_ref().map(|failure| failure.category) {
            Some(ToolFailureCategory::Cancelled) => AnalysisRunStatus::Cancelled,
            Some(ToolFailureCategory::Timeout) => AnalysisRunStatus::TimedOut,
            _ => AnalysisRunStatus::Failed,
        }
    }
    ```
    `InvalidArguments`, `Unavailable`, `Transient`, `Permanent`, and
    `PartialSideEffect` all collapse to `Failed`. `recovery`,
    `side_effect`, `retry_after_ms`, `idempotency_key`, and
    `postcondition` are entirely discarded.
  - `AnalysisRunStatus` is consumed by `tauri/commands/analysis.rs:6`
    and `cli/cmd_impls/analysis.rs:9` — the analysis/diagnostic
    subsystem (EKO workspace-eval report status), NOT the live
    tool-execution UI.
- Reachability: every analysis-run status computation. The analysis
  subsystem is a developer-facing diagnostic surface
  (`workspace_root_for_agent`, eval reports), not the primary
  tool-execution projection.
- Expected invariant: a downstream consumer that needs only a coarse
  status is free to flatten — but the flattening should be documented
  and the finer taxonomy should remain available to the consumer if
  it later needs it.
- Observed behavior: the flattening is silent (no comment explaining
  why 7 variants collapse to 3). A future analysis feature that wants
  to distinguish "permanent" from "transient" failures would have to
  re-introduce the finer taxonomy at the call site.
- Impact: low. The flattening is in a secondary diagnostic surface;
  the live tool-execution path (V02) preserves the full taxonomy. No
  correctness defect.
- Root cause: `AnalysisRunStatus` was designed as a coarse
  report-level enum; the mapping was written when only Cancelled and
  Timeout needed distinct treatment.
- Direction: either (a) leave as-is with a one-line comment
  documenting the intentional flattening (cheapest), or (b) if
  analysis ever needs finer granularity, widen `AnalysisRunStatus`
  and map each `ToolFailureCategory` variant distinctly. Not a fix
  target for this review.
- Regression validation: doc-only under (a).
- Validation reports: [V02](../validations/X-TOL-01/V02-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Field mapping: every `ToolResult` field traced model → framework → EKO → Tauri → frontend; drops named | yes | passed (with finding) | [V01-01](../validations/X-TOL-01/V01-01.md) |
| V02 | Error taxonomy: `ToolFailureCategory` / `ToolRecoveryAction` survive end-to-end; flattening consumers confined | yes | passed (with finding) | [V02-01](../validations/X-TOL-01/V02-01.md) |
| V03 | Long-output checksum / cursor: real sha256; UTF-8-safe cursor pagination; spill-reach asymmetry | yes | passed (with finding) | [V03-01](../validations/X-TOL-01/V03-01.md) |
| V04 | Invalid / timeout / cancel / partial-side-effect fixtures: end-to-end coverage exists per family | yes | failed | [V04-01](../validations/X-TOL-01/V04-01.md) |
| V05 | Historical-document drift | conditional | n/a | No prior X-TOL-01 report exists in this reviewer directory; the three dependency-report claims treated as hypotheses are classified inline in the Inputs section (all current). |

No `cargo` or `npx vitest` command was executed in this review. All
cited test outcomes (F-RCT-04's 29 framework tests, A-FE-02's 26-file /
101-test vitest run, A-TOOL-01's repository observations) are taken
from the dependency reports at the cited commits.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| A-TOOL-01 V04 "two-layer output bounding... no unbounded output is loaded into the frontend or model context" | partially current | Model-facing bounding (framework spill + truncation) is correct and survives end-to-end (the spill pointer is in the output text). GUI-facing bounding (EKO `read_output` + `manifest.truncated`) is correct ONLY for streaming tools; non-streaming spilled tools lose the structured handle (X-TOL-01-P2-01). The model context IS bounded; the GUI's reach into spilled content is asymmetric. |
| A-FE-02 `InlineToolCall.tsx:23` `LIVE_DETAIL_AUTOLOAD_CHARS = 256 KiB` "the live cap" | current | V03 confirms the cap is enforced at `InlineToolCall.tsx:92-99` and surfaced at `:137-138, 244-248`. |
| AGENTS.md "prompt-driven-over-state-machine" / Claude Code / Codex research rule | current | The framework's `VerifyThenRetry` is advisory metadata, not a runtime gate (F-RCT-04-P3-02). This task confirms the advisory nature survives the cross-layer hop: `manifest.failure.recovery` reaches the frontend as a display label, not as a behavioral gate. |
| F-EXT-01 "ToolFailure carries category + recovery" | current (load-bearing) | V02 confirms the failure path carries `category`, `recovery`, `side_effect`, `retry_after_ms`, `idempotency_key`, `postcondition` losslessly through every hop to the frontend manifest. |
| F-EXT-02 "ShellTool and RunCodeTool are the streaming tools; WriteFileTool emits PartialSideEffect + idempotency_key + postcondition" | current | This task relies on the streaming/non-streaming split for X-TOL-01-P2-01. The `PartialSideEffect` emission of `WriteFileTool` (files.rs:678-685, 700-707 per F-EXT-02) reaches the framework `ToolResult.failure`, but (per V01) the failure path is the ONE path that survives the framework boundary losslessly. |
| F-RCT-04 "execute_tool_with_policy is the thin seam that maps pipeline result into `Result<String, ToolCallFailure>`" | current (load-bearing) | This is the exact seam where X-TOL-01-P2-01 places the drop. F-RCT-04 noted the seam is "thin"; this task confirms it is thin AND lossy for the success path's `truncated` / `metadata` fields. |

## Coverage And Uncertainty

Inspected in full: the cross-layer pipeline from `Tool::execute` through
to `InlineToolCall` rendering, with every hop's source anchor cited. The
field-preservation table is exhaustive for the `ToolResult` struct's 10
fields. The four validations cover the four required facets (field
mapping, error taxonomy, long-output handling, edge-case fixtures).

Not inspected (out of scope or deferred):

- **TUI projection** of the same tool-execution state. The TUI
  (`src/tui/`) consumes `RuntimeEventKind` / `RuntimeTaskEvent` but does
  not have an `InlineToolCall`-equivalent lazy reader for the
  `ToolExecutionRepository` (per A-TOOL-01-P3-02 / A-BOOT-01, the TUI
  has gaps in surface parity). Whether the TUI preserves
  `manifest.truncated` / `failure.category` is not audited here.
- **Channel / cron surfaces**. Same caveat — out of scope for the GUI
  conformance audit.
- **Per-provider tool-call schema serialization** (`parameters()` JSON
  Schema → OpenAI / Anthropic wire). Owned by F-LLM-01..03; this task
  did not verify that the model receives a schema that round-trips to
  the same `ToolParameters` map. F-EXT-01 V01 established the contract;
  it is consumed as-is.
- **MCP tool failure classification**. The framework's MCP error →
  `ToolFailureCategory` mapping (`echo-core/tools/mod.rs:227-251`) was
  spot-checked by F-EXT-01 V04; this task did not re-verify each
  `code: -32xxx` branch.
- **The `read_artifact` tool** (the model's verifier for spilled
  content). Referenced in the spill pointer text; not separately
  audited. Its bounded-page contract is part of F-EXT-02.

Environmental constraints:

- Read-only review against `echo-agent` `9b0e0fa` (with the noted dirty
  paths bypassed via `git show`) and `echo-agent-cli` `b3b2e81` (clean).
  No `cargo` / `npx` command was executed in this review; test outcomes
  are cited from F-RCT-04 / A-FE-02 / A-TOOL-01.

Uncertain claims:

- Whether ANY user has hit the X-TOL-01-P2-01 spill-reach gap in
  practice. The gap is structural (deterministic for non-streaming
  spilled tools), but its user-visible frequency depends on how often
  non-streaming tools produce output above the 4000-token / 32 KiB
  thresholds in real sessions. No bug report was searched.
- Whether option (a) (widen `execute_tool_with_policy` return type) in
  X-TOL-01-P2-01 is acceptable as a framework minor-version bump. The
  `AgentEvent::ToolResult` variant is part of the public framework
  event contract; changing it is a breaking change for any
  `echo-agent` consumer. The decision belongs to the framework owner.
- Whether the in-flight dirty diff in `echo-agent` (the
  `phases/tools.rs` rewrite that buffers concurrent results in call
  order, plus the `mock_tool`/`mock_llm` expansion) lands before or
  after a fix for X-TOL-01-P2-01. The diff does NOT touch
  `snapshot.rs:1242` (the actual drop point), so the finding stands
  regardless; but if the diff adds new framework tests for the
  concurrent path, the V04 coverage gap may partially close.

## Handoff

Conclusions downstream tasks may rely on:

1. **`ToolFailure` survives the failure path losslessly.** Every hop
   from `ToolResult.failure` to `AgentEvent::ToolError.failure` to
   `tool_executions.finish(false, ..., Some(failure), ...)` to
   `manifest.failure` to the frontend `ToolFailure` TS type preserves
   `category`, `recovery`, `side_effect`, `retry_after_ms`,
   `idempotency_key`, and `postcondition`. Downstream tasks auditing
   agent error handling, HITL retry UI, or eval scoring can rely on the
   frontend seeing the full failure taxonomy (V02).
2. **The success path loses `truncated` + spill `metadata` at the
   framework boundary for non-streaming tools (X-TOL-01-P2-01).** Any
   downstream task that depends on the GUI lazy reader reaching spilled
   content, or on `manifest.truncated` being accurate, must account for
   this asymmetry. Streaming tools (`shell`, `run_code`) recover the
   metadata via `pending_tool_completions`; non-streaming tools do not.
3. **The spill `sha256` is a real hasher output, not a text digest.**
   It is exposed both in the model-facing output text (always) and in
   the structured metadata (streaming tools only at the frontend).
   Tasks that need to verify artifact integrity can rely on the
   framework-computed `artifact_sha256` (V03).
4. **Cursor pagination is split across two independent mechanisms.**
   Framework in-tool pagination (`PageRequest` / `PageInfo`, bound to a
   SHA-256 fingerprint of the query+items) lives inside `ToolResult.data`
   for `web_search` / `sql_query`. EKO projection pagination
   (`read_output(cursor, limit)`) pages through the textual output log
   or the spill file. Tasks should not conflate the two.
5. **The tool-execution wire types are hand-written TS, not generated
   (X-TOL-01-P2-02).** Tasks adding new `ToolFailureCategory` variants
   or manifest fields must update BOTH the Rust source and the
   hand-written `types/api.ts:49-116`. Migrating to `#[derive(TS)]` is
   the durable fix.

Reports downstream tasks must read:

- This report (X-TOL-01) for the cross-layer field preservation matrix
  and the dual-projection observation.
- `tasks/F-EXT-01.md` for the framework `Tool` / `ToolResult` /
  `ToolFailure` contract that this task consumes.
- `tasks/F-RCT-04.md` for the framework's batch orchestration,
  `execute_tool_with_policy` (the seam where X-TOL-01-P2-01 places the
  drop), and the conservative retry / cancellation / timeout behavior.
- `tasks/A-TOOL-01.md` for the EKO `ToolExecutionRepository` (pure
  projection, no scheduling authority) and the observer bridge.
- `tasks/A-FE-02.md` for the frontend reducer identity, the
  `toolExecutionStore.ingest` live-overwrite defect (A-SRF-03-P2-01),
  and the `InlineToolCall` lazy reader.

Conditions that make this report stale:

- Widening `execute_tool_with_policy` to return `Result<ToolResult,
  ToolCallFailure>` (or adding `truncated` / `metadata` to
  `AgentEvent::ToolResult`) invalidates X-TOL-01-P2-01 and the V01 /
  V03 field tables.
- Migrating the tool-execution wire types to `#[derive(TS)]` (resolving
  X-TOL-01-P2-02) invalidates the V02 "hand-written TS mirror"
  observation.
- Adding cross-layer fixtures for the four edge-case families
  (resolving X-TOL-01-P2-03) changes V04 from `failed` to `passed`.
- Any change to the spill-pointer text format
  ("Full output artifact: {path} ... sha256 {hash}") invalidates
  option (b) of X-TOL-01-P2-01.
- Any change to `analysis.rs::run_status` (resolving X-TOL-01-P3-01)
  invalidates the V02 flattening-consumer observation.

Follow-up task IDs (no fixes implemented in this review):

- A **framework seam-widening task** should decide between options (a)
  and (b) for X-TOL-01-P2-01. This is the highest-value fix: it makes
  streaming and non-streaming tools symmetric and restores GUI reach
  into spilled content. Pairs with F-RCT-04 (the framework owns the
  seam) and A-TOOL-01 (the application owns the projection).
- A **tool-execution IPC contract generation task** should resolve
  X-TOL-01-P2-02 by migrating the wire types to `#[derive(TS)]` and
  deleting the hand-written `types/api.ts:49-116` mirror. Pairs with
  A-FE-01 (which established the gold standard for the task-runtime
  family).
- A **cross-layer fixture task** should resolve X-TOL-01-P2-03 by
  adding the four end-to-end tests (invalid-args / timeout / cancel /
  partial-side-effect) through `TauriChatSink` into
  `ToolExecutionRepository`. Pairs with Q-FLT-01 (the fault-injection
  suite) and Q-TST-01 (the test-credibility audit).
- **X-STA-01** (persistence, recovery, identity continuity) should
  consume the dual-projection observation (store vs trace-sink) and
  verify that the two records can be rejoined by `call_id` after a
  restart.
