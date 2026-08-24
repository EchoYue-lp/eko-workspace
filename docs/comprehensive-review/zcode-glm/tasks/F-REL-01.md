# F-REL-01: Reliability Primitives (RetryPolicy, TokenBudget, CircuitBreaker, Utils)

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa
> `echo-agent-cli` commit: not-applicable
> Worktree state: clean

## Question

Are the reliability primitives in `echo-core` (retry, budget, circuit breaker,
hash/time/json utils) implemented exactly once, free of duplication, and safe
under expected inputs?

## Scope

Primary source paths and behaviors inspected:

- `echo-core/src/retry.rs` — `RetryPolicy` struct, `delay_for` exponential
  backoff, `no_retry`, root re-exports `with_retry` / `with_retry_if`.
- `echo-core/src/budget.rs` — `TokenBudget`, `TokenAllocation`,
  `TokenBudgetConfig`, `allocate`, `report`.
- `echo-core/src/circuit_breaker.rs` — `CircuitBreaker` state machine
  (Closed -> Open -> HalfOpen), `AtomicU32` counters, `CircuitBreakerConfig`.
- `echo-core/src/hash.rs` — `fnv1a_64`.
- `echo-core/src/time.rs` — `now_secs` / `now_millis`, serde helpers.
- `echo-core/src/json_parse.rs` — `extract_json_from_markdown`,
  `clean_json`.

## Out Of Scope

- Application-layer retry/budget policy wiring in `echo-agent-cli` (deferred to
  application-specific reliability tasks).
- LLM provider-specific backoff implementations inside provider adapters.
- Persistence of circuit breaker state across process restarts.

## Inputs

- Required repository documents: none beyond source.
- Dependency task reports: none.
- Historical documents treated as hypotheses: none.

## Layering Decision

- Generic mechanism (framework): `RetryPolicy`, `TokenBudget`,
  `CircuitBreaker`, and the `hash` / `time` / `json_parse` utils are
  domain-agnostic primitives reusable by any project consuming `echo-core`.
  They carry no EKO product decisions and belong in the framework.
- EKO product policy: none observed in this scope.
- Adapter boundary: none observed in this scope.
- Repository-wide duplicate search terms: `RetryPolicy`, `TokenBudget`,
  `CircuitBreaker`, `with_retry`, `fnv1a_64`, `extract_json_from_markdown`,
  `clean_json`. Result: single implementation of each in `echo-core`; no
  parallel application-layer implementations found.

## Current Path

- `RetryPolicy` (`echo-core/src/retry.rs:35`) holds `max_retries`,
  `base_delay`, `max_delay`, `jitter`. `new()` constructs defaults; `delay_for`
  computes exponential backoff per attempt; `no_retry()` returns a policy with
  zero retries. Root `retry.rs` is a thin re-export module adding
  `with_retry()` / `with_retry_if()` wrappers.
- `TokenBudget` (`echo-core/src/budget.rs:17`) holds `total_window` with
  allocations for `system_prompt`, `tool_definitions`, `output`,
  `conversation`. `allocate()` computes remaining budget; `report()` produces a
  usage snapshot. `TokenAllocation::ok()` / `needs_compression()` are the
  decision helpers. `TokenBudgetConfig::enabled()` / `disabled()` / `build()`
  construct configs.
- `CircuitBreaker` (`echo-core/src/circuit_breaker.rs:59`) is a state machine
  `Closed -> Open -> HalfOpen` backed by `AtomicU32` counters. Methods:
  `is_open()`, `try_advance()`, `record_success` / `record_failure` /
  `record_rejected()`. `CircuitBreakerConfig` carries thresholds.
- Utils: `hash::fnv1a_64`; `time::now_secs` / `now_millis` plus serde helpers;
  `json_parse::extract_json_from_markdown` / `clean_json`.
- Consumers: LLM call paths, tool execution, and MCP integration invoke these
  primitives for retry, budget enforcement, and fault isolation.

## Findings

