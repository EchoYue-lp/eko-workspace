# F-OPS-01: Scheduler, headless mode, tracing, and telemetry

> Status: complete
> Reviewer: Codex primary reviewer (delegated static evidence independently sampled)
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: both source repositories clean; an external commit transition
> from `9b0e0faf74d35c9a432370b923acabfbb5f32d63` was reconciled in V14

## Question

Are reusable scheduler, headless, trace, audit, and telemetry adapters bounded,
observable, cancellable, and free of hidden lifecycle or persistence ownership?

## Scope

- `echo-orchestration/src/scheduler`: cron definition/store and runner.
- Root `scheduler`, `headless`, `trace`, `trace/analyzer`, `telemetry`, `audit`
  facades and live ReAct trace/audit producers.
- `echo-core` audit contract and `echo-state` audit implementations/callback.
- Root exports/features/examples/tests, EKO scheduler alias, run-store injection,
  conditional logging/telemetry adapter, and real external reuse points.
- Definition, duplicate authority, registration, reachability, stable identity,
  trigger/terminal/cancel/recovery, bounded retention, panic, UTF-8 and overflow.

## Out Of Scope

- Source fixes and Cargo/rustc/test/build/fixture/network execution.
- Pure `--channels` service bypass, fake surface-contract test, and application
  scheduler/background cancel/join ownership already owned by B-PATH-01.
- GUI startup error, prewarm rollback, Agent/pool/channel shutdown and duplicate
  AppState already owned by A-BOOT-01.
- ReAct core terminal/cancellation fragmentation owned by F-RCT-02; this task
  covers only the additional headless and operational adapter contracts.
- Workflow task scheduling/checkpoint semantics owned by F-WFL-01 and general
  EventBus/error/identity/token contracts owned by F-CORE-01.

## Inputs

- `AGENTS.md`; shared `README.md`, `REPORTING.md`, `TASKS.md`; Codex protocol.
- Codex dependency/boundary reports B-PATH-01, A-BOOT-01, F-CORE-01,
  F-WFL-01, and direct dependency F-RCT-02.
- Current source, local dependency source needed to verify subscriber/provider
  lifecycle semantics, and scoped Git history. No other reviewer was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Cron scheduling, non-interactive invocation result, run/audit persistence, analysis, tracing and telemetry lifecycle are independently reusable framework capabilities and remain in `echo-agent`. |
| EKO product policy | Which services start in GUI/TUI/channels, product log target, UI projections and process ownership remain in EKO. |
| Adapter boundary | EKO may bind a FireFn, store location and telemetry config, but must not implement a second cron occurrence authority, trace state machine or exporter lifecycle. |
| Duplicate search | Searched types, traits, fields, features, exports, registration, examples, tests and live calls for CronTask/SchedulerRunner, HeadlessConfig/Result, Run/RunStore, AuditLogger/Callback and telemetry init/record/shutdown across both repositories. |
| Migration deletion | Retain public framework options. Replace faulty store/time/lifecycle bodies and delete ambiguous prefix APIs or inert fields when the corrected typed authority lands; do not delete APIs merely because EKO does not call them. |

## Current Path

```text
framework cron consumer / EKO adapter
  -> CronTaskStore whole-vector state
  -> SchedulerRunner::new cache
  -> 30s tick -> CronTask::next_run(now-of-call)
  -> serial FireFn await -> best-effort update_last_run

standalone headless consumer
  -> HeadlessConfig -> ReactAgentBuilder -> Agent::execute
  -> success/error flattened to HeadlessResult -> text or JSON

framework/EKO trace
  -> start_scoped_trace_run(raw input) -> Run{Running}
  -> pipeline/phase RunEvent -> RunStore::append_event
  -> finalizer load/mutate/save terminal
  -> JsonlRunStore complete-snapshot append + cache
  -> TraceAnalyzer queries

framework audit
  -> prepare/finalize/tool pipeline or AuditCallback
  -> raw AuditEvent -> FileAuditLogger/InMemoryAuditLogger

telemetry feature
  -> init_telemetry local trace/meter providers
  -> global tracing subscriber + OnceLock instruments
  -> shutdown_telemetry(global tracer provider only)
```

