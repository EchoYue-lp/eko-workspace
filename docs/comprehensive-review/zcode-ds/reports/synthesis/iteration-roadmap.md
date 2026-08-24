# S-RDM-01: Prioritized Iteration Roadmap (ZCode-ds)

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Synthesis date: 2026-08-12
> `echo-agent` commit reviewed: 9b0e0faf74d35c9a432370b923acabfbb5f32d63 (= baseline 9b0e0fa)
> `echo-agent-cli` commit reviewed: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5 (= baseline b3b2e81)
> Worktree state: both repositories clean; HEAD identical to the shared baseline (verified for this task in V01-01/V03-01)
> Deliverable contract: TASKS.md S-RDM-01 — canonical finding IDs; P0-P3 order; framework/application/adapter placement; dependency DAG; cross-repository merge order; deletion targets; estimated scope; regression validations; measurable acceptance; proposed implementation milestones small enough for fresh tasks. Validations: V01 (every roadmap item backlinks to evidence), V02 (every critical design decision backlinks to mature implementation research), V03 (no duplicate authority is left as an indefinite migration state) — see validation reports.

## 1. Question And Method

**Question: in what order should the two repositories be fixed so that
correctness and data integrity come first, every known defect is carried by
exactly one implementation task, no second authority is introduced or left
behind, and every milestone is small enough for one fresh task?**

This roadmap is the final deliverable of the ZCode-ds review track. It
consumes the four completed phase syntheses (S-FW-01 framework-review.md,
S-APP-01 application-review.md, S-X-01 cross-repository-review.md, S-QA-01
quality-and-validation-review.md), the placement map with the 32-row
deletion-target matrix (X-BND-01), and the mature-implementation reference
matrix (B-REF-01). Per REPORTING.md Synthesis Rules it (a) orders work by
correctness and data integrity first, then authority convergence and
layering, surface parity, maintainability, performance, and documentation
(README.md Final Deliverables); (b) counts every canonical defect exactly
once; (c) backlinks every roadmap item to the syntheses and their
validations; (d) backlinks every critical design decision to B-REF-01; and
(e) terminates every staged migration with a deletion inside the same or an
adjacent milestone — no duplicate authority survives as an indefinite
migration state (AGENTS.md "未完全收敛必须显式归档" is bounded, not open).

No source file was modified (read-only). No `codex/` or `zcode-glm/` material
was read.

## 2. Canonical Finding Census (total check)

**Total: 80 canonical P1 findings, 0 P0 findings.**

| Phase synthesis | Canonical P1 | P0 | Source table |
|---|---:|---:|---|
| S-FW-01 (framework) | 49 | 0 | framework-review.md §2 (clusters A–J) |
| S-APP-01 (application) | 25 | 0 | application-review.md §1 (themes 1–6) |
| S-X-01 (cross-repository) | 6 | 0 | cross-repository-review.md §2 |
| **Total** | **80** | **0** | — |

Merge/alias bookkeeping (counted once each in the roadmap):

1. **F-SKL-01-P1-02 is canonical**; F-RCT-05-P1-03 is a merged alias
   (framework-review.md §4.1).
2. **A-TOOL-01-P1-01 is canonical** for the writer-subagent-read-only defect;
   **F-EXT-01-P1-01 is a cross-phase alias backlink** (cross-repository-review.md
   §6.1). The roadmap carries this defect once, in M10.
3. **A-TSK-03-P1-01 is canonical** for pause-in-wave (merged A-TSK-04-P1-01);
   **A-TSK-03-P2-01 re-rated P1** (merged A-TSK-04-P1-02) is a distinct
   recovery error and is carried separately (application-review.md §1 Theme 1,
   §4).
4. **A-CFG-01-P1-03** merges A-SRF-01-P1-01; **A-SRF-04-P1-02** re-rates
   A-BOOT-01-P2-02; **A-MEM-01-P1-01** is canonical for the hot-projection
   refresh (application-review.md §4).
5. **Q-phase P1 verdicts are not additional canonical defects but are
   carried as milestone inputs**: Q-TST-01-P1-01..03 (test credibility,
   S-QA-01 §3) feed milestone M1; Q-FLT-02-P1-01 (pause-crash strand,
   preserved as an open question with high mechanism confidence in S-QA-01
   §6) feeds M8 alongside A-TSK-03 — it is the one Q-phase P1 added to the
   milestone inventory **in addition to** the 80 canonicals, and is counted
   separately; Q-E2E-01-P1-01..03 are scenario-level verdicts of canonical
   defects (Q-E2E-01-P1-02 = A-TOOL-01-P1-01, Q-E2E-01-P1-03 = F-OPS-01-P1-01;
   cross-repository-review.md §2.1, quality-and-validation-review.md §7.6).
   Q-DEP-01-P2-01 (RUSTSEC) and Q-PERF-01-P2-01 are P2 and appear in M19/M21
   (owning reports [Q-DEP-01](../tasks/Q-DEP-01.md), [Q-PERF-01](../tasks/Q-PERF-01.md)).
6. P2/P3 canonicals (duplicate authority, dead surface, layering, gate
   hygiene, doc contract) are carried by the milestones below through the
   D1-D32 matrix and the quality inventory (X-BND-01 §Deletion-target matrix;
   S-QA-01 §7).

## 3. Mature Implementation Reference (B-REF-01, C1-C7)

Constraint set from B-REF-01 V05-01 (the cross-system convergence report;
convergence = supported by ≥3 independent implementations of Claude Code,
OpenAI Codex, Cursor/Devin, Temporal). Every critical design decision in this
roadmap cites one of these; the V01–V04 per-system lookups are
[V01-01](../validations/B-REF-01/V01-01.md) (Claude Code plan mode),
[V02-01](../validations/B-REF-01/V02-01.md) (Codex event stream/sandbox),
[V03-01](../validations/B-REF-01/V03-01.md) (Temporal durable execution),
[V04-01](../validations/B-REF-01/V04-01.md) (Cursor/Devin plan-then-execute).

| ID | Converged pattern | Evidence row (B-REF-01 V05-01) | Roadmap use |
|---|---|---|---|
| **C1** | Plan = editable artifact + permission-gated approval; **no run-level plan-approval state machine** | CC markdown plan + checkPermissions; Codex approval policy flags; Cursor propose→confirm→execute; Temporal approval via signals/updates | M8 pause fix, M9 approval port, M13 skip propagation |
| **C2** | Append-only event/rollout record as **recovery authority** | Codex JSONL rollout + SQLite index; Temporal event history; Claude transcript partial | M6 checkpoints, M8 events.jsonl authority + delete cascade |
| **C3** | Durable side-effect recording, **payload-before-event, no re-execution on recovery** | Codex payload-before-event; Temporal SideEffect | M6 paired-prefix checkpoints, M13 NextStep checkpoint |
| **C4** | **Typed terminal events**, no text scraping | Codex typed items/events; Temporal typed history events; CC typed tool results | M2, M3, M11 typed terminals end-to-end |
| **C5** | **Sandbox and approval policy separable**, typed concepts | CC permission modes + rules; Codex SandboxPolicy × ApprovalPolicy | M9, M10 (permission wiring stays separate from sandbox) |
| **C6** | **Review is a separate agent/phase after writing** | Devin review agent; CC critic/teammate; Codex review command | M10 (EKO review/acceptance policy stays separate from execution), M17 (A-EVO-01-P1-01 → Review Inbox) |
| **C7** | Background/parallel agents are **first-class with lifecycle/cancellation ownership**; subagents excluded from interactive approval where possible | CC teams/teammates; Codex threads/subagents; Cursor background agents; Temporal workflows/activities | M12 cancellation ownership at every spawn site; M10 fail-closed background policy |

## 4. Roadmap Phases And Milestones (M1-M22)

Milestones are ordered per README.md: **correctness and data integrity →
authority convergence and layering → surface parity → maintainability →
performance → documentation**. Each milestone fits one fresh task (AGENTS.md
"小步快跑" 5-10 rounds; REPORTING.md context budget); where a milestone is
still too wide, a suggested split is given — each split is itself a fresh
task. Every milestone states: canonical finding IDs (with validation
backlinks), repository ownership and placement, dependency order, deletion
targets (D rows), regression validations, measurable acceptance, estimated
scope, and cross-repository merge order where both repos are touched.

Merge-order convention used everywhere: **echo-agent merges to main first,
then echo-agent-cli** (AGENTS.md cross-repo dependency rule). Within a
milestone that touches both, the framework-side change lands in `echo-agent`
before the consumer side lands in `echo-agent-cli`.

### Phase 1 — Correctness And Data Integrity (M1-M14)

