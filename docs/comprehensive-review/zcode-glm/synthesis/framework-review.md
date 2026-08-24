# S-FW-01: Framework Review Synthesis (echo-agent)

> Synthesis task: S-FW-01
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Baseline: `echo-agent` `9b0e0fa`, `echo-agent-cli` `b3b2e81`
> Sources synthesized: 38 F-phase task reports (`F-*.md`) + `Q-STA-01.md` + `Q-DEP-01.md`
> Synthesis date: 2026-08-12

This document merges, deduplicates, and prioritizes every finding produced by
the framework review phase. Canonical IDs retain backlinks to the originating
task reports. Contradictions between reports are resolved inline. The action
list at the end sequences the work.

---

## 1. Finding Count Summary

| Priority | Count | Scope |
|---|---:|---|
| P0 | 0 | — |
| P1 | 16 | 15 framework + 1 application-layer (Q-STA-01) |
| P2 | 87 | 82 framework + 5 cross-cutting/quality (Q-STA-01, Q-DEP-01) |
| P3 | 113 | 110 framework + 3 quality |
| **Total** | **216** | |

Per-report breakdown (reports with findings):

| Report | P1 | P2 | P3 | Total |
|---|---:|---:|---:|---:|
| F-OPS-01 | 3 | 4 | 3 | 10 |
| F-EXT-02 | 2 | 5 | 4 | 11 |
| F-SUB-02 | 2 | 3 | 3 | 8 |
| F-LLM-03 | 1 | 5 | 4 | 10 |
| F-MAC-01 | 0 | 3 | 8 | 11 |
| F-INT-02 | 1 | 3 | 4 | 8 |
| F-LLM-02 | 0 | 4 | 4 | 8 |
| F-API-01 | 0 | 3 | 5 | 8 |
| F-WFL-01 | 1 | 3 | 4 | 8 |
| F-MEM-02 | 0 | 3 | 4 | 7 |
| F-SUB-01 | 0 | 4 | 3 | 7 |
| F-MAG-01 | 0 | 3 | 4 | 7 |
| F-EXT-03 | 1 | 1 | 4 | 6 |
| F-CORE-01 | 0 | 2 | 4 | 6 |
| F-CTX-01 | 0 | 4 | 2 | 6 |
| F-INT-01 | 1 | 3 | 2 | 6 |
| F-NBK-01 | 1 | 1 | 4 | 6 |
| F-RCT-03 | 1 | 2 | 2 | 5 |
| F-CMP-01 | 0 | 3 | 2 | 5 |
| F-SEC-01 | 0 | 2 | 3 | 5 |
| F-SKL-01 | 0 | 2 | 3 | 5 |
| F-TSK-03 | 0 | 2 | 3 | 5 |
| F-TST-01 | 0 | 3 | 2 | 5 |
| Q-STA-01 | 1 | 3 | 1 | 5 |
| F-RCT-01 | 0 | 0 | 4 | 4 |
| F-RCT-02 | 0 | 3 | 1 | 4 |
| F-RCT-05 | 0 | 2 | 2 | 4 |
| F-HITL-01 | 0 | 1 | 3 | 4 |
| F-INTENT-01 | 0 | 1 | 3 | 4 |
| F-LLM-01 | 0 | 2 | 2 | 4 |
| F-MEM-01 | 1 | 2 | 2 | 5 |
| F-RCT-04 | 0 | 1 | 2 | 3 |
| F-FEAT-01 | 0 | 1 | 2 | 3 |
| Q-DEP-01 | 0 | 1 | 2 | 3 |
| F-EXT-01 | 0 | 0 | 2 | 2 |
| F-EVO-01 | 0 | 1 | 1 | 2 |
| F-PLG-01 | 0 | 1 | 1 | 2 |
| F-REL-01 | 0 | 0 | 2 | 2 |
| F-TSK-01 | 0 | 0 | 1 | 1 |
| F-TSK-02 | 0 | 0 | 1 | 1 |

**No P0 findings.** The review surfaced no data-corruption-with-secret-exposure
or unrecoverable-system-level-defect class issues; the worst defects are P1
(capability failure on valid input, silent data loss on crash, leaked
resources). Reports `F-TST-01`, `F-REL-01`, `F-EXT-01` are close to clean
(only low-priority observations).

---

## 2. Stale-Finding Check

All 38 F-phase reports and both Q-phase reports were generated against the same
baseline commits (`echo-agent` `9b0e0fa`, `echo-agent-cli` `b3b2e81`). No
reviewed commit changed underneath any report. **Zero findings are marked
stale.** Each report's "Conditions that make this report stale" clause remains
the forward-looking invalidation contract; none have triggered yet.

The one cross-report correction worth recording: `F-MEM-01` hypothesised (in
its Handoff) that `SqliteStore` likely satisfies the crash-durability contract
`FileStore` is missing. `F-MEM-02` **confirmed** that hypothesis (WAL +
`synchronous=NORMAL` + per-op transactions). This is recorded as a resolved
contradiction under the Memory subsystem below, not a stale finding.

---

## 3. Findings by Subsystem (P0 → P1 → P2 → P3)

Findings are grouped into subsystems. Within each subsystem, items are ordered
P1 → P2 → P3. Duplicate / overlapping findings from multiple reports are merged
under a canonical ID with backlinks to the originals.

### 3.1 LLM Provider Layer

#### P1

**FW-LLM-001 (was F-LLM-03-P1-01): Anthropic streaming tool-call index/key desync drops every tool call whenever any non-tool-use block precedes a tool_use block**
- Layer: framework (`echo-integration/src/providers/anthropic.rs:520-583`)
- Backlinks: F-LLM-03-P1-01
- Defect: `ContentBlockStart(ToolUse)` inserts into `tool_call_args` at key
  `tool_call_args.len()` (a dense tool counter), but `ContentBlockDelta`/`Stop`
  look up by the event's content-block `index` (which counts text + thinking +
  tool_use blocks). When text precedes a tool_use (the common Claude agentic
  case — "Let me check..."), the keys diverge and every tool call is silently
  dropped. The stream then reports `finish_reason="tool_calls"` with zero tool
  calls. The `_index` field on `ContentBlockStart` was renamed to suppress the
  unused-warning rather than fix the desync.
- Impact: severe — every Claude streaming agentic flow that emits text
  alongside tool calls silently loses the tool call. No test covers it.
- Fix: rename `_index` → `index` and use it as the insertion key.

#### P2

**FW-LLM-002 (merged): Anthropic thinking blocks are never surfaced through the neutral contract — observational asymmetry vs OpenAI-family reasoning models**
- Backlinks: F-LLM-01-P2-01 (contract gap), F-LLM-03-P2-01 (adapter gap)
- Defect: `ProviderCapabilities::anthropic().reasoning_content = false`; the
  adapter `ContentBlock` enum has no `Thinking` variant; non-streaming hard-
  codes `reasoning_content: None`; streaming deltas hard-code `None`;
  `thinking_delta`/`signature_delta` events deserialize to null and are
  dropped. OpenAI-compatible reasoning models (GPT-5, DeepSeek-r1, Qwen3)
  faithfully stream `reasoning_content`; Anthropic is silent.
- Resolution: contract is fine (field exists); fix belongs to the adapter
  (add `Thinking` variant, map to `reasoning_content`, flip capability flag).

**FW-LLM-003 (merged): Signed thinking blocks cannot round-trip — multi-turn extended-thinking-with-tool-use is unreachable on Anthropic**
- Backlinks: F-LLM-01-P2-02 (contract), F-LLM-03-P3-02 (adapter)
- Defect: `Message::reasoning_content: Option<String>` is a flat string and
  cannot carry Anthropic's per-block `signature`; `ContentBlock` has no
  request-side `Thinking` variant. Multi-turn thinking replay is impossible.
- Note: latent today (masked by FW-LLM-002); becomes the next bottleneck once
  FW-LLM-002 is fixed.

**FW-LLM-004 (merged): `ChatResponse` has no top-level `usage` field — non-streaming consumers must reach into `raw.usage` (asymmetric with streaming `ChatChunk.usage`)**
- Backlinks: F-LLM-02-P2-04, F-LLM-03-P3-04
- Defect: streaming exposes `chunk.usage`; non-streaming hides it in
  `response.raw.usage`. The React engine already works around it
  (`react_loop.rs:100`). Every new non-streaming consumer rediscovers the trap.

**FW-LLM-005 (was F-LLM-03-P2-02): Non-streaming Anthropic response hard-fails on any unknown content-block type (thinking, redacted_thinking, future blocks)**
- The streaming types have `#[serde(other)]`; the non-streaming `ContentBlock`
  does not. An unknown block type fails deserialization, surfaces as
  `LlmError::NetworkError` (mislabeled — see FW-LLM-010), and is treated
  retryable so the call retries uselessly against an unchanged shape. Combined
  with FW-LLM-002, the adapter cannot handle thinking-enabled Claude responses
  on either path.

**FW-LLM-006 (was F-LLM-03-P2-03): Streaming path omits `anthropic-beta: prompt-caching-2024-07-31` header — cache-control behavior diverges across paths**
- The non-streaming send includes the beta header; the streaming send does
  not, while both emit `cache_control` markers in the shared body. Streaming
  cache hit-rate observability is unreliable; on strict API accounts the
  streaming path may 400. Root cause: header set was copy-pasted and the beta
  line was dropped; no streaming HTTP test exists.

**FW-LLM-007 (was F-LLM-03-P2-04): `ChatRequest.tool_choice` is silently dropped on the Anthropic path**
- `AnthropicRequest` has no `tool_choice` field; `convert_request` never reads
  it. The production caller (`think.rs`) is gated off by
  `supports_tool_choice_none=false`, so impact is latent today; future callers
  that bypass the capability gate get silent no-op.

