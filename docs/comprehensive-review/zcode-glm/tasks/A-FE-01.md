# A-FE-01: Rust/TypeScript API and event type contract

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa (read-only; `ToolFailure`, `ToolFailureCategory`, `ToolRecoveryAction`, `ToolSideEffect`, `Message`)
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only review)

## Question

Do Tauri command DTOs, emitted payloads, TypeScript endpoint types, and
stores match field-for-field and variant-for-variant?

## Scope

Primary source paths and behaviors inspected:

- **Rust DTO / enum sources (app-core)**:
  - `echo-agent-cli/echo-agent-app-core/src/types/response.rs` (full,
    213 lines) — ts-rs-exported DTOs: `ChatResponse`, `ToolCallInfo`,
    `ContextStats`, `ToolInfo`, `ToolSource`, `McpServerInfo`,
    `McpConnectionStatus`, `McpToolInfo`, `SkillInfo`, `SkillSource`,
    `AgentConfigResponse`, `FullConfigResponse` + family, `SessionInfo`,
    `ServerMessage`.
  - `echo-agent-cli/echo-agent-app-core/src/types/request.rs` —
    request DTOs (read via grep for `TS` derives).
  - `echo-agent-cli/echo-agent-app-core/src/tool_execution.rs`
    (1-200, 830-980) — manual DTOs: `ToolExecutionOwner`,
    `ToolExecutionStatus`, `ToolExecutionSummary`,
    `ToolExecutionDetailManifest`, `ToolExecutionDetailChannel`,
    `ToolExecutionDetailChunk`, `ToolExecutionDetailPage`; plus the
    UTF-8 safe preview test (`preview_is_utf8_safe`), journal-repair
    test, and pagination test.
  - `echo-agent-cli/echo-agent-app-core/src/persistence.rs` (1-220) —
    `SavedMessage`, `SavedAttachment`, `SavedExecutionStep`,
    `SavedExecutionRound`, `AttachmentsPayload`, `SavedToolCall`,
    `ConversationRecord`.
  - `echo-agent-cli/echo-agent-app-core/src/model_config.rs` (1-90) —
    `ProviderTemplate`, `ConfiguredModelView`.
  - `echo-agent-cli/echo-agent-app-core/src/evolution/evidence.rs`
    (1-220) — `EvidenceSource`, `EvidenceKind`, `EvidenceScope`,
    `EvidenceRef`, `EvidenceAction`, `EvidenceCandidateStatus`,
    `EvidenceTarget`, `EvidenceCandidate`.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs`
    (360-560) — `TodoStatus`, `TaskRunStatus`, `ReviewOutcome` (and
    spot-check of the `RuntimeEventKind` / `RuntimeTaskEvent` /
    `TaskRun` / `TaskPlan` / `TodoItem` regions).
  - `echo-agent/echo-core/src/tools/mod.rs` (1-130) — framework
    `ToolFailureCategory`, `ToolRecoveryAction`, `ToolSideEffect`,
    `ToolFailure`.
- **Rust Tauri command emitters**:
  - `echo-agent-cli/src/tauri/commands/chat.rs` (1-220, 1341-1411) —
    `ChatEvent` enum + `emit_chat_event` + `emit_execution_event` +
    `emit_tool_execution_summary` + `TauriChatSink::on_event`
    ordering.
  - `echo-agent-cli/src/tauri/commands/tools.rs` (full) — 4 tool
    commands returning `serde_json::to_value(Vec<ToolInfo>)`.
  - `echo-agent-cli/src/tauri/commands/panels.rs` (340-500) —
    `list_skills` + `get_skill` returning hand-built JSON via
    `skill_descriptor_json` / `hub_skill_json`.
  - `echo-agent-cli/src/tauri/commands/mcp.rs` (1-100) —
    `list_mcp_servers` returning hand-built JSON.
  - `echo-agent-cli/src/tauri/commands/conversations.rs` (369-541) —
    `list_conversations`, `save_conversation`, `get_conversation`
    returning hand-built JSON.
  - `echo-agent-cli/src/tauri/commands/files.rs` (1-100) — `FileEntry`,
    `FileContent`, `DiffLine`, `DiffHunk`, `TreeNode`, `BrowseResult`
    struct DTOs.
  - `echo-agent-cli/src/tauri/commands/tool_executions.rs` (full) —
    3 commands returning typed Rust structs.
  - `echo-agent-cli/src/tauri/commands/task_runtime.rs` (signatures
    only — argument names + return types via grep).
  - `echo-agent-cli/src/tauri/mod.rs` (335-770) — the subagent event
    bridge that hand-builds `execution://event` payloads per
    `SubagentEvent` variant.
- **Frontend contract**:
  - `echo-agent-cli/web-frontend/src/types/api.ts` (full, 797 lines) —
    every manual type, plus the re-export bridge to `generated/`.
  - `echo-agent-cli/web-frontend/src/api/endpoints.ts` (1-200, 405-615,
    740-830) — every `*Api` object and the inline TS types it
    declares.
  - `echo-agent-cli/web-frontend/src/api/client.ts` — HTTP fallback
    (read in A-SRF-03, not re-audited).
  - `echo-agent-cli/web-frontend/src/stores/subagentRunStore.ts`
    (1-200, 300-450) — the `ExecutionEvent` interface (the wire
    contract for `execution://event` kind="subagent"), the
    `STORED_SUBAGENT_EVENTS` set, the `taskRuntimeSubagentExecutionEvents`
    projection adapter.
  - `echo-agent-cli/web-frontend/src/stores/toolExecutionStore.ts`
    (head, 1-120) — the live + hydrate reducer entry points.
  - `echo-agent-cli/web-frontend/src/generated/` — full directory
    inventory (80 files) and `index.ts` barrel.
  - `echo-agent-cli/web-frontend/src/generated/ToolInfo.ts`,
    `SkillInfo.ts`, `McpServerInfo.ts`, `TodoItem.ts`, `TodoStatus.ts`,
    `TaskRunStatus.ts`, `RuntimeEventKind.ts`, `RuntimeTaskEvent.ts`,
    `TaskExecutionSummary.ts`, `SubagentRun.ts`, `SubagentRunUsage.ts`,
    `AttendedMode.ts`, `AttachmentSource.ts`, `UnattendedWriteMode.ts`
    — read in full.
  - `echo-agent-cli/web-frontend/src/components/tools/ToolsPanel.tsx`
    (read for the load-bearing field-access proof in P2-01).
- Whole-frontend greps for `from.*generated`, `from.*types/api`,
  `input_schema`, `parameters`, `need_approval`, `SubagentRun\b`,
  `AttendedMode`, `UnattendedWriteMode`, `AttachmentSource`,
  `SubagentRunUsage`.

## Out Of Scope

Deferred to downstream tasks:

- **A-SRF-02 / A-SRF-03** own the Tauri command-side adapter
  correctness and the *receive-side* reducer behavior. This task
  consumes their channel inventory and audit only the *type contract*.
  Specifically, A-SRF-02-P3-02 (untyped `execution://event` emit) and
  A-SRF-03-P3-03 (untyped receive cast) are the upstream/downstream
  pair of the present task's findings on the same channel; this task
  describes the *field-level* drift, those describe the typing-level
  drift.
- **A-FE-02** owns the projection identity and reducer monotonicity
  in stores. This task stops at the static type contract; how a store
  handles late / duplicate / out-of-order events of the correct shape
  is A-FE-02 territory.
- **A-CHAT-01** owns the chat-turn lifecycle. The `ChatEvent::RunStatus`
  emission order is consumed here only as a typed-variant inventory
  check, not a lifecycle check.
- **A-TSK-04** owns the persistence-side correctness of
  `TaskRuntimeStore`. The generated TS types for the task-runtime
  family are spot-checked here for variant coverage; their state-
  machine semantics belong to A-TSK-04.

## Inputs

Required repository documents read in full:

- Repository root `AGENTS.md` (multi-mode parity rule; framework-vs-
  application layering gate; the implementation gate "first prove no
  duplicate exists"; UTF-8 / panic safety; the cleanup rule).
- `docs/comprehensive-review/REPORTING.md`,
  `templates/task-report.md`, `templates/validation-report.md`.
- `docs/comprehensive-review/TASKS.md` (A-FE-01 card + dependency
  list).

Dependency reports read:

- **A-SRF-02** (complete) — establishes the emit-side contract:
  four channels (`chat://event`, `execution://event`,
  `browser://event`, `terminal-output`/`terminal-exit`);
  `chat://event` typed via `ChatEvent` enum (20 variants — actual
  count is 19 in current code; A-SRF-02's "20 variants" figure was
  approximate); `execution://event` is hand-built `serde_json::Map`
  (A-SRF-02-P3-02). Load-bearing for V01/V02: this task verifies the
  field-level / variant-level alignment that A-SRF-02's emit-side
  typing classification implies.
- **A-TSK-04** (complete) — establishes that the
  task-runtime-family Rust types (`TaskRun`, `TaskPlan`, `TodoItem`,
  `RuntimeTaskEvent`, `RuntimeEventKind`, `TaskRunStatus`,
  `TodoStatus`) are the single authority for task-runtime state and
  are the same types ts-rs exports to `generated/`. Load-bearing for
  V01: the task-runtime family is the one place where the frontend
  consumes `generated/` directly (`endpoints.ts:49-59`), so its
  contract is the gold standard against which the manual-only
  contracts are compared.
- **A-SRF-03** (complete) — established the receive-side cast
  (`as unknown as ExecutionEvent`) and the
  `useToolExecutionStore.ingest` direct-overwrite finding. Quoted in
  P3-04 (no contract test exists to catch the next drift).

Historical documents treated as hypotheses:

- `web-frontend/src/types/api.ts:1-36` — the migration note claiming
  "Phase 6.4: The canonical types are auto-generated by ts-rs in
  `src/generated/`. This file contains hand-written types that either
  (a) extend generated types with UI-only fields, (b) are not yet
  generated, or (c) are legacy types pending migration." Treated as
  the **migration-in-progress claim** this task falsifies for
  `ToolInfo` / `SkillInfo` / `McpServerInfo` (manual versions do NOT
  extend a generated counterpart — they shadow it with different
  fields).

## Layering Decision

This is an **application-layer** task. All inspected code lives in
`echo-agent-cli/{echo-agent-app-core/src, src/tauri, web-frontend/src}`.
The framework supplies only the `ToolFailure` family and `Message`
structs, which the application serializes verbatim.

| Classification | Required answer |
|---|---|
| Generic mechanism | ts-rs codegen + serde `rename_all = "snake_case"` + Tauri v2 IPC (which auto-converts snake_case Rust arg names to camelCase JS arg names — verified non-issue: every frontend `apiInvoke('cmd', { runId })` correctly maps to `run_id: String` in Rust). |
| EKO product policy | All DTOs are EKO product shapes. The question is contract consistency, not layer placement. |
| Adapter boundary | DTOs returned by Tauri commands should be one of: (a) a typed Rust struct serialized verbatim, (b) hand-built JSON whose shape is documented in a TS type. The audit shows three different patterns coexist: (1) typed-struct verbatim (`tool_executions.rs`, `files.rs`); (2) hand-built JSON matching the manual TS (`panels.rs` skills, `mcp.rs`, `conversations.rs`); (3) hand-built JSON matching neither the manual nor the generated TS (none — the wire always matches *some* TS, just sometimes the wrong one). |
| Duplicate search | Searched the frontend tree for every nominal DTO type that exists in BOTH `types/api.ts` and `generated/` — found 8 shadowed names (`ToolInfo`, `SkillInfo`, `McpServerInfo`, `McpToolInfo`, `ConversationRecord`, `SavedMessage`, `ConnectMcpRequest`, `FullConfigResponse`). The first four have substantive field drift; the last four agree. Searched `generated/*.ts` against `generated/index.ts` and found 5 orphan generated files. Searched for any test guarding the manual↔wire contract — found none. |
| Migration deletion | No deletion proposed. The findings identify type drift, orphan files, and missing tests; resolution is left to follow-up task IDs. |

## Current Path

### Type contract inventory (V01 + V04)

Three distinct sources of truth coexist on the IPC boundary:

```text
1. ts-rs generated (generated/*.ts) — derived from Rust structs that
   carry #[derive(...TS...)] + #[ts(export, rename = "...")]. Regenerated
   by `cargo test` (ts-rs auto-enables its __ts_rs feature). 79 type
   files + 1 barrel.

2. Manual hand-written (types/api.ts, endpoints.ts) — written by hand.
   Some re-export generated types (types/api.ts:11-17 re-exports
   ChatRequest/ChatResponse/ToolCallInfo/ContextStats/SessionInfo);
   some shadow generated names with different fields (ToolInfo,
   SkillInfo, McpServerInfo, McpToolInfo); some are unique to manual
   (ChatEvent, ToolExecution*, EvidenceCandidate, ConversationRecord
   wire shape).

3. Hand-built wire JSON (Tauri commands) — some commands return
   typed Rust structs verbatim via serde_json::to_value; others
   build serde_json::json!({...}) objects by hand.
```

The frontend consumes types from `types/api.ts` and `endpoints.ts` for
everything except the task-runtime family, which it consumes directly
from `generated/` (`endpoints.ts:49-59`). The contract is therefore
*real* (compiler-checked) only for the task-runtime family; everywhere
else it depends on manual sync.

Verified wire-vs-TS alignment (V01):

| DTO | Source | Wire pattern | TS contract | Align? |
|---|---|---|---|---|
| `ChatEvent` | chat.rs:30-112 | typed Rust enum verbatim | manual `types/api.ts:125-177` | yes (19 variants) |
| `ToolExecution*` family | tool_execution.rs:54-110 | typed Rust verbatim | manual `types/api.ts:49-117` | yes |
| `ToolFailure` | echo-core/tools/mod.rs:78-88 | typed Rust verbatim | manual `types/api.ts:71-78` | yes |
| `TaskRun` / `TaskPlan` / `TodoItem` / `RuntimeTaskEvent` / `RuntimeEventKind` / `TaskRunStatus` / `TodoStatus` | task_runtime/types.rs | typed Rust verbatim | `generated/*.ts` (consumed via `endpoints.ts:49-59`) | yes |
| `EvidenceCandidate` + family | evidence.rs | typed Rust verbatim | manual `types/api.ts:692-757` | yes |
| `ProviderTemplate` / `ConfiguredModelView` | model_config.rs | typed Rust verbatim | manual `types/api.ts:761-783` | yes |
| `SavedMessage` | persistence.rs:34-60 | typed Rust verbatim | manual `types/api.ts:428-449` | yes |
| `FileEntry` / `FileContent` / `DiffLine` / `DiffHunk` | files.rs | typed Rust verbatim | manual `endpoints.ts:745-797` | partial (null-vs-undefined; V03) |
| `ConversationRecord` (wire) | conversations.rs:533-540 | **hand-built JSON** | manual `types/api.ts:451-458` | yes (manual matches wire; the Rust `ConversationRecord` struct in persistence.rs is unused on this path) |
| `SkillInfo` (wire) | panels.rs:357-386 | **hand-built JSON** | manual `types/api.ts:187-206` | yes (manual matches wire; `response.rs:91-99` SkillInfo struct is unused) |
| `McpServerInfo` (wire) | mcp.rs:60-86 | **hand-built JSON** | manual `types/api.ts:241-250` | yes (manual matches wire; `response.rs:57-69` McpServerInfo struct is unused) |
| **`ToolInfo` (wire)** | tools.rs:14 (returns `Vec<ToolInfo>`) | **typed Rust verbatim** | manual `types/api.ts:179-185` | **NO — Rust sends `parameters/need_approval/source: ToolSource`; manual TS declares `input_schema?: ...` + no `need_approval` + `source: string`** |

The last row is the load-bearing defect: the manual `ToolInfo` does
not match the wire, and the only consumer (`ToolsPanel.tsx`) reads the
wrong field name. See P2-01.

### Variant coverage (V02)

Audited 13 enums. 11 align exactly; 2 drift:

- `McpConnectionStatus` (`response.rs:71-79`) — ts-rs-exported Rust
  enum has 3 variants (`Connected`, `Disconnected`, `Error(String)`).
  Wire emitted by `list_mcp_servers` (`mcp.rs:60-86`) uses 4 strings:
  `connected`, `error`, `disconnected`, `disabled`. Manual TS
  (`types/api.ts:243`) lists all 4 wire strings; generated TS
  misrepresents the wire. Folded into P3-01.
- `SubagentRunEventKind` (manual TS in `subagentRunStore.ts:32-40`)
  — includes `'artifact'`, but no backend emit site produces
  `event: "artifact"` for the `execution://event` channel
  (`grep -rn "DispatchArtifact\|\"artifact\"" src/tauri` returns no
  bridge emission). Dead variant on the TS side. P3-05.

### Option/null/undefined (V03)

Spot-checked every `Option<T>` field in IPC DTOs. All high-traffic
DTOs (`ChatEvent::SelectionRequest`, `ToolExecutionSummary`,
`EvidenceCandidate`, `ProviderTemplate`, `ConfiguredModelView`,
`RuntimeTaskEvent`) use the consistent pattern. Three cosmetic drifts
in the file-panel DTOs (`FileEntry.modified/extension`,
`DiffLine.old_line/new_line`) where the wire sends `null` but TS
declares `undefined`. P3-03.

### Serialization tests (V04)

ts-rs is wired (`echo-agent-app-core/Cargo.toml:23`) and produces 80
files. But:

1. No npm/CI script regenerates them on a discoverable hook — the
   `web-frontend/package.json` scripts are `dev/test/build/preview`
   only; `README.md` does not document regeneration.
2. 5 of 80 generated files are orphaned — not re-exported from
   `generated/index.ts`, not imported anywhere:
   `AttachmentSource.ts`, `AttendedMode.ts`, `SubagentRun.ts`,
   `SubagentRunUsage.ts`, `UnattendedWriteMode.ts`. P3-02.
3. No fixture / snapshot / round-trip test guards the manual-only
   contracts (`ChatEvent`, `ToolExecution*`, `SkillInfo` wire,
   `McpServerInfo` wire, `ConversationRecord` wire). The existing
   `*.test.ts` files feed hand-written JS objects that match the
   manual TS, not the wire — they would not catch a wire-side
   regression. P3-04.
4. Three nominal types are shadowed with field-level disagreement:
   `ToolInfo` (P2-01), `SkillInfo` + `McpServerInfo` + `McpToolInfo`
   (P3-01).

## Findings

### A-FE-01-P2-01: Manual `ToolInfo` type and `ToolsPanel` field access drift from the wire — tool parameters are never rendered

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/types/response.rs:39-47`
    — Rust DTO returned by `list_tools` / `get_tool` / `enable_tool` /
    `disable_tool`:
    ```rust
    pub struct ToolInfo {
        pub name: String,
        pub description: String,
        pub parameters: Value,           // ← field name on the wire
        pub enabled: bool,
        pub need_approval: bool,         // ← not in manual TS
        pub source: ToolSource,          // ← enum (Builtin), not string
    }
    ```
    No `#[serde(rename)]` on `parameters`; the wire key is
    `"parameters"`.
  - `echo-agent-cli/src/tauri/commands/tools.rs:9-15, 23-30` —
    `list_tools` returns `serde_json::to_value(infos)` where
    `infos: Vec<crate::types::ToolInfo>`. Confirmed via
    `state.rs:722-748` that the construction populates `parameters`,
    `need_approval`, and `source: ToolSource::Builtin` — no rename.
  - `echo-agent-cli/web-frontend/src/types/api.ts:179-185` — manual
    TS:
    ```ts
    export interface ToolInfo {
      name: string;
      description: string;
      source: string;
      input_schema?: Record<string, unknown>;   // ← WRONG name
      enabled: boolean;
      // need_approval absent
    }
    ```
  - `echo-agent-cli/web-frontend/src/components/tools/ToolsPanel.tsx:89,94`
    — the only consumer of the schema field:
    ```tsx
    {tool.input_schema && (
      <pre ...>
        {JSON.stringify(tool.input_schema, null, 2)}
      </pre>
    )}
    ```
    `tool.input_schema` is `undefined` on every payload (the wire
    field is `tool.parameters`), so the conditional is always false
    and the `<pre>` block never renders.
  - `echo-agent-cli/web-frontend/src/generated/ToolInfo.ts` — the
    ts-rs-generated version (UNUSED by consumers;
    `grep -rn "from.*generated.*ToolInfo" web-frontend/src` returns
    zero hits) correctly has `parameters: JsonValue`,
    `need_approval: boolean`, `source: ToolSource`.
- Reachability: every GUI tool-panel expansion.
  `endpoints.ts:141` `toolsApi.list()` → `apiInvoke<ToolInfo[]>('list_tools')`
  → `ToolsPanel.tsx:84-98` expansion block. The user clicks the
  expand arrow, expects to see the JSON parameter schema, and gets
  nothing. The tool name, description, and enable/disable toggle all
  work (they read `tool.name`, `tool.description`, `tool.enabled`,
  which are correctly named on both sides). Only the parameter schema
  is affected.
- Expected invariant: the TS field name should equal the wire field
  name. The shadowing of `ToolInfo` (same name in `types/api.ts` and
  `generated/ToolInfo.ts`) should not produce two different shapes.
- Observed behavior: the manual `ToolInfo` declares `input_schema`
  (the field name that belongs to `McpToolInfo`, not `ToolInfo`) and
  omits `parameters` / `need_approval`. The component reads
  `tool.input_schema`, which is always undefined because the Rust DTO
  sends `parameters`. The bug is silent: there is no error, no
  warning, no test failure. The user simply never sees the parameter
  schema.
- Impact: a documented feature (viewing tool parameter schemas in the
  GUI) is broken silently. The tool panel renders correctly otherwise.
  Severity is medium because the surrounding functionality (enable /
  disable, name, description) is unaffected and the user can still
  invoke the tool, but the schema-viewing feature is dead.
- Root cause: the manual `ToolInfo` was written by analogy with
  `McpToolInfo` (which DOES have `input_schema`) instead of by reading
  the Rust DTO. The shadowed name made the mistake invisible: a
  developer checking "is ToolInfo defined?" finds the manual version
  and never notices the generated version disagrees. No contract test
  guards the wire shape (V04).
- Direction: pick one of:
  1. **Switch consumers to the generated `ToolInfo`** (preferred).
     `ToolsPanel.tsx` imports `ToolInfo` from `../generated` instead
     of `../types/api`; the field access becomes `tool.parameters` and
     `tool.need_approval`. Delete the manual `ToolInfo` from
     `types/api.ts:179-185`. Add a small UI element for
     `need_approval` (which is currently invisible).
  2. **Fix the manual `ToolInfo` in place** if the
     `need_approval: bool` / `source: ToolSource` additions are not
     desired in the UI yet: rename `input_schema` → `parameters`,
     mark `parameters: Record<string, unknown>` (required, not
     optional — Rust sends it always), align `source` with the
     generated `ToolSource` enum.
  Either way, add a contract test (P3-04) so this can't regress.
- Regression validation: a vitest unit test that feeds a
  `ToolInfo`-shaped fixture `{name, description, parameters: {...},
  enabled, need_approval, source: {Builtin: ...}}` through
  `ToolsPanel` and asserts the parameter schema is rendered. Pair
  with a Rust-side snapshot test asserting
  `serde_json::to_value(ToolInfo { ... })` produces the
  `"parameters"` key.
- Validation reports: [V01-01](../validations/A-FE-01/V01-01.md),
  [V04-01](../validations/A-FE-01/V04-01.md).

### A-FE-01-P3-01: `SkillInfo`, `McpServerInfo`, `McpToolInfo`, `ConversationRecord` shadow their generated counterparts with different fields — generated Rust DTOs are vestigial HTTP-server types, manual TS matches the hand-built Tauri wire

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/types/response.rs:57-99`
    — Rust structs `McpServerInfo`, `McpConnectionStatus`,
    `McpToolInfo`, `SkillInfo`, `SkillSource`. All carry
    `#[derive(...TS...)]` + `#[ts(export, rename = "...")]`, so each
    produces a generated TS file.
  - `echo-agent-cli/src/tauri/commands/panels.rs:357-386` —
    `list_skills` returns hand-built JSON via `hub_skill_json` /
    `skill_descriptor_json`, NOT `serde_json::to_value(Vec<SkillInfo>)`.
    The wire fields are: `name, description, file, loaded, source,
    category, is_baseline, is_builtin, upstream_version, license,
    version, author, tags, has_sandbox, depends_on,
    missing_dependencies, has_updates, triggers`. The Rust struct's
    fields are: `name, description, enabled, tool_names, source`.
    The two sets are nearly disjoint.
  - `echo-agent-cli/src/tauri/commands/mcp.rs:14-93` —
    `list_mcp_servers` returns hand-built JSON. The wire `status` is
    one of `"connected" / "error" / "disconnected" / "disabled"`; the
    wire `tools` is an array of `{name, description}` (no
    `input_schema`). The Rust `McpServerInfo` struct's `status:
    McpConnectionStatus` enum has only `Connected / Disconnected /
    Error(String)` (no `Disabled`); its `tools: Vec<McpToolInfo>`
    would carry `input_schema`.
  - `echo-agent-cli/src/tauri/commands/conversations.rs:533-540` —
    `get_conversation` returns hand-built JSON with `id,
    conversation_id, title, messages, created_at, updated_at`. The
    Rust `persistence.rs:167-174` `ConversationRecord` struct has
    `id, title, messages, model, created_at, updated_at` (no
    `conversation_id`; has `model`). The struct is unused on the
    Tauri path; the wire is built by hand and matches the manual TS.
  - `echo-agent-cli/web-frontend/src/types/api.ts:187-206, 241-256,
    451-458` — manual `SkillInfo`, `McpServerInfo`, `McpToolInfo`,
    `ConversationRecord` correctly match the hand-built wire (this is
    why the GUI works).
  - `echo-agent-cli/web-frontend/src/generated/SkillInfo.ts,
    McpServerInfo.ts, McpToolInfo.ts` — generated versions matching
    the unused Rust structs. None are imported anywhere in
    `web-frontend/src` (verified by grep).
- Reachability: a future contributor who writes
  `import { SkillInfo } from '../generated'` (reasonable — it IS the
  generated module, and IDE autocomplete will offer it) gets a type
  that does NOT match the runtime payload. Any code they write against
  the generated `SkillInfo.enabled` / `SkillInfo.tool_names` will be
  undefined at runtime.
- Expected invariant: a nominal type's manual and generated
  definitions should agree, OR one of them should not exist. Same-name
  shadowing with different fields is a footgun.
- Observed behavior: the Rust `SkillInfo` / `McpServerInfo` /
  `McpToolInfo` / `ConversationRecord` structs in `response.rs` /
  `persistence.rs` are vestigial — they describe a server (Axum)
  response shape that the Tauri command path doesn't use. The
  `#[ts(export, ...)]` annotation on each keeps regenerating the
  unused TS file every `cargo test` run, creating permanent shadow
  drift.
- Impact: (a) shadow footgun for future contributors (above); (b) the
  generated barrel `index.ts` re-exports these names, so a careless
  `import { SkillInfo } from '../generated'` is one keystroke away;
  (c) the vestigial Rust structs accumulate `#[ts(export)]` cost
  (minor) and conceptual clutter (larger).
- Root cause: the GUI was migrated from an Axum HTTP server to Tauri
  IPC. The old server-typed DTOs (`response.rs` / `persistence.rs`)
  were kept and gained `#[derive(TS)]` for the new ts-rs pipeline,
  but the Tauri command path was written to emit hand-built JSON
  instead of serializing those DTOs. The two paths diverged; nobody
  deleted the vestigial structs.
- Direction: pick one of:
  1. **Make the wire and the Rust struct agree** — have
     `list_skills` / `list_mcp_servers` / `get_conversation` return
     `serde_json::to_value(Vec<SkillHubEntry>)` (or whatever the real
     shape is) and add `#[derive(TS)]` to that struct. Then delete
     the manual TS in `types/api.ts` and import from `generated/`.
  2. **Stop exporting the vestigial structs** — remove
     `#[derive(TS)]` + `#[ts(export, ...)]` from the unused Rust
     `SkillInfo` / `McpServerInfo` / `McpToolInfo` /
     `ConversationRecord`. The orphan generated files disappear on
     the next `cargo test`. The manual TS remains the single
     contract.
  Option 2 is the smaller change and matches the AGENTS.md cleanup
  rule ("delete over retain" for vestigial code). Option 1 is the
  better long-term contract.
- Regression validation: after the change, run
  `cd echo-agent-cli && cargo test -p echo-agent-app-core --locked`
  (regenerates ts-rs), then
  `cd echo-agent-cli/web-frontend && npx vitest run` (existing store
  tests must still pass), then
  `npm run build` (TypeScript must compile). Add a manual smoke test
  of the skills / mcp panels.
- Validation reports: [V01-01](../validations/A-FE-01/V01-01.md),
  [V02-01](../validations/A-FE-01/V02-01.md),
  [V04-01](../validations/A-FE-01/V04-01.md).

### A-FE-01-P3-02: Five generated types are orphaned — not re-exported from `index.ts`, not imported anywhere

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/web-frontend/src/generated/index.ts` — barrel
    re-exports 75 generated modules.
  - `echo-agent-cli/web-frontend/src/generated/` — 80 `.ts` files (1
    `index.ts` + 79 type files).
  - Cross-reference (every `*.ts` in `generated/` checked against
    `index.ts`): 5 files are not re-exported and not imported
    anywhere else in `web-frontend/src`:
    - `AttachmentSource.ts`
    - `AttendedMode.ts`
    - `SubagentRun.ts`  ← structurally important
    - `SubagentRunUsage.ts`
    - `UnattendedWriteMode.ts`
  - `grep -rn "from.*generated.*SubagentRun\b\|...AttendedMode\|...UnattendedWriteMode\|...AttachmentSource\|...SubagentRunUsage" web-frontend/src`
    returns zero hits (verified).
- Reachability: low — these are types, not runtime code. The cost is
  clutter and a missing contract: `SubagentRun` especially is the
  durable subagent record (per A-TSK-04 — `SubagentRun` is the
  framework-side instance, identity `subagent_run_id =
  {run_id}:{task_id}:{plan_revision}:{attempt}`). The frontend's
  `subagentRunStore.SubagentRunState` (`subagentRunStore.ts:98-136`)
  is hand-written and only loosely tracks the Rust `SubagentRun`
  fields; the orphaned generated `SubagentRun.ts` could be the
  authoritative contract if it were wired up.
- Expected invariant: every generated file should be reachable from
  the barrel (`index.ts`), OR explicitly deleted. Orphans are dead
  weight and indicate that the codegen pipeline produced something
  nobody consumes.
- Observed behavior: 5 generated files exist, are regenerated on
  every `cargo test`, but are not consumed. The `SubagentRun.ts`
  case is particularly misleading because the Rust `SubagentRun`
  struct DOES exist and is used at runtime (A-TSK-04 V01), so the
  orphan is a "should-be-contracted-but-isn't" gap.
- Impact: clutter; missing contract for `SubagentRun`. Minor unless
  the team wants to tighten the IPC type discipline.
- Root cause: when these Rust types gained `#[ts(export)]`, the
  `index.ts` barrel was not updated (the barrel is hand-maintained).
  There is no CI check that "every generated file is re-exported."
- Direction: add the 5 missing re-exports to `generated/index.ts`.
  For `SubagentRun` specifically, consider switching
  `subagentRunStore.SubagentRunState` to compose the generated
  `SubagentRun` so the durable-record fields become type-checked.
  Optionally add a CI check (`for f in generated/*.ts; do grep -q
  "$(basename $f .ts)" generated/index.ts || exit 1; done`) so this
  doesn't recur.
- Regression validation: `cd echo-agent-cli/web-frontend && npm run
  build` succeeds after the barrel update; new test that imports
  `SubagentRun` from `generated/` and asserts the field set matches
  the documented identity contract.
- Validation reports: [V04-01](../validations/A-FE-01/V04-01.md).

### A-FE-01-P3-03: `FileEntry` / `DiffLine` optional fields declare `undefined`-only but the wire sends `null`

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/tauri/commands/files.rs:12-20` —
    ```rust
    pub struct FileEntry {
        pub name: String,
        pub path: String,
        pub is_dir: bool,
        pub size: u64,
        pub modified: Option<String>,    // no skip_serializing_if
        pub extension: Option<String>,   // no skip_serializing_if
    }
    ```
    No `#[serde(skip_serializing_if = "Option::is_none")]`, so the
    wire always sends `{modified: null, extension: null}` when the
    values are `None`.
  - `echo-agent-cli/src/tauri/commands/files.rs:57-63` — `DiffLine`:
    `old_line: Option<usize>`, `new_line: Option<usize>`, no skip.
    Wire sends `null`.
  - `echo-agent-cli/web-frontend/src/api/endpoints.ts:745-752` —
    ```ts
    export interface FileEntry {
      ...
      modified?: string;       // ← forbids null
      extension?: string;      // ← forbids null
    }
    ```
  - `echo-agent-cli/web-frontend/src/api/endpoints.ts:777-782` —
    `DiffLine`: `old_line?: number; new_line?: number;` (forbid null).
  - **Inconsistency**: the adjacent `FileContent.mime_type` and
    `data_url` (`files.rs:23-32` ↔ `endpoints.ts:761-770`) ARE typed
    as `string | null`. The file-panel DTOs are internally
    inconsistent — some Option fields include `| null`, others don't.
- Reachability: every file-panel render that displays
  `entry.modified` / `entry.extension` / `diffLine.old_line` /
    `diffLine.new_line`. Current consumer code uses truthiness
    checks (e.g. `if (entry.modified)`), which work for both `null`
    and `undefined`. Strict-null-checked consumer code that does
    `entry.modified === undefined` returns false (wire value is
    `null`); consumers that pass the value to a `string | undefined`
    parameter violate the type at runtime (the value is `null`).
- Expected invariant: the TS type should reflect the wire. If the
  wire always sends `null`, the TS type should be `T | null`.
- Observed behavior: the wire sends `null`; the TS declares
  `undefined`. The two are indistinguishable to truthiness checks
  but distinguishable to identity checks.
- Impact: cosmetic. No current runtime defect; future strict
  consumer code could be misled.
- Root cause: the manual TS was written with the convention "absent
  ↔ ?:" without checking whether the Rust side uses
  `skip_serializing_if`. Inconsistent application within the same
  file.
- Direction: align the TS to the wire. Either (a) add `| null` to
  `FileEntry.modified/extension` and `DiffLine.old_line/new_line`,
  matching the `FileContent.mime_type/data_url` convention; or (b)
  add `#[serde(skip_serializing_if = "Option::is_none")]` on the
  Rust side so the fields are absent (matching `?:`).
  Option (a) is one-line-per-field on the TS side; do that.
- Regression validation: a snapshot test asserting
  `serde_json::to_value(FileEntry { modified: None, ... })` produces
  the expected JSON shape, paired with a TS-side fixture that
 consumes
  the same JSON and asserts the field's value is `null` (not
  undefined).
