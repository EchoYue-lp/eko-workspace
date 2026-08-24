# A-OBS-01: Diagnostics, webhooks, and operational visibility

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean at review time

> Correction (2026-08-12, appended per REPORTING.md): after this review's
> command runs completed, an external process regenerated
> `web-frontend/src/generated/*.ts` (79 files, ts-rs output, mtime 16:48:48).
> This churn was not caused by this review — the ts-rs export is gated behind
> the non-default `__ts_rs` feature (echo-agent-app-core/Cargo.toml:56,
> workspace/mod.rs:236) which none of the V04 commands enabled — and does not
> affect any finding anchored in `echo-agent-app-core` or `src/`.

## Question

Are diagnostics, run context, webhook events, and logs wired to live lifecycle
facts without globals, secret leakage, or misleading success?

**Answer: partially.** Configuration identity holds — one non-global
`WebhookEmitter` per process, shared by all four interactive surfaces, the
scheduler, and the config watcher; the global emitter was removed; webhook
`ChatCompleted`/`CronTaskCompleted` fire only on real terminal success. But four
lifecycle/security invariants fail: (P1) EKO's `save_trace` terminal writer
records a **paused (resumable) run as `Completed`** and writes no record at all
on executor faults, so the diagnostics surface reports false terminal facts
(misleading success); (P1) the webhook payload carries **raw tool arguments and
raw error text** to external endpoints with no secret redaction — an outbound
channel for the same data the framework redacts in trace events; (P1) user
cancellations surface on the webhook as a **fabricated `agent_error`** (the
envelope-normalized cancel terminal) and the `WebhookEvent` enum has no
cancel/failure chat terminal, so external consumers cannot distinguish cancel
from failure (webhook channel manifestation of A-CHAT-01-P1-01); (P2) cron and
background runs emit no failure webhook, and `save_trace` is a second trace
authority that duplicates every task run into the diagnostics panel,
regressing MASTER-PLAN:713's "framework RunStore 为唯一诊断事实".

## Scope

Primary source paths inspected (deep read):

- `echo-agent-cli/echo-agent-app-core/src/webhook/{emitter,events,mod}.rs`
  (full): endpoint config, emit/deliver, HMAC signing, retry, payload shape.
