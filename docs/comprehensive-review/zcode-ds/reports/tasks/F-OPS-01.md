# F-OPS-01: Scheduler, headless mode, tracing, and telemetry

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean

## Question

Are operational adapters (scheduler, headless mode, tracing, telemetry, audit)
bounded, observable, cancellable, and free of hidden lifecycle ownership?

## Scope

- `echo-agent/echo-orchestration/src/scheduler/` (full read: `cron_task.rs`
  367 lines, `runner.rs` 192 lines, `mod.rs`).
- `echo-agent/src/scheduler.rs` (re-export), `echo-agent/src/headless.rs`
  (full read), `echo-agent/src/telemetry.rs` (full read),
  `echo-agent/src/audit.rs` + `echo-core/src/audit.rs` (full read),
  `echo-state/src/audit/{mod,file,memory}.rs` (mod.rs full read).
- `echo-agent/src/trace/mod.rs` (full read) and `trace/analyzer.rs` (full read).
- Producers/consumers: `agent/react/mod.rs:160,1890-1960`,
  `agent/react/run/stream_channel.rs:75-120,205-245`,
  `agent/react/run/pipeline.rs:269-330,424-457,815-870,935-953`,
  `agent/react/run/phases/finalize.rs:175,216,261`, `agent/snapshot.rs:539-559`,
  `agent/react/builder.rs:767-768,801-802,1001-1024`,
  `src/security.rs:80-154`.
- EKO side: `echo-agent-cli/echo-agent-app-core/src/scheduler/{mod,runner,task}.rs`,
  `src/cli/modes.rs:32-88`, `src/tauri/desktop.rs:232`,
  `echo-agent-app-core/src/state.rs:349,375-384,540-546,639-676`,
  `echo-agent-app-core/src/infra.rs:360-390,1500-1560`,
  `echo-agent-app-core/src/plugin_runtime.rs:133,209-217,590-680`,
  `echo-agent-app-core/src/tasks/task_runtime/executor.rs:3505-3545,3893-3922`,
  `echo-agent-app-core/src/observability/diagnostics.rs`,
  `echo-agent-app-core/src/evolution/dashboard.rs:157`,
  both `Cargo.toml` trees (feature wiring).

## Out Of Scope

- ReAct loop/turn semantics (F-RCT-02), event envelope identity (F-CORE-01),
  tool execution and guards (F-EXT-01/F-SEC-01), EKO observability/webhook
  surfaces (A-OBS-01), EKO cron/background entry parity (A-SRF-04), task
  runtime executor internals (A-TSK-01..06), eval/improve consumers of trace
  events (F-EVO-01).
- The `TraceAnalyzer` math beyond reachability (V01/V02); `RunStore`
  implementations other than `InMemoryRunStore`/`JsonlRunStore`.

## Inputs

- Root `AGENTS.md`, shared `README.md`, `REPORTING.md`, `TASKS.md` (F-OPS-01
  card), `zcode-ds/README.md`, report templates.
- Dependency task reports read: zcode-ds `F-CORE-01` (event envelope/identity
  contract) and `F-RCT-02` (non-streaming loop, terminal/trace-finalization
  ownership, dead authorities). Cross-references used: F-RCT-02-P1-01
  (empty-string success), F-RCT-02-P2-01 (tools-branch terminal never
  finalizes trace run), F-RCT-02-P2-02 (dead LoopDetector pattern), F-RCT-02-P3-02
  (dead `run/approval.rs`).
- Historical documents treated as hypotheses: `docs/MASTER-PLAN.md` (M2/M9/M10
  claims), `docs/PROJECT-ANALYSIS.md` (cron subsystem), `echo-agent/AUDIT_REPORT.md`
  (2.5 redaction byte positions) — classified in the Historical Claim Status
  section.

## Layering Decision