- Validation reports: [V03-01](../validations/A-FE-01/V03-01.md).

### A-FE-01-P3-04: No contract test guards the manual DTO shapes against wire drift

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `find echo-agent-cli -name "fixtures" -o -name "__fixtures__"` —
    no fixtures directory.
  - `grep -rln "snapshot\|wire.*shape\|to_value.*ToolInfo" \
      echo-agent-cli/echo-agent-app-core/src
      echo-agent-cli/web-frontend/src --include="*.rs" --include="*.ts"`
    — no IPC-contract snapshot tests. The Rust-side tests
    (`tool_execution.rs:833-979`) cover repository internals
    (UTF-8 safety, journal repair, pagination) but never assert
    `serde_json::to_value(summary)` field names.
  - The TS-side tests (`stores/*.test.ts`,
    `components/chat/*.test.tsx`) feed hand-written JS objects that
    match the manual TS. They would not detect a wire-side rename
    (e.g. `parameters` → `input_schema`, which is exactly the
    A-FE-01-P2-01 defect).
  - `web-frontend/package.json` has only `dev/test/build/preview`
    scripts — no `generate-types`, no `check-types-against-wire`.
- Reachability: every manual DTO (`ChatEvent`, `ToolExecution*`,
  `SkillInfo` wire, `McpServerInfo` wire, `ConversationRecord` wire,
  `EvidenceCandidate`, `ProviderTemplate`, `ConfiguredModel`).