- `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:83-176`
  (`WebhookTurnObserver`), `:515-568` (drive loop + finish).
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:437-584`
  (terminal mapping) and `:3504-3549` (`save_trace`), `:3895-3928`
  (`launch_cron_run` status gate).
- `echo-agent-cli/echo-agent-app-core/src/observability/{mod,types,diagnostics}.rs`
  (full): durable run diagnostics, grouping, status aggregation.
- `echo-agent-cli/echo-agent-app-core/src/scheduler/runner.rs:47-130`
  (`build_fire_fn` webhook emission).
- Wiring: `echo-agent-cli/src/main.rs:107-110,229-235,255-300,360-430`,
  `src/tauri/desktop.rs:125-200`, `src/tauri/commands/chat.rs:655-720,1141-1417`,
  `src/tauri/commands/panels.rs:1133-1178`, `src/tui/events.rs:4100-4160`,
  `src/cli/modes.rs:32-128`, `src/cli/repl.rs:69-89,520-535`,
  `src/cli/channels.rs:39-57,230-255`, `echo-agent-app-core/src/state.rs:398-437,460-575`,
  `echo-agent-app-core/src/config_watcher.rs:55-65,275-276`.
- GUI observability: `web-frontend/src/components/observability/ObservabilityPanel.tsx`,
  `web-frontend/src/api/endpoints.ts:615-740`.
- Framework cross-references: `echo-agent/src/agent/react/mod.rs:1890-1965`
  (trace run lifecycle), `src/agent/react/run/stream_channel.rs:90-180`
  (trace parent), `src/trace/mod.rs:418-450` (`new_tool_call` redaction),
  `src/security.rs:105-128` (`redact_secrets`), `echo-core/src/agent/mod.rs:190-235`
  (`AgentEvent::ToolCall.args` type), `echo-core/src/agent/event_envelope.rs:134-191`
  (envelope normalization, consumed as F-RCT-03 fact).

## Out Of Scope

- Framework trace/audit/telemetry internals and their redaction gaps →
  F-OPS-01 (P2-01 trace plaintext, P2-04 audit, P3-01 dead RunEvent variants),
  consumed as dependency facts, not re-filed.
- Chat driver lifecycle/Result decoupling and GUI `TurnStatus` mislabeling →
  A-CHAT-01-P1-01 (canonical), re-surfaced here only on the webhook channel
  (P1-03).
- Task-runtime claim/ledger/recovery semantics → A-TSK-01..06 (A-TSK-04
  consumed for `save_trace`'s terminal context).
- Frontend rendering of tool-execution projections → A-FE-01/02.
- Scheduler trigger lifecycle (broken tick) → F-OPS-01-P1-01.
- Feishu channel / inbound `WebhookHumanLoopProvider` (distinct inbound
  concepts).

## Inputs

- Root `AGENTS.md` (local threat model: no secrets in logs/events; UTF-8/panic
  safety; layering gate; no parallel semantics; surface parity), shared
  `README.md`, `REPORTING.md`, `TASKS.md` (A-OBS-01 card), `zcode-ds/README.md`,
  report templates.
- Dependency task reports read (zcode-ds): `A-CHAT-01` (complete; P1-01
  Result/terminal decoupling, P2-01 dead Interrupt variant, WebhookTurnObserver
  finish-only-on-FinalAnswer), `A-TSK-04` (complete; terminal monotonicity,
  pause semantics, `set_task_status` surfaces), `F-OPS-01` (complete; P2-01
  trace plaintext JSONL, P2-04 audit raw args, save_trace handed to A-OBS-01,
  P1-01 scheduler tick dead).
- Historical documents treated as hypotheses: `docs/MASTER-PLAN.md` (lines 379,
  605, 709, 713, 721, 771), `echo-agent-cli/docs/2026-07-28-app-core-full-audit.md`
  (A1), `docs/PROJECT-ANALYSIS.md:244`.

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism (framework, correctly placed) | `WebhooksConfig`/`WebhookEntryConfig` config types (config.rs:642-661); trace `Run`/`RunStore`/`RunEvent` lifecycle (start/finalize, redaction via `new_tool_call`); `redact_secrets` (security.rs:105). All reused as-is. |
| EKO product policy (application, correctly placed) | `WebhookEmitter` + `WebhookEvent` enum + delivery policy (fire-and-forget, HMAC, retry, subscription filter); `WebhookTurnObserver`; `save_trace` terminal records; diagnostics aggregation (`observability/`); the decision to emit webhooks on cron completion. |
| Adapter boundary | All four entry adapters are thin and uniform on the main path (emitter via `ChatResources` → `drive_chat`); scheduler `build_fire_fn` is a thin `FireFn` adapter. **Deviations**: `save_trace` is an application-side second trace writer with no event linkage (P2-01); the observer's webhook payloads bypass the framework's redaction contract (P1-02). |
| Duplicate search (V01-01) | Terms: `WebhookEmitter`, `WebhookEvent`, `WebhookTurnObserver`, `emit`, `init_global`, `emit_global`, `global_emitter`, `has_endpoints`, `list_diagnostic_runs`, `load_run_diagnostics`, `format_run_diagnostics`, `RunDiagnostics`, `DiagnosticRunSummary`, `save_trace`, `start_scoped_trace_run`, `finalize_run`, `RunStore`, `redact`, `webhook`, `worker`. Results: one definition per concept; zero global-emitter remnants; zero redaction in the EKO webhook path; one application-side terminal trace writer (`save_trace`) duplicating the framework writer's authority; zero `worker` terms in touched files. |
| Migration deletion | New targets: `save_trace` + its four call sites (P2-01) once the framework finalize path covers the terminals; the `WebhookEvent::AgentError` arm for cancel-driven terminals (P1-03); the dead emitter construction in `AppState::from_shared` (P3-02). |

## Current Path

Verified call graph (V02-01):

1. Emitter construction: `WebhookEmitter::from_config(&app_config)` — main.rs:107
   (CLI/TUI/channel) and desktop.rs:135 (GUI), each after `apply_env_overrides`.
   `AppState::from_shared` also constructs one (state.rs:469) that every live
   entry point overwrites (desktop.rs:196) — dead construction (P3-02).
2. Injection: one Arc per process shared by GUI chat.rs:668
   (`state.app_state.webhook.emitter`), TUI tui/mod.rs:1954, REPL
   repl.rs:529 (via modes.rs:106), channels.rs:234-251, scheduler
   `new_scheduler_runner` (main.rs:264), and the config watcher (main.rs:235,
   desktop.rs:169) which hot-reloads endpoints into the same instance
   (config_watcher.rs:275-276). No globals (V01-01).
3. Observation: `drive_chat_inner` creates `WebhookTurnObserver`
   (chat_driver.rs:461-462); every envelope item is observed (:543); `finish()`
   (:563) emits `ChatCompleted` only when a `FinalAnswer` was seen (:158,
   :163-175). Task-runtime executor loops (:3119-3130, :3734-3789) and cron
   `build_fire_fn` (scheduler/runner.rs:47-130) have no observer — no
   per-step/failure webhook events on those paths (P2-02).
4. Delivery: `emit` (emitter.rs:128-176) spawns detached tasks; per endpoint:
   event filter → HMAC-SHA256 sign (`X-Webhook-Signature`) → POST with 10s
   timeout → 1 retry after 2s → `tracing::warn` on failure (P3-01).
5. Trace records for task runs: the framework creates a parented `Run`
   (start_scoped_trace_run via stream_channel.rs:100-109, parent = product
   run_id), and EKO's `save_trace` (executor.rs:3504-3549) writes a second,
   parentless, event-less `Run` (`run_id` = product run id, `agent_name`
   "task-runtime") at each terminal except the executor `Err` branch
   (:570-582); the status match (:3521-3526) maps `"paused"` → `Completed`
   (P1-01) and `"failed"` → generic `Some("run failed")` (cause lost).
6. Diagnostics surface: `list_diagnostic_runs` groups by `parent_run_id`
   (diagnostics.rs:21-27) — save_trace records become their own empty groups;
   `aggregate_status` (:174-197) derives the displayed status from
   `RunSummary.status`; TUI `/trace` (events.rs:4115-4145) and GUI commands
   (panels.rs:1134-1178, frontend ObservabilityPanel) render it.

## Findings

### A-OBS-01-P1-01: `save_trace` records a paused (resumable) run as `Completed` and writes no terminal record on executor faults — the diagnostics surface reports false lifecycle facts

- Priority: P1
- Confidence: high (deterministic static chain; all four call sites verified)
- Layer: application (EKO trace writer)
- Evidence: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:562-568`
  (Paused outcome calls `save_trace(..., "paused")`), `:3521-3526` (status match
  fallback `_ => RunStatus::Completed`), `:449/:491/:509` (completed/failed/
  cancelled call sites), `:570-582` (executor `Err` branch — no `save_trace`),
  `:3530-3533` (failed records carry only `Some("run failed")`).
