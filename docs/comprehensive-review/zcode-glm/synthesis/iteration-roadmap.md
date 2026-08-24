# S-RDM-01: Iteration Roadmap

> Task: S-RDM-01 (FINAL deliverable)
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Baseline: `echo-agent` `9b0e0fa`, `echo-agent-cli` `b3b2e81`
> Synthesis date: 2026-08-13
> Sources: `framework-review.md` (S-FW-01), `application-review.md` (S-APP-01), `cross-repository-review.md` (S-X-01), `B-REF-01.md`

This roadmap merges all three synthesis reports into a single sequenced
action plan. Every finding is deduplicated across repositories. The
ordering reflects dependency chains (shared helpers unblock multiple
fixes), severity (P1 first), and leverage (cheap fixes that close whole
classes ranked higher than expensive one-offs).

---

## 1. Executive Summary

### Finding inventory

| Source synthesis | P0 | P1 | P2 | P3 | Total |
|---|---:|---:|---:|---:|---:|
| Framework (S-FW-01) | 0 | 16 | 87 | 113 | 216 |
| Application (S-APP-01) | 0 | 8 | 57 | 74 | 139 |
| Cross-repo (S-X-01) | 0 | 2 | 23 | 14 | 39 |
| **Cross-synthesis dedup** | 0 | -4 | -18 | -7 | -29 |
| **Net unique** | **0** | **22** | **~149** | **~194** | **~365** |

Cross-synthesis dedup removes findings filed in multiple reports:
`FW-QUAL-001` = `APP-CROSS-P1-01` (globstar panic); `APP-OBS-P1-03` is a
re-prioritization of `FW-OPS-003` (RunStore confirmed live); `X-MEM-01-P1-01`
= `APP-STATE-MEM-P1-01`; `X-SRF-01-P1-01` = `APP-TOOL-INT-P1-01`;
`X-BND-01-P2-01` absorbs 4 atomic-write findings across all three syntheses.

**No P0 findings.** No unrecoverable-system-level data-corruption-with-
secret-exposure issues exist. The worst defects are P1: capability failures
on valid input, silent data loss on crash, leaked resources, plaintext
secret persistence, and process-crashing panics.

### Architecture verdict

**Sound.** The review confirms the layering is structurally correct:

- **One revisioned TaskRun graph** (framework) with a thin EKO adapter.
  No parallel task/plan/todo CRUD. AGENTS.md rule 6 holds end-to-end.
- **All adapters are thin** — no EKO adapter owns scheduling, DAG loop,
  generic retry, validator, or deadlock-detection authority.
- **5 of 6 AGENTS.md invariants hold cleanly** (Subagent-only terminology,
  CLI no-SQLite, no parallel task CRUD, relative Cargo paths, panic-keyword
  baseline). One regression: IQR index-out-of-bounds panic (`FW-TOOLS-003`).
- **Two prior UTF-8 violations** reaffirmed unchanged (`globstar`,
  `RuleGuard.max_length`).

The gaps are **wiring defects and dead code**, not structural flaws. The
framework provides every primitive the surfaces need; the application
simply does not wire some of them on every mode. Every parity gap is an
undocumented absence the AGENTS.md rule explicitly classifies as "待补的
缺口,不是产品定位."

### Cross-cutting patterns (fixing the pattern fixes a class)

| Pattern | Sites | Findings closed |
|---|---:|---|
| Dead infrastructure / dormant APIs | 16+ | FW-CORE-001, FW-RCT-004/005, FW-LLM-011/012, FW-NBK-001, FW-SUB-010/013/019, FW-FEAT-001, APP-STATE-P2-01, X-BND-01-P2-02/03, etc. |
| Non-atomic file operations | 6 | FW-TOOLS-002, FW-MEM-001/002, X-BND-01-P2-01, APP-OUT-P3-04, APP-PROJ-P3-02 |
| UTF-8 / byte-length violations | 7 | FW-QUAL-001, FW-TOOLS-003/011, FW-SEC-003, FW-MEM-006, FW-CMP-004, FW-WFL-008 |
| Streaming event loss under backpressure | 2 | FW-RCT-001, FW-RCT-002 |
| Cancelled-vs-error collapse | framework + 4 surfaces | FW-RCT-003, X-EVT-01-P2-01 |
| Secret leakage (no `redact_secrets`) | 3 boundaries | FW-OPS-003, APP-OBS-P1-01/02 |
| Surface parity gaps (TUI/GUI/CLI/channels) | 15 | APP-SURF-*, APP-BOOT-P2-02, X-SRF-01-* |
| Hand-written TS mirrors of Rust enums | 2 families | X-TOL-01-P2-02, X-EVT-01-P3-02 |

---

## 2. P1 Critical Fix List (All 22)

Each P1 is listed with its canonical ID, subsystem, one-line description,
fix complexity (S = small, <1 day; M = medium, 1-3 days; L = large, 3+ days),
and fix pattern. Cross-repo duplicates are annotated.

### 2.1 redact_secrets pattern (3 P1s, 3 code boundaries)

The framework exposes a `pub`, UTF-8-safe `redact_secrets` function (~18
patterns). Three persistence/emission boundaries persist raw content
without invoking it. A single shared redaction pass closes the entire class.

| # | ID | Subsystem | Description | Complexity |
|---|---|---|---|---|
| 1 | FW-OPS-003 (= APP-OBS-P1-03) | Operations / RunStore | Secrets persisted to `JsonlRunStore` via unredacted `Run.input` / `final_output` / `ToolResult.output_preview` / `ToolError.message`. Confirmed LIVE in production (`infra.rs:374-385`). | S |
| 2 | APP-OBS-P1-01 | Observability / ToolExecution | `ToolExecutionRepository` persists tool args / output / failure in plaintext to `~/.echo-agent/tool-executions/`. | S |
| 3 | APP-OBS-P1-02 | Observability / Webhook | `WebhookTurnObserver` ships raw tool args and error messages to external HTTP endpoints (no URL scheme validation). | S |

