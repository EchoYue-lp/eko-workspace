# S-FW-01: Framework Review Synthesis (ZCode-ds)

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Synthesis date: 2026-08-12
> `echo-agent` commit reviewed: 9b0e0faf74d35c9a432370b923acabfbb5f32d63 (= baseline 9b0e0fa)
> `echo-agent-cli` commit reviewed: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5 (= baseline b3b2e81)
> Worktree state: both repositories clean; HEAD unchanged since every task report was written
> Inputs: all 38 F-* task reports, Q-FW-01, Q-FW-02, Q-STA-01, Q-FLT-01 task reports (zcode-ds), shared README.md / REPORTING.md / TASKS.md, zcode-ds README.md
> Deliverable contract: TASKS.md S-FW-01 — canonical P0/P1 summary, duplicate reconciliation, contradiction/open-question handling, stale-commit check, framework health verdict. Validations: V01 (coverage), V02 (reconciliation), V03 (stale check) — see validation reports.

## 1. Scope And Method

This synthesis consumes the completed zcode-ds framework-phase reports only
(F-*, Q-FW-01, Q-FW-02, Q-STA-01, Q-FLT-01). Per REPORTING.md Synthesis Rules it
merges duplicates into canonical IDs, preserves minority conclusions as open
questions, runs a stale-commit check against the shared baseline, and does not
prescribe a second authority during staged migrations. A-*/X-* reports are
consumed only where a F-* report itself names a cross-phase alias (recorded in
section 4). No source code was read for this task beyond the anchors already
cited in the task reports; no source file was modified.

Counting method: P0/P1 findings were extracted per report by exact finding ID
(`<task>-P1-<seq>`), then de-duplicated across reports. There are **zero P0
findings** in the entire framework phase. There are **50 raw P1 findings** in
scope; after canonical merge (F-RCT-05-P1-03 → F-SKL-01-P1-02) the canonical
count is **49 P1 findings**.

## 2. Canonical P1 Summary Table

Layer legend: **F** = framework (echo-agent / echo_core / echo_state /
echo_orchestration / echo_execution / echo_tools / echo_macros), **A** =
adapter (echo-integration providers/transports; framework↔application wiring
boundary), **AP** = application (EKO wiring that makes a framework behavior a
product defect). Validation links point at the owning task's validation
reports; the owning task report is the backlink for every row.

### 2.1 Cluster A — Turn/stream terminal and event integrity (silent-failure core)