- Expected invariant: a contract test should fail when the wire
  shape and the TS type disagree, OR the TS type should be generated
  from the Rust model (so the contract is structural).
- Observed behavior: no such test exists. The P2-01 drift
  (`parameters` vs `input_schema`) went undetected at compile time
  on both sides and at test time on either side.
- Impact: the next wire-side rename will regress silently the same
  way P2-01 did. The task-runtime family is safe (consumes
  `generated/` directly); everything else is exposed.
- Root cause: the ts-rs pipeline was set up for the task-runtime
  family (where it works — A-TSK-04 V01 confirms the generated types
  are the single authority); the rest of the IPC surface was never
  brought under the same discipline, and no parallel fixture-based
  test was added.
- Direction: pick one of:
  1. **Expand ts-rs coverage** to every Rust DTO that crosses IPC
     (`ToolExecutionSummary`, `ToolExecutionDetailManifest`,
     `ToolExecutionDetailPage`, `FileEntry`, `FileContent`,
     `DiffLine`, `DiffHunk`, `EvidenceCandidate`, `ProviderTemplate`,
     `ConfiguredModelView`, `SavedMessage`). Delete the manual
     counterparts in `types/api.ts`; import from `generated/`.
     For `ChatEvent`, add `#[derive(TS)]` to the Rust enum (the
     `#[serde(tag = "type")]` convention ts-rs handles natively).
  2. **Add fixture tests** if expanding ts-rs is too large a change.
     For each manual DTO, add a Rust test that serializes a sample
     instance to JSON and asserts the field set; commit the same
     JSON as a TS fixture and consume it in a vitest test that
     casts to the manual type. The pair becomes a locked contract.
  Option 1 is the structural fix; option 2 is the incremental fix.