- Reachability: every TaskRuntime run terminal. Pause reaches the Paused
  outcome via `request_pause` (store.rs:598-622, A-TSK-04 handoff); the Err
  branch fires on executor faults (A-TSK-04-P1-02 class). The records are read
  by `list_diagnostic_runs`/`aggregate_status`
  (observability/diagnostics.rs:16-35,174-197) and rendered in TUI `/trace`
  (events.rs:4130-4145) and the GUI panel (ObservabilityPanel.tsx:202-205,
  `run.status`).
- Expected invariant: a resumable paused run is never recorded as terminal
  Completed; every run end (including executor faults) yields a truthful
  terminal trace record carrying the actual cause; trace facts agree with
  `TaskRunStatus` (A-TSK-04 terminal monotonicity).
- Observed behavior: `"paused"` → `RunStatus::Completed` (the run is actually
  Paused and resumable); executor `Err` → no record at all (combined with the
  framework tools-branch finalize gap F-RCT-02-P2-01, the run can be absent or
  stuck `running`); failed runs lose the cause in the trace record.
- Impact: the operational visibility surface (diagnostics panel, `/trace`, any
  RunStore consumer including evolution dashboard) shows paused runs as
  completed and can hide failures — misleading success and missing failure
  facts, exactly the class this task is asked to catch; users resume a run that
  diagnostics declared done.
