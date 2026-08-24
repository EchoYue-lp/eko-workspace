# A-FE-01: Rust/TypeScript API and event type contract

> Status: complete
> Reviewer: Codex review subagent
> Executor: Codex review subagent
> Review date: 2026-08-13
> `echo-agent` commit: 3aa7929928442aab91e4dce9c426d909a5f0a1ab
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: CLI clean; framework externally dirty in paths listed by V00 and excluded from review, except clean `src/agent/subagent/events.rs`
> Accepted by: Codex primary reviewer after independent source-anchor,
> reachability, finding-count, link, executor, commit, and isolation sampling.

## Question

Do Tauri command DTOs, emitted payloads, TypeScript endpoint types, and stores match field-for-field and variant-for-variant while preserving the canonical TaskRun/PlanTask/SubagentRun authority across GUI and other surfaces?

## Scope

- `echo-agent-cli/echo-agent-app-core/src/error.rs`, `types/request.rs`, `types/response.rs`, and `tasks/task_runtime/types.rs`.
- `echo-agent-cli/src/tauri/error.rs`, command registration, chat/execution emitters, workflow commands, and terminal command response construction.
- `echo-agent-cli/web-frontend/src/generated`, `types/api.ts`, `api/endpoints.ts`, `lib/tauri-bridge.ts`, event hooks, TaskRuntime/Subagent stores, WorkflowPanel, and TerminalDrawer.
- Clean canonical Subagent event definition in `echo-agent/src/agent/subagent/events.rs` solely for the variant matrix.

## Out Of Scope

- Reducer identity, ordering, monotonicity, and rendering details owned by `A-FE-02`.
- Desktop startup, browser bridge lifecycle, terminal resource cleanup, and GUI-only workflow layering owned by `A-SRF-02`.
- Claim/revision/recovery/hook semantics owned by `A-TSK-04`.
- Any source fix, Cargo/rustc/frontend build, test execution, dynamic fixture, or network access.
- All externally dirty `echo-agent` paths recorded in V00.

## Inputs

- Root `AGENTS.md` product constraints.
- `docs/comprehensive-review/TASKS.md` exact A-FE-01 card.
- `docs/comprehensive-review/REPORTING.md` and `docs/comprehensive-review/codex/README.md`.
- Authorized Codex dependency reports `A-SRF-02` and `A-TSK-04`.
- V00 discloses an accidental A-CHAT-01 report overread; none of its content was used.

## Layering Decision

The generic Subagent event vocabulary stays in the framework. Canonical TaskRun, PlanTask, SubagentRun, event, artifact, and their ts-rs bindings are application-core domain contracts shared by every EKO surface. Tauri and React are thin adapters/projections: they may add transport identity or UI state, but must not silently delete variants, reinterpret fields, or assert invented response shapes. Searches covered DTO/type names, serde/ts-rs attributes, command names, event names, emit/listen paths, generated imports, store projections, and mounted callers. No second TaskRun/PlanTask/SubagentRun domain model was found; the contract fragmentation is confined to handwritten Tauri/chat/execution/error/workflow/terminal boundaries.

## Current Path

- Canonical task data: Rust TaskRuntime types (`tasks/task_runtime/types.rs`) -> ts-rs generated files -> `taskRuntimeApi` -> `taskRuntimeStore` and task components. This path reuses one authority.
- Chat stream: Rust handwritten `ChatEvent` -> `emit_chat_event` adds `message_key`/`conversation_id` -> `chat://event` -> handwritten TypeScript `ChatEvent` -> `handleChatEvent`.
- Subagent stream: framework `SubagentEventBus` -> dynamic match/`serde_json::Value` assembly in Tauri setup -> `execution://event` -> `Record<string, unknown>` -> unchecked cast to handwritten `ExecutionEvent` -> Subagent/tool stores.
- Commands: frontend generic assertions in `endpoints.ts` -> `apiInvoke<T>` -> registered workflow/terminal/task commands. Runtime has no decoder confirming `T`.
- Errors: `IpcError` serializes `{kind,message,error}` -> `apiInvoke` catch -> new native Error containing only a message.

## Findings