| Canonical ID | Layer | file:line | Finding (one line) | Fix direction |
|---|---|---|---|---|
| F-RCT-02-P1-01 | F | `run/react_loop.rs:711-727,729-750` | Non-streaming turn returns `Ok("")` success when the spawned core loop errors (error only logged; `finalize_completed_run` propagates intervention errors out-of-band) | Forward loop errors on the channel like `stream_channel.rs:306-311`; send intervention cancel/block errors before returning; [V02/V03/V04](../validations/F-RCT-02/V02-01.md) |
| F-RCT-03-P1-01 | F | `run/stream_macros.rs:38-53`, `finalize.rs:226,267` | `try_send` drops events — including terminal errors — when the 256-slot buffer is full; raw stream can end with no terminal; envelope fabricates a generic Error | Blocking sends for intermediates and the four terminal-Err paths; typed drop counter; [V01/V02/V03](../validations/F-RCT-03/V01-01.md) |
| F-RCT-03-P1-02 | F | `react/mod.rs:2821-2933`, `echo-core/src/agent/mod.rs:552-553` | Cancelled turns never yield the documented `AgentEvent::Cancelled` (ReactAgent overrides bypass `cancel_aware_stream`; cancel → NoResponse error or fabricated envelope Error) | Emit `Cancelled` at the think/tools cancel terminal points with trace finalization, or restore `cancel_aware_stream`; [V01/V02/V03](../validations/F-RCT-03/V01-01.md) |
| F-RCT-04-P1-01 | F | `run/phases/tools.rs:207-240`, `processor.rs:141-147` | Concurrent batch results inserted in completion order while the assistant message carries calls in stream-index order → strict providers (Anthropic/Gemini/Kimi/GLM) reject the next request 400 after tools ran | Buffer and insert results in call order (or execute in call order); conformance fixture; [V01/V03](../validations/F-RCT-04/V01-01.md) |
| F-RCT-04-P1-02 | F | `phases/tools.rs:284-300,418-423`, `stream_macros.rs:65-75` | Batch timeout/cancel end the turn without a typed terminal (droppable `try_send` error / `ToolBatchEnd` only), trace stays `Running`, and an already verifier-accepted `final_answer` is discarded on peer timeout | Typed error/cancelled terminal with guaranteed delivery + `finalize_run` on both exits; prefer completing when `finish_output` is set; [V02/V03/V05](../validations/F-RCT-04/V02-01.md) |
| Q-FLT-01-P1-01 | F | `run/phases/think.rs:110-125`, `finalize.rs:128-208`, `client.rs:305-306` | Truncated / cleanly disconnected provider streams are accepted as complete final answers (no `finish_reason` check; partial content finalized `Completed`) — dynamically reproduced | Consult `finish_reason`/`stop_reason` at stream end; flag or error on truncated output; never finalize `Completed` on partial; [V01/V06](../validations/Q-FLT-01/V01-01.md) |
| Q-FLT-01-P1-02 | F | `providers/client.rs:240-301`, `run/retry.rs:13-68` | Configured LLM stream timeouts (first-chunk/idle) fire inside the stream as items and are never retried — retry wraps only stream creation; mid-stream stall fails the turn (dynamically reproduced) | Hoist timeouts to the constructor layer or document non-retried stream timeouts; [V04](../validations/Q-FLT-01/V04-01.md) |
| F-LLM-01-P1-01 | A | `providers/client.rs:99-105` (vs `:163-164`) | Shared SSE transport silently drops malformed chunks (`warn!` + `None`); streaming diverges from non-streaming error handling; `usage_reported` can be falsely false | Typed error or counted drop for unparseable chunks; align with non-streaming contract; malformed-chunk fixtures; [V02/V04](../validations/F-LLM-01/V02-01.md) |
| F-LLM-03-P1-01 | A | `providers/anthropic.rs:520-528,547-583` | Streaming tool-call accumulator keyed by map length, not stream block index — interleaved [text, tool_use] streams silently lose or corrupt tool calls | Key by the event's stream `index`; interleaved-block fixtures; [V01/V02](../validations/F-LLM-03/V01-01.md) |
| F-LLM-03-P1-02 | A | `providers/anthropic.rs:1037-1046,1063-1068,510` | `message_delta.usage` ({output_tokens} only) fails the strict `AnthropicUsage` deserializer → final usage/finish chunk silently dropped on every real Anthropic stream; `usage_reported` always false | Split message_start/delta usage structs (delta = output_tokens only, `#[serde(default)]`); fixture with the real payload; [V03/V04/V05](../validations/F-LLM-03/V03-01.md) |
| F-LLM-03-P1-03 | A | `providers/anthropic.rs:73-77`, `echo-state/src/compression/mod.rs:900-931` | Multiple leading system messages collapse to the last one — base system prompt silently dropped after canonical-context reinjection | Join all leading system texts in order (cache_control on last); convert_request fixture; [V01](../validations/F-LLM-03/V01-01.md) |
| F-LLM-03-P1-04 | A | `providers/anthropic.rs:827-866,1080-1093` | Response thinking blocks unmodeled — non-streaming `chat()` fails to parse thinking responses; streaming silently discards thinking deltas; `reasoning_content` always None | Add Thinking/RedactedThinking variants; map deltas to `reasoning_content`; fixtures for both modes; [V01/V02](../validations/F-LLM-03/V01-01.md) |
| F-LLM-02-P1-01 | A | `providers/openai.rs:289,343`, `types.rs:537-538` | `max_tokens` sent unconditionally; o1/o3/o4/gpt-5 reasoning models reject it with HTTP 400 — reasoning-model usage broken whenever a cap is configured | `max_completion_tokens` wire field resolved by model family; never send both; request-JSON fixtures; [V01/V02](../validations/F-LLM-02/V01-01.md) |
| F-CTX-01-P1-01 | A | `echo-core/src/llm/capabilities.rs:197-217`, `agent/config.rs:11-12`, `infra.rs:23` | Provider window mapping bypassed — builder default and EKO runtime hardcode 396K; kimi k2.x (256K real) can overflow; 1M-window models compress ~4x early | Derive `token_limit` from ModelProfile at construction (explicit override wins); drop the hardcoded constant; regression test per model family; [V01/V02/V03](../validations/F-CTX-01/V01-01.md) |
| F-CMP-01-P1-01 | F | `compressor/sliding_window.rs:48-66`, `summary.rs:299-317`, `compression/mod.rs:1317-1345` | Message-count windows never bound tokens — a few large messages over the limit compress to nothing; `prepare` never re-checks the result | Token-aware windows or post-compression escalation (spill/emergency cut) with an explicit over-limit signal; [V02/V03/V04](../validations/F-CMP-01/V02-01.md) |
| F-CMP-01-P1-02 | F | `compressor/summary.rs:346-348,292-296`, `levels.rs:578-583` | One immortal system summary appended per compression pass — system region grows without bound under repeated compression | Replace-or-merge: single running summary updated per pass, or cap + merge oldest; wire the stateful path; [V02/V03](../validations/F-CMP-01/V02-01.md) |
| F-CMP-01-P1-03 | F | `compression/levels.rs:392-396`, `compression/mod.rs:1550-1557` | Adaptive L1 fold inserts a `Role::User` message between an assistant's tool_calls and its kept tool results — breaks the framework's own pairing-contiguity invariant → provider 400 on adaptive strategy | Insert the fold summary after kept results or strip the folded call ids; sanitize-level contiguity regression test; [V03/V04](../validations/F-CMP-01/V03-01.md) |

### 2.2 Cluster B — Resume, checkpoint, and recovery integrity

| Canonical ID | Layer | file:line | Finding | Fix direction |
|---|---|---|---|---|
| F-RCT-05-P1-01 | F | `phases/tools.rs:203,296,309,336,419`, `state/mod.rs:186-231`, `run/context.rs:245-248` | Interrupting mid-tool-batch persists a checkpoint the resume validator rejects; restore falls back to `reset_messages()` → full conversation-context wipe on the next turn (dynamically proven) | Checkpoint only the paired prefix at cancel/error arms; preserve previous/in-process state on rejection; surface rejection as an error event; move compact checkpoint post-prepare; [V02/V03-03/V04-04](../validations/F-RCT-05/V02-01.md) |
| F-RCT-05-P1-02 | F | `run/stream_channel.rs:111-122,333`, `steer.rs:117-119` | Same-turn steer silently dropped on the EKO main path — mailbox lease keyed by `turn_id`, drain keyed by `current_run_id` (= `run_id`, None in Chat/Auto); UI reports success | Drain by the same id the lease used (carry it in the snapshot); tests with `run_id != turn_id` and `run_id = None`; [V02/V04-02/V04-03](../validations/F-RCT-05/V02-01.md) |
| F-SKL-01-P1-02 (= F-RCT-05-P1-03, merged) | F | `capabilities.rs:660-677`, `snapshot.rs:206-222`, `react/mod.rs:1703-1704`, `resource_tool.rs:98-103` | Dual `SkillRegistry` divergence — checkpoint resume marks only the tracking registry while the three skill tools consult the shared registry → "not activated" after fresh-process resume; re-activation duplicates instructions | Single activation authority (mark both registries on resume, or merged set, or drop tracking registry); save/restore round-trip test; [F-SKL-01 V02](../validations/F-SKL-01/V02-01.md), [F-RCT-05 V02](../validations/F-RCT-05/V02-01.md) |
| F-MEM-01-P1-01 | F | `echo-state/src/memory/store.rs:235-238,254-278` | FileStore silently discards a corrupt/truncated store file (warn + empty state) and overwrites it on the next write — permanent memory/cron-task loss with no error | `FileStore::new` returns a serialization error on unparseable content (mirror `file_conversation.rs:153-157`); refuse first flush or back up `.corrupt`; corrupt-file tests; [V02/V03/V04](../validations/F-MEM-01/V02-01.md) |
| F-OPS-01-P1-01 | F | `scheduler/runner.rs:80-93`, cron 0.12.1 `upcoming` | Scheduler tick can never fire — `upcoming()` is strictly-future so `next <= now` is unsatisfiable; all cron tasks and plugin monitors silently never run (empirically proven) | Fire relative to a last-tick reference (`schedule.after(&last_tick).next()` in `(last_tick, now]`); tick unit tests; [V02/V03/V04-01](../validations/F-OPS-01/V02-01.md) |