Positive boundaries worth retaining: scheduler FireFn is provider-neutral; EKO
aliases rather than forks SchedulerRunner; trace identities include parent/turn/
execution fields; live ToolCall creation redacts arguments and ToolResult preview
uses UTF-8-safe character truncation; telemetry-disabled EKO still installs local
logging; token aggregation in Run uses saturating arithmetic.

## Findings

### F-OPS-01-P0-01: Run IDs escape the JsonlRunStore directory

- Priority: P0; confidence: high; layer: framework.
- Evidence: `echo-agent/src/trace/mod.rs:38`, `:710-727`, `:733-745`, `:752-760`.
- Reachability: public/deserializable Run or RunStore load ID -> `run_path` ->
  OpenOptions/read; EKO injects JsonlRunStore in
  `echo-agent-cli/echo-agent-app-core/src/infra.rs:374-380`.
- Expected invariant: logical run identity never grants filesystem path authority;
  all reads/writes remain canonically beneath the configured store directory.
- Observed behavior: `dir.join(format!("{run_id}.jsonl"))` accepts absolute and
  parent-containing IDs. Absolute paths replace `dir`; `../` escapes it, and save
  creates/appends while load reads the derived external path.
- Impact: malformed framework input can append to or read JSON-shaped files outside
  the trace root, causing local data corruption or unintended data disclosure.
- Root cause: public display identity is reused directly as a path component.
- Direction: use an opaque encoded filename or strict single-component ID,
  canonicalize the parent, verify containment before I/O, and delete direct
  `format!("{run_id}.jsonl")` path authority.
- Regression validation: absolute/parent/separator/empty/Unicode/symlink IDs and
  prove files outside a temporary root remain unchanged.
- Validation reports: [V07](../validations/F-OPS-01/V07-01.md)

### F-OPS-01-P0-02: Trace and audit production stores persist sensitive content unredacted and unbounded

- Priority: P0; confidence: high; layer: framework.
- Evidence: `echo-agent/src/trace/mod.rs:72-82`, `:302-365`, `:423-444`,
  `echo-agent/src/agent/react/mod.rs:1907-1944`,
  `echo-agent/src/agent/react/run/phases/prepare.rs:43-54`,
  `run/phases/finalize.rs:65-76`, `run/pipeline.rs:424-453`, `:827-867`,
  `echo-agent/echo-core/src/audit.rs:44-96`,
  `echo-agent/echo-state/src/audit/file.rs:39-53`.
- Reachability: every EKO agent receives JsonlRunStore; framework consumers may
  attach documented FileAuditLogger. Live prepare/finalize/tool phases construct
  the cited raw values and persist them.
- Expected invariant: known secret patterns are redacted at the durable boundary,
  and every persisted content field has a configurable size ceiling.
- Observed behavior: only the ToolCall helper redacts arguments and ToolResult
  preview is bounded. Raw user input, final output, run/tool errors, audit tool
  input/output and final answer reach files without persistence-bound redaction
  or record/field limits; public variants can bypass the helper.
- Impact: API keys, tokens, credentials or arbitrarily large local/model content
  can be retained in trace/audit files despite the framework already having a
  secret scanner, with storage and disclosure consequences on the user's machine.
- Root cause: redaction/truncation is optional producer behavior rather than one
  store-bound invariant.
- Direction: define typed content-retention policy on durable stores; recursively
  redact secret-bearing values immediately before serialization; bound previews
  while preserving explicitly referenced complete artifacts where required.
- Regression validation: secrets in every field/direct variant, nested JSON,
  Chinese/emoji, maximum lengths, and no raw secret in persisted bytes.
