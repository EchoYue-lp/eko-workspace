# A-OBS-01: Diagnostics, webhooks, and operational visibility

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa (read-only; supplies `RunStore`, `RunEvent`, `redact_secrets`)
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only review + targeted `cargo test` execution)

## Question

Are diagnostics, run context, webhook events, and logs wired to live
lifecycle facts without globals, secret leakage, or misleading success?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent-cli/echo-agent-app-core/src/observability/` (full module):
  - `mod.rs` (re-exports)
  - `types.rs` (full, 140 lines — `DiagnosticRunSummary`,
    `RunDiagnostics`, `TraceInvocationDiagnostic`, `LlmCallDiagnostic`,
    `CacheDiagnostic`, `ContextDiagnostic`, `CompressionDiagnostic`,
    `DiagnosticIssue`).
  - `diagnostics.rs` (full, 612 lines — `list_diagnostic_runs`,
    `load_run_diagnostics`, `format_run_diagnostics`,
    `summarize_group`, `aggregate_status`, `build_run_diagnostics`,
    `build_cache_diagnostic`, `build_issues`; 3-test `#[cfg(test)]`
    module).
- `echo-agent-cli/echo-agent-app-core/src/webhook/` (full module):
  - `mod.rs` (re-exports)
  - `emitter.rs` (full, 209 lines — `WebhookEmitter`,
    `WebhookEndpoint`, `from_config`, `reload_from_config`, `emit`,
    `deliver`; documents removal of `init_global`/`emit_global`/
    `global_emitter`).
  - `events.rs` (full, 55 lines — `WebhookEvent` enum, `WebhookPayload`).
- `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:83-176`
  (`WebhookTurnObserver`) and `:425-569` (`drive_chat_inner`'s call to
  `webhook_observer.observe/finish`).
- `echo-agent-cli/echo-agent-app-core/src/state.rs:398-406, 469, 569-595`
  (`WebhookState`, `ObservabilityState`, `AppState::from_shared`).
- `echo-agent-cli/echo-agent-app-core/src/tool_execution.rs` (full,
  900+ lines — `ToolExecutionRepository`, `start`/`append_output`/
  `finish`/`cancel`, `preview_args`).
- `echo-agent-cli/echo-agent-app-core/src/infra.rs:184-385`
  (`create_agent_with_diagnostics` including the JsonlRunStore wiring
  at `:374-385`) and `:1536-1630` (logging/telemetry init).
- `echo-agent-cli/echo-agent-app-core/src/runtime.rs:60-200`
  (`AgentRuntime::bootstrap` step 1 calls `create_agent_with_diagnostics`).
- `echo-agent-cli/echo-agent-app-core/src/config_watcher.rs:200-278`
  (live-reload `handle_config_change` and `emitter.reload_from_config`).
- `echo-agent-cli/echo-agent-app-core/src/scheduler/runner.rs:42-128`
  (`build_fire_fn` and its single `WebhookEvent::CronTaskCompleted`
  emit site at `:116`).
- `echo-agent-cli/src/main.rs:100-310` (headless emitter construction +
  TUI/REPL/channels entry wiring).
- `echo-agent-cli/src/tauri/desktop.rs:125-220` (GUI emitter
  construction + AppState override + bootstrap).
- `echo-agent-cli/src/cli/modes.rs:28-130` (`start_headless_services`
  state.webhook.emitter override + `run_cli_mode`/`run_channels_mode`).
- `echo-agent-cli/src/cli/channels.rs:240-270` (per-message
  ChatResources.webhook_emitter).
- `echo-agent-cli/src/cli/repl.rs:495-545` (REPL ChatResources).
- `echo-agent-cli/src/tui/events.rs:1294-1435, 2010-2224` (TUI
  ChatResources + `TuiChatSink::on_event` mapping), and
  `:3698-4138` (TUI `/trace`, `/runs`, `/run` slash commands).
- `echo-agent-cli/src/cli/cmd_impls/observability.rs` (full, 179 lines —
  CLI slash commands `/trace`, `/prompt-diagnostics`, `/runs`,
  `/run show|export`).
- `echo-agent-cli/src/tauri/commands/panels.rs:1129-1178`
  (`list_diagnostic_runs` / `get_run_diagnostics` Tauri commands).
- `echo-agent-cli/src/tauri/commands/chat.rs:1148-1572`
  (`TauriChatSink.handle_tool_event` + `agent_event_to_chat_event`).
- `echo-agent-cli/web-frontend/src/api/endpoints.ts:630-741`
  (TypeScript diagnostics DTOs + `runDiagnosticsApi`).
- `echo-agent-cli/web-frontend/src/components/observability/ObservabilityPanel.tsx`
  (frontend consumer).
- Framework dependencies read (read-only contract):
  - `echo-agent/src/trace/mod.rs:38-97, 301-560, 601-801` (`Run`,
    `RunEvent`, `RunStore` trait, `JsonlRunStore` save/append).
  - `echo-agent/src/security.rs:33-128` (`SECRET_PATTERNS`,
    `redact_secrets`).
  - `echo-agent/src/config.rs:640-661` (`WebhooksConfig`,
    `WebhookEntryConfig`).
  - `echo-agent/echo-core/src/agent/mod.rs:143-310` (`AgentEvent`
    variants — needed for `agent_event_to_chat_event` reachability).

## Out Of Scope

Deferred to downstream tasks:

- **A-CHAT-01 / F-RCT-02 / F-RCT-03**: the one-terminal invariant,
  the ReactAgent-never-emits-`Cancelled` defect, and the dead `Err`
  arm in `drive_chat_inner`. A-CHAT-01 already classified those; this
  task inherits them as the upstream contract and only inspects their
  observability-side effects (cancelled chat → no `chat_cancelled`
  webhook).
- **A-TSK-03 / A-TSK-04**: `TaskRuntimeStore` internals, the
  `drive_run_async` / `drive_agent_run` task-run lifecycle, and the
  SubagentRun state machine. This task only confirms that those
  lifecycles emit no webhook events and rely on the `RunStore` for
  diagnostics.
- **F-OPS-01**: framework-layer `JsonlRunStore` redaction, size bound,
  telemetry gating, scheduler shutdown. This task consumes F-OPS-01 as
  the framework contract and only adds the application-layer
  corollaries (ToolExecutionRepository, webhook content, CLI surfaces).
