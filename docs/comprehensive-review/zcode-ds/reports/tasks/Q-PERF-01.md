# Q-PERF-01: Performance And Resource-Lifecycle Audit

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean (both repositories, verified via `git status --porcelain`)

## Question

Where can prompt assembly, event fanout, persistence, DAG execution, frontend
reducers, logs/artifacts, locks, tasks, processes, or caches grow without
bound or block critical execution?

**Core conclusion: the unbounded-growth inventory.** All growth points below
that were already archived are referenced by canonical ID; four are new
(Q-PERF-01-P2-01, P3-01, P3-02, P3-03). The two dominant paths are the
TaskRuntime file-shadow full rebuild per state event (P2-01) and the two
O(N²)/unbounded-payload persistence paths (canonical F-OPS-01-P2-02,
A-TSK-06-P3-02), both now measured against real `~/.eko` data (V02-01).
Everything else is either bounded or a conditional low-impact gap (P3).

## Scope

- Prompt assembly / context: framework ReAct loop (react_loop.rs,
  stream_channel.rs, pipeline.rs), compression (echo-state/compression),
  EKO compact_context.rs, loop/budget bounds.
- Event fanout: framework stream channel (stream_macros.rs, stream_channel.rs),
  EKO ChannelChatSink/TUI channels, HookEventDispatcher.
- Persistence: JsonlRunStore (trace/mod.rs), TaskRuntime file shadow
  (file_shadow.rs, store.rs, ledger.rs), FileConversationStore, runtime
  checkpoints (state/file.rs), tool artifacts (echo-core/tools/artifact.rs).
- DAG execution: framework RuntimeDagExecutor, EKO executor adapter,
  scheduler/runner.rs, AgentPool lifecycle.
- Frontend reducers: chatStore, taskRuntimeStore, subagentRunStore,
  conversationStore, toolExecutionStore.
- Locks: per-run plan locks, store mutexes, pool RwLock, handoff mutex.

## Out Of Scope

- Q-FLT-01/02 fault injection (separate tasks).
- Q-E2E-01 end-to-end scenarios.
- Deep log-rotation audit of tracing (operator-visible; noted in V04).
- Real-network LLM latency benchmarks (no credentials/network bench harness).

## Inputs

- Repository root `AGENTS.md` (full).
- `docs/comprehensive-review/README.md`, `REPORTING.md`, `TASKS.md`
  (Q-PERF-01 card only), `zcode-ds/README.md`.
- Dependency reports (performance-relevant findings): F-RCT-02/03/04,
  F-CMP-01, F-OPS-01, F-SUB-02, F-MAG-01, F-MEM-01/02, F-INT-02, F-HITL-01,
  A-TSK-01/03/04/05/06, A-STATE-01, X-STA-01, X-EVT-01, A-FE-02/03,
  Q-STA-01, Q-WEB-01, A-SRF-03/04.
- Historical documents treated as hypotheses; every canonical finding
  re-verified against current code (V05-01).

## Layering Decision

| Classification | Answer |
|---|---|
| Generic mechanism | Framework owns: bounded stream channels, compression windows, RunStore, ConversationStore, DAG wave executor, artifact spill/cleanup. |
| EKO product policy | TUI/CLI unbounded fanout choice, TaskRuntime file shadow rebuild cadence, `run_budget: None` invocation policy, pool lifecycle. |
| Adapter boundary | EKO executor adapter (executor.rs) is thin over `RuntimeDagController`; store/write policy stays in app-core. |
| Duplicate search | Per-path searches: `rewrite_plan`, `save_messages`, `UnboundedSender`, `append_event`, cache eviction, `max_iterations`, retention/cleanup. Results in V05-01; no duplicates found for the four new IDs. |
| Migration deletion | N/A (no authority movement proposed; fixes belong to the roadmap). |

## Current Path

Verified data flows (anchors in findings/validation reports):