#### M1 — Test-credibility re-basing (mock 隐身衣 removal) — prerequisite for every silent-failure fix

- **Canonical IDs**: F-TST-01-P1-01, F-TST-01-P1-02, Q-TST-01-P1-01, Q-TST-01-P1-02, Q-TST-01-P1-03 ([framework-review.md §2.9](../framework-review.md), [quality-and-validation-review.md §3](../quality-and-validation-review.md))
- **Repo / placement**: `echo-agent`; framework (test infrastructure).
- **Dependency order**: none — first milestone. Blocks M2/M3/M4 (fixes land with failing-then-passing fixtures).
- **Deletion targets**: print-only `test_sliding_window_compressor` and toy `Mutex`-only summary test (Q-TST-01-P2-01); stale "deferred" full-loop claim in `tests/react_smoke.rs` (quality-and-validation-review.md §7).
- **Regression validations**: chunk-sequence mock API (deltas / usage-only final chunk / finish / mid-stream Err); loop-level test asserting `usage_reported` semantics on the real wire shape; re-basing of `pipeline.rs:1634-1720` (`multiplexed_streams_preserve_identity_and_terminal_order`) from completion order to stream-index (call) order — the negative control for F-RCT-04-P1-01.
- **Acceptance**: each prescribed fixture fails before its fix and passes after; the re-based pipeline test asserts `["call-a","call-b"]` order; `cargo test --workspace --all-targets --all-features --locked` green with the new fixtures.
- **Estimated scope**: ~4-6 files (`src/testing/mock_llm.rs`, `src/testing/mock_tool.rs`, `react/tests.rs`, new non-streaming loop test family), ~400-600 LOC.
- **Evidence**: [F-TST-01 V03-01](../validations/F-TST-01/V03-01.md), [F-TST-01 V01-01](../validations/F-TST-01/V01-01.md), Q-TST-01 V01-01..V05.

#### M2 — Loop terminal integrity (silent-failure core)

- **Canonical IDs**: F-RCT-02-P1-01, F-RCT-03-P1-01, F-RCT-03-P1-02, F-RCT-04-P1-01, F-RCT-04-P1-02 ([framework-review.md §2.1](../framework-review.md))
- **Repo / placement**: `echo-agent`; framework (ReAct run loop: `react_loop.rs`, `stream_macros.rs`, `finalize.rs`, `phases/tools.rs`, `processor.rs`).
- **Dependency order**: after M1.
- **Critical design decision → C4**: typed terminal events must be guaranteed **in the loop**, not by strengthening the envelope mask — "fixes must move the guarantee into the loop (terminal before continuation, finalize on every exit) rather than strengthening the mask; do not build a second adapter authority" (framework-review.md §7.2 Family 2). The envelope truncation becomes pure defense after this lands.
- **Deletion targets**: none mandatory; after landing, the envelope's terminal fabrication paths (F-RCT-03-P2-02/P2-04, F-RCT-02-P2-04) may be simplified to assert-only (optional cleanup, not a second authority).
- **Regression validations**: full-channel event-drop fixture (F-RCT-03 V01/V02); cancel → `AgentEvent::Cancelled` emitted with trace finalization; strict-provider conformance fixture for batch results in stream-index order (F-RCT-04 V01); batch timeout/cancel → typed terminal + `finalize_run`, verifier-accepted `final_answer` preserved (F-RCT-04 V02/V03).
- **Acceptance**: one-terminal and truthful-status invariants hold at the loop level without envelope fabrication; non-streaming turn returns the real error instead of `Ok("")`; existing Q-FLT-01/Q-E2E-01 scenario pairs stay green.
- **Estimated scope**: ~8-10 files, ~600-900 LOC. Suggested split if too large: (a) terminal emission/cancel (F-RCT-02-P1-01, F-RCT-03-P1-01, F-RCT-03-P1-02); (b) batch ordering and batch terminal (F-RCT-04-P1-01, F-RCT-04-P1-02).
- **Evidence**: [F-RCT-03 V01-01](../validations/F-RCT-03/V01-01.md), [F-RCT-04 V01-01](../validations/F-RCT-04/V01-01.md), [F-RCT-02 V02-01](../validations/F-RCT-02/V02-01.md).
- **Merge order / consumers**: producer for M11 (X-EVT-01-P1-01/P1-02, A-CHAT-01-P1-01) — must land in `echo-agent` before M11's CLI side.

#### M3 — Provider stream contract (truncation, timeout, dropped chunks)

- **Canonical IDs**: Q-FLT-01-P1-01, Q-FLT-01-P1-02, F-LLM-01-P1-01 ([framework-review.md §2.1](../framework-review.md))
- **Repo / placement**: `echo-agent`; adapter layer (`providers/client.rs`, `run/phases/think.rs`, `finalize.rs`, `run/retry.rs`).
- **Dependency order**: after M1, M2.
- **Critical design decision → C4**: consult `finish_reason`/`stop_reason` at stream end — truncated/disconnected streams must never be finalized as `Completed`; malformed chunks become counted typed drops instead of `warn!`+`None`.
- **Regression validations**: truncated and clean-disconnect stream fixtures (Q-FLT-01 V01/V06); mid-stream stall timeout retried per `RetryPolicy` (Q-FLT-01 V04); malformed-chunk fixture with counted drop.
- **Acceptance**: no partial output ever terminates `Completed`; mid-stream timeout retried or surfaced with a typed error; streaming/non-streaming error handling aligned; `usage_reported` only false when the provider truly omitted usage.
- **Estimated scope**: ~3-4 files, ~300 LOC.
- **Evidence**: [Q-FLT-01 V01-01](../validations/Q-FLT-01/V01-01.md), [Q-FLT-01 V04-01](../validations/Q-FLT-01/V04-01.md), [F-LLM-01 V02-01](../validations/F-LLM-01/V02-01.md).

#### M4 — Anthropic/OpenAI adapter wire fixes

- **Canonical IDs**: F-LLM-03-P1-01, F-LLM-03-P1-02, F-LLM-03-P1-03, F-LLM-03-P1-04, F-LLM-02-P1-01 ([framework-review.md §2.1](../framework-review.md))
- **Repo / placement**: `echo-agent`; adapter layer (`providers/anthropic.rs`, `providers/openai.rs`, `types.rs`).
- **Dependency order**: after M1 (Anthropic SSE wire fixtures are the Q-TST-01-P1-03 gap), M2.
- **Regression validations**: interleaved [text, tool_use] block fixtures keyed by stream `index`; literal `message_delta.usage` payload fixture (output_tokens only); multi-system collapse fixture; thinking-block fixtures for streaming and non-streaming; request-JSON fixtures per model family for `max_completion_tokens`.
- **Acceptance**: `usage_reported` true on real Anthropic streaming; tool calls preserved on interleaved streams; all leading system messages retained in order; thinking blocks parsed and mapped to `reasoning_content`; reasoning models receive `max_completion_tokens` only.
- **Estimated scope**: ~3 files, ~400-600 LOC.
- **Evidence**: [F-LLM-03 V03-01](../validations/F-LLM-03/V03-01.md), [F-LLM-03 V01-01](../validations/F-LLM-03/V01-01.md), [F-LLM-02 V01-01](../validations/F-LLM-02/V01-01.md).

#### M5 — Compression and window integrity

- **Canonical IDs**: F-CMP-01-P1-01, F-CMP-01-P1-02, F-CMP-01-P1-03, F-CTX-01-P1-01 ([framework-review.md §2.1](../framework-review.md))
- **Repo / placement**: `echo-agent`; framework (`echo-state/src/compression/`, `echo-core/src/llm/capabilities.rs`, `agent/config.rs`).
- **Dependency order**: independent (may run parallel to M2-M4).
- **Regression validations**: few-large-messages over-limit fixture with explicit over-limit signal; repeated-pass summary-growth fixture; adaptive-L1 contiguity regression test (no `Role::User` between tool_calls and kept results); per-model-family window regression (kimi 256K, 1M-window models).
- **Acceptance**: compression never exceeds the token limit silently; exactly one running system summary; pairing-contiguity invariant holds on the adaptive strategy; `token_limit` derived from the model profile with explicit override winning.
- **Estimated scope**: ~5-6 files, ~500 LOC.
- **Evidence**: [F-CMP-01 V02-01](../validations/F-CMP-01/V02-01.md), [F-CMP-01 V03-01](../validations/F-CMP-01/V03-01.md), [F-CTX-01 V02-01](../validations/F-CTX-01/V02-01.md).

#### M6 — Resume/checkpoint and scheduler recovery

