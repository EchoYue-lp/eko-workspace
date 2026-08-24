# F-REL-01: Retry, budgets, circuit breakers, and utility invariants

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean

## Question

Are generic retry/backoff, budget arithmetic, circuit breaker, hashing, time,
and JSON parsing primitives deterministic and safe under overflow,
cancellation, clock, and malformed-input edges?

## Scope

- `echo-core/src/retry.rs` (full read), `echo-core/src/budget.rs` (full read),
  `echo-core/src/circuit_breaker.rs` (full read), `echo-core/src/utils/`
  (hash/time/json_parse, full read), root facade `src/retry.rs` (full read).
- Second/third retry implementations found during V01:
  `src/agent/react/run/retry.rs` (full read), `echo-orchestration/src/tasks/executor.rs:100-160,1130-1230`.
- Live callers: `echo-integration/src/providers/client.rs:100-220`,
  `anthropic.rs:370-435`; engine LLM path `src/agent/react/run/react_loop.rs:20-140,750-780`,
  `phases/think.rs:270-380`; breaker wiring `src/agent/react/mod.rs:335-360,515-525,1360-1370`,
  `builder.rs:165-175,785-800,1005-1015`, `snapshot.rs:305-320`;
  channel reconnect backoffs `echo-integration/src/channels/channels/feishu/long_poll.rs:134-205,675-682`,
  `qq/channel.rs:134-189`; budget consumers `src/agent/react/mod.rs:340-346`,
  `echo-state/src/compression/mod.rs:470-480,1295-1315`; cancellation context
  `src/agent/react/run/pipeline.rs:515-560`, `phases/tools.rs:170-180,300-310`.

## Out Of Scope

- Tool-failure retry classification / idempotency contracts (F-EXT-01; the
  `ToolFailure::allows_automatic_retry` classification is consumed centrally).
- EKO TaskRuntime retry/recovery (A-series tasks; EKO has its own
  `circuit_breaker_action` in `echo-agent-app-core/src/tasks/task_runtime/review.rs` — application layer, different semantic).
- ReAct loop streaming/cancellation internals beyond the retry wait itself (F-RCT-02/03).
- EKO token-budget projection/trim policy (application layer).

## Inputs

- Root `AGENTS.md`, shared `README.md`, `REPORTING.md`, `TASKS.md`,
  `zcode-ds/README.md`.
- Dependency reports: zcode-ds `F-CORE-01` (complete) and `B-ARCH-01`
  (facade ownership; root is facade + engine, `src/retry.rs` is a pure
  re-export — confirmed here).
- Historical documents treated as hypotheses: root `docs/MASTER-PLAN.md`
  M3/M4 recovery claims, `AGENTS.md` framework/app layering rule for retry.

## Layering Decision

- Generic mechanism: retry/backoff/budget/circuit-breaker/hash/time/json are
  framework-core primitives — correctly placed in `echo_core` (B-ARCH-01
  consistent). The engine's own `retry_llm_call` duplicates this mechanism in
  the root package (P2-01).
- EKO product policy: none inside scope; EKO does not use the framework
  circuit breaker or `TaskExecutor` (V01-01).
- Adapter boundary: none — all reviewed primitives are framework-owned.
- Duplicate search terms (both repositories): `RetryPolicy`, `with_retry`,
  `with_retry_if`, `backoff`, `retry_llm_call`, `retry_delay_for_attempt`,
  `CircuitBreaker`, `try_advance`, `record_success`, `record_failure`,
  `TokenBudget`, `TokenBudgetConfig`, `allocate`, `TaskExecutor`,
  `fnv1a_64`, `now_secs`, `extract_json_from_markdown`, `clean_json`.
  Result: single budget, breaker, and utils authority; **three retry/backoff
  implementations** (two live, one dormant); root facade is a pure re-export
  (no second authority there).

## Current Path

Verified call graph:

1. **Provider retry (live)**: `OpenAiClient::chat` →
   `providers/client.rs:122,203` → `with_retry_if(RetryPolicy::default(), …)`
   (3 retries, 500ms base, 30s cap, jitter, 429/5xx/network classification) —
   `echo_core::retry`.
2. **Engine LLM retry (live)**: `think.rs:289,354` and `react_loop.rs:81,124`
   → `retry_llm_call` (`src/agent/react/run/retry.rs:13-68`) — own backoff
   (500ms base default, exponent cap 5, `fastrand` jitter, no duration cap),
   own retryable classification (`is_retryable_llm_error`,
   `src/agent/react/mod.rs:76-84`), and the **only** caller of
   `CircuitBreaker::record_success/record_failure` (`run/retry.rs:61,63`).
   `compute_concurrent_tool_batch_timeout` (`run/retry.rs:70-109`) feeds tool
   batch timeout via `react_loop.rs:310`, `phases/tools.rs:170`.