- Regression validation: this finding IS the regression validation
  for P2-01 / P3-01 — the existence of a contract test would have
  caught both.
- Validation reports: [V04-01](../validations/A-FE-01/V04-01.md).

### A-FE-01-P3-05: `SubagentRunEventKind` includes an `'artifact'` variant that no backend emit site produces

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/web-frontend/src/stores/subagentRunStore.ts:32-40`
    — the manual TS union:
    ```ts
    export type SubagentRunEventKind =
      | 'started' | 'usage' | 'isolation_observed'
      | 'artifact'                                   // ← dead
      | 'completed' | 'failed' | 'timed_out' | 'cancelled';
    ```
  - `echo-agent-cli/web-frontend/src/stores/subagentRunStore.ts:146-155`
    — `STORED_SUBAGENT_EVENTS` includes `'artifact'`, so the reducer
    would accept such an event if it ever arrived.
  - `echo-agent-cli/src/tauri/mod.rs:383-702` — the bridge maps
    every `SubagentEvent` variant to an `event_type` string. The
    emitted `event_type`s are: `"started"` (DispatchStarted),
    `"isolation_observed"` (DispatchIsolationObserved),
    `"completed"` (DispatchCompleted), `"failed"` or `"timed_out"`
    (DispatchFailed — `status.as_str()`), `"cancelled"`
    (DispatchCancelled), `"usage"` (DispatchLlmUsage). DispatchTool*
    events are consumed by the tool-execution repository and emit on
    the `kind="tool"` branch instead. No `SubagentEvent::DispatchArtifact*`
    exists and no `event_type: "artifact"` is ever produced.
  - `grep -rn "\"artifact\"\|DispatchArtifact\|emit_execution_event.*artifact" src/tauri`
    returns zero hits in the bridge emission path.
- Reachability: dead. The TS branch is unreachable on the current
  backend.
- Expected invariant: every variant in the receive-side union should
  correspond to at least one emit site, OR be documented as
  reserved-for-future-use.
- Observed behavior: the `'artifact'` variant is listed but never
  produced. It is also in `STORED_SUBAGENT_EVENTS`, so if a future
  backend change emits it, the reducer would land in an untested
  code path.
- Impact: clutter; trap for the next contributor who assumes the
  variant is reachable.
- Root cause: the `SubagentRunEventKind` union was likely authored
  speculatively against a planned-but-unimplemented artifact-streaming
  feature; the backend never caught up.
- Direction: either (a) remove `'artifact'` from
  `SubagentRunEventKind` and `STORED_SUBAGENT_EVENTS` until the
  backend emits it, OR (b) document it as reserved-for-future-use
  with a comment pointing to the planned feature. Option (a) is
  smaller.
- Regression validation: a TS-side test asserting that the union
  equals exactly the set of variants the backend emits (the
  contract test from P3-04 would catch this automatically).
- Validation reports: [V02-01](../validations/A-FE-01/V02-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | DTO field matrix: Rust DTO ↔ TS type field-by-field | yes | passed (with finding) | [V01-01](../validations/A-FE-01/V01-01.md) |
| V02 | Enum/event variant coverage | yes | passed (with finding) | [V02-01](../validations/A-FE-01/V02-01.md) |
| V03 | Optional/null/undefined semantics | yes | passed (with finding) | [V03-01](../validations/A-FE-01/V03-01.md) |
| V04 | Generated/fixture serialization tests, ts-rs orphan inventory | yes | passed (with finding) | [V04-01](../validations/A-FE-01/V04-01.md) |
| V05 | Historical-document drift | conditional (applicable — the `types/api.ts:1-36` migration-in-progress claim is treated as a hypothesis) | passed | classified inline in Historical Claim Status |

No `cargo` or `vitest` command was required: this is a static-inspection
review of the type contract. A-SRF-03 already executed the full vitest
matrix (`npx vitest run` → 26 files, 101 tests, exit 0); A-TSK-04
executed the relevant cargo test subsets. No new executable claim is
made here.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `types/api.ts:1-36` — "Phase 6.4: The canonical types are auto-generated by ts-rs in `src/generated/`. This file contains hand-written types that either (a) extend generated types with UI-only fields, (b) are not yet generated, or (c) are legacy types pending migration." | partially overstated | The migration-in-progress framing is correct for the task-runtime family (consumed from `generated/`) but understates the shadow-drift problem: `ToolInfo` (P2-01), `SkillInfo` / `McpServerInfo` / `McpToolInfo` / `ConversationRecord` (P3-01) have manual versions that DO NOT extend a generated counterpart — they shadow it with different fields, and in the `ToolInfo` case the manual version is just wrong against the wire. The "extend generated types with UI-only fields" pattern is not what's happening for these four. |
| `types/api.ts:33-36` — "The following types have identical generated counterparts and are imported from ../generated instead of being duplicated here: ChatRequest, ChatResponse, ToolCallInfo, ContextStats, SessionInfo." | current | Verified: those 5 names are re-exported from `generated/` via `types/api.ts:11-17` and are not duplicated in `types/api.ts`. |
| `types/api.ts:27-31` — "Known gaps (types NOT yet generated): ChatRunStatus (enum with frontend-only states like 'connecting'), ApprovalRequest / InputRequest / SelectionRequest (extend generated), ExecutionRound (frontend-specific aggregation)." | partially stale | `ChatRunStatus` is correct as a frontend-only enum (Rust emits plain `String` for `ChatEvent::RunStatus`), but the union does NOT include `'connecting'` (the comment says it does — `types/api.ts:38-47` lists `'idle' / 'running' / 'thinking' / 'using_tool' / 'waiting_approval' / 'waiting_input' / 'completed' / 'failed' / 'cancelled'`, no `'connecting'`). The ApprovalRequest / InputRequest / SelectionRequest TS shapes ARE NOT generated — they are inlined into the `ChatEvent` discriminated union (`types/api.ts:156-172`); the comment "extend generated" is misleading (there is no generated ApprovalRequest to extend). |
| A-SRF-02-P3-02 (untyped `execution://event` emit) | current (load-bearing) | This task's V01/V02 confirm the field-level inventory of the untyped channel: the TS-side `ExecutionEvent` interface in `subagentRunStore.ts:52-96` matches the hand-built emit in `mod.rs:703-752` field-for-field (with one dead variant, P3-05). The typing-level gap (no compiler-checked schema) is A-SRF-02-P3-02; the field-level match is documented here. |
| A-SRF-03-P3-03 (untyped `execution://event` receive cast) | current (load-bearing) | This task confirms the cast target (`ExecutionEvent` interface) is the correct shape; the cast hazard (no runtime validation) is A-SRF-03-P3-03. The two findings are complementary. |
| A-TSK-04 generated task-runtime types as the gold standard | current (load-bearing) | Verified: `endpoints.ts:49-59` consumes `TaskRun / TaskPlan / TaskUpdateRequest / TodoItem / RuntimeTaskEvent / RuntimeArtifact / ReviewResult / TaskExecutionSummary / RecoveryBlocker` directly from `generated/`. The generated versions match the Rust structs (V01). This is the only part of the IPC surface with a real compile-checked contract. |