- **A-SRF-04**: cron / channel / headless mode parity. The webhook
  coverage gap on cron failure (A-OBS-01-P2-02) is surfaced here as an
  observability finding; surface-parity classification belongs to
  A-SRF-04.
- **F-SEC-01** (if chartered): the secret-leak findings A-OBS-01-P1-01
  and A-OBS-01-P1-02 should feed any cross-cutting secret-boundary
  review.

## Inputs

Required repository documents read in full:

- Repository root `AGENTS.md` (local-assistant threat model —
  "本地也成立的通用安全(如不把密钥打进日志)"; multi-mode parity rule;
  no-panic / UTF-8 safety; "check whether it already exists before
  adding"; framework-vs-application layering gate).
- `docs/comprehensive-review/REPORTING.md` (finding + validation
  contract, cross-repository boundary gate).
- `docs/comprehensive-review/templates/task-report.md`,
  `templates/validation-report.md`.
- `docs/comprehensive-review/TASKS.md` (A-OBS-01 card and dependency
  list: `A-CHAT-01`, `A-TSK-04`, `F-OPS-01`).

Dependency task reports read:

- `zcode-glm/tasks/A-CHAT-01.md` (complete) — establishes
  `drive_chat` as the single chat-turn lifecycle owner, the
  `WebhookTurnObserver` as a cross-cutting observer living INSIDE
  `drive_chat_inner` (correct layering), and the cancelled-chat
  terminal-mismatch (ReactAgent never emits `Cancelled` → the sink
  sees an `Error`). Load-bearing for V01 (the observer's wiring) and
  V04 (the cancel/cron-success asymmetry).
- `zcode-glm/tasks/F-OPS-01.md` (complete) — establishes the
  framework-layer trace-store secret leak (P1-03), the unbounded
  growth / O(N²) write pattern (P2-01), the dead telemetry module
  (P2-02), and the scheduler's missing shutdown path (P1-01).
  Load-bearing for V03 (the framework redaction contract) and for
  A-OBS-01-P2-01 (the F-OPS-01 handoff claim that RunStore "is not
  wired in the CLI by default" is stale).
- `zcode-glm/tasks/A-TSK-04.md` (complete) — establishes that
  `TaskRuntimeStore` is the application-side authority for TaskRun /
  SubagentRun lifecycle and that `drive_run_async` / `drive_agent_run`
  are the background-task half. Load-bearing for V04's claim that
  background TaskRuns emit no webhook events (no emit sites in
  `tasks/task_runtime/*`).

Historical documents treated as hypotheses:

- `webhook/emitter.rs:7-12` docstring — claims the global singleton
  was removed because `init_global` was never called and `emit_global`
  was a no-op masking "webhook not actually firing". Treated as
  **current** (V01 step 5 confirms zero global emitter state).
- `chat_driver.rs:1-15` module docstring (per A-CHAT-01) — claims
  `drive_chat` is "the single, thin entry for a chat turn across TUI /
  CLI channel / GUI". Treated as **current** for the
  WebhookTurnObserver layering (V01).
- `F-OPS-01.md` Coverage/Uncertainty claim: "the CLI does not
  currently wire a RunStore into the ReactAgent by default (verified:
  `with_run_store` is only set in tests and examples today), so this
  is a latent framework hazard rather than a current EKO disk-growth
  incident". Treated as **regressed** — see A-OBS-01-P2-01 and the
  Historical Claim Status table.

## Layering Decision

This is an **application-layer** task. The diagnostics module, the
webhook emitter, the `WebhookTurnObserver`, the
`ToolExecutionRepository`, the GUI/CLI/TUI diagnostic surfaces, and the
slash commands are all EKO product policy and correctly live in
`echo-agent-app-core` / `src/{cli,tui,tauri}`.

| Classification | Required answer |
|---|---|
| Generic mechanism | The framework supplies the right primitives: `RunStore` trait + `JsonlRunStore` (persistence), `RunEvent` variants (data shape), `redact_secrets` (UTF-8-safe redactor covering ~18 secret categories), `WebhooksConfig` (serde-derived config). All are `pub` and consumed unmodified by the application layer. Correct layering. |
| EKO product policy | (1) `DiagnosticRunSummary` / `RunDiagnostics` shape (EKO-specific rollup over the framework's `Run` records, including prompt-assembly report and protected-context budget warnings). (2) `WebhookEvent` enum (ChatCompleted/ToolCalled/ToolFailed/AgentError/CronTaskCompleted — EKO's chosen event surface). (3) `WebhookTurnObserver` (EKO policy: emit on terminal only; gather LlmUsage into ChatCompleted totals). (4) `ToolExecutionRepository` (EKO-specific per-conversation tool history with manifest+JSONL detail). All correctly in the application layer. |
| Adapter boundary | The diagnostics module is a thin read-side adapter: it loads `Run`s from the framework `RunStore` and projects them into EKO's `RunDiagnostics`. The webhook emitter is a thin write-side adapter: it converts lifecycle facts into HTTP POSTs. The `WebhookTurnObserver` is a thin in-process adapter: it converts `AgentEvent`s into `WebhookEvent`s. None of these own scheduling or state authority beyond the per-turn observer locals. |
| Duplicate search | Searched both repos for: `WebhookEmitter`, `WebhookEvent`, `WebhookEndpoint`, `WebhookPayload`, `WebhookTurnObserver`, `WebhookState`, `ObservabilityState`, `DiagnosticRunSummary`, `RunDiagnostics`, `list_diagnostic_runs`, `load_run_diagnostics`, `format_run_diagnostics`, `TraceInvocationDiagnostic`, `LlmCallDiagnostic`, `prompt_assembly`, `ToolExecutionRepository`, `preview_args`, `with_run_store`, `run_store`, `redact_secrets`, `init_global`, `emit_global`, `global_emitter`. Result: one definition per symbol; no parallel observability/webhook/diagnostics implementation in either repo. The framework's `RunEvent::LlmCall`/`ContextCompression` and the application's `LlmCallDiagnostic`/`CompressionDiagnostic` are deliberately distinct (read-side projection vs persisted event) — not a duplicate. |
| Migration deletion | No deletion proposed by this review. The findings propose adding `redact_secrets` calls at three boundaries (P1-01/P1-02 + CLI corollary) and adding webhook emit sites for failure/cron-failure paths (P2-02); none require removing existing code. |

## Current Path

### Verified diagnostics data flow (V01)

```text
[User prompt or tool args]
   │
   ├─ Framework ReactAgent (echo-agent)
   │    ├─ react/mod.rs:1934     → Run { input: prompt.to_string(), ... }   (verbatim, no redact — F-OPS-01-P1-03)
   │    ├─ react/run/pipeline.rs → RunEvent::ToolCall { args: redact_secrets(args) }   (the ONE redacted field)
   │    ├─ react/run/pipeline.rs → RunEvent::ToolResult { output_preview: chars().take(200) }  (unredacted)
   │    └─ react/run/pipeline.rs → RunEvent::ToolError { message: error.clone() }  (unredacted)
   │
   ↓ with_run_store(Arc::new(JsonlRunStore::new(runs_dir)))   ← infra.rs:374-385
   │  (wired on EVERY agent: primary + pooled, via main.rs:168 / desktop.rs:160 → bootstrap → create_agent_with_diagnostics;
   │    agent_pool.rs:895-897 re-injects shared.run_store on pooled agents.)
   │
   ↓ JsonlRunStore::save appends a full Run as one JSON line per event  (F-OPS-01-P2-01: O(N²) write pattern)
   │
   ↓ stored at ~/.echo-agent/runs/{run_id}.jsonl  (unbounded, unredacted)
   │
   ↓ list_diagnostic_runs / load_run_diagnostics   ← observability/diagnostics.rs:16-63
   │  (read-side projection; NO transformation; Run.input/final_output/error and ToolResult.output_preview
   │   pass through verbatim into RunDiagnostics; DiagnosticRunSummary.input_preview is Run.input.chars().take(80).)
   │
   ↓ Surfaces:
   │  • Tauri IPC list_diagnostic_runs / get_run_diagnostics  → panels.rs:1133-1178 → frontend runDiagnosticsApi.list/.get
   │  • TUI /trace, /runs, /run show, /run export              → tui/events.rs:4110-4138, cli/cmd_impls/observability.rs
   │  • REPL /trace, /runs, /run show, /run export             → cli/cmd_impls/observability.rs (same module)
   │  • Channels /runs, /trace, /run                           → cli/channels.rs:349-367
```

### Verified webhook emit flow (V01)

```text
drive_chat_inner (chat_driver.rs:425)
   │  webhook_observer = WebhookTurnObserver::new(webhook_emitter, model)  [:461-462]
   ↓
   For each envelope event from drive_chat's stream:
   │  webhook_observer.observe(&event.payload)   [:543]
   │    ├─ AgentEvent::LlmUsage { prompt_tokens, completion_tokens } → accumulate into self.input_tokens/output_tokens
   │    ├─ AgentEvent::ToolCall { call_id, name, args }              → store (name, args.to_string().chars().take(240), now)
   │    ├─ AgentEvent::ToolResult { call_id, name }                  → emit ToolCalled { name, args_summary, elapsed_ms }
   │    ├─ AgentEvent::ToolError { call_id, name, error }            → emit ToolFailed { name, error.clone() }
   │    ├─ AgentEvent::Error { message }                             → emit AgentError { error: message.clone() }
   │    └─ AgentEvent::FinalAnswer(_)                                → self.completed = true
   │
   At stream end (chat_driver.rs:563):
   │  webhook_observer.finish()
   │    └─ if self.completed { emit ChatCompleted { model, input_tokens, output_tokens, elapsed_ms } }
   │
   Stream-setup error path (chat_driver.rs:517-536):
   │  emitter.emit(AgentError { error: e.to_string() })   (raw error)
   │
   Per-emit (webhook/emitter.rs:128-176):
   │  tokio::spawn → for each endpoint with non-empty events filter match:
   │    tokio::spawn(deliver(client, url, secret, body))
   │       HMAC-SHA256 signature if secret set, POST body=serde_json::to_vec(&WebhookPayload)
   │       on Err: warn, sleep 2s, retry once, warn again, drop.

Cron path (scheduler/runner.rs:111-127):
   │  launch_cron_run → Ok(run_id)   → emit CronTaskCompleted { task_id, task_name, result_summary }
   │  launch_cron_run → Err(e)       → no emit; warn only.

Background TaskRun path (tasks/task_runtime/executor.rs):
   │  NO emit sites — by grep, zero `WebhookEvent::` references in tasks/.
```

### Verified configuration identity (V02)

| Mode | Emitter construction | AppState override | RunStore source | Prompt-assembly source |
|---|---|---|---|---|
| TUI    | `main.rs:107 WebhookEmitter::from_config(&app_config)` | `modes.rs:58 state.webhook.emitter = webhook_emitter` | `infra.rs:379` | `runtime.rs:385 with_prompt_assembly` |
| REPL   | shared with TUI | shared with TUI | `infra.rs:379` | `runtime.rs:385` |
| Tauri  | `desktop.rs:135 WebhookEmitter::from_config(&app_config)` | `desktop.rs:196 state_inner.webhook.emitter = webhook_emitter` | `infra.rs:379` | `desktop.rs:194 with_prompt_assembly` |
| Channels | shared with TUI | shared with TUI (via `start_headless_services`) | `infra.rs:379` | not surfaced (no `/trace` IM command) |

All four entry points reach `AgentRuntime::bootstrap` (headless via
`main.rs:168`, GUI via `desktop.rs:160`), which calls
`infra::create_agent_with_diagnostics` once per process; that function
attaches the `JsonlRunStore` to the agent at `infra.rs:374-385`. Pooled
agents are re-injected with the same shared run_store handle
(`agent_pool.rs:895-897`). Result: one emitter, one run_store, one
prompt-assembly report per process — uniform across modes.

Live reload: `config_watcher::handle_config_change`
(`config_watcher.rs:275-277`) calls `emitter.reload_from_config(&new_config)`
on every config change. The watcher is spawned by both headless
(`main.rs`) and GUI (`desktop.rs:166-171`).

### Verified redaction coverage (V03)

See the V03-01 field-by-field redaction matrix. The application layer
persists raw content at three boundaries and emits raw content at
three boundaries; none of them invoke `redact_secrets`. The framework
redactor is `pub` and UTF-8-safe but unused outside the framework's
`RunEvent::new_tool_call` and a handful of framework sites documented
in F-OPS-01.

### Verified failure / retry semantics (V04)

See the V04-01 misleading-success matrix. The RunStore captures full
lifecycle facts on every path; the webhook channel only emits on
foreground-chat and cron-success paths. The webhook delivery retry
policy is one fixed 2-second retry, then drop with `tracing::warn!`.

## Findings

### A-OBS-01-P1-01: `ToolExecutionRepository` persists tool args / output / failure to disk in plaintext

- Priority: P1
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/tool_execution.rs:217` —
    `args_preview: preview_args(args)` where `preview_args` (line
    579-592) is `serde_json::to_string(args)` + UTF-8-safe truncation;
    no `redact_secrets`.
  - `echo-agent-cli/echo-agent-app-core/src/tool_execution.rs:226` —
    `args_full: args.clone()` writes the full args JSON into the
    StoredManifest.
  - `echo-agent-cli/echo-agent-app-core/src/tool_execution.rs:232` —
    `write_json_atomic(&location.manifest, &manifest)` persists the
    manifest to `~/.echo-agent/tool-executions/{scope}/details/{detail_ref}.json`.
  - `echo-agent-cli/echo-agent-app-core/src/tool_execution.rs:260-291`
    (`append_output`) writes raw `chunk` bytes to `{detail_ref}.jsonl`.
  - `echo-agent-cli/echo-agent-app-core/src/tool_execution.rs:293-358`
    (`finish`) persists `output` (full tool result) and `failure` (a
    structured `ToolFailure`) into the manifest.
  - `echo-agent-cli/src/tauri/commands/chat.rs:1193-1330` —
    `TauriChatSink::handle_tool_event` calls
    `self.tool_executions.start/append_output/finish` on every tool
    event, BEFORE rendering, unconditionally for GUI turns.
  - The same `AgentEvent::ToolCall` payload is also routed through
    `RunEvent::new_tool_call` (framework), where `redact_secrets` IS
    applied to `args` (echo-agent/src/trace/mod.rs:425-444). The
    application layer bypasses that redaction.
- Reachability: every GUI chat turn that invokes a tool. `send_chat`
  (chat.rs:625) constructs `TauriChatSink { tool_executions: ... }`
  from `state.app_state` and passes it as `ChatResources.sink`;
  `drive_chat` (chat_driver.rs:544) forwards every
  `AgentEvent::ToolCall/ToolStream/ToolResult/ToolError` to
  `sink.on_event`, which calls `handle_tool_event`, which calls
  Repository::start/append_output/finish.
- Expected invariant: AGENTS.md local-assistant threat model still
  requires "本地也成立的通用安全(如不把密钥打进日志)". The framework's
  own `redact_secrets` helper exists and is used for `RunEvent::ToolCall.args`;
  the application-layer parallel persistence of the same data must
  apply the same redaction or it is a secret-leak.
- Observed behavior: tool args (potentially `{"api_key": "sk-..."}`,
  `{"token": "ghp_..."}`, `{"connection_string": "..."}`) and tool
  output (potentially a `.env` file or stack trace containing secrets)
  are written verbatim to `~/.echo-agent/tool-executions/.../manifest.json`
  and `.../details/{detail_ref}.jsonl`. A-CHAT-01-P2-01 already noted
  that this repository is GUI-only (TUI/CLI/channels do not persist
  tool history); this finding is about the content of the persisted
  records, not which mode persists them.
- Impact: secret material in tool args/output is written to local disk
  in plaintext, outside the user's awareness. The same secret that the
  framework redacts in `~/.echo-agent/runs/*.jsonl` is left unredacted
  in `~/.echo-agent/tool-executions/.../manifest.json`. A user who
  inspects or backs up their `~/.echo-agent` directory (a reasonable
  thing to do) ships the secrets out inadvertently.
- Root cause: `ToolExecutionRepository` was added as a GUI-side
  per-conversation detail store; it was not plumbed through
  `redact_secrets` because the framework redaction was scoped to
  `RunEvent::ToolCall.args` only (F-OPS-01-P1-03).
- Direction: apply `redact_secrets` to (a) `args_preview`/`args_full`
  before writing the manifest, (b) the output chunks before appending
  to JSONL, (c) the `failure` struct's string fields before writing
  the manifest. The simplest form is a small helper on
  `StoredManifest`/`ActiveExecution` that runs the redactor over every
  string field just before persistence. Use the framework's
  `echo_agent::security::redact_secrets` (already `pub`,
  UTF-8-safe, ~18 patterns).
- Regression validation:
  - Build a `ToolExecutionRepository` in a temp dir; call `start`
    with `args = json!({"api_key": "sk-<40 chars>"})`; reload the
    manifest from disk; assert `"sk-"` substring is absent and
    `[REDACTED:` is present.
  - Same for an `append_output` chunk and a `finish` output string
    containing a synthetic GitHub PAT.
- Validation reports: [V03-01](../validations/A-OBS-01/V03-01.md).

### A-OBS-01-P1-02: `WebhookTurnObserver` ships raw tool args and error messages to external HTTP endpoints

- Priority: P1
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:124` —
    `let args_summary = args.to_string().chars().take(240).collect::<String>();`
    inside `observe(AgentEvent::ToolCall)`. No `redact_secrets`.
  - `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:135-139` —
    `emitter.emit(WebhookEvent::ToolCalled { name, args_summary, elapsed_ms })`
    ships the args_summary over HTTP.
  - `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:148-152` —
    `emitter.emit(WebhookEvent::ToolFailed { name, error: error.clone() })`
    ships the raw `AgentEvent::ToolError.error` string.
  - `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:154-157,
    :521-525, :554-558` — three sites that emit
    `WebhookEvent::AgentError { error: <raw string> }`.
  - `echo-agent-cli/echo-agent-app-core/src/webhook/emitter.rs:180-209`
    (`deliver`) POSTs `body = serde_json::to_vec(&WebhookPayload)` to
    `endpoint.url`. The URL is whatever the user configured — including
    third-party services like Discord/Slack/webhook.site for debugging.
- Reachability: every chat turn (TUI / REPL / Tauri / channels) that
  has at least one webhook endpoint configured AND triggers a ToolCall,
  ToolError, or stream Error event. The observer is constructed
  unconditionally inside `drive_chat_inner`
  (`chat_driver.rs:461-462`); `observe` early-returns when
  `emitter.is_none()`, but as soon as the user adds any endpoint, the
  raw content ships.
- Expected invariant: AGENTS.md "不把密钥打进日志". Webhook endpoints
  are explicitly user-configured HTTP receivers; shipping raw tool
  args / errors to them is the same category as writing secrets to
  logs, with the additional hazard that the data leaves the machine.
- Observed behavior: if a user invokes a tool with
  `{"api_key": "sk-..."}` while a webhook is configured, the args
  (truncated to 240 chars — enough for most short keys) leave the
  machine in the `chat_called` (actually `tool_called`) webhook POST
  body. If a tool error includes a stack trace or echoed env var, the
  raw error string is shipped in `tool_failed` / `agent_error`. The
  framework's `RunEvent::ToolCall.args` redaction does not help here
  — the application-layer observer consumes the same `AgentEvent`
  before any framework persistence.
- Impact: secret material in tool args/error messages is shipped over
  HTTP to user-configured URLs. The HMAC signature protects integrity,
  not confidentiality — the body is plain JSON over HTTPS (if the URL
  is https) or even over HTTP (the URL scheme is not validated;
  `WebhookEntryConfig.url` accepts any string).
- Root cause: `WebhookTurnObserver` was written to mirror
  `AgentEvent::ToolCall/ToolError` payloads to webhook consumers; the
  redaction gap mirrors the framework's gap in `RunEvent::ToolCall`'s
  sibling fields (F-OPS-01-P1-03).
- Direction: in `WebhookTurnObserver::observe`, run
  `echo_agent::security::redact_secrets` over `args_summary` and over
  the `error`/`message` clones before constructing each `WebhookEvent`.
  Three small call-site additions; no API change. Optionally also
  validate `WebhookEntryConfig.url` is `https://` (or `http://localhost`)
  at config-load time, mirroring the local-assistant positioning note
  in AGENTS.md ("URL 用了明文 http" is one of the explicit "obviously
  wrong input" cases worth a lightweight check).
- Regression validation:
  - Stand up a mock HTTP server (e.g. `wiremock` or a `tokio::net::TcpListener`
    that records the request body); configure a `WebhookEmitter` with
    that endpoint; drive a synthetic `AgentEvent::ToolCall { args:
    json!({"api_key": "sk-<40 chars>"}) }` through
    `WebhookTurnObserver::observe` then `finish`; assert the recorded
    body has no `"sk-"` substring.
  - Same shape for `AgentEvent::ToolError { error: "<secret>" }` and
    `AgentEvent::Error { message: "<secret>" }`.
- Validation reports: [V03-01](../validations/A-OBS-01/V03-01.md).

### A-OBS-01-P2-01: F-OPS-01 handoff "RunStore not wired in CLI by default" is stale — F-OPS-01-P1-03 and P2-01 are LIVE defects

- Priority: P2
- Confidence: high
- Layer: application (the wiring site) + framework (the defect)
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/infra.rs:374-385`:
    ```rust
    // Initialize JSONL run store for trace persistence (before build)
    {
        let run_dir = echo_agent::paths::user_data_path("runs");
        match JsonlRunStore::new(&run_dir) {
            Ok(store) => {
                builder = builder.with_run_store(Arc::new(store));
            }
            Err(e) => {
                tracing::warn!("Failed to initialize run store: {e}");
            }
        }
    }
    ```
    inside `create_agent_with_diagnostics` (the live production agent
    builder).
  - `echo-agent-cli/echo-agent-app-core/src/runtime.rs:103` —
    `AgentRuntime::bootstrap` calls
    `infra::create_agent_with_diagnostics(&params, app_config)`.
  - `echo-agent-cli/src/main.rs:168` — headless entry calls
    `AgentRuntime::bootstrap`.
  - `echo-agent-cli/src/tauri/desktop.rs:160` — GUI entry calls
    `AgentRuntime::bootstrap`.
  - `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:895-897` —
    pooled agents are re-injected with the shared run_store:
    `if let Some(ref rs) = self.shared.run_store { agent.set_run_store(rs.clone()); }`.
  - F-OPS-01.md Coverage And Uncertainty states: "the CLI does not
    currently wire a RunStore into the ReactAgent by default (verified:
    `with_run_store` is only set in tests and examples today), so this
    is a latent framework hazard rather than a current EKO disk-growth
    incident". This claim is incorrect as of `b3b2e81`.
- Reachability: every chat / cron / background-task run that goes
  through the primary agent or a pooled agent. The `JsonlRunStore`
  writes to `~/.echo-agent/runs/{run_id}.jsonl` on every
  `start_trace_run` and every `append_event` / `save`.
- Expected invariant: F-OPS-01-P1-03 (secret leakage into trace) and
  F-OPS-01-P2-01 (unbounded growth / O(N²) write pattern) are
  classified as "latent" only if the RunStore is not actually attached
  in production. Once attached, they become LIVE defects.
- Observed behavior: the RunStore IS attached in production. Every
  chat turn writes at least one `Run` record to disk on
  `start_trace_run` and again on `finalize_run`. Mid-run
  `append_event` calls rewrite the entire `Run` as a new JSON line
  (`trace/mod.rs:793-801`). The `Run.input` / `final_output` / `error`
  / `ToolResult.output_preview` / `ToolError.message` fields are
  unredacted (F-OPS-01-P1-03). The file has no size bound
  (F-OPS-01-P2-01).
- Impact: F-OPS-01-P1-03 and P2-01 must be re-prioritized from
  "latent framework hazard" to "live production defect". Every EKO
  user with the default config has a `~/.echo-agent/runs/` directory
  that grows without bound on every chat turn, with plaintext prompts
  and tool output (including any secrets the user pasted).
- Root cause: the wiring was added in `infra.rs:374-385` after
  F-OPS-01's review, or F-OPS-01's grep missed it. Either way, the
  handoff conclusion is stale.
- Direction: this finding is a re-classification, not a fix. The fixes
  remain F-OPS-01's: apply `redact_secrets` at the `JsonlRunStore::save`
  boundary and add a `max_file_bytes` / `max_runs` eviction policy.
  This task only flags that the priority must be P1 (live) rather than
  P2 (latent).
- Regression validation: same as F-OPS-01-P1-03 and P2-01.
- Validation reports: [V01-01](../validations/A-OBS-01/V01-01.md).

### A-OBS-01-P2-02: Webhook coverage has misleading-success gaps on cron failure and on the entire background TaskRun lifecycle

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/scheduler/runner.rs:111-127` —
    ```rust
    match result {
        Ok(run_id) => {
            if let Some(emitter) = webhook_emitter.as_ref() {
                emitter.emit(WebhookEvent::CronTaskCompleted { ... });
            }
            Ok(...)
        }
        Err(e) => Err(ReactError::Other(format!("cron run failed: {e}")))
    }
    ```
    The Err arm emits nothing.
  - `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:3918-3932` —
    `launch_cron_run` converts `TaskRunStatus::Failed` / `Cancelled` /
    `Paused` into `Err(ExecError::Other(...))`, which the FireFn then
    drops without emitting.
  - `grep -rn "WebhookEvent::" echo-agent-cli/echo-agent-app-core/src/tasks --include="*.rs"`
    returns 0 hits — the entire task-runtime executor
    (`drive_run_async` / `drive_agent_run` / SubagentRun transitions)
    emits no webhook events.
  - `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:163-176` —
    `WebhookTurnObserver::finish` emits `ChatCompleted` only when
    `self.completed` (i.e. `AgentEvent::FinalAnswer` was observed).
    There is no `ChatCancelled` or `ChatFailed` event variant
    (`webhook/events.rs:9-33` has only the five variants enumerated
    in V01).
- Reachability: every cron run that fails or is cancelled; every
  background TaskRun lifecycle event (start, plan-task-transition,
  SubagentRun terminal); every cancelled or mid-stream-dropped
  foreground chat turn that does NOT reach `FinalAnswer`.
- Expected invariant: TASKS.md V04 — "Is success reported correctly?".
  A webhook channel that emits on success and is silent on failure is
  a misleading-success channel.
- Observed behavior: see the V04-01 misleading-success matrix. An
  operator who configures a webhook to monitor "is my agent working"
  sees `chat_completed`, `tool_called`, `tool_failed`, `agent_error`,
  and `cron_task_completed`. The operator NEVER sees: cron task
  failed, cron task cancelled, background TaskRun completed, background
  TaskRun failed, background SubagentRun completed, foreground chat
  cancelled. The RunStore captures all of these (visible via
  `/trace`); the webhook channel does not.
- Impact: monitoring and incident-response setups built on webhooks
  are blind to the most operationally-important events (failures).
  The asymmetry between the complete PULL channel (diagnostics) and
  the partial PUSH channel (webhooks) is itself a design smell: the
  data exists, it just is not pushed.
- Root cause: webhook coverage was added incrementally — chat turns
  first (WebhookTurnObserver), cron success second — without a
  lifecycle-event inventory to drive completeness. Background TaskRuns
  predate webhooks and were never wired in.
- Direction: add three `WebhookEvent` variants
  (`CronTaskFailed { task_id, task_name, error }`,
  `TaskRunCompleted { run_id, goal, status }`,
  `TaskRunFailed { run_id, goal, error }`) and emit them at:
  (a) `scheduler/runner.rs:124-126` Err arm (CronTaskFailed);
  (b) the TaskRuntime executor's terminal transitions
  (`executor.rs:1632-1663` already calls
  `run_store.transition_run(run_id, status)` — add a webhook emit
  alongside, mirroring how `WebhookTurnObserver` mirrors chat events);
  (c) the `finalize_task_mode_run` boundary in `chat_driver.rs` so a
  foreground Task-mode chat emits a single TaskRun terminal event
  when the run completes (success or failure).
- Regression validation:
  - Mock-FireN harness that drives `launch_cron_run` to a `Failed`
    TaskRun status; assert a `cron_task_failed` webhook event is
    emitted to a test endpoint (no `cron_task_completed`).
  - Harness that drives a background `TaskRun` to `Completed` and
    `Failed`; assert the corresponding webhook events fire.
- Validation reports: [V04-01](../validations/A-OBS-01/V04-01.md).

### A-OBS-01-P3-01: Webhook module has zero unit tests

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `find echo-agent-cli/echo-agent-app-core/src/webhook -name "*.rs" -exec grep -l "#\[test\]\|#\[tokio::test\]" {} \;`
    returns nothing.
  - `grep -rn "mod tests" echo-agent-cli/echo-agent-app-core/src/webhook/`
    returns nothing.
  - The emitter's retry path, the HMAC signature, the event-subscription
    filter (`endpoint.events`), and `reload_from_config` are all
    untested. `WebhookTurnObserver::observe`'s variant dispatch is
    untested.
- Reachability: every webhook emission in production.
- Expected invariant: a module that ships data over HTTP and signs it
  with HMAC should have at least one round-trip test and one
  redaction test (once A-OBS-01-P1-02 is fixed).
- Observed behavior: zero tests; the module relies entirely on the
  type system and on `WebhookTurnObserver`'s ad-hoc correctness.
- Impact: regressions in delivery, retry, signature, filtering, or
  redaction would not be caught by CI. The A-OBS-01-P1-02 fix
  requires a regression test that does not exist today.
- Root cause: the module was added without a test plan; tests were
  never backfilled.
- Direction: add a `#[cfg(test)] mod tests` in `emitter.rs` covering:
  (a) `reload_from_config` replaces the endpoint set atomically;
  (b) the `endpoint.events` filter skips non-matching events;
  (c) `deliver` produces a correct `X-Webhook-Signature` header for a
  known secret + body (HMAC test vector); (d) emit-then-drop-on-empty-
  endpoints does not spawn. Use a `tokio::net::TcpListener` mock
  server for the round-trip.
- Regression validation: the four tests above are themselves the guard.
- Validation reports: [V04-01](../validations/A-OBS-01/V04-01.md).

### A-OBS-01-P3-02: `aggregate_status` priority orders Running above Failed — partial failures are hidden behind "running"

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/echo-agent-app-core/src/observability/diagnostics.rs:174-197`:
    ```rust
    fn aggregate_status(traces: &[RunSummary]) -> String {
        if traces.iter().any(|t| t.status == RunStatus::Running) {
            return "running".to_string();
        }
        if traces.iter().any(|t| t.status == RunStatus::Failed) {
            return "failed".to_string();
        }
        if traces.iter().any(|t| t.status == RunStatus::Cancelled) {
            return "cancelled".to_string();
        }
        if traces.iter().all(|t| t.status == RunStatus::Completed) {
            return "completed".to_string();
        }
        "pending".to_string()
    }
    ```
  - A multi-trace group where one trace is `Running` and another is
    `Failed` reports `running`, hiding the failure.
- Reachability: every `list_diagnostic_runs` query against a group
  containing mixed statuses. The most common case is a parent run with
  multiple subagent traces, one of which crashed.
- Expected invariant: a status aggregator should surface the most
  severe status, not the busiest. Standard severity order is
  `Failed > Cancelled > Running > Pending > Completed`.
- Observed behavior: priority is `Running > Failed > Cancelled >
  Completed > Pending`. A long-running group with one dead trace
  perpetually shows "running" until every trace stops, then jumps to
  "failed".
- Impact: low (cosmetic; the underlying `Run.status` is preserved per
  trace and surfaced via `RunDiagnostics.traces`). But the
  `DiagnosticRunSummary.status` field is the headline shown in the
  GUI panel and the `/runs` CLI listing, so a casual operator can be
  misled.
- Root cause: the priority was written to default to "still going"
  rather than to "something broke".
- Direction: reorder to
  `Failed > Cancelled > Running > Completed > Pending`. One-line
  predicate swap; existing test
  `parent_run_projection_uses_one_durable_diagnostic_contract` covers
  the happy path and will continue to pass (its traces are all
  `Completed`).
- Regression validation: extend the diagnostics tests with a group
  containing one `Running` + one `Failed` trace; assert the aggregate
  is `"failed"`. Add a similar test for `Cancelled` + `Running`.
- Validation reports: [V01-01](../validations/A-OBS-01/V01-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Diagnostics + webhook wiring to live lifecycle (definition + reachability + emitter identity + no globals) | yes | passed | [V01-01](../validations/A-OBS-01/V01-01.md) |
| V02 | Configuration identity (AppConfig -> emitter / run_store / prompt_assembly uniform across modes; live reload) | yes | passed | [V02-01](../validations/A-OBS-01/V02-01.md) |
| V03 | Secret / content redaction at application-layer boundaries (ToolExecutionRepository, webhook, CLI slash) | yes | passed (with findings) | [V03-01](../validations/A-OBS-01/V03-01.md) |
| V04 | Failure / retry / misleading-success scenarios + targeted cargo test execution | yes | passed (with findings) | [V04-01](../validations/A-OBS-01/V04-01.md) |
| V05 | Historical-document drift | conditional | n/a — see Historical Claim Status table; the F-OPS-01 handoff drift is reclassified inline. |

Executed cargo commands (all exit 0):

```text
cd echo-agent-cli
cargo test -p echo-agent-app-core --lib observability::    (3 passed)
cargo test -p echo-agent-app-core --lib chat_driver::      (9 passed)
cargo test -p echo-agent-app-core --lib tool_execution::   (5 passed)
cd echo-agent
cargo test -p echo_agent --lib trace::                     (21 passed)
```

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `webhook/emitter.rs:7-12` — "global singleton removed; `init_global` was never called, `emit_global` was a no-op masking the silent failure" | current | V01 step 5 confirms zero global state; the per-process `AppState.webhook.emitter` is the single emitter, overridden by every production caller. |
| `chat_driver.rs` WebhookTurnObserver is a cross-cutting observer inside `drive_chat_inner` (correct layering) | current | V01 confirms `webhook_observer.observe` is called once per envelope event at `chat_driver.rs:543`, before the sink; identical for every chat surface. |
| `F-OPS-01.md` Coverage — "the CLI does not currently wire a RunStore into the ReactAgent by default... a latent framework hazard rather than a current EKO disk-growth incident" | **regressed / stale** | V01 step 2 and A-OBS-01-P2-01 confirm `infra.rs:374-385` attaches `JsonlRunStore` on the production `create_agent_with_diagnostics` path, reached by `main.rs:168` + `desktop.rs:160` via `AgentRuntime::bootstrap`. Pooled agents are re-injected at `agent_pool.rs:895-897`. F-OPS-01-P1-03 (trace secret leak) and F-OPS-01-P2-01 (unbounded growth / O(N²) writes) are LIVE defects today. |
| `F-OPS-01.md` P2-02 — "`Metrics::record_*` are defined but never invoked; telemetry is dead-on-arrival" | current (load-bearing) | V04 step 1 confirms no application-layer metrics call site was added; the framework's `telemetry` feature is still gated but inert. A-OBS-01 inherits this as the upstream contract and adds no new metrics wiring. |
| `A-CHAT-01.md` Handoff — "ReactAgent never emits `AgentEvent::Cancelled`; cancel ends the stream without a terminal" | current (load-bearing) | V04 confirms the webhook observer sees an `AgentEvent::Error` (not `Cancelled`) on the cancelled stream-end, emits `WebhookEvent::AgentError`, and `WebhookTurnObserver::finish` returns silently (no `ChatCancelled` event exists). |
| `A-CHAT-01.md` Finding A-CHAT-01-P2-01 — "`TauriChatSink` owns tool-execution persistence authority" | current (load-bearing) | V03 step 2 confirms `TauriChatSink::handle_tool_event` is the unique writer to `ToolExecutionRepository`; this task adds the corollary that the persisted content is unredacted (A-OBS-01-P1-01). |

## Coverage And Uncertainty

Inspected in full: `observability/` (3 files, 612+ lines),
`webhook/` (3 files, 209+ lines), `chat_driver.rs:83-176` (observer) +
`:425-569` (drive_chat_inner), `state.rs:398-406, 469, 569-595`,
`tool_execution.rs:185-360, 579-592`, `infra.rs:184-385`,
`runtime.rs:60-200`, `config_watcher.rs:200-278`,
`scheduler/runner.rs:42-128`, `main.rs:100-310`,
`tauri/desktop.rs:125-220`, `cli/modes.rs:28-130`,
`cli/channels.rs:240-270`, `cli/repl.rs:495-545`,
`tui/events.rs:1294-1435, 3698-4138`,
`cli/cmd_impls/observability.rs` (full),
`tauri/commands/panels.rs:1129-1178`,
`tauri/commands/chat.rs:1148-1572`,
`web-frontend/src/api/endpoints.ts:630-741`, and
`web-frontend/src/components/observability/ObservabilityPanel.tsx:80-140`.
Framework cross-references: `trace/mod.rs:38-97, 301-560, 601-801`,
`security.rs:33-128`, `config.rs:640-661`,
`echo-core/src/agent/mod.rs:143-310`.

Not inspected (out of scope or deferred):

- The full TUI `TuiChatSink::on_event` mapping (`tui/events.rs:2032-2224`)
  was inspected at the structural level (it is a pure renderer per
  A-CHAT-01-P2-01's analysis). This task only re-confirmed it does not
  persist anything beyond what `WebhookTurnObserver` already observes.
- The frontend rendering of `RunDiagnostics`
  (`ObservabilityPanel.tsx:140-end`) was inspected at the data-flow
  level only; React rendering correctness belongs to A-FE-03.
- The framework's `Metrics::record_*` call sites were not re-audited
  (F-OPS-01-P2-02 owns this; A-OBS-01 only inherits the "telemetry is
  inert" conclusion).
- The CLI's `/prompt-diagnostics` rendering
  (`cmd_impls/observability.rs:56-103`) was inspected at the wiring
  level; its estimated-token logic is the framework's `Context`'s
  responsibility (out of scope).
- The `agent_event_to_chat_event` catch-all arm at
  `chat.rs:1566-1570` (`other => Notice { format!("{other:?}") }`)
  is dead today (the match is exhaustive over the current
  `AgentEvent` variants). If the framework ever marks `AgentEvent`
  `#[non_exhaustive]` or adds a variant without updating this match,
  the catch-all would silently ship Debug-formatted payloads to the
  GUI — flagged here, not promoted to a finding because the
  exhaustive match currently makes it unreachable.

Environmental constraints:

- `cargo test` runs used the existing incremental cache under
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/target`
  and `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/target`.
  Worktree clean at `b3b2e81` (CLI) and `9b0e0fa` (framework). No
  feature-matrix re-run was needed: no `#[cfg(...)]` gates exist in
  `observability/`, `webhook/`, or `tool_execution.rs` outside
  `#[cfg(test)]` modules.

Uncertain claims:

- Whether EKO users in the field actually configure webhook endpoints
  (which would make A-OBS-01-P1-02 a fired leak rather than a latent
  one) depends on user behavior. The leak path is deterministic; whether
  it fires today is usage-dependent.
- Whether any operator has built monitoring on the existing
  `chat_completed` / `cron_task_completed` events (which would make the
  A-OBS-01-P2-02 gap a regression rather than a missing feature). No
  evidence either way in the repo.
- The framework's `RunEvent::LlmCall` records `cache_fingerprint` /
  `context_breakdown` etc. but no per-call error string; if a future
  framework change adds an error field to `RunEvent::LlmCall`, the
  application-layer diagnostics would forward it unredacted. Not a
  current defect; flagged for the persistence-boundary redaction fix
  recommended by F-OPS-01-P1-03.

## Handoff

Conclusions downstream tasks may rely on:

1. **The application layer does NOT redact anything.** All three
   application-layer persistence / emission boundaries
   (`ToolExecutionRepository`, `WebhookEmitter`, CLI `/run` slash
   commands) ship raw content. The framework `redact_secrets` helper
   is `pub`, UTF-8-safe, and covers ~18 secret categories — it is
   usable as-is at every boundary listed in A-OBS-01-P1-01 / P1-02.
2. **The RunStore IS wired in production.** F-OPS-01's "latent
   hazard" framing is stale (A-OBS-01-P2-01). Any downstream task
   that consumes F-OPS-01-P1-03 / P2-01 (notably F-SEC-01 if
   chartered, and A-STATE-01 for persisted-data hygiene) must treat
   them as LIVE P1 defects.
3. **The webhook channel is a partial PUSH surface.** It emits on
   foreground-chat and cron-success paths only (A-OBS-01-P2-02).
   Cron failures and the entire background TaskRun lifecycle are
   silent. Any downstream task reasoning about operational
   observability (A-BOOT-01 shutdown sequencing, A-SRF-04 mode parity)
   should consume this asymmetry rather than re-derive it.
4. **The diagnostics module is correctly layered.** It is a read-side
   projection over the framework `RunStore`, with one canonical
   AppConfig-sourced prompt-assembly report. Configuration identity
   is uniform across TUI / REPL / Tauri / channels.
5. **The global-emitter removal is real.** No globals remain; the
   per-process emitter is shared via `AppState.webhook.emitter`
   (overridden by every production entry). The historical claim in
   `emitter.rs:7-12` is current.

Reports they must read:

- This report (A-OBS-01) for the application-layer leak / misleading-success
  conclusions.
- `tasks/A-CHAT-01.md` for the `drive_chat` lifecycle / sink
  responsibility split / one-terminal invariant that this task
  inherits.
- `tasks/F-OPS-01.md` for the framework-layer trace-store / telemetry
  / scheduler findings that this task extends (especially P1-03, P2-01,
  P2-02).
- `tasks/A-TSK-04.md` for the TaskRuntime / SubagentRun lifecycle
  boundaries that the webhook-coverage gap (A-OBS-01-P2-02) refers to.

Conditions that make this report stale:

- Any change to `infra.rs:374-385` (RunStore wiring) invalidates
  A-OBS-01-P2-01's "live not latent" reclassification.
- Any change to `tool_execution.rs::start/append_output/finish` or
  `chat_driver.rs::WebhookTurnObserver::observe` that applies
  `redact_secrets` invalidates A-OBS-01-P1-01 / P1-02.
- Any new `WebhookEvent::` variant or new emit site in
  `tasks/task_runtime/*` invalidates the V04 misleading-success
  matrix and likely resolves A-OBS-01-P2-02.
- Any new chat-streaming entry point that bypasses `drive_chat`
  invalidates the V01 emit-flow map.

Follow-up task IDs (no fixes implemented in this review):

- An **application-layer redaction** task — resolve A-OBS-01-P1-01 /
  P1-02 by applying `echo_agent::security::redact_secrets` at the
  three boundaries listed. Touches `tool_execution.rs`,
  `chat_driver.rs::WebhookTurnObserver`, and
  `cli/cmd_impls/observability.rs`. Should land alongside the
  F-OPS-01-P1-03 fix (trace-store redaction at `JsonlRunStore::save`)
  so all persistence boundaries are redacted consistently.
- A **webhook coverage completeness** task — resolve A-OBS-01-P2-02 by
  adding `CronTaskFailed` / `TaskRunCompleted` / `TaskRunFailed`
  variants and emit sites. Touches `webhook/events.rs`,
  `scheduler/runner.rs`, `tasks/task_runtime/executor.rs`, and
  `chat_driver.rs::finalize_task_mode_run`.
- A **webhook test backfill** task — resolve A-OBS-01-P3-01 by adding
  the four-test `#[cfg(test)] mod tests` to `webhook/emitter.rs`,
  including the regression tests required by A-OBS-01-P1-02.
- **F-SEC-01** (if chartered) — should consume A-OBS-01-P1-01 / P1-02
  and A-OBS-01-P2-01 when reasoning about the local-data secret
  boundary; the same redactor that protects tool args in the framework
  RunStore must protect the application-layer persistence and webhook
  channels.
