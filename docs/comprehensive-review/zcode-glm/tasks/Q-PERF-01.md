# Q-PERF-01: Performance and resource-lifecycle audit

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-13
> `echo-agent` commit: 3aa7929 (one `fix(tests)` commit ahead of the task
> card's `9b0e0fa`; all perf anchors re-verified at HEAD — see Coverage)
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (both repos `git status --short` empty)

## Question

Where can prompt assembly, event fanout, persistence, DAG execution,
frontend reducers, logs/artifacts, locks, tasks, processes, or caches grow
without bound or block critical execution?

## Scope

Primary source paths and behaviors inspected (read-only static trace +
analytical cost reasoning; no live model/network fixture):

- **Event fanout / streaming**: `echo-agent/src/agent/react/run/stream_channel.rs`
  (channel construction, detached core-loop spawn, max_iterations ceiling),
  `react_loop.rs`, `phases/*`.
- **Persistence / trace store**: `echo-agent/src/trace/mod.rs` (`JsonlRunStore`,
  `InMemoryRunStore`, `Run.events`), `echo-agent/src/agent/snapshot.rs`
  (`record_event`→`append_event`), `echo-state/src/memory/file_conversation.rs`
  (`FileConversationStore` full-rewrite path).
- **DAG / subagent execution**: `echo-orchestration/src/tasks/executor.rs`
  (semaphore, cancel, `running_tasks` DashMap), `background_task.rs`
  (`TaskSpawner.tasks` registry, `prune_completed`), `runtime.rs` (DAG cancel).
- **EKO application persistence**: `echo-agent-app-core/src/infra.rs` (live
  `JsonlRunStore` wiring, `app.log` writer), `persistence.rs` (`save_session`),
  `conversation_file.rs` (`SessionSearchEngine`), `tasks/task_runtime/file_shadow.rs`
  (`seq_cache`, `run_write_locks`, run-dir layout), `tool_execution.rs`
  (append-only journals), `evolution/evidence.rs`, `runtime.rs` (PROJECT.md).
- **Cancellation / process lifecycle**: `echo-execution/src/sandbox/local.rs`
  (child process spawn/kill/timeout), `agent_pool.rs` (idle eviction).
- **Frontend reducers/stores**: `web-frontend/src/stores/chatStore.ts`,
  `taskRuntimeStore.ts`, `subagentRunStore.ts` (cap inventory).

Consumed findings from prior reports: `F-CMP-01-P2-01` (summary
accumulation), `F-OPS-01-P2-01`/`P1-03` (trace store + secrets), `A-FE-03-P2-01`
(MessageBubble O(N·T)), `F-CTX-01-P2-01` (phantom budget).

## Out Of Scope

Deferred to named task IDs:

- **Correctness of compression/context budget** → `F-CMP-01`, `F-CTX-01`
  (consumed here only for the growth axis: summary accumulation is
  confirmed current, not re-derived).
- **Secret redaction in persisted runs** → `F-OPS-01-P1-03` (already
  established; not a perf axis).
- **Frontend re-render cost (O(N·T) MessageBubble)** → `A-FE-03-P2-01`
  (confirmed current; this task only notes the frontend *arrays* are
  bounded, which is the orthogonal growth axis).
- **Live end-to-end timing / fault injection** → `Q-FLT-01`, `Q-E2E-01`
  (this task is static + analytical; no model credentials available).
- **Dependency-tree duplicate cost / build time** → `Q-DEP-01`.

## Inputs

- Repository documents read: root `AGENTS.md` (UTF-8/panic constraints,
  framework-vs-application layering, framework-deletion rule, "EKO needs no
  SQLite"), `REPORTING.md`, both report templates, `TASKS.md` (Q-PERF-01
  entry only).
- Dependency task reports read: `F-RCT-02`, `F-CTX-01`, `F-CMP-01`,
  `F-OPS-01`, `A-FE-03` (findings sections, to consume and not duplicate).
- Historical documents treated as hypotheses: none beyond the consumed
  findings, each re-verified against current code (see Historical Claim
  Status).

## Layering Decision

This task spans both repositories; findings are classified per item.

- **Generic mechanism (framework)**: the `JsonlRunStore` write/read path
  (`trace/mod.rs`), the `TaskSpawner` registry (`background_task.rs`), the
  `max_iterations` ceiling, the `FileConversationStore` rewrite path, and
  the sandbox child-process cleanup are all framework API that any
  consumer inherits. A framework defect here is not excused by "EKO chose
  FileStore" — these are the framework's *provided* store/spawner
  implementations (per AGENTS.md framework-deletion rule: a pub API is
  alive if it is a reasonable offered option, even if one consumer doesn't
  use it; here EKO *does* use all of them).
- **EKO product policy (application)**: `app.log`, `SessionSearchEngine`,
  `save_session`, the task `file_shadow` maps, and run-directory retention
  are application-layer concerns (local-desktop persistence policy, UI
  search index, GUI log sink).
- **Adapter boundary**: `infra.rs:377` is the adapter that makes the
  framework `JsonlRunStore` *live* in EKO — it is what promotes the
  framework O(E²) from "dormant API" to "primary chat path".

Repository-wide duplicate search: this task searched for the *same*
growth pattern across both repos rather than a duplicated *concept*. The
two conversation-rewrite paths (`FileConversationStore.save_messages` in
the framework and `save_session` in the app) are **not** duplicates — the
app `save_session` writes the EKO *session* shape (with thinking segments,
execution steps, attachments) while the framework store writes the
`ConversationRecord` shape; both are live and both are full-rewrite. No
merge target; both incur O(M²).

## Current Path

The verified write/fanout graph on the default EKO chat path:

1. User turn → `run_stream_channel` (`stream_channel.rs`) spawns
   `run_core_loop` in a **detached** `tokio::spawn` (`:302-312`) holding
   the owned execution mutex; events flow over a **bounded** `mpsc::channel`
   (`buffer = stream_buffer_size`, default 256, `config.rs:235`).
2. Each iteration records events via `record_event`
   (`snapshot.rs:539-544`) → `JsonlRunStore::append_event`
   (`trace/mod.rs:793-801`) → `load` (whole-file `read_to_string`, `:716`)
   + `push_event` + `save` (append a **full Run snapshot**, `:733-749`).
   `JsonlRunStore` is wired live at `infra.rs:377`.
3. On turn finalize, `chatStore` autoSave (`chatStore.ts:117-120`) → EKO
   `save_session` (`persistence.rs:211-235`) → `write_json` full rewrite
   (`:350`); the framework `FileConversationStore` mirrors this for the
   `ConversationRecord`.
4. Complex tasks: `TaskRuntimeStore` + `FileTaskShadow`
   (`file_shadow.rs`) append per-run `events.jsonl` under
   `~/.eko/tasks/{run_id}/` and register run-keyed maps (`:26,32`); the
   DAG executor bounds *concurrency* (`max_concurrent: 5`, `executor.rs:69`)
   but the `TaskSpawner.tasks` registry is never pruned live.
5. Tracing: `app.log` (`infra.rs:1658-1668`) is an append-only subscriber
   sink for the default GUI/Stderr target.

State owners: `JsonlRunStore.cache` (`trace/mod.rs:680`),
`TaskSpawner.tasks` (`background_task.rs:381`), `FileTaskShadow.seq_cache`
+ `run_write_locks` (`file_shadow.rs:26,32`), `SessionSearchEngine.entries`
(`conversation_file.rs:42`). Terminal/recovery: explicit cancel via
`cancel_token` (honored in think/tool phases); child-process kill via
`select!` `tx.closed()`/deadline + `kill_on_drop` (`local.rs:406,637-655`).

## Findings

### Q-PERF-01-P1-01: `JsonlRunStore` is O(E²) per run, unbounded on disk, and on the live chat path

- Priority: P1
- Confidence: high
- Layer: framework (defect) + adapter (live wiring)
- Evidence:
  - `echo-agent/src/trace/mod.rs:793-801` — `append_event` =
    `load → push_event → save`.
  - `echo-agent/src/trace/mod.rs:733-749` — `save` opens the per-run file
    in `append(true)` and writes `serde_json::to_string(&run)` — a **full
    `Run` snapshot** including `events: Vec<RunEvent>` (`:76`).
  - `echo-agent/src/trace/mod.rs:716-721,724-728` — `load_last_line` reads
    the **entire** file (`read_to_string`) to find the last line, so each
    `append_event`'s `load` is O(current file size).
  - `echo-agent/src/agent/snapshot.rs:539-544` — `record_event` calls
    `append_event` on every recorded event of every run.
  - `echo-agent-cli/echo-agent-app-core/src/infra.rs:377` —
    `JsonlRunStore::new(user_data_path("runs"))` wired into every agent
    build → live on the primary chat path.
  - `echo-agent/src/trace/mod.rs:680` — `cache: RwLock<HashMap<String,Run>>`
    gains one entry per run_id, **never evicted** (no retention grep hits
    in the module).
- Reachability: definition (`trace/mod.rs:677`) → adapter registration
  (`infra.rs:377`) → live caller (`record_event` on every run, every event).
- Expected invariant: a trace store's per-event cost should be O(1) append
  and its on-disk size O(E); a long run should not write super-linear bytes
  or require a full-file read per event.
- Observed behavior: a run with E events performs E appends, the k-th
  appending a ~O(k)-byte snapshot and doing an O(k)-byte read → cumulative
  **O(E²)** bytes written and O(E²) read work; the on-disk file is E full
  snapshots; the in-memory cache grows unbounded with run count.
- Impact: for a tool-heavy 100-iteration run (≈300 events) the trace file
  reaches ~45k-snapshot-lines' worth of cumulative bytes and every event
  re-reads the whole growing file. On the default chat path this is the
  single highest-impact perf defect: slow long runs, growing latency as a
  run progresses, disk bloat under `~/.eko/runs/`, and unbounded trace
  cache memory over many sessions. (Sharpens `F-OPS-01-P2-01`, which named
  "no size bound or retention"; this adds the O(E²) write/read cost and
  the live-path confirmation.)
- Root cause: `append_event` was implemented as load-modify-save of a whole
  snapshot rather than a true append of one event line, and the store has
  no retention/compaction.
- Direction: make `append_event` a real single-line append (write only the
  new event) and reconstruct the `Run` aggregate lazily/on-read, or cap
  the events retained per run; add a retention bound to the `runs/` dir
  and to the in-memory cache (evict by age/count). The whole-snapshot
  append in `save` is the code to replace.
- Regression validation: a test that runs N events and asserts
  `bytes_written = O(N)` (not O(N²)) and that `load` does not read the
  whole file; plus a long-run fixture (V02 style) timing per-event latency
  stays flat.
- Validation reports: [V01](../validations/Q-PERF-01/V01-01.md),
  [V02](../validations/Q-PERF-01/V02-01.md),
  [V04](../validations/Q-PERF-01/V04-01.md).

### Q-PERF-01-P2-01: In-process run/task registries grow without bound (`TaskSpawner.tasks` never pruned live; `FileTaskShadow` maps never evicted; run directories never deleted)

- Priority: P2
- Confidence: high
- Layer: framework (`TaskSpawner`) + application (`FileTaskShadow`, run-dir retention)
- Evidence:
  - `echo-orchestration/src/tasks/background_task.rs:381` —
    `tasks: Arc<DashMap<String, Arc<dyn AnyBackgroundTask>>>`; inserted at
    `:560`.
  - `echo-orchestration/src/tasks/background_task.rs:606-609` —
    `prune_completed` exists; repo-wide `grep -rn "prune_completed"` finds
    **zero** live callers (definition + tests only; module test `:769`
    documents "still tracked until pruned").
  - `echo-agent-cli/.../tasks/task_runtime/file_shadow.rs:26` (`seq_cache`)
    and `:32-34` (`run_write_locks`) — `Arc<Mutex<HashMap<String,…>>>`
    keyed by run_id; `grep` for `remove|prune|evict|retain|clear()` inside
    the file returns no matches.
  - `file_shadow.rs:6` — on-disk layout `{root}/{run_id}/events.jsonl`;
    the only `remove_dir_all`/`remove_file` hits repo-wide are a temp dir
    (`ledger.rs:289`) and `#[cfg(test)]` code (`worktree.rs:1575,1626,1646`).
    The `"retention": "conversation_or_30d"` string at `store.rs:2259` is
    test-fixture metadata, not a policy.
- Reachability: `TaskSpawner` is constructed by the executor
  (`executor.rs` shared spawner) and used for subagent/background dispatch;
  `FileTaskShadow` is the live task-runtime authority
  (`~/.eko/tasks/`). Both are exercised on every complex-task run.
- Expected invariant: a process registry of completed runs should be
  bounded (prune/evict on completion or by capacity); completed run
  artifacts should be retained only up to a policy.
- Observed behavior: every spawned task and every task run adds a
  permanent entry/map-slot and a permanent on-disk directory; nothing on
  the live path ever removes them.
- Impact: monotonic memory growth of the `tasks` DashMap and the two
  shadow maps over the app lifetime, and monotonic disk growth of
  `~/.eko/tasks/`. Concurrency itself is bounded (semaphore, `executor.rs:69`),
  so this is a slow leak, not a runaway — hence P2 not P1.
- Root cause: a prune API was added (`prune_completed`) but never wired
  into the executor's run-completion or a periodic sweep; the shadow maps
  and run dirs have no retention at all.
- Direction: call `prune_completed` on run completion (and/or a periodic
  sweep); add eviction to `seq_cache`/`run_write_locks` when a run
  finalizes; add a retention policy (age/count) that deletes
  `~/.eko/tasks/{old_run_id}/`. Delete the dead `prune_completed`-is-never-
  called gap rather than leaving it.
- Regression validation: a test that spawns N tasks to completion and
  asserts `spawner.task_count()` returns to baseline after prune; a test
  that N finalized runs leave ≤ retention-limit directories.
- Validation reports: [V01](../validations/Q-PERF-01/V01-01.md),
  [V04](../validations/Q-PERF-01/V04-01.md).

### Q-PERF-01-P2-02: `app.log` and other append-only journals grow without rotation

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/.../infra.rs:1658-1668` — `app_log_file` opens
    `~/.eko/logs/app.log` with `create(true).append(true)`; the comment
    (`:1655-1657`) states verbatim *"Append mode keeps history across
    restarts; rotate/truncate manually if it grows too large"*.
  - `infra.rs:1600-1605` — wired as a `tracing_subscriber` file layer in
    the default GUI/Stderr branch (dual sink with stderr).
  - `grep -rn "rolling|RollingFileAppender|tracing_appender|rotation"` over
    `src-tauri` + `app-core` → **no matches** (no rotation anywhere).
    (Contrast: the `TuiFile` branch uses `File::create`, `:1580`, which
    truncates each start.)
  - `tool_execution.rs:641-647` — `append_json_line` appends to per-scope
    tool-execution journals on every tool call (`:237,276,318,345,379`);
  - `runtime.rs:481-486` — checkpoint reflections append to `PROJECT.md`;
  - `evolution/evidence.rs:920-926` — evidence log append. None rotate.
- Reachability: `app.log` is the default GUI tracing sink; the journals
  fire on tool use / checkpoints / evidence writes — all live.
- Expected invariant: a local app's logs/journals should be bounded by
  rotation or size cap so they cannot exhaust disk over a long lifetime.
- Observed behavior: append-only, never truncated/rotated on the default
  path (only the TUI log is truncated).
- Impact: unbounded disk growth of `~/.eko/logs/app.log` and the journals
  over weeks/months of use. On a local single-user machine this is slow
  but real; the code already concedes it needs manual rotation.
- Root cause: no rotation layer was ever wired (no `tracing_appender`
  rolling, no size check).
- Direction: replace the plain `append(true)` file with a
  `tracing_appender::rolling::daily`/`Builder` (or a size-based rotation)
  for `app.log`, and add a retention cap to the JSONL journals.
- Regression validation: a test asserting the log writer rolls (file name
  changes / old file retained up to N); an assertion that journal append
  respects a max-size guard.
- Validation reports: [V04](../validations/Q-PERF-01/V04-01.md).

### Q-PERF-01-P2-03: Conversation/session persistence is a full rewrite per save → O(M²) over long chats

- Priority: P2
- Confidence: high
- Layer: application (`save_session`) + framework (`FileConversationStore`)
- Evidence:
  - `echo-agent-cli/.../persistence.rs:211-235` — `save_session` builds the
    full `SavedSession` (all messages) and calls `write_json` (`:350`) →
    whole-file atomic rewrite on every save.
  - `echo-agent/echo-state/src/memory/file_conversation.rs:350-376` —
    `save_messages` rebuilds the whole `ConversationRecord` (`record.messages
    = assigned`, `:370`) and calls `write_record` (`:168-175`,
    `serde_json::to_string_pretty` + `atomic_write`) → whole-file rewrite.
  - `web-frontend/src/stores/chatStore.ts:117-120,158-159` — autoSave fires
    on each message-add; `addMessage`/streaming appends re-run trim+save.
- Reachability: `save_session` is the EKO chat-save authority; the framework
  store is the `ConversationStore` impl EKO constructs (`conversation_file.rs`
  module doc). Both fire per turn/message on the live chat path.
- Expected invariant: appending one message to a long conversation should
  cost O(1)–O(new message), not O(M).
- Observed behavior: each save serializes and rewrites **all** M messages;
  M saves over a chat → O(M²) cumulative write bytes and O(M²) fsync/IO.
- Impact: a 1000-message conversation does ~1000 full rewrites; the last
  save serializes 1000 messages, and every prior save did too. Visible as
  growing per-turn latency and disk churn on long chats. (Not a data-loss
  issue; correctness is fine — atomic_write is safe.)
- Root cause: JSON-file persistence with whole-document rewrite, no
  incremental/append path.
- Direction: either (a) accept the rewrite for local single-user scale and
  document the M² ceiling, or (b) make message save append a single record
  to an append-only log and compact periodically. Not urgent (P2) but it is
  the clearest "long-chat gets slow" cause.
- Regression validation: a test asserting save cost scales ~linearly with
  *new* messages, not total (e.g. N single-message saves on an M-message
  session are O(N) not O(N·M)).
- Validation reports: [V01](../validations/Q-PERF-01/V01-01.md),
  [V02](../validations/Q-PERF-01/V02-01.md).

### Q-PERF-01-P2-04: `SessionSearchEngine` holds ~2× conversation content in memory and scans linearly per query

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/.../conversation_file.rs:46-51` — `IndexedSession`
    stores both `content_lower` (lowercased full join) and `raw_content`
    (original-case full join) → **two full copies** of every indexed
    session's messages.
  - `conversation_file.rs:107-130` — `search` iterates **all** sessions and
    does `String::contains` on the full content → O(total chars) per query.
  - `conversation_file.rs:138-167` — `reindex_all` reads **every**
    conversation JSON file into memory at startup
    (`std::fs::read_to_string` per file); `index_session` is otherwise only
    called from `reindex_all` (no live per-message re-index — confirmed:
    `index_session` has no caller outside tests/startup).
- Reachability: `SessionSearchEngine` is created in `AppState` init
  (`state.rs:500-501`) with `reindex_all()` at startup; `search` backs the
  sessions-search UI.
- Expected invariant: a local substring-search index should bound resident
  memory (e.g. one normalized copy, capped session count) and not re-scan
  all content per keystroke.
- Observed behavior: 2× full transcript resident per session; each search
  is a linear scan over all sessions' full content.
- Impact: memory ≈ 2× the sum of all conversation transcripts; a search
  across many long sessions is O(total chars). Startup `reindex_all` loads
  every conversation file. For a heavy local user this is a real
  memory/latency cost, but bounded by total history (not unbounded per
  session) — hence P2.
- Root cause: a naive in-memory index with duplicated casing and no
  cap/ranking structure.
- Direction: store one normalized copy; cap the number/size of indexed
  sessions; consider indexing by token or lazy-loading. The UI's `rank`
  field is already a constant (`0.0`), so relevance ranking is not a
  constraint.
- Regression validation: a test asserting resident index size ≤ ~1×
  content and a bounded result-latency as session count grows.
- Validation reports: [V02](../validations/Q-PERF-01/V02-01.md).

### Q-PERF-01-P3-01: `max_iterations == 0` means an unlimited ReAct loop (config footgun)

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/react/run/stream_channel.rs:521-528` —
    `max_iterations == 0` is mapped to `usize::MAX`; `for iteration in
    0..max_iterations`.
  - `echo-agent/src/agent/config.rs:212` — default is `100` (safe).
- Reachability: any consumer (third-party framework user or a future EKO
  profile) calling `.max_iterations(0)` gets an unbounded loop. EKO's own
  default is 100, so this is latent for EKO.
- Expected invariant: a "0" iteration cap should either be rejected or
  documented as "unlimited"; silently mapping it to `usize::MAX` is a
  footgun (note: the dead `LoopDetector` from `F-RCT-02-P2-01` would have
  been the secondary guard, but it is unwired).
- Observed behavior: 0 → unlimited, no secondary loop protection
  (`LoopDetector` is dead infrastructure, per F-RCT-02).
- Impact: low for default EKO; a misconfiguration can produce a
  non-terminating run whose only bound is the token budget / model refusal.
- Root cause: ambiguous sentinel semantics for 0.
- Direction: either reject `max_iterations(0)` at build time, or document
  it loudly as unlimited and rely on `RunBudgetPolicy` (which is wired).
- Regression validation: a builder test asserting `0` is rejected or
  explicitly treated as unlimited with a logged warning.
- Validation reports: [V01](../validations/Q-PERF-01/V01-01.md).

### Q-PERF-01-P3-02: ReAct core-loop driver is a detached task; cleanup on stream-drop is cooperative, not guaranteed

- Priority: P3
- Confidence: medium
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/react/run/stream_channel.rs:302-312` —
    `tokio::spawn(async move { run_core_loop(…) })` with the returned
    `JoinHandle` **dropped**; the owned execution mutex + active-turn lease
    are moved in and released only on `run_core_loop` return.
  - `stream_channel.rs:410-411` — the Token branch checks
    `tx.send(...).await.is_err()` ("Receiver dropped — caller cancelled")
    and the loop returns; other phases rely on the agent `cancel_token`.
  - `stream_channel.rs:1974` — `test_run_stream_cancelled_mid_llm_call`
    confirms explicit token-cancel propagates to the LLM layer.
- Reachability: every streaming run spawns this detached driver.
- Expected invariant: dropping the consumer stream (or cancelling) should
  guarantee the driver task and its held mutex are released in bounded time.
- Observed behavior: explicit cancel works (token); implicit cancel
  (drop `rx`) works **only if** every await point notices a send error.
  The `JoinHandle` is not retained, so there is no `abort()` fallback if a
  future await becomes non-cancellable.
- Impact: robustness gap, not an observed deadlock. If a future phase adds
  a long non-cancellable await, a dropped stream could leave the driver
  (and the execution mutex) alive past consumer drop.
- Root cause: handle is not tracked; termination relies on cooperative
  send-error propagation.
- Direction: retain the `JoinHandle` in a run-scoped registry and
  `abort()` it on stream-drop / finalize, or add a cancel-token `select!`
  arm at the driver level. Low priority because explicit cancel already
  works and no current await is non-cancellable.
- Regression validation: a fault-injection test (Q-FLT-01) that drops the
  stream mid-run and asserts the mutex is released within bounded time.
- Validation reports: [V03](../validations/Q-PERF-01/V03-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Static unbounded-growth lifecycle trace (registries, maps, loop ceiling, full-rewrite) | yes | passed | [V01-01](../validations/Q-PERF-01/V01-01.md) |
| V02 | Representative large-fixture cost analysis (1000 msgs / 100 tasks / 50 subagents / large output) | yes | passed | [V02-01](../validations/Q-PERF-01/V02-01.md) |
| V03 | Cancellation cleanup trace (spawned tasks / processes / connections on cancel) | yes | passed | [V03-01](../validations/Q-PERF-01/V03-01.md) |
| V04 | Disk/cache growth analysis (trace store, run dirs, logs, journals, retention) | yes | passed | [V04-01](../validations/Q-PERF-01/V04-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `F-OPS-01-P2-01` "JsonlRunStore/InMemoryRunStore no size bound or retention" | **current + sharpened** | `trace/mod.rs:733-801` (still load-modify-save full snapshot; no retention). This report adds the **O(E²)** cost and the live wiring at `infra.rs:377`. → Q-PERF-01-P1-01 |
| `F-OPS-01-P3-03` "`last_fired` HashMap grows without eviction" | current | scheduler cron map; orthogonal to this audit's findings (not re-traced). |
| `F-CMP-01-P2-01` "summary system messages accumulate across compression cycles" | current | prompt-assembly growth axis; consumed, not re-derived. Confirms one prompt-assembly unbounded-growth site. |
| `F-CTX-01-P2-01` "tool defs/system prompt not accounted against budget" | current | budget axis, not a growth axis; consumed only. |
| `A-FE-03-P2-01` "MessageBubble O(N·T) re-render" | current | frontend *re-render* cost; this audit confirms the frontend *arrays* are bounded (`MAX_MESSAGES=500`, `MAX_EVENTS=500`), so the two findings are orthogonal. |
| (new) TaskSpawner/file_shadow registries never pruned | new | Q-PERF-01-P2-01 |
| (new) app.log no rotation | new | Q-PERF-01-P2-02 |

## Coverage And Uncertainty

- **Inspected**: trace store write/read path, ReAct streaming spawn +
  iteration ceiling, DAG/subagent executor + spawner, sandbox child
  cleanup, agent pool, EKO persistence (`save_session`,
  `FileConversationStore`), session search engine, task `file_shadow`,
  `app.log` + journals, frontend store caps.
- **Not live-measured**: no model/network fixture was run, so all cost
  claims are asymptotic from the write loops (sufficient to establish
  super-linearity/unboundedness; constant factors unmeasured). A real
  end-to-end timing pass (`Q-E2E-01`) and fault injection (`Q-FLT-01`)
  would refine magnitudes and confirm the detached-task cleanup
  empirically.
- **Not deeply inspected**: the IM-channel fanout paths
  (`echo-orchestration/src/human_loop/websocket.rs` uses an unbounded
  channel — noted but not traced to a finding; likely low-impact for the
  local single-user model), the LLM streaming token-buffer cost, and the
  `evolution/dashboard` aggregates (they read from the trace store and
  thus inherit P1-01's cost indirectly).
- **Commit drift**: echo-agent HEAD is `3aa7929`, one `fix(tests)` commit
  ahead of the card's `9b0e0fa`. `git diff --name-only 9b0e0fa HEAD` touches
  `stream_channel.rs` (+ `pipeline.rs`, `phases/tools.rs`, test mocks);
  the two `stream_channel.rs` anchors used here (detached spawn `:302`,
  `0==unlimited` `:523`) were re-grepped at HEAD and are unchanged. All
  other perf anchors are in files untouched by that commit.

## Handoff

- **Downstream tasks may rely on**: (1) the trace store is the top
  perf priority — P1-01, O(E²), live on the chat path; (2) the unbounded
  registries (P2-01) and no-rotation logs (P2-02) are real but slow leaks;
  (3) cancellation of child processes / subagents / agent pool is sound —
  only the detached ReAct driver (P3-02) is a robustness gap; (4) frontend
  arrays are bounded, so the A-FE-03 re-render finding stands alone.
- **Must read**: `F-OPS-01` (P2-01/P1-03 — this report's P1-01 sharpens
  its trace-store finding and adds the O(E²) cost), `F-CMP-01` (prompt
  growth), `A-FE-03` (render cost).
- **Makes this report stale**: any change to `trace/mod.rs::append_event`
  (real single-line append), wiring of `prune_completed`, addition of log
  rotation, or a retention policy on `~/.eko/runs|tasks|logs`.
- **Follow-up task IDs (no fixes here)**: `Q-FLT-01` (fault-inject stream
  drop / cancel against the detached driver), `Q-E2E-01` (capture a real
  `~/.eko` footprint after a long session), `S-APP-01`/`S-FW-01` (fold
  P1-01/P2-01 into the framework vs application roadmap — trace store fix
  is framework; retention/log-rotation is application).