1. **ReAct loop** (stream_channel.rs:521-527): `max_iterations=0` means
   `usize::MAX`; wind-down budget gated on `max_iterations > 0`; EKO default
   0 (config.rs:469) and `run_budget: None` (chat_driver.rs:509) → loop
   bounded only by model behavior. LoopDetector never consulted (canonical
   F-RCT-02-P2-02). Per-iteration trace events: LlmCall/ToolCall/ToolResult/
   BudgetDecision → `record_event` → `append_event` (full-run rewrite).
2. **Event fanout**: framework stream bounded + drop-on-full (F-RCT-03-P1-01);
   EKO `ChannelChatSink`/TUI channels unbounded (new P3-01); HookEventDispatcher
   bounded with backpressure.
3. **Persistence**: JsonlRunStore save=full-run line (F-OPS-01-P2-02, measured
   O(N²)); TaskRuntime shadow appends event + rebuilds plan.json/run-state.json
   from the full event stream per state event under the per-run lock (new
   P2-01); FileConversationStore rewrites the full record per save (new
   P3-02); JsonlRunStore cache never evicts (new P3-03).
4. **DAG execution**: framework wave executor semaphore-bounded
   (runtime_executor.rs:201-366); EKO executor adapter + store methods on
   the wave critical path (amplified by P2-01).
5. **Frontend reducers**: all arrays capped (chat 500, runtime events 500,
   subagent events 300); O(n²) render per token canonical (A-FE-03-P2-01).
6. **Processes/locks**: AgentPool bounded (max_agents + 5-min monitor +
   RAII release); per-run plan locks unbounded across runs (folded into
   P3-03); handoff mutex held across execution canonical (F-MAG-01-P3-02).

## Findings