**Fix pattern:** Apply `redact_secrets` to `args_preview`/`args_full`, output
chunks, failure fields, and error/message clones before writing/POSTing.
Optionally validate webhook URL is `https://` or `http://localhost`.

### 2.2 atomic_write / crash-safe persistence (3 P1s)

The canonical atomic-write recipe (uuid temp -> write -> fsync temp -> rename
-> fsync parent dir) is implemented correctly in exactly one place
(`FileConversationStore`). Every other persistent-write path is broken
differently. Extracting one shared `atomic_write(path, bytes)` helper into
framework `echo-state::util` resolves this class.

| # | ID | Subsystem | Description | Complexity |
|---|---|---|---|---|
| 4 | FW-TOOLS-002 | Tools / File ops | `WriteFileTool`, `UpdateFileTool`, `EditFileTool`, `CreateFileTool` all use bare `tokio::fs::write` (O_TRUNC, no temp/rename/fsync). Crash between truncate and fsync leaves file truncated/partial. | M |
| 5 | FW-MEM-001 | Memory / FileStore | `FileStore::new` parses corrupt JSON with `unwrap_or_else(HashMap::new)`, then next `flush()` overwrites the corrupt file via rename, destroying recovery chances. Silent permanent loss of all long-term memory. | S |
| 6 | FW-WFL-001 | Workflow / Graph | `Graph::resume()` parallel branch mutates shared state in place (no fork, no merge), diverging from `run()`/`run_until_interrupt()`/`run_stream()`. Resumed parallel+interrupt workflows produce different state than straight-through runs. | L |

**Fix pattern:** Extract `pub fn atomic_write(path, bytes) -> io::Result<()>`
with parent-dir fsync. Route all 6 call sites through it (4 file tools +
`FileStore::flush` + `EmbeddingStore::flush_index`). For FW-MEM-001, also
make `FileStore::new` return `Err` on corrupt JSON (matching
`FileConversationStore`). For FW-WFL-001, fork each branch into isolated
`SharedState` and `deep_merge` back.

### 2.3 UTF-8 safety / panic prevention (2 P1s)

AGENTS.md mandates `chars().count()` / `chars().take()` and bans byte
slicing, `unwrap`, direct indexing, and any panic-on-valid-input API.

| # | ID | Subsystem | Description | Complexity |
|---|---|---|---|---|
| 7 | FW-QUAL-001 (= APP-CROSS-P1-01) | Application / gitignore | `globstar_match` iterates byte positions (`for j in 0..=remaining.len()`) then slices `&remaining[j..]`, panicking on any multi-byte UTF-8 path (Chinese, emoji) when a `**` pattern is loaded. Empirically reproduced. | S |
| 8 | FW-TOOLS-003 (= X-INV-01-P2-01) | Tools / Data quality | `outlier_detection` IQR method: `sorted[n / 4.min(n - 1)]` — method-call precedence binds `4.min(n-1)` to the divisor, not the index. For `n==4`: `sorted[4]` is out of bounds -> panic (exit 101). | S |

**Fix pattern:** Rewrite `globstar_match`/`simple_glob` over `Vec<char>`.
Replace IQR `sorted[...]` with `sorted.get(...)` or the existing safe
`quantile()` helper. Add n=4 and Chinese-path regression tests.

### 2.4 lifecycle / cancellation (5 P1s)

Cancellation tokens are wired but never polled, or JoinHandles are detached.
Every defect in this group results in leaked LLM budget, tool side effects,
or stale runs after the user believes the operation stopped.

| # | ID | Subsystem | Description | Complexity |
|---|---|---|---|---|
| 9 | FW-INT-001 | Integrations / MCP HttpTransport | `notification_tx` declared but never fed; 202-Accepted branch parks on a `oneshot::Receiver` nothing fulfills, hangs to 60s timeout. Any async MCP server is unusable. | M |
| 10 | FW-INT-002 | Integrations / A2A sync | `tasks/send` stores `CancellationToken` but never polls it during `agent.execute().await`. Cancel racing late completion regresses Canceled -> Completed. Terminal monotonicity violated. | M |
| 11 | FW-OPS-001 | Operations / Scheduler | `SchedulerRunner::spawn` detaches `JoinHandle`; no caller cancels `cancel_token`. In-flight cron runs not drained at exit; appear as stale runs on restart. | M |
| 12 | FW-SUB-001 | Subagents / Team mode | `dispatch_team` ignores `req.cancel`; zero `CancellationToken` references in team code. Cancelling a 5-subagent team cannot stop it — tokens, API calls, tool side effects continue. | M |
| 13 | FW-SUB-002 | Subagents / Team timeout | `execute_with_usage` wraps in `tokio::time::timeout`; on timeout the `Vec<JoinHandle>` is dropped (detaches tasks). Spawned `agent.execute` tasks complete silently, results discarded. | M |

**Fix pattern:** FW-INT-001: implement GET-SSE listener or drop the 202
scaffold and fail fast. FW-INT-002: route terminal writes through
`update_task_state`; wrap `agent.execute` in `select!` against cancel.
FW-OPS-001: return `JoinHandle`; fire cancel + await on shutdown.
FW-SUB-001/002: thread `req.cancel` into team execution; `JoinHandle::abort`
on timeout (pair both fixes — shared cleanup path).

### 2.5 silent data loss / corruption (3 P1s)

Data that the user believes was saved or refreshed, but was not.

