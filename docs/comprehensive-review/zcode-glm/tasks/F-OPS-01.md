# F-OPS-01: Scheduler, headless mode, tracing, and telemetry

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa
> `echo-agent-cli` commit: b3b2e81 (read-only cross-reference)
> Worktree state: clean

## Question

Are operational adapters (cron scheduler, headless runner, execution trace store,
OpenTelemetry plumbing) bounded, observable, cancellable, and free of hidden
lifecycle ownership?

## Scope

Primary source paths inspected:

- `echo-agent/echo-orchestration/src/scheduler/mod.rs`
- `echo-agent/echo-orchestration/src/scheduler/cron_task.rs`
- `echo-agent/echo-orchestration/src/scheduler/runner.rs`
- `echo-agent/src/scheduler.rs` (re-export shim)
- `echo-agent/src/headless.rs`
- `echo-agent/src/telemetry.rs`
- `echo-agent/src/trace/mod.rs`
- `echo-agent/src/audit.rs`
- `echo-agent/src/security.rs` (`redact_secrets`, used by the trace layer)
- Application wiring in
  `echo-agent-cli/echo-agent-app-core/src/scheduler/runner.rs`,
  `echo-agent-cli/echo-agent-app-core/src/state.rs`,
  `echo-agent-cli/src/main.rs`,
  `echo-agent-cli/src/cli/modes.rs`,
  `echo-agent-cli/echo-agent-app-core/src/infra.rs`.

## Out Of Scope

- `echo-orchestration/src/tasks/scheduler.rs` (`TaskScheduler`, DAG parallel
  strategy) — separate concept owned by F-TSK-02 / F-TSK-03.
- Background task service lifecycle — covered by A-BOOT-01 / A-SRF-04.
- Webhook emitter mechanics — covered by A-OBS-01.
- Provider-neutral LLM streaming contract — covered by F-LLM-01 / F-RCT-03.
- `trace/analyzer.rs` analytics correctness — only inspected for output sizing.

## Inputs

Required documents read:

- `docs/comprehensive-review/REPORTING.md`
- `docs/comprehensive-review/templates/task-report.md`,
  `templates/validation-report.md`
- `docs/comprehensive-review/TASKS.md` (F-OPS-01 entry)
- `AGENTS.md` (invariants: panic/UTF-8 safety, no SQLite in CLI, layering,
  terminology; product positioning for threat model)

Dependency task reports read:

- `docs/comprehensive-review/zcode-glm/tasks/B-BASE-01.md` (feature topology
  baseline; confirms `telemetry` is an optional dep feature, `tasks` feature
  gate, etc.)
- `docs/comprehensive-review/zcode-glm/tasks/B-PATH-01.md` (composition-root
  inventory; cross-references scheduler wiring in `start_headless_services`).
- `docs/comprehensive-review/zcode-glm/tasks/Q-STA-01.md` (static-safety audit;
  documents the `block_in_place` / runtime-assumption surface in scheduler).

Historical documents treated as hypotheses: none for this slice — F-OPS-01 is a
fresh subsystem review; prior phase plans referenced here are cited as evidence
in V04 (`Phase 3.1`, `Phase C` annotations inside `scheduler/runner.rs`).

## Layering Decision

| Concept | Classification | Reason |
|---|---|---|
| `CronTask` / `CronTaskStore` / `SchedulerRunner` | **framework** (generic mechanism) | Generic cron-style scheduler parameterized over a `FireFn`. Any `echo-agent` consumer (not only EKO) may need it. Demo `examples/demo70_scheduler.rs` exercises it without EKO. |
| `run_headless` + `HeadlessConfig`/`HeadlessResult` | **framework** (generic mechanism) | Public one-shot runner that any consumer can use; only caller outside tests is a demo example. Not EKO-specific. |
| `RunStore` / `JsonlRunStore` / `InMemoryRunStore` / `RunEvent` | **framework** (generic mechanism) | Provider-neutral execution trace store; consumed by `ReactAgent` via `run_store`, useful for any agent. |
| `telemetry::{TelemetryConfig, init_telemetry, shutdown_telemetry, Metrics}` | **framework** (generic mechanism, optional feature) | OpenTelemetry export wiring; gated by `telemetry` feature so consumers can opt out. |
| `redact_secrets` | **framework** (generic mechanism) | Local-valid safety helper (block secrets from logs/traces). |
| `build_fire_fn` / `new_scheduler_runner` in `echo-agent-app-core/src/scheduler/runner.rs` | **adapter** | Thin wrapper that wires `AgentHandle` + `TaskRuntimeStore` + pool into the framework `FireFn`. Contains product policy (cron routes through `launch_cron_run`, `[plan]` strip, pool acquire/release) — correct placement. |
| `AppState::start_scheduler_with_store` | **application** | Owns the scheduler lifetime, decides whether to enable it per mode, supplies the file Store backend. |

Repository-wide duplicate-search terms used: `SchedulerRunner`,
`CronTaskStore`, `CronTask`, `TaskScheduler`, `run_headless`, `HeadlessConfig`,
`RunStore`, `JsonlRunStore`, `init_telemetry`, `shutdown_telemetry`,
`Metrics::record_`, `redact_secrets`. The `tasks/scheduler.rs::TaskScheduler`
collision is a different concept (DAG parallel strategy) and is intentionally
out of scope.