### A-FE-01-P1-01: Tauri drops every live Subagent thinking and output-token event before the GUI contract

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent/src/agent/subagent/events.rs:114`; `echo-agent/src/agent/subagent/events.rs:153`; `echo-agent-cli/src/tauri/mod.rs:383`; `echo-agent-cli/src/tauri/mod.rs:649`; `echo-agent-cli/web-frontend/src/stores/subagentRunStore.ts:31`
- Reachability: live Agent SubagentEventBus subscription in Tauri setup -> explicit bridge match -> `continue` for thinking start/delta/end and token delta -> GUI `execution://event` listener/store whose event union also omits them.
- Expected invariant: TUI, GUI, CLI, and channel surfaces expose the same Agent capabilities; a transport adapter must preserve user-visible execution variants or provide a lossless alternate projection.
- Observed behavior: lifecycle, usage, tools, and terminal results cross the bridge, but all live Subagent reasoning and final-answer token deltas are discarded. The omission is encoded in both Rust control flow and the TypeScript union.
- Impact: GUI Subagent cards cannot show live reasoning/output progress that the framework emits; users see only lifecycle/usage/final results. This is a major surface-parity failure, not a valid GUI product distinction.
- Root cause: `execution://event` is a hand-maintained subset assembled with strings/dynamic JSON instead of an exhaustive shared Rust/TypeScript event contract.
- Direction: define one serializable application event enum that losslessly adapts framework Subagent events, generate its TypeScript union, forward all user-visible variants, and delete the current string/JSON subset match plus handwritten event-kind list once migrated. Keep tool detail authority in `toolExecutionStore`; do not create a second TaskRuntime authority.
- Regression validation: serialize every Subagent execution-flow variant in Rust, decode it in TypeScript, feed GUI reducers/components, and assert thinking/token order plus exactly one terminal result.
- Validation reports: [V02](../validations/A-FE-01/V02-01.md), [V04](../validations/A-FE-01/V04-01.md), [V07](../validations/A-FE-01/V07-01.md)

### A-FE-01-P2-01: Live workflow and exported terminal endpoint types assert response fields Rust never returns

- Priority: P2
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent-cli/src/tauri/commands/panels.rs:677`; `echo-agent-cli/src/tauri/commands/panels.rs:710`; `echo-agent-cli/web-frontend/src/types/api.ts:303`; `echo-agent-cli/web-frontend/src/components/workflow/WorkflowPanel.tsx:72`; `echo-agent-cli/src/tauri/terminal.rs:278`; `echo-agent-cli/src/tauri/terminal.rs:402`; `echo-agent-cli/web-frontend/src/api/endpoints.ts:956`
- Reachability: mounted WorkflowPanel -> workflowApi -> registered Tauri workflow commands; exported terminalApi -> registered Tauri terminal commands (TerminalDrawer currently bypasses this endpoint in Tauri mode).
- Expected invariant: `apiInvoke<T>` response assertions match command serialization field-for-field; live UI fields are present.
- Observed behavior: workflow list omits required `definition` and `status`, yet WorkflowPanel renders `wf.status`; create returns `{success,id}` while asserted as WorkflowInfo. Terminal create returns `{id,pid}` while asserted as `{id,cwd,created_at}`, and close returns `{success}` while asserted as `{closed}`.
- Impact: the live workflow panel renders an undefined status and future consumers can rely on fields that never exist. Terminal endpoint lies are latent today because the Tauri drawer calls raw `apiInvoke`, but the exported contract is materially misleading and can fail on reuse.
- Root cause: command handlers return ad hoc `serde_json::Value` while the frontend independently invents structural result types; generic `apiInvoke<T>` performs an unchecked cast.
- Direction: introduce one typed Rust response DTO per command (or a shared command-result schema), generate/export it, use it in endpoints, and delete the handwritten WorkflowInfo/TerminalSession assertions or rename truly distinct HTTP/Tauri shapes. Preserve A-SRF-02 ownership of workflow layering and terminal lifecycle.
- Regression validation: fixture-test list/get/create/delete/execute and create/list/close responses against frontend decoders, including rendered workflow status.
- Validation reports: [V03](../validations/A-FE-01/V03-01.md), [V07](../validations/A-FE-01/V07-01.md), [V08](../validations/A-FE-01/V08-01.md)

### A-FE-01-P2-02: Generated optional fields contradict serde omission semantics

- Priority: P2
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent-cli/echo-agent-app-core/src/error.rs:23`; `echo-agent-cli/echo-agent-app-core/src/types/response.rs:218`; `echo-agent-cli/web-frontend/src/generated/ApiError.ts:22`; `echo-agent-cli/web-frontend/src/generated/ServerMessage.ts:8`
- Reachability: WebError -> ApiError HTTP response and ServerMessage -> live WebSocket serialization -> generated frontend types.
- Expected invariant: a Rust Option omitted with `skip_serializing_if` is represented as an optional TypeScript property; nullable means explicit JSON null.
- Observed behavior: Rust omits ApiError `details`/`request_id` and every non-Pong ServerMessage `id` when None, while generated TypeScript requires the property and permits only `T | null`.
- Impact: compile-time narrowing lies about runtime values: consumers checking `=== null` miss `undefined`, object construction/fixtures must add fields Rust does not send, and generated bindings cannot serve as faithful decoders.
- Root cause: ts-rs export is used without an annotation/generation policy that maps serde field omission to TypeScript optionality.
- Direction: align serde and ts-rs annotations so omitted fields generate `?:`, reserve `| null` for explicit null, and regenerate rather than hand-edit binding files.
- Regression validation: golden JSON for Some/None variants checked against TypeScript compile-time fixtures and runtime schema decoding.
- Validation reports: [V01](../validations/A-FE-01/V01-01.md), [V05](../validations/A-FE-01/V05-01.md), [V07](../validations/A-FE-01/V07-01.md)

### A-FE-01-P2-03: The shared Tauri adapter discards the machine-readable IpcError contract