| # | ID | Subsystem | Description | Complexity |
|---|---|---|---|---|
| 14 | APP-STATE-MEM-P1-01 (= X-MEM-01-P1-01) | State / Memory | All 8 MEMORY.md-mutating sites call `refresh_instruction_projection` (wrong target) instead of `refresh_hot_memory_projection`. Promoted hot memories never appear in stable prefix until restart. Dreaming headline capability silently broken. | S |
| 15 | FW-RESUME-002 (P2 in framework, elevated here for data-loss class) | ReAct / Resume | Corrupt checkpoint silently swallowed (`warn!` + `reset_messages()`), then next turn's `save_checkpoint` overwrites the corrupt file. Original corruption unrecoverable. No `version` field on `AgentCheckpoint`. | M |
| 16 | APP-BOOT-P1-02 | Boot / Config watcher | Config watcher targets built once from cwd-at-spawn; `switch_workspace` mutates CWD but not the watcher. After switching to workspace W, edits to `W/.eko/hooks.yaml` never hot-reload. | S |

**Fix pattern:** APP-STATE-MEM-P1-01: replace 8 wrong-target calls with
`refresh_memory_projections` (idempotent, refreshes both). FW-RESUME-002:
preserve corrupt file to `.corrupt-{ts}`; surface resume failure via event;
add `version` field. APP-BOOT-P1-02: give `switch_workspace` a handle to
re-register the new workspace's hooks file.

### 2.6 functional / parity / over-gating (6 P1s)

Capability failures on valid input, mode-equivalence violations, and
over-gating that blocks legitimate local use.

| # | ID | Subsystem | Description | Complexity |
|---|---|---|---|---|
| 17 | FW-LLM-001 | LLM / Anthropic adapter | Streaming tool-call `ContentBlockStart(ToolUse)` inserts at key `tool_call_args.len()` but `ContentBlockDelta`/`Stop` look up by content-block `index`. When text precedes a tool_use, keys diverge and every tool call is silently dropped. The `_index` field was renamed to suppress the unused-warning. | S |
| 18 | FW-RCT-001 | ReAct / Streaming | `yield_event_or!` uses `tx.try_send`; on `Full` it `warn!`s and drops the event. 16 intermediate-event sites use it. Live UX rendering has holes under backpressure. | M |
| 19 | FW-OPS-002 | Operations / Headless | `run_headless` calls `agent.execute`, returns `HeadlessResult`; no `AgentEvent` stream, no `RunStore`, no `CancellationToken`, no callback/metrics/trace. Violates AGENTS.md mode-equivalence mandate. | L |
| 20 | FW-NBK-001 | Notebook | `NotebookTracker` / `NotebookCell` / six public methods; always compiled (not feature-gated). Zero live callers; `enable_notebook` setter never read. Aspirational stub with misleading pub API. | S (delete) |
| 21 | APP-BOOT-P1-01 | Boot / Config | Bootstrap silently succeeds with no resolvable auth; first user message triggers opaque 401. GUI launches worst case (shell env vars absent). | S |
| 22 | APP-TOOL-INT-P1-01 (= X-SRF-01-P1-01) | Tools / MCP IPC | `validate_ipc_mcp_stdio` rejects non-allowlisted executables; `validate_ipc_mcp_url` rejects loopback/private-range URLs. Legitimate local MCP servers unreachable via GUI while on-disk config accepts them. Same class as historical `require_full_auto` over-gating. | S |

**Fix pattern:** FW-LLM-001: rename `_index` -> `index`, use as insertion key;
pair with streaming fixture test. FW-RCT-001: change `yield_event_or!` to
`send().await`. FW-OPS-002: reimplement on `execute_stream` + optional
`RunStore`. FW-NBK-001: delete (zero live callers, no-backward-compat).
APP-BOOT-P1-01: surface typed error / setup screen when both config and env
are empty. APP-TOOL-INT-P1-01: drop executable allowlist + private-range
rejection; keep denylist + shell-metacharacter + path-traversal guards.

**Note on APP-SURF-CLI-P1-01:** CLI `/workspace switch` is a no-op for live
state (opens registry, prints, never calls `AppState::switch_workspace`).
This is a P1 capability failure, folded into the functional/parity group
above as a companion to APP-BOOT-P1-02 (both workspace-switch defects).

---

## 3. Tier-0: Immediate Actions (data integrity + secrets + panics)

**Goal:** eliminate every defect that can corrupt user data, leak secrets,
or crash the process on valid input. All are small, localized changes.

| Priority | Action | Findings | Effort |
|---|---|---|---|
| T0-1 | **Secret redaction sweep:** apply `redact_secrets` at the 3 persistence/emission boundaries (`JsonlRunStore::save`, `ToolExecutionRepository`, `WebhookTurnObserver`). Single shared pass. | FW-OPS-003, APP-OBS-P1-01, APP-OBS-P1-02 | 0.5 day |
| T0-2 | **Panic prevention:** fix `globstar_match` (rewrite over `Vec<char>`) and IQR outlier detection (use `sorted.get()` or safe `quantile()`). Add regression tests. | FW-QUAL-001, FW-TOOLS-003 | 0.5 day |
| T0-3 | **Crash-safe FileStore:** make `FileStore::new` return `Err` on corrupt JSON (matching `FileConversationStore`). One-line behavioral change. | FW-MEM-001 | 0.5 day |
| T0-4 | **Atomic-write helper extraction:** create `pub fn atomic_write(path, bytes)` in framework `echo-state::util` with parent-dir fsync. Migrate the 4 file tools to it immediately (unblocks the P2 store migration later). | FW-TOOLS-002 | 1 day |
| T0-5 | **Hot-memory projection fix:** replace 8 wrong-target `refresh_instruction_projection` calls with `refresh_memory_projections`. Restores Dreaming/`/remember` headline capability. | APP-STATE-MEM-P1-01 | 0.5 day |
| T0-6 | **Anthropic streaming tool-call fix:** rename `_index` -> `index`, use as insertion key. The single most impactful adapter bug — every Claude agentic streaming flow loses tool calls today. | FW-LLM-001 | 0.5 day |

**Tier-0 exit criteria:** no P1 secret-leak, panic, or data-loss finding
remains open. All 6 actions land in one merge cycle.

---

## 4. Tier-1: High-Priority (lifecycle + parity + dead infra)