**FW-LLM-008 (was F-LLM-03-P2-05): `ChatRequest.response_format` is silently dropped — Anthropic structured-output translation absent**

**FW-LLM-009 (was F-LLM-02-P2-03): Malformed SSE chunks are silently dropped at the transport layer**
- `parse_sse_chunk` on serde failure: `warn!` + return `None`; the stream
  continues. Neither adapter, React engine, nor caller can distinguish
  "provider sent fewer chunks" from "provider sent an unparseable chunk".

**FW-LLM-011 (was F-LLM-02-P2-01): `AdapterClient` + `ProviderAdapter` are dormant — zero implementors, zero consumers, doc-comment claims routing it does not perform**
- Pub-exported abstraction whose doc claims it routes DeepSeek/GLM/Kimi/Qwen;
  the live path routes all of them through `OpenAiClient` +
  `translate_thinking_openai_compat` keyed on `provider_name`. A contributor
  reading `adapter_client.rs` would extend the wrong abstraction.
- Resolution per AGENTS.md framework-delete test: this is the ✅ branch 1
  (superseded internal dead code with pub surface). Either wire it up for one
  real provider or delete it; do not leave the misleading doc.

**FW-LLM-012 (was F-LLM-02-P2-02): `DefaultLlmClient` is dormant — pub-exported, never constructed, and silently bypasses thinking translation**
- Named as if it were the default; the real default is `OpenAiClient`. If a
  downstream consumer picks it up by name, it silently drops thinking config
  with only a `warn!`. Superseded → deletion candidate.

#### P3

- **FW-LLM-013 (F-LLM-01-P3-01)**: `ChatRequest.tool_choice` is stringly-typed OpenAI wire format, not the typed `ToolChoice` enum.
- **FW-LLM-014 (F-LLM-01-P3-02)**: Two overlapping thinking-protocol enums at different layers (`ThinkingProtocol` framework vs `ThinkingProtocolPreference` transport) with edge disagreements.
- **FW-LLM-010 (F-LLM-03-P3-03)**: Body parse failures mislabeled as `LlmError::NetworkError` instead of `InvalidResponse`, causing useless retries.
- **FW-LLM-015 (F-LLM-02-P3-01)**: `[DONE]` stream terminator encoded as a sentinel string inside `LlmError::NetworkError`, detected by `to_string().contains(sentinel)` — brittle to Display changes.
- **FW-LLM-016 (F-LLM-02-P3-02)**: `cache_hints` silently dropped on every OpenAI-compat path (no log, no doc).
- **FW-LLM-017 (F-LLM-02-P3-03)**: `choices.first()` silently ignores non-first choices (no `n>1` today).
- **FW-LLM-018 (F-LLM-02-P3-04)**: Empty-string tool-call arguments (`""`) cause the call to be silently dropped (should resolve to `{}`).
- **FW-LLM-019 (F-LLM-03-P3-01)**: Multiple `Role::System` messages collapse to the last; earlier system messages silently dropped on Anthropic.

---

### 3.2 ReAct Engine (Loop, Streaming, Snapshot, Steer, Tools)

#### P1

**FW-RCT-001 (was F-RCT-03-P1-01): Non-terminal streaming events are silently dropped under backpressure — live UX rendering is corrupted**
- Layer: framework (`stream_macros.rs:38-53`, 16 production sites)
- Backlinks: F-RCT-03-P1-01
- Defect: `yield_event_or!` uses `tx.try_send`; on `Full` it `warn!`s and drops
  the event. 16 intermediate-event sites use it (`Token(reasoning/content)`,
  `ThinkStart/End`, `LlmUsage`, `ToolBatchStart`, `ToolCall`, `ToolResult`,
  `ToolError`, `MemoryRecalled`, `ContextCompressed`, `ToolBatchEnd`). The
  agent's internal buffers stay correct, but the consumer's live stream has
  holes. Reasoning models (Qwen3/DeepSeek) can fill 256 slots in one think
  phase. The lossy contract is documented at `config.rs:111` but defeats the
  purpose of streaming.
- Fix: change `yield_event_or!` to `send().await` (backpressure is the desired
  behavior — a slow consumer should slow the producer, not receive garbled
  output), or raise the buffer as a stopgap.

#### P2

**FW-RCT-002 (was F-RCT-03-P2-01): Terminal error events (`Err(NoResponse)`, `Err(MaxIterations)`, intervention cancel/block) use `try_send` and can be silently dropped, leaving the consumer with no terminal signal**
- 5 sites (`finalize.rs:226,267`, `think.rs:47,65`, `stream_channel.rs:310`).
  On a full buffer the error is dropped, the task exits, the channel closes,
  and the consumer sees a clean stream end. The non-streaming collector
  returns `Ok("")` for a turn that actually hit `MaxIterationsExceeded` — a
  correctness defect (cannot distinguish empty success from dropped failure).

**FW-RCT-003 (was F-RCT-03-P2-02): `ReactAgent` never emits `AgentEvent::Cancelled` — the trait's cancellation terminal is bypassed by the override**
- ReactAgent overrides `chat_stream_with_cancel`/`execute_stream_with_cancel`
  and does NOT wrap with `cancel_aware_stream`. Cancellation is handled inside
  the loop (5s grace), but the documented `Cancelled` terminal is never
  emitted. The non-streaming arm `Ok(AgentEvent::Cancelled) => Ok("Cancelled.")`
  is dead code. Consumers must poll `cancel.is_cancelled()` separately.

**FW-RCT-004 (was F-RCT-02-P2-01): `LoopDetector` + config plumbing are dead infrastructure that advertises loop detection the runtime never performs**
- Structurally identical to FW-CORE-001 (GLOBAL_EVENT_BUS). `LoopDetector::new`
  is called only inside `#[cfg(test)]`; the builder method has zero callers;
  `run_core_loop` never references it. The only loop protection is the hard
  `max_iterations=100` ceiling + optional soft budgets. An agent calling the
  same read-only tool 99 times burns 99 LLM rounds undetected.
- Resolution: delete (preferred under cleanup rule) or wire it.

**FW-RCT-005 (was F-RCT-02-P2-02): `process_steps` + `execute_tool_feedback_raw/_helper` + `ToolExecutionOutcome/Failure` are dead code superseded by the unified `run_core_loop`**
- ~325 lines of non-streaming tool-batch implementation retained with
  `#[allow(dead_code)]`. It duplicates `phases::tools::run_tools` semantics
  but bypasses the 13-stage pipeline and cancellation grace. Two stale module
  docstrings still reference it.

**FW-RCT-006 (was F-RCT-02-P2-03): `finalize_completed_run` (tool-branch success) does not finalize the trace run — the primary happy path leaves `run.status=Running` forever**
- The text-success (`emit_final_text`) and both failure helpers call
  `snap.finalize_run(...)`; the tool-success path omits it. Every successful
  tool-based turn (the normal case for a tool-using agent) leaves the trace
  run stuck in `Running`. Dashboards counting active runs are wrong.

**FW-RCT-007 (was F-RCT-04-P2-01): Concurrent batch timeout returns Abandoned without emitting `ToolBatchEnd` or saving a checkpoint (asymmetric with every other terminal path)**
- 6 other terminal arms in `run_tools` pair `ToolBatchEnd` + checkpoint; the
  timeout arm uses `try_send_or!` and returns immediately. Consumers that
  gate "batch finished" on `ToolBatchEnd` hang; completed tool results are not
  persisted to `RuntimeStateStore`, so resume after timeout loses them.

#### P3

- **FW-RCT-008 (F-RCT-02-P3-01)**: Abandoned/Cancelled/Blocked arms also skip trace finalization (compounds FW-RCT-006).
- **FW-RCT-009 (F-RCT-03-P3-01)**: Stop-hook continuation can emit two `FinalAnswer` events on streaming; non-streaming returns the first. Narrow but real semantic divergence.
- **FW-RCT-010 (F-RCT-03-P3-02)**: Content `Token` events are batched on the full-ReAct path but per-chunk on DirectAnswer — inconsistent streaming granularity.
- **FW-RCT-011 (F-RCT-04-P3-01)**: Concurrent tool results are inserted into history in completion order (nondeterministic across runs) — eval/benchmark noise source.
- **FW-RCT-012 (F-RCT-04-P3-02)**: `VerifyThenRetry` recovery is advisory metadata — no framework verification gate runs before the model retries (prompt-driven design; document).
- **FW-RCT-013 (F-RCT-01-P3-01)**: Feature-gated builder methods (`enable_human_in_loop`, `enable_subagent`, `register_agent_dispatch_tool`) silently no-op when the Cargo feature is disabled.
- **FW-RCT-014 (F-RCT-01-P3-02)**: Project rules are duplicated in context after compression (once in system prompt, once as `[Canonical context — project rules restored]`).
- **FW-RCT-015 (F-RCT-01-P3-03)**: Five builder options bypass `AgentConfig` and write directly to `ReactAgent` internals.
- **FW-RCT-016 (F-RCT-01-P3-04)**: `enable_planning()` builder method is misnamed (toggles `enable_task`); `enable_task` docstring is misleading.
- **FW-RCT-017 (F-RCT-05-P3-01)**: Replay protection is structural (pairing validation), not an idempotency gate — `completed_tool_call_ids` is trace-only.
- **FW-RCT-018 (F-RCT-05-P3-02)**: In-memory `StateSnapshot` captures only messages, not full agent state; the name oversells scope (opt-in, default off).

---

### 3.3 Snapshot / Resume / Cancellation

#### P2

