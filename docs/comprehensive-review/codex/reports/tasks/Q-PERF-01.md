# Q-PERF-01: Performance and resource-lifecycle audit

> Status: complete
> Reviewer: Codex review subagent
> Executor: Codex review subagent
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: framework had unrelated external source changes and was inspected only through committed Git objects; CLI had only unrelated `Cargo.lock` modification, excluded

## Question

Where can prompt assembly, event fanout, persistence, DAG execution, frontend reducers, logs/artifacts, locks, tasks, processes, or caches grow without bound or block critical execution?

## Scope

- Static cross-layer allocation, persistence, lock, task/thread/process, cache, and retention trace.
- Independent committed-source verification of new TaskRuntime, hook-dispatch, and EKO logging causes.
- Deduplication against completed atomic owners for prompt/ReAct, DAG, persistence/recovery, frontend, observability, artifacts, integrations, evolution, plugins, and Skills.
- Existing-test coverage inventory and future dynamic validation definitions.

## Out Of Scope

- Cargo, rustc, tests, builds, Clippy, frontend commands, dynamic fixtures, network, profiling, or benchmarks, explicitly prohibited.
- Source, lock-file, documentation-index, or shared-status changes.
- Renumbering atomic defects solely because their impact is performance-related.
- Removing useful framework APIs because EKO does not consume them; adding cloud-service permission gates; enabling SQLite in EKO.

## Inputs

- Root `AGENTS.md`, shared README/REPORTING/TASKS exact card, and Codex README.
- Codex F-CTX-01, F-CMP-01, F-RCT-03, F-TSK-01..03, A-TSK-01..04, A-STATE-01, X-STA-01, A-FE-02/03, X-EVT-01, F-OPS-01, A-EVO-01, A-OUT-01, F-SKL-01, F-INT-01/02, A-OBS-01, and A-PLG-01.
- Only committed source at the pinned revisions; no other reviewer directory was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Framework context/event/hook budgets, bounded queues, cancellation, process ownership, and reusable store retention contracts stay in `echo-agent`. |
| EKO product policy | TaskRuntime event authority, concrete retention periods/byte caps, `~/.eko` log policy, surface projection costs, and local-assistant operational defaults belong in the application. |
| Adapter boundary | EKO adapters may translate durable events into hook/UI events, but must not hold authoritative state locks while framework extensions execute. Delivery ordering needs a cursor/outbox, not a second lifecycle authority. |
| Duplicate search | Searched growth/retention/rotation/compact/remove/delete, queue/backpressure/send, lock/task/process/cache, full-log reads, fsync, reducer/filter/sort, prompt budgets, and live call paths across both repositories and authorized reports. |
| Migration deletion | Replace full-replay-on-every-write and synchronous under-lock hook dispatch; delete the obsolete replay/cache paths only after one incremental authority and durable hook-delivery cursor are live. Keep reusable framework capabilities. |

## Current Data Flow

```text
TaskRuntime mutation
  -> same-run std::sync::Mutex
  -> append events.jsonl + fsync
  -> HookEventDispatcher::dispatch while lock is held
       -> bounded sync_channel(256)
       -> one consumer -> framework bridge -> serial configured hook actions
  -> release append lock
  -> rewrite_plan acquires same-run lock
       -> read whole events.jsonl -> deserialize/rebuild -> snapshot fsync

GUI logging
  -> tracing Stderr target
  -> stderr + ~/.eko/logs/app.log opened append across restarts
  -> no byte/age/count retention owner
```

## Findings