## Coverage And Uncertainty

Inspected in full: every Rust DTO in `types/response.rs`, every Rust
DTO in `tool_execution.rs`, every Rust DTO in `persistence.rs`,
`model_config.rs`, `evolution/evidence.rs` (head), the
`ChatEvent` enum and emit helpers in `chat.rs`, the hand-built
wire emitters in `panels.rs` (skills), `mcp.rs`, `conversations.rs`,
the `FileEntry` / `FileContent` / `DiffLine` / `DiffHunk` /
`TreeNode` / `BrowseResult` family in `files.rs`, the
`tool_executions.rs` typed return path, the task_runtime command
signatures, the full `execution://event` bridge in `mod.rs:335-770`.
On the TS side: the full `types/api.ts`, the full `endpoints.ts`
type declarations, the `subagentRunStore.ts` contract for
`execution://event`, the `generated/index.ts` barrel, and 14
specific `generated/*.ts` files. Whole-frontend greps for every
shadowed nominal type.

Not inspected (out of scope or deferred):

- The `endpoints.ts` `*Api` objects beyond their declared TS types
  (e.g. the inline response shapes for `sessionApi.getLatest`,
  `autoMemoryApi.*`, `permissionsApi.*`, `analysisApi.*`,
  `researchApi.*`, `pluginsApi.*`, `workspaceApi.*`,
  `hooksApi.*`, `tasksApi.*`). These follow the same
  `isTauri() ? apiInvoke<T>('cmd', args) : http<T>(path)` pattern;
  spot-checked 5 of them (the conversation, tool-execution,
  task-runtime, mcp, and skills APIs — the highest-cardinality ones).
  The remaining ~15 `*Api` objects likely contain additional drift
  but are lower-traffic and use inline anonymous types (which makes
  drift less consequential — there's no named contract to break).
- The framework's `RuntimeEventKind` payload schemas (the
  `RuntimeTaskEvent.payload: serde_json::Value` is opaque on the
  wire). A-TSK-04 owns the payload-shape contract; this task only
  verified the discriminator (`event_type`) coverage.