**FW-RESUME-001 (was F-RCT-05-P2-01): Chat-mode turns never restore from `RuntimeStateStore` — cross-process chat resume silently starts from empty context**
- `run_chat_direct` and the `StreamMode::Chat` arms skip
  `restore_thread_context`. A user who closes and reopens the app, then sends
  a chat message in an existing conversation, gets a response computed from
  empty context. The next turn's checkpoint then overwrites the richer
  pre-restart checkpoint. The Execute path resumes correctly; only Chat is
  broken. This breaks the AGENTS.md "TUI/GUI feature parity / Claude-Code-like
  continuity" expectation for the most common interaction mode.

**FW-RESUME-002 (was F-RCT-05-P2-02): Corrupt or schema-incompatible checkpoint is silently swallowed and then destroyed (compounds FW-MEM-001)**
- `restore_thread_context`'s `Err` arm: `warn!` + `reset_messages()`. The
  store layer correctly returns `Err`; the consumer suppresses it, then the
  next successful turn's `save_checkpoint` overwrites the corrupt file.
  Original corruption is unrecoverable. No `version` field on `AgentCheckpoint`
  means version skew is indistinguishable from corruption. Sister to
  FW-MEM-001 (FileStore) — same silent-data-loss pattern, worse because the
  overwrite destroys evidence.

---

### 3.4 Tools (Contract + Builtin File/Git/Data tools)

#### P1

**FW-TOOLS-001 (was F-EXT-02-P1-01): Worktree `path_suffix` traversal writes outside `.worktrees/`**
- `Path::join(".worktrees").join("../evil-target/wt")` produces the literal
  escaped path and `git worktree add` accepts it. Combined with
  `remove_worktree`'s unconditional `--force` + `git branch -D`
  (FW-TOOLS-005), the escaped worktree can later be force-removed, discarding
  user files. This is exactly the "防止用户无意中的数据丢失" category AGENTS.md
  endorses as a legitimate local safety concern.

**FW-TOOLS-002 (was F-EXT-02-P1-02): No file tool performs atomic (crash-safe) writes**
- `WriteFileTool`, `UpdateFileTool`, `EditFileTool`, `CreateFileTool` all use
  `tokio::fs::write` (O_WRONLY|O_CREAT|O_TRUNC). A crash between truncate and
  final fsync leaves the file truncated/partial. `EditFileTool` creates a
  `.bak` but the live file is still corrupted on crash and the `.bak` is never
  cleaned up; `WriteFileTool`/`UpdateFileTool` do neither. This is the
  cross-cutting "non-atomic file operations" pattern (see §4) on the
  user-data path.

**FW-TOOLS-003 (was F-EXT-03-P1-01): `outlier_detection` (IQR method) panics on any numeric column with exactly four finite values**
- `sorted[n / 4.min(n - 1)]` — due to method-call precedence, `4.min(n-1)`
  binds to the divisor, not the index. For `n==4`: divisor = `min(4,3)=3`,
  `q3_idx = (3*4)/3 = 4`, `sorted[4]` is out of bounds → panic (exit 101).
  Violates AGENTS.md "禁止任何会导致系统 panic 的 API". Existing test uses
  `n==9` so the bug stayed latent. Confirmed by isolated `rustc` reproduction.

#### P2

- **FW-TOOLS-004 (F-EXT-02-P2-01)**: `UpdateFileTool` duplicates `EditFileTool` with strictly fewer safety features (no multi-occurrence gate, no dry_run, no .bak, no git checkpoint). Parallel implementation of the same semantics.
- **FW-TOOLS-005 (F-EXT-02-P2-03)**: `exit_worktree` missing-worktree fallback force-deletes a namesake branch (data loss).
- **FW-TOOLS-006 (F-EXT-02-P2-04)**: `merge_worktree` leaves the repo in MERGE state on conflict and disrupts concurrent workers.
- **FW-TOOLS-007 (F-EXT-02-P2-05)**: `create_worktree` lacks `--` separator after `-b <branch>`, enabling flag-shaped branch names.
- **FW-TOOLS-008 (F-EXT-02-P2-02)**: Worktree tools ignore the agent's `working_dir`.
- **FW-TOOLS-009 (F-EXT-03-P2-01)**: Research search tools return collections but expose no cursor pagination, unlike `web_search`/`sql_query`.

#### P3

- **FW-TOOLS-010 (F-EXT-02-P3-01)**: `cleanup_direct_child` invokes synchronous `kill` in an async context.
- **FW-TOOLS-011 (F-EXT-02-P3-02)**: `EditFileTool::find_occurrence_lines` uses byte slicing on `&str` (safe today but violates AGENTS.md UTF-8 guidance).
- **FW-TOOLS-012 (F-EXT-02-P3-03)**: `git_checkpoint` runs synchronous git subprocesses inside async tool bodies.
- **FW-TOOLS-013 (F-EXT-02-P3-04)**: `git_checkpoint` tag name uses second-resolution timestamps, colliding under concurrent writers.
- **FW-TOOLS-014 (F-EXT-03-P3-01)**: `DataProfileTool` sample variance divides by zero for single-row numeric columns, silently emitting `null` stddev/variance.
- **FW-TOOLS-015 (F-EXT-03-P3-02)**: Research HTTP failures returned as raw `ToolError::ExecutionFailed`, not classified as retryable `ToolFailure`.
- **FW-TOOLS-016 (F-EXT-03-P3-03)**: `ImageFetchTool::is_image_url` (dead helper) bypasses the SSRF-safe connect path.
- **FW-TOOLS-017 (F-EXT-03-P3-04)**: `rag_index` advertises an `overlap` parameter but never applies it.
- **FW-TOOLS-018 (F-EXT-01-P3-01)**: `ToolResult` exposes parallel output channels (`output`, `data`, `bytes`, `kind`) without documenting which is authoritative per `kind`.
- **FW-TOOLS-019 (F-EXT-01-P3-02)**: Default `ToolFailureCategory → ToolRecoveryAction` mapping is hardcoded and not per-tool overridable without bypassing `ToolFailure::new`.

---

### 3.5 Memory & Conversation Stores

#### P1

**FW-MEM-001 (was F-MEM-01-P1-01): `FileStore` silently swallows corrupt JSON on load (data loss)**
- `FileStore::new` parses with `serde_json::from_str(&raw).unwrap_or_else(|e|
  { warn!(...); HashMap::new() })`. A truncated/malformed file is
  indistinguishable from a fresh install. The next mutation `flush()`es the
  now-empty map over the corrupt file via rename, destroying recovery chances.
  Sister `FileConversationStore` documents and enforces "Corrupt JSON is an
  error". Silent permanent loss of all long-term memory in that file.

#### P2

**FW-MEM-002 (merged): Atomic-write recipe is inconsistent across backends — `FileStore` and `EmbeddingStore` omit parent-directory fsync and use static temp names**
- Backlinks: F-MEM-01-P2-01 (parent-dir fsync), F-MEM-01-P2-02 (static temp names)
- `FileStore::flush` and `EmbeddingStore::flush_index` fsync the temp and
  rename, but do not call `sync_parent_directory`. On Linux ext4 (default
  mount) the rename may not reach disk before a crash. They also derive the
  temp name purely from the final path (`{path}.tmp`), so cross-instance or
  cross-process overlap races on the same temp file. `FileConversationStore`
  has the correct recipe (uuid temp + parent-dir sync).
- Fix: factor one shared `atomic_write(path, bytes)` helper and route all
  three through it. (Cross-references FW-TOOLS-002 — the file tools need the
  same helper.)

**FW-MEM-003 (was F-MEM-02-P2-01): `SqliteStore::prune_expired` orphans FTS5 and vector index entries (violates the three-table lockstep `delete` upholds)**
- `delete` wraps all three DELETEs in one transaction; `prune_expired` issues
  a single `DELETE FROM store_items` and returns. Orphaned FTS/vector rows
  accumulate forever, wasting query budget and compute.

**FW-MEM-004 (was F-MEM-02-P2-02): `SqliteConversationStore` omits `busy_timeout` PRAGMA (concurrency asymmetry with `SqliteStore`)**
- `SqliteStore` waits up to 5000ms on lock contention; the conversation store
  returns `SQLITE_BUSY` immediately. No documented reason for the divergence.

**FW-MEM-005 (was F-MEM-02-P2-03): `SqliteStore` uses `std::sync::Mutex` with synchronous rusqlite I/O (executor-blocking, `!Send`-guard footgun); `SqliteConversationStore` uses `tokio::sync::Mutex`**
- The `MutexGuard` is `!Send`; the code carefully avoids holding it across
  `.await` today, but there is no compile-time guard. A maintainer who adds an
  `.await` under the guard silently makes the future `!Send`, breaking
  multi-threaded tokio at runtime. Neither store wraps rusqlite work in
  `spawn_blocking`.

#### Resolved contradiction