**Goal:** close capability regressions, restore mode equivalence, and remove
the most misleading dead surfaces. These are the fixes that unblock real
product use on TUI/channels/headless.

### 4.1 Lifecycle / cancellation cluster

| Action | Findings | Effort |
|---|---|---|
| Team-mode cancel + timeout cleanup: thread `req.cancel` into team execution; abort JoinHandles on timeout. | FW-SUB-001, FW-SUB-002 | 2 days |
| Scheduler graceful shutdown: return JoinHandle, fire cancel + await on shutdown. | FW-OPS-001 | 1 day |
| HttpTransport 202 branch: implement GET-SSE listener or fail fast. | FW-INT-001 | 1.5 days |
| A2A sync cancel + terminal monotonicity: route writes through `update_task_state`, wrap `execute` in `select!`. | FW-INT-002 | 1.5 days |
| `ReactAgent` Cancelled emission: wrap `cancel_aware_stream` in overrides (framework root-cause for X-EVT-01-P2-01). Unblocks every surface's Cancelled arm. | FW-RCT-003 | 1 day |

### 4.2 Surface parity cluster

| Action | Findings | Effort |
|---|---|---|
| CLI `/workspace switch` through `AppState::switch_workspace`. | APP-SURF-CLI-P1-01 | 1 day |
| MCP IPC over-gating removal + threat-model comment rewrite + `IpcAuth` deletion (single patch). | APP-TOOL-INT-P1-01, X-AUT-01-P2-01, APP-TOOL-HITL-P2-02 | 1 day |
| Channels-only mode: route through `start_headless_services`; extend `resume_pending` to cron runs; register chat-lane cancel + `/cancel`. | APP-BOOT-P2-02, X-SRF-01-P2-02/03 | 2 days |
| GUI window close: register `on_window_event(CloseRequested)` -> `terminal_manager.close_all()`; graceful MCP/LSP shutdown. | APP-SURF-GUI-P2-01, APP-TOOL-INT-P2-01 | 1 day |
| Missing API key fast-fail gate. | APP-BOOT-P1-01 | 0.5 day |

### 4.3 Dead infrastructure removal (high-misleading-value)

| Action | Findings | Effort |
|---|---|---|
| Delete `GLOBAL_EVENT_BUS` / `EventBus` (zero producers/consumers in either repo). | FW-CORE-001, X-BND-01-P2-03 | 0.5 day |
| Delete notebook stub (`NotebookTracker` + `enable_notebook`). | FW-NBK-001 | 0.5 day |
| Delete `LoopDetector` + `process_steps` + helpers (superseded by `run_core_loop`). | FW-RCT-004, FW-RCT-005 | 0.5 day |
| Delete `AdapterClient` / `DefaultLlmClient` (superseded by `OpenAiClient`). | FW-LLM-011, FW-LLM-012 | 0.5 day |
| Delete `TaskSubagent` trait + return types (superseded by `RuntimeDagController`). | X-BND-01-P2-02 | 0.5 day |

### 4.4 Streaming integrity

| Action | Findings | Effort |
|---|---|---|
| Change `yield_event_or!` to `send().await` (backpressure is desired behavior). Pair with terminal-error sites (FW-RCT-002). | FW-RCT-001, FW-RCT-002 | 1 day |
| Trace finalization symmetry: add `finalize_run` to tool-success + abandoned/cancelled/blocked arms. | FW-RCT-006, FW-RCT-008 | 0.5 day |

---

## 5. Tier-2: Medium (P2 cleanup + convergence)

**Goal:** close the systemic patterns that generate whole classes of future
bugs, and address high-frequency UX papercuts.

### 5.1 Atomic-write consolidation (the largest live-duplicate pattern)

Extract one canonical `atomic_write` with parent-dir fsync into framework
`echo-state::util`. Migrate all 6 call sites. Delete the 5 redundant copies.

**Closes:** X-BND-01-P2-01, X-STA-01-P2-02, FW-MEM-002, APP-OUT-P3-04,
APP-PROJ-P3-02. **Effort:** 1.5 days.

### 5.2 Conversation-deletion cascade

Add `delete_runs_for_conversation(conversation_id)` to `TaskRuntimeStore`;
wire `state_store.clear_conversation`; extract
`AppState::delete_conversation_cascade(id)` called from BOTH Tauri and TUI.

**Closes:** X-STA-01-P2-03, APP-STATE-P2-02, APP-TSK-P2-02. **Effort:** 1 day.

### 5.3 ToolExecutionObserver extraction (keystone for surface parity)

Extract tool-execution recording into a driver-level
`ToolExecutionObserver` constructed inside `drive_chat_inner` from
`ChatResources`. `TauriChatSink` becomes pure render; TUI/CLI/channels gain
durable history. The subagent bridge shrinks to event-routing only.

**Closes:** APP-CHAT-P2-01, APP-SURF-GUI-P2-03, X-SRF-01-P3-03. **Effort:** 3 days.

### 5.4 Budget accounting + compression fixes

Pass real tool-def and system-prompt token costs into `budget.allocate`;
deduct protected content from compressor's effective limit; fix the
oversized `396_000` fallback; remove accumulated summary system messages.

**Closes:** FW-CTX-001/002/004, FW-CMP-001. **Effort:** 2 days.

### 5.5 Chat-mode resume

Call `restore_thread_context` on chat mode when context is empty. Restores
cross-process chat resume (the most common interaction mode).

**Closes:** FW-RESUME-001. **Effort:** 1 day.

### 5.6 Checkpoint corruption handling

Preserve corrupt checkpoint to `.corrupt-{ts}`; surface resume failure via
event/enum; add `version` field to `AgentCheckpoint` and `Checkpoint`.

**Closes:** FW-RESUME-002, FW-WFL-004. **Effort:** 1.5 days.

### 5.7 Collision non-determinism (framework loaders)