### Q-PERF-01-P1-01: Hook backpressure can block authoritative TaskRuntime writes under the same-run lock

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_shadow.rs:132-176`; `hook_event_dispatcher.rs:70-101,125-149,152-213`; `echo-agent/src/hooks_bridge.rs:90-100,116-126,158-171,226-239,273-286`; `echo-agent/echo-execution/src/skills/hooks.rs:131-200,217-307,786-836,995-1051`
- Reachability: every translated TaskRuntime lifecycle mutation reaches the file-authority append hook. The dispatcher is attached by application bootstrap, and the one consumer fires configured framework hooks.
- Expected invariant: authoritative persistence releases its lock before extension latency/backpressure; same-run cancellation and shutdown have finite progress independent of user hook speed.
- Observed behavior: the synchronous hook runs while `append_event_line` owns the run mutex. A bounded `SyncSender` deliberately blocks at 256 queued translations, while one consumer serially awaits all matching actions. MCP/subagent hooks accept timeout zero and then await indefinitely. Once saturated, later persistence, cancellation, flush, and shutdown can block behind the hook.
- Impact: a valid slow or stuck local extension can stop TaskRun progress and terminal persistence, cascading from an optional hook into the canonical Agent capability on every surface.
- Root cause: lossless hook ordering was coupled directly to the state transaction instead of represented by a durable post-commit delivery cursor/outbox with a separate lifecycle.
- Direction: commit state and a hook-delivery record atomically, release the run lock, then drain asynchronously in persisted order. Enforce finite action/cancellation deadlines and observable failed delivery. Delete the synchronous callback-under-lock path once the outbox is authoritative; do not silently drop lifecycle events.
- Regression validation: capacity+1 events with a never-resolving hook through the real file path; assert same-run cancellation and persistence remain bounded, restart resumes undelivered hooks exactly once/in order, and shutdown has a finite terminal.
- Validation reports: [V05](../validations/Q-PERF-01/V05-01.md), [V08](../validations/Q-PERF-01/V08-01.md), [V11](../validations/Q-PERF-01/V11-01.md)

### Q-PERF-01-P2-01: TaskRuntime mutation cost and retained state grow with the complete run history

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_shadow.rs:19-50,105-178,180-219,282-329,361-435`; `store.rs:380,422,485,881,1102,1174,1206,1360,1396`
- Reachability: production run/plan/task mutations append through the file authority; projection-affecting mutations then invoke `rewrite_plan`. The same service lives for application lifetime across all EKO surfaces.
- Expected invariant: incremental mutation cost is bounded independently of historical event count; terminal/historical run cache and disk retention have explicit caps.
- Observed behavior: `rewrite_plan` reads the entire JSONL file into one String, deserializes every event, rebuilds projections, writes and fsyncs snapshots. Repetition gives cumulative O(N^2) parse/allocation work for N projection mutations. `seq_cache` and `run_write_locks` keep one entry for every run ever touched and never evict; run directories have no retention/compaction owner.
- Impact: long TaskRuns become progressively slower on their critical state path, while long-lived sessions accumulate process memory and `~/.eko/tasks` disk state without a declared bound.
- Root cause: an append-only recovery log is also used as the per-mutation projection algorithm, and process/disk lifecycle was described as “bounded by total runs ever written” rather than assigned a finite policy.
- Direction: maintain one incremental projection/checkpoint with a crash-consistent cursor and compact archived event segments under explicit retention; evict per-run cache/lock entries only through a race-safe terminal/reference lifecycle. Delete full replay from normal mutation after checkpoint recovery is authoritative.
- Regression validation: geometrically increase events/runs; measure mutation latency, bytes read/written, allocations, fsyncs, resident cache entries, and retained bytes; assert bounded incremental slopes plus restart equivalence.
- Validation reports: [V03](../validations/Q-PERF-01/V03-01.md), [V04](../validations/Q-PERF-01/V04-01.md), [V08](../validations/Q-PERF-01/V08-01.md), [V10](../validations/Q-PERF-01/V10-01.md)