- Validation reports: [V06-02](../validations/F-OPS-01/V06-02.md)

### F-OPS-01-P0-03: Automatic cron ticks calculate only future occurrences and normally never fire

- Priority: P0; confidence: high; layer: framework.
- Evidence: `echo-agent/echo-orchestration/src/scheduler/cron_task.rs:59-70`,
  `runner.rs:67-101`.
- Reachability: public SchedulerRunner `spawn` -> run_loop -> tick for every
  framework/EKO recurring task.
- Expected invariant: a tick evaluates the occurrence in `(previous_tick, now]`
  against one supplied clock/reference time.
- Observed behavior: tick builds a window ending at its captured `now`, then
  `next_run` independently returns `Schedule::upcoming(Utc).next()` after a later
  current-time read. It is therefore normally greater than tick's `now`, making
  `next <= now` false. Tests ask only whether a future occurrence exists and the
  demo exercises manual run_once.
- Impact: the framework's core automatic scheduler path is effectively unusable;
  enabled recurring tasks remain silent.
- Root cause: next-future-occurrence computation is used as a previous-window query.
- Direction: inject one clock; compute the first occurrence strictly after the
  last checked durable boundary (or a per-task next occurrence), advance it only
  after a claimed terminal, and delete the independent `Utc::now` authority.
- Regression validation: clock-controlled exact boundary, delayed tick, DST/timezone,
  restart and duplicate-tick cases.
- Validation reports: [V02](../validations/F-OPS-01/V02-01.md),
  [V14](../validations/F-OPS-01/V14-01.md)

### F-OPS-01-P1-01: CronTaskStore can panic and lose concurrent or corrupt scheduler state

- Priority: P1; confidence: high; layer: framework.
- Evidence: `echo-agent/echo-orchestration/src/scheduler/cron_task.rs:110-181`,
  `:208-301`, `runner.rs:31-41`, `:122-160`, `:185-190`.
- Reachability: public Store-backed constructor/load/save and every add/status/
  execution result update; EKO selects Store-backed persistence when available.
- Expected invariant: public runtime use returns typed failure without panic and
  scheduler mutations are atomic/revision-safe and recoverable.
- Observed behavior: sync APIs bridge async Store via `block_in_place` and
  `Handle::block_on`, which panics on a current-thread runtime. Mutations are
  unsynchronized whole-vector load/modify/save; file save truncates directly;
  cache and store updates are separate. Constructor load and migration read/parse/
  remove errors are discarded, converting damage into empty/stale state.
- Impact: valid external runtime selection can panic; concurrent management/fire
  updates can overwrite tasks or terminal results; crash/corruption can erase the
  visible schedule without a diagnostic.
- Root cause: a synchronous whole-document repository was placed inside an async
  concurrent runner without transaction/revision and recovery semantics.
- Direction: make store operations async; add one revisioned atomic authority or
  per-task records, atomic file replacement/locking and explicit corrupt-state
  errors; construct SchedulerRunner with Result and delete nested runtime blocking.
- Regression validation: current-thread runtime, two writers, append/status race,
  crash-before-rename, corrupt JSON, migration failures and cache/store failure.
- Validation reports: [V03](../validations/F-OPS-01/V03-01.md),
  [V12](../validations/F-OPS-01/V12-01.md), [V14](../validations/F-OPS-01/V14-01.md)

### F-OPS-01-P1-02: One scheduled fire can block all schedules and has no durable occurrence terminal

- Priority: P1; confidence: high; layer: framework.
- Evidence: `echo-agent/echo-orchestration/src/scheduler/runner.rs:51-64`,
  `:67-117`, `:168-182`.
- Reachability: every automatic occurrence reaches serial `fire_task`; manual
  `run_once` uses the same unbounded FireFn and best-effort persistence.
- Expected invariant: each occurrence has a stable durable claim/attempt/terminal;
  runner cancellation reaches in-flight work; one task cannot block unrelated tasks.