## Current Path

### Scheduler trigger lifecycle (V01)

1. `AppState::start_scheduler_with_store`
   (`echo-agent-cli/echo-agent-app-core/src/state.rs:644`) builds a
   `CronTaskStore` (file or `echo_agent::memory::Store`-backed) and calls
   `new_scheduler_runner` (`echo-agent-cli/echo-agent-app-core/src/scheduler/runner.rs:164`),
   which constructs the framework `SchedulerRunner` with a `FireFn` that
   dispatches to `launch_cron_run`. `runner.clone().spawn()` then calls
   `tokio::spawn` (`echo-orchestration/src/scheduler/runner.rs:46`).
2. The background task enters `run_loop` (line 52) which is a
   `tokio::select!` over `self.cancel.cancelled()` and a 30 s sleep → `tick`.
3. `tick` (line 68) reads the in-memory `tasks: RwLock<Vec<CronTask>>`,
   computes `window_start = now - 30 s`, and selects tasks whose `next_run()`
   falls in `[window_start, now]` and that were not fired within the last 30 s
   (tracked by `last_fired: RwLock<HashMap<String, DateTime<Utc>>>`). Selected
   tasks are cloned into `to_fire` and fired **serially** via `fire_task`
   outside the locks (line 99).
4. `fire_task` (line 105) calls the `FireFn`, then
   `store.update_last_run(&task.id, &result)` which truncates the result to 500
   chars (`cron_task.rs:231`) and persists.
5. Cancellation: the loop exits when `self.cancel.cancelled()` resolves. The
   `cancel` token is `AppState.scheduler.cancel_token`
   (`state.rs:378`). `main.rs` constructs its **own** `cancel_token`
   (`src/main.rs:230`) only for `spawn_config_watcher` — it is **not** the
   scheduler's token. Grep for `scheduler.cancel_token.cancel()` returns zero
   hits across both repositories.
6. Termination: `tokio::spawn` returns a `JoinHandle` that is **dropped** at
   `spawn()` (`runner.rs:46-48`); the task is detached. There is no
   `.abort()`, no `JoinHandle::join`, no graceful shutdown barrier anywhere.
   In-flight `fire_task` invocations are not awaited or cancelled at process
   exit.

### Headless event contract (V02)

`run_headless` (`echo-agent/src/headless.rs:106`) constructs a
`ReactAgentBuilder`, optionally applies `max_iterations`, builds the agent, and
calls `agent.execute(&config.prompt)` (line 145), where `execute` returns
`Result<String>` via `run_direct`
(`echo-agent/src/agent/react/mod.rs:2767`). It then returns a single
`HeadlessResult { output, success, model, format }` with two serialization
modes (`text` raw, `json` pretty-printed).

What it does NOT do, which the interactive/streaming path does:

- No `execute_stream` / `chat_stream_message` — no `BoxStream<AgentEvent>`
  consumer is ever attached.
- No `RunStore` registration (no `with_run_store`, no
  `start_trace_run`/`finalize_trace_run`).
- No callback / `AgentCallback` registration.
- No `CancellationToken` plumbing — `max_iterations` is the only stopper; an
  infinite tool loop without iterations cap, or a hung LLM provider, is
  uncancellable from outside.
- No tool/permission/metrics emission — `Metrics::record_*` is never invoked
  from headless (nor anywhere else; see V04).

Live callers in the codebase: `examples/demo54_headless.rs` only.
`echo-agent-cli` does not use `run_headless`; it uses `run_cli_mode` /
`run_tui` which wire full streaming.

### Trace redaction and size (V03)

`RunEvent::new_tool_call` (`echo-agent/src/trace/mod.rs:425`) applies
`crate::security::redact_secrets` to tool args. Everywhere else, raw strings
flow into the persisted `Run`:

- `Run.input` (`trace/mod.rs:73`) is set verbatim from the user prompt at
  `react/mod.rs:1934` (`input: input.to_string()`).
- `Run.final_output` (`trace/mod.rs:79`) is set verbatim from the assistant
  answer at `react/mod.rs:1964` / `1989`.
- `Run.error` (`trace/mod.rs:83`) is set verbatim from the agent error.
- `RunEvent::ToolResult.output_preview`
  (`react/run/pipeline.rs:834-849`) is `result.output.chars().take(200)` with
  no redaction.
- `RunEvent::ToolError.message` (`react/run/pipeline.rs:864`) is
  `result.error.clone()` with no redaction.
- `RunSummary.input_preview` is `self.input.chars().take(80)` — also
  unredacted (`trace/mod.rs:136`).

`JsonlRunStore::save` (`trace/mod.rs:733`) opens
`{dir}/{run_id}.jsonl` in append mode and writes the full `Run` as one JSON
line on every `save` (so each event append rewrites the entire run as a new
line; the doc comment "latest line always represents the current run state"
holds). There is no max-file-size, no rotation, no compaction, no eviction. The
in-memory `cache: RwLock<HashMap<String, Run>>` (`trace/mod.rs:680`) is
unbounded — grows monotonically with the number of distinct `run_id`s for the
lifetime of the process. `InMemoryRunStore::runs` (`trace/mod.rs:606`) is
likewise unbounded.