### 2.3 Cluster C — Approval and permission boundary

| Canonical ID | Layer | file:line | Finding | Fix direction |
|---|---|---|---|---|
| F-HITL-01-P1-01 | F | `snapshot.rs:841-852`, `pipeline.rs:268-346`, dead `run/approval.rs:159-165` | The live tool-approval path cannot ask the human — `RequireApproval`/`Ask` become opaque tool errors; the only ask-capable implementation is dead code; Bubble mode's "bubble up" is unimplemented | Port `request_human_approval` semantics into the live `check_tool_approval` (provider request, audit, scope, modified args); delete dead path only after porting; [V01/V02/V03](../validations/F-HITL-01/V01-01.md) |
| F-HITL-01-P1-02 | F | `service.rs:729-731`, `snapshot.rs:834`, dead `run/approval.rs:143` | User-modified tool args silently discarded on the live path — `last_modified_args` side channel written but only the dead code reads it; the original args execute | Read `take_modified_args()` in the live Allow arm and return as `Ok(Some(modified))`; live-path handler fixture; [V01/V02/V04-02](../validations/F-HITL-01/V01-01.md) |
| F-HITL-01-P1-03 | F | `service.rs:791-807,892-908`, `permission.rs:706-711` | Approval scope widened — session approvals collapse to per-tool global cache entries; the live EKO-used bridge maps `SessionAllTools` to a `"*"` wildcard rule covering ALL tools | Carry the response's true scope; tool-scoped rules via `build_matcher`; drop the `"*"` mapping; three-granularity service tests; [V01/V03-03](../validations/F-HITL-01/V01-01.md) |

### 2.4 Cluster D — Subagent / team / handoff / routing lifecycle

| Canonical ID | Layer | file:line | Finding | Fix direction |
|---|---|---|---|---|
| F-SUB-01-P1-01 | F | `subagent/types.rs:144`, `executor.rs:120-135`, `agent_dispatch.rs:224-299` | Per-role `tool_filter` has zero production readers — the LLM-facing `agent_tool` cannot restrict a subagent's tools (false scope guarantee) | Consume `tool_filter` → `AgentInvocationContext.disabled_tools` in all dispatch modes (single authority at invocation), keep per-task allowlist precedence; [V01/V02/V03](../validations/F-SUB-01/V01-01.md) |
| F-SUB-02-P1-01 | F | `subagent/executor.rs:992-1113`, `team/mod.rs:343-353` | Team mode has no cancellation path — zero `CancellationToken` in `team/`; parent cancel is invisible; members keep running/writing detached after the parent stops | Thread the token through `TeamAgent::execute_with_usage` → orchestrator planning/fan-out/synthesis; emit standard `DispatchCancelled`; [V01/V02/V03](../validations/F-SUB-02/V01-01.md) |
| F-SUB-02-P1-02 | F | `team/mod.rs:346-348`, `manager_subagent.rs:277-284` | Team timeout/abort detaches `tokio::spawn`ed members — the outer future drops but members keep executing; a reported failure coexists with live member execution (side effects continue) | Cancel-aware fan-out (`JoinSet::abort_all` or per-member token race), optional grace; sibling cancellation on member failure; fixture with a hung member; [V01/V03/V04-02](../validations/F-SUB-02/V01-01.md) |
| F-MAG-01-P1-01 | F | `handoff/mod.rs:262-273`, `tool.rs:104-105` | Handoff executes target agents as detached, uncancellable, timeout-less `tokio::spawn` + oneshot; dropped caller orphans the target; no lifecycle events; manager mutex held across execution | Reimplement over `SubagentExecutor::dispatch` (Sync + child token) or thread token + timeout; release the lock before awaiting; Q-FLT-02 fixtures must fail pre-fix; [V01/V02/V03](../validations/F-MAG-01/V01-01.md) |
| F-INTENT-01-P1-01 | F | `intent/trigger_supervisor.rs:55-60`, `intent/mod.rs:136-150` | TriggerSupervisor hook-fusion emits confidence 0.6 but the router re-applies the 0.7 threshold → documented skill-activation retry silently never fires (deterministic constant mismatch) | Align fusion confidence with router threshold (or trust the hook decision via a distinct flag); single cache-write prepare path; classify-through-router test; [V01/V03/V04-01](../validations/F-INTENT-01/V01-01.md) |

### 2.5 Cluster E — Task graph and workflow recovery