Single coordinated fix across both framework loaders: sort `read_dir`
entries by path; emit `warn!` naming both paths on collision; converge on
first-scope-wins for both.

**Closes:** X-PLG-01-P2-01, FW-SKL-001/002, FW-PLG-001. **Effort:** 1 day.

### 5.8 events.jsonl partial-tail recovery

Factor `read_journal_repairing_last_line` logic into a shared
`read_jsonl_repairing_last_line`; route `file_shadow::read_events` through
it. Decouple projection reads from the event file.

**Closes:** X-STA-01-P2-01, APP-TSK-P2-01. **Effort:** 1 day.

### 5.9 Frontend high-impact fixes

| Action | Finding | Effort |
|---|---|---|
| Switch `ToolsPanel` to generated `ToolInfo` (or fix field name `input_schema` -> `parameters`). | APP-FE-P2-01 | 0.5 day |
| Lift `lastAssistantMessageId`/`messageIds` out of `MessageBubble` into `ChatPanel`. Highest-impact perf fix (O(N*T) -> O(T)). | APP-FE-P2-02 | 1 day |
| Shared `Modal`/`Dialog` primitive with focus trap, Escape, autofocus; migrate 3 modals. | APP-FE-P2-03 | 1 day |
| Migrate tool-execution TS types to `#[derive(TS)]`; add `default: never` exhaustiveness guard to `chatEventHandler.ts`. | X-TOL-01-P2-02, X-EVT-01-P3-02 | 1 day |

### 5.10 Remaining dead-code sweep (batch)

Delete: `ProjectIndex` (488 lines), `FileChangeTracker`/`CodingLoop`
(~150 lines), `Persistence`/`SessionSearchEngine` (~600 lines),
`output::OutputFormat`/`FormatContext` (~140 lines), `LatexExporter`
(~160 lines), `auto_memory::run_auto_memory_extraction`,
`SandboxConfigData.security_level`, `parallel_tasks`/`TaskStrip` scaffold,
`HandoffManager`/`HandoffTool`, `TopologyTracker`, `isolated::run_isolated`,
5 dead Cargo features.

**Effort:** 2 days (batch). Each item is small and self-contained; most can
ride along with nearby changes per AGENTS.md "随手清理."

---

## 6. Tier-3: Low (P3 incremental)

Per AGENTS.md "随手清理是强制要求" — address when touching the relevant
module. Notable clusters:

### 6.1 Testing infrastructure

- **Enrich `MockLlmClient`** for multi-chunk streaming, mid-stream errors,
  fragmented tool calls (FW-TST-001..005). Unblocks better coverage for
  everything above.
- **Add HTTP-mock streaming fixtures** for Anthropic + OpenAI. Would have
  caught FW-LLM-001/005/006 at authoring time (cross-cutting pattern 4.5).
- **Cross-layer fixtures** for `{InvalidArguments, Timeout, Cancelled,
  PartialSideEffect}` through framework -> EKO -> frontend
  (X-TOL-01-P2-03).
- **Contract tests** for manual DTO shapes against wire drift (APP-FE-P3-04).

### 6.2 Macros

- `#[derive(Tool)]` generics handling + schemars path generation + trybuild
  compile-fail tests (FW-MAC-001..011).

### 6.3 Dependency convergence

- Collapse `hashbrown` (5 major versions) and `quick-xml` (4-5 versions)
  across both lockfiles (FW-QUAL-005, FW-QUAL-007).
- Run `cargo tree -d` before/after; coordinate framework + CLI in one cycle.

### 6.4 Documentation drift

- `infra.rs:125` "sqlite-backed" -> "file-backed" (X-INV-01-P3-01)
- `types.rs:917-920` narrow "lossless" claim (X-TSK-01-P3-01)
- `path_validator.rs:7-9` drop XSS framing (X-AUT-01-P3-01)
- `worktree.rs` module doc references removed `panels.rs` (APP-TSK-P3-06)
- CLI/GUI strings say "AGENTS.md" but writes `learned-rules.md` (APP-EVO-P3-01)

### 6.5 UTF-8 sweep (7 remaining sites)

Fix byte-length/byte-slice sites: `RuleGuard.max_length`, `tokenize`
byte filter, Horizon compact-summary byte comparison,
`find_occurrence_lines` byte slicing, `sequential.rs` doc example,
`print_tool_result` char/byte mix.

### 6.6 File splitting

Split 5000+ line files along sub-responsibility boundaries:
`executor.rs` (6272), `tui/events.rs` (5746), `data.rs` (3751),
`subagent/executor.rs` (3672), `task_runtime/store.rs` (3496).
Behavior-preserving.

### 6.7 ESLint + a11y

Install ESLint + `eslint-plugin-jsx-a11y`; wire into `npm test`/CI
(APP-FE-P3-13). Add `aria-label` to primary chat textarea (APP-FE-P3-14).

---

## 7. Dependency DAG

### 7.1 Repository ordering

```
echo-agent (framework) fixes FIRST
  |
  +-- atomic_write helper (echo-state::util)
  +-- redact_secrets (already pub, just needs call sites)
  +-- cancel_aware_stream wrapping (ReactAgent)
  +-- streaming fixture tests
  |
  v
echo-agent-cli (application) fixes SECOND
  (depend on framework helpers being available)
```

**Cross-repo merge order:** if a framework change adds a new pub API that
the CLI consumes, merge `echo-agent` to main first, then `echo-agent-cli`.

### 7.2 Helper-extraction unblocks