`redact_secrets` itself (`echo-agent/src/security.rs:105`) is UTF-8-safe (it
checks `is_char_boundary` before `replace_range`, line 115-117) and applies
~18 patterns (AWS, GitHub, SSH, Bearer, Anthropic, OpenAI, JWT, etc.). The
redaction comment in `RunEvent::ToolCall.args` says "may be redacted for
secrets" — true for tool args only.

### Telemetry-disabled behavior (V04)

Feature gate: `telemetry = ["dep:opentelemetry", "dep:opentelemetry_sdk",
"dep:opentelemetry-otlp", "dep:tracing-opentelemetry"]`
(`echo-agent/Cargo.toml:72`). `pub mod telemetry`
(`echo-agent/src/lib.rs:99-100`) and the prelude re-export
(`src/lib.rs:296-298`) are both behind `#[cfg(feature = "telemetry")]`.

The CLI's `init_logging_with_target`
(`echo-agent-cli/echo-agent-app-core/src/infra.rs:1536`) wraps the telemetry
init in `#[cfg(feature = "telemetry")]` and provides a plain
`tracing_subscriber` fallback under `#[cfg(not(feature = "telemetry"))]`
(line 1563-1630) — clean gating, no orphan imports.

Two functional gaps inside the gated module:

1. `Metrics::record_llm_call`, `record_llm_tokens`, `record_llm_latency`,
   `record_tool_execution`, `record_tool_latency`
   (`echo-agent/src/telemetry.rs:87-146`) are **never called** from anywhere
   in `echo-agent` or `echo-agent-cli` outside the `telemetry.rs` definitions
   themselves (grep returns only the five definition sites). Even with the
   feature on and an OTLP endpoint configured, no LLM/tool metric is ever
   recorded. The `Metrics` struct is initialized but never fed.
2. `shutdown_telemetry` (`telemetry.rs:263`, which calls
   `opentelemetry::global::shutdown_tracer_provider`) is **never called** from
   the CLI (`grep` returns only the definition and the prelude re-export).
   Pending spans/metrics in the OTLP batch exporter are not flushed at process
   exit; they are silently dropped. The OTel SDK batch exporter runs on a
   tokio task that is detached at shutdown.

## Findings

### F-OPS-01-P1-01: Scheduler has no graceful shutdown path

- Priority: P1
- Confidence: high
- Layer: framework (mechanism is generic; impact realised through the application)
- Evidence:
  - `echo-agent/echo-orchestration/src/scheduler/runner.rs:45-49` (`spawn`
    drops the `JoinHandle`).
  - `echo-agent/echo-orchestration/src/scheduler/runner.rs:52-65` (`run_loop`
    only observes `self.cancel.cancelled()`).
  - `echo-agent-cli/src/main.rs:230,336,400,445` (`cancel_token` is for
    `spawn_config_watcher`, not the scheduler).
  - `echo-agent-cli/echo-agent-app-core/src/state.rs:376-379` (the scheduler's
    own `cancel_token` field).
- Reachability: `AppState::start_scheduler_with_store` →
  `crate::scheduler::new_scheduler_runner` → `runner.clone().spawn()` →
  `tokio::spawn(run_loop)`; live in TUI, CLI, and (via `bind_scheduler`) plugin
  monitor paths.