3. **Circuit breaker (wired but inert)**: constructed only via
   `builder.with_circuit_breaker` (`builder.rs:792,1011-1012` →
   `mod.rs:1366-1367`), default `None` (`builder.rs:172`, `mod.rs:521`). Its
   gate methods `try_advance`/`record_rejected`/`is_open`/`consecutive_failures`/
   `rejected_count` have **zero call sites** anywhere in either repository
   (V01-01). EKO never enables it.
4. **Tasks backoff (dormant)**: `TaskExecutorConfig::retry_delay_for_attempt`
   (`echo-orchestration/src/tasks/executor.rs:125-141`) consumed only by
   `run_task_with_retry` (`:1168,1175`), which is reached only when
   `TaskExecutor` is constructed — only in its own `#[cfg(test)]`
   (`executor.rs:1854-1855`; single external ref is the `mod.rs:37`
   re-export). Cancellation model there is exemplary
   (`select! { cancel.cancelled() / sleep }`, `:1204-1213`).
5. **Budget (live)**: `TokenBudget` built from config
   (`src/agent/react/mod.rs:340-346`) and consumed by compression prepare via
   `budget.allocate` (`echo-state/src/compression/mod.rs:474,1300,1311`).
6. **Utils (live)**: `fnv1a_64` (content dedup), `now_secs/now_millis`
   (audit/display), `local_rfc3339` serde (EKO timestamps), `extract_json_from_markdown`/`clean_json` (LLM tool-output parsing).

## Findings

### F-REL-01-P1-01: Circuit breaker gate is never called — the breaker is passive telemetry in the live LLM path

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: gate API `try_advance` at
  `echo-core/src/circuit_breaker.rs:106-140`; recording API
  `record_success`/`record_failure` at `:155-239`; the **only** production
  callers are `src/agent/react/run/retry.rs:61,63` (inside `retry_llm_call`);
  zero references to `try_advance`/`record_rejected`/`is_open`/
  `consecutive_failures`/`rejected_count` outside `circuit_breaker.rs` itself
  (grep, V01-01); default disabled `builder.rs:172`, `mod.rs:521`; EKO never
  constructs it (V01-01).
- Reachability: `retry_llm_call` is invoked on every live engine LLM call
  (`react_loop.rs:81,124`, `think.rs:289,354`). When a consumer enables the
  breaker, `record_*` runs but the Open state never rejects a request and the
  HalfOpen probe path is unreachable — the state machine only accumulates
  bookkeeping.
- Expected invariant (module doc, `circuit_breaker.rs:1-18`): "Prevents the
  Agent from entering a futile retry loop when the LLM service is persistently
  unavailable"; Open = reject all requests; HalfOpen = limited probes.
- Observed behavior: no code path consults the breaker before or during a
  retry; on a persistent outage each engine call still runs its full
  `max_retries` backoff, and every provider call inside additionally runs its
  own `with_retry_if` — up to (3+1)×(3+1) = 16 HTTP attempts per logical LLM
  call.
- Impact: the documented recovery capability is non-functional (major
  capability failure); futile retries continue against a down provider (added
  latency and provider cost); `state_name`/logging implies protection that
  does not exist, misleading operators.
- Root cause: the breaker was wired for recording only — the gating call
  (`try_advance` before each attempt, `record_rejected` on rejection) was
  never added to `retry_llm_call`; the doc header of `run/retry.rs:9` mentions
  "circuit breaker update" (recording) but not gating.
- Direction: wire the gate into the unified retry loop — call `try_advance()`
  before each attempt; on rejection either fail fast or park until timeout,
  and call `record_rejected`; then delete the now-dead alternative. If the
  product decision is telemetry-only, instead delete `try_advance`,
  `record_rejected`, `probes_in_flight`, and the HalfOpen machinery and
  re-document the breaker as a failure-rate monitor (AGENTS.md code-cleanup:
  no dual systems). Do both only after the P2-01 unification decision.
- Regression validation: unit test where breaker is Open: `retry_llm_call`
  performs zero `call_fn` invocations; HalfOpen probe admission counts exactly
  one concurrent probe; after a dropped probe future the breaker recovers
  (guards P3-02).