```
atomic_write helper ──┬──> FW-TOOLS-002 (4 file tools)
                      ├──> FW-MEM-001 (FileStore corrupt handling)
                      ├──> FW-MEM-002 (FileStore/EmbeddingStore parent-dir fsync) [P2]
                      ├──> X-BND-01-P2-01 (6-site consolidation) [P2]
                      └──> APP-OUT-P3-04, APP-PROJ-P3-02 [P3]

redact_secrets call ──┬──> FW-OPS-003 (JsonlRunStore)
                      ├──> APP-OBS-P1-01 (ToolExecutionRepository)
                      └──> APP-OBS-P1-02 (WebhookTurnObserver)

refresh_memory_projections ──┬──> APP-STATE-MEM-P1-01 (8 sites)
                             ├──> X-MEM-01-P2-01 (pool helper widening) [P2]
                             └──> X-MEM-01-P2-02 (CLI fan-out) [P2]

cancel_aware_stream (framework) ──┬──> FW-RCT-003 (ReactAgent Cancelled emission)
                                  ├──> X-EVT-01-P2-01 (all surface Cancelled arms)
                                  └──> obsoletes GUI post-hoc compensation

ToolExecutionObserver extraction ──┬──> APP-CHAT-P2-01 (sink purity)
                                    ├──> APP-SURF-GUI-P2-03 (ExecutionEvent enum)
                                    ├──> X-SRF-01-P3-03 (TUI/CLI durable history)
                                    └──> APP-TOOL-SUB-P2-01 (subagent bridge shrink)
```

### 7.3 Critical path

The longest dependency chain is:

1. `atomic_write` helper (framework) — 1 day
2. File tools migration (framework) — 0.5 day
3. CLI consumes updated framework — merge order gate
4. `ToolExecutionObserver` extraction (application) — 3 days
5. TUI/CLI/channels durable tool history (application) — 1 day

Total critical path: ~6 days if sequenced; ~3 days if framework and
application work proceeds in parallel worktrees.

---

## 8. Deletion Targets

All items meet AGENTS.md deletion criterion branch 1 (superseded internal
dead code) or branch 2 (superseded by equivalent). None are legitimate
framework menu options (the `SqliteStore`/`HybridCompressor`/`EmbeddingStore`
retention test excludes them).

### 8.1 Framework dead code (echo-agent)

| Item | Location | Lines | Superseded by | Finding |
|---|---|---|---|---|
| `GLOBAL_EVENT_BUS` / `EventBus` | `echo-agent/src/event_bus.rs` | ~45 | direct stream composition | FW-CORE-001, X-BND-01-P2-03 |
| `TaskSubagent` trait + `TaskExecutionSummary` + `SuggestedTask` | `echo-orchestration/src/tasks/runtime.rs:296-331` | ~35 | `RuntimeDagController` | X-BND-01-P2-02 |
| `NotebookTracker` / `NotebookCell` / `enable_notebook` | `echo-agent/src/notebook.rs` | ~200 | none (aspirational) | FW-NBK-001 |
| `LoopDetector` + config plumbing | ReAct module | ~100 | `max_iterations` ceiling | FW-RCT-004 |
| `process_steps` + `execute_tool_feedback_*` + `ToolExecutionOutcome/Failure` | ReAct module | ~325 | `run_core_loop` | FW-RCT-005 |
| `AdapterClient` + `ProviderAdapter` | `adapter_client.rs` | ~80 | `OpenAiClient` | FW-LLM-011 |
| `DefaultLlmClient` | LLM module | ~40 | `OpenAiClient` | FW-LLM-012 |
| `HandoffManager` + `HandoffTool` | subagent module | ~200 | `SubagentEventBus` dispatch | FW-SUB-010/011/012 |
| `TopologyTracker` + `TopologyCallback` | subagent module | ~150 | none | FW-SUB-019..022 |
| `isolated.rs::run_isolated` | subagent module | ~100 | none | FW-SUB-013 |
| 5 dead Cargo features | `Cargo.toml` | config | none | FW-FEAT-001 |
| `validate_event_trajectory` (or wire it) | `event_envelope.rs:197` | ~100 | structural `break` in wrapper | X-EVT-01-P2-03 |
| ~50 `#[allow(dead_code)]` annotations | various | — | per-site audit | FW-QUAL-002 |

### 8.2 Application dead code (echo-agent-cli)

| Item | Location | Lines | Finding |
|---|---|---|---|
| `ProjectIndex` + cache | `project/index.rs` | 488 | APP-PROJ-P2-01 |
| `FileChangeTracker` / `CodingLoop.record_file_*` | `project/` | ~150 | APP-PROJ-P2-02 |
| `Persistence` (5 methods) + `SessionSearchEngine` | `persistence.rs`, `conversation_file.rs` | ~600 | APP-STATE-P2-01 |
| `output::OutputFormat` / `FormatContext` / `format_response` | `output/format.rs` | ~140 | APP-OUT-P2-01 |
| `LatexExporter` + `ResearchOutputFormat::Latex` + `Profile.output_format` | `export/latex.rs` | ~160 | APP-OUT-P2-02 |
| `auto_memory::run_auto_memory_extraction` | `auto_memory/mod.rs` | ~10 | APP-EVO-P2-01 |
| `IpcAuth` / `IpcPermission` / `require_full_auto` / `require_not_strict` | `tauri/error.rs` | ~70 | APP-TOOL-HITL-P2-02 |
| `SandboxConfigData.security_level` | `state.rs` | field | APP-TOOL-P2-01 |
| `parallel_tasks` Vec + `TaskStrip` widget + types | `tui/mod.rs` | ~150 | APP-SURF-TUI-P2-01 |
| `register_lifecycle` (pub API, test-only callers) | `plugin_runtime.rs:387-410` | ~25 | X-PLG-01-P3-01 |
| `add_artifact` / `Artifact` / `ArtifactProduced` / `list_task_artifacts` | `tasks/task_runtime/store.rs` | ~30 | X-STA-01-P3-01 |
| `eslint.config.js` (no ESLint installed) | `web-frontend/` | file | APP-FE-P3-13 |
| 5 orphaned generated TS files | `web-frontend/src/types/` | 5 files | APP-FE-P3-02 |
| `SubagentRunEventKind` `'artifact'` variant | `web-frontend/` | line | APP-FE-P3-05 |

**Total deletion target:** ~2,800 lines of dead code across both repos.