- Generic mechanism (framework): orchestration scheduler (`CronTask`,
  `CronTaskStore`, `SchedulerRunner`), headless (`run_headless`), trace
  (`Run`/`RunEvent`/`RunStore`/`TraceAnalyzer`), telemetry (`init_telemetry`/
  `Metrics`), audit (`AuditLogger` trait + `echo-state` backends) are all
  generic, product-independent capabilities — correctly placed in
  `echo-orchestration`/`echo-agent`/`echo-core`/`echo-state`.
- EKO product policy (application): `build_fire_fn` → `launch_cron_run`
  routing, per-run pool-agent isolation, webhook emission on cron completion,
  and the decision to drive cron/background through TaskRuntime
  (`echo-agent-app-core/src/scheduler/runner.rs`); the EKO `save_trace` terminal
  record (`executor.rs:3512`) is an application-side second trace writer;
  EKO's `init_logging_with_target` OnceLock guard is application policy.
- Adapter boundary: EKO scheduler module is a type alias + `FireFn` adapter
  (thin, lossless); no second scheduler authority found. EKO `save_trace`
  bypasses framework `finalize_run` (no events, generic `"run failed"` error) —
  thin but observability-lossy (handed to A-OBS-01).
- Duplicate-search terms (both repositories, see V01): `SchedulerRunner`,
  `CronTask`, `CronTaskStore`, `run_once`, `next_run`, `run_headless`,
  `HeadlessConfig`, `RunStore`, `JsonlRunStore`, `InMemoryRunStore`,
  `TraceAnalyzer`, `init_telemetry`, `TelemetryConfig`, `Metrics`,
  `AuditLogger`, `AuditCallback`, `audit_logger`, `redact_secrets`. Results:
  one authority per concept; EKO's `BackgroundTaskService` is a distinct
  immediate-background mechanism, not a cron scheduler; `RunEvent::ToolCall`
  is produced only through the `new_tool_call` constructor (redaction point).

## Current Path

- Scheduler: `SchedulerRunner::new` loads tasks once (runner.rs:34) → `spawn`
  (tokio task, 30s tick) → `run_loop` `tokio::select!` on cancel vs sleep →
  `tick` (window `[now-30s, now]`, `last_fired` dedup) → `fire_task` awaits the
  `FireFn` serially. EKO wires it at `state.rs:644` (TUI/CLI via modes.rs:62,
  GUI via desktop.rs:232), binds plugin monitors (plugin_runtime.rs:590-680),
  and routes fires to `launch_cron_run` (executor.rs:3895).
- Headless: `run_headless(config, configure)` → `ReactAgentBuilder` +
  `agent.execute` (non-streaming) → `HeadlessResult`. Live caller: only
  `examples/demo54_headless.rs`.
- Trace: `start_scoped_trace_run` (mod.rs:1907) saves a `Run` (raw `input`) →
  live loop phases record events via `record_event` → `finalize_run`
  (snapshot.rs:548) on text/no-response/max-iterations/direct-answer terminals
  (NOT on the tools-branch terminal, F-RCT-02-P2-01) → `JsonlRunStore` appends
  full-run JSONL lines. EKO attaches the store at infra.rs:377 and additionally
  writes terminal `save_trace` records for task runs (executor.rs:3512).
- Telemetry: exported under the optional `telemetry` feature (off in both
  default builds); EKO wraps `init_telemetry` in a OnceLock (infra.rs:1525-1545)
  and ignores its result; `Metrics::record_*` have zero callers.
- Audit: `builder.audit_logger(...)` → `snapshot.guard.audit_logger` →
  `AuditStage` (raw `ctx.input`), guard-block paths, and `AuditCallback`
  (raw args/output). EKO never sets a logger → audit inactive in EKO today.

## Findings

### F-OPS-01-P1-01: `SchedulerRunner` tick can never fire — the schedule trigger lifecycle is completely broken