- Validation reports: [V01-01](../validations/F-REL-01/V01-01.md),
  [V02-01](../validations/F-REL-01/V02-01.md), [V03-01](../validations/F-REL-01/V03-01.md)

### F-REL-01-P2-01: Three parallel retry/backoff authorities with divergent math and safety properties

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: (1) `echo-core/src/retry.rs:82-101` — saturating arithmetic,
  exponent cap 10, max-delay cap, `rand` jitter; live only in provider clients
  (`echo-integration/src/providers/client.rs:122,203`, `anthropic.rs:374,431`)
  and demo44. (2) `src/agent/react/run/retry.rs:28-68` — plain `*`/`+`
  (`:31,33,87`), exponent cap 5, `fastrand` jitter, **no duration cap**;
  live in the engine LLM path (`react_loop.rs:81,124`, `think.rs:289,354`).
  (3) `echo-orchestration/src/tasks/executor.rs:125-141` — f64 `powi` +
  `Duration::from_secs_f64` (panics on negative factor); dormant (test-only
  construction, V01-01). Channel reconnects
  (`feishu/long_poll.rs:137-189`, `qq/channel.rs:137-188`) are a fourth
  hand-rolled backoff but a different semantic (reconnect loops) — recorded,
  not merged into this finding. `echo_core::retry`'s own doc
  (`retry.rs:3-4`): "for unified use by all external calls: LLM / MCP / A2A /
  Sandbox, etc."