- Root cause: `save_trace`'s status mapping is a local string match with a
  catch-all `Completed` default written before the Paused outcome existed; the
  fault branch was never wired into the terminal writer; the error cause was
  discarded instead of reused from `store.note` (executor.rs:489).
- Direction: pass a typed terminal outcome (or map `"paused"` to no terminal
  record / the run's durable status), write a Failed record on the `Err`
  branch, and persist the real error text (reuse the `note`/`RunFailed`
  payload already produced at :471-489); add `save_trace` fixtures.
- Regression validation: fixture — paused run → trace record not Completed;
  executor-Err run → one Failed record with the cause; failed run → cause
  present in `run.error`.
- Validation reports: [V02-01](../validations/A-OBS-01/V02-01.md),
  [V03-01](../validations/A-OBS-01/V03-01.md), [V04-02](../validations/A-OBS-01/V04-02.md)

### A-OBS-01-P1-02: Webhook payloads carry raw tool arguments and raw error text to external endpoints — no secret redaction on the outbound channel

- Priority: P1
- Confidence: high (static; zero redaction in the path, verified by grep)
- Layer: application (webhook delivery policy)
- Evidence: `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:124`
  (`let args_summary = args.to_string().chars().take(240).collect::<String>()`
  — raw `AgentEvent::ToolCall.args: serde_json::Value`, echo-core/src/agent/mod.rs:194-202),
  `:148-151` (`ToolFailed.error` raw), `:153-157` (`AgentError.error` raw),
  `:521-525` (setup-failure message raw); delivered verbatim by
  `webhook/emitter.rs:138-149` (`serde_json::to_vec(&payload)` then HTTP POST,
  optionally plain `http://` — no scheme validation). Contrast: the framework
  redacts the same data class in trace events via `new_tool_call`
  (echo-agent/src/trace/mod.rs:425-444, `security::redact_secrets`
  security.rs:105-128); V01-01 confirms zero `redact` usage in the EKO
  webhook/observability path.
- Reachability: any chat turn on any of the four surfaces with tool calls or
  errors, when at least one webhook endpoint is configured; the payload leaves
  the machine over the network.
- Expected invariant: the same secret-redaction posture as the trace path
  (AGENTS.md: "不把密钥打进日志"; framework precedent) applies to outbound
  events; tool arguments/error text containing tokens (bash commands, file
  contents, env values, HTTP bodies in errors) never leave unredacted.
- Observed behavior: up to 240 chars of raw tool args and full error messages
  are serialized into `tool_called`/`tool_failed`/`agent_error` webhook events
  with no redaction pass.
- Impact: an outbound secret-exposure channel whenever a user pastes or tools
  handle secrets — the same defect class as F-OPS-01-P2-01 (local trace) and
  F-OPS-01-P2-04 (audit), but network-facing; contradicts the product's own
  redaction standard.
- Root cause: the observer predates the framework's redaction contract and was
  never audited against it; there is no redaction choke point at the
  webhook/observability boundary.
- Direction: apply `echo_agent::security::redact_secrets` (and bounded
  truncation for error fields) at a single choke point — in
  `WebhookTurnObserver::observe` or inside `WebhookEmitter::emit` before
  serialization — mirroring `new_tool_call`; add redaction fixtures.
- Regression validation: fixture — `ToolCall` args containing a key pattern →
  serialized payload contains no key (same assertion style as the browser
  redaction test, browser/mod.rs:1889-1894); `ToolError` with an embedded token
  → redacted.
- Validation reports: [V01-01](../validations/A-OBS-01/V01-01.md),
  [V03-01](../validations/A-OBS-01/V03-01.md), [V04-02](../validations/A-OBS-01/V04-02.md)

### A-OBS-01-P1-03: Webhook channel reports user cancellations as a fabricated `agent_error` and has no cancel/failure terminal event — external consumers cannot distinguish outcomes

- Priority: P1
- Confidence: high (chain static; the envelope normalization is a verified
  F-RCT-03 fact)
- Layer: application (webhook event contract) / adapter (envelope boundary)
- Evidence: `echo-agent-cli/echo-agent-app-core/src/webhook/events.rs:9-33`
  (`WebhookEvent` has no `chat_cancelled`/`chat_failed` variant);
  `chat_driver.rs:153-157` (every `AgentEvent::Error` payload → `AgentError`
  webhook); `:163-175` (`finish` emits `ChatCompleted` only after `FinalAnswer`);
  envelope normalization `echo-core/src/agent/event_envelope.rs:134-191`
  (every raw `Err` becomes an `Error` payload; terminal-less ends fabricate
  `Error{"agent stream ended without a terminal event"}`); F-RCT-03-P1-01/P1-02
  (cancel ends the stream as NoResponse Err or terminal-less abandon — the
  framework never emits `Cancelled` on the chat path).
- Reachability: every cancelled chat turn (TUI Ctrl+C / GUI cancel) and every
  envelope-normalized error on all four surfaces with an endpoint configured.
- Expected invariant: cancellation and failure are distinguishable from each
  other and from success on every observability channel (AGENTS.md surface
  parity; A-CHAT-01-P1-01 invariant applied to the webhook); a user action is
  never reported as an agent error.
- Observed behavior: on cancel the observer receives the envelope-normalized
  error payload and emits `agent_error` with the fabricated message ("agent
  stream ended without a terminal event" / no-response); no completion or
  cancel event follows; `ChatCompleted` is correctly withheld. External
  consumers receive a phantom error per cancellation and cannot detect failure
  at all on turns that end without `FinalAnswer`.
- Impact: misleading external reporting — webhook-driven dashboards/
  notifications misreport every user cancel as an agent failure and miss real
  failures; the webhook contract cannot represent the product's own lifecycle
  (cancel/failed are first-class run outcomes per A-TSK-04).
- Root cause: `WebhookEvent` was designed with success-only chat terminals
  while the observer forwards the envelope's normalized payload — the
  envelope/Result decoupling (A-CHAT-01-P1-01) propagates onto the external
  channel.
- Direction: add `chat_cancelled`/`chat_failed` variants (or a typed
  `ChatFinished{outcome}`) and emit them from the driver's terminal outcome
  (coordinate with A-CHAT-01-P1-01's typed `TurnOutcome` fix); suppress
  `AgentError` for cancel-driven terminals; align with the F-RCT-03 fixes that
  remove the fabricated message.
- Regression validation: driver-level fixture — cancelled-token turn → exactly
  one `chat_cancelled` webhook event and zero `agent_error`; error-terminal
  turn → one `chat_failed` carrying the cause; success turn → `chat_completed`
  only.
- Validation reports: [V02-01](../validations/A-OBS-01/V02-01.md),
  [V03-01](../validations/A-OBS-01/V03-01.md)

### A-OBS-01-P2-01: `save_trace` is a second trace authority — every task run is duplicated in the diagnostics panel as a parentless event-less record, regressing MASTER-PLAN:713/709

- Priority: P2
- Confidence: high
- Layer: application (adapter boundary deviation)
- Evidence: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:3504-3549`
  (Run with `parent_run_id: None`, `run_id` = product run id, `agent_name`
  "task-runtime", empty `events`, terminal only) vs the framework record
  parented to the run id (`echo-agent/src/agent/react/run/stream_channel.rs:100-109`,
  `src/agent/react/mod.rs:1907-1931`); grouping in
  `observability/diagnostics.rs:21-27` (parentless → own diagnostic group);
  MASTER-PLAN:713 "EKO 以 framework `RunStore` 为唯一诊断事实" and :709
  "TaskRuntime run_id 只作为 parent correlation" (V05-01: regressed).
- Reachability: every task-mode chat turn and every cron/background run — two
  records in the same `JsonlRunStore` (infra.rs:377), both surfaced by the
  diagnostics panel.
- Expected invariant: one diagnostic authority per MASTER-PLAN:713; EKO never
  persists a parallel Run record; trace records agree (status, cause) with the
  product run.
- Observed behavior: each task run appears twice in `list_diagnostic_runs`
  (once as the real parent group, once as an empty standalone entry); the two
  writers can disagree on status (framework record stuck `Running` on the
  tools-branch terminal per F-RCT-02-P2-01 vs EKO `Completed` for paused runs
  per P1-01).
- Impact: diagnostics panel pollution and conflicting lifecycle facts —
  misleading operational visibility; consumers cannot trust the store as a
  single source of truth.
- Root cause: `save_trace` predates the M9 convergence and was kept as an
  application-side terminal convenience instead of routing terminals through
  the framework's `finalize_scoped_trace_run` (or relying on the ExecEvent
  `RunCompleted` terminal already emitted at executor.rs:439-446).
- Direction: remove `save_trace` and its four call sites; drive terminal facts
  through the framework finalize path (which requires the F-RCT-02-P2-01 fix so
  the framework record is actually finalized); keep the store append-only.
- Regression validation: after the change — exactly one trace record per run
  with a truthful status; zero parentless records originating from task runs;
  diagnostics fixture asserting group counts (existing
  `parent_run_projection_uses_one_durable_diagnostic_contract` extended).
- Validation reports: [V02-01](../validations/A-OBS-01/V02-01.md),
  [V05-01](../validations/A-OBS-01/V05-01.md)

### A-OBS-01-P2-02: Cron and background runs emit no failure webhook — failed cron executions are invisible to webhook consumers

- Priority: P2
- Confidence: high (static; both branches verified)
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/scheduler/runner.rs:111-127`
  (only `CronTaskCompleted` on `Ok`; the `Err` branch just returns the error,
  no webhook), `:47-58` (`build_fire_fn` never installs an observer);
  `WebhookTurnObserver` exists only in `drive_chat` (chat_driver.rs:83);
  `launch_cron_run` (executor.rs:3895-3928) returns `Err` for Failed/Cancelled/
  Paused — those outcomes emit nothing.
- Reachability: any cron task or background AgentChat run that ends Failed/
  Cancelled/Paused, with endpoints configured.
- Expected invariant: lifecycle parity with chat turns (AGENTS.md; F-OPS-01
  handoff: webhook emission on cron completion exists) — failures are reported
  on the same channel as success; a missing event is not the "no news" of a
  healthy system.
- Observed behavior: a failed cron run is completely silent on the webhook
  channel; only the (dead in practice, F-OPS-01-P1-01) scheduler tick logs and
  the TaskRuntime store know.
- Impact: external monitoring of cron (the primary headless use case) silently
  misses failures — misleading availability; a user relying on webhooks cannot
  distinguish "cron not firing" from "cron succeeded".
- Root cause: the fire_fn was written before the observer; success-only
  completion events were added without the failure counterpart.
- Direction: emit a `CronTaskFailed` (or reuse `AgentError`) webhook with the
  error on the `Err` branch of `build_fire_fn`; optionally route cron agent
  streams through the observer for step events.
- Regression validation: fixture — failing cron `FireFn` with an emitter →
  exactly one failure webhook event with the cause; success → `CronTaskCompleted`
  only.
- Validation reports: [V02-01](../validations/A-OBS-01/V02-01.md),
  [V03-01](../validations/A-OBS-01/V03-01.md)

### A-OBS-01-P3-01: Webhook delivery failure is log-only, retry is fixed at 1, and in-flight deliveries die silently at shutdown — no in-app reporting of a broken integration

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `webhook/emitter.rs:164-172` (detached `tokio::spawn`, 1 retry
  after 2s, `tracing::warn` only), `:204-207` (HTTP non-success → warn),
  `:133` (delivery tasks detached from any handle — process exit drops them
  without notice); EKO logs land in the TUI log file (infra.rs:1504) reachable
  only via the manual GUI log entry (MASTER-PLAN:605).
- Reachability: any configured endpoint that fails (network, 5xx) or any
  process exit during delivery.
- Expected invariant: the failure/retry/reporting scenario — a user with
  webhooks configured can learn the integration is broken; the emitter doc
  (emitter.rs:10-12) explicitly warns that silent no-op delivery once masked
  the "webhook not active" state.
- Observed behavior: after both attempts fail, only a warn log line exists; no
  UI surface, status counter, or error surface; shutdown silently drops
  in-flight deliveries.
- Impact: silent webhook outage — the same failure shape the module was
  refactored to eliminate (the old global emitter).
- Root cause: fire-and-forget design with log-only failure reporting; no
  delivery status was ever modeled.
- Direction: expose a bounded delivery-status surface (e.g. last-error
  per endpoint read by a diagnostics command, or a startup warn when endpoints
  are configured), or document best-effort semantics in the module docs;
  optionally join in-flight deliveries on shutdown.
- Regression validation: fixture with an unreachable endpoint → status/last
  error observable through the new surface.
- Validation reports: [V03-01](../validations/A-OBS-01/V03-01.md)

### A-OBS-01-P3-02: `AppState::from_shared` constructs a `WebhookEmitter` that every live entry point immediately replaces — dead construction

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/state.rs:469-470`
  (`WebhookEmitter::from_config(&app_config)` inside the constructor),
  `:569-570` (stored into `WebhookState`), overwritten at
  `src/tauri/desktop.rs:196` (`state_inner.webhook.emitter = webhook_emitter`);
  sole reader `src/tauri/commands/chat.rs:668` reads the overwritten value;
  the CLI/TUI/channel paths never construct AppState's emitter (V02-01).
- Reachability: n/a (always-replaced construction on the GUI path).
- Expected invariant: constructors don't build state they never own.
- Observed behavior: one redundant `from_config` per AppState build (also
  double-parses config).
- Impact: minor — misleading constructor, negligible cost.
- Root cause: the emitter moved to entry-point construction during the
  global-emitter removal (audit doc A1 fix) and the constructor was never
  updated.
- Direction: pass the emitter into `AppState::from_shared` (or remove the field
  construction and set it only at the entry points).
- Regression validation: grep — exactly one `from_config` per process entry
  point.
- Validation reports: [V02-01](../validations/A-OBS-01/V02-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search (emitter/observer/event enum/diagnostics/save_trace/global remnants/redaction/worker terms, both repositories) | yes | passed | [V01-01](../validations/A-OBS-01/V01-01.md) |
| V02 | Registration and runtime reachability (emitter wiring per surface + scheduler + config watcher; observer live-path; cron/background absence; two trace writers; GUI panel commands) | yes | passed | [V02-01](../validations/A-OBS-01/V02-01.md) |
| V03 | Invariant/edge cases (config identity; secret/content redaction; terminal truthfulness incl. pause/cancel/error; failure/retry/reporting; duplicate authority) | yes | failed (7 findings) | [V03-01](../validations/A-OBS-01/V03-01.md) |
| V04a | `cargo test -p echo-agent-app-core --locked observability::diagnostics` | yes | passed (exit 0; 3 ok) | [V04-01](../validations/A-OBS-01/V04-01.md) |
| V04b | `cargo test -p echo-agent-app-core --locked webhook` | yes | passed (exit 0; 0 tests — coverage gap documented) | [V04-02](../validations/A-OBS-01/V04-02.md) |
| V04c | `cargo test -p echo-agent-app-core --locked chat_driver` | yes | passed (exit 0; 9 ok) | [V04-03](../validations/A-OBS-01/V04-03.md) |
| V04d | `cargo check -p echo-agent-app-core --locked` | yes | passed (exit 0) | [V04-04](../validations/A-OBS-01/V04-04.md) |
| V05 | Historical-document drift (MASTER-PLAN webhook/diagnostics/identity claims; app-core audit A1; PROJECT-ANALYSIS) | conditional | passed | [V05-01](../validations/A-OBS-01/V05-01.md) |

All required validations executed; every reported command has a known exit
code; no validation is pending.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN:379 "M9 已完成…同一 durable run 诊断;GUI/TUI/CLI 共用 DTO/formatter" | current | observability/diagnostics.rs; panels.rs:1134; events.rs:4115 (V05-01) |
| MASTER-PLAN:713 "EKO 以 framework RunStore 为唯一诊断事实,按 parent run 聚合" | regressed | save_trace parentless records (executor.rs:3504) → P2-01 (V05-01) |
| MASTER-PLAN:709 "每次 Agent invocation 使用唯一 trace_run_id;TaskRuntime run_id 只作为 parent correlation" | regressed | save_trace uses the product run id as its own record id with parent None → P2-01 (V05-01) |
| MASTER-PLAN:721 "业务 run 与 trace invocation 不再复用 identity" | current (framework path) | stream_channel.rs:100-109 parents by run_id (V05-01) |
| MASTER-PLAN:771 "cron/background 继续以 append-only RuntimeTaskEvent 为权威" | current | ExecEvent bridge; webhook failure invisibility is P2-02, not a contradiction of this claim |
| MASTER-PLAN:605 "GUI 增加完整日志打开入口" | current | log file target; delivery failures land in logs only → P3-01 (V05-01) |
| app-core audit doc A1: "delete global singleton, unify AppState.webhook.emitter with real emit calls" | fixed | V01-01 (global gone), V02-01 (one shared emitter, live emit calls) |
| PROJECT-ANALYSIS:244 "飞书 webhook 仅处理 text" | current | Feishu channel, distinct inbound concept (out of scope) |

## Coverage And Uncertainty

- All conclusions are static except four command runs (V04); no live webhook
  HTTP delivery and no real cancelled/paused task run were exercised
  (read-only review). P1-01 and P1-02 are deterministic static proofs; P1-03's
  exact fabricated message text depends on the F-RCT-03 envelope behavior
  (verified there, not re-run here).
- The `gui` feature bin (`src/tauri/commands/chat.rs`, `panels.rs`) and the
  `channels` feature were inspected statically but not compiled in this task
  (Q-GUI-01/Q-CLI-01 own the conditional matrix); `cargo check
  -p echo-agent-app-core` (V04-04) covers the app-core anchors.
- Frontend rendering of the diagnostics panel (ObservabilityPanel.tsx) was read
  only at the status/input_preview consumers; full panel behavior is
  A-FE-01/02 scope.
- The framework trace lifecycle (parenting, finalize gaps) is consumed from
  F-RCT-02/F-RCT-03/F-OPS-01 facts, spot-verified at the two anchors cited
  (stream_channel.rs:100-109, react/mod.rs:1907-1931); no new framework finding
  is filed here.
- Whether an endpoint URL uses plain `http://` is not validated (emitter.rs:186)
  — noted in P1-02's direction as part of the redaction/transport review, not
  filed separately (AGENTS.md: light validation only).
- WebhookTurnObserver drops in-flight tool entries on cancel without emitting
  ToolCalled/ToolFailed (chat_driver.rs:125-159) — folded into P1-03's impact,
  not filed separately.

## Handoff

- Downstream tasks may rely on: configuration identity holds (one non-global
  emitter per process, hot-reload wired, no globals — V01/V02); webhook
  success events are truthful (`ChatCompleted` only after FinalAnswer,
  `CronTaskCompleted` only after real Completed — V02/V03); the four invariant
  failures above (P1-01 paused-as-completed + missing fault records, P1-02
  unredacted outbound payloads, P1-03 cancel-as-agent_error + missing terminal
  variants, P2-01 duplicate trace authority, P2-02 cron failure invisibility)
  and the two P3 items; the webhook module and `save_trace` have zero tests
  (V04-02).
- Reports to read: this report + V01-01..V05-01; dependency reports A-CHAT-01
  (canonical P1-01), A-TSK-04 (terminal/pause semantics), F-OPS-01 (P2-01/P2-04
  redaction class, save_trace handoff, scheduler P1-01).
- Stale triggers: any change to `chat_driver.rs` (observer, drive loop,
  finish), `webhook/*`, `save_trace`/terminal mapping in
  `tasks/task_runtime/executor.rs:437-584`, `scheduler/runner.rs` fire_fn,
  `observability/diagnostics.rs` grouping/status, the emitter wiring in
  main.rs/desktop.rs/state.rs, or the config watcher reload path invalidates
  the corresponding claims; also if the envelope stops fabricating cancel
  terminals or `AgentEvent::Cancelled` becomes live on the chat path (P1-03
  fixed).
- Follow-up task IDs (fixes are not implemented in this review): A-SRF-02/03
  (GUI lifecycle signals), X-EVT-01 (webhook terminal conformance), X-SEC-01
  (redaction contract across trace/audit/webhook), Q-FLT-01 (webhook
  failure/cancel/pause fixtures), S-RDM-01 (roadmap: P1-01 save_trace truth,
  P1-02 redaction choke point, P1-03 terminal variants, P2-01 delete save_trace,
  P2-02 cron failure events).