- Priority: P1
- Confidence: high (static chain + vendored cron-crate source + empirical run)
- Layer: framework
- Evidence: `echo-agent/echo-orchestration/src/scheduler/runner.rs:80-93`
  (tick predicate `if next >= window_start && next <= now`); `cron_task.rs:62-71`
  (`next_run` = `schedule.upcoming(Utc).next()`); cron 0.12.1 source
  `schedule.rs:222-236` (`upcoming` = `after(now)`), `queries.rs:25-40`
  (`NextAfterQuery::from` sets `initial_datetime = after + 1 second`), so the
  first yielded instant is strictly after now; empirical proof in
  [V04-01](validations/F-OPS-01/V04-01.md): for `*/1 * * * *` sampled mid-minute,
  `next > now` and `next <= now` is false.
- Reachability: framework public API (`lib.rs` via `echo_orchestration::scheduler`)
  → EKO `state.rs:644,671` (`start_scheduler_with_store` + `spawn`) in CLI/TUI
  (modes.rs:62) and GUI (desktop.rs:232) → plugin monitors bound via
  `plugin_runtime.rs:209-217` → `tick()` → `to_fire` always empty. `run_once`
  (runner.rs:169) is the only working trigger.
- Expected invariant: enabled tasks whose `next_run` falls in the 30s window
  fire; the `last_fired` map prevents double-fires.
- Observed behavior: `next_run()` always returns a strictly-future instant, so
  `next <= now` is never true; automatic firing never happens; no log or error
  signals the failure (the loop logs only "Scheduler runner started" and per-
  fire logs that never occur). The existing tests never exercise `tick`
  (`runner.rs` has no `#[cfg(test)]`; `demo70_scheduler.rs` uses only
  `run_once`).
- Impact: every cron task in EKO (user cron tasks and plugin monitors) silently
  never fires; the framework's primary scheduled-trigger capability is
  non-functional while appearing healthy; EKO's webhook/`launch_cron_run` cron
  integration is dead on the automatic path.
- Root cause: the window-based catch-up design assumed `next_run()` could
  return a past/current instant, but `upcoming(Utc)` is strictly-future; the
  predicate was never validated against the crate semantics and never tested.
- Direction: compute occurrences relative to a last-tick reference, e.g. keep
  `last_tick: DateTime<Utc>` and fire when
  `schedule.after(&last_tick).next()` falls within `(last_tick, now]`; keep the
  `last_fired` dedup; add unit tests for tick firing (due task fires once,
  not-due task skipped) and a short-interval end-to-end test. Delete the dead
  `next <= now` branch when replaced.
- Regression validation: a unit test driving `tick()` with a task whose cron
  minute matches now and asserting one fire + `last_fired` update; a test with
  `last_fired` set within 30s asserting no double-fire.
- Validation reports: [V02-01](validations/F-OPS-01/V02-01.md),
  [V03-01](validations/F-OPS-01/V03-01.md), [V04-01](validations/F-OPS-01/V04-01.md),
  [V04-02](validations/F-OPS-01/V04-02.md)