- **Canonical IDs**: F-RCT-05-P1-01, F-RCT-05-P1-02, F-SKL-01-P1-02, F-MEM-01-P1-01, F-OPS-01-P1-01 ([framework-review.md §2.2](../framework-review.md))
- **Repo / placement**: `echo-agent`; framework (`run/state/mod.rs`, `run/context.rs`, `stream_channel.rs`, `capabilities.rs`, `echo-state/src/memory/store.rs`, `scheduler/runner.rs`).
- **Dependency order**: independent; uses M1 fixtures for dynamic resume tests.
- **Critical design decisions**: **C2/C3** — the checkpoint is a durable event-record append point: persist only the paired prefix at cancel/error arms (no re-execution, no fabricated state), and on validator rejection preserve the previous in-process state instead of wiping; cron fires relative to a last-tick reference (`schedule.after(&last_tick).next()` in `(last_tick, now]`) per Temporal schedule semantics.
- **Regression validations**: interrupt-mid-tool-batch → resume fixture (F-RCT-05 V02/V04); steer fixtures with `run_id != turn_id` and `run_id = None`; SkillRegistry save/restore round-trip (F-SKL-01 V02); corrupt-store-file fixtures (F-MEM-01 V02/V03); cron tick unit test proving `next > now` fires exactly once per window.
- **Acceptance**: resume never wipes conversation context; same-turn steer delivered on the EKO main path; skills activated after fresh-process resume; `FileStore::new` errors on unparseable content instead of silently wiping; cron tasks actually fire.
- **Estimated scope**: ~8 files, ~500-700 LOC.
- **Evidence**: [F-RCT-05 V02-01](../validations/F-RCT-05/V02-01.md), [F-SKL-01 V02-01](../validations/F-SKL-01/V02-01.md), [F-MEM-01 V02-01](../validations/F-MEM-01/V02-01.md), [F-OPS-01 V04-01](../validations/F-OPS-01/V04-01.md).

#### M7 — UTF-8 panic batch (small, high-yield)

- **Canonical IDs**: X-INV-01-P1-01, X-INV-01-P1-02, Q-STA-01-P1-01, F-EXT-02-P1-01, F-EXT-03-P1-03 (+ Q-STA-01-P2-01 guard arm) ([cross-repository-review.md §2](../cross-repository-review.md), [framework-review.md §2.10, §2.6](../framework-review.md))
- **Repo / placement**: `echo-agent`; framework (`echo-tools/src/pdf.rs`, `src/eval/runner.rs`, `echo-tools/src/web/providers/utils.rs`, `echo-tools/src/files/edit.rs`, `echo-tools/src/data_quality.rs`).
- **Dependency order**: none — first correctness batch per cross-repository-review.md §7 ("the panic pair is small and testable").
- **Regression validations**: `%`+CJK fixture; non-ASCII PDF date fixture; multilingual eval-output fixture; empty `old_content` edit; 4-value IQR column (+n=1, n=2 arms); guard-parse fixture.
- **Acceptance**: all reproduced exit-101 sites return typed errors; multibyte regression tests per site; panic-safety clippy gate (AGENTS.md) stays zero; a byte-slice lint pass finds no new sites.
- **Estimated scope**: ~7 files, ~300 LOC — one fresh task.
- **Evidence**: [X-INV-01 V01-01](../validations/X-INV-01/V01-01.md), [Q-STA-01 V03-01](../validations/Q-STA-01/V03-01.md), [F-EXT-02 V01-01](../validations/F-EXT-02/V01-01.md), [F-EXT-03 V03-03](../validations/F-EXT-03/V03-03.md).

#### M8 — EKO persistence/crash recovery + deletion cascade

- **Canonical IDs**: A-TSK-01-P1-01 (merged A-TSK-04-P1-03), A-TSK-03-P1-01 (merged A-TSK-04-P1-01), A-TSK-03-P2-01 (re-rated P1, merged A-TSK-04-P1-02), A-STATE-01-P1-01, X-TSK-01-P3-01 (with A-TSK-01-P2-01), X-STA-01-P1-01 (+ Q-FLT-02-P1-01 pause-crash strand) ([application-review.md §1 Theme 1](../application-review.md), [cross-repository-review.md §2](../cross-repository-review.md))
- **Repo / placement**: `echo-agent-cli` (application: `task_runtime/file_shadow.rs`, `executor.rs`, `state.rs`, `conversations.rs`) with one `echo-agent` framework API addition (`RuntimeStateStore::delete_conversation`, `echo-agent/src/state/mod.rs`).
- **Dependency order**: independent of M2-M7; merge order inside the milestone: framework trait addition to `echo-agent` first, then the CLI consumer.
- **Critical design decisions**: **C2/C3** — `events.jsonl` is the append-only recovery authority (Codex rollout / Temporal history): torn-tail repair truncates to the last valid record boundary (no re-execution); the deletion cascade completes the record lifecycle (mirrors `ConversationStore::delete_conversation`). **C1** — the pause-in-wave fix consults `controller.interruption_outcome` after the wave drain and writes `Pending`, not `Cancelled`: no new run-level state machine (S-APP-01 handoff, application-review.md §1 Theme 1).
- **Deletion targets**: `Persistence::base_dir()`-based conversation-store construction in `exit_workspace` (replaced by `infra::create_conversation_store()`); unguarded `set_task_status` block writes hardened (A-TSK-04-P2-01) before the deleted-run cleanup path relies on them.
- **Regression validations**: torn-tail fixture (A-TSK-01 V03-02); pause-during-active-wave fixture; mid-wave store-fault fixture (sibling cleanup + resume); exit_workspace store-root fixture; deleted-conversation id-reuse fixture (deleted context never restored).
- **Acceptance**: a torn tail never bricks a run; pause stays resumable; store fault marks failed and clears siblings so resume does not poll forever; conversation deletion removes `runtime_state/<id>/`; exiting a workspace keeps global history readable.
- **Estimated scope**: ~6-8 files, ~600 LOC.
- **Evidence**: [A-TSK-01 V03-02](../validations/A-TSK-01/V03-02.md), [A-TSK-03 V03-01](../validations/A-TSK-03/V03-01.md), [A-STATE-01 V02-01](../validations/A-STATE-01/V02-01.md), [X-STA-01 V01-01](../validations/X-STA-01/V01-01.md), [X-TSK-01 V01-01](../validations/X-TSK-01/V01-01.md).

#### M9 — Approval/HITL live path + permission-rule wiring

- **Canonical IDs**: F-HITL-01-P1-01, F-HITL-01-P1-02, F-HITL-01-P1-03 ([framework-review.md §2.3](../framework-review.md)); A-HITL-01-P1-01, A-HITL-01-P1-02, A-HITL-01-P1-03 ([application-review.md §1 Theme 2](../application-review.md))
- **Repo / placement**: `echo-agent` first (framework `service.rs`/`snapshot.rs` live approval path), then `echo-agent-cli` (application `hitl/` providers, GUI rule commands).
- **Dependency order**: none mandatory; A-HITL-01-P1-02 (REPL provider) is standalone.
- **Critical design decisions**: **C1/C5** — approval stays a permission/prompt gate inside the single `PermissionService` pipeline: port the ask semantics from the dead `run/approval.rs` into the live `check_tool_approval` (a parallel ask path is forbidden — framework-review.md §8); sandbox and approval policy remain separable. **C4-adjacent** — approval scope is carried as the response's true scope with tool-scoped rules; the `"*"` session wildcard mapping is removed.
- **Deletion targets**: D8 (`run/approval.rs` + `process_steps` + `execute_tool_feedback*`) — deleted in this milestone **after** the port; D9 (`TauriHumanLoopHandler` parallel transport) after GUI approvals route through `HitlDispatcher`; D10 (`IpcAuth`/`IpcPermission`, 3 modules).
- **Regression validations**: live-path ask fixture reaching the human provider; modified-args round-trip (F-HITL-01 V04-02); three-granularity scope tests (tool/session/task); REPL EOF → `Rejected` fixture; GUI rule-application fixture.
- **Acceptance**: `RequireApproval`/`Ask` produce a human request on every surface; user-modified args execute; no `"*"` rule is ever inserted; GUI permission rules actually apply to tool calls; EOF no longer auto-approves and the shared deadline is respected.
- **Estimated scope**: ~10 files across both repos, ~700 LOC. Suggested split: (a) framework port + scope (F-HITL-01-P1-01/02/03); (b) EKO surfaces (A-HITL-01-P1-01/02/03).
- **Evidence**: [F-HITL-01 V01-01](../validations/F-HITL-01/V01-01.md), [A-HITL-01 V01-01](../validations/A-HITL-01/V01-01.md), [A-HITL-01 V02-01](../validations/A-HITL-01/V02-01.md).

#### M10 — Subagent automation permission boundary + writer capability (batch of X-AUT-01-P1-01 + A-TOOL-01-P1-01)