| Canonical ID | Layer | file:line | Finding | Fix direction |
|---|---|---|---|---|
| F-TSK-02-P1-01 | F | `tasks/runtime.rs:449-455,481-483`, `runtime_executor.rs:308-313` | Skip is terminal without dependency propagation — skipping a task with Pending dependents stalls and fails the whole run with a misleading "cycle or blocked" error | Treat `Skipped` (and cancelled, at safe-point policy) as satisfying readiness or propagate skip at the safe point; differentiate the stall reason; framework fixture A(Skipped)→B; [V02/V03/V04](../validations/F-TSK-02/V03-01.md) |
| F-WFL-01-P1-01 | F | `workflow/graph.rs:826-849,913-1092` | AfterNode checkpoint stores only the join node — resume skips pending parallel fan-out branches and bypasses the next node's before-interrupt (approval gate silently skipped) | Persist the pending `NextStep` (targets + then, or granted-before-interrupt fact) in the checkpoint and replay it in `resume`; two regression fixtures; [V03-03/V01](../validations/F-WFL-01/V03-03.md) |

### 2.6 Cluster F — Domain tool correctness (framework tooling)

| Canonical ID | Layer | file:line | Finding | Fix direction |
|---|---|---|---|---|
| F-EXT-01-P1-01 | AP | `infra.rs:963` vs `snapshot.rs:282-285`, `pipeline.rs:1000-1018` | Writer subagents are silently read-only — `set_plan_mode(true)` in the writer builder collides with the plan-mode tool filter (post-dates the wiring); write tools invisible and blocked | Remove `set_plan_mode(true)` from `build_writer_subagent_agent`; EKO test asserting writer visibility; (canonical for the A-TOOL-01-P1-01 mirror — see section 4); [V02/V04-02](../validations/F-EXT-01/V02-01.md) |
| F-EXT-01-P1-02 | AP | `echo-execution/src/tools.rs:529-532`, `agent_pool.rs:96,127,671-672,923` | AgentPool shares one ToolManager and re-registers per-agent memory tools under the same four names — silent overwrite routes all pooled agents' memory calls to one agent's layer manager | Observable registration (log/reject duplicate names); per-agent registry wrapper or per-invocation resolution via ToolContext; [V02/V04-02](../validations/F-EXT-01/V02-01.md) |
| F-EXT-02-P1-01 | F | `echo-tools/src/files/edit.rs:260-270` | `edit_file` with empty `old_content` panics — byte-slice OOB / non-char-boundary in `find_occurrence_lines` (reproduced; no `catch_unwind` barrier → aborts the agent run) | Reject empty `old_content` with InvalidArguments (or char-boundary-safe iteration); ASCII + multibyte regression tests; [V01](../validations/F-EXT-02/V01-01.md) |
| F-EXT-03-P1-01 | F | `echo-tools/src/research/memory.rs:66-121,159-180` | `research_remember`/`research_recall` are non-persistent stubs that fabricate "stored successfully" and silently discard findings; recall always returns "no findings" | Implement on the framework Store/FileStore or remove from both registries; never report success without storing; [V03-04/V02](../validations/F-EXT-03/V03-04.md) |
| F-EXT-03-P1-02 | F | `echo-tools/src/registry.rs:55-62,161-176`, `bibtex.rs:21-23,94-101`, `rag.rs:305-306` | Read-only Subagent surface registers Write-permission tools — `bibtex_generate` writes arbitrary unvalidated paths; `rag_index` mutates shared global state; physical no-write guarantee broken | Derive the readonly subset from `Tool::permissions()`/`risk_level()` at registration; exclude/validate bibtex+rag; route `output_file` through `validate_output_file`; [V03-03/V02](../validations/F-EXT-03/V03-03.md) |
| F-EXT-03-P1-03 | F | `echo-tools/src/data_quality.rs:249-255` | `outlier_detection` (IQR) panics on a numeric column with exactly 4 values — quantile index OOB (`3*n / 4.min(n-1)` = 4 on 4 elements); formula also wrong for small n (Q-STA-01 adds n=1 division-by-zero, n=2 OOB) | Bounds-checked quantile helper (`div_ceil` clamped, or `polars::Series::quantile`); 4-value regression test; [V03-01/V04-02](../validations/F-EXT-03/V03-01.md) |

### 2.7 Cluster G — External protocol integrations (MCP / LSP / channels / A2A)

| Canonical ID | Layer | file:line | Finding | Fix direction |
|---|---|---|---|---|
| F-INT-01-P1-01 | F | `mcp/transport/http.rs:69-74,161-179` | HTTP transport's 202-async path is dead — nothing ever fires the pending oneshots; compliant Streamable HTTP servers hang every call for 60 s; 200+SSE fails JSON parse | Implement the SSE GET receive stream (route `message` events to pending) or reject 202/SSE loudly; fake-server fixtures; [V02/V03/V04-02](../validations/F-INT-01/V02-01.md) |
| F-INT-01-P1-02 | F | `mcp/transport/http.rs:96-147,251-270` | HTTP transport retries non-idempotent `tools/call` on ambiguous failures — duplicates side effects, bypassing the framework `allows_automatic_retry` gate | Remove transport-level retries for `tools/call` (keep for reads); surface `ToolFailure` so the manager gate decides; align backoff with `RetryPolicy`; [V03/V05](../validations/F-INT-01/V03-01.md) |
| F-INT-02-P1-01 | F | `integration/src/lsp/client.rs:225-227,311-329` | LSP JSON-RPC requests have no timeout and no cancel cleanup — a hung server blocks `shutdown`/plugin reload forever and leaks the pending map; `LspError::Timeout` is dead contract | Timeout `rx.await`; remove pending entry on timeout/cancel; bound `shutdown`; stub-server tests; [V02/V03](../validations/F-INT-02/V02-01.md) |
| F-INT-02-P1-02 | F | `channels/channels/qq/channel.rs:108-132,198-208` | QQ channel send task busy-loops a CPU core after `stop()` — `loop { if let Some }` on a closed channel, JoinHandle discarded | `while let` + stored JoinHandle aborted in `stop()` (Feishu pattern); stop-lifecycle test; [V02-02/V03-05](../validations/F-INT-02/V02-02.md) |
| F-INT-02-P1-03 | F | `a2a/server.rs:404-414,439-442` | A2A `tasks/cancel` does not cancel sync execution (token stored, never used) and Completed overwrites Canceled — terminal-state regression observable via `tasks/get` | Cancel-aware execution variant on the sync path; terminal-state fixture; [V03-09/V03-11](../validations/F-INT-02/V03-09.md) |