- The `tool_execution.rs` repository internals beyond the typed DTO
  surface (already covered by A-TSK-04 V03 for the file-shadow
  write path).

Environmental constraints:

- Read-only static review against `echo-agent-cli` commit `b3b2e81`.
  No code was modified. No `cargo test` regeneration of ts-rs was
  performed (would have required write access to the workspace).

Uncertain claims:

- Whether `parameters` vs `input_schema` for `ToolInfo` is reachable
  in a released build. The static evidence (tools.rs:14 returns
  `serde_json::to_value(infos)` with `infos: Vec<ToolInfo>` from
  response.rs:39-47) is conclusive on the wire shape; the
  ToolsPanel.tsx:89,94 read is conclusive on the consumer. The only
  uncertainty is whether any user has noticed the missing schema
  rendering (no bug report was searched).
- Whether the 5 orphan generated files have ever been imported in
  dead code that was since removed. The current grep is conclusive
  for the present commit; historical usage would require `git log`.

## Handoff

Conclusions downstream tasks may rely on:

1. **The task-runtime family is the only IPC surface with a real
   compile-checked contract.** `endpoints.ts:49-59` imports from
   `generated/`; ts-rs keeps it in sync; A-TSK-04 verified the
   state-machine semantics. Downstream tasks auditing task-runtime
   features can trust the types.