### Q-PERF-01-P2-01: TaskRuntime file shadow rebuilds `plan.json` + `run-state.json` from the entire event stream on every state event — O(N²) cumulative I/O on the DAG executor's critical path

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_shadow.rs:208-280`
  (`rewrite_plan` → `read_events` full-file read + `rebuild_plan_from_events`
  from scratch), `:362-379` (`read_events` = whole `events.jsonl` into memory
  per call), `store.rs` 14 call sites (:354, :380, :422, :485, :669, :881,
  :1102, :1174, :1206, :1360, :1396, :1427, :1450, :1475).
- Reachability: `TaskRuntimeStore::set_task_status`/`transition_run`/
  `apply_task_patch`/`create_run` → `shadow.rewrite_plan` → full read+rebuild,
  all inside `with_run_lock` (store.rs:276-293). Executor waves call store
  methods between dispatches (executor.rs), so every state transition on the
  run's critical path pays the full-stream cost.
- Expected invariant: state-event persistence cost is proportional to the
  event being appended; projections update incrementally.
- Observed behavior: every state-affecting event re-reads and re-parses the
  whole events.jsonl and rewrites both projection files with fsync. A run
  with 46 events / 140 KB (real data, one `subagent_released` line = 33.9 KB
  with `full_output`+`result`) pays a full read+rebuild per transition;
  larger runs pay linearly more per transition → O(N²) per run, executed
  synchronously under the per-run write lock.
- Impact: long TaskRuntime runs (many tasks/subagents, large outputs)
  progressively slow every state transition; the DAG executor stalls on
  synchronous file I/O; concurrent writes to other runs are independent
  (per-run locks) but the same run's hook dispatch and status progression
  serialize behind the rebuild.
- Root cause: event-sourced file authority implemented as "append event then
  rebuild all projections from the full stream on every write"; no
  incremental projection or cached read (the seq cache only avoids line
  counting, not the rebuild).
- Direction: keep an incremental in-memory projection that is replayed once
  at store open (from the event log) and updated per append; rewrite
  plan.json/run-state.json only at safe points or from the in-memory state;
  bound `full_output`/`result` payloads (ties into A-TSK-06-P3-02). Delete
  the full-stream `rewrite_plan` path once the incremental one is in.
- Regression validation: a synthetic run with 500 events must show per-
  transition cost ~constant (not growing with file size); parity test
  `shadow_parity_after_full_lifecycle` must still pass; executor wave timing
  on a 50-task run with 100 KB outputs.
- Validation reports: [V01-01](../validations/Q-PERF-01/V01-01.md),
  [V02-01](../validations/Q-PERF-01/V02-01.md),
  [V04-01](../validations/Q-PERF-01/V04-01.md)

### Q-PERF-01-P3-01: EKO TUI/CLI event fanout uses unbounded mpsc channels — no backpressure anywhere on the product fanout path

- Priority: P3
- Confidence: high (definition), medium (impact — requires a slow consumer)
- Layer: application
- Evidence:
  `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:575-591`
  (`ChannelChatSink { UnboundedSender<ChatDriverEvent> }`); consumers at
  `echo-agent-cli/src/cli/repl.rs:494`, `src/cli/channels.rs:246`,
  `chat_driver.rs:1129`; eight `UnboundedSender<AgentEvent>` in
  `src/tui/events.rs:1042,1225,1297,1344,1479,2022,2615`.
- Reachability: `drive_chat` (chat_driver.rs:538-566) forwards every streamed
  event through `ChannelChatSink`; TUI/REPL/channels consume from the
  unbounded receiver. Framework side is bounded-with-drop (F-RCT-03-P1-01);
  the EKO side deliberately chose unbounded.
- Expected invariant: event fanout is bounded; a slow consumer must not
  accumulate memory without limit (boundedness chosen on the framework side
  for the same producer stream).
- Observed behavior: when the consumer is alive but slow (blocking terminal
  writes over SSH/Windows Terminal, paused renderer), the unbounded queue
  retains every event — including `Token` events that carry whole buffered
  responses (F-RCT-03-P2-01) — with no cap.
- Impact: memory growth proportional to producer-consumer skew for the
  duration of the stream; in the extreme, a long output to a stalled
  terminal accumulates the full transcript in RAM.
- Root cause: backpressure was never designed into the product fanout;
  `on_event` only returns false when the receiver is *dropped*, not when it
  is slow.
- Direction: bounded channel + drop-with-warning policy (matching the
  framework), or a coalescing/renderer-side bounded buffer; document the
  chosen trade-off. Keep `on_event -> bool` semantics for closed receivers.
- Regression validation: simulate a slow consumer (sleep in the render loop)
  and assert bounded memory on a 10k-event stream; TUI smoke test unchanged.
- Validation reports: [V01-01](../validations/Q-PERF-01/V01-01.md),
  [V03-01](../validations/Q-PERF-01/V03-01.md)

### Q-PERF-01-P3-02: `FileConversationStore` rewrites the entire conversation record per message save — O(N²) cumulative disk I/O per conversation; list/search read every record

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  `echo-agent/echo-state/src/memory/file_conversation.rs:350-376`
  (`save_messages` → read whole record → replace `messages` → `write_record`
  full-file atomic rewrite), `:215-241` (`read_all_records` on
  `list_conversations` and `search_conversations`), `:168-175`
  (`write_record` serializes all messages).
- Reachability: EKO conversation save path (infra.rs / conversation_file.rs
  reindex) and any framework consumer using the file backend; save happens
  per message batch (per turn).
- Expected invariant: per-message persistence cost is O(1)-ish; a
  conversation with M messages does not rewrite M× the history.
- Observed behavior: each save writes the full record (all messages),
  fsync'd; with M messages of average size S the cumulative I/O is
  O(S·M²). `list_conversations`/`search` read every conversation file on
  every call.
- Impact: long-lived conversations (thousands of messages) make every turn
  slow (multi-MB fsync'd rewrites); session search degrades with total
  conversation count. Local-scale efficiency, hence P3.
- Root cause: file-per-conversation with whole-record JSON and no
  append/delta or cached write coalescing.
- Direction: append per message to a JSONL per conversation with a periodic
  compaction, or cache the record in memory and write-through with
  coalescing; keep the atomic-rename robustness of the current write.
- Regression validation: 2000-message conversation — average save time
  should not scale with message count; corruption-detection tests
  (F-MEM-01 family) must still pass.
- Validation reports: [V01-01](../validations/Q-PERF-01/V01-01.md),
  [V04-01](../validations/Q-PERF-01/V04-01.md)

### Q-PERF-01-P3-03: `JsonlRunStore` in-memory cache and `FileTaskShadow` per-run maps never evict — RAM grows with every run ever saved in the process lifetime

- Priority: P3
- Confidence: high
- Layer: framework (cache), application (shadow maps)
- Evidence:
  `echo-agent/src/trace/mod.rs:677-708` (cache `HashMap<run_id, Run>`,
  populated at open for every existing file), `:748` (insert on every save,
  never removed), `:752-769` (load always reads cache);
  `echo-agent-cli/.../file_shadow.rs:26-34` (seq_cache and run_write_locks
  maps), `:185` ("entries are never removed on Drop ... bounded by total
  runs ever written").
- Reachability: EKO attaches a JsonlRunStore to every chat/task agent
  (infra.rs:376-379); every turn starts a scoped trace run
  (react_loop.rs:546, stream_channel.rs:100/109) and saves it at least once;
  a long-lived GUI/TUI process accumulates one full `Run` (with all events)
  per turn in RAM, plus one `plan_locks`/`seq_cache` entry per run.
- Expected invariant: caches have an eviction bound; completed runs leave
  memory after a bounded time.
- Observed behavior: no eviction, no cap, no LRU; cache growth is linear in
  total runs ever executed in-process. Note: this is the RAM-side aspect of
  the canonical F-OPS-01-P2-02 (which files the disk O(N²) + no-retention
  side); filed separately because the impact surface (memory, not disk) and
  fix (eviction) differ.
- Impact: for a local personal assistant running days/weeks with frequent
  turns, cache growth is slow but unbounded; each entry includes the full
  event list of its run.
- Root cause: cache-as-grow-only-map design with no eviction policy.
- Direction: evict completed runs older than a bound (e.g., LRU or
  last-write time) or cap total cached bytes; shadow maps can be cleared of
  entries for terminal runs.
- Regression validation: long-running-process test saving 1000 runs must
  keep cache size bounded; `load_last_line` cold-path behavior preserved.
- Validation reports: [V01-01](../validations/Q-PERF-01/V01-01.md),
  [V04-01](../validations/Q-PERF-01/V04-01.md)

## Canonical Finding Matrix (archived, cross-verified current)

| Canonical ID | Growth/unbound aspect | Status |
|---|---|---|
| F-OPS-01-P2-02 | JsonlRunStore O(N²) disk, no caps/retention | current, measured (V02-01) |
| A-TSK-06-P3-02 | `full_output`/`result` unbounded in events.jsonl; CWD-relative trace archives | current, measured (V02-01) |
| F-CMP-01-P1-01 | compression windows never bound tokens | current |
| F-CMP-01-P1-02 | immortal system summary appended per pass | current |
| F-CMP-01-P1-03 | adaptive L1 fold breaks tool-pairing contiguity | current |
| F-RCT-02-P2-02 | LoopDetector dead; unlimited-iteration agents | current |
| F-RCT-03-P1-01 | bounded channel drops events incl. terminal | current |
| F-RCT-03-P2-01 | whole response buffered, single Token burst | current |
| F-RCT-03-P2-04 | abandoned streams leave trace runs Running | current |
| F-RCT-04-P3-01/02 | batch concurrency unbounded default; timer exemptions | current |
| F-SUB-02-P1-01/02 | Team cancel/timeout detach members | current |
| F-MAG-01-P1-01 | handoff detached uncancellable target | current |
| F-MAG-01-P3-02 | handoff manager mutex held across execution | current |
| F-INT-02-P1-01..03 | LSP blocks shutdown; QQ busy-spin; A2A cancel lost | current |
| F-HITL-01-P1-02 / A-HITL-01-P1-02 | blocking read_line defeats deadline | current |
| F-OPS-01-P3-04 | scheduler fires unbounded/serial; cancel never reaches in-flight fires; EKO never cancels scheduler token | current |
| X-EVT-01-P1-01/02 | cancel/timeout class lost at envelope; timed-out turns end 'completed' | current |
| A-SRF-03-P1-01 | interrupt_prompt strands frontend turn state | current |
| A-SRF-04-P1-01 | REPL/channel turns not cancellable | current |
| A-TSK-03-P2-01 / A-TSK-04-P1-02 | mid-wave store fault orphans Running claims | current |
| A-TSK-05-P2-02 | no lifecycle sweep for leaked fork worktrees | current |
| A-TSK-01-P1-01 | torn tail in events.jsonl bricks run | current |
| X-STA-01-P1-01 | deletion leaves runtime transcript + plan on disk | current |
| F-MEM-01-P1-01 | corrupt store file silently overwritten | current |
| A-STATE-01-P2-02 | no cross-process write serialization | current |
| A-FE-03-P2-01 | O(n²) per-token Set construction, no windowing | current |
| A-FE-03-P3-04 | MAX_MESSAGES=500 cap has zero direct unit tests | current |
| A-FE-03-P3-05 | authStore interval + focus listener never cleaned | current |
| Q-STA-01-P3-04 | oversized modules (31 files > 1500 lines) | current |
| A-TSK-06-P3-03 | parent summary chains: per-item bound, no aggregate bound | current |

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---|---|---|
| V01 | Static allocation/lock/task-lifecycle trace (both repos) | yes | passed | [V01-01](validations/Q-PERF-01/V01-01.md) |
| V02 | Representative large-input measurement (real `~/.eko` data + static estimates + `cargo test -p echo_agent --lib trace`) | yes | passed | [V02-01](validations/Q-PERF-01/V02-01.md) |
| V03 | Cancellation cleanup aggregation + resource-leak sweep | yes | passed | [V03-01](validations/Q-PERF-01/V03-01.md) |
| V04 | Disk/cache growth analysis | yes | passed | [V04-01](validations/Q-PERF-01/V04-01.md) |
| V05 | Cross-check with existing findings (canonical matrix) | yes | passed | [V05-01](validations/Q-PERF-01/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `JsonlRunStore` doc "Append a single event ... without rewriting the entire run" (trace/mod.rs:586-596) | stale | `append_event` (trace/mod.rs:793-801) rewrites the full run; matches canonical F-OPS-01-P2-02 |
| scheduler/runner.rs:103-105 "drive_run_async notably does NOT release — a pre-existing minor leak" | stale | run_driver.rs:100 now calls `pool.release`; comment is doc drift only |
| All dependency-report findings in the canonical matrix | current | re-verified at current commits (V05-01) |

## Coverage And Uncertainty

- EKO tracing/log rotation not inspected in depth (V04 deviation); low
  relevance for a local single-user app.
- The `~/.eko` measurements are a single-machine sample (earlier
  development runs), not a controlled benchmark; they confirm growth shape,
  not worst-case magnitude.
- Frontend O(n²) rendering is canonical (A-FE-03-P2-01 with its own
  measurement); not re-benchmarked here.
- No controlled perf benchmark was added (forbidden: read-only review,
  no new test code). V02 used existing tests + real data only.

## Handoff

- Downstream tasks may rely on: the unbounded-growth inventory (4 new
  findings + 31 canonical), the measured O(N²) persistence behavior, and the
  bounded/unbounded classification of every fanout/pool/store path.
- Reports to read: this task report; validations V01-01..V05-01; canonical
  F-OPS-01-P2-02, A-TSK-06-P3-02, F-CMP-01-*, F-RCT-02/03, F-SUB-02,
  X-EVT-01, A-SRF-03/04.
- Stale triggers: any change to file_shadow.rs rewrite cadence, store.rs
  call sites, chat_driver sink construction, FileConversationStore write
  path, or trace cache eviction invalidates the corresponding finding; the
  scheduler/runner.rs comment should be fixed opportunistically.
- Follow-up: S-RDM-01 roadmap items (perf bucket): P2-01 first (executor
  critical path), then P3-01/P3-02/P3-03; canonical F-OPS-01-P2-02 and
  A-TSK-06-P3-02 fixes are prerequisites for the P2-01 payload-bounding
  direction.