### 2.8 Cluster H — Build surface and reliability primitives

| Canonical ID | Layer | file:line | Finding | Fix direction |
|---|---|---|---|---|
| F-MAC-01-P1-01 | F | `echo-macros/src/derive_tool.rs:387`, `echo-agent/src/tools/mod.rs:109-114` | `#[derive(Tool)]` emits `<Self as ::echo_agent::tools::ToolRunner>` but the facade never exports `ToolRunner` — documented facade-only usage fails E0405 (reproduced) | Export `ToolRunner` from the facade; facade-only compile fixture; align with F-API-01-P3-01; [V03/V04-02](../validations/F-MAC-01/V03-01.md) |
| F-REL-01-P1-01 | F | `echo-core/src/circuit_breaker.rs:106-140`, `run/retry.rs:61,63` | Circuit breaker gate is never called — `try_advance` has zero callers; the documented Open/HalfOpen protection is passive telemetry; a persistent outage runs up to 16 HTTP attempts per logical LLM call | Wire `try_advance`/`record_rejected` into the unified retry loop (or re-document as telemetry and delete the HalfOpen machinery); breaker-open regression tests; [V01/V02/V03](../validations/F-REL-01/V01-01.md) |

### 2.9 Cluster I — Test infrastructure fidelity ("mock invisibility cloak")

| Canonical ID | Layer | file:line | Finding | Fix direction |
|---|---|---|---|---|
| F-TST-01-P1-01 | F | `src/testing/mock_llm.rs:410-425`, `think.rs:112-113,147` | Mock emits content+usage in a single chunk — the loop suite certifies `usage_reported: true` in a wire shape no real provider produces (usage-only final chunk impossible), hiding F-LLM-03-P1-02 | Scriptable chunk sequences incl. usage-only final chunk; loop-level regression asserting `usage_reported` semantics; [V03/V04-03](../validations/F-TST-01/V03-01.md) |
| F-TST-01-P1-02 | F | `src/testing/mock_llm.rs:410-458` | Streaming is not scriptable at all — one `stream::once` chunk; no ordering, mid-stream errors, or incremental tool-call deltas; F-RCT-03/F-RCT-04/F-LLM-01/F-LLM-03 defect classes structurally unreproducible at loop level | Chunk-sequence API (deltas/usage/finish/mid-stream Err); script the canonical defect sequences; [V03/V01](../validations/F-TST-01/V03-01.md) |

### 2.10 Cluster J — Static safety (panic family)

| Canonical ID | Layer | file:line | Finding | Fix direction |
|---|---|---|---|---|
| Q-STA-01-P1-01 | F | `echo-tools/src/web/providers/utils.rs:26-46` | `percent_decode` byte-slices a `&str` at computed offsets — panic on non-ASCII after `%`; live in EKO `web_search` via the DuckDuckGo fallback parsing remote hrefs (reproduced exit 101) | Decode on `as_bytes()` windows (u8 + `from_str_radix`) or gate on `is_char_boundary`; `%`+CJK fixture; [V03](../validations/Q-STA-01/V03-01.md) |

## 3. P1 Distribution And Zero-P1 Tasks

- Canonical P1 total: **49** (after merging F-RCT-05-P1-03 into F-SKL-01-P1-02).
- By layer: framework **40**, adapter **7** (F-CTX-01-P1-01, F-LLM-01-P1-01, F-LLM-02-P1-01, F-LLM-03-P1-01..04), application **2** (F-EXT-01-P1-01, F-EXT-01-P1-02).
- No P0 anywhere in the framework phase (no data-loss/corruption via a live happy path, no secret exposure beyond local logs, no core-path-unusable defect reached the P0 bar; the closest candidates — F-RCT-05-P1-01 context wipe and F-MEM-01-P1-01 silent store loss — are conditional/recovery-path and were held at P1 per the REPORTING.md priority definitions).
- Tasks with zero P1 findings (11): F-API-01, F-CORE-01, F-EVO-01, F-FEAT-01, F-MEM-02, F-NBK-01, F-PLG-01, F-RCT-01, F-SEC-01, F-TSK-01, F-TSK-03. Their P2/P3 findings (feature topology always-on files/shell, six no-op marker features, README/manifest drift, dead task-graph surface, dormant sqlite menu options, notebook aspirational API, plugin path listing divergence, approval cache/scope P2s, Retrying variant, etc.) remain part of the framework review and feed S-RDM-01.

## 4. Duplicate Finding Reconciliation (canonical merges)

Per REPORTING.md Synthesis Rules, duplicates were merged under one canonical ID with backlinks retained. Merges found in-scope:

1. **F-SKL-01-P1-02 ↔ F-RCT-05-P1-03** (merged; canonical **F-SKL-01-P1-02**). F-RCT-05 independently re-verified the same dual-SkillRegistry resume divergence and explicitly designated F-SKL-01-P1-02 the canonical ID ("fix belongs with it"). F-RCT-05-P1-03 is retained as an alias backlink to the F-RCT-05 validation evidence (V02-01) and its dynamic resume framing. Count effect: −1.
2. **F-FEAT-01-P2-01 ↔ F-API-01-P2-02** (P2; merged; canonical **F-FEAT-01-P2-01**). Both independently filed the same "files/shell effectively always-on" defect (echo_execution `default = ["files","shell"]` + ungated facade re-exports). Q-FW-02 V01 compile-confirmed the merged claim (sqlite-only build compiles 8 tree-sitter crates). F-API-01-P2-02 is an alias; the fix (gate `tools::shell`/`tools::files`, trim echo_execution defaults) is owned by the feature-topology task. Count effect: −1 within P2.
3. **F-EXT-01-P1-01 ↔ A-TOOL-01-P1-01** (cross-phase alias; canonical for this synthesis stays **F-EXT-01-P1-01**). The A-phase report mirrors the same writer-subagent-read-only defect from the application side; the F report owns the framework-facing chain (`plan_mode` filter vs writer builder). Reconciliation of the A-side copy belongs to S-APP-01/S-X-01; this synthesis records the alias so the roadmap does not double-count it. Count effect: none here (A-phase out of scope).
4. **F-RCT-02-P2-04 ↔ F-RCT-03-P2-02** (P2; same Stop-hook continuation defect; F-RCT-03 "independently confirmed rather than copied"). Canonical **F-RCT-02-P2-04**; F-RCT-03-P2-02 adds the envelope-truncation (raw-stream second-FinalAnswer) evidence and is retained as evidence-extending alias; fixes must land together.
5. **F-CMP-01-P1-03 ↔ F-CTX-01-P2-02 / F-RCT-01-P2-02**: NOT merged — distinct arms of one canonical-authority problem (rules duplication/staleness vs L1-fold pairing break) with separate fix surfaces; cross-linked instead. Same for **F-SUB-01-P1-01 ↔ F-RCT-01-P2-01** (different fields, same enforcement-gap class) and **F-SKL-01-P3-04 / F-MEM-02-P3-02 / F-MEM-01-P3-03 ↔ F-EXT-01-P1-02** (registration-overwrite class: F-EXT-01-P1-02 canonical for the ToolManager overwrite; others are distinct entry points, not duplicates).

No in-scope contradictions between reports required reopening a validation:
the only cross-report tension found — B-ARCH-01's "13 marker features are empty
no-ops" vs F-FEAT-01's refined classification (7 of 13 gate real code, 6 pure
no-ops) — was already resolved inside F-FEAT-01 with fresh evidence
(V01/V02 census) and is recorded there as a refinement, not a contradiction.

### 4.1 Resolved cross-report tension: Q-FW-01 "gate green" vs Q-FW-02 V14 "doctest phase red"

The apparent conflict between Q-FW-01 (all-features test gate exit 0, 1,930
tests) and Q-FW-02 V14 (`cargo test --doc -p echo_agent --all-features
--locked` exit 101, 81 passed / 1 failed) was resolved by reopening the
smallest relevant validation (S-FW-01 V02-01 evidence + this rerun):

1. **Re-run of V14 on 2026-08-12** at the same commit reproduces the failure
   exactly: exit 101, `test result: FAILED. 81 passed; 1 failed; 25 ignored`,
   failing doctest `src/testing/mod.rs - testing (line 24)` (the stale
   `CompressionInput` initializer missing `focus_instructions`). Q-FW-02 V14
   is **confirmed correct**.