### Q-PERF-01-P2-02: GUI application logging has no automatic retention boundary

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/infra.rs:1574-1617,1625-1666`
- Reachability: GUI/Tauri uses the Stderr log target, which mirrors filtered tracing output into `~/.eko/logs/app.log`; every GUI restart reopens it in append mode.
- Expected invariant: persistent diagnostic logs rotate under a total byte/age/count cap without manual intervention.
- Observed behavior: the app log is append-only across restarts and the code explicitly delegates rotate/truncate to a human. No retention implementation exists. TUI truncates its separate file at startup, so surface resource behavior also differs.
- Impact: normal GUI operation can consume disk indefinitely and ultimately disrupt unrelated local application writes. Existing A-SRF-02 content-leak impact grows with the same retention gap but remains separately owned.
- Root cause: “rotating-ish” dual-sink intent was implemented as raw `OpenOptions::append` without a lifecycle component.
- Direction: introduce EKO-owned size/time rotation with a bounded aggregate and safe concurrent rename/reopen; apply canonical redaction before both sinks. Delete the manual-rotation contract.
- Regression validation: produce concurrent logs past each threshold, restart during rotation, assert no write failure, secret redaction, newest diagnostic availability, and total retained bytes within cap.
- Validation reports: [V06](../validations/Q-PERF-01/V06-01.md), [V08](../validations/Q-PERF-01/V08-01.md), [V10](../validations/Q-PERF-01/V10-01.md)

## Canonical Owner Matrix

| Resource family | Current owner/result |
|---|---|
| Prompt/context/compression | F-CTX-01, F-CMP-01; Skill catalog/body counts F-SKL-01-P2-06 |
| ReAct/event fanout and disconnect | F-RCT-03 and X-EVT-01 |
| Task/DAG retry, spin, cancellation | F-TSK-01..03 and A-TSK-01..04 |
| TaskRuntime recovery/corruption | A-TSK-01 and X-STA-01; Q-PERF owns only asymptotic/lifecycle cause above |
| Frontend reducers/projections/full output | A-FE-02-P2-05 and A-FE-03-P1-01/P2-02/P2-03 |
| Trace/audit stores | F-OPS-01-P0-02/P1-04 |
| Evidence history | A-EVO-01-P1-04 |
| Artifact content/lineage | A-OUT-01 and X-STA-01 |
| Detached webhooks/processes/integration readers | A-OBS-01 and F-INT-01/02 |
| Plugin hook generation | A-PLG-01-P1-03; Q-PERF owns the distinct under-lock backpressure path |

## Validation Matrix

| ID | Claim or execution | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Commit/dirty/isolation boundary | yes | passed | [V01](../validations/Q-PERF-01/V01-01.md) |
| V02 | Completed atomic-owner deduplication | yes | passed | [V02](../validations/Q-PERF-01/V02-01.md) |
| V03 | TaskRuntime projection replay complexity | yes | failed | [V03](../validations/Q-PERF-01/V03-01.md) |
| V04 | TaskRuntime memory/disk lifecycle | yes | failed | [V04](../validations/Q-PERF-01/V04-01.md) |
| V05 | Hook queue/lock/timeout lifecycle | yes | failed | [V05](../validations/Q-PERF-01/V05-01.md) |
| V06 | EKO log rotation/retention | yes | failed | [V06](../validations/Q-PERF-01/V06-01.md) |
| V07 | Cross-layer resource-family coverage | yes | passed | [V07](../validations/Q-PERF-01/V07-01.md) |
| V08 | Existing-test coverage inventory | yes | failed | [V08](../validations/Q-PERF-01/V08-01.md) |
| V09 | Historical/current dependency classification | yes | passed | [V09](../validations/Q-PERF-01/V09-01.md) |
| V10 | Representative large-fixture measurements | yes | not_run by explicit constraint | [V10](../validations/Q-PERF-01/V10-01.md) |
| V11 | Saturation cancellation/shutdown cleanup | yes | not_run by explicit constraint | [V11](../validations/Q-PERF-01/V11-01.md) |
| V12 | Exact ID/header/link/executor/isolation integrity | yes | passed | [V12](../validations/Q-PERF-01/V12-01.md) |
| V30 | Primary committed-source sampling and acceptance | yes | passed | [V30](../validations/Q-PERF-01/V30-01.md) |

## Historical Claim Status

| Claim | Classification | Evidence |
|---|---|---|
| Prompt/context can exceed or fail post-compression budget | current | F-CTX-01/F-CMP-01; V09 |
| Stream/event loss and disconnect cleanup defects | current | F-RCT-03/X-EVT-01; V09 |
| Trace/audit persistence is unbounded and quadratic | current | F-OPS-01-P0-02/P1-04; V02/V09 |
| Evidence JSONL rereads unbounded history | current | A-EVO-01-P1-04; V02/V09 |
| Frontend global scans and eager full-output retention | current | A-FE-02/03; V02/V09 |
| Hook queue preserves ordering by applying backpressure | current, but unsafe at persistence boundary | current source V05; A-PLG-01 owns generation semantics |

## Coverage And Uncertainty

- Static evidence proves topology, absent lifecycle owners, and asymptotic operations; it does not quantify user-visible latency, allocation size, disk rate, or cancellation duration.
- V10/V11 are deliberately `not_run` under the review constraint. They remain mandatory implementation regressions, but do not block the source-conclusive static lifecycle review after primary acceptance.
- Existing tests were only inventoried. No test result is claimed.
- Framework dirty source was excluded through committed-object reads; CLI `Cargo.lock` was excluded. Any change to the pinned paths makes this report stale.

## Handoff

- Primary reproduced V03-V06 statically in V30. Run V10/V11 when dynamic validation is allowed before accepting fixes.
- Fix order: isolate hook delivery from the TaskRuntime write lock (P1), make projection/checkpoint and run retention incremental/bounded (P2), then add GUI log rotation (P2).
- Preserve atomic owner IDs in the matrix; do not reopen prompt, DAG, frontend, trace/audit, Evidence, artifact, or integration findings under Q-PERF.
- The eventual design must keep TaskRun lifecycle authoritative, adapters thin, EKO local policy in the application, and framework extension APIs reusable.

## Primary Acceptance

The primary reviewer independently traced the same-run lock through event append
and bounded hook dispatch, full JSONL replay/snapshot rewrite and process-lifetime
run maps, and the GUI append-only log sink. The three finding roots, priorities,
and atomic-owner deduplication are accepted in V30.