- Observed behavior: FireFns are awaited serially without timeout or cancellation
  selection. Cancellation is checked only between sleeps. Dedup is process-local,
  marked before execution, and has no occurrence/attempt ID; multiple runners may
  duplicate work. Result persistence errors are discarded.
- Impact: a hung LLM/tool stops every future schedule and shutdown; restart or
  multiple consumers can duplicate side effects, while durable state may claim no
  terminal or retain stale output.
- Root cause: polling, execution and durable occurrence lifecycle are collapsed
  into one best-effort loop.
- Direction: create a stable occurrence key and atomic claim/terminal protocol;
  run bounded child executions with cancellation and overlap policy; persist
  typed success/failure/cancel and surface persistence failure. This does not add
  a second EKO scheduler.
- Regression validation: hung and slow jobs, cancellation, simultaneous due tasks,
  two runners, restart, retry/overlap and store failure.
- Validation reports: [V04](../validations/F-OPS-01/V04-01.md)

### F-OPS-01-P1-03: RunStore interleavings lose events and can regress terminal trace state

- Priority: P1; confidence: high; layer: framework.
- Evidence: `echo-agent/src/trace/mod.rs:586-596`, `:669-800`,
  `echo-agent/src/agent/snapshot.rs:538-557`,
  `echo-agent/src/agent/react/mod.rs:1874-1886`, `:1949-1995`.
- Reachability: live phase/pipeline event writers and all trace finalizers use
  append/load/save; EKO supplies JsonlRunStore and shares it through AgentPool.
- Expected invariant: concurrent append and finalization preserve every event,
  enforce monotonic terminal state and surface missing/corrupt state.
- Observed behavior: append is unlocked load/clone/push/save. Concurrent appends
  and finalize can overwrite the cache/latest snapshot, including Completed back
  to Running. Separate instances have stale caches and file writes are not
  serialized as one record. Missing/corrupt state becomes successful absence;
  live callers frequently drop store errors.
- Impact: diagnostics, recovery, eval and evolution consumers can observe missing
  events, false Running status or a different terminal from the actual run.
- Root cause: append-only file format stores full mutable snapshots without a
  revision/CAS authority or per-run writer lease.
- Direction: serialize or revision-check per-run mutations; use atomic single-record
  append, validate run/filename identity, make terminal transition monotonic and
  return Missing/Corrupt/Conflict errors; remove cache-as-authority semantics.
- Regression validation: concurrent events, event/finalize race, stale handles,
  partial/corrupt tail, missing IDs and terminal rollback.
- Validation reports: [V08](../validations/F-OPS-01/V08-01.md)

### F-OPS-01-P1-04: Trace and audit persistence have quadratic growth and no retention boundary

- Priority: P1; confidence: high; layer: framework.
- Evidence: `echo-agent/src/trace/mod.rs:557-596`, `:669-676`, `:733-800`,
  `trace/analyzer.rs:170-251`, `:254-340`,
  `echo-agent/echo-state/src/audit/file.rs:39-100`, `audit/memory.rs:21-59`.
- Reachability: every live event rewrites the whole Run as another JSONL line;
  EKO uses the backend for all agents and diagnostics/evolution query it.
- Expected invariant: persistent operational data has configurable record/file/
  age limits, bounded query APIs and overflow-safe aggregation.
- Observed behavior: event N serializes the complete N-event run, producing
  quadratic bytes; no trace/audit rotation, compaction or retention exists.
  In-memory stores are unbounded, parent listing requests `usize::MAX`, and
  TraceAnalyzer contains unchecked u64/usize aggregate addition over public data.
- Impact: normal long-lived local use can consume disk/memory and increasingly
  block event persistence/query; extreme stored counters can panic or wrap analyses.
- Root cause: JSONL was treated as both event log and full-snapshot store without
  an operational retention/aggregation budget.