---

## 9. Implementation Milestones

Each milestone is scoped for a fresh task window (per AGENTS.md context
management: strong-coupling work stays in one window, atomic/deterministic
work moves to a new window). Estimated effort assumes one developer.

### Milestone 1: Tier-0 Data Integrity + Secrets + Panics (CRITICAL)

**Scope:** All 6 Tier-0 actions. Touches both repos but each change is
small and localized.

| Item | Repo | Finding |
|---|---|---|
| Secret redaction at 3 boundaries | both | FW-OPS-003, APP-OBS-P1-01/02 |
| globstar + IQR panic fixes | both | FW-QUAL-001, FW-TOOLS-003 |
| FileStore corrupt-JSON Err return | FW | FW-MEM-001 |
| atomic_write helper + file tools migration | FW | FW-TOOLS-002 |
| Hot-memory projection fix (8 sites) | CLI | APP-STATE-MEM-P1-01 |
| Anthropic streaming tool-call desync | FW | FW-LLM-001 |

**Effort:** 3-4 days. **Exit:** zero open P1 in secret/panic/data-loss class.
**Verification:** `cargo test --workspace --all-targets --all-features`;
add n=4 IQR test, Chinese-path globstar test, streaming fixture test.

### Milestone 2: Lifecycle + Cancellation (framework)

**Scope:** All 5 lifecycle/cancel P1s + streaming event-loss fix.

| Item | Repo | Finding |
|---|---|---|
| Team cancel propagation + timeout abort | FW | FW-SUB-001/002 |
| Scheduler graceful shutdown | FW | FW-OPS-001 |
| HttpTransport 202 fix | FW | FW-INT-001 |
| A2A sync cancel + terminal monotonicity | FW | FW-INT-002 |
| ReactAgent Cancelled emission | FW | FW-RCT-003 |
| yield_event_or! backpressure fix | FW | FW-RCT-001/002 |
| Trace finalization symmetry | FW | FW-RCT-006/008 |

**Effort:** 5-6 days. **Exit:** every cancellation path drains correctly;
streaming has no silent holes. **Merge echo-agent first.**

### Milestone 3: Surface Parity + Over-gating Removal (application)

**Scope:** All parity-blocking P1s + high-leverage P2 parity fixes.

| Item | Repo | Finding |
|---|---|---|
| CLI `/workspace switch` wiring | CLI | APP-SURF-CLI-P1-01 |
| MCP IPC over-gating + `IpcAuth` deletion | CLI | APP-TOOL-INT-P1-01, APP-TOOL-HITL-P2-02 |
| Missing API key fast-fail | CLI | APP-BOOT-P1-01 |
| Config watcher workspace-switch refresh | CLI | APP-BOOT-P1-02 |
| Channels-only services + cron resume + cancel | CLI | APP-BOOT-P2-02, X-SRF-01-P2-01/02/03 |
| GUI window close terminal cleanup | CLI | APP-SURF-GUI-P2-01 |
| Headless mode equivalence | FW | FW-OPS-002 |

**Effort:** 5-6 days. **Exit:** every mode has cancel, workspace switch,
MCP config, and graceful shutdown working.

### Milestone 4: Dead Infrastructure Sweep (both repos)

**Scope:** Batch deletion of all confirmed-dead surfaces (Section 8).

| Item | Repo | Finding |
|---|---|---|
| `GLOBAL_EVENT_BUS`, notebook, LoopDetector, process_steps | FW | FW-CORE-001, FW-NBK-001, FW-RCT-004/005 |
| `AdapterClient`, `DefaultLlmClient`, `TaskSubagent` | FW | FW-LLM-011/012, X-BND-01-P2-02 |
| `HandoffManager`, `TopologyTracker`, `run_isolated` | FW | FW-SUB-010/013/019 |
| 5 dead Cargo features | FW | FW-FEAT-001 |
| `ProjectIndex`, `Persistence`, output dead code, `auto_memory` | CLI | APP-PROJ-P2-01/02, APP-STATE-P2-01, APP-OUT-P2-01/02, APP-EVO-P2-01 |
| `parallel_tasks` scaffold, `SandboxConfigData.security_level` | CLI | APP-SURF-TUI-P2-01, APP-TOOL-P2-01 |

**Effort:** 3-4 days. **Exit:** ~2,800 lines removed; `cargo check` clean;
no misleading pub API remains.

### Milestone 5: Atomic-write Consolidation + Deletion Cascade

**Scope:** Complete the atomic_write 6-site migration and the
conversation-deletion cascade.

| Item | Repo | Finding |
|---|---|---|
| atomic_write: migrate FileStore + EmbeddingStore | FW | FW-MEM-002 |
| atomic_write: migrate analysis/research/tool_execution | CLI | X-BND-01-P2-01 |
| `delete_runs_for_conversation` + cascade helper | CLI | X-STA-01-P2-03, APP-STATE-P2-02 |
| events.jsonl partial-tail recovery | CLI | X-STA-01-P2-01 |
| Checkpoint corruption preservation + version field | FW | FW-RESUME-002, FW-WFL-004 |

**Effort:** 3-4 days. **Exit:** one atomic_write authority; deletion
cascade reaches every artifact directory.

### Milestone 6: ToolExecutionObserver + Frontend Contract

**Scope:** The keystone extraction for surface parity + frontend type safety.

| Item | Repo | Finding |
|---|---|---|
| Extract `ToolExecutionObserver` to driver level | CLI | APP-CHAT-P2-01 |
| Introduce `ExecutionEvent` typed enum | CLI | APP-SURF-GUI-P2-03 |
| Migrate tool-execution TS to `#[derive(TS)]` | CLI | X-TOL-01-P2-02 |
| `chatEventHandler.ts` exhaustiveness guard | CLI | X-EVT-01-P3-02 |
| `ToolsPanel` generated type fix | CLI | APP-FE-P2-01 |
| MessageBubble render optimization | CLI | APP-FE-P2-02 |
| Shared Modal primitive (focus trap, a11y) | CLI | APP-FE-P2-03 |