### F-OPS-01-P2-01: Trace persistence stores unredacted user input, final output, and error text in plaintext JSONL

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/react/mod.rs:1934` (`input: input.to_string()`
  in `start_scoped_trace_run`); `snapshot.rs:548-559` (`finalize_run` stores
  raw `final_output`/`error`); `trace/mod.rs:349-359` (`ToolError.message` raw);
  redaction exists only for ToolCall args (`trace/mod.rs:425-444`,
  `pipeline.rs:480`) and ToolResult preview (post-`OutputGuardStage`,
  pipeline.rs:943-953).
- Reachability: EKO attaches `JsonlRunStore` at infra.rs:377; every chat turn
  and task run writes a `Run` whose `input` is the raw user prompt (and
  `final_output`/`error` raw) to `~/.echo-agent/runs/run_*.jsonl`.
- Expected invariant: secrets are not written into persisted operational
  artifacts (AGENTS.md: "不把密钥打进日志"; the pipeline already redacts the
  model-facing paths).
- Observed behavior: a user pasting an API key/token into chat persists it in
  plaintext; `ToolError`/`run.error` may embed secret fragments from tool
  failures.
- Impact: silent secret persistence in long-lived trace files; contradicts the
  product's own redaction posture and AGENTS.md local-security rule.
- Root cause: redaction was applied at the model-facing tool boundary
  (`new_tool_call`, output guard) but never at the trace-run boundary
  (`start_scoped_trace_run`, `finalize_run`, `ToolError`).
- Direction: apply `crate::security::redact_secrets` to `input`, `final_output`,
  `error`, and `ToolError.message` at write time (single choke point in
  `start_scoped_trace_run`/`finalize_run`/`record_event`), matching the
  `new_tool_call` pattern; add a redaction unit test. F-SEC-01 owns the general
  redaction contract; this finding is the trace-specific gap.
- Regression validation: fixture — Run with an API-key-bearing input →
  `serde_json::to_string(&run)` contains no key; same for error fields.
- Validation reports: [V02-01](validations/F-OPS-01/V02-01.md),
  [V03-01](validations/F-OPS-01/V03-01.md)

### F-OPS-01-P2-02: `JsonlRunStore` appends a full-run JSON line per event — quadratic disk growth, no size caps or retention

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `trace/mod.rs:733-750` (`save` appends the complete serialized
  `Run` as one line); `:793-801` (`append_event` = load + push + save — a full
  rewrite per event); no event-count/byte caps on `Run.events` or
  `ToolCall.args`; no retention/rotation anywhere in `trace/`.
- Reachability: live loop records per-iteration events (LlmCall, ToolCall,
  ToolResult, BudgetDecision, ContextCompression — see V02) and EKO attaches
  the store to every chat/task agent (infra.rs:377); unlimited-iteration EKO
  agents (F-RCT-01-P2-03) amplify it.
- Expected invariant: trace persistence is bounded and append-efficient
  (doc at trace/mod.rs:586-596 claims "Append a single event ... without
  rewriting the entire run" — the JSONL default implementation contradicts it).
- Observed behavior: a run with N events produces N lines each up to the full
  run size → O(N²) bytes; large tool args (redacted but untruncated) inflate
  every line; `runs/` grows without bound over time.
- Impact: unbounded disk usage in a local app (AGENTS.md disk-pressure rule);
  long turns make save/append cost grow per event; `load_last_line` reads the
  whole file on cold cache.
- Root cause: the "latest line wins" JSONL design with whole-run lines was
  implemented as the default `append_event`, defeating its own doc; no
  bounds were added.
- Direction: switch `JsonlRunStore` to event-per-line appends (or delta
  records) with a terminal-line compaction, or cap events/args and rotate
  files; update the trait doc to match. Regression validation: a 100-event run
  whose file size is O(events × event size) not O(events²); an append loop
  benchmark.
- Validation reports: [V02-01](validations/F-OPS-01/V02-01.md),
  [V03-01](validations/F-OPS-01/V03-01.md), [V04-03](validations/F-OPS-01/V04-03.md)

### F-OPS-01-P2-03: `CronTaskStore::with_store` migration deletes the legacy tasks file even when the new backend is volatile — cron tasks can be permanently lost

- Priority: P2
- Confidence: medium (requires a backend-creation failure plus an existing
  legacy file)
- Layer: framework (triggered through the EKO fallback)
- Evidence: `cron_task.rs:111-119` (`with_store` unconditionally runs
  `migrate_from_file`), `:275-301` (`migrate_from_file` deletes the legacy
  file after `save_all` into the new backend — no durability check); EKO
  fallback `echo-agent-cli/src/cli/modes.rs:58-78`: `FileStore::new` error →
  `InMemoryStore` + `CronTaskStore::with_store` (which migrates and deletes).
- Reachability: EKO CLI/GUI boot path when `FileStore::new` fails (e.g.,
  permission error on the scheduler_store dir) while
  `~/.echo-agent/scheduler/tasks.json` still exists (pre-migration data).
- Expected invariant: migration must never destroy the only durable copy of
  cron tasks (AGENTS.md: protection against accidental user-data loss).
- Observed behavior: with a volatile in-memory backend, the legacy file is
  deleted after a "successful" migration; the data vanishes at process exit.
- Impact: permanent loss of all scheduled tasks on a rare-but-plausible boot
  failure; silent (no warning about durability).
- Root cause: migration assumes the target backend is durable; the framework
  API accepts any `Store` implementation.
- Direction: only delete the legacy file when the target backend is a durable
  file/store backend (or make deletion opt-in with an explicit
  `migrate_and_delete` API); EKO's in-memory fallback should skip migration
  entirely. Regression validation: test — `with_store(InMemoryStore)` with an
  existing legacy file must leave the file in place.
- Validation reports: [V03-01](validations/F-OPS-01/V03-01.md)

### F-OPS-01-P2-04: Audit path persists raw tool inputs and outputs without secret redaction

- Priority: P2
- Confidence: medium (audit is opt-in; EKO does not enable it today)
- Layer: framework
- Evidence: `echo-agent/src/agent/react/run/pipeline.rs:439-450` (`AuditStage`
  stores `ctx.input` raw); `echo-state/src/audit/mod.rs:117-181`
  (`AuditCallback` stores raw `args`/`result`); `echo-state/src/audit/file.rs`
  writes raw JSONL; no redaction anywhere in `echo-core/src/audit.rs`.
- Reachability: any consumer calling `ReactAgentBuilder::audit_logger(...)`
  (builder.rs:767, live pipeline stage always present, snapshot.rs:901 and
  execution.rs:244 guard-block paths) with `FileAuditLogger`/`InMemoryAuditLogger`.
- Expected invariant: the same secret-redaction posture as the trace path
  (AGENTS.md); audit logs must not become a second plaintext channel for keys
  in tool arguments (e.g., shell/env tools).
- Observed behavior: tool inputs containing tokens/keys and raw outputs are
  stored verbatim in audit events; the trace path redacts the same data.
- Impact: secret leakage into audit artifacts whenever audit is enabled — a
  security-sensitive capability (compliance review) that undermines the
  product's redaction standard.
- Root cause: redaction was added to the trace path only; `AuditStage` and
  `AuditCallback` predate/omit it.
- Direction: redact `ToolCall.input`/`output` in `AuditStage` and
  `AuditCallback` (reuse `crate::security::redact_secrets`), or document the
  raw-storage contract explicitly and gate it; add a redaction fixture.
- Regression validation: audit fixture with a key-bearing tool arg asserting
  `[REDACTED:` in the persisted event.
- Validation reports: [V02-01](validations/F-OPS-01/V02-01.md),
  [V03-01](validations/F-OPS-01/V03-01.md)

### F-OPS-01-P3-01: Five `RunEvent` variants are never produced — the documented trace event contract is partly aspirational

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `trace/mod.rs:361-365` (`Error` with `#[allow(dead_code)]`),
  `:390-421` (`FileEdit`, `TestRun`, `SubAgentRun`); `PermissionDecision`
  produced only in the dead module `run/approval.rs:179` (F-RCT-02-P3-02);
  live `PermissionStage` (`pipeline.rs:272-330`) records no trace event;
  repo-wide grep (tests included) finds zero construction of `Error`,
  `FileEdit`, `TestRun`, `SubAgentRun`; `analyzer.rs:440` still pattern-matches
  `RunEvent::Error`.
- Reachability: never at runtime; the variants exist in the public serialized
  contract that consumers (analyzer.rs, EKO diagnostics.rs, improve/eval
  modules) match against.
- Expected invariant: every documented `RunEvent` kind is emitted by some live
  loop path, or the contract is reduced to the emitted kinds.
- Observed behavior: five kinds are dead contract surface; sub-agent
  dispatches and permission decisions never appear in traces even though the
  contract documents them.
- Impact: consumers cannot rely on the documented contract (observability
  gaps for permission decisions and sub-agent runs); misleading public API;
  maintenance burden.
- Root cause: contract designed ahead of producers (same pattern as
  F-RCT-02-P2-02's LoopDetector); producers were never wired or the variants
  never removed.
- Direction: either record `PermissionDecision` in the live `PermissionStage`
  (and file-edit/subagent/test events at their authorities — coordinate with
  F-SUB-01/F-EXT-02) or delete the dead variants and their analyzer arms.
  Regression validation: `cargo check -p echo_agent` and a grep asserting each
  remaining variant has ≥1 producer.
- Validation reports: [V01-01](validations/F-OPS-01/V01-01.md),
  [V02-01](validations/F-OPS-01/V02-01.md)

### F-OPS-01-P3-02: `HeadlessConfig.exit_on_error` is never read, and headless inherits the non-streaming empty-success path

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `headless.rs:36` (field), `:106-159` (`run_headless` never
  references `exit_on_error`; only the example demo54_headless.rs and unit
  tests touch it); `run_headless` uses `agent.execute` (non-streaming), whose
  intervention-cancel path returns `Ok("")` instead of an error
  (F-RCT-02-P1-01, react_loop.rs:729-750).
- Reachability: any caller of the public `run_headless` API (currently the
  demo example).
- Expected invariant: `exit_on_error` (documented "Exit with error if the
  agent reports failure") must change exit semantics; a failed/cancelled turn
  must not report success.
- Observed behavior: the option is a silent no-op; success is solely
  `agent.execute`'s `Result`; an empty-string success exits 0.
- Impact: headless scripts cannot rely on the documented exit contract;
  CI-pipeline usage (the module's stated purpose) would pass on swallowed
  failures.
- Root cause: config field scaffolded before the error-mapping semantics were
  settled; the non-streaming wrapper bug (F-RCT-02-P1-01) compounds it.
- Direction: honor `exit_on_error` by mapping empty output/`success` (or
  deleting the field); once F-RCT-02-P1-01 is fixed, add a headless regression
  test asserting `Err`-driven exit codes.
- Regression validation: mocked-LLM headless test — turn ending with empty
  output → `exit_code() == 1` when `exit_on_error` is true.
- Validation reports: [V02-01](validations/F-OPS-01/V02-01.md),
  [V03-01](validations/F-OPS-01/V03-01.md), [V04-04](validations/F-OPS-01/V04-04.md)

### F-OPS-01-P3-03: Telemetry metrics are unwired (zero callers) and `init_telemetry` panics on a second call

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `telemetry.rs:80-147` (`Metrics` + `record_*`) — zero callers in
  either repository (V02); `telemetry.rs:183-192`
  (`tracing_subscriber::registry()...init()` panics when a global subscriber is
  already installed; no guard inside `init_telemetry`); EKO wraps it in a
  OnceLock and discards the result (`infra.rs:1525-1545`, `let _ =`).
- Reachability: feature-gated module (`Cargo.toml:72`, default off) — only when
  a consumer enables `telemetry`; metrics never recorded even then.
- Expected invariant: an enabled telemetry feature exports real data; a
  framework init API must not panic on double initialization.
- Observed behavior: OTLP tracing works through existing `tracing` macros, but
  the metrics instruments export nothing; a second `init_telemetry` (or any
  pre-registered subscriber) panics.
- Impact: consumers enabling telemetry get an empty metrics pipeline (no LLM/
  tool counters) and a panic hazard at init; misleading optional API.
- Root cause: metrics instruments were scaffolded without recording call
  sites; init was written as a plain one-shot without idempotence (EKO had to
  work around it).
- Direction: either wire `Metrics::record_llm_call/record_tool_execution` into
  the loop/tool paths (think.rs/pipeline.rs) or remove the instruments;
  guard `init_telemetry` (OnceLock or return an error) and propagate its result
  in EKO. Regression validation: feature-gated compile (V04-06 pattern) plus a
  double-init test asserting `Err` (not panic).
- Validation reports: [V02-01](validations/F-OPS-01/V02-01.md),
  [V03-01](validations/F-OPS-01/V03-01.md), [V04-06](validations/F-OPS-01/V04-06.md)

### F-OPS-01-P3-04: Scheduler fires are unbounded and serial; cancellation never reaches in-flight fires, and EKO never cancels its scheduler token

- Priority: P3
- Confidence: medium (design limitation; impact depends on fire_fn runtime)
- Layer: framework (EKO wiring noted)
- Evidence: `runner.rs:99-101` (`for task in to_fire { self.fire_task(task).await }`
  — serial, no timeout); `:54-64` (cancel checked only between ticks);
  `fire_task` has no cancel token to pass; EKO `state.rs:542` creates
  `scheduler.cancel_token` which is **never cancelled** (main.rs:336,400,445
  cancel a different token; desktop.rs:261 likewise); `launch_cron_run`
  (executor.rs:3895) receives a fresh per-fire token unrelated to the
  scheduler's.
- Reachability: any long-running fire (a cron Agent run) delays all subsequent
  due tasks and the 30s cadence; a stuck fire blocks the loop forever.
- Expected invariant: the scheduler remains responsive and cancellable while
  fires are in flight (task question: "cancellable, bounded").
- Observed behavior: one stuck fire stalls the whole scheduler; shutdown
  relies on process exit killing the detached tokio task; EKO has no
  in-process stop path.
- Impact: missed/delayed cron executions; no observability of in-flight
  fires beyond logs.
- Root cause: the runner was built as a simple sequential tick loop without
  per-fire isolation; cancellation was scoped to the loop only.
- Direction: spawn each fire on its own task with a per-fire timeout and a
  derived cancel token; join handles on shutdown; document that cancel
  terminates the loop (and after P1-01's fix, the window accounting). EKO
  should cancel `scheduler.cancel_token` in its shutdown path.
- Regression validation: test with a slow `FireFn` — a second due task still
  fires after the first completes (or the timeout aborts it); cancel while a
  fire is in flight.
- Validation reports: [V02-01](validations/F-OPS-01/V02-01.md),
  [V03-01](validations/F-OPS-01/V03-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search across both repositories (scheduler/headless/trace/telemetry/audit) | yes | passed | [V01-01](validations/F-OPS-01/V01-01.md) |
| V02 | Registration and runtime reachability (scheduler wiring, headless callers, telemetry init, trace producers/finalizers, audit producers) | yes | passed | [V02-01](validations/F-OPS-01/V02-01.md) |
| V03 | Invariant/edge-case inspection (scheduler trigger lifecycle, headless contract, trace redaction/size, telemetry-disabled, secret leakage) | yes | passed | [V03-01](validations/F-OPS-01/V03-01.md) |
| V04a | cron 0.12.1 `upcoming` empirical check (/tmp/croncheck) | yes | passed (exit 0) | [V04-01](validations/F-OPS-01/V04-01.md) |
| V04b | `cargo test -p echo_orchestration --lib --locked` | yes | passed (exit 0; 294 passed) | [V04-02](validations/F-OPS-01/V04-02.md) |
| V04c | `cargo test -p echo_agent --lib --locked 'trace::'` | yes | passed (exit 0; 21 passed) | [V04-03](validations/F-OPS-01/V04-03.md) |
| V04d | `cargo test -p echo_agent --lib --locked 'headless::'` | yes | passed (exit 0; 4 passed) | [V04-04](validations/F-OPS-01/V04-04.md) |
| V04e | `cargo test -p echo_agent --lib --locked 'security::'` | yes | passed (exit 0; 30 passed) | [V04-05](validations/F-OPS-01/V04-05.md) |
| V04f | `cargo check -p echo_agent --no-default-features --features telemetry --locked` | yes | passed (exit 0) | [V04-06](validations/F-OPS-01/V04-06.md) |
| V05 | Historical-document drift (MASTER-PLAN, PROJECT-ANALYSIS, AUDIT_REPORT) | yes | passed | [V05-01](validations/F-OPS-01/V05-01.md) |

All required validations executed; every reported command has a known exit
code; no validation is pending.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| PROJECT-ANALYSIS:38 "`tokio::select!` + `sleep(30s)`, `[now-30s, now]` 窗口, 对 `next_run` 落窗且 Enabled 的任务 fire" | stale (described behavior impossible) | runner.rs:80-93; cron `upcoming` strictly-future (V04-01); P1-01 |
| PROJECT-ANALYSIS:20,33-39 cron → `launch_cron_run` routing + pool-per-run isolation | current | scheduler/runner.rs:47-129; executor.rs:3895-3913 |
| MASTER-PLAN M10 (:380) "trace、scheduler 和 cron terminal 缺口已收敛" | current (routing half); trigger defect undocumented | executor.rs:3895; V02-01 |
| MASTER-PLAN M9 (:721) framework = generic facts + trace correlation, EKO = aggregation | current | trace/mod.rs RunEvent; observability/diagnostics.rs |
| MASTER-PLAN (:535) TraceAnalyzer.tool_reliability_report reuse | current | evolution/dashboard.rs:157 |
| AUDIT_REPORT 2.5 (MEDIUM) redact_secrets byte-position panic risk | fixed | security.rs:105-128 char-boundary checks; V04-05 |
| AUDIT_REPORT (:621) 12-pattern secret scanning | current | security.rs SECRET_PATTERNS; V04-05 |

## Coverage And Uncertainty

- All conclusions are static except six command runs (V04) and the /tmp cron
  check; no dynamic run exercised a real scheduler fire (the tick is broken),
  an OTLP export, or a headless end-to-end turn.
- `TraceAnalyzer` internals beyond reachability (aggregation math) were not
  re-audited (F-EVO-01/Q-* scope); `improve`/`eval` RunEvent consumers were
  inventoried (V02) but not reviewed.
- EKO `observability/diagnostics.rs` and `webhook` are A-OBS-01 scope;
  `save_trace` (executor.rs:3512) is flagged here only for its trace-contract
  shape (event-less terminal records, generic `"run failed"` error).
- The corrupt-store path (FileStore parse failure → empty state,
  store.rs:238-244; `SchedulerRunner::new` `unwrap_or_default`, runner.rs:34)
  silently empties the scheduler task list — noted, folded into P2-03's
  direction, not filed separately.
- Tool-call args in trace events are redacted but not size-capped; this
  amplifies P2-02 and is covered there.
- Telemetry-disabled behavior was verified statically (feature defaults) and
  by feature compile (V04-06); no runtime OTLP attempt was made (no collector
  in the environment).
- Scheduler: no test exists for tick/fire (V04-02), so P1-01's regression
  validation must be written during the fix; `runner.rs`'s `FireFn` is
  `Result<String>` with no timeout/cancel channel by contract.

## Handoff

- Downstream tasks may rely on: the scheduler trigger is dead (P1-01, with
  empirical proof V04-01); trace finalization asymmetry for the tools-branch
  terminal (F-RCT-02-P2-01) leaves EKO trace runs `Running`; the EKO scheduler
  cancel token is never cancelled (P3-04); five `RunEvent` variants are
  never produced (P3-01); trace/audit redaction gaps (P2-01/P2-04);
  `JsonlRunStore` quadratic growth (P2-02); telemetry metrics unwired and
  `init_telemetry` panic-prone (P3-03); headless `exit_on_error` dead (P3-02).
- Reports to read: this report + [V01-01](validations/F-OPS-01/V01-01.md)
  through [V05-01](validations/F-OPS-01/V05-01.md); dependency reports
  F-CORE-01, F-RCT-02.
- Stale triggers: any change to `echo-orchestration/src/scheduler/*`,
  `src/trace/*`, `src/headless.rs`, `src/telemetry.rs`, `echo-core/src/audit.rs`,
  `echo-state/src/audit/*`, `pipeline.rs` stage order, or the cron crate
  version invalidates the corresponding claims.
- Follow-up task IDs (fixes not implemented in this review): A-OBS-01 (EKO
  trace finalization, `save_trace`, scheduler cancel wiring), A-SRF-04 (cron
  trigger parity — blocked by P1-01), F-SEC-01 (trace/audit redaction
  contract), X-EVT-01 (trace event contract completeness, P3-01),
  Q-PERF-01 (JsonlRunStore growth), Q-FLT-01 (scheduler fault fixtures after
  the P1-01 fix), B-DOC-01 (PROJECT-ANALYSIS scheduler section rewrite).