- Direction: persist one bounded event/terminal record per append or compact by
  revision; add age/count/byte retention and bounded paging; use checked/saturating
  aggregates with overflow facts; delete `usize::MAX` fallback scans.
- Regression validation: long-run byte complexity, rotation, crash-safe compaction,
  bounded paging, retention and maximum-counter analysis.
- Validation reports: [V09](../validations/F-OPS-01/V09-01.md),
  [V12](../validations/F-OPS-01/V12-01.md)

### F-OPS-01-P1-05: Audit records claim pre-execution success and lose call correlation

- Priority: P1; confidence: high; layer: framework.
- Evidence: `echo-agent/src/agent/react/run/pipeline.rs:424-453`, `:572-620`,
  `:758-809`, `:934-955`, `echo-agent/echo-state/src/audit/mod.rs:43-113`,
  `:116-190`, `audit/file.rs:58-100`, `audit/memory.rs:45-60`.
- Reachability: audit_logger builder enables default pipeline AuditStage; exported
  AuditCallback is attachable through the normal callback list.
- Expected invariant: one stable call ID links start to exactly one truthful
  terminal, and persistence/query loss is observable.
- Observed behavior: AuditStage writes `success:true`, empty output and zero
  duration before execution and never updates normal success; infrastructure
  failure adds another failed row. Callback END always invokes `on_tool_end` even
  for failed ToolResult. AuditCallback correlates concurrent same-name calls FIFO
  by name, not call ID, and discards all logger errors. Memory poison/file read or
  parse failure can also return successful absence.
- Impact: compliance/diagnostic consumers receive false success, duplicate or
  mismatched inputs/durations and silent gaps precisely on failure paths.
- Root cause: audit schema/callback lacks call identity and start/terminal event
  distinction; best-effort backends expose success even after dropping records.
- Direction: carry existing tool call ID through callback/audit schema; define
  started and one terminal outcome or emit only terminal facts; propagate or
  publish logging failures; delete the pre-execution success row and name-prefix FIFO.
- Regression validation: success/failure/blocked/panic-equivalent result,
  concurrent same-name reversed completion, writer poison/I/O and corrupt query.
- Validation reports: [V10](../validations/F-OPS-01/V10-01.md)

### F-OPS-01-P1-06: Telemetry initialization can panic and shutdown does not own the initialized providers

- Priority: P1; confidence: high; layer: framework.
- Evidence: `echo-agent/src/telemetry.rs:59-84`, `:151-205`, `:208-265`,
  `echo-agent-cli/echo-agent-app-core/src/infra.rs:1531-1563`.
- Reachability: optional public facade and advanced re-export; EKO telemetry feature
  calls it once through init_logging while external framework hosts commonly have
  an existing subscriber.
- Expected invariant: Result-returning init never panics; ownership is explicit;
  shutdown flushes both exact providers and reports failure; disabled mode is no-op.
- Observed behavior: `.init()` panics if another global subscriber/logger exists.
  The tracing layer retains its local provider, but global shutdown targets a
  different provider. The local metric provider is not retained, so its last
  handle drops and automatically shuts the pipeline as `init_metrics` returns,
  leaving OnceLock instruments attached to a stopped pipeline. Metrics setup
  errors are swallowed as overall success and EKO discards the facade Result.
- Impact: embedding echo-agent can crash during observability setup, and callers
  cannot reliably flush/reinitialize spans/metrics or learn that metrics are absent.
- Root cause: global subscriber installation, provider creation and process shutdown
  have no returned ownership handle or explicit single-use state machine.
- Direction: return an owned TelemetryGuard containing both providers; use try_init
  or accept a caller subscriber/layer; make partial init typed and guard shutdown/
  force-flush idempotent; remove global-only shutdown and non-resettable orphan state.
- Regression validation: existing subscriber, init twice, partial exporter failure,
  record/flush/shutdown, disabled feature and post-shutdown behavior.