**Effort:** 5-6 days. **Exit:** sinks are pure render; frontend has
compile-time wire contract.

### Milestone 7: Budget + Compression + Remaining P2

**Scope:** Context-budget correctness, compression-cycle fixes, chat-mode
resume, collision determinism, LSP surface, and remaining P2 convergence.

| Item | Repo | Finding |
|---|---|---|
| Budget accounting (tool defs + system prompt + protected) | FW | FW-CTX-001/002/004 |
| Summary accumulation unbounded fix | FW | FW-CMP-001 |
| Chat-mode cross-process resume | FW | FW-RESUME-001 |
| Collision determinism (both loaders) | FW | X-PLG-01-P2-01 |
| LSP restart surface (TUI + Tauri) | CLI | X-SRF-01-P2-05 |
| TUI subagent detail extension | CLI | X-SRF-01-P2-04 |
| Cross-layer tool-failure fixtures | CLI | X-TOL-01-P2-03 |
| Dependency convergence (`hashbrown`, `quick-xml`) | both | FW-QUAL-005/007 |

**Effort:** 4-5 days. **Exit:** budget never phantom-exceeds the window;
compression doesn't accumulate; all loaders deterministic.

---

## 10. Verification Strategy

### 10.1 Per-milestone gate (mandatory before merge)

```
# Framework (echo-agent)
cargo fmt --all && cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo clippy --workspace --lib --bins --all-features --locked -- \
  -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::unreachable
cargo test --workspace --all-targets --all-features --locked

# Application (echo-agent-cli)
cargo fmt --all && cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
```

### 10.2 Regression tests to add (per fix)

| Fix | Regression test |
|---|---|
| FW-TOOLS-003 (IQR) | `outlier_detection` with n=4 numeric column |
| FW-QUAL-001 (globstar) | `globstar_match("z**", "中文模块")` no panic |
| FW-LLM-001 (Anthropic) | Streaming fixture: text block precedes tool_use |
| FW-MEM-001 (FileStore) | Truncated JSON file -> `Err`, not empty map |
| FW-RCT-001 (backpressure) | Slow consumer + 256+ events: no drops |
| APP-STATE-MEM-P1-01 | MEMORY.md edit -> `eko:hot-memory-context` refreshed |
| APP-TOOL-INT-P1-01 | `localhost:8100/mcp` accepted via IPC |

### 10.3 Conformance fixtures (cross-cutting)

- Streaming/non-streaming event identity conformance (B-REF-01 C6)
- Cancelled-vs-Error terminal monotonicity per surface
- Tool-failure taxonomy round-trip (framework -> EKO -> frontend)
- Conversation deletion cascade reaches all 6 artifact directories

---

## 11. What Is Clean (do not re-litigate)

These structural invariants were confirmed by the review and should NOT be
revisited during implementation:

- **One revisioned TaskRun graph** with thin EKO adapter; AGENTS.md rule 6
  holds end-to-end. No parallel task/plan/todo CRUD.
- **All adapters are thin** — no EKO adapter owns scheduling/DAG/retry/
  validator/deadlock authority.
- **Single ReAct loop body** (`run_core_loop`); streaming and non-streaming
  share one authoritative loop.
- **Single tool registry** with idempotent replacement; the historical
  `todo_write` is gone.
- **`SqliteStore`/`SqliteConversationStore`/`HybridCompressor`/
  `EmbeddingStore`/`InMemoryStore`** are legitimate framework menu options,
  NOT deletion targets (AGENTS.md "框架 API 删了,复用方的代码会断").
- **Tool-pair protocol validity** guaranteed by single
  `sanitize_tool_call_pairing` choke point.
- **Plugin/skill/hook lifecycle is fully reversible** (discover -> prepare
  -> activate -> use -> reload -> unload, each with a defined inverse).
- **Per-attempt subagent identity is deterministic**
  (`{run_id}:{task_id}:{revision}:{attempt}`).
- **Two-lane architecture is sound** (chat lane via `drive_chat`, task lane
  via `execute_run`). No third lane.
- **Agent-vs-user permission boundary is correctly separated.**

---

## Appendix: Cross-Reference to Source Syntheses

| This roadmap section | Source synthesis |
|---|---|
| Section 2 (P1 list) | S-FW-01 section 3 + 6; S-APP-01 section 3 + 6; S-X-01 section 7 |
| Section 3-6 (Tiers) | S-FW-01 section 6 (Tiers A-D); S-APP-01 section 6 (Tiers 0-4); S-X-01 section 10 (Tiers A-D) |
| Section 7 (DAG) | S-X-01 section 5 (duplicate-authority map); S-FW-01 section 4 (cross-cutting patterns) |
| Section 8 (Deletion) | S-FW-01 section 4.1; S-APP-01 section 4.2; S-X-01 section 5.2 |
| Section 9 (Milestones) | Sequenced from all three action lists + B-REF-01 convergence constraints |
| Section 11 (Clean) | S-FW-01 section 7; S-APP-01 section 4.5; S-X-01 section 11 |

### Conditions that invalidate this roadmap

- Any baseline commit change underneath `echo-agent` `9b0e0fa` or
  `echo-agent-cli` `b3b2e81` requires re-running affected source reports'
  validations before trusting the roadmap's finding references.
- Resolving any P1 finding invalidates its row in Section 2 and its
  milestone assignment in Section 9.
- Adding a new `AgentEvent`/`ChatEvent`/`SubagentEvent`/`ToolFailureCategory`
  variant requires re-evaluating the parity matrix and event-conformance
  fixtures.
- Adding a new persistence boundary requires re-evaluating the secret-leak
  cluster (Section 2.1).
- A new mode added (e.g. `--daemon`, plugin channel) requires a new column
  in the surface-parity matrix and a new milestone-3 entry.