- **Canonical IDs**: X-AUT-01-P1-01, A-TOOL-01-P1-01 (canonical; F-EXT-01-P1-01 alias) ([cross-repository-review.md §5](../cross-repository-review.md), [application-review.md §1 Theme 6](../application-review.md))
- **Repo / placement**: `echo-agent-cli` first (adapter: `infra.rs:881-1010` subagent factories), `echo-agent` optional (framework: no-service fallback deny for Write/Execute/Network/Sensitive).
- **Dependency order**: after M9 (permission wiring available). Must be coordinated with A-TOOL-01-P1-01 because both touch `build_writer_subagent_agent`/`build_readonly_subagent_agent` — removing `set_plan_mode(true)` must not be masked by the permission wiring, and vice versa (cross-repository-review.md §5).
- **Critical design decisions**: **C5/C7** — writer/readonly subagents receive the shared `PermissionService` + current mode at factory construction (mirroring `agent_pool.rs:928-932`); background runs use a fail-closed empty-dispatcher provider policy (subagents excluded from interactive approval where possible → deny, never hang); the sandbox floor and worktree containment stay as physical bounds (separation retained).
- **Regression validations**: subagent `run_code`/web call in default mode → approval request or denial, never `Ok(None)`; `.git/config` writes denied on subagents; write-tool visibility test for Implementation/Debugging agents; Q-E2E-01-P1-02 scenario rerun.
- **Acceptance**: the permission-mode matrix governs subagent automation; Task write work actually completes on every surface (Q-E2E-01-P1-02 green); zero permission gate on direct-user interactions (terminal/file picker/MCP config) — AGENTS.md historical lesson preserved.
- **Estimated scope**: ~4-6 files, ~400 LOC.
- **Evidence**: [X-AUT-01 V01-01](../validations/X-AUT-01/V01-01.md), [A-TOOL-01 V02-01](../validations/A-TOOL-01/V02-01.md), [Q-E2E-01 V01-01](../validations/Q-E2E-01/V01-01.md).

#### M11 — Terminal semantics convergence (envelope → wire → reducer → webhook → trace)

- **Canonical IDs**: X-EVT-01-P1-01, X-EVT-01-P1-02 ([cross-repository-review.md §2](../cross-repository-review.md)); A-CHAT-01-P1-01, A-SRF-03-P1-01, A-SRF-03-P1-02, A-OBS-01-P1-01, A-OBS-01-P1-03 ([application-review.md §1 Themes 3 and 6](../application-review.md))
- **Repo / placement**: `echo-agent` (envelope `event_envelope.rs`, subagent status mapping) then `echo-agent-cli` (adapter wire `chat.rs`, frontend `chatStore.ts`/`chatEventHandler.ts`, application `chat_driver.rs`, webhook `events.rs`, `save_trace`) — framework first.
- **Dependency order**: after M2 (producer fix), M8 (save_trace typed terminal), M9 (cancel-aware path).
- **Critical design decision → C4**: one typed terminal per turn at every layer — `drive_chat` returns a typed `TurnOutcome`; `ChatEvent` gains cancel/timeout variants; the TS reducer derives `TurnStatus` from the typed terminal and never labels error/cancel/timeout turns `'completed'`; webhook gains `chat_cancelled`/`chat_failed`.
- **Deletion targets**: X-EVT-01-P3-01 dead `ChatEvent::Cancelled` variant — rewired in this milestone (not deleted while live); after M2, the envelope truncation becomes assert-only (simplification option).
- **Regression validations**: cancelled/timed-out subagent persists as cancelled, not `failed` (X-EVT-01 V01); timed-out turn ends with a typed Timeout terminal; GUI error/cancel turn keeps partial content and ends truthfully (chatStore monotone terminal transitions); webhook variant fixtures; save_trace terminal record fixture.
- **Acceptance**: Q-E2E-01-P1-01 GUI chat error/cancel turns complete with equivalent facts; `'completed'` never used for error/cancel/timeout on any surface; external consumers (webhooks) can distinguish outcomes.
- **Estimated scope**: ~10 files across both repos, ~800 LOC. Suggested split: (a) envelope + wire + status mapping (X-EVT-01-P1-01/P1-02); (b) frontend reducer + webhook + trace (A-SRF-03-P1-01/P1-02, A-OBS-01-P1-01/P1-03, A-CHAT-01-P1-01).
- **Evidence**: [X-EVT-01 V01-01](../validations/X-EVT-01/V01-01.md), [A-CHAT-01 V02-01](../validations/A-CHAT-01/V02-01.md), [A-SRF-03 V02-01](../validations/A-SRF-03/V02-01.md), [A-OBS-01 V02-01](../validations/A-OBS-01/V02-01.md).

#### M12 — Detached execution lifecycle ownership

- **Canonical IDs**: F-SUB-01-P1-01, F-SUB-02-P1-01, F-SUB-02-P1-02, F-MAG-01-P1-01 ([framework-review.md §2.4](../framework-review.md)); F-INT-02-P1-01, F-INT-02-P1-02, F-INT-02-P1-03 ([framework-review.md §2.7](../framework-review.md))
- **Repo / placement**: `echo-agent`; framework (subagent/team/handoff, LSP/QQ/A2A integrations).
- **Dependency order**: independent (may run parallel to M2-M6).
- **Critical design decision → C7**: the existing cancellation/ownership contract (Sync/Fork/Teammate) is extended to every spawn site — token threaded through `TeamAgent::execute_with_usage`, `JoinSet::abort_all` on fan-out, handoff reimplemented over `SubagentExecutor::dispatch` (Sync + child token) or token+timeout with the manager mutex released before awaiting; one invocation authority for `tool_filter` → `disabled_tools`.
- **Deletion targets**: D6 (`TeamCoordinator`/`TeamRunner`/`mailbox.rs`/`message.rs`) and D7 (`src/handoff/` module or feature) — deleted in this milestone once the orchestrator/dispatch covers the surface; D4 (7 dead `SubagentDefinition` fields) optional here or in M16.
- **Regression validations**: hung-member fixture (V04-02); cancel-token fixtures at planning/fan-out/synthesis; handoff cancel + timeout fixture (Q-FLT-02 fixtures must fail pre-fix); LSP stub-server timeout; QQ stop-lifecycle test (Feishu pattern).
- **Acceptance**: no spawn site without cancellation/ownership; team timeout aborts members (no detached side effects); handoff targets cancellable and bounded; LSP shutdown bounded; QQ send task stops on `stop()`; subagent `tool_filter` actually restricts tools.
- **Estimated scope**: ~12 files, ~900 LOC. Suggested split: (a) team + handoff + tool_filter (F-SUB-01-P1-01, F-SUB-02-P1-01/02, F-MAG-01-P1-01); (b) integrations (F-INT-02-P1-01/02/03).
- **Evidence**: [F-SUB-02 V01-01](../validations/F-SUB-02/V01-01.md), [F-MAG-01 V01-01](../validations/F-MAG-01/V01-01.md), [F-INT-02 V02-01](../validations/F-INT-02/V02-01.md), [F-SUB-01 V01-01](../validations/F-SUB-01/V01-01.md).

#### M13 — MCP transport + task graph/workflow recovery

- **Canonical IDs**: F-INT-01-P1-01, F-INT-01-P1-02 ([framework-review.md §2.7](../framework-review.md)); F-TSK-02-P1-01, F-WFL-01-P1-01 ([framework-review.md §2.5](../framework-review.md))
- **Repo / placement**: `echo-agent`; framework (MCP HTTP transport, task runtime, workflow engine).
- **Dependency order**: independent.
- **Critical design decisions**: **C1** — Skip/cancel satisfy readiness at the safe point instead of new terminal states ("treat `Skipped` as satisfying readiness or propagate at the safe point; differentiate the stall reason"); **C3** — AfterNode checkpoint persists the pending `NextStep` (targets + then, or the granted-before-interrupt fact) and replays it in `resume`.
- **Regression validations**: fake 202/SSE server fixtures; `tools/call` non-retry fixture (retry kept only for reads, `ToolFailure` surfaced to the manager gate); framework fixture A(Skipped)→B; AfterNode two regression fixtures (fan-out replay, before-interrupt re-run).
- **Acceptance**: compliant Streamable HTTP servers complete (or reject loudly, never hang 60 s); ambiguous `tools/call` failures are never duplicated by the transport; skipping a task with Pending dependents no longer stalls with "cycle or blocked"; resume replays pending fan-out branches and approval gates.
- **Estimated scope**: ~6 files, ~600 LOC.
- **Evidence**: [F-INT-01 V02-01](../validations/F-INT-01/V02-01.md), [F-TSK-02 V03-01](../validations/F-TSK-02/V03-01.md), [F-WFL-01 V03-03](../validations/F-WFL-01/V03-03.md).