- Priority: P2
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent-cli/src/tauri/error.rs:72`; `echo-agent-cli/src/tauri/error.rs:102`; `echo-agent-cli/web-frontend/src/lib/tauri-bridge.ts:44`; `echo-agent-cli/web-frontend/src/lib/tauri-bridge.ts:164`
- Reachability: registered command returns IpcError -> Tauri rejects invoke with serialized object -> shared `apiInvoke` catch -> endpoints/components receive a new untyped Error.
- Expected invariant: typed validation/not-found/internal categories survive the transport so callers can select recovery and presentation without parsing localized messages.
- Observed behavior: Rust emits `kind`, `message`, and legacy `error`; `apiInvoke` retains only the message and throws away the other fields. No frontend IpcError type or structured match remains.
- Impact: recovery conflicts, stale-file validation, not-found resources, and internal failures are indistinguishable to every endpoint using the shared adapter; callers can only show or parse prose.
- Root cause: the error normalization layer treats a structured IPC payload as display text instead of a typed domain error.
- Direction: generate/share the IPC error shape, preserve fields in a typed Error subclass or Result decoder, and delete the lossy catch/rethrow once callers migrate. Do not add cloud-style permission gates; this is error integrity for a local application.
- Regression validation: make each IpcError variant cross a mocked Tauri boundary and assert kind/message preservation at endpoint/store callers.
- Validation reports: [V02](../validations/A-FE-01/V02-01.md), [V06](../validations/A-FE-01/V06-01.md), [V07](../validations/A-FE-01/V07-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Exact inputs, commits, dirty-path isolation | yes | failed/disclosed | [V00](../validations/A-FE-01/V00-01.md) |
| V01 | Definition, generated-file, and duplicate-authority search | yes | passed | [V01](../validations/A-FE-01/V01-01.md) |
| V02 | Registration and runtime reachability | yes | passed | [V02](../validations/A-FE-01/V02-01.md) |
| V03 | Tauri command DTO field matrix | yes | failed/finding | [V03](../validations/A-FE-01/V03-01.md) |
| V04 | Enum/event variant coverage | yes | failed/finding | [V04](../validations/A-FE-01/V04-01.md) |
| V05 | Optional/null and numeric semantics | yes | failed/finding/risk | [V05](../validations/A-FE-01/V05-01.md) |
| V06 | Typed IPC error preservation | yes | failed/finding | [V06](../validations/A-FE-01/V06-01.md) |
| V07 | Existing serialization/fixture test inventory | yes | failed/gap | [V07](../validations/A-FE-01/V07-01.md) |
| V08 | Authorized dependency classification and deduplication | yes | passed | [V08](../validations/A-FE-01/V08-01.md) |
| V09 | Generated/fixture serialization execution | future | not_run | [V09](../validations/A-FE-01/V09-01.md) |
| V10 | Exact-link/header/source-isolation integrity gate | yes | passed | [V10](../validations/A-FE-01/V10-01.md) |
| V30 | Primary acceptance sampling | yes | passed | [V30](../validations/A-FE-01/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| A-SRF-02 workflow/terminal DTO observations | current | V03; A-FE-01 owns the field contract, while A-SRF-02 retains lifecycle/layering ownership |
| A-SRF-02 terminal/browser lifecycle findings | current, out of scope | V08 |
| A-TSK-04 canonical claim/revision/recovery findings | current, out of scope | V01 and V08 confirm no second TaskRuntime authority was introduced |
| Source comment that execution://event is unified | current but incomplete | V04 shows it is the single GUI channel while four user-visible variants are discarded |

## Coverage And Uncertainty

- The review statically sampled all canonical TaskRuntime definitions and the highest-risk handwritten Tauri boundaries, but did not produce a field matrix for every `serde_json::Value` command in the application.
- Dynamic serialization, Tauri IPC behavior, frontend compilation, and reducer fixtures were expressly not run; V09 is future work and does not imply a pass.
- All 79 named generated files exist, but freshness against Rust source was not executable in this stage.
- `SubagentArtifactResult.bytes` uses generated bigint while the dynamic JSON adapter accepts numbers/strings/bigints. A real current overflow impact was not established, so this remains a regression risk rather than a finding.
- ChatEvent's Rust and TypeScript variants presently align by inspection, but the two handwritten definitions plus optional `message_key` remain drift-prone. No separate finding is raised without a current mismatch.
- The externally dirty framework paths listed in V00 were not used. Any source commit or changes to event/DTO/command registration make this report stale.

## Handoff

- Primary should independently sample P1-01 at `src/tauri/mod.rs:383` and compare it with the clean framework event variants before acceptance.
- A-FE-02 may rely on the canonical TaskRuntime import result in V01 and the wire-event subset in V04, but should own only reducer identity/order/monotonicity/rendering defects.
- The implementation roadmap should first establish generated typed Tauri command/event/error boundaries, then delete the handwritten/dynamic authorities identified here; acceptance requires Rust-produced fixtures consumed by TypeScript tests.
- Preserve A-SRF-02 and A-TSK-04 ownership boundaries and the local-assistant threat model. Do not introduce SQLite or cloud-service permission gates.