- Reachability: both authorities #1 and #2 are live on the same LLM call stack
  (provider clients are invoked through the engine's `llm_client`), so the
  same logical call is retried twice with two policies.
- Expected invariant: one authoritative retry policy implementation; consumers
  choose policy *values*, not policy *machinery* (AGENTS.md "严禁平行实现同一
  语义"; F-REL-01 card: "确认只有 echo_core 一套权威实现").
- Observed behavior: three independent backoff implementations with different
  caps (10 vs 5 vs unbounded-f64), different RNGs (`rand` vs `fastrand`),
  different overflow behavior (saturating vs plain vs f64) and different
  cancellation handling (echo-orchestration selects on token; the other two
  are drop-only).
- Impact: fixes and safety properties do not propagate across implementations
  (e.g., `RetryPolicy` is overflow-safe, `retry_llm_call` is not — P3-01);
  stacked engine×provider retries amplify outage latency/cost; future policy
  changes (e.g., per-provider max delays) must be made three times.
- Root cause: `echo_core::retry` was introduced as the unified policy while
  the pre-existing engine loop `retry_llm_call` was never migrated; the
  tasks-domain backoff predates or paralleled both.
- Direction: migrate `retry_llm_call` onto `with_retry_if` + `RetryPolicy`
  (keeping `is_retryable_llm_error` as the retryable closure and the breaker
  recording hook — or moving recording into the unified helper); delete the
  backoff math in `src/agent/react/run/retry.rs` (keep
  `compute_concurrent_tool_batch_timeout` — it is tool-timeout math, not
  retry policy); for echo-orchestration: if `TaskExecutor` stays dormant,
  keep it as framework API but document the policy as domain-specific, or
  reuse `RetryPolicy::delay_for` internally; do not delete without the
  framework-wide dead-code gate (AGENTS.md).
- Regression validation: after unification, engine LLM tests
  (`run/retry.rs::tests`, V04-02) plus a test asserting max-delay cap and
  no-overflow on both the engine and provider paths; demo33/demo44 still
  compile and run.
- Validation reports: [V01-01](../validations/F-REL-01/V01-01.md),
  [V02-01](../validations/F-REL-01/V02-01.md)

### F-REL-01-P3-01: Plain `*`/`+` arithmetic in engine retry math and budget accounting

- Priority: P3
- Confidence: high (code fact); low (practical reachability)
- Layer: framework
- Evidence: `src/agent/react/run/retry.rs:31` `retry_delay_ms * (1u64 <<
  (attempt-1).min(5))`, `:33` `base_delay + jitter`, `:87`
  `.map(...retry_delay_ms * ...).sum::<u64>()`; `echo-core/src/budget.rs:106`
  `system_size + tool_defs_size + conversation_size`.
- Reachability: config-driven inputs — `llm_retry_delay_ms` (default 500,
  `src/agent/config.rs:238`) and `ToolExecutionConfig.retry_delay_ms` set by
  consumers; budget sizes are tokenizer estimates at
  `echo-state/src/compression/mod.rs:474,1300,1311`. Overflow needs
  `retry_delay_ms > u64::MAX/32` (~18 billion years) or token counts summing
  past `usize::MAX` — no current caller reaches it; in debug builds it would
  panic, in release it silently wraps (wrong delays/usage percentages).
- Expected invariant: AGENTS.md overflow rule — checked/saturating arithmetic
  on all integer operations.
- Observed behavior: plain arithmetic, protected only by the low default
  values.
- Impact: theoretical debug panic / silent misbehavior under pathological
  configuration; divergence from the safe `RetryPolicy` (P2-01) shows the fix
  surface.
- Root cause: engine retry written before `echo_core::retry` existed; budget
  accounting written with the assumption of realistic token counts.
- Direction: `saturating_mul`/`saturating_add`/`saturating_sum` at the four
  sites (they disappear entirely if P2-01 unification lands).
- Regression validation: unit tests with `retry_delay_ms = u64::MAX/2` and
  `allocate(usize::MAX, usize::MAX, usize::MAX)` asserting no panic and sane
  outputs.
- Validation reports: [V02-01](../validations/F-REL-01/V02-01.md),
  [V04-02](../validations/F-REL-01/V04-02.md)

### F-REL-01-P3-02: Circuit breaker probe slot is released only by explicit `record_*` — cancellation leaks it; `record_failure` while Open resets the open timer

- Priority: P3 (latent — gate currently never called; becomes P1/P2 the moment
  P1-01 is wired)
- Confidence: high
- Layer: framework
- Evidence: slot take `circuit_breaker.rs:110-120` (mutex-serialized, no
  double-take race); slot release only inside `record_success`/`record_failure`
  (`:157-163,198-204`) — no RAII/drop guard; Open→Open `record_failure`
  replaces `opened_at` with `Instant::now()` (`:233-237`).
- Reachability: latent today — `try_advance` has zero callers (P1-01). If the
  gate is wired as designed: a half-open probe task cancelled mid-flight
  (run cancellation drops the LLM call future) never calls `record_*`, leaving
  `probes_in_flight = 1` forever → every subsequent `try_advance` rejects →
  the breaker is permanently wedged (all LLM calls rejected) until restart.
  The Open-timer reset means each failure recorded while Open extends the open
  window indefinitely, so a noisy caller can prevent recovery.
- Expected invariant: probe admission recovers automatically after the probe
  completes, fails, or is cancelled; the open window is time-bounded.
- Observed behavior: slot persists across cancellation; Open duration is
  extensible by repeated `record_failure`.
- Impact: when the breaker is enabled and gated (post-fix), one cancelled
  probe kills the LLM path permanently; today only misleading state.
- Root cause: resource lifecycle is caller-convention-based instead of
  RAII/scope-based; Open-state recording conflates "request failed" with
  "trip timer refresh".
- Direction: replace the atomic slot with a scoped guard returned by
  `try_advance` that releases on drop (including cancellation); ignore or
  coalesce `record_failure` in Open state (do not refresh `opened_at`).
- Regression validation: test that drops the probe future mid-flight and
  asserts `try_advance` admits a new probe after the timeout; test that N
  `record_failure` calls while Open do not extend the window past the first
  timeout.
- Validation reports: [V02-01](../validations/F-REL-01/V02-01.md),
  [V03-01](../validations/F-REL-01/V03-01.md)

### F-REL-01-P3-03: `retry_delay_for_attempt` can panic via `Duration::from_secs_f64` on malformed backoff config

- Priority: P3
- Confidence: high (behavior); low (reachability)
- Layer: framework
- Evidence: `echo-orchestration/src/tasks/executor.rs:125-141` —
  `retry_backoff_factor.powi((attempt as i32).saturating_sub(1))`; with
  `retry_backoff_factor < 0` (e.g. -2.0) and `attempt ≥ 2` the delay is
  negative, `delay.min(max)` keeps it negative, and
  `Duration::from_secs_f64(negative)` panics (std contract: panics on
  negative/NaN/overflow); oversized `retry_max_delay_secs` panics the same
  way. Fields are pub config (`:72-79`), default factor 2.0, max 60s.
- Reachability: `TaskExecutor` has no production constructor in either
  repository (V01-01) — reachable today only from framework consumers that
  construct it directly with malformed config.
- Expected invariant: malformed configuration must not panic (AGENTS.md).
- Observed behavior: unvalidated pub config flows into a panicking std API.
- Impact: consumer with a negative-factor config gets a panic instead of an
  error; dormant in-repo today.
- Root cause: config trust boundary not enforced at the config type or at the
  conversion site.
- Direction: validate/clamp in `retry_delay_for_attempt` (factor ≥ 0, max
  delay within Duration range) or in config construction, and switch to
  checked conversion returning `Result`/clamped value.
- Regression validation: unit test with `retry_backoff_factor = -2.0` and
  `retry_max_delay_secs = u64::MAX` asserting clamped values instead of a
  panic.
- Validation reports: [V02-01](../validations/F-REL-01/V02-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search (retry/backoff/budget/circuit_breaker) | yes | passed | [V01-01](../validations/F-REL-01/V01-01.md) |
| V02 | Transition and arithmetic tables (overflow audit) | yes | passed | [V02-01](../validations/F-REL-01/V02-01.md) |
| V03 | Cancellation/time edge cases (sleep primitive, token, clock) | yes | passed | [V03-01](../validations/F-REL-01/V03-01.md) |
| V04 | `cargo test -p echo_core --lib --locked -- retry budget circuit_breaker utils` | yes | passed (exit 0, 43 passed) | [V04-01](../validations/F-REL-01/V04-01.md) |
| V04 | `cargo test -p echo_agent --lib --locked retry` (engine retry path) | conditional | passed (exit 0, 3 passed) | [V04-02](../validations/F-REL-01/V04-02.md) |
| V05 | Historical-document drift check | not applicable | - | - |

V05 is not applicable as a separate execution: historical claims relevant to
this task are classified in the next section from V01/V02 evidence; the
F-CORE-01 report carries no retry/budget claims.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `echo_core::retry` doc: "unified use by all external calls: LLM / MCP / A2A / Sandbox" | regressed (engine LLM path bypasses it — `retry_llm_call`) | [V01-01](../validations/F-REL-01/V01-01.md), finding P2-01 |
| AGENTS.md: "依赖 DAG、重试、取消…跨产品成立的机制放框架" | current (retry primitives are in `echo_core`; but duplicated in root engine — P2-01) | [V01-01](../validations/F-REL-01/V01-01.md) |
| MASTER-PLAN M4: "有限重试、幂等键、postcondition…已接通" | current for EKO-side recovery contract (F-EXT-01 scope); framework retry unification claim not made there | [V01-01](../validations/F-REL-01/V01-01.md) |
| MASTER-PLAN M13 Phase 3: "per-task retry/timeout 保留为单任务 pipeline" | current-but-dormant: `TaskExecutor` retry pipeline exists with zero production constructors | [V01-01](../validations/F-REL-01/V01-01.md), finding P2-01 |
| Circuit breaker module doc: "Prevents the Agent from entering a futile retry loop… Open: Reject all requests" | regressed (gate never called; breaker is recording-only) | [V01-01](../validations/F-REL-01/V01-01.md), finding P1-01 |

## Coverage And Uncertainty

- Not inspected: `echo-integration/src/providers/anthropic.rs:370-435` retry
  context in full (classification closure read; policy call sites verified),
  EKO TaskRuntime retry logic (application layer, A-series), F-EXT-01 tool
  retry contract.
- `Duration::from_secs_f64` and `random_range` behavior claims are from std /
  rand documented contracts, not executed here.
- Whether `TaskExecutor`'s dormancy is intended (framework capability menu) is
  a product decision for X-BND-01 / roadmap; this report only records
  reachability.
- No dynamic test exercises overflow boundaries, cancellation-during-probe, or
  negative-factor config; those gaps are captured as regression validations on
  the findings.
- A parallel reviewer may have compiled the workspace concurrently; all V04
  runs completed with exit 0 under the shared file lock.

## Handoff

- Downstream tasks may rely on: retry authority map (V01-01 — 3
  implementations, 2 live), breaker gate deadness (P1-01), overflow/panic
  audit table (V02-01), cancellation inventory (V03-01), test green state
  (V04-01/02).
- `F-RCT-02/03` (ReAct loop) should treat P1-01 + P2-01 as part of the LLM
  call-path design when touching `react_loop.rs`/`think.rs`.
- `F-EXT-01` (tool failure contract) should coordinate on the unified retry
  direction (P2-01) so tool-level retry semantics keep one authority.
- `X-BND-01` (authority map) should record: breaker gate wiring or
  telemetry-only decision; `TaskExecutor` dormancy; the three-way retry
  unification target.
- This report becomes stale if the engine retry path, `circuit_breaker.rs`
  gate wiring, or `echo_core::retry` API changes.