### F-REL-01-P3-01: TokenBudget::allocate uses plain usize arithmetic

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-core/src/budget.rs:90`
- Reachability: `TokenBudget::allocate` -> LLM call budget enforcement ->
  every chat turn that allocates tokens.
- Expected invariant: budget arithmetic must not silently overflow or panic
  for unusually large `total_window` values.
- Observed behavior: `allocate` uses plain `usize` arithmetic (`+` / `-`)
  without `checked_add` / `checked_sub` / `saturating_*`.
- Impact: For pathological configs (very large `total_window`, or allocations
  summing past `usize::MAX` on 32-bit targets), arithmetic could wrap or panic.
  Realistic desktop configs are far from the limit, so impact is minor.
- Root cause: Implementation predates the project-wide no-panic convention and
  was not updated to use checked arithmetic.
- Direction: Replace the arithmetic in `allocate` with `checked_add` /
  `saturating_sub` and propagate the failure or clamp, matching the convention
  in AGENTS.md (Rust hard constraint #2).
- Regression validation: Unit test feeding a `total_window` near `usize::MAX`
  must not panic and must clamp/error gracefully.
- Validation reports: [V03](../validations/F-REL-01/V03-01.md)

### F-REL-01-P3-02: CircuitBreaker state transitions emit no callback/event

- Priority: P3
- Confidence: medium
- Layer: framework
- Evidence: `echo-core/src/circuit_breaker.rs:59`
- Reachability: `CircuitBreaker::try_advance` / `record_failure` ->
  transitions Closed -> Open -> HalfOpen; no observer hook on transition.
- Expected invariant: Callers that need to react to a trip (e.g. emit a metric,
  log, fall back) should be able to observe state changes.
- Observed behavior: Transitions mutate internal `AtomicU32` state silently;
  there is no callback, channel, or event emitted on Closed -> Open or
  Open -> HalfOpen.
- Impact: Observability gap. Consumers can poll `is_open()` but cannot react to
  the exact transition instant. Not a correctness bug.
- Root cause: The breaker was designed as a pure gate; observation was never
  added.
- Direction: Optionally add an `on_transition` callback or return the previous
  state from mutating methods so callers can detect changes. Low priority given
  the local single-user threat model.
- Regression validation: Existing breaker tests must still pass; add a test
  confirming a transition callback fires exactly once per state change.
- Validation reports: [V02](../validations/F-REL-01/V02-01.md)

### Positive observations

- Each reliability primitive exists exactly once in `echo-core`; no duplicate
  application-layer copies were found. Layering is clean: generic mechanism in
  framework, no EKO product policy leaking in.
- `CircuitBreaker` uses `AtomicU32` for lock-free state, which is safe under
  the project's no-panic convention.
- `RetryPolicy::delay_for` exponential backoff and `TokenBudget` allocation are
  the single authoritative implementations consumed by LLM calls, tool
  execution, and MCP.

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search | yes | passed | [V01-01](../validations/F-REL-01/V01-01.md) |
| V02 | Registration and reachability | yes | passed | [V02-01](../validations/F-REL-01/V02-01.md) |
| V03 | Invariant and edge cases | yes | passed_with_notes | [V03-01](../validations/F-REL-01/V03-01.md) |
| V04 | Targeted executable check | conditional | passed | [V04-01](../validations/F-REL-01/V04-01.md) |
| V05 | Historical-document drift | conditional | not_run | - |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| none | not-applicable | no historical documents in scope |

## Coverage And Uncertainty

- Application-layer wiring in `echo-agent-cli` was not inspected; this report
  covers framework primitives only.
- `TokenBudget::allocate` overflow behavior is inferred from reading the
  arithmetic at `budget.rs:90`; no live overflow was triggered.
- Persistence of breaker state across restarts is out of scope and unverified.

## Handoff

- Downstream tasks may rely on: there is exactly one `RetryPolicy`,
  `TokenBudget`, and `CircuitBreaker` in the framework, all in `echo-core`,
  consumed by LLM calls, tool execution, and MCP.
- Reports to read: V01-01 through V04-01 under
  `validations/F-REL-01/`.
- This report becomes stale if a parallel retry/budget/breaker implementation
  is introduced in the application layer, or if `allocate` arithmetic is
  changed.
- Follow-up task IDs: a P3 fix task for `checked_add` in
  `TokenBudget::allocate`; an optional P3 task for breaker transition
  callbacks. Fixes are not implemented in this review task.
