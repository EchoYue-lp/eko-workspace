# A-FE-01: Rust/TypeScript API and event type contract

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63 (baseline 9b0e0fa)
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5 (baseline b3b2e81)
> Worktree state: dirty — 79 files modified, all `web-frontend/src/generated/*.ts`; formatting-only, pre-existing (see A-FE-01-P3-02). No other dirty paths in either repository.

## Question

Do Tauri command DTOs, emitted payloads, TypeScript endpoint types, and stores match field-for-field and variant-for-variant?

**Answer: Mostly yes on the emitted-event and command-registration surfaces, with two material type-contract defects on the DTO surface.** The chat/execution/terminal event payloads match their TS types variant-for-variant and field-for-field (ChatEvent 19/19; kind=tool/subagent/run contracts verified; terminal events match), all 184 frontend-invoked Tauri commands exist and are registered with matching arg keys, and the TaskRuntime DTOs (TaskRun/TodoItem/RuntimeTaskEvent/TaskRunStatus) are single-source generated and consistent. However: (P2) the hand-written `ToolInfo` in `types/api.ts` diverges from the live wire type — `parameters`/`need_approval` are hidden and the phantom `input_schema` is consumed by the Tools panel, so the schema display is dead; (P2) the same-name families `SkillInfo`/`McpServerInfo`/`McpToolInfo` have two divergent wire shapes (Tauri commands' hand-rolled JSON vs generated ts-rs types for a dormant HTTP surface) with the frontend using only one; plus three P3 items (dead wrong-shape `ConnectMcpRequest`, phantom `FullConfigResponse.agent.enable_tools`, and a 79-file generated-artifact commit-state inconsistency that fails the prettier gate).

## Scope

Primary source paths inspected:

- `echo-agent-cli/web-frontend/src/types/api.ts` (full, 797 lines — hand-written supplement), `src/generated/*.ts` (all 82 files incl. index.ts), `src/api/endpoints.ts` (2003 lines — Tauri/HTTP dual-path API layer), `src/api/client.ts`, `src/lib/tauri-bridge.ts`, `src/hooks/useTauriChat.ts` (chat://event + execution://event listeners), `src/hooks/chatEventHandler.ts` (full dispatcher), `src/hooks/useBrowserEvents.ts`, `src/components/terminal/Terminal.tsx`, `src/stores/chatStore.ts`, `toolExecutionStore.ts`, `subagentRunStore.ts`, `taskRuntimeStore.ts`, `src/components/tools/ToolsPanel.tsx`, `src/components/mcp/McpPanel.tsx`, `McpManagerPanel.tsx`, `src/components/config/ConfigPanel.tsx`, `src/components/layout/SettingsDialog.tsx`.
- `echo-agent-cli/src/tauri/commands/chat.rs` (ChatEvent enum :30-112, emitters, TauriChatSink, projector :957-1114, emit_tauri_execution_event :1419-1446), `task_runtime.rs` (command signatures), `mcp.rs` (:7-94 list wire, :210-258 connect), `panels.rs` (:346-377 skill JSON helpers, :459-579 skills commands), `tools.rs`, `config.rs` (:156-161), `terminal.rs` (:148-166), `mod.rs` (invoke_handler :69-310, setup closures, SubagentEventBus bridge :353-768).
- `echo-agent-cli/echo-agent-app-core/src/types/request.rs` (full), `types/response.rs` (full), `types/error.rs`, `tool_execution.rs` (ToolExecutionSummary/Owner/Status), `chat_driver.rs`, `surface_contract.rs`, `tasks/task_runtime/types.rs` (TaskRunStatus/RuntimeEventKind), `tasks/task_runtime/executor.rs` (ExecEvent :71-101, run_started payload :352-360), `state.rs` (get_tool_infos :722-760), `workspace/mod.rs` (__ts_rs test :236).
- Historical docs: `docs/superpowers/plans/2026-07-10-subagent-parity-roadmap.md`, `echo-agent-cli/docs/2026-07-16-agent-lifecycle-audit.md`, `2026-07-25-gui-tool-execution-lazy-loading.md`, `browser-runtime-design.md`, `MASTER-PLAN.md`.

## Out Of Scope

- Store reducer correctness / rendering / duplicate event handling — A-FE-02 (frontend projections), A-SRF-03 (chat surface integration).
- Tauri command lifecycle/state/authority — A-SRF-02 (its P1-01 `browser://event` and P2-01 duplicate projection are cross-referenced, not re-filed).
- TaskRuntime claim/recovery semantics — A-TSK-04 (read as dependency; terminal-status derivation cross-referenced only).
- Frontend architecture/performance — A-FE-03; formatting/build gates as full submission gates — Q-WEB-01.
- Dynamic GUI verification (no app launched) — Q-GUI-01 / Q-E2E-01.

## Inputs

- Root `AGENTS.md` (full; UTF-8/panic safety, no-duplicate-authority, framework-vs-app layering, "动手前先查是不是已经有了", read-only review), shared `README.md`, `REPORTING.md`, `TASKS.md` (A-FE-01 card), `zcode-ds/README.md`, report templates.
- Dependency task reports read: `A-SRF-02` (complete — command registration, setup closures, bridge/projector duplicate projection, browser://event), `A-TSK-04` (complete — claim/event-replay authority; terminal monotonicity).
- Historical documents treated as hypotheses: `docs/MASTER-PLAN.md`, `echo-agent-cli/docs/MASTER-PLAN.md`, the audit/roadmap docs listed in Scope (classified in V05-01).

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism (framework) | `serde`/ts-rs derive machinery, `ToolExecutionSummary`/`ToolExecutionOwner` app-core types as reusable app-core DTOs. No movement recommended. |
| EKO product policy (application, correct placement) | `types/api.ts` hand-written supplement, `endpoints.ts` dual-path API layer, Tauri command DTOs, event channel payloads, generated TS artifacts. |
| Adapter boundary (findings) | The Tauri commands' hand-rolled JSON builders for skills/MCP (panels.rs:346-377, mcp.rs:60-85) create a second, undocumented wire shape per name that shadows the typed `types/response.rs` DTOs (P2-02); `toolsApi`/`skillsApi`/`mcpApi` type their responses with the diverged hand-written types (P2-01/P2-02). |
| Duplicate search (terms + results, V01-01) | `ToolInfo`, `SkillInfo`, `McpServerInfo`, `McpToolInfo`, `ConnectMcpRequest`, `McpTransportConfig`, `FullConfigResponse`, `FullConfigUpdateRequest` vs `UpdateFullConfigRequest`, `ChatRequest`, `ChatResponse`, `ToolCallInfo`, `ContextStats`, `SessionInfo`, `StreamingEvent`, `ServerMessage`, `ClientMessage`, `ChatEvent`, `ExecutionEvent`, `ToolExecution`, `__ts_rs`. Result: 5 names deduplicated (re-exported from generated); 7+ names still double-defined with divergence; 3 HTTP-only generated families (StreamingEvent/ServerMessage/ClientMessage) + ChatRequest/ChatResponse/UpdateConfigRequest have zero consumers and no server. |
| Migration deletion | P2-01: delete hand-written `ToolInfo` (api.ts:179-185) and re-export generated; P2-02: delete the hand-rolled JSON shapes or make Tauri return the typed response.rs structs, then delete the superseded TS shape; P3-01: delete `ConnectMcpRequest`/`McpTransportConfig` from api.ts; P3-03: delete `agent.enable_tools` from the hand-written `FullConfigResponse`. |

## Current Path

Verified data flow (V01-01/V02-01/V03-01):

1. **Command surface**: `src-tauri/src/main.rs` -> `desktop.rs` -> `build_tauri_app` (mod.rs:29-773) registers 212 commands in `invoke_handler` (:69-310 incl. `terminal::*` :304-309). Frontend `endpoints.ts` calls `apiInvoke<T>('cmd', args)` (`lib/tauri-bridge.ts:164-173`) for the Tauri path and `get/post/put/del` (client.ts) for the HTTP path; `isTauri()` (tauri-bridge.ts:19-30) is memoized and true inside the desktop app. 184 invoked command names: 184/184 defined as `#[tauri::command]` fns and registered; arg keys match Rust snake_case params via Tauri camelCase conversion (verified samples: send_chat_message, task_runtime reads/writes, connect_mcp_server, terminal commands).
2. **chat://event**: single producer family (emit_chat_event chat.rs:114-143; direct emitters at :322/:370/:409/:516-534/:620/:656/:709/:1203/:1371/:1460) -> single listener `useTauriChat.ts:93` -> `handleChatEvent` (chatEventHandler.ts) covering all 19 variants.
3. **execution://event**: three producers — the SubagentEventBus bridge (mod.rs:353-768, kind=subagent + kind=tool summaries for subagent tools), `emit_tool_execution_summary` (chat.rs:185-208, kind=tool), `TauriExecutionProjector` (chat.rs:957-1114 -> :1419-1446, kind=run/task/subagent from `ChatDriverEvent::Execution`/ExecEvent) — consumed by `useTauriChat.ts:109` (kind=subagent -> subagentRunStore, kind=tool -> toolExecutionStore, kind=run run_started -> taskRuntimeStore.loadByConversation). Note: bridge and projector double-persist subagent tool events (A-SRF-02-P2-01, cross-referenced).
4. **Terminal**: `terminal-output`/`terminal-exit` (terminal.rs:148-166) -> Terminal.tsx:112-121.
5. **browser://event**: listener live (useBrowserEvents.ts:12) but producer dead — forwarder sits in the overwritten first `.setup()` (mod.rs:40-68; A-SRF-02-P1-01).
6. **TaskRuntime DTOs**: generated `TaskRun`/`TaskPlan`/`TodoItem`/`RuntimeTaskEvent`/`TaskRunStatus` etc. (ts-rs from app-core types) used directly by taskRuntimeStore and TaskRuntimePanel; `seq` serialized as string matching `list_task_events(since_seq: Option<String>)`.
7. **Dormant surface**: the HTTP/WebSocket DTO families (ChatRequest/ChatResponse/ServerMessage/ClientMessage/StreamingEvent/UpdateConfigRequest/UpdateFullConfigRequest HTTP-side, McpConnectionStatus "Connected" shape) describe a wire no server in this workspace produces (no axum binary; `axum` only an app-core dependency) and have zero frontend consumers — registered-but-dormant definitions.

## Findings

### A-FE-01-P2-01: Hand-written `ToolInfo` diverges from the live wire type — `parameters`/`need_approval` hidden and phantom `input_schema` consumed, so the Tools panel schema display is dead

- Priority: P2
- Confidence: high
- Layer: application (frontend type contract) with adapter drift (types/response.rs DTO)
- Evidence:
  - Hand-written type: `web-frontend/src/types/api.ts:179-185` — `{ name, description, source: string, input_schema?: Record<string, unknown>, enabled }`.
  - Wire type (generated from Rust): `web-frontend/src/generated/ToolInfo.ts` from `echo-agent-cli/echo-agent-app-core/src/types/response.rs:40-47` — `{ name, description, parameters: Value, enabled, need_approval, source: ToolSource }`.
  - Wire producer: `state.rs:722-760` `get_tool_infos` populates `parameters` (JSON schema), `need_approval`, `source` per tool; consumed by `src/tauri/commands/tools.rs:7-15` (`list_tools` returns `serde_json::to_value(infos)`).
  - Frontend consumption: `endpoints.ts:140-152` types `toolsApi.list` with the hand-written `ToolInfo` (import at endpoints.ts:7); `ToolsPanel.tsx:3` imports it; `ToolsPanel.tsx:89` renders `{tool.input_schema && <pre>{JSON.stringify(tool.input_schema, ...)}</pre>}`.
- Reachability: SettingsDialog (`src/components/layout/SettingsDialog.tsx:211`) -> ToolsPanel mount -> `toolsApi.list()` -> `list_tools` on every GUI settings open. Verified live path.
- Expected invariant: the TS type of every command response matches the serialized Rust type field-for-field; fields the UI renders must exist on the wire (AGENTS.md "防重复造轮子 / 单一事实源"; api.ts header "prefer updating the Rust-side serde derives and re-generating over hand-writing").
- Observed behavior: the wire carries `parameters` and `need_approval`; the TS type hides both and declares `input_schema`, which the backend never sends. The expanded tool card's schema block is always empty at runtime; tool permission state (`need_approval`) is invisible to the UI.
- Impact: user-visible — the schema display in the Tools panel is dead and the backend's schema JSON is never shown; the approval-needed flag is never surfaced; two `ToolInfo` definitions coexist with different field sets (generated one unused), guaranteeing future drift.
- Root cause: the 6-10 dedupe pass (api.ts:8-17, 33-36) re-exported only 5 types from `generated/`; `ToolInfo` was left hand-written from an older API shape (`input_schema` predates `parameters`), and no fixture pins the wire shape.
- Direction: delete the hand-written `ToolInfo` (api.ts:179-185), import `ToolInfo` from `../generated`, and change `ToolsPanel.tsx:89` to render `tool.parameters`; add `need_approval` display or intentionally omit. Delete the superseded definition.
- Regression validation: a vitest fixture rendering ToolsPanel against a real `list_tools` payload (with `parameters`/`need_approval`) asserting the schema block renders; `tsc` passes after the type swap.
- Validation reports: [V01-01](../validations/A-FE-01/V01-01.md), [V05-01](../validations/A-FE-01/V05-01.md)

### A-FE-01-P2-02: Same-name type families `SkillInfo` / `McpServerInfo` / `McpToolInfo` have two divergent wire shapes — Tauri commands' hand-rolled JSON vs generated ts-rs types (dormant HTTP surface) — with one TS name, two Rust authorities

- Priority: P2
- Confidence: high (mechanism), medium (impact confined to the dormant HTTP path today)
- Layer: adapter (Tauri commands) with application type-contract drift
- Evidence:
  - Tauri wire shapes are hand-rolled: `list_skills` -> `skill_descriptor_json`/`hub_skill_json` (panels.rs:346-377: name/description/triggers/file/loaded/source/category/is_baseline/is_builtin/upstream_version/version/author/tags/has_sandbox/depends_on/missing_dependencies/has_updates/license); `list_mcp_servers` (mcp.rs:60-85: lowercase `status` `"connected"/"error"/"disconnected"/"disabled"` + `enabled`, `tools` as `{name, description}` only).
  - Generated shapes from Rust `types/response.rs`: `SkillInfo` :93-99 (`enabled`, `tool_names`, `source: SkillSource`), `McpServerInfo` :59-69 (`status: McpConnectionStatus` = `"Connected"|"Disconnected"|{"error": string}` — response.rs:72-79, no `enabled`, required `tool_count`/`tools` with `input_schema`), `McpToolInfo` :81-87.
  - Frontend uses only the hand-written variants: `endpoints.ts:6-47` imports `SkillInfo`/`McpServerInfo` from `../types/api` (api.ts:187-206, 241-256); generated `SkillInfo.ts`/`McpServerInfo.ts` are exported from `generated/index.ts` but imported by no component.
  - Consumers confirm the hand-written shapes match the live Tauri wires: `McpPanel.tsx:277,310-317` compares lowercase `srv.status === 'connected'/'error'`; SkillsPanel consumes file/triggers/category (hand-written fields).
- Reachability: live — GUI Skills/MCP panels via `list_skills`/`list_mcp_servers`; the generated shapes are reachable only if the HTTP path re-activates (no server today, V02-01) or a developer imports `generated/SkillInfo` expecting the Rust type.
- Expected invariant: one wire shape per type name (AGENTS.md "严禁平行实现同一语义"; api.ts header claims generated types are canonical); the type name in the frontend describes exactly the JSON the backend sends.
- Observed behavior: `SkillInfo` and `McpServerInfo` each have two Rust-side producers with different fields and value domains; the frontend type matches one and hides the other. Already drifted (`McpConnectionStatus` "Connected" vs Tauri "connected"; `enabled`/`tool_names` present in one shape only).
- Impact: material maintainability defect and misleading public types — the generated types (the ones a developer would trust, being ts-rs output) describe a wire no live server produces; if the HTTP surface is ever re-enabled (endpoints.ts keeps the dual path), the same TS types silently mis-parse both surfaces; no compile-time error protects either side.
- Root cause: the HTTP API DTO layer (`types/response.rs`) and the Tauri commands evolved separately — Tauri commands build JSON by hand (json! macros) instead of returning the typed DTOs; the 6-10 dedupe then removed only the types whose shapes happened to agree.
- Direction: make the Tauri commands return the typed `types/response.rs`/`types/request.rs` DTOs (or add typed DTOs for the skill-hub shape) so each name has one Rust authority; regenerate TS; delete the divergent hand-written duplicates (api.ts:187-206, 241-256) and the hand-rolled json! shapes they mirror. Do not keep two shapes.
- Regression validation: vitest fixtures asserting `list_skills`/`list_mcp_servers` payloads validate against the single TS type (runtime schema check or typed fixture); grep proving one definition per name after cleanup.
- Validation reports: [V01-01](../validations/A-FE-01/V01-01.md), [V02-01](../validations/A-FE-01/V02-01.md), [V05-01](../validations/A-FE-01/V05-01.md)

### A-FE-01-P3-01: `ConnectMcpRequest` / `McpTransportConfig` hand-written types describe a wire shape that matches neither the Rust type nor the endpoint signature, and the connect flow has zero frontend callers (dead surface)

- Priority: P3
- Confidence: high
- Layer: application (dead frontend type + dormant flow)
- Evidence:
  - Hand-written: api.ts:258-266 — `{ name, transport: {stdio: {command, args?, env?}} | {http: ...} | {sse: ...} }` (variant-name-keyed, nested, optional arrays).
  - Rust: `types/request.rs:25-53` — `ConnectMcpRequest { name, #[serde(flatten)] transport: McpTransportConfig }` with `#[serde(tag = "transport", rename_all = "lowercase")]` — wire `{name, transport: "stdio", command, args, env}` (flattened, internally tagged). Generated `ConnectMcpRequest.ts` matches the Rust shape.
  - Endpoint: `endpoints.ts:204` types `connect` with `{ name: string; transport: { transport: string; [key: string]: unknown } }` — a third, different shape.
  - Dead: `mcpApi.connect` has zero callers in components/stores; `McpManagerPanel` (the only component referencing a connect form) is never imported anywhere (V01-01/V02-01).
- Reachability: none in the GUI. The Tauri command itself (`connect_mcp_server`, mcp.rs:210-258) is registered and reachable by a crafted client, and it deserializes the flattened internally-tagged shape — sending the api.ts shape would fail validation.
- Expected invariant: no dead, wrong-shape type under a live-sounding name; every exported type describes a wire the backend accepts (AGENTS.md "删死代码").
- Observed behavior: three shapes for one request family, none reconciled; the flow is dormant.
- Impact: misleading API surface; a future developer wiring the connect UI would hit silent deserialization failure on the Tauri path (or the MCP allowlist validation after wrong parsing).
- Root cause: the connect UI was never completed/wired; the type was written before the Rust `#[serde(flatten)]`+internally-tagged design.
- Direction: delete the hand-written `ConnectMcpRequest`/`McpTransportConfig` from api.ts and import the generated ones when the connect flow is actually built; keep the endpoint param type aligned to the generated shape.
- Regression validation: grep for `ConnectMcpRequest` after deletion (only generated remains); a fixture posting the generated shape to `connect_mcp_server` succeeds (Q-E2E-01 when the UI lands).
- Validation reports: [V01-01](../validations/A-FE-01/V01-01.md)

### A-FE-01-P3-02: Generated TS artifacts are committed prettier-formatted but a fresh ts-rs regeneration writes raw unformatted output — 79 generated files are currently modified (formatting-only drift) and the prettier gate fails (exit 1)

- Priority: P3
- Confidence: high
- Layer: application (build hygiene)
- Evidence:
  - `git status`: exactly 79 modified files, all `web-frontend/src/generated/*.ts`; normalized comparison (quotes/whitespace/commas stripped) shows zero semantic differences — working tree is raw ts-rs output, HEAD is prettier-formatted (V01-01).
  - Generation mechanism: `echo-agent-app-core/src/workspace/mod.rs:236` `#[cfg(feature = "__ts_rs")]` test (`echo-agent-app-core/Cargo.toml:56` check-cfg), i.e. `cargo test --features __ts_rs` rewrites the directory.
  - Formatting expectation: `web-frontend/.prettierrc` (`singleQuote: true`, `trailingComma: "es5"`, `printWidth: 100`) matches the committed HEAD shape, not the raw output.
  - Gate failure: `npx prettier --check "src/**/*.{ts,tsx}"` reports issues in exactly the 79 generated files; exit code 1 (V04-03).
- Reachability: any developer/build that runs the `__ts_rs` generation (the documented workflow per workspace/mod.rs:239) dirties the tree and turns the formatting gate red until `prettier --write` is manually re-run; the committed state is not reproducible from generation alone.
- Expected invariant: the repo is clean after the documented generation step; committed generated files are reproducible from the generator output (AGENTS.md: fmt/prettier gate must pass; "提交前门禁").
- Observed behavior: generation → dirty tree + gate failure; the committed state depends on an undocumented post-generation prettier pass.
- Impact: CI/fmt-gate fragility and permanent dirty status for anyone who regenerates; the current worktree is exactly in this state (Q-WEB-01 will observe the same prettier failure).
- Root cause: no generation script wraps ts-rs output with prettier (and no CI check enforces it).
- Direction: add a generation script (or document in the `__ts_rs` test comment) that runs `npx prettier --write src/generated/**/*.ts` immediately after export; or commit raw output and exclude `generated/` from the prettier gate; then restore/format the current tree.
- Regression validation: `git status` clean after generate→format; `npx prettier --check` exit 0; `git diff` empty after a fresh generation cycle.
- Validation reports: [V01-01](../validations/A-FE-01/V01-01.md), [V04-03](../validations/A-FE-01/V04-03.md)

### A-FE-01-P3-03: Hand-written `FullConfigResponse` adds a phantom `agent.enable_tools` field absent from the Rust `AgentConfigResponse`, alongside the generated counterpart

- Priority: P3
- Confidence: high (field absence certain), low (impact — no current consumer reads the field)
- Layer: application (frontend type contract)
- Evidence:
  - Hand-written: api.ts:504-536 — `agent.enable_tools: boolean` (:520) inside `FullConfigResponse`.
  - Rust: `types/response.rs:112-122` `AgentConfigResponse` has no `enable_tools` (only model/system_prompt/max_iterations/token_limit/enable_memory/enable_human_loop/session_id/available_models); generated `FullConfigResponse.ts`/`AgentConfigResponse.ts` match the Rust shape.
  - Live consumer: `ConfigPanel.tsx:3,7` uses the hand-written `FullConfigResponse` via `get_full_config` (config.rs:156-161 returns the Rust `FullConfigResponse`).
- Reachability: GUI Settings → ConfigPanel on every open; the phantom field is undefined at runtime but no current component reads `agent.enable_tools` (grep zero hits in components).
- Expected invariant: TS response types contain only fields the wire provides (or the Rust type gains the field).
- Observed behavior: type allows reading a field the backend never sends; value would be `undefined` if read.
- Impact: latent bug (a future UI toggle for tools would read `undefined`); type noise. Low today.
- Root cause: hand-written type predates/diverged from `AgentConfigResponse`; the 6-10 dedupe did not cover it.
- Direction: delete `enable_tools` from api.ts:520 (or add the field to the Rust struct if the UI needs it); prefer importing the generated `FullConfigResponse`.
- Regression validation: tsc + ConfigPanel fixture asserting the rendered agent settings come only from real fields.
- Validation reports: [V01-01](../validations/A-FE-01/V01-01.md)

## Cross-Verified Dependency Findings (canonical IDs elsewhere; independently confirmed here)

| Canonical ID | Claim | Independent confirmation |
|---|---|---|
| A-SRF-02-P1-01 | `browser://event` producer dead (first `.setup()` overwritten) | Confirmed in the event-channel matrix (V02-01): listener `useBrowserEvents.ts:12` live, producer in the discarded closure mod.rs:40-68. |
| A-SRF-02-P2-01 | bridge + projector double-persist subagent tool events, duplicate `execution://event kind=tool` | Confirmed (V02-01/V03-01): both producers call `tool_executions.start/finish` and `emit_tool_execution_summary` for the same owner/call_id. Frontend tool-card reducers must tolerate duplicates (A-FE-02). |
| A-CHAT-01-P1-01 | error-terminated turns labeled `completed`; cancel fabricates error | Confirmed in the TurnStatus derivation (V03-01): chat.rs:663-668 derives terminal from `outcome.is_ok()` which is always Ok for envelope-normalized errors. |
| A-CHAT-01-P2-01 | `ChatDriverEvent::Interrupt` dead; GUI emits `InterruptPrompt` directly | Confirmed (V03-01): chat.rs:516-534 emits `ChatEvent::InterruptPrompt` directly; TS `interrupt_prompt` variant is wired to the store. |

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition + duplicate search (DTO field matrix; generated vs hand-written drift; normalized 79-file diff) | yes | passed | [V01-01](../validations/A-FE-01/V01-01.md) |
| V02 | Registration and runtime reachability (184 commands, invoke_handler, event producer/consumer pairs, dormant HTTP surface) | yes | passed | [V02-01](../validations/A-FE-01/V02-01.md) |
| V03 | Invariant/edge cases (ChatEvent 19/19 variant coverage; optional/null semantics; execution://event payload contracts; terminal events; TurnStatus values) | yes | passed | [V03-01](../validations/A-FE-01/V03-01.md) |
| V04 | `npx vitest run` | yes | passed (exit 0, 26 files / 101 tests) | [V04-01](../validations/A-FE-01/V04-01.md) |
| V04 | `npx tsc -b` | yes | passed (exit 0) | [V04-02](../validations/A-FE-01/V04-02.md) |
| V04 | `npx prettier --check` (frontend gate) | yes | failed (exit 1, 79 generated files) | [V04-03](../validations/A-FE-01/V04-03.md) |
| V05 | Historical-document drift (api.ts header, MASTER-PLAN, lifecycle audit, lazy-loading doc, browser-runtime-design) | yes | passed | [V05-01](../validations/A-FE-01/V05-01.md) |

All required validations executed; every reported command has a known exit code; no validation is pending. No command that regenerates `web-frontend/src/generated/*.ts` was executed; the pre-existing dirty state was recorded as a baseline before and after every run and was not modified by this review.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| api.ts:1-36 "canonical types are auto-generated; hand-written only for UI-only/not-yet-generated/legacy" + "(6-10) re-export previously duplicated types" | stale/regressed | 7+ same-name hand-written types still exist and have diverged from generated (P2-01/P2-02/P3-03; V01-01, V05-01) |
| MASTER-PLAN.md:75 generated TypeScript preserves TodoStatus/RuntimeEventKind terminals | current | generated + consumed consistently (V03-01) |
| 2026-07-16-agent-lifecycle-audit.md:19 run_status pollutes taskRuntimeStore.activeRun.status | fixed | chatStore.setRunStatus touches only the chat store (chatStore.ts:391-397; V05-01) |
| 2026-07-16-agent-lifecycle-audit.md:139 chat turns fake execution://event kind=run/run_started with message_key | fixed | current sink emits ChatEvent::RunStatus; kind=run originates only from ExecEvent (chat.rs:620,1365-1373,921; executor.rs:352-360; V05-01) |
| 2026-07-25-gui-tool-execution-lazy-loading.md:91-92 kind=tool carries only ToolExecutionSummary | current | emit_tool_execution_summary (chat.rs:185-208); chat://event carries ChatEvent only (V03-01/V05-01) |
| browser-runtime-design.md:215 browser://event forwarder | regressed | producer in overwritten first `.setup()` (mod.rs:40-68; A-SRF-02-P1-01, V02-01) |

## Coverage And Uncertainty

- All conclusions are static except the three V04 frontend runs; no GUI process was launched (Q-GUI-01/Q-E2E-01 own dynamic confirmation). The ToolsPanel dead-schema behavior is proven by wire-field comparison, not observed on screen.
- The 184-command arg-key check was sampled (chat/task/mcp/terminal families), not exhaustively machine-verified for every command; no mismatch was found in the samples.
- The dormant HTTP path assumption ("no server") is based on repo-wide grep (no axum router, no server binary); if a server is delivered in a later milestone, the generated HTTP shapes become live and P2-02 becomes P1-severity.
- `kind=task` execution://event events have no frontend listener by design (polling path used); verified no consumer exists, so no event is silently mis-typed.
- The generated-dir dirty state predates this review (previous interrupted attempt); this review neither regenerated nor restored it — restoration is left to the implementation milestone (P3-02).
- Cross-process/dual-driver and real subagent runs were not exercised (A-TSK-04/Q-FLT-02 scope).

## Handoff

- Downstream tasks may rely on: the emitted-payload contracts (chat://event, execution://event kind=tool/subagent/run, terminal events) match their TS types variant-for-variant (V03-01); the command surface is complete and registered (V02-01); TaskRuntime DTOs are single-source generated; the drift findings above (P2-01 ToolInfo, P2-02 SkillInfo/McpServerInfo dual shapes, P3-01..03) with the generated-dir state (P3-02).
- Reports to read: this report + V01-01..V05-01; dependency reports A-SRF-02 (P1-01 browser://event, P2-01 duplicate tool projection) and A-TSK-04 (claim/event authority). Cross-referenced canonical IDs: A-CHAT-01-P1-01, A-CHAT-01-P2-01.
- Stale conditions: this report becomes stale if `types/api.ts`, `src/generated/` regeneration workflow (P3-02 fix), `types/response.rs`/`request.rs` shapes, the Tauri skill/MCP command JSON builders (panels.rs:346-377, mcp.rs:60-85), `ChatEvent` (chat.rs:30-112), the execution://event emitters, or `chatEventHandler.ts` change; also if an Axum server is introduced (dormant HTTP shapes become live).
- Follow-up task IDs (fixes are not implemented in this review): A-FE-02 (tool-card duplicate tolerance from A-SRF-02-P2-01; ToolInfo render fix), A-SRF-03 (run_status/chat-surface reconciliation), X-EVT-01 (event-conformance matrix incl. browser://event), X-TOL-01 (tool schema/artifact conformance incl. parameters vs input_schema), Q-WEB-01 (prettier gate after P3-02), Q-E2E-01 (Tools panel schema render, MCP status rendering, connect flow), S-RDM-01 (roadmap: P2-01/P2-02 type unification, P3-02 generation script).