- `F-MEM-01` hypothesised SQLite satisfies the durability contract `FileStore`
  lacks. **`F-MEM-02` confirmed**: WAL + `synchronous=NORMAL` + per-op
  transactions. Framework consumers needing durable concurrent memory should
  prefer `SqliteStore`; `FileStore` remains the lightweight single-file option
  (and the CLI's choice). No stale finding — hypothesis upgraded to confirmed
  conclusion.

#### P3

- **FW-MEM-006 (F-MEM-01-P3-01)**: `tokenize` filters tokens by byte length (`s.len() > 1`), inconsistent with UTF-8 rule — single-char CJK (3 bytes) passes, single-char ASCII (1 byte) doesn't.
- **FW-MEM-007 (merged F-MEM-01-P3-02 + F-MEM-02-P3-04)**: Empty-query behavior diverges across the three `Store` impls (`SqliteStore` returns empty; `FileStore`/`InMemoryStore` return all with score 1.0). Trait never fixed the semantics.
- **FW-MEM-008 (F-MEM-02-P3-01)**: `cosine_similarity` propagates NaN from embedder output; `bytes_to_vec` silently truncates non-4-aligned blobs.
- **FW-MEM-009 (F-MEM-02-P3-02)**: `SqliteConversationStore` is not re-exported from the `echo_agent` facade (asymmetry with `SqliteStore`).
- **FW-MEM-010 (F-MEM-02-P3-03)**: Schema migration swallows non-"duplicate column" errors; the migrated columns are written but never read (JSON is source of truth on read).

---

### 3.6 Context Budget & Compression

#### P2

**FW-CTX-001 (was F-CTX-01-P2-01): Tool definitions and system prompt are not accounted against the budget; per-category reservations are phantom**
- `budget.allocate` is always called with `system_size=0, tool_defs_size=0`.
  The 15% reserved for system+tools sits empty while the system prompt
  (inside `messages`) silently consumes conversation budget and tool defs
  consume nothing. For MCP-heavy EKO (30K of tool schemas + 8K system prompt)
  the prepared request routinely exceeds the real window, tripping provider
  400 / `context_length_exceeded`.

**FW-CTX-002 (was F-CTX-01-P2-02): Protected content survives compression but its token cost is not deducted from the compressor's effective limit**
- `effective_limit` is derived from `estimated_tokens` (which includes
  protected messages), but the compressor is asked to fit only the
  compressible subset. Protected content is merged back on top. A large
  protected block (e.g. 30K-token subagent brief) blows the real window.

**FW-CTX-003 (was F-CTX-01-P2-03): `infer_context_window` ignores its `provider` argument despite the name and docstring**

**FW-CTX-004 (was F-CTX-01-P2-04): Unknown-model fallback `396_000` is larger than the real windows of the most common 128K/200K models**
- An oversized fallback hides context-overflow failures: the budget believes
  it has 396K when the real model has 128K, so compression never fires and the
  request 400s at the provider.

**FW-CMP-001 (was F-CMP-01-P2-01): Summary system messages accumulate across repeated compression cycles — long sessions re-fill the window with accumulated summaries**
- `SummaryCompressor`/`IncrementalSummaryCompressor` partition by
  `Role::System`, so the previous `[对话历史摘要]` survives in `system_msgs`
  and the new summary is appended. After N cycles the system region has N
  summaries (~500-2000 tokens each). The YAML default strategy is `summary`,
  so this is capability degradation on the core chat path under the default
  config. Compounds with FW-CTX-001/002 (the accumulated summaries are
  themselves unbounded).

**FW-CMP-002 (was F-CMP-01-P2-02): Compressors hard-code `HeuristicTokenizer`, diverging from the ContextManager's configured `CalibratedTokenizer`**
- The decision-to-compress path uses the calibrated tokenizer; the compressor
  internals use the uncalibrated heuristic. Adaptive thresholds set as
  percentages of the real window are compared against the wrong base; Hybrid
  short-circuit fires at the wrong boundary; checkpoint token counts differ
  from `/context` estimates.

**FW-CMP-003 (was F-CMP-01-P2-03): `current_query` is plumbed through `CompressionInput` but unused by all compressors — no query-aware eviction protection**
- The field's docstring claims it protects active task context from eviction.
  No compressor consumes it. Misleading API.

#### P3

- **FW-CTX-005 (F-CTX-01-P3-01)**: Unchecked `usize` arithmetic on the budget path violates AGENTS.md checked-arithmetic rule.
- **FW-CTX-006 (F-CTX-01-P3-02)**: `ContextAssembler` uses a byte-based token estimator that diverges from the configured `Tokenizer`.
- **FW-CMP-004 (F-CMP-01-P3-01)**: Horizon compact-summary length check uses byte length (`summary.len() > max_chars`), inconsistent with char-weighted tokenizer for CJK.
- **FW-CMP-005 (F-CMP-01-P3-02)**: L1 Fold inserts a user message inside tool-result sequences, producing non-contiguous tool results after sanitize (Adaptive only).

---

### 3.7 Integrations (MCP, LSP, A2A, Channels)

#### P1

**FW-INT-001 (was F-INT-01-P1-01): HttpTransport advertises a notification channel it never feeds; 202-Accepted branch hangs to 60s timeout**
- `notification_tx` is declared and `notification_rx()` returns `Some(...)`,
  but `http.rs` never calls `notification_tx.send`. On a Streamable-HTTP
  server returning `202 Accepted` (the spec-compliant async pattern), the
  future parks on a `oneshot::Receiver` nothing can fulfill and resolves only
  after the 60s timeout with a misleading "等待 HTTP 异步响应超时" error.
  Any MCP server using async tool execution is unusable through this client.

**FW-INT-002 (was F-INT-02-P1-01): A2A sync `tasks/send` does not honor cancellation; terminal monotonicity violated when cancel races a late completion**
- The sync path stores a `CancellationToken` but never polls it during
  `agent.execute().await`. The Completed/Failed writes bypass
  `update_task_state` (raw `tasks.insert`). When `tasks/cancel` races a late
  completion, the task regresses from Canceled → Completed. A client that
  issued `tasks/cancel` cannot trust the cancel response. The streaming path
  honors both contracts; the sync path does not.

#### P2

- **FW-INT-003 (F-INT-01-P2-01)**: `SseTransport` terminates permanently on clean stream close.
- **FW-INT-004 (F-INT-01-P2-02)**: `SseTransport` retry budget is never reset; 5 lifetime failures is permanent.
- **FW-INT-005 (F-INT-01-P2-03)**: No MCP tool-call cancellation; server-side `notifications/cancelled` is a no-op.
- **FW-INT-006 (F-INT-02-P2-01)**: LSP `restart_count` and `last_error` are dead fields; status report is misleading.
- **FW-INT-007 (F-INT-02-P2-02)**: LSP `send_request` has no per-request timeout.
- **FW-INT-008 (F-INT-02-P2-03)**: `ChannelManager::Drop` only logs; cannot run async cleanup, leaking per-channel background tasks.

#### P3

- **FW-INT-009 (F-INT-01-P3-01)**: `McpToolAdapter` drops optional metadata (output_schema, title, icons, meta, execution).
- **FW-INT-010 (F-INT-01-P3-02)**: `StdioTransport` Drop best-effort kill may leak the subprocess if the runtime is gone.
- **FW-INT-011 (F-INT-02-P3-01)**: LSP silently drops JSON-RPC responses whose id is a string, leaving pending requests hung forever.
- **FW-INT-012 (F-INT-02-P3-02)**: LSP reader and writer tasks are not cancellable; cancellation requires explicit shutdown or child death.
- **FW-INT-013 (F-INT-02-P3-03)**: Channel `stop()` leaks heartbeat / ping sub-tasks until next I/O fails.
- **FW-INT-014 (F-INT-02-P3-04)**: `A2AServer` has no `Drop`; in-flight tasks are not cancelled on shutdown.

---

### 3.8 Operations (Scheduler, Trace, Telemetry, Headless)

#### P1

**FW-OPS-001 (was F-OPS-01-P1-01): Scheduler has no graceful shutdown path**
- `SchedulerRunner::spawn` detaches the `JoinHandle`; no caller ever cancels
  `AppState.scheduler.cancel_token`. At process exit the runtime is torn down
  while `fire_task` may still be mid-`launch_cron_run`. In-flight cron runs
  (which write to worktrees, spawn subagents, hold pool entries) are not
  drained; they appear as stale `TaskRuntimeStore` runs on restart.

**FW-OPS-002 (was F-OPS-01-P1-02): Headless mode is not event-equivalent to interactive mode**
- `run_headless` calls `agent.execute`, returns `HeadlessResult`; no
  `AgentEvent` stream consumed, no `RunStore` attached, no `CancellationToken`
  exposed, no callback/metrics/trace. Violates AGENTS.md "多模式功能对等"
  (TUI/GUI/CLI/headless must be functionally equivalent). The product's own
  CLI does not use it precisely because it lacks these.

**FW-OPS-003 (was F-OPS-01-P1-03): Secrets are persisted into `JsonlRunStore` via unredacted `Run.input` / `final_output` / `ToolResult.output_preview` / `ToolError.message`**
- Only `RunEvent::ToolCall.args` is redacted. A user pasting "rotate AWS key
  AKIA…" into the prompt, or a `read_file` returning a `.env`, lands the
  secret in `Run.input`/`ToolResult.output_preview` written to disk in
  plaintext. `Run.input` is also surfaced via `RunSummary.input_preview`.
  Violates AGENTS.md "本地也成立的通用安全(如不把密钥打进日志)".

#### P2

- **FW-OPS-004 (F-OPS-01-P2-01)**: `JsonlRunStore` and `InMemoryRunStore` have no size bound or retention (unbounded growth).
- **FW-OPS-005 (F-OPS-01-P2-02)**: `Metrics::record_*` are defined but never invoked; telemetry is dead-on-arrival.
- **FW-OPS-006 (F-OPS-01-P2-03)**: Scheduler fires tasks serially and holds in-flight state without timeout.
- **FW-OPS-007 (F-OPS-01-P2-04)**: `CronTaskStore` panics on `current_thread` runtimes via `block_in_place`.

#### P3

- **FW-OPS-008 (F-OPS-01-P3-01)**: `CronTask::cron_expr` 5-field vs 7-field handling repeats unvalidated.
- **FW-OPS-009 (F-OPS-01-P3-02)**: `CronTaskStore::remove`/`set_status`/`update_last_run`/`get` use ID prefix match inconsistently.
- **FW-OPS-010 (F-OPS-01-P3-03)**: `last_fired` HashMap grows without eviction.
- **FW-REL-001 (F-REL-01-P3-01)**: `TokenBudget::allocate` uses plain usize arithmetic (duplicate of FW-CTX-005, cited from the reliability scan).
- **FW-REL-002 (F-REL-01-P3-02)**: CircuitBreaker state transitions emit no callback/event.

---

### 3.9 Subagents & Multi-Agent

#### P1

**FW-SUB-001 (was F-SUB-02-P1-01): Team mode does not propagate parent cancellation (`dispatch_team` ignores `req.cancel`)**
- Zero `CancellationToken` references in team production code. Each subagent
  spawned via `tokio::spawn(agent.execute(&task))` — `Agent::execute` has no
  cancel parameter. Cancelling the parent run has no effect on a running
  team. Sync/Fork/Teammate all derive `req.cancel.child_token()` and race it
  in `select!`; Team does not. A user cancelling a 5-subagent team with a
  600s timeout cannot stop it — tokens, API calls, and tool side effects
  continue for the full window.

**FW-SUB-002 (was F-SUB-02-P1-02): Team subagent tasks are detached (leaked) on team-level timeout**
- `execute_with_usage` wraps `execute_inner` in `tokio::time::timeout`. On
  timeout the future (holding `Vec<JoinHandle>`) is dropped; dropping a
  JoinHandle **detaches** the task. Spawned `agent.execute` tasks continue
  running, complete their LLM calls, run tools, and their results are silently
  discarded. The parent gets `Err("Team execution timed out")` and cannot
  observe or stop the lingering tasks. Team is the only mode that exhibits
  detached execution.

#### P2

- **FW-SUB-003 (F-SUB-02-P2-01)**: Team timeout source disconnected from `SubagentDefinition.timeout_secs`, silently floored at `.max(60)`, always 600s.
- **FW-SUB-004 (F-SUB-02-P2-02)**: Team subagents bypass `execute_agent_streaming` (no events, no isolation, no invocation context, no cancel).
- **FW-SUB-005 (F-SUB-02-P2-03)**: Team partial-failure has no sibling cancellation, no per-subagent timeout, always reports `Completed`.
- **FW-SUB-006 (F-SUB-01-P2-01)**: `SubagentDefinition.tool_filter` is declared and settable but never enforced at dispatch.
- **FW-SUB-007 (F-SUB-01-P2-02)**: Registration-time system-prompt compiler (`compile_system`) is never called in production.
- **FW-SUB-008 (F-SUB-01-P2-03)**: `context_builder::SubagentOutput` is a duplicate unused result type; `ContextBuilder` is an unused convenience constructor.
- **FW-SUB-009 (F-SUB-01-P2-04)**: `SubagentDefinition.lightweight` is a dead field (no setter, no reader).
- **FW-SUB-010 (F-MAG-01-P2-01)**: `HandoffManager` + `HandoffTool` are a parallel identity/dispatch authority with zero production consumers.
- **FW-SUB-011 (F-MAG-01-P2-02)**: `HandoffManager::handoff` spawns detached, non-cancellable agent execution; the "lock" comment is stale.
- **FW-SUB-012 (F-MAG-01-P2-03)**: `HandoffResult` is an unstructured plain-string result; `return_to_source` is dead.

#### P3

- **FW-SUB-013 (F-SUB-02-P3-01)**: `isolated.rs::run_isolated` is dead code (zero production callers).
- **FW-SUB-014 (F-SUB-02-P3-02)**: `TeamRunner` and `TeamCoordinator` reassignment logic are dead in the production dispatch path.
- **FW-SUB-015 (F-SUB-02-P3-03)**: Non-`ManagerSubagent` strategies (`Pipeline`/`Debate`/`Swarm`) have no production callers; `Swarm` silently drops errors.
- **FW-SUB-016 (F-SUB-01-P3-01)**: `SubagentKind::Custom { path }` has no `.md` definition loader.
- **FW-SUB-017 (F-SUB-01-P3-02)**: `inherit_history` has inconsistent `Some(0)` semantics across its two consumers.
- **FW-SUB-018 (F-SUB-01-P3-03)**: Sync dispatch skips the tool-allowlist enforcement that Fork applies.
- **FW-SUB-019 (F-MAG-01-P3-01)**: `TopologyTracker` + `TopologyCallback` have zero production consumers.
- **FW-SUB-020 (F-MAG-01-P3-02)**: `TopologyTracker` silently swallows every internal `RwLock` error.
- **FW-SUB-021 (F-MAG-01-P3-03)**: `TopologyCallback` hardcodes `NodeType::{Subagent, Tool}`, misclassifying every observed agent.
- **FW-SUB-022 (F-MAG-01-P3-04)**: `TopologyTracker` has no integration with `SubagentEvent`; subagent-to-subagent dispatch is invisible.

---

### 3.10 Workflow Engine

#### P1

**FW-WFL-001 (was F-WFL-01-P1-01): `Graph::resume()` parallel branch diverges from `run()`/`run_until_interrupt()`/`run_stream()` — no fork, no merge**
- Three paths fork each branch into isolated `SharedState` and `deep_merge`
  back; `resume()` mutates shared state in place, sequentially, with no
  isolation. A workflow resumed from a checkpoint produces different state
  than the same workflow run straight through. Silent correctness defect for
  parallel+interrupt workflows. The only resume test uses a linear graph.

#### P2

- **FW-WFL-002 (F-WFL-01-P2-01)**: Two overlapping graph implementations (`Graph` + `DagWorkflow`) with asymmetric validation and asymmetric concurrency.
- **FW-WFL-003 (F-WFL-01-P2-02)**: Conditional-edge targets not validated at build time — declarative typos pass build and fail at runtime.
- **FW-WFL-004 (merged F-WFL-01-P2-03 + FW-RESUME-002 theme)**: No schema-version field on `Checkpoint` — persisted checkpoints break silently on struct evolution (same pattern as `AgentCheckpoint`).

#### P3

- **FW-WFL-005 (F-WFL-01-P3-01)**: `add_parallel_edge` doc comment mis-describes the sequential-execution constraint.
- **FW-WFL-006 (F-WFL-01-P3-02)**: `SharedState::merge()` (non-overwrite) contains dead/confused lock acquisitions and a nonsensical SAFETY comment.
- **FW-WFL-007 (F-WFL-01-P3-03)**: `FileCheckpointStore` filename derived from raw user-supplied id (path-traversal surface) and `list()` silently skips corrupt entries.
- **FW-WFL-008 (F-WFL-01-P3-04)**: Doc-comment example in `sequential.rs` uses byte slicing that can panic on UTF-8.

---

### 3.11 Security (Guards, Redaction)

#### P2

**FW-SEC-001 (was F-SEC-01-P2-01): ContentGuard Redact mode is a no-op through GuardManager**
- `with_content_guard(Redact)` → `Guard::check` produces
  `ContentGuardResult::Redacted(String)` → the `Guard` impl converts to
  `GuardResult::Warn` and **drops the redacted String on the floor**. The
  inline comment admits it. PII still reaches the LLM/trace/snapshot.
  `GuardResult` has no "transformed content" variant.

**FW-SEC-002 (was F-SEC-01-P2-02): Password-in-URL secret pattern produces documented false positives**
- `://[^:]+:[^@]+@` matches any `scheme://anything:anything@`, including
  markdown documentation URLs. Benign content gets
  `[REDACTED: Password in URL]` injected. The codebase already has a TODO
  acknowledging this.

#### P3

- **FW-SEC-003 (F-SEC-01-P3-01)**: `RuleGuard.max_length` uses byte length, not character count (UTF-8 violation — CJK content over-truncated).
- **FW-SEC-004 (F-SEC-01-P3-02)**: Parallel secret scanner implementations in framework.
- **FW-SEC-005 (F-SEC-01-P3-03)**: `LocalConfig.enable_os_sandbox` naming is misleading on Windows.

---

### 3.12 Core (Identities, Events, Errors) & Facade

#### P2

**FW-CORE-001 (was F-CORE-01-P2-01): `GLOBAL_EVENT_BUS` and `EventBus` are dead infrastructure that advertises a multi-sink transport that does not exist**
- Doc promises "Webhook/Trace/UI/Audit" fan-out; zero callers of
  `GLOBAL_EVENT_BUS.send`/`subscribe`, `EventBus::new/default`, in either
  repo. Real event distribution uses direct stream composition. The canonical
  "dead infra" template — `LoopDetector` (FW-RCT-004), `HandoffManager`
  (FW-SUB-010), notebook (FW-NBK-001), `AdapterClient`/`DefaultLlmClient`
  (FW-LLM-011/012) all follow this pattern.

**FW-CORE-002 (was F-CORE-01-P2-02): Cross-stream `event_id` collision is silently possible when concurrent envelope streams share identity fields**
- `event_id` = SHA-256 over `(schema_version, conversation_id, run_id,
  turn_id, execution_id, sequence)`. Two concurrent streams sharing all five
  identity fields produce identical ids for the same sequence. Today's callers
  avoid it by always setting a unique `turn_id`/`execution_id`; the framework
  does not enforce or document the invariant.

#### P3

- **FW-CORE-003 (F-CORE-01-P3-01)**: Two parallel IO error channels and an untyped catch-all (`ReactError::Other(String)`) inside an otherwise typed error hierarchy.
- **FW-CORE-004 (F-CORE-01-P3-02)**: `StepType` is the only public agent type without documented non-serialization.
- **FW-CORE-005 (F-CORE-01-P3-03)**: `EventIdentity` derives `Default`, exposing empty `turn_id` to callers that should be forced to construct one.
- **FW-CORE-006 (F-CORE-01-P3-04)**: No explicit Serialize→Deserialize round-trip test for `EventEnvelope`.

---

### 3.13 Feature Topology, Facade, Evolution

#### P2

**FW-FEAT-001 (was F-FEAT-01-P2-01): 5 dead features with zero cfg matches and zero code references**
- `sandbox`, `semantic-memory`, `macros`, `provider-factory`, `multimodal` —
  all declared `= []`, gate nothing, reference nothing. Dead config expands
  the framework's feature surface without effect.

- **FW-API-001 (F-API-01-P2-01)**: `workspace` module is an incomplete and asymmetric escape hatch.
- **FW-API-002 (F-API-01-P2-02)**: Facade exposes parallel access paths to the same items.
- **FW-API-003 (F-API-01-P2-03)**: docs.rs metadata omits the `research` feature, hiding `echo_agent::tools::research` from rendered docs.
- **FW-EVO-001 (F-EVO-01-P2-01)**: React engine auto-writes memory during runs, in tension with the "evolution = diagnostics-only / no automatic memory mutation" boundary.

#### P3

- **FW-FEAT-002 (F-FEAT-01-P3-01)**: `full` aggregator includes the 5 dead features unnecessarily.
- **FW-FEAT-003 (F-FEAT-01-P3-02)**: `workflow` standalone compile needs spot verification.
- **FW-API-004..008 (F-API-01-P3-01..05)**: prelude mixes import styles; `workspace` aliases ambiguous; examples compile (positive); `demo03_approval` bypasses facade; README "67 tools" claim undocumented.
- **FW-EVO-002 (F-EVO-01-P3-01)**: `evolution` unconditionally compiled while `eval`/`improve` are feature-gated.

---

### 3.14 Tasks (TaskRun/PlanTask/DAG)

#### P2

- **FW-TSK-001 (F-TSK-03-P2-01)**: Stall detection does not cover the externally in-flight branch; the executor can poll indefinitely.
- **FW-TSK-002 (F-TSK-03-P2-02)**: Cancellation abort silently drops in-flight claims without resolving them (orphaned Running tasks).

#### P3

- **FW-TSK-003 (F-TSK-01-P3-01)**: `TaskPlanPatchOp` and `TaskPlanPatchInputOp` are two same-family enums whose proximity invites confusion.
- **FW-TSK-004 (F-TSK-02-P3-01)**: `tasks/dag.rs` recursive `get_dependency_chain` helper is private and its cycle-handling is not verified at the report layer.
- **FW-TSK-005 (F-TSK-03-P3-01)**: `TaskExecutor::execute_ready_tasks` and `execute_all_async` are parallel scheduling paths that bypass `RuntimeDagExecutor` (test-only / dead in production).
- **FW-TSK-006 (F-TSK-03-P3-02)**: `DagExecutionState::refresh_in_flight` is a dead public API (used only in its own unit test).
- **FW-TSK-007 (F-TSK-03-P3-03)**: The stall-detection, external-polling, and cancellation-abort branches have no test coverage.

---

### 3.15 Skills, Plugins, Notebook, HITL, Macros, Intent, Testing

#### P2

- **FW-SKL-001 (F-SKL-01-P2-01)**: Within-scope skill-name collision resolution is non-deterministic across runs.
- **FW-SKL-002 (F-SKL-01-P2-02)**: `SkillRegistry::register_descriptor` silently overwrites same-named skills and leaks stale `legacy_instructions`/`plugin_variables`.
- **FW-PLG-001 (F-PLG-01-P2-01)**: Plugin name collisions during scan are silently overwritten with no warning and non-deterministic resolution.
- **FW-NBK-002 (F-NBK-01-P2-01)**: `pub fn enable_notebook()` is a no-op public setter on a prelude-exported struct.
- **FW-HITL-001 (F-HITL-01-P2-01)**: `TimeoutStrategy` config is dead — stored and settable but never consulted; effective timeout behavior is hardcoded to Deny regardless of the configured strategy.
- **FW-MAC-001 (F-MAC-01-P2-01)**: `#[derive(Tool)]` silently ignores struct generics, producing a downstream compile error with no macro-side diagnostic.
- **FW-MAC-002 (F-MAC-01-P2-02)**: `#[derive(Tool)]` and `#[tool]` generate `::schemars::*` paths, forcing every consumer crate to declare `schemars` as a direct dependency — undocumented.
- **FW-MAC-003 (F-MAC-01-P2-03)**: No `trybuild` compile-fail tests and no integration tests for procedural macros — only the declarative macros have inline unit tests.
- **FW-INTENT-001 (F-INTENT-01-P2-01)**: Non-streaming DirectAnswer does not persist the assistant reply to context; diverges from streaming and breaks multi-turn continuity.

#### P1 (notebook stub)

**FW-NBK-001 (was F-NBK-01-P1-01): The notebook capability is an unreachable, aspirational stub with zero live callers**
- `NotebookTracker`, `NotebookCell`, six public methods; `pub mod notebook`
  always compiled (not feature-gated). Whole-workspace grep: zero references
  outside the definition + `enable_notebook` setter trio. `enable_notebook`
  appears only as declaration/default/setter body — **no read**. A consumer
  importing `echo_agent::notebook::NotebookTracker` or calling
  `.enable_notebook(true)` gets silent no-ops.

#### P3

- **FW-SKL-003..005 (F-SKL-01-P3-01..03)**: skill name validation warning-only; user tools named `activate_skill`/etc silently replaced; `scan_directory` lacks symlink-loop protection.
- **FW-PLG-002 (F-PLG-01-P3-01)**: `NativePlugin` trait and `export_to_env` are dead public API with zero callers.
- **FW-NBK-003..006 (F-NBK-01-P3-01..04)**: notebook data model is a flat in-memory tool-call log (not Jupyter); no persistence/artifact integration/schema version; docs overstate implementation; AUDIT_REPORT "RwLock poison panics" fixed (positive).
- **FW-HITL-002..004 (F-HITL-01-P3-01..03)**: per-args `Session` cache granularity unreachable; `sha256_hex` misnamed (SipHash not SHA-256); dead HITL approval impl block in `approval.rs`.
- **FW-MAC-004..011 (F-MAC-01-P3-01..08)**: attribute macros silently discard args; crate-path resolution asymmetric; generated visibility ignores input; error-field extraction brittle; duplicate `#[allow(dead_code)]`; `#[tool]` return-type check insufficient; lifetime handling limited; positive: signatures match all seven target traits.
- **FW-INTENT-002..004 (F-INTENT-01-P3-01..03)**: hook activation slot never cleared (stale skill leak); no intent-layer timeout; intent decisions only partially explainable.
- **FW-TST-001..005 (F-TST-01-P2-01..03, P3-01..02)**: `MockLlmClient` emits exactly one chunk per stream (no multi-chunk/fragmented tool-call/DeepSeek repair coverage); cannot simulate mid-stream errors/cancellation; `MockTool` never constructs `ToolFailure`/`bytes`/`data`/`truncated`/pagination; `FailingMockAgent` models exactly one error variant; `MockAgent::execute_stream` emits only `FinalAnswer`.

---

### 3.16 Codebase Quality (Q-STA-01, Q-DEP-01)

#### P1 (application layer)

**FW-QUAL-001 (was Q-STA-01-P1-01): `globstar_match` byte-slices a `&str`, panicking on non-ASCII paths**
- Layer: application (`echo-agent-cli/echo-agent-app-core/src/project/gitignore.rs:178-179`)
- `for j in 0..=remaining.len()` (every byte position) then
  `&remaining[j..]`. If a path contains any multi-byte UTF-8 char (Chinese,
  emoji — common for a China-targeted product) and a loaded `.gitignore`
  pattern contains `**`, `&remaining[1..]` slices inside the first character
  and panics. Any project file-scan crashes the process. The project's own
  `.gitignore` files don't use `**` today, lowering likelihood but not
  severity. The adjacent `simple_glob` correctly uses `as_bytes()`.

#### P2

- **FW-QUAL-002 (Q-STA-01-P2-01)**: ~50 production `#[allow(dead_code)]` annotations in framework, violating the cleanup policy (representative sites span pipeline.rs, execution.rs, context.rs, agent_box.rs, image_fetch.rs, qq/gateway.rs).
- **FW-QUAL-003 (Q-STA-01-P2-02)**: 25 source files exceed 1000 lines; 2 exceed 5000 lines (maintainability risk).
- **FW-QUAL-004 (Q-STA-01-P2-03)**: Duplicate crate versions in both lockfiles (38 framework / 76 CLI) — compile-time and binary-size cost.
- **FW-QUAL-005 (Q-DEP-01-P2-01)**: `hashbrown` resolves to 5 major versions across the workspace (0.12/0.14/0.15/0.16/0.17) — leaf-hot crate compiled early and widely depended on.

#### P3

- **FW-QUAL-006 (Q-STA-01-P3-01)**: No clippy guard for numeric `as` casts (pedantic lints not in gate).
- **FW-QUAL-007 (Q-DEP-01-P3-01)**: `quick-xml` resolves to 4 (framework) / 5 (CLI) versions.
- **FW-QUAL-008 (Q-DEP-01-P3-02)**: `@tailwindcss/vite` declared in both `dependencies` and `devDependencies`.

#### Positive confirmations

- **Panic safety is clean** across the audited framework paths (Q-STA-01) — the exceptions are the specific defects filed above (FW-TOOLS-003 IQR, FW-QUAL-001 globstar).
- **`unsafe` is minimal and guarded** (Q-STA-01).
- **Licenses are clean** — all MIT, no native deps, frontend current (Q-DEP-01).

---

## 4. Cross-Cutting Patterns

These recur across multiple subsystems and represent systemic issues, not
one-off bugs. Fixing the pattern fixes a whole class of findings at once.

### 4.1 Dead Infrastructure & Dormant APIs (the largest pattern)

The same "scaffolded, never wired, pub-exported, doc-overstates" shape repeats
across at least 11 distinct sites:

| Canonical finding | Item | Status |
|---|---|---|
| FW-CORE-001 | `GLOBAL_EVENT_BUS` / `EventBus` | zero producers/consumers |
| FW-RCT-004 | `LoopDetector` + config plumbing | only called in `#[cfg(test)]` |
| FW-RCT-005 | `process_steps` + helpers | `#[allow(dead_code)]`, superseded by `run_core_loop` |
| FW-LLM-011 | `AdapterClient` + `ProviderAdapter` | zero implementors/consumers, doc lies |
| FW-LLM-012 | `DefaultLlmClient` | never constructed, silently drops thinking |
| FW-NBK-001 | `NotebookTracker` module | zero live callers, `enable_notebook` unread |
| FW-SUB-010 | `HandoffManager` + `HandoffTool` | parallel dispatch authority, zero consumers |
| FW-SUB-019 | `TopologyTracker` + `TopologyCallback` | zero production consumers |
| FW-SUB-013 | `isolated.rs::run_isolated` | zero production callers |
| FW-FEAT-001 | 5 dead Cargo features | `= []`, gate nothing |
| FW-QUAL-002 | ~50 `#[allow(dead_code)]` | suppressed lint across many modules |

**Pattern**: scaffolding was added ahead of integration, the integration never
landed, and the `pub` surface + doc comments make the gap invisible to
`cargo check`. Only reachability grep finds it. AGENTS.md "代码清理:无需兼容,
过时代码可直接删" + "删除框架代码的判定" branch 1 (superseded) directly
apply: delete or wire, do not leave dormant with a misleading doc.

### 4.2 UTF-8 / Byte-Length Safety Violations

AGENTS.md mandates `chars().count()` / `chars().take()` over `str::len()` /
byte slicing. Violations:

| Finding | Site | Form |
|---|---|---|
| FW-QUAL-001 (P1) | `globstar_match` | `&remaining[j..]` byte slice → panic on non-ASCII |
| FW-TOOLS-003 (P1) | IQR outlier_detection | index-out-of-bounds (precedence bug, not UTF-8, but same panic class) |
| FW-SEC-003 | `RuleGuard.max_length` | byte length, not char count |
| FW-MEM-006 | `tokenize` | `s.len() > 1` filters by bytes |
| FW-CMP-004 | Horizon compact-summary | `summary.len() > max_chars` byte comparison |
| FW-TOOLS-011 | `find_occurrence_lines` | byte slicing on `&str` (safe today) |
| FW-WFL-008 | `sequential.rs` doc example | byte slicing can panic on UTF-8 |

### 4.3 Non-Atomic File Operations

The atomic-write recipe (uuid temp → write → fsync temp → rename → fsync
parent dir) is implemented correctly in exactly one place
(`FileConversationStore::atomic_write`). Everywhere else is broken differently:

| Finding | Site | Gap |
|---|---|---|
| FW-TOOLS-002 (P1) | `WriteFileTool`/`UpdateFileTool`/`EditFileTool`/`CreateFileTool` | bare `tokio::fs::write`, no temp/rename |
| FW-MEM-002 | `FileStore::flush`, `EmbeddingStore::flush_index` | no parent-dir fsync; static temp names |
| FW-MEM-001 (P1) | `FileStore::new` corrupt handling | silently swallows + overwrite-destroys |
| FW-RESUME-002 | `restore_thread_context` Err arm | silently swallows + overwrite-destroys |
| FW-WFL-007 | `FileCheckpointStore` | raw user id in filename (path traversal); `list()` skips corrupt |

**Pattern**: each backend reimplemented the recipe independently and only one
got it fully right. Fix: factor one `atomic_write(path, bytes)` helper (with
`sync_parent_directory`) into a shared util module and route every persistent
write through it. This single refactor resolves FW-TOOLS-002, FW-MEM-002, and
the durability half of FW-MEM-001/FW-RESUME-002.

### 4.4 Streaming Event Loss Under Backpressure

The mpsc channel (default 256) uses three send policies. `yield_event_or!`
(`try_send`, drop on Full) is applied too broadly — to events consumers
render live. `yield_final_event!` (`send().await`, block) is correctly applied
to success terminals. Error terminals use bare `try_send` + `let _`.

| Finding | Defect |
|---|---|
| FW-RCT-001 (P1) | 16 intermediate-event sites drop on Full — live UX corruption |
| FW-RCT-002 | 5 terminal-error sites drop on Full — no terminal signal, non-streaming returns `Ok("")` for failures |

A slow consumer SHOULD slow the producer (backpressure), not receive garbled
output. The "drop is safe because buffers accumulate" mental model is correct
for agent state, incorrect for the consumer's streaming UX.

### 4.5 Provider Adapter Gaps (Anthropic especially)

The neutral contract is well-designed; the Anthropic adapter is behind:

| Finding | Gap |
|---|---|
| FW-LLM-001 (P1) | streaming tool-call index/key desync drops every tool call |
| FW-LLM-002 | thinking blocks never surfaced (observational asymmetry) |
| FW-LLM-003 | signed thinking cannot round-trip |
| FW-LLM-005 | non-streaming hard-fails on unknown block types |
| FW-LLM-006 | streaming omits cache beta header |
| FW-LLM-007/008 | `tool_choice`/`response_format` silently dropped |
| FW-LLM-009 | malformed SSE silently dropped (OpenAI path too) |

The root cause for several (FW-LLM-001, FW-LLM-005, FW-LLM-006): **no
HTTP-mock fixture test exists for the Anthropic streaming path**. Static
inspection found the bugs; executable tests would have caught them at
authoring time.

### 4.6 Budget Accounting Defects

The budget's per-category reservations are phantom — every caller passes
`system_size=0, tool_defs_size=0`:

| Finding | Defect |
|---|---|
| FW-CTX-001 | tool defs + system prompt not accounted; 15% reserved sits empty while real bytes consume conversation budget |
| FW-CTX-002 | protected content not deducted from compressor's effective limit |
| FW-CTX-004 | oversized fallback `396_000` hides overflow |
| FW-CMP-001 | summary accumulation unbounded (compounds the above) |

Net effect: MCP-heavy EKO routinely exceeds the real window under normal
config, tripping provider 400. The budget gives a false sense of safety.

### 4.7 Checkpoint Corruption: Silent Swallow + Overwrite-Destroy

Three independent sites follow the same "fail open, then destroy evidence" anti-pattern:

| Finding | Site |
|---|---|
| FW-MEM-001 (P1) | `FileStore::new` corrupt JSON → empty map → flush overwrites |
| FW-RESUME-002 | `restore_thread_context` Err → reset → save overwrites |
| FW-WFL-004 | `Checkpoint` has no version field → version skew indistinguishable from corruption |

The sister store layer (`FileConversationStore`, `FileRuntimeStateStore`)
correctly returns `Err` for corrupt JSON — the defect is uniformly in the
*consumer* suppressing the Err. Fix pattern: (a) preserve corrupt file to
`.corrupt-{ts}` before overwrite, (b) surface resume failure via event/enum,
(c) add `version` field.

### 4.8 Concurrency / Cancellation / Lifecycle Gaps

Cancellation tokens are wired but not polled, or JoinHandles are detached, in
several lifecycle paths:

| Finding | Gap |
|---|---|
| FW-SUB-001 (P1) | Team mode: cancel token wired but never polled |
| FW-SUB-002 (P1) | Team timeout: JoinHandles detached |
| FW-INT-002 (P1) | A2A sync: cancel token stored but never polled; terminal monotonicity violated |
| FW-INT-001 (P1) | HttpTransport: 202 branch hangs on unfulfillable oneshot |
| FW-OPS-001 (P1) | Scheduler: JoinHandle detached, cancel token never fired |
| FW-INT-008 | `ChannelManager::Drop` cannot run async cleanup |

### 4.9 Trace Finalization Asymmetry

`finalize_run` is called on text-success and both failure helpers, but NOT on
tool-success (FW-RCT-006) or abandoned/cancelled/blocked arms (FW-RCT-008).
Every successful tool-based turn (the normal case) leaves the trace run stuck
in `Running`.

### 4.10 Parallel / Duplicate Implementations

Multiple subsystems carry two implementations of the same concern:

| Finding | Pair |
|---|---|
| FW-WFL-002 | `Graph` + `DagWorkflow` |
| FW-TOOLS-004 | `UpdateFileTool` + `EditFileTool` |
| FW-SEC-004 | parallel secret scanners |
| FW-TSK-005 | `execute_ready_tasks`/`execute_all_async` vs `RuntimeDagExecutor` |
| FW-SUB-010 | `HandoffManager` vs `SubagentEventBus` dispatch |
| FW-LLM-014 | `ThinkingProtocol` (framework) vs `ThinkingProtocolPreference` (transport) |

---

## 5. Contradiction Resolution

The reports are largely consistent (they were produced sequentially with
dependency handoffs). Three explicit reconciliations:

1. **SQLite durability hypothesis (resolved)**: `F-MEM-01` hypothesised
   `SqliteStore` satisfies the durability contract `FileStore` lacks;
   `F-MEM-02` **confirmed** (WAL + synchronous=NORMAL + per-op transactions).
   Recorded as a confirmed conclusion, not a stale finding.

2. **`AdapterClient`/`DefaultLlmClient` framework-delete test (resolved)**:
   `F-LLM-02` flagged them as dormant. AGENTS.md "删除框架代码的判定" retains
   pub API by default, but these meet the ✅ branch 1 criterion (superseded
   internal dead code with a pub surface — `OpenAiClient` is the live
   replacement). Resolution: deletion is valid; the reports correctly
   classify them as deletion candidates, not permanent menu options. The
   neighboring `SqliteStore` (also zero CLI callers) is correctly retained as
   a legitimate framework menu option (trait impl, real code, docs) — the
   distinction is "superseded by an equivalent" vs "a parallel backend".

3. **Headless equivalence (aligned, not contradictory)**: `F-OPS-01` flags
   headless as not event-equivalent; AGENTS.md "多模式功能对等" mandates
   equivalence. These align — the finding identifies a gap against the
   mandated invariant, not a contradiction with it.

---

## 6. Prioritized Action List

Ordered by a combination of severity (P1 first), blast radius, and fix cost
(cheaper fixes that unblock other work ranked higher). Each item links back
to its canonical finding ID(s).

### Tier A — Fix first (P1, high-impact, mostly small)

1. **FW-TOOLS-003**: Fix IQR index precedence in `outlier_detection`
   (`data_quality.rs:253-254`). One-line fix; turns a panic on valid input
   into correct behavior. Add `n==4` regression test.
2. **FW-LLM-001**: Fix Anthropic streaming tool-call desync (rename `_index`
   → `index`, use as insertion key). The single most impactful adapter bug —
   every Claude agentic streaming flow loses tool calls today. Pair with a
   streaming fixture test (none exist).
3. **FW-QUAL-001**: Fix `globstar_match` byte-slicing
   (`gitignore.rs:178-179`, application layer). Use `char_indices()` or
   `as_bytes()`. Prevents process crash on non-ASCII paths.
4. **FW-RCT-001**: Change `yield_event_or!` to `send().await` (or add a
   backpressure macro). Stops silent streaming-event loss under backpressure.
   Pair with FW-RCT-002 (terminal-error sites).
5. **FW-MEM-001**: Make `FileStore::new` return `Err` on corrupt JSON
   (matching `FileConversationStore`). One-line change; prevents silent
   permanent memory loss.
6. **FW-TOOLS-001**: Sanitize worktree `path_suffix` (reject `..`/`/`/
   absolute; single-component only). Prevents path-traversal data loss.
7. **FW-TOOLS-002**: Introduce shared `atomic_write(path, bytes)` helper and
   migrate the four file tools to it. **Unblocks FW-MEM-002** (route the
   stores through the same helper). One refactor closes ~4 findings.
8. **FW-OPS-003**: Redact `Run.input`/`final_output`/`ToolResult.output_preview`/
   `ToolError.message` in `JsonlRunStore::save` (and `InMemoryRunStore`).
   Prevents secrets in plaintext trace files.
9. **FW-SUB-001 + FW-SUB-002**: Thread `req.cancel` into team execution and
   abort JoinHandles on timeout. Pair the fix — both share the cleanup path.
   Stops leaked LLM budget and tool side effects on team cancel/timeout.

### Tier B — Fix next (remaining P1 + high-impact P2)

10. **FW-INT-001**: Either implement HttpTransport GET-SSE listener or drop
    the 202 scaffold and fail fast. Stops 60s hangs on async MCP servers.
11. **FW-INT-002**: Route A2A sync terminal writes through
    `update_task_state`; wrap `agent.execute` in `select!` against cancel.
    Restores terminal monotonicity.
12. **FW-OPS-001**: Return scheduler `JoinHandle`; fire cancel + await on
    shutdown. Stops stale runs on restart.
13. **FW-OPS-002**: Reimplement `run_headless` on `execute_stream` + optional
    `RunStore`. Restores mode equivalence (AGENTS.md mandate).
14. **FW-WFL-001**: Make `Graph::resume()` parallel branch fork+merge like
    the other three sites. Add parallel+interrupt resume test.
15. **FW-NBK-001**: Decide wire-or-delete for notebook. Under
    no-backward-compat, deletion is safe (zero live callers).
16. **FW-RCT-006 + FW-RCT-008**: Add `finalize_run` to
    `finalize_completed_run` and the abandoned/cancelled/blocked arms.
    Symmetric trace finalization.
17. **FW-RCT-007**: Add `ToolBatchEnd` + checkpoint to the concurrent batch
    timeout arm (mirror the cancellation arms).
18. **FW-RCT-004 + FW-RCT-005**: Delete `LoopDetector` and `process_steps`
    + helpers. Two independent dead-code removals; cleans the loop module.
19. **FW-CORE-001**: Delete or wire `GLOBAL_EVENT_BUS`/`EventBus`.
20. **FW-CTX-001 + FW-CTX-002**: Pass real tool-def and system-prompt token
    costs into `budget.allocate`; deduct protected content. Prevents provider
    400 on MCP-heavy config.
21. **FW-CMP-001**: Remove previous `[对话历史摘要]` system messages before
    appending new (`retain` before append). Stops unbounded summary growth on
    the default strategy.
22. **FW-RESUME-001**: Call `restore_thread_context` on chat mode when
    context is empty. Restores cross-process chat resume.
23. **FW-RESUME-002**: Preserve corrupt checkpoint to `.corrupt-{ts}`;
    surface resume failure; add `version` field to `AgentCheckpoint`.

### Tier C — Dedupe and converge (cross-cutting)

24. **Atomic-write helper** (Tier A item 7 expansion): route `FileStore`,
    `EmbeddingStore`, file tools, and (where applicable) checkpoint stores
    through one helper. Closes FW-MEM-002, FW-TOOLS-002, halves of
    FW-MEM-001/FW-RESUME-002.
25. **Dead-infra sweep**: delete the 11 dormant items in §4.1 (or wire
    them). Batch the safe deletions; per AGENTS.md "随手清理是强制要求".
26. **UTF-8 sweep**: fix the 7 byte-length/byte-slice sites in §4.2.
27. **Adapter fixture tests**: add HTTP-mock streaming fixtures for
    Anthropic (and OpenAI). Would have caught FW-LLM-001/005/006 at authoring
    time; prevents regressions after Tier A item 2.
28. **Feature cleanup**: remove the 5 dead features (FW-FEAT-001) and their
    `full` entries (FW-FEAT-002).

### Tier D — Lower-priority P2 and P3

The remaining 87 P2 and 113 P3 findings are real but lower-impact. They
should be addressed incrementally per AGENTS.md "随手清理是强制要求" — when
touching a module, audit and fix its findings in the same change. The
per-subsystem lists in §3 serve as the work queue. Notable clusters:

- **Macros** (FW-MAC-001..011): `#[derive(Tool)]` generics + schemars path +
  trybuild tests are the highest-value macro fixes.
- **Memory** (FW-MEM-003..010): `prune_expired` orphan cleanup, mutex
  convergence, facade re-export.
- **Testing** (FW-TST-001..005): enrich `MockLlmClient`/`MockTool` to cover
  multi-chunk streaming, mid-stream errors, `ToolFailure` — unblocks better
  coverage for everything above.
- **Dependency convergence** (FW-QUAL-005/007): collapse `hashbrown` (5
  majors) and `quick-xml` (4-5) versions.

---

## 7. What Is Clean (Positive Conclusions)

To balance the defect inventory, the review confirmed several invariants hold:

- **Single ReAct loop body** (`run_core_loop`): streaming and non-streaming
  share one authoritative loop (F-RCT-02).
- **Single tool registry** with idempotent replacement; the historical
  `todo_write` is gone; one task API (F-RCT-01).
- **Tool-pair protocol validity** is guaranteed by the single
  `sanitize_tool_call_pairing` choke point (F-CMP-01).
- **Protected content survives compression** (F-CTX-01, F-CMP-01).
- **`Usage` normalization** is single-authority and handles OpenAI-inclusive
  / DeepSeek-inclusive / Anthropic-exclusive cache semantics (F-LLM-01).
- **Cancellation is graceful on the React loop** (5s tool drain) where wired
  (F-RCT-04).
- **`SqliteStore`/`SqliteConversationStore` are legitimate framework menu
  options**, cleanly feature-gated, satisfying the durability contract
  (F-MEM-02). Not deletion candidates.
- **Workflow engine is cleanly distinct from the dynamic task system** —
  zero code references across the boundary (F-WFL-01).
- **Panic safety is clean** across audited paths, modulo the specific defects
  filed (Q-STA-01).
- **`unsafe` is minimal and guarded** (Q-STA-01).
- **Licenses are clean** — all MIT (Q-DEP-01).

---

## Appendix: Source Index

All 40 source reports (38 F-phase + 2 Q-phase), in the order referenced:

F-API-01, F-CMP-01, F-CORE-01, F-CTX-01, F-EVO-01, F-EXT-01, F-EXT-02,
F-EXT-03, F-FEAT-01, F-HITL-01, F-INT-01, F-INT-02, F-INTENT-01, F-LLM-01,
F-LLM-02, F-LLM-03, F-MAC-01, F-MAG-01, F-MEM-01, F-MEM-02, F-NBK-01,
F-OPS-01, F-PLG-01, F-RCT-01, F-RCT-02, F-RCT-03, F-RCT-04, F-RCT-05,
F-REL-01, F-SEC-01, F-SKL-01, F-SUB-01, F-SUB-02, F-TSK-01, F-TSK-02,
F-TSK-03, F-TST-01, F-WFL-01, Q-DEP-01, Q-STA-01.

Each report lives at
`docs/comprehensive-review/zcode-glm/tasks/<ID>.md` with its validation
evidence at `docs/comprehensive-review/zcode-glm/validations/<ID>/`. All
reports are at baseline `echo-agent` `9b0e0fa` / `echo-agent-cli` `b3b2e81`.