#### M14 — Domain tool and facade correctness

- **Canonical IDs**: F-EXT-01-P1-02, F-EXT-03-P1-01, F-EXT-03-P1-02, F-INTENT-01-P1-01, F-MAC-01-P1-01, F-REL-01-P1-01 ([framework-review.md §2.6, §2.8](../framework-review.md))
- **Repo / placement**: `echo-agent`; framework (echo-tools, echo-execution pool, intent router, macros facade, circuit breaker).
- **Dependency order**: independent (F-EXT-02-P1-01 and F-EXT-03-P1-03 panics already fixed in M7).
- **Regression validations**: pooled-agent memory-tool routing fixture (registration conflict observable); research-store persistence or registry-removal fixture (never report success without storing); readonly-subset derivation test (`bibtex_generate`/`rag_index` excluded or validated); facade-only `#[derive(Tool)]` compile fixture; breaker-open regression test.
- **Acceptance**: no silent memory-tool overwrite across pool agents; `research_remember` persists (or is removed from registries); no write-permission tool on the readonly surface; `ToolRunner` exported from the facade; the circuit breaker gates repeated failures instead of passive telemetry; skill-activation retry fires at the aligned threshold.
- **Deletion targets**: D13 (`ToolRiskClassifier`/`ToolRiskCategory`), D14 (`ToolManager::result_cache`), D15 (3 of the 4-5 URL-download tools — merge into one family with a single size-cap policy), D16 (duplicate `parse_page_range`).
- **Estimated scope**: ~8 files, ~700 LOC.
- **Evidence**: [F-EXT-01 V02-01](../validations/F-EXT-01/V02-01.md), [F-EXT-03 V03-03](../validations/F-EXT-03/V03-03.md), [F-INTENT-01 V01-01](../validations/F-INTENT-01/V01-01.md), [F-MAC-01 V03-01](../validations/F-MAC-01/V03-01.md), [F-REL-01 V01-01](../validations/F-REL-01/V01-01.md).

### Phase 2 — Authority Convergence And Layering (M15-M16)

#### M15 — EKO authority convergence (one authority per semantic)

- **Canonical IDs**: X-BND-01-P2-01 (three project-root resolvers), A-OBS-01-P2-01 (`save_trace` second ledger), A-PROJ-01-P2-03 (three diff engines + duplicate types + dead `DiffViewer.tsx`), A-OUT-01-P2-01 (second `export_conversation`), X-BND-01-P3-01 (`safe_segment` copy), A-CFG-01-P2-03 (dead `web_config`), A-CFG-01-P2-05 (dual `DEFAULT_CONTEXT_WINDOW`), A-SRF-02-P2-01 / A-FE-02-P2-01 (duplicate tool-event projection producer), A-EVO-01-P3-02 (dead `reflect_on_session`), A-FE-03-P3-05 (second frontend auth-check), A-TSK-05-P2-04 (`panels.rs` worktree helpers), F-WFL-01-P3-08 (EKO `WorkflowDef`/`WorkflowStep`) ([cross-repository-review.md §3.4](../cross-repository-review.md), [application-review.md §1 cross-references](../application-review.md))
- **Repo / placement**: `echo-agent-cli` (application copies); one `echo-agent` hoist for `safe_segment` into `echo-state`.
- **Dependency order**: after M11 (`save_trace` typed terminal facts land before the second-ledger deletion) and M8.
- **Deletion targets**: D20, D21, D23, D24, D25, D26, D27, D28, D30, D31, D32.
- **Regression validations**: four-consumer same-project-root fixture (instruction files, project context, rule promotion, memory store resolve identically, incl. after `exit_workspace`); diff hunk parity tests across GUI/REPL; export parity test; worktree helper parity; tool-event dedupe tests (frontend + backend).
- **Acceptance**: grep-verified single authority per semantic on the application side; zero second resolver/diff engine/export/trace ledger/projection producer; framework `InstructionResolver` documented as the instruction-file authority (distinct purpose); gates green.
- **Estimated scope**: ~12 files, ~800 LOC. Suggested split: (a) resolvers/config/auth (D26, D30, D32, D28); (b) diff/export/trace/worktree/projection (D21, D23, D24, D25, D31, D20).
- **Evidence**: [X-BND-01 V02-01](../validations/X-BND-01/V02-01.md), [X-BND-01 V04-01](../validations/X-BND-01/V04-01.md), [A-PROJ-01 V02-01](../validations/A-PROJ-01/V02-01.md), [A-OBS-01 V02-01](../validations/A-OBS-01/V02-01.md).

#### M16 — Framework authority convergence + legacy surface deletion