- Expected invariant (from TASKS.md question "cancellable ... free of hidden
  lifecycle ownership"): an operational adapter must (a) terminate on shutdown
  signal and (b) let its owner observe or await termination.
- Observed behavior: no caller ever cancels
  `AppState.scheduler.cancel_token`. The runner's `JoinHandle` is detached at
  `spawn`. At process exit the runtime is torn down while `fire_task` may
  still be mid-`launch_cron_run`. The 30 s `tokio::select!` sleep means even a
  SIGINT-after-cancel can take up to 30 s to register if the runtime does not
  drop first; in practice the process simply exits and the task is killed.
- Impact: in-flight cron runs (which may write to worktrees, spawn
  subagents, or hold pool entries) are not gracefully drained. On EKO restart
  these appear as stale `TaskRuntimeStore` runs that must be reaped by
  recovery logic rather than awaited. For a generic framework consumer using
  `SchedulerRunner` directly, there is no API to wait for shutdown.
- Root cause: the application owns the cancellation token but never fires it,
  and the framework provides no join handle.
- Direction:
  - Framework: change `spawn` to return the `JoinHandle` (or store it on
    `SchedulerRunner`) so a caller can `await` termination after cancel.
  - Application: in `AppState` Drop or a dedicated `shutdown()` method, call
    `self.scheduler.cancel_token.cancel()` and await the runner handle before
    dropping the runtime.
- Regression validation: a test that starts the runner with a `FireFn` that
  sleeps for 5 s, fires cancel mid-fire, and asserts the run-loop task exits
  within bounded time and the in-flight future is observed to completion or
  cancelled cleanly.
- Validation reports: [V01-01](../validations/F-OPS-01/V01-01.md)

### F-OPS-01-P1-02: Headless mode is not event-equivalent to interactive mode

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/headless.rs:106-159` (`run_headless` calls
    `agent.execute`, returns `HeadlessResult`).
  - `echo-agent/src/agent/react/mod.rs:2767-2778` (`execute` returns
    `Result<String>` via `run_direct`).
  - Compare `execute_stream` at `react/mod.rs:2780-2789` which yields
    `BoxStream<Result<AgentEvent>>`.
- Reachability: `pub async fn run_headless` re-exported in prelude
  (`src/lib.rs:265`); only live caller is `examples/demo54_headless.rs`.
- Expected invariant (TASKS.md V02: "Is it equivalent to interactive mode?"):
  headless should be a non-interactive rendering policy over the same
  streaming execution, per AGENTS.md "多模式功能对等" (TUI/GUI/CLI/headless
  must be functionally equivalent).
- Observed behavior: no `AgentEvent` stream is consumed; no `RunStore` is
  attached; no `CancellationToken` is exposed; no callback, metrics, or trace
  is emitted. A consumer that wants the framework's own observability
  (events, traces, metrics) from a CI/scripted run gets none of it.
- Impact: headless is an informational demo path, not a complete operational
  mode. CI consumers using `run_headless` cannot observe tool calls, capture
  streaming JSON for incremental UI, recover via snapshot, or cancel a hung
  provider. The product's own CLI does not use it precisely because it lacks
  these — confirming the gap is real, not stylistic.
- Root cause: `run_headless` predates the streaming/event-bus run loop and was
  never upgraded.
- Direction: either (a) reimplement `run_headless` on top of
  `execute_stream`/`execute_stream_with_cancel` plus an optional `RunStore`,
  yielding a streaming variant `run_headless_stream` that emits
  `HeadlessEvent` (= `AgentEvent` + final summary), or (b) document
  `run_headless` as a minimal one-shot helper and point CI consumers at the
  streaming API. Recommend (a); the current implementation is a public API
  that mis-advertises equivalence.
- Regression validation: a test that runs `run_headless_stream` against a
  `MockLlmClient` returning one tool call, and asserts the consumer receives
  `ToolCall`/`ToolResult`/terminal events and a `Run` is persisted when a
  `RunStore` is attached.
- Validation reports: [V02-01](../validations/F-OPS-01/V02-01.md)

### F-OPS-01-P1-03: Secrets are persisted into `JsonlRunStore` via unredacted `Run.input` / `final_output` / `ToolResult.output_preview` / `ToolError.message`

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/trace/mod.rs:73,79,83` (`Run.input`, `final_output`,
    `error` typed as plain `String`).
  - `echo-agent/src/agent/react/mod.rs:1934,1964,1989` (assigned verbatim from
    user prompt / agent output / agent error).
  - `echo-agent/src/agent/react/run/pipeline.rs:834-849` (ToolResult preview
    is `chars().take(200)`, no redact), `pipeline.rs:864` (ToolError message
    verbatim).
  - `echo-agent/src/trace/mod.rs:425-444` (`new_tool_call` is the **only**
    site that calls `redact_secrets`).
- Reachability: any agent constructed with `with_run_store(...)` and the
  default `JsonlRunStore` writes these fields to
  `~/.echo-agent/.../{run_id}.jsonl` on every `save`. `ReactAgentBuilder`'s
  default wiring persists on `start_trace_run` and again on `finalize_run`.
- Expected invariant (TASKS.md V03: "Are secrets redacted from traces?"):
  user-supplied text and tool output flowing into a persistent trace must be
  scrubbed with the same redactor used for tool args, or the trace store must
  apply it on save.
- Observed behavior: only `RunEvent::ToolCall.args` is redacted. If a user
  pastes "rotate AWS key AKIA… " into the prompt, or a `read_file` tool
  returns a `.env` file, the secret lands in `Run.input` /
  `ToolResult.output_preview` and is written to disk in plaintext.
  `Run.input` is also surfaced via `RunSummary.input_preview` in
  `list_all`/`list_by_session` responses.
- Impact: secret material is written to local JSONL files in plaintext. Per
  AGENTS.md the local-assistant threat model still requires
  "本地也成立的通用安全(如不把密钥打进日志)" — secret-in-trace-file is exactly
  that category. The product's own redaction helper exists and is used for
  tool args; the gap is its non-application to other persisted fields.
- Root cause: `redact_secrets` was wired into `RunEvent::new_tool_call` only;
  the `Run` struct fields and the other `RunEvent` variants were never plumbed
  through it.
- Direction:
  - Easiest correct fix: in `JsonlRunStore::save` (and `InMemoryRunStore::save`
    for consistency), run the serialized payload through `redact_secrets`
    before write. This catches all current and future fields at the
    persistence boundary.
  - Or field-level: add a `Run::redact()` constructor that applies the
    redactor to `input`, `final_output`, `error`, and the
    `ToolResult.output_preview` / `ToolError.message` variants, and call it
    at the React finalize boundary. Field-level preserves JSON validity for
    nested `args` JSON values.
- Regression validation:
  - Unit test: build a `Run` whose `input` contains a synthetic OpenAI key
    (`sk-` + 30 chars), save through `JsonlRunStore`, reload, assert the key
    substring is absent and `[REDACTED:` is present.
  - Same for `ToolResult.output_preview` containing a GitHub PAT.
- Validation reports: [V03-01](../validations/F-OPS-01/V03-01.md)

### F-OPS-01-P2-01: `JsonlRunStore` and `InMemoryRunStore` have no size bound or retention

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/trace/mod.rs:733-750` (`save` appends a full JSON line
    every call; no `max_file_size`, no rotation, no compaction).
  - `echo-agent/src/trace/mod.rs:677-681` (`cache: RwLock<HashMap<String, Run>>`,
    no eviction).
  - `echo-agent/src/trace/mod.rs:605-625` (`InMemoryRunStore::runs` HashMap,
    no eviction, no `len()` cap).
  - `RunEvent::LlmCall` records `cache_fingerprint`, `context_breakdown`,
    `messages` count, etc. — every iteration appends; long sessions produce
    multi-MB JSONL files.
- Reachability: `JsonlRunStore` is the documented "production" backend
  (`trace/mod.rs:561,676`). The CLI does not currently wire a `RunStore` into
  the ReactAgent by default (verified: `with_run_store` is only set in tests
  and examples), so this is a latent framework hazard rather than a current
  EKO disk-growth incident. Any consumer (including a future EKO feature)
  that turns the trace store on inherits unbounded growth.
- Expected invariant (TASKS.md V03: "Is trace size bounded?").
- Observed behavior: a single long-running run with many LLM iterations can
  grow its JSONL file without limit (one full-`Run` JSON line per event
  append — the file size is roughly O(N²) in event count because each append
  serializes the cumulative event list). The in-memory cache duplicates the
  full `Run` for every `run_id`.
- Impact: disk exhaustion and slow reloads for any consumer that enables the
  trace store on long sessions. The O(N²) write pattern compounds the
  problem.
- Root cause: `save` always rewrites the entire run as a new line; no
  retention policy; no compaction; no `max_runs`/`max_bytes` knob on the
  store.
- Direction:
  - Make `append_event` actually append a single event line (the trait
    already has `append_event`; `JsonlRunStore` overrides it but currently
    rewrites the whole run — `trace/mod.rs:793-801`).
  - Add `max_runs` and `max_file_bytes` to `JsonlRunStore::new` with sensible
    defaults; evict oldest on write.
  - Bound the in-memory cache with an LRU.
- Regression validation: a test that writes 10 000 events to one run and
  asserts file size stays below a configured cap, plus a multi-run test that
  asserts oldest runs are evicted at the `max_runs` threshold.
- Validation reports: [V03-01](../validations/F-OPS-01/V03-01.md)

### F-OPS-01-P2-02: `Metrics::record_*` are defined but never invoked; telemetry is dead-on-arrival

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - Definitions at `echo-agent/src/telemetry.rs:87,101,115,128,141`.
  - Grep `Metrics::record_llm|Metrics::record_tool|record_llm_call|record_tool_execution|record_llm_tokens|record_llm_latency|record_tool_latency` across `echo-agent/` and `echo-agent-cli/` returns only the five definition lines in `telemetry.rs` (no callers in the React loop, providers, or `echo-execution/src/tools.rs`).
  - `shutdown_telemetry` (`telemetry.rs:263`) likewise has no caller.
- Reachability: `pub fn record_*` are re-exported via prelude
  (`src/lib.rs:298`); they are public API. They are simply not called.
- Expected invariant (TASKS.md V04: "When telemetry feature is off, what
  happens? Is it cleanly gated?"). Gating is clean, but the gated module
  delivers no actual telemetry — defeats the purpose of the feature.
- Observed behavior: enabling the `telemetry` feature and pointing
  `OTEL_EXPORTER_OTLP_ENDPOINT` at a collector initializes an OTLP
  `SdkMeterProvider` with five instruments, none of which ever receive data.
  A collector operator sees `service.name="echo-agent"` attach and emit zero
  points. On shutdown, `shutdown_telemetry` is not called, so even span
  exports are best-effort.
- Impact: misleading capability — the framework claims OTLP metrics support
  that does not function. Operators waste time debugging "why is my collector
  empty." Consumers reading `telemetry.rs` reasonably assume wiring exists.
- Root cause: metrics instruments were added without the call-site wiring in
  the LLM provider client or the tool execution pipeline; shutdown hook was
  never hooked into the application's shutdown sequence.
- Direction:
  - Either wire `Metrics::record_llm_call` / `record_llm_latency` /
    `record_llm_tokens` into the neutral LLM provider trait (or the OpenAI /
    Anthropic adapters) and `Metrics::record_tool_execution` /
    `record_tool_latency` into `echo-execution/src/tools.rs` around tool
    dispatch; and call `shutdown_telemetry` from `AppState` shutdown (or
    document that the consumer must).
  - Or, if the framework deliberately leaves this to consumers, demote the
    `Metrics` API to a trait/extension point and stop publishing
    instrument-creation code that implies auto-collection. Recommend the
    wiring option: the call sites are small and the value (operator
    visibility into LLM cost) is concrete.
- Regression validation: a test that builds a `MockLlmClient`, drives one
  turn with `telemetry` on and a test-harness `MeterProvider`, and asserts
  `llm.calls` and `tool.executions` counters increment.
- Validation reports: [V04-01](../validations/F-OPS-01/V04-01.md)

### F-OPS-01-P2-03: Scheduler fires tasks serially and holds in-flight state without timeout

- Priority: P2
- Confidence: medium
- Layer: framework
- Evidence:
  - `echo-orchestration/src/scheduler/runner.rs:99-101` (`for task in to_fire
    { self.fire_task(task).await; }` — strictly serial).
  - `runner.rs:105-118` (`fire_task` awaits `fire_fn(task).await` with no
    `tokio::time::timeout`, no per-task cancel).
- Reachability: every scheduler tick; in EKO the `FireFn` dispatches to
  `launch_cron_run` which can run an unbounded agent task.
- Expected invariant: an operational scheduler should not let one slow task
  starve the schedule, and should bound each fire.
- Observed behavior: if two cron tasks become due in the same tick and the
  first triggers a 10-minute agent run, the second fires 10 minutes late
  (and may be skipped as outside the 30 s window on the next tick, depending
  on its schedule). A cron task whose `fire_fn` hangs indefinitely blocks the
  runner forever (the loop's `cancel.cancelled()` select arm is only checked
  between ticks, not during `fire_task`).
- Impact: missed cron fires and no cancellation observability for long cron
  runs. For a personal assistant the blast radius is small (the user's own
  schedules), but the scheduler is a generic framework API and a consumer
  running several cron tasks is impacted.
- Root cause: `fire_task` is a plain `.await` with no
  `tokio::time::timeout` / `tokio::select!` against the cancel token; tasks
  are awaited serially in the to_fire loop instead of `join_all` /
  `FuturesUnordered`.
- Direction:
  - Wrap `fire_fn(task).await` in `tokio::select!` against
    `self.cancel.cancelled()` and a configurable per-task timeout.
  - Spawn each `fire_task` onto its own `tokio::task` and collect handles
    (bounded by a semaphore if needed) so simultaneous due tasks run in
    parallel.
- Regression validation: a test with two tasks due in the same tick where
  the first `FireFn` sleeps 2 s; assert the second fires within e.g. 100 ms
  of the first being scheduled (not after it). Plus a timeout test where a
  `FireFn` that never resolves is aborted at the configured deadline.
- Validation reports: [V01-01](../validations/F-OPS-01/V01-01.md)

### F-OPS-01-P2-04: `CronTaskStore` panics on `current_thread` runtimes via `block_in_place`

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-orchestration/src/scheduler/cron_task.rs:132-134,164-166`:
    `tokio::task::block_in_place(|| rt.block_on(backend.get(...)))`.
  - `tokio::task::block_in_place` documents: "This function will panic if
    called from a current-thread runtime."
- Reachability: any consumer of the `Store`-backed `CronTaskStore` (i.e. the
  `with_store` constructor, used by `AppState::start_scheduler_with_store`
  in EKO). The default EKO `#[tokio::main]` is multi-thread, so the panic
  does not fire in production today; it fires for any consumer that builds a
  `current_thread` runtime (a common choice for embedded / single-purpose
  agents).
- Expected invariant: AGENTS.md "禁止任何会导致系统 panic 的 API".
- Observed behavior: `block_in_place` panics on a current-thread runtime.
  Additionally, `block_in_place(... rt.block_on(...))` is a sync->async
  bridge inside an otherwise-async trait surface, which can deadlock under
  nested scheduling.
- Impact: framework consumer using `current_thread` hits a panic the first
  time `SchedulerRunner::new` calls `store.load_all()`. EKO itself is
  unaffected because it uses the multi-thread runtime, so this is a
  framework-portability defect, not an EKO outage.
- Root cause: `CronTaskStore` exposes a synchronous
  `load_all/save_all -> Result` API but the underlying `Store` trait is
  async; the bridge uses the most ergonomic-but-panicky primitive.
- Direction:
  - Make `load_all` / `save_all` (and the dependent `add` / `remove` /
    `set_status` / `update_last_run`) `async fn` and drop `block_in_place`.
    `SchedulerRunner` is already async, so callers (`runner.rs:34`,
    `runner.rs:186-191` `reload`) adapt trivially.
  - If sync API must be retained for back-compat, gate `block_in_place`
    behind a runtime-handle probe and return `Err` rather than panicking
    when no multi-thread handle is present.
- Regression validation: a test that constructs a `current_thread` tokio
  runtime, builds a `CronTaskStore::with_store(...)`, and asserts
  `load_all()` returns `Err` (or succeeds under the async variant) rather
  than panicking.
- Validation reports: [V01-01](../validations/F-OPS-01/V01-01.md)

### F-OPS-01-P3-01: `CronTask::cron_expr` 5-field vs 7-field handling repeats unvalidated

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-orchestration/src/scheduler/cron_task.rs:62-81`: `next_run` and
    `validate_cron` both duplicate the 5->7 field padding logic
    (`split_whitespace().count() == 5`).
  - No shared helper; if a sixth field is passed, neither branch fires and
    parsing fails opaquely.
- Reachability: every cron task add/update.
- Expected invariant: small dedup; not a runtime defect.
- Impact: minor maintainability; a future change to padding logic must be
  made twice.
- Direction: extract `fn normalize_expr(&self) -> String` and call from both.
- Regression validation: existing `test_cron_task_invalid_expr` plus a test
  asserting a 6-field expression is rejected with a clear error.
- Validation reports: [V01-01](../validations/F-OPS-01/V01-01.md)

### F-OPS-01-P3-02: `CronTaskStore::remove` / `set_status` / `update_last_run` / `get` use ID prefix match inconsistently

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `cron_task.rs:188,213,229,241`: `remove`, `set_status`,
    `update_last_run`, `get` use `task.id.starts_with(id)`.
  - `cron_task.rs:200,142`: `remove_exact` and the new `remove_task_exact`
    use full equality.
- Reachability: every management call from the CLI/TUI scheduler commands.
- Expected invariant: ID lookup should be exact, or prefix-match should be
  consistently applied and documented.
- Observed behavior: prefix match lets `remove("abc")` delete `abc` and
  `abc-def` together; only `remove_exact` is tested
  (`remove_exact_does_not_remove_tasks_with_the_same_prefix`,
  `cron_task.rs:339`). The prefix-match path is untested and surprising.
- Impact: a caller passing a truncated ID silently mutates multiple tasks.
  Local-assistant risk is low but the API is a footgun for any framework
  consumer.
- Direction: deprecate prefix-match variants and route all callers through
  `remove_exact` / a new `get_exact`. If prefix match is retained, document
  it on the signature and add a test that asserts multi-match behavior.
- Regression validation: extend the existing `remove_exact` test with the
  inverse case (prefix `remove` removes both) and assert the documented
  behavior explicitly.
- Validation reports: [V01-01](../validations/F-OPS-01/V01-01.md)

### F-OPS-01-P3-03: `last_fired` HashMap grows without eviction

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-orchestration/src/scheduler/runner.rs:27` (`last_fired:
  Arc<RwLock<HashMap<String, DateTime<Utc>>>>`); entries are inserted in
  `tick` (line 90) but never removed, even when the task is deleted via
  `remove_task`.
- Reachability: long-running scheduler.
- Expected invariant: auxiliary state for deleted tasks should not linger.
- Observed behavior: deleted task IDs remain in `last_fired` forever. For
  EKO's small personal schedule this is a few hundred bytes; for a
  generic framework consumer with many cron tasks over a long uptime it is
  a slow leak.
- Impact: negligible in practice; flagged for completeness.
- Direction: in `remove_task` / `remove_task_exact`, also
  `last_fired.remove(&id)`.
- Regression validation: unit test that adds, fires, removes a task, and
  asserts `last_fired` no longer contains its id.
- Validation reports: [V01-01](../validations/F-OPS-01/V01-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Scheduler trigger lifecycle: triggers, cancellation, shutdown, runtime-portability inspection of `scheduler/runner.rs` + `cron_task.rs` + EKO wiring | yes | passed (read-only static; supports F-OPS-01-P1-01, P2-03, P2-04, P3-01/02/03) | [V01-01](../validations/F-OPS-01/V01-01.md) |
| V02 | Headless event contract: confirm `run_headless` produces no `AgentEvent` stream / no RunStore / no cancel; live-caller inventory | yes | passed (supports F-OPS-01-P1-02) | [V02-01](../validations/F-OPS-01/V02-01.md) |
| V03 | Trace redaction + size inspection: redaction coverage of Run fields; `JsonlRunStore` size bound / retention absence | yes | passed (supports F-OPS-01-P1-03, P2-01) | [V03-01](../validations/F-OPS-01/V03-01.md) |
| V04 | Telemetry-disabled behavior: feature gate cleanliness + dead metrics wiring + missing shutdown hook | yes | passed (supports F-OPS-01-P2-02) | [V04-01](../validations/F-OPS-01/V04-01.md) |
| V05 | Historical-document drift | conditional | not_applicable — no historical claim hinges on this slice; phase annotations inside `scheduler/runner.rs` (`Phase 3.1`, `Phase C`) are summarised under the Current Path and reflected as the live code, not as historical claims requiring revalidation. |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `scheduler/runner.rs` header comment "Phase 3.1: ALL cron tasks route through the unified TaskRuntime executor" | current | `echo-agent-cli/echo-agent-app-core/src/scheduler/runner.rs:7-11,47-101` confirms `build_fire_fn` always calls `launch_cron_run`; the legacy `execute_direct` path is removed. |
| `scheduler/runner.rs` "Phase C: cron now runs on a POOL-ACQUIRED per-run agent" | current | `echo-agent-cli/echo-agent-app-core/src/scheduler/runner.rs:42-46,86-98` confirms pool acquire/release around each cron run. |
| `infra.rs` "Phase 3.1+: cron → launch_cron_run (unified)" | current | `echo-agent-cli/echo-agent-app-core/src/scheduler/runner.rs:100-101` is the live call. |
| `telemetry.rs` doc claim "provides OTLP export configuration and initialization functions ... recording key indicators such as LLM calls, Token usage, and tool execution" | **regressed/misleading** | The init functions exist and work; the recording helpers (`Metrics::record_*`) are defined but have zero callers (see F-OPS-01-P2-02). The documented behavior does not occur at runtime. |

## Coverage And Uncertainty

Inspected but not executed:

- No `cargo test` was run for this task — it is a static review. All findings
  are based on source reads + grep verification. The relevant dynamic tests
  (scheduler existing tests, `JsonlRunStore` round-trip tests, telemetry
  smoke) are owned by Q-FW-01 / Q-FW-02 and are out of scope to reproduce
  here.
- `trace/analyzer.rs` was inspected only for output sizing
  (`fields.truncate(24)`, `analyzer.rs:754`); its analytical correctness is
  out of scope.
- `echo-state/src/audit/file.rs` and `memory.rs` were inspected only for
  redaction / size concerns; both truncate to `limit` on read
  (`file.rs:97`, `memory.rs:95`) but neither applies `redact_secrets` on
  write — same pattern as `JsonlRunStore`. Flagged here only if a future
  audit-task touches them; not promoted to a finding because audit logging
  is opt-in and the framework's `AuditEvent` is caller-supplied.

Uncertain claims:

- The exact runtime behaviour of `tokio::task::block_in_place` under EKO's
  specific tokio configuration (multi-thread, worker count) is taken from
  tokio's documented contract; no runtime probe was performed. Confidence is
  high based on docs.
- Whether any `echo-agent-cli` GUI/TUI mode attaches a `RunStore` to the
  primary agent was checked via grep for `with_run_store` / `set_run_store`
  in the CLI tree; only tests/examples matched. If a future EKO change turns
  the trace store on, F-OPS-01-P1-03 and P2-01 become live defects rather
  than latent ones.

Residual risk:

- The scheduler's in-flight `fire_task` cancellation behaviour depends on
  tokio's cooperative cancellation semantics through `launch_cron_run`; this
  was not traced end-to-end here (owned by A-TSK-03 / A-TSK-04).

## Handoff

Conclusions downstream tasks may rely on:

- The scheduler framework (`SchedulerRunner` / `CronTaskStore`) is a sound
  generic primitive but is missing graceful shutdown (P1-01), per-task
  timeout / concurrency (P2-03), and `current_thread` safety (P2-04). Any
  task that depends on scheduler reliability (A-SRF-04, A-BOOT-01 shutdown
  sequencing, X-SRF-01 mode parity) should consume these findings rather
  than re-derive them.
- `run_headless` is **not** event-equivalent to interactive mode (P1-02).
  X-SRF-01 / A-SRF-04 should treat headless as a deficient path until
  fixed, and any "all modes equivalent" claim must explicitly exclude
  `run_headless`.
- The trace store (`JsonlRunStore`) is the canonical persistence boundary
  for `Run` records and is the right place to enforce redaction (P1-03)
  and size bounds (P2-01). F-MEM-01 / A-STATE-01 should consume the
  redaction finding when reasoning about persisted conversation/trace
  hygiene.
- The telemetry module is feature-gated cleanly (positive) but functionally
  inert (P2-02). Q-FW-02 / A-OBS-01 should not assume OTLP metrics work
  today.

Reports they must read:

- This task report and V01-01..V04-01.
- `zcode-glm/tasks/B-PATH-01.md` for the entry-point / composition-root
  context of the scheduler wiring.
- `zcode-glm/tasks/Q-STA-01.md` for the broader panic-safety context
  around `block_in_place` and `unwrap`-style patterns.

Conditions that make this report stale:

- Any change to `runner.rs:45-65` (spawn/run_loop) or the application's
  shutdown sequence that introduces cancellation/join semantics.
- Any change to `headless.rs` that introduces streaming / RunStore
  integration.
- Any change to `trace/mod.rs:733-801` (`JsonlRunStore::save`) or
  `react/mod.rs:1917-1942` (Run construction) that applies `redact_secrets`
  at the persistence boundary.
- Any new caller of `Metrics::record_*` or `shutdown_telemetry`.

Follow-up task IDs (do not implement in this review):

- A-BOOT-01: own the scheduler shutdown sequencing fix (consume P1-01).
- A-SRF-04: re-evaluate CLI/channel/cron trigger parity once headless is
  upgraded (consume P1-02, P2-03).
- X-SRF-01: include headless in the surface-parity matrix and mark it
  deficient today.
- A-OBS-01: wire the OTLP metrics call sites and shutdown hook (consume
  P2-02), and reason about whether the wiring belongs in the framework
  (recommended) or in an EKO adapter.
- F-SEC-01: consume P1-03 (trace secret leakage) when reasoning about the
  local-data secret boundary; the same redactor that protects tool args
  must protect the trace store.