2. **Q-FW-01's exit 0 is also correct**: its command was `cargo test
   --workspace --all-targets --all-features --locked`. The preserved log
   artifact `/tmp/qfw01_v04_test.log` contains **zero "Doc-tests" lines**;
   per Cargo's documented semantics, `--all-targets` does not include doc
   tests. Q-FW-01 V04's note "Doctests included in the run" is **factually
   incorrect** and should receive a dated Correction by its owner (flagged
   for S-QA-01).
3. **Conclusion**: the two reports are not contradictory — they measured
   different scopes. The literal AGENTS.md gate command does not exercise
   doctests, so the all-features doctest phase is RED at the baseline while
   the literal gate is GREEN: **a gate blind spot, recorded as an open
   question for S-QA-01 / the roadmap** (the framework's own mandatory gate
   misses the Q-FW-02-P2-02 staleness; "gate green" must not be read as
   "doctests pass").

## 5. Minority / Uncertain Conclusions Preserved As Open Questions

Per Synthesis Rules, uncertain or minority conclusions are preserved, not
erased. Open questions carried into S-RDM-01 / S-X-01:

1. **Provider-dependence of adapter findings**: F-LLM-03-P1-02 (some
   DeepSeek-Anthropic gateways echo `input_tokens` into `message_delta` and
   would parse — usage loss may not occur there); F-LLM-02-P1-01 (OpenAI 400
   behavior documented externally, not locally executed; gateway strictness
   varies); F-RCT-04-P1-01 (provider ordering constraint from ecosystem docs;
   OpenAI path unaffected). Confidence medium, not high — dynamic live-provider
   confirmation is a Q-E2E-01 follow-up.
2. **F-RCT-05-P1-01 in-process wipe** is statically derived beyond the probe's
   checkpoint-rejection proof; the full restart wipe chain is a Q-E2E-01 target.
3. **F-SKL-01-P1-02 / F-RCT-05-P1-03** fresh-process resume was not executed
   dynamically (needs the EKO restart harness); medium confidence on end-to-end
   impact, high on the code asymmetry.
4. **F-CTX-01-P1-01 overrun magnitude** (kimi 256K vs hardcoded 396K) is argued
   from window arithmetic, not executed with a live provider.
5. **F-MEM-01-P2-01 cross-process store loss** (GUI + CLI on one scheduler
   store) is statically assessed; no multi-process harness exists.
6. **F-HITL-01-P1-03 "which UI action maps to which scope"** on EKO surfaces is
   A-HITL-01 territory; the framework-side contract defect is proven
   independent of it (console provider alone reaches the `"*"` mapping).
7. **TaskManager/TaskExecutor legacy surface** (F-TSK-01-P3-01): retained as a
   framework capability-menu item per AGENTS.md deletion rules; deletion
   decision deferred to the roadmap with the framework-wide grep gate.
8. **F-INT-02-P2-08 RS256** and other adapter items are P2 but the A2A family
   has zero EKO consumers — impact is framework-contract-only until a consumer
   exists.
9. **Q-STA-01-P2-01 (`parse_guard_response`) reachability** is feature-gated
   (`guard`, non-default); the panic is reproduced, the production trigger
   depends on consumers enabling the feature.
10. **EKO-side unknowns**: whether `enable_direct_answer` waste
    (F-INTENT-01-P2-02) and intent-classifier unboundedness (F-INTENT-01-P2-01)
    are user-visible today depends on A-phase wiring verification; flagged for
    S-APP-01.
11. **Gate blind spot (open question for S-QA-01 / roadmap)**: the literal
    AGENTS.md test gate (`cargo test --workspace --all-targets
    --all-features --locked`) does not execute doctests (`--all-targets`
    excludes them; Q-FW-01's log has zero Doc-tests lines), so the red
    all-features doctest phase (Q-FW-02-P2-02, `src/testing/mod.rs:39`) is
    invisible to the mandatory gate. Decide: extend the gate with a `--doc`
    leg, or accept the blind spot. Q-FW-01 V04's "Doctests included in the
    run" note is incorrect and needs a dated Correction by its owner.

## 6. Stale-Commit Check

All 38 F-* reports, Q-FW-01, Q-FW-02, Q-STA-01, and Q-FLT-01 declare reviewed
commits `9b0e0faf…` (echo-agent) and `b3b2e81f…` (echo-agent-cli) — identical
to the shared baseline (docs/comprehensive-review/README.md). Verification on
2026-08-12 (V03-01): `git rev-parse HEAD` returns exactly those hashes on both
repos, both working trees clean, and no commit exists after the baseline. Every
finding's file:line anchors were therefore evaluated against the same code the
reports cite; **zero findings are stale due to commit drift**. Findings remain
subject to per-report stale triggers (listed in each task report's handoff) if
the code changes before the roadmap lands. The only within-scope "stale"
markers are documentation claims the reports themselves classified as stale or
regressed (e.g., README feature tables, MASTER-PLAN terminal-convergence and
resume-skip claims) — those are findings, not evidence decay.

## 7. Framework Health Verdict

**The framework compiles, lints, and tests green under its literal submission
gate (Q-FW-01: fmt 0 diff, all-features clippy 0 warnings, panic-safety clippy
0 hits, 1,930 tests 0 failed, no-default clean — noting the gate's test leg
does not execute doctests, whose all-features phase is red per Q-FW-02 V14;
see open question 11) — yet its runtime correctness contracts are materially
weaker than its green gate suggests.**

### 7.1 Strong subsystems

- **Build/feature hygiene**: single-authority definitions across the facade
  (F-API-01 V02), gate green, 12/12 standalone feature compiles (Q-FW-02 V01–V12).
- **Task model convergence**: one revisioned task model + one PlanValidator +
  one `RuntimeDagExecutor` with CAS-protected claims, safe points, bounded
  waves and no second frontier/retry/stall loop in EKO (F-TSK-01/02/03) — the
  M13 architecture holds; zero `worker` terms, zero CLI SQLite, zero parallel
  task CRUD (X-INV-01).
- **Plugin/skill lifecycle**: transactional reload with real source-scoped
  unload and rollback (F-PLG-01 V02/V03-02); single skill runtime authority
  (F-SKL-01 V01) apart from the one registry divergence (P1-02).
- **Tool execution core**: UTF-8-safe spill/truncation with artifact integrity
  (F-EXT-01 V03, Q-FLT-01 V03), shell/run_code process-group cancellation and
  fail-closed sandbox floor (F-EXT-02 V02, F-SEC-01 V03).
- **Hardened file backends**: FileConversationStore/FileRuntimeStateStore
  explicit corrupt-file errors, path-safe ids, atomic writes (F-MEM-01 V02/V03).
- **Panic-safety macro gate**: zero production unwrap/expect/panic!/
  unreachable! in both workspaces under the executable clippy gate (Q-STA-01
  V01); remaining panics are narrow, reachable slice bugs (see 7.2).

### 7.2 Systemic defect families (the review's headline conclusions)

**Family 1 — the "silent failure" family (the dominant pattern, ~20 P1s).**
Errors and losses are logged, dropped, or converted to success instead of
surfacing: malformed SSE chunks dropped (F-LLM-01-P1-01, F-LLM-03-P1-01/02/04),
events dropped on full channels (F-RCT-03-P1-01), loop errors swallowed into
`Ok("")` (F-RCT-02-P1-01), truncated streams finalized as complete answers
(Q-FLT-01-P1-01), mid-stream timeouts never retried (Q-FLT-01-P1-02), stubs
fabricating "stored successfully" (F-EXT-03-P1-01), FileStore wiping corrupt
files (F-MEM-01-P1-01), cron silently never firing (F-OPS-01-P1-01), circuit
breaker silently never gating (F-REL-01-P1-01), terminal-less turn ends
(F-RCT-04-P1-02), and no typed cancel terminal anywhere on the main loop
(F-RCT-03-P1-02). Root causes repeat: lenient-parse + drop-instead-of-error,
best-effort `try_send` on terminal paths, log-instead-of-forward error paths,
scaffolded-but-unwired APIs (LoopDetector, timeout_strategy, `exit_on_error`,
telemetry metrics, notebook, HandoffManager, ProviderAdapter), and
name-list/flag classifications without a single enforcement authority
(`WRITE_TOOLS`, readonly subset, `allowed_tools`, `tool_filter`,
`execution_mode`). This family is the top roadmap priority: it is where
"works while appearing healthy" lives.

**Family 2 — terminal/event integrity is only guaranteed at the envelope
adapter, not in the loop.** One-terminal, truthful-status, and typed-cancel
invariants hold only because `envelope_event_stream` truncates at the first
terminal and fabricates an Error on terminal-less ends (F-RCT-03-P2-02/P2-04,
F-RCT-02-P2-01, F-RCT-04-P1-02, F-RCT-03-P2-04). Fixes must move the guarantee
into the loop (terminal before continuation, finalize on every exit) rather
than strengthening the mask; do not build a second adapter authority.

**Family 3 — the mock invisibility cloak (mock 隐身衣), now evidenced.**
The loop-level suite is green because `MockLlmClient`/`MockTool`/`MockAgent`
model wire shapes no real provider produces: single-chunk streams with
content+usage together, loud-error cancellation, Permanent-only tool failures
(F-TST-01-P1-01/P1-02/P2-01/P2-03). The F-LLM-03-P1-02 usage loss, F-RCT-04-P1-01
ordering, F-RCT-03-P1-02 cancel, and Q-FLT-01-P1-01 truncation all shipped under
green suites that certify the opposite (Q-TST-01-P1-01..03). Fix the mocks
first, then re-run the loop suite against real wire shapes.

**Family 4 — detached execution / lifecycle leaks.** Team timeout/cancel leaves
members running (F-SUB-02-P1-01/02), handoff spawns uncancellable targets
(F-MAG-01-P1-01), QQ send task spins a core after stop (F-INT-02-P1-02), A2A
cancel is cosmetic (F-INT-02-P1-03), LSP hangs block shutdown (F-INT-02-P1-01),
sessions never evict and drop does not stop channels (F-INT-02-P2-04/P2-05).
A cancellation/ownership contract exists for Sync/Fork/Teammate; it must be
extended to every spawn site, not re-argued per module.

**Family 5 — the panic family (narrow, but real).** Four live UTF-8 byte-slice
panics (X-INV-01-P1-01 pdf, X-INV-01-P1-02 eval, Q-STA-01-P1-01 percent_decode,
Q-STA-01-P2-01 guard) plus F-EXT-02-P1-01 (empty-needle edit), F-EXT-03-P1-03
(IQR n=4), and the F-SKL-01-P1-01 cyclic-dependency stack overflow (process
abort). Each is a small, precisely-testable fix; the AGENTS.md no-panic rule
needs a byte-slice lint pass (Q-STA-01 V03 was the second independent scan and
still found two new sites).

**Family 6 — scope/authority drift at boundaries.** Approval scope collapse and
the `"*"` session wildcard (F-HITL-01-P1-03), session "approve all" (A-HITL),
and the blocked-reason string-literal cross-repo contract (F-TSK-02-P2-01) are
typed-contract violations at adapter boundaries; they silently widen or break
behavior on EKO's main paths.

### 7.3 Overall verdict

The framework's architecture (single authorities, feature isolation, hardened
file patterns, CAS task claims) is the right skeleton and is mostly in place —
this is a review of **wiring and contract enforcement gaps**, not of design
direction. The dominant theme is that dozens of advertised contracts
(documented fields, features, tool descriptions, events, retry/timeout
policies) are inert or silently lossy, and the green test gate cannot see it
because the mocks certify the lossy shapes. Priority for the roadmap
(correctness and data integrity first, per README.md): the silent-failure and
terminal-integrity families (F-RCT-02..05, Q-FLT-01-P1-01/02, F-LLM-01/03,
F-CMP-01-P1-01/02, F-MEM-01-P1-01, F-OPS-01-P1-01), then mock fidelity
(F-TST-01), then the detached-execution family, then scope/approval
(F-HITL-01), then the panic family, then dead-surface cleanup (which already
has a 32-row deletion-target matrix from X-BND-01 for S-RDM-01).

## 8. Handoff

- Downstream tasks may rely on: the canonical P1 table (section 2) with
  backlinks; the merge map (section 4); the open questions (section 5); the
  stale verdict (section 6: zero stale); the health verdict and family
  decomposition (section 7).
- **S-APP-01**: reconcile the A-side mirrors (A-TOOL-01-P1-01 = F-EXT-01-P1-01;
  A-HITL-01 scope expansion on F-HITL-01-P1-03; F-INTENT-01-P2-01/P2-02 EKO
  visibility; F-EXT-03-P1-01 research-memory impact on A-DOM-01/A-MEM-01).
- **S-X-01**: use the merge map and the boundary-contract findings
  (F-TSK-02-P2-01 string blocker, F-HITL-01-P1-03 scope bridges, F-SKL-01-P3-01/02
  five-parser and probe duplication, F-MEM-02 backend-drift P2s, F-WFL-01-P3-08
  dead app model) as canonical duplicate-authority input.
- **S-RDM-01**: every P1 row above carries its regression-validation target
  (per-finding "Regression validation" in the owning report) as measurable
  acceptance; deletion targets are named in the owning reports (dead
  `run/approval.rs` + `process_steps`, `ContextBuilder`/scoped-context types,
  `TeamRunner`/`TeamCoordinator`/mailbox, handoff module or feature, notebook
  module, `LoopDetector` if unwired decision, `TaskStatus::Retrying`,
  `ToolRiskClassifier`, `result_cache`, legacy `TaskManager`/`TaskExecutor`,
  six no-op marker features). Do not introduce a second authority in any staged
  migration (e.g., fix F-HITL-01 by porting, not by a parallel ask path; unify
  retry under `echo_core::retry`, not a fourth implementation).
- Stale triggers for this synthesis: any commit on either repo after
  9b0e0fa/b3b2e81 touching the anchors above; a toolchain change; or the
  re-architecture of any finding's owning module.
- Follow-up tasks: S-APP-01, S-X-01, S-QA-01, S-RDM-01 (all pending in
  TASKS.md Phase S).