- Validation reports: [V11-02](../validations/F-OPS-01/V11-02.md),
  [V12](../validations/F-OPS-01/V12-01.md)

### F-OPS-01-P2-01: Prefix task management accepts empty and ambiguous identities

- Priority: P2; confidence: high; layer: framework.
- Evidence: `echo-agent/echo-orchestration/src/scheduler/cron_task.rs:184-241`,
  `runner.rs:129-181`.
- Reachability: public store/runner remove, set_status, get, run_once and
  update_last_run paths; exact removal exists beside them.
- Expected invariant: destructive/mutating lookup is exact or rejects empty and
  multiple matches.
- Observed behavior: methods use starts_with and silently choose first or all;
  empty remove matches every task and ambiguous prefixes mutate/run arbitrary tasks.
- Impact: local operator input can delete the schedule set or execute/mutate the
  wrong recurring task without a typed ambiguity error.
- Root cause: CLI convenience prefix semantics were embedded in storage authority.
- Direction: keep fuzzy resolution only in an adapter returning zero/one/many;
  make store/runner identity exact and delete prefix-based mutation methods.
- Regression validation: empty, collision, exact, Unicode and reordered store IDs.
- Validation reports: [V03](../validations/F-OPS-01/V03-01.md)

### F-OPS-01-P2-02: The headless automation contract has an inert option and flattens terminal identity

- Priority: P2; confidence: high; layer: framework.
- Evidence: `echo-agent/src/headless.rs:30-89`, `:93-159`, `:161-216`,
  `echo-agent/examples/demo54_headless.rs:166-220`.
- Reachability: public root/prelude facade advertised for CI/CD and scripting.
- Expected invariant: every public option affects behavior; machine output has
  typed terminal/error/cancel identity and automation can cancel or set a deadline.
- Observed behavior: exit_on_error is never read and exit_code ignores it;
  arbitrary format strings silently become text. Build/execution errors are folded
  into output+boolean with no typed terminal/error, run/session identity, usage,
  cancellation token, deadline or event stream. The demo simulates rather than
  invokes most scenarios.
- Impact: independent consumers cannot reliably distinguish failure/cancel/limit,
  correlate output to a run, or stop a hung operation; configured policy appears
  supported but is inert.
- Root cause: a print-and-exit convenience result is documented as a structured
  automation boundary while discarding the core invocation context/outcome.
- Direction: accept typed invocation context (identity/cancel/deadline), return a
  tagged terminal/result with usage/events as appropriate, validate an enum format,
  and either implement or delete exit_on_error. Reuse one core outcome, not a
  second headless state machine.
- Regression validation: all terminal kinds, JSON schema, invalid format,
  exit policy, cancel/deadline and identity correlation.