2. **`ChatEvent` and `ToolExecution*` are correct manual contracts.**
   Despite being manual (no ts-rs), they match the Rust enum / struct
   field-for-field and variant-for-variant (V01 / V02). The risk is
   future drift, not current incorrectness.
3. **`ToolInfo` is broken in the GUI today (P2-01).** Any task
   touching the tool panel MUST fix the `input_schema` → `parameters`
   drift; otherwise the schema-viewing feature remains dead.
4. **`SkillInfo`, `McpServerInfo`, `McpToolInfo`,
   `ConversationRecord` are correct on the wire but the Rust structs
   in `response.rs` / `persistence.rs` are vestigial.** Downstream
   tasks should not trust those Rust structs as the wire contract —
   the hand-built JSON in the Tauri commands is the actuality. A
   cleanup task should either delete the vestigial structs or make
   the commands serialize them.
5. **No contract test guards the manual DTO shapes.** The next
   wire-side rename will regress silently. Any task that adds a new
   IPC DTO should add either ts-rs generation or a fixture test for
   it; downstream tasks should not assume "the type compiles" implies
   "the type matches the wire."
6. **The `execution://event` channel field set matches field-for-field
   between the bridge emit and the TS `ExecutionEvent` interface**
   (V01 / V02). The typing-level hazard (no runtime validator) is
   owned by A-SRF-02-P3-02 (emit) and A-SRF-03-P3-03 (receive) —
   this task's field-level match is the input those typing fixes
   will preserve.

Reports downstream tasks must read:

- This report (A-FE-01) for the IPC type-contract matrix, the
  vestigial-struct classification, and the orphan-generated
  inventory.
- `tasks/A-SRF-02.md` for the emit-side channel inventory (4
  channels; typed vs untyped).
- `tasks/A-SRF-03.md` for the receive-side reducer policies and the
  cast hazard on the untyped channel.
- `tasks/A-TSK-04.md` for the task-runtime state-machine semantics
  that the generated types encode.

Conditions that make this report stale:

- Replacing the manual `ToolInfo` with the generated version (or
  renaming `input_schema` → `parameters` in the manual version)
  invalidates P2-01's central claim.
- Removing the vestigial `SkillInfo` / `McpServerInfo` /
  `McpToolInfo` / `ConversationRecord` Rust structs OR making the
  Tauri commands serialize them (resolving P3-01) invalidates the
  shadow-drift evidence.
- Re-exporting the 5 orphan generated files from `index.ts`
  (resolving P3-02) invalidates the orphan inventory.
- Adding a contract-test suite (resolving P3-04) invalidates the
  "no regression test would catch this" claim.
- Adding `#[serde(skip_serializing_if = "Option::is_none")]` to the
  `FileEntry` / `DiffLine` Rust fields OR adding `| null` to the TS
  (resolving P3-03) invalidates the null-vs-undefined evidence.
- Removing the `'artifact'` variant from `SubagentRunEventKind`
  (resolving P3-05) invalidates the dead-variant evidence.
- Any change to the `ChatEvent` enum variant set requires re-running
  V02 (currently 19 variants).

Follow-up task IDs (no fixes implemented in this review):

- A **`ToolInfo` field-name fix** task — resolve A-FE-01-P2-01 by
  switching `ToolsPanel.tsx` to the generated `ToolInfo` (or
  renaming the manual field), and rendering `need_approval`. This is
  the only P2 fix; it is small and self-contained.
- A **vestigial-DTO cleanup** task — resolve A-FE-01-P3-01 by either
  deleting `#[derive(TS)]` from the unused `SkillInfo` /
  `McpServerInfo` / `McpToolInfo` / `ConversationRecord` Rust
  structs, OR making the Tauri commands serialize them. Pair with
  the `ToolInfo` fix above so the four shadowed names are resolved
  together.
- An **orphan-generated-barrel** task — resolve A-FE-01-P3-02 by
  adding 5 missing re-exports to `generated/index.ts` and adding a
  CI check that prevents recurrence.
- An **IPC contract-test** task — resolve A-FE-01-P3-04 by either
  expanding ts-rs coverage to all IPC DTOs (preferred) or adding
  paired Rust-serialize / TS-consume fixture tests for every manual
  DTO. This is the structural fix that prevents the next P2-01-class
  drift.
- A **file-panel null-alignment** task — resolve A-FE-01-P3-03 by
  aligning `FileEntry.modified/extension` and `DiffLine.old_line/
  new_line` TS types with the wire (add `| null`). One-line-per-field.
- A **dead-variant removal** task — resolve A-FE-01-P3-05 by removing
  `'artifact'` from `SubagentRunEventKind` + `STORED_SUBAGENT_EVENTS`,
  OR documenting it as reserved.