- **Canonical IDs**: F-REL-01-P2-01 (`retry_llm_call` second authority), F-CTX-01-P2-03 (`ContextAssembler`/`ContextSelector`), F-LLM-01-P2-03 (`ProviderAdapter`/`AdapterClient`), F-SUB-01-P2-01 (7 dead `SubagentDefinition` fields), F-SUB-01-P2-03 (`ContextBuilder`/`OutputSchema`/`MemoryScope`/`isolated.rs`), F-SUB-02-P2-03 (Team surface), F-MAG-01-P2-01 (handoff second registry), F-NBK-01-P2-01 (notebook dead API), F-TSK-01-P3-01 (legacy `TaskManager`/`TaskExecutor`/`TaskHooks`/`VerifierFactory`), F-TSK-02-P2-02 (inert `execution_mode: sequential`), F-TSK-02-P3-01 (`refresh_in_flight`), F-FEAT-01-P2-01 (files/shell effectively always-on; merged F-API-01-P2-02), F-SKL-01-P3-01/P3-02 (frontmatter parsers, binary probes), F-PLG-01-P3-03 (second plugin data-dir computation), F-INT-02-P2-08 (A2A RS256 decision) ([framework-review.md §4, §7.3, §8](../framework-review.md))
- **Repo / placement**: `echo-agent`; framework across sub-crates.
- **Dependency order**: after M4 (D12 thinking-protocol authority moves with M4's thinking fixes), M12 (D6/D7), M14 (D13-D16). The `echo-agent` framework deletion gate applies: framework-wide grep + "capability menu" judgment before each pub-API deletion (AGENTS.md; X-BND-01 V03).
- **Critical design decision → C5 (separation) + AGENTS.md one-authority gate**: retry unifies under `echo_core::retry` (never a fourth implementation); the six no-op marker features and the legacy task surface are deleted, not kept as "menu items" (B-ARCH-01/F-FEAT-01 census: 6 pure no-ops); `execution_mode: sequential` is removed or enforced with a schema test.
- **Deletion targets**: D1, D2, D3, D4, D5, D11, D12, D17, D18, D19, D22, D29 (plus D6/D7 if deferred from M12).
- **Regression validations**: per-deletion: full `echo-agent` submission gate (fmt / both clippy configs / all-features tests / no-default check) + the specific fixtures named in each D row (X-BND-01 matrix); capability-menu tests retained for kept options (e.g., `SqliteStore`, `HybridCompressor` stay as framework options).
- **Acceptance**: zero live duplicate authorities in the framework; every deletion has a named regression; AGENTS.md gate + 12-feature standalone matrix green; zero `worker` terms remain.
- **Estimated scope**: ~20 files across crates, ~1200 LOC — split into 2-3 fresh tasks: (a) retry/LLM/context (D11, D12, D22); (b) subagent/team/handoff (D4-D7); (c) skill/plugin/notebook/feature topology (D17-D19, D29, F-FEAT-01-P2-01, D1-D3).
- **Evidence**: [X-BND-01 V03-01](../validations/X-BND-01/V03-01.md), [F-REL-01 V01-01](../validations/F-REL-01/V01-01.md), [F-CTX-01 V02-01](../validations/F-CTX-01/V02-01.md), [F-FEAT-01 V01-01](../validations/F-FEAT-01/V01-01.md).

### Phase 3 — Surface Parity (M17-M18)

#### M17 — Workspace-scope convergence

- **Canonical IDs**: A-CFG-01-P1-01, A-CFG-01-P1-02, A-CFG-01-P1-03 (merged A-SRF-01-P1-01), A-PLG-01-P1-01, A-MEM-01-P1-01, A-EVO-01-P1-01 ([application-review.md §1 Themes 1 and 4](../application-review.md))
- **Repo / placement**: `echo-agent-cli`; application (state.rs switch/exit, config watcher, hook loader, plugin runtime, unified memory, REPL exit path).
- **Dependency order**: after M8 (same `exit_workspace` family), M15 (project-root resolvers converge first so all consumers re-scope consistently).
- **Critical design decision → C6**: REPL session reflection is removed or routed through the Review Inbox (review gate separate from writing); every workspace-scoped subsystem (watcher/hooks/config/plugins/memory) follows the workspace.
- **Regression validations**: switch-reload fixtures (watcher targets, hook registry, AppConfig re-merge, plugin reload, LSP root per apply); CWD-restore fixture; TUI/REPL workspace command fixtures; memory hot-layer refresh fixture (8 wrong-target sites).
- **Acceptance**: workspace switch re-scopes all subsystems; `exit_workspace` restores CWD; REPL/TUI workspace switching is real (parity matrix row flips from ✗/stub to ✓); MEMORY.md mutations refresh the hot-memory projection; no unaudited automatic memory write on REPL exit.
- **Estimated scope**: ~10 files, ~800 LOC. Suggested split: (a) switch/exit scope (A-CFG-01-P1-01/02/03, A-PLG-01-P1-01); (b) memory refresh + evolution (A-MEM-01-P1-01, A-EVO-01-P1-01).
- **Evidence**: [A-CFG-01 V02-01](../validations/A-CFG-01/V02-01.md), [A-PLG-01 V02-01](../validations/A-PLG-01/V02-01.md), [A-MEM-01 V02-01](../validations/A-MEM-01/V02-01.md), [A-EVO-01 V02-01](../validations/A-EVO-01/V02-01.md).

#### M18 — Mode/surface parity

- **Canonical IDs**: A-SRF-04-P1-01, A-SRF-04-P1-02 (re-rated A-BOOT-01-P2-02), A-INP-01-P1-01 ([application-review.md §1 Themes 3 and 4](../application-review.md)); X-SRF-01-P2-01, X-SRF-01-P2-02, X-SRF-01-P3-01 ([cross-repository-review.md §4](../cross-repository-review.md)); A-SRF-02-P1-01, A-INT-01-P1-01 ([application-review.md §1 Theme 6](../application-review.md))
- **Repo / placement**: `echo-agent-cli`; application (main.rs modes, REPL/channel turn handles, Tauri setup, MCP config, frontend).
- **Dependency order**: after M12 (cancel tokens exist for REPL/channel turns), M11 (terminals truthful before parity is measured).
- **Critical design decision → parity invariant (AGENTS.md)**: all six entry classes (GUI/TUI/CLI/channels/cron/background) share one core; gaps are surface-adapter gaps to close, not "mode doesn't use Y" policy. Channels-only mode calls the shared headless-service assembly; REPL/channel turns become cancellable; the browser://event bridge is revived via a single `.setup()`.
- **Deletion targets**: none new; A-INT-01-P1-01 replaces the non-persistent GUI MCP config path with atomic persistence + boot seeding; dead frontend `exit()` gets a real command (A-CFG-01-P1-02 sibling).
- **Regression validations**: channels-mode service fixtures (scheduler + background task service present); REPL/channel cancel fixtures (signal handler + per-turn token); steer fixtures on REPL/channel; builder-level test asserting exactly one `.setup()` closure; MCP persist + seed round-trip.
- **Acceptance**: parity matrix rows flip to ✓ (task-run management on channels, workspace on TUI/REPL, REPL/channel cancel and steer, browser panel alive, MCP config durable across restart); Q-E2E-01 scenario pairs green per surface.
- **Estimated scope**: ~12 files, ~900 LOC. Suggested split: (a) channels/REPL services + cancel/steer (A-SRF-04-P1-01/P1-02, A-INP-01-P1-01, X-SRF-01-P2-02/P3-01); (b) GUI management surfaces (A-SRF-02-P1-01, A-INT-01-P1-01, X-SRF-01-P2-01).
- **Evidence**: [A-SRF-04 V02-01](../validations/A-SRF-04/V02-01.md), [A-SRF-02 V02-01](../validations/A-SRF-02/V02-01.md), [A-INT-01 V02-01](../validations/A-INT-01/V02-01.md), [A-INP-01 V02-01](../validations/A-INP-01/V02-01.md).

### Phase 4 — Security And Dependencies (M19)

#### M19 — Outbound security and advisory upgrades

- **Canonical IDs**: A-OBS-01-P1-02 (webhook redaction; closest-to-P0 item, application-review.md §1 Theme 5), Q-DEP-01-P2-01 (6 active RUSTSEC advisories in the shipped binary: lopdf, quick-xml, crossbeam-epoch), plus Q-DEP-01-P3-01..06 and the indexed secret gaps F-OPS-01-P2-01/P2-04, F-SEC-01-P3-11 ([quality-and-validation-review.md §7](../quality-and-validation-review.md), [cross-repository-review.md §4](../cross-repository-review.md))
- **Repo / placement**: `echo-agent-cli` (redaction choke point in the observer/emitter; dependency bumps) and `echo-agent` (dependency bumps for lopdf/quick-xml/crossbeam-epoch).
- **Dependency order**: after M11 (webhook typed variants land first); advisory bumps independent.
- **Critical design decision → local threat model (AGENTS.md)**: this is the only outbound surface — redaction applies at a single choke point (`redact_secrets` + bounded truncation) before serialization; no permission gating added to direct-user interactions.
- **Deletion targets**: dompurify + @types/dompurify if unused (Q-DEP-01-P3-01).
- **Regression validations**: redaction fixture at the choke point (secrets never leave the process); `cargo audit` clean for the shipped binary after bumps; `npm audit` clean.
- **Acceptance**: zero raw tool-arg/error secrets in webhook payloads; zero active advisories in the reachable dependency graph; advisory gate wired into CI (Q-DEP-01-P3-06).
- **Estimated scope**: ~4-6 files + Cargo.toml bumps, ~400 LOC.
- **Evidence**: [A-OBS-01 V01-01](../validations/A-OBS-01/V01-01.md), [Q-DEP-01 V01-01](../validations/Q-DEP-01/V01-01.md), [Q-DEP-01 V03-01](../validations/Q-DEP-01/V03-01.md).

### Phase 5 — Maintainability And Quality Gates (M20)

#### M20 — Quality gates and doc contract

- **Canonical IDs**: Q-FW-02-P2-02 (all-features doctest red + gate blind spot), Q-FW-02-P2-01 (demo45 required-features), Q-TST-01-P2-01..03 (second-tier test gaps), Q-GUI-01-P3-01 (no boot-composition test), Q-WEB-01-P3-01 (= A-FE-01-P3-02, ts-rs regeneration formatting), A-FE-03-P3-04 (chatStore reducer untested), Q-FW-02-P3-01 (28 unresolved intra-doc links) ([quality-and-validation-review.md §2, §7](../quality-and-validation-review.md))
- **Repo / placement**: `echo-agent` + `echo-agent-cli` (gate definitions live in AGENTS.md; doc fix in `src/testing/mod.rs`).
- **Dependency order**: after M1 (second-tier test fixtures), M15 (revisioned_adapter round-trip tests).
- **Critical design decision → gate truthfulness**: the mandatory gate must see the doc contract — extend the gate with a `--doc` leg (or default `cargo test`), so a stale doc example fails the gate instead of being invisible (`--all-targets` excludes doctests; quality-and-validation-review.md §2.2). The stale `CompressionInput` initializer at `src/testing/mod.rs:39` is the first red item.
- **Regression validations**: `cargo test --doc -p echo_agent --all-features --locked` exits 0; a deliberately stale doc example turns the gate red; demo45 compiles with exactly its declared `required-features` (+`content-guard`); compressor assertion tests; revisioned_adapter round-trip tests; chatStore reducer tests; builder-level GUI boot-composition test; ts-rs generation wrapper keeps prettier green.
- **Acceptance**: doctest contract green and gate-visible; example contract verified for all examples; zero second-tier test gaps from the Q-TST-01 list; zero unresolved intra-doc links.
- **Estimated scope**: ~8 files + AGENTS.md gate wording, ~500 LOC.
- **Evidence**: [Q-FW-02 V14-01](../validations/Q-FW-02/V14-01.md), [Q-WEB-01 V03-01](../validations/Q-WEB-01/V03-01.md), [Q-GUI-01 V01-01](../validations/Q-GUI-01/V01-01.md), [Q-TST-01 V01-01](../validations/Q-TST-01/V01-01.md).

### Phase 6 — Performance (M21)

#### M21 — Performance

- **Canonical IDs**: Q-PERF-01-P2-01 (TaskRuntime file shadow O(N²) I/O on the executor critical path, real-data measured), Q-PERF-01-P3-01 (unbounded fanout), A-DOM-01-P2-02 (run-artifact destruction on rerun), F-CMP-01 P2 efficiency items ([quality-and-validation-review.md §7](../quality-and-validation-review.md), [application-review.md §1 cross-references](../application-review.md))
- **Repo / placement**: `echo-agent-cli` (file shadow read path), `echo-agent` (compression efficiency).
- **Dependency order**: after M5 (compression), M8 (file authority fixes — avoid conflicting with the crash-recovery rework).
- **Regression validations**: before/after measurement on the Q-PERF-01 real-data dataset; bounded fanout test.
- **Acceptance**: file-shadow read path no longer O(N²) (measured improvement with a named number on the Q-PERF-01 dataset); fanout bounded; compression efficiency without losing the M5 invariants.
- **Estimated scope**: ~3-4 files, ~300 LOC.
- **Evidence**: [Q-PERF-01 V01-01](../validations/Q-PERF-01/V01-01.md), [Q-PERF-01 task report](../tasks/Q-PERF-01.md).

### Phase 7 — Documentation (M22)

#### M22 — Documentation convergence

- **Canonical IDs**: Q-DOC-01 items, Q-FW-02-P3-01 (doc links), B-REF-01-P3-01 (safe-point terminology), README feature-table drift (F-API-01), MASTER-PLAN convergence records (F-TSK-03 "Phase 3" legacy task surface) ([quality-and-validation-review.md §7](../quality-and-validation-review.md), [B-REF-01 §Findings](../tasks/B-REF-01.md))
- **Repo / placement**: both repositories, docs only.
- **Dependency order**: after M15/M16 (record actual deletions and remaining migrations in `docs/MASTER-PLAN.md` per AGENTS.md "未完全收敛必须显式归档").
- **Deletion targets**: doc claims referencing deleted surfaces (README feature tables, stale audit sections); the local "safe point" term is retained but documented as "revision safe point = event-history append point" (B-REF-01-P3-01 direction).
- **Acceptance**: docs match code at HEAD; MASTER-PLAN lists every completed deletion and any remaining migration with its termination milestone; zero unresolved intra-doc links (with M20); no "worker" terminology in docs.
- **Estimated scope**: docs only, ~10-15 files.
- **Evidence**: [B-REF-01 V02-01](../validations/B-REF-01/V02-01.md), [B-REF-01 V03-01](../validations/B-REF-01/V03-01.md), Q-DOC-01 V01-01..V05-01.

## 5. Dependency DAG (milestone order)

```text
M1 (test re-basing) ──► M2 (loop terminals) ──► M3 (provider streams)
                  │                            └──► M4 (Anthropic/OpenAI)
                  └──► M20 (second-tier tests)     └──► M16 (D12, after thinking authority moves)

M2 ──► M11 (envelope/wire/reducer terminals; producer first)
M4 ──► M16
M5, M6, M7, M8, M12, M13, M14  ──  independent of each other and of M2-M4
M8 ──► M15 (store roots, save_trace facts) ──► M20 (adapter round-trip)
M9 ──► M10 (permission wiring before subagent boundary)
M9 ──► M11 (cancel-aware path)
M11 ──► M15 (save_trace ledger deletion) ──► M19 (webhook variants)
M12 ──► M16 (D6/D7), M18 (cancel tokens)
M14 ──► M16 (D13-D16)
M15 ──► M17 (resolvers first), M21
M5/M8 ──► M21 (compression/file authority)
M15/M16 ──► M22 (MASTER-PLAN records)
```

Parallelizable tracks after the M1 prerequisite: Track A = M2→M3→M4→M11;
Track B = M5, M6, M7, M13, M14 (framework correctness); Track C = M8, M9→M10,
M12 (framework/app recovery, approval, lifecycle); Track D = M15→M17→M18
(application convergence/parity); Track E = M19→M20→M21→M22 (security,
gates, performance, docs). Tracks B and C run concurrently; M11 and M15/M16
join the results.

## 6. Cross-Repository Merge Order

Rule (AGENTS.md): `echo-agent` merges to main first, then `echo-agent-cli`;
worktree conventions apply (relative `path` in Cargo.toml before merge,
merge main before squash, `-D` branch removal, `.worktrees/` in .gitignore).

| Milestone | echo-agent (framework) | echo-agent-cli (application) | Merge order |
|---|---|---|---|
| M1 | test infra + fixtures | — | framework only |
| M2-M7, M12-M14 | fixes | — | framework only |
| M8 | `RuntimeStateStore::delete_conversation` trait addition | torn-tail/pause/cascade consumers | framework first |
| M9 | live approval path + scope fix | rule wiring, REPL provider, GUI routing | framework first |
| M10 | optional no-service deny fallback | subagent factory permission wiring + plan-mode removal | CLI first (adapter owns the wiring), framework fallback after |
| M11 | envelope typed classes + subagent status mapping | wire + reducer + webhook + save_trace | framework first |
| M15 | `safe_segment` hoist (echo-state) | resolver/diff/export/trace/projection convergence | framework first |
| M16 | framework deletions | — | framework only |
| M17-M18 | — | application | CLI only |
| M19 | dependency bumps | redaction choke point + bumps | framework first |
| M20 | doctest + gate + examples | frontend/CLI gate items | framework first |
| M21 | compression efficiency | file shadow read path | framework first |
| M22 | framework docs | CLI docs + MASTER-PLAN | either |

Within a milestone touching both repos, each commit lands on the framework
side before the CLI side compiles against it; a milestone is complete only
when both sides are merged and the full CLI submission gate runs against the
framework HEAD.

## 7. Deletion-Target Mapping (D1-D32 → milestone)

| D | Target | Canonical source | Milestone |
|---|---|---|---|
| D1 | Legacy `TaskManager`/`TaskExecutor`/`TaskHooks`/`VerifierFactory` | F-TSK-01-P3-01, F-TSK-03 | M16 (framework gate) |
| D2 | inert `execution_mode: sequential` | F-TSK-02-P2-02 | M16 |
| D3 | `refresh_in_flight`/`DagRefresh` | F-TSK-02-P3-01 | M16 |
| D4 | 7 dead `SubagentDefinition` fields | F-SUB-01-P2-01 | M16 (or M12) |
| D5 | `ContextBuilder`/`OutputSchema`/`MemoryScope`/`isolated.rs` | F-SUB-01-P2-03 | M16 |
| D6 | `TeamCoordinator`/`TeamRunner`/`mailbox` | F-SUB-02-P2-03 | M12 |
| D7 | `src/handoff/` (module/feature) | F-MAG-01-P2-01/P2-02 | M12 |
| D8 | `run/approval.rs` + `process_steps` + `execute_tool_feedback*` | F-HITL-01-P2-03, F-RCT-02-P3-01/P3-02 | M9 (after port) |
| D9 | `TauriHumanLoopHandler` parallel transport | A-HITL-01-P2-01 | M9 |
| D10 | `IpcAuth`/`IpcPermission` | A-HITL-01-P2-04, A-TOOL-01-P3-01, A-SRF-02-P3-01 | M9 |
| D11 | `ContextAssembler`/`ContextSelector` | F-CTX-01-P2-03 | M16 |
| D12 | `ProviderAdapter`/`AdapterClient` | F-LLM-01-P2-03 | M16 (after M4) |
| D13 | `ToolRiskClassifier`/`ToolRiskCategory` | F-EXT-01-P3-02 | M14 |
| D14 | `ToolManager::result_cache` | F-EXT-01-P3-01 | M14 |
| D15 | 3 of 4-5 URL-download tools | F-EXT-03-P2-01 | M14 |
| D16 | duplicate `parse_page_range` | F-EXT-03-P3-06 | M14 |
| D17 | 3 of 5 frontmatter parsers | F-SKL-01-P3-01 | M16 |
| D18 | hub inline binary probe | F-SKL-01-P3-02 | M16 |
| D19 | second plugin data-dir computation | F-PLG-01-P3-03 | M16 |
| D20 | EKO `WorkflowDef`/`WorkflowStep` | F-WFL-01-P3-08 | M15 |
| D21 | GUI inline diff engine + dup types + `DiffViewer.tsx` | A-PROJ-01-P2-03 | M15 |
| D22 | `retry_llm_call` backoff | F-REL-01-P2-01 | M16 |
| D23 | `save_trace` second ledger | A-OBS-01-P2-01 | M15 (after M11) |
| D24 | second `export_conversation` | A-OUT-01-P2-01 | M15 |
| D25 | `panels.rs` worktree helpers | A-TSK-05-P2-04 | M15 |
| D26 | two EKO project-root resolvers | X-BND-01-P2-01 | M15 |
| D27 | `safe_segment` copy | X-BND-01-P3-01 | M15 |
| D28 | dead `reflect_on_session` | A-EVO-01-P3-02 | M15 |
| D29 | `NotebookTracker` module | F-NBK-01-P2-01 | M16 |
| D30 | `web_config` + dual `DEFAULT_CONTEXT_WINDOW` | A-CFG-01-P2-03/P2-05 | M15 |
| D31 | duplicate tool-event projection producer | A-SRF-02-P2-01 / A-FE-02-P2-01 | M15 |
| D32 | second frontend auth-check | A-FE-03-P3-05 | M15 |

Deletion execution order, per-repo ownership, and acceptance criteria were
explicitly delegated to this roadmap by X-BND-01-P2-02 (cross-repository-review.md §3.4, §7).

## 8. Migration Convergence Policy (no indefinite dual authority)

Every staged migration in this roadmap terminates with the old authority
deleted inside the same or an adjacent milestone; the AGENTS.md archive rule
("未完全收敛必须显式归档") is bounded per migration, never open-ended:

| Migration | New authority (lands in) | Old authority deleted in | Termination condition |
|---|---|---|---|
| Approval ask path | live `check_tool_approval` (M9) | D8 `run/approval.rs` (M9) | same milestone |
| Approval transport | `HitlDispatcher` (M9) | D9 `TauriHumanLoopHandler` (M9) | same milestone |
| GUI permission gates | `PermissionService` (M9) | D10 `IpcAuth`/`IpcPermission` (M9) | same milestone |
| Retry | `echo_core::retry` (M16) | D22 `retry_llm_call` (M16) | same milestone |
| Terminal integrity | loop-level guarantees (M2) | envelope fabrication → assert-only (M2/M16) | M2 lands, M16 simplifies |
| Trace facts | event ledger + typed terminal (M11) | D23 `save_trace` second ledger (M15) | adjacent milestone |
| Handoff | `SubagentExecutor::dispatch` (M12) | D7 `src/handoff/` (M12) | same milestone |
| Diff | one engine (M15) | D21 GUI inline engine + `DiffViewer.tsx` (M15) | same milestone |
| URL download | one fetch family (M14) | D15/D16 (M14) | same milestone |
| Project root | one EKO resolver (M15) | D26 inline walks (M15) | same milestone |
| Conversation deletion | `RuntimeStateStore::delete_conversation` (M8) | none (new capability) | n/a — additive with framework gate |
| Legacy task surface | `RuntimeDagExecutor` (M16) | D1-D3 (M16) | same milestone |

Constraint that makes this enforceable (X-BND-01-P2-02): a milestone that
introduces or retains a parallel implementation must name its deletion row
and milestone in the same commit series; a merge is not accepted with a live
duplicate without a tracked termination milestone. This roadmap carries no
"keep both until later" items.

## 9. Regression And Acceptance Framework

- **Submission gates (AGENTS.md, per-repo)**: every milestone passes the full
  applicable gate before merge — `echo-agent`: fmt check, both clippy
  configurations (incl. panic-safety `-D clippy::unwrap_used` etc.),
  all-features tests, no-default check, plus the feature matrix when feature
  topology changes (M16, M20); `echo-agent-cli`: same + GUI/feature checks
  and frontend prettier/vitest/build when `web-frontend/` is touched (M11,
  M15, M18, M19, M20).
- **Doctest gate (M20)**: `cargo test --doc -p echo_agent --all-features
  --locked` exits 0; the gate gains a `--doc` leg (Q-FW-02-P2-02).
- **Scenario baseline (Q-E2E-01)**: the 46 static scenario/surface pairs are
  the per-surface acceptance matrix; each milestone reruns the rows it
  touches (M10 → P1-02 Task write; M11 → P1-01 GUI chat terminals; M6/M8 →
  cron/restart rows; M17/M18 → parity rows).
- **Per-finding regression targets**: every canonical finding's owning report
  names its regression validation (listed per milestone above); each fix
  lands with a failing-then-passing fixture (M1 makes this possible).
- **Measurable acceptance**: each milestone's acceptance criteria above are
  concrete (exit codes, fixture outcomes, parity-row flips, measured
  performance numbers); none is "behaves better".

## 10. Open Questions And Product Decisions (carried, not erased)

Per Synthesis Rules, minority/uncertain conclusions are preserved as open
questions with a default direction so implementation is not blocked:

1. **X-STA-01-P1-01 framework API vs app-side removal** — resolved in this
   roadmap: add `RuntimeStateStore::delete_conversation` (M8), mirroring
   `ConversationStore::delete_conversation` (framework capability menu,
   AGENTS.md gate applies).
2. **A-INT-01-P2-03** GUI MCP URL rejection (SSRF-style over-gating vs
   deliberate) — default: keep the light URL scheme validation (AGENTS.md
   "明显错误输入" level), remove any wider gate; product decision recorded in
   M18.
3. **A-TOOL-01-P2-01** tool toggle gate vs display-only — product decision;
   default: display-only + visibility fix from A-TOOL-01-P1-01 first.
4. **A-SUB-01-P2-04 / A-PLG-01-P1-01** refresh-on-switch vs document-boot-only —
   default: refresh on switch (parity invariant), chosen in M17.
5. **A-TSK-06-P2-01** runtime artifact projection — default: delete (dead),
   batched with M15.
6. **A-CHAT-01-P2-01** interrupt routing — default: route through the shared
   sink (fixes the ghost-turn class with M11).
7. **A-SRF-01-P2-02** research-workbench placement on TUI — product decision;
   closing the row by documented decision is acceptable without a fix.
8. **F-MEM-01-P2-01** cross-process store loss — needs a multi-process
   harness; tracked with M8 (no committed date).
9. **Q-E2E-01 dynamic confirmations (V47-V49 not_run)** — needs credentials
   and a GUI-capable host; the static verdicts remain authoritative until
   then.
10. **F-INT-02-P2-08** A2A RS256 — framework-contract-only until a consumer
    exists; decision (support vs drop) recorded in M16.

## 11. Validation Matrix

| ID | Claim | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Every roadmap item backlinks to evidence — every canonical finding ID in the roadmap resolves in its phase synthesis; every cited validation link exists; each milestone carries at least one evidence link per finding | yes | passed | [V01-01](../validations/S-RDM-01/V01-01.md) |
| V02 | Every critical design decision backlinks to mature implementation research — each C1-C7 citation matches the B-REF-01 convergence matrix and its per-system lookups | yes | passed | [V02-01](../validations/S-RDM-01/V02-01.md) |
| V03 | No duplicate authority is left as an indefinite migration state — every staged migration has a termination milestone and deletion row; zero "keep both" items | yes | passed | [V03-01](../validations/S-RDM-01/V03-01.md) |

## 12. Coverage And Uncertainty

- The roadmap is a synthesis over the four completed phase syntheses, the
  X-BND-01 matrix, and B-REF-01; it does not re-read source beyond the
  anchors already cited by the task reports (both repositories verified at
  the baseline commits, V01-01/V03-01).
- Milestone scope estimates are planning bounds (files/LOC) derived from the
  finding anchors; actual scope is set by the implementing task.
- Milestones M2, M11, M12, M15, M16, M17, M18 carry suggested splits; each
  split is a fresh task, so no implementing task exceeds the context budget.
- P2/P3 canonicals not named individually above are carried by the D1-D32
  matrix rows and the quality inventory (S-QA-01 §7) referenced in M15/M16/
  M20; the 80-P1 census is the completeness baseline (section 2).

## 13. Handoff

- **Implementers may rely on**: the 80-P1 census with the merge map (section
  2); the C1-C7 constraint set (section 3); the 22-milestone order with
  per-milestone findings, ownership, dependencies, deletions, regression and
  acceptance (section 4); the DAG and merge order (sections 5-6); the
  D1-D32 mapping (section 7); the bounded migration policy (section 8); the
  acceptance framework (section 9); open questions with defaults (section
  10).
- **Reports to read**: this report + its three validations; the four phase
  syntheses; X-BND-01 (D1-D32); B-REF-01 (C1-C7); the owning task reports of
  each milestone's findings.
- **Stale triggers for this roadmap**: any commit on either repository after
  9b0e0fa/b3b2e81 touching the anchors cited above (the same triggers as
  S-FW-01/S-APP-01/S-X-01/S-QA-01, plus any change to the writer-builder
  plan-mode line which invalidates the A-TOOL-01-P1-01 alias ruling); a
  toolchain change; a re-architecture of a finding's owning module. When a
  stale trigger fires, re-run the owning synthesis's targeted revalidation
  and re-anchor the affected milestone(s).
- **Follow-up**: implementation tasks are created from individual milestones
  (or their splits); each implementing task must record its termination of
  the named deletions in `docs/MASTER-PLAN.md` per AGENTS.md.