- Validation reports: [V05](../validations/F-OPS-01/V05-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Current definition/export/duplicate/layering search | yes | passed | [V01-02](../validations/F-OPS-01/V01-02.md) |
| V02 | Scheduler occurrence trigger lifecycle | yes | failed | [V02](../validations/F-OPS-01/V02-01.md) |
| V03 | Scheduler persistence/runtime/identity edge matrix | yes | failed | [V03](../validations/F-OPS-01/V03-01.md) |
| V04 | Scheduler in-flight cancel/terminal/occurrence lifecycle | yes | failed | [V04](../validations/F-OPS-01/V04-01.md) |
| V05 | Headless field and terminal contract | yes | failed | [V05](../validations/F-OPS-01/V05-01.md) |
| V06 | Current trace/audit redaction and payload bounds | yes | failed | [V06-02](../validations/F-OPS-01/V06-02.md) |
| V07 | JsonlRunStore path containment | yes | failed | [V07](../validations/F-OPS-01/V07-01.md) |
| V08 | Trace concurrency, terminal monotonicity and recovery | yes | failed | [V08](../validations/F-OPS-01/V08-01.md) |
| V09 | Retention, growth, paging and aggregation bounds | yes | failed | [V09](../validations/F-OPS-01/V09-01.md) |
| V10 | Audit truth, correlation and logger failure | yes | failed | [V10](../validations/F-OPS-01/V10-01.md) |
| V11 | Telemetry init/disabled/shutdown lifecycle | yes | failed | [V11-02](../validations/F-OPS-01/V11-02.md) |
| V12 | Panic, UTF-8 and overflow scan | yes | passed | [V12](../validations/F-OPS-01/V12-01.md) |
| V13 | Tests/examples/scoped-history drift | yes | passed | [V13](../validations/F-OPS-01/V13-01.md) |
| V14 | External commit transition/current-source reconciliation | yes | passed | [V14](../validations/F-OPS-01/V14-01.md) |
| V15 | Final report/link/executor/commit/source-clean gate | yes | passed | [V15](../validations/F-OPS-01/V15-01.md) |
| V30 | Primary current-commit source sampling and acceptance | yes | passed | [V30](../validations/F-OPS-01/V30-01.md) |

V01-01 and V06-01 are immutable pre-transition attempts and are superseded by
the linked attempts 02. Targeted executable scheduler/headless/store/telemetry
fixtures are future `not_run` validations because the user prohibited builds,
tests and dynamic fixtures during review; no fake validation reports were created.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| Scheduler is a periodic cron execution module | regressed | Tick cannot select the future `next_run`; [V02](../validations/F-OPS-01/V02-01.md) |
| JsonlRunStore is suitable for production persistent traces | current API, contradicted quality claim | It is live, but path/concurrency/retention invariants fail; V07-V09 |
| Tool arguments are automatically redacted from traces | current but incomplete | Live helper is used, while other persisted fields/direct variants are raw; [V06-02](../validations/F-OPS-01/V06-02.md) |
| Audit module completely records the tool chain | regressed | Pre-execution success and name-based callback pairing violate terminal truth; [V10](../validations/F-OPS-01/V10-01.md) |
| HeadlessResult is structured CI/CD output | current surface, misleading contract | Boolean/string result lacks typed terminal/identity and option is inert; [V05](../validations/F-OPS-01/V05-01.md) |
| shutdown_telemetry flushes spans and metrics | stale/incorrect | It targets an unrelated global tracer and the local metric provider has already dropped/shut down; [V11-02](../validations/F-OPS-01/V11-02.md) |
| Commit 3aa7929 changes operational findings | fixed scope check | It changes tool result order/mocks only; [V14](../validations/F-OPS-01/V14-01.md) |

## Coverage And Uncertainty

- No Cargo, rustc, tests, builds, network calls or dynamic fixtures ran. All
  implementation regressions remain future validations and do not block static
  source conclusions.
- OpenTelemetry/tracing lifecycle semantics were checked against the exact local
  dependency source already present; exporter delivery and shutdown timing were
  not executed.
- Generic TaskScheduler/workflow execution and application startup ownership were
  intentionally excluded under F-WFL-01/B-PATH-01/A-BOOT-01.
- The review did not prescribe online-service permission gates. Findings concern
  local data integrity, secret persistence, truthful observability and framework bugs.

## Handoff

- Downstream synthesis may rely on two canonical ownership conclusions: scheduler
  occurrence/persistence belongs in the framework; EKO adapters remain thin and
  must not add a second authority. Trace/audit retention/redaction also belongs
  at framework persistence boundaries.
- Read V02-V15, using V01-02, V06-02 and V11-02 rather than their superseded attempts.
- Keep B-PATH-01, A-BOOT-01 and F-RCT-02 findings canonical for their named
  application/core terminal scopes; do not duplicate them from this task.
- This report becomes stale if scheduler time/store logic, headless types, RunStore/
  audit sinks/producers, telemetry ownership, relevant feature gates, or commits change.
- Primary current-commit sampling and acceptance are recorded in V30. Dynamic
  commands remain future regression validation, not missing review evidence.
