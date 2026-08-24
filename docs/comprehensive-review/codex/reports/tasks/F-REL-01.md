# F-REL-01: Retry, budgets, circuit breakers, and utility invariants

> Status: complete
> Reviewer: Codex primary reviewer, with isolated subagent evidence
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d35c9a432370b923acabfbb5f32d63`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: both source repositories clean at final inspection; review reports and fixture sources are outside both source repositories

## Question

Are generic retry/backoff, budget arithmetic, circuit breaker, hashing, time,
and JSON parsing primitives deterministic and safe under overflow,
cancellation, clock, and malformed-input edges?

## Scope

- `echo-agent/echo-core/src/retry.rs`, `budget.rs`,
  `circuit_breaker.rs`, and `utils/{hash,json_parse,time}.rs`.
- Root retry facade and live ReAct retry/timeout integration.
- OpenAI-compatible and Anthropic provider retry edges required to establish
  nested ownership.
- `echo_execution::ToolManager` retry arithmetic required to compare generic
  retry duplication and batch timeout assumptions.
- Framework/EKO callers only where needed to prove current reachability.
- Current transitions, overflow behavior, cancellation, malformed JSON, local
  time conversion, deterministic hash vectors, and historical drift.

## Out Of Scope

- Source fixes, index edits, commits, or a full workspace submission gate.
- Security research or internet access; this task performed neither.
- Transport-specific reconnect policy (`F-INT-01`), complete provider semantics
  (`F-LLM-01`, `F-LLM-02`, `F-LLM-03`), task retry state machines
  (`F-TSK-01`, `F-TSK-02`, `F-TSK-03`), and full context selection
  (`F-CTX-01`) except direct contact with reviewed primitives.
- Cost accounting and EKO product budgets.
- Persistence of circuit-breaker state across process restart.
- Conclusions or evidence from any other reviewer. One accidental search output
  breached directory isolation; its content is quarantined in V07-01 and not
  used. This makes primary reconstruction mandatory.

## Inputs

- Root `AGENTS.md`.
- Shared `README.md`, `REPORTING.md`, and the `F-REL-01` card in `TASKS.md`.
- Codex reviewer protocol `codex/README.md`.
- Dependency report [F-CORE-01](F-CORE-01.md), limited to the accepted no-panic,
  typed-failure, time/recovery-authority, and independent-consumer constraints.
- Project-owned `docs/PROJECT-ANALYSIS.md`, English/Chinese config references,
  and relevant `echo-agent` file history, treated as hypotheses.
- No external web source was used, as explicitly required for this continuation.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Validated retry policy, checked backoff/budget arithmetic, cancellation-aware sleeps, circuit permits, deterministic hash, clock conversion, and syntax-aware JSON decoding are reusable framework concerns. |
| EKO product policy | EKO chooses concrete retry counts, context percentages, whether to enable the breaker, and how failures render. It must not own a second generic retry loop. |
| Adapter boundary | Provider adapters classify typed transport errors and perform wire conversion. The selected logical retry owner must receive cancellation and breaker state; an adapter must not silently install an additional default retry budget. |
| Duplicate search | Searches covered types, constructors, configuration fields, helper names, exponential arithmetic, definitions, re-exports, all callers, and production/test distinction across both repositories. |
| Migration deletion | After the LLM contract selects one logical retry owner, delete the other ReAct/provider retry loop and its backoff arithmetic. Replace duplicated time helpers in crates already depending on `echo_core`; delete the ad hoc JSON repair if no syntax-aware replacement owns recovery. |

The root `src/retry.rs` facade is thin and lossless. The split occurs below it:
ReAct, provider adapters, tool execution, and task orchestration each own retry
arithmetic. Domain-specific terminal policy may remain separate, but generic
attempt/backoff/cancel math should not.

## Current Path

```text
ReactAgent invocation
  -> retry_llm_call (outer: AgentConfig max/delay, random jitter)
       -> optional CircuitBreaker record only (no try_advance gate)
       -> LlmClient::chat / chat_stream
            -> OpenAI/Anthropic HTTP helper
                 -> with_retry_if(RetryPolicy::default) (inner)

CancellationToken
  -> ReactAgent run snapshot / streaming ChatRequest
  -X-> retry_llm_call sleep
  -X-> provider with_retry_if sleep and HTTP setup

TokenBudgetConfig
  -> ReactAgent::new_inner -> TokenBudget
  -> ContextManager::should_compress / prepare
       -> TokenBudget::allocate(0, 0, estimated_tokens)

Tool batch
  -> compute_concurrent_tool_batch_timeout (unchecked retry-delay sum)
  -> ToolManager retry loop (separate capped delay helper)

LLM critic parse
  -> serde_json direct parse
  -> markdown extraction + serde_json
  -> clean_json global replacements + serde_json
  -> default non-passing critique
```

With defaults, one ReAct call can make four outer attempts. The inspected
OpenAI-compatible/Anthropic provider adapter can make four HTTP attempts inside
each outer attempt, yielding up to 16 transport attempts before one logical call
terminates. The public live fixture also proves an Open breaker still allows a
second LLM client call and that cancellation during outer backoff does not stop
the second attempt.

Time/hash are positive exceptions: `to_local` consults the zone for the supplied
instant, and FNV-1a uses deliberate wrapping. The New York winter/summer fixture
and known FNV vector pass.

## Findings

### F-REL-01-P1-01: The configured circuit breaker never breaks a production LLM call

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/circuit_breaker.rs:95`,
  `echo-agent/src/agent/react/run/retry.rs:58`,
  `echo-agent/src/agent/react/builder.rs:789`,
  `echo-agent/src/agent/react/mod.rs:1362`
- Reachability: `ReactAgentBuilder::with_circuit_breaker` installs the public
  breaker; every live ReAct LLM path calls `retry_llm_call`; repository-wide
  search finds production calls only to `record_success`/`record_failure`, not
  `try_advance` or `record_rejected`. A public Agent fixture opens at threshold
  one and the second call still increments the mock LLM count.
- Expected invariant: Open rejects before external I/O, records the rejection,
  and transitions to one half-open probe after timeout.
- Observed behavior: failures change internal state to Open, but no caller asks
  the state whether to proceed. Every later call continues to reach the LLM and
  can reset `opened_at` on failure.
- Impact: the advertised resilience control provides no outage isolation;
  persistent provider failure continues consuming attempts, latency, and API
  quota despite the breaker reporting Open.
- Root cause: state recording and admission control were integrated separately;
  only the post-call half was wired into ReAct.
- Direction: make one cancellation-aware LLM operation guard acquire a breaker
  permit before its first transport attempt and finalize it exactly once. Record
  rejected calls there. Delete post-hoc breaker updates from the old outer loop
  when that owner is replaced; do not add a second gate in provider adapters.
- Regression validation: a threshold-one real Agent fixture must call the mock
  once, reject the second call with a typed circuit-open error, permit exactly
  one probe after paused-time advance, and emit rejection/transition facts.
- Validation reports: [V02-01](../validations/F-REL-01/V02-01.md),
  [V05-01](../validations/F-REL-01/V05-01.md),
  [V06-06](../validations/F-REL-01/V06-06.md),
  [V20-02](../validations/F-REL-01/V20-02.md),
  [V20-13](../validations/F-REL-01/V20-13.md)

### F-REL-01-P1-02: Cancellation during LLM backoff still starts another provider attempt

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/retry.rs:133`,
  `echo-agent/src/agent/react/run/retry.rs:28`,
  `echo-agent/src/agent/react/run/react_loop.rs:52`,
  `echo-agent/src/agent/react/run/phases/think.rs:321`,
  `echo-agent/echo-integration/src/providers/client.rs:203`
- Reachability: streaming Agent entry points store a real token; ReAct passes it
  into streaming `ChatRequest`, but outer retry sleeps independently. The live
  public fixture cancels 20 ms into a 250-ms backoff after a 429; mock call count
  becomes two and the late successful response is consumed.
- Expected invariant: after cooperative cancellation is observed, no new
  external attempt starts and every pending backoff/request setup terminates
  promptly.
- Observed behavior: outer and core retry helpers use plain
  `tokio::time::sleep`. Provider setup retries also lack a cancellation branch.
  Non-streaming ReAct requests explicitly set `cancel_token: None`.
- Impact: stop/cancel can still issue an LLM request and wait through user-sized
  backoff. With nested retries it may consume quota and return work after the
  user has ended the run.
- Root cause: cancellation is carried by the streaming payload rather than the
  logical retry operation that owns attempt admission and delay.
- Direction: make cancellation a required input to the selected LLM retry owner
  and race it against every delay and attempt setup. Return the existing typed
  cancellation terminal. Delete token-less retry wrappers on live Agent paths;
  keep an explicitly non-cancellable convenience wrapper only for independent
  callers that choose that contract.
- Regression validation: paused-time tests cancel during outer delay, inner
  delay, and HTTP setup; assert no next attempt, bounded completion, one typed
  Cancelled terminal, and unchanged call count.
- Validation reports: [V03-02](../validations/F-REL-01/V03-02.md),
  [V06-02](../validations/F-REL-01/V06-02.md),
  [V06-15](../validations/F-REL-01/V06-15.md),
  [V20-06](../validations/F-REL-01/V20-06.md),
  [V20-12](../validations/F-REL-01/V20-12.md)

### F-REL-01-P2-03: One logical LLM call has two independent retry budgets

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/retry.rs:1`,
  `echo-agent/src/agent/react/run/retry.rs:9`,
  `echo-agent/echo-integration/src/providers/client.rs:122`,
  `echo-agent/echo-integration/src/providers/client.rs:203`,
  `echo-agent/echo-integration/src/providers/openai.rs:302`
- Reachability: all ReAct LLM calls use the outer helper; inspected production
  OpenAI-compatible and Anthropic adapters use core retry around HTTP setup.
- Expected invariant: one logical operation has one explicit retry budget and
  one observable attempt sequence.
- Observed behavior: default outer `max_retries=3` wraps an inner default
  `max_retries=3`, so up to 16 HTTP attempts occur. The two layers use different
  caps/jitter and independently sleep/log, while Agent configuration controls
  only the outer layer.
- Impact: user-visible retry settings do not bound requests or latency;
  telemetry cannot describe one authoritative attempt number, and cancellation/
  breaker policy cannot reliably govern the hidden inner attempts.
- Root cause: the later generic `RetryPolicy` was added without migrating the
  pre-existing ReAct retry loop or defining whether `LlmClient` is single-attempt.
- Direction: define `LlmClient` adapters as single transport attempts and place
  the logical retry/cancel/breaker composite at the reusable Agent operation
  boundary, or select the inverse contract explicitly. In either case delete
  one loop, inject policy instead of hard-coding `RetryPolicy::default`, and keep
  provider adapters limited to typed retryability facts.
- Regression validation: with both outer and provider settings at three, one
  logical fixture must prove the selected total is four, not sixteen, with one
  monotonic attempt counter and deterministic paused-time schedule.
- Validation reports: [V01-01](../validations/F-REL-01/V01-01.md),
  [V02-01](../validations/F-REL-01/V02-01.md),
  [V05-01](../validations/F-REL-01/V05-01.md),
  [V20-07](../validations/F-REL-01/V20-07.md)

### F-REL-01-P2-04: TokenBudget accepts impossible allocations and panics on public counts

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/budget.rs:43`,
  `echo-agent/echo-core/src/budget.rs:66`,
  `echo-agent/echo-core/src/budget.rs:81`,
  `echo-agent/echo-core/src/budget.rs:106`,
  `echo-agent/echo-state/src/compression/mod.rs:1299`
- Reachability: Agent construction builds enabled budget config and live context
  compression calls `allocate`; public framework consumers can construct the
  budget and pass provider/tokenizer-sized `usize` values directly.
- Expected invariant: percentages are finite and within `[0,1]`, their sum is
  at most one, every category is within `total_window`, and aggregate accounting
  cannot panic/wrap.
- Observed behavior: no validation exists. A negative system allocation creates
  a 150-token conversation category in a 100-token window. The public allocation
  `(usize::MAX,1,1)` panics in debug at its unchecked sum and may wrap when
  overflow checks are absent.
- Impact: malformed custom framework config can disable/retarget compression,
  and extreme external counts violate the repository's no-panic/accounting
  contract. Normal defaults remain correct.
- Root cause: raw public `f64` and `usize` fields are treated as already valid,
  and derived totals use unchecked arithmetic.
- Direction: replace raw allocation construction with a validated `Result`
  constructor/config build; reject non-finite, negative, over-one, and zero-window
  policies as appropriate. Use checked aggregate arithmetic and return typed
  overflow/configuration errors; delete the unchecked builder after callers move.
- Regression validation: table/property test zero, boundary one, sum over one,
  negative, NaN, infinity, maximum window/counts, and exact conservation without
  panic in debug/release semantics.
- Validation reports: [V02-02](../validations/F-REL-01/V02-02.md),
  [V03-01](../validations/F-REL-01/V03-01.md),
  [V04-02](../validations/F-REL-01/V04-02.md),
  [V06-01](../validations/F-REL-01/V06-01.md),
  [V20-04](../validations/F-REL-01/V20-04.md)

### F-REL-01-P2-05: Accepted retry values panic inside Agent tasks and surface as successful empty output

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/config.rs:624`,
  `echo-agent/src/agent/react/run/retry.rs:31`,
  `echo-agent/src/agent/react/run/retry.rs:85`,
  `echo-agent/echo-core/src/tools/mod.rs:525`,
  `echo-agent/src/agent/react/run/phases/tools.rs:170`
- Reachability: public Agent setters accept any `u64` LLM delay and public
  `ToolExecutionConfig` accepts maximum retry/delay values. Both arithmetic
  blocks execute on live ReAct LLM/tool paths before retry completion.
- Expected invariant: accepted numeric configuration is validated, checked, or
  saturated; internal failure cannot be normalized to success.
- Observed behavior: LLM jitter addition and tool-batch delay multiplication/sum
  panic in debug for maximum public values. In both public Agent fixtures the
  panic occurred in a spawned stream task, did not escape, and `Agent::chat`
  returned `Ok("")`.
- Impact: a malformed/extreme local configuration can convert execution failure
  into a successful empty answer, hiding the root cause and violating terminal
  semantics as well as the no-panic rule.
- Root cause: duplicated arithmetic has no validated bound, and the stream join/
  terminal boundary treats producer panic/closure as empty success.
- Direction: validate retry counts/delays at Agent/Tool configuration creation,
  reuse one checked delay primitive, and propagate spawned task join failure as
  a typed terminal error. Delete `compute_concurrent_tool_batch_timeout`'s
  parallel delay summation once it consumes the authoritative schedule.
- Regression validation: maximum and just-over-safe values must return a stable
  config/arithmetic error without panic; forced producer panic must yield one
  Error terminal and never `Ok("")`.
- Validation reports: [V03-01](../validations/F-REL-01/V03-01.md),
  [V06-11](../validations/F-REL-01/V06-11.md),
  [V06-16](../validations/F-REL-01/V06-16.md),
  [V20-14](../validations/F-REL-01/V20-14.md)

### F-REL-01-P2-06: A cancelled half-open probe permanently consumes the only probe slot

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/circuit_breaker.rs:100`,
  `echo-agent/echo-core/src/circuit_breaker.rs:118`,
  `echo-agent/echo-core/src/circuit_breaker.rs:132`,
  `echo-agent/echo-core/src/circuit_breaker.rs:154`,
  `echo-agent/echo-core/src/circuit_breaker.rs:195`
- Reachability: public consumers call `try_advance` then must remember exactly
  one record method. Current ReAct does not call `try_advance`, so this becomes
  live in that path when P1-01 is fixed unless the API changes simultaneously.
- Expected invariant: cancellation/drop of an admitted half-open request releases
  capacity and cannot strand recovery.
- Observed behavior: admission increments an atomic quota; only explicit
  `record_success`/`record_failure` decrements it. Omitting completion leaves
  state HalfOpen and all later requests rejected indefinitely.
- Impact: a single cancelled recovery probe can make an otherwise healthy remote
  service permanently unavailable until the breaker instance/process resets.
- Root cause: admission and completion are separate calls rather than one owned
  RAII permit/future lifecycle.
- Direction: return a probe permit whose `Drop` releases quota; require explicit
  success/failure finalization and define drop as cancellation/neutral recovery.
  Delete the bare bool admission contract once callers migrate.
- Regression validation: cancel/abort/drop half-open futures at every await,
  then assert exactly one next probe can proceed and counters/state stay bounded.
- Validation reports: [V03-01](../validations/F-REL-01/V03-01.md),
  [V04-03](../validations/F-REL-01/V04-03.md),
  [V06-03](../validations/F-REL-01/V06-03.md),
  [V20-03](../validations/F-REL-01/V20-03.md)

### F-REL-01-P2-07: JSON auto-fix silently rewrites quoted LLM content

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/utils/json_parse.rs:27`,
  `echo-agent/src/agent/critic/llm_critic.rs:85`
- Reachability: LLM critic directly parses, extracts markdown, then applies
  `clean_json`; repaired structured values become evaluation score/pass/feedback.
- Expected invariant: syntax recovery never treats bytes inside quoted strings
  as structural commas/braces and never corrupts an apostrophe in content.
- Observed behavior: global replacement changes `"keep ,} literal"` to
  `"keep } literal"` while making the outer malformed object parseable.
  Single-quoted `don't` becomes invalid `"don"t"`.
- Impact: malformed-output recovery can silently alter evaluation feedback and
  accept the mutated value as authoritative structured output instead of
  reporting parse failure.
- Root cause: JSON syntax is repaired by unstructured string replacement rather
  than tokenizer/parser-aware recovery.
- Direction: parse with a structured tolerant parser or implement a small
  quote/escape-aware lexer; otherwise remove semantic auto-fix and return the
  typed parse/fallback outcome with original text. Delete global replacement.
- Regression validation: property fixtures preserve every string value across
  repair, including escapes, apostrophes, `,}`, `,]`, Unicode, fenced JSON, and
  unrecoverable truncation.
- Validation reports: [V02-02](../validations/F-REL-01/V02-02.md),
  [V03-03](../validations/F-REL-01/V03-03.md),
  [V04-04](../validations/F-REL-01/V04-04.md),
  [V06-04](../validations/F-REL-01/V06-04.md),
  [V20-05](../validations/F-REL-01/V20-05.md)

### F-REL-01-P3-08: Shared epoch helpers still have six framework/application copies

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/utils/time.rs:6`,
  `echo-agent/echo-orchestration/src/tasks/time.rs:2`,
  `echo-agent/echo-state/src/memory/snapshot.rs:168`,
  `echo-agent/echo-state/src/memory/store.rs:510`,
  `echo-agent/echo-state/src/memory/sqlite_store.rs:947`,
  `echo-agent/src/evolution/dreaming.rs:219`,
  `echo-agent-cli/echo-agent-app-core/src/tool_execution.rs:603`
- Reachability: the copies timestamp task state, snapshots, memory TTL/access,
  evolution items, and EKO tool executions. Both framework subcrates already
  depend on `echo_core`.
- Expected invariant: the documented shared helper is the framework authority,
  so clock-failure and narrowing semantics do not drift.
- Observed behavior: six `now_secs` and two `now_millis` definitions remain.
  EKO's millisecond copy safely saturates `u128 -> u64`, while core uses `as u64`.
- Impact: low immediate risk, but future injectable-clock/error semantics require
  multiple migrations and already differ for theoretical narrowing overflow.
- Root cause: shared time helpers were added after local copies without completing
  call-site migration.
- Direction: replace framework copies with `echo_core::utils::time`; let EKO use
  the facade helper or a product clock adapter only if tests need injection.
  Delete local functions and duplicate tests after migration.
- Regression validation: repository-wide definition search returns only the
  canonical helper plus explicitly named test clocks; task/memory/EKO timestamp
  tests use the same error/narrowing contract.
- Validation reports: [V01-01](../validations/F-REL-01/V01-01.md),
  [V03-04](../validations/F-REL-01/V03-04.md),
  [V05-02](../validations/F-REL-01/V05-02.md),
  [V20-08](../validations/F-REL-01/V20-08.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition/duplicate inventory | yes | failed | [V01](../validations/F-REL-01/V01-01.md) |
| V02 | Circuit/retry runtime reachability | yes | failed | [V02-01](../validations/F-REL-01/V02-01.md) |
| V02 | Budget/utility runtime reachability | yes | passed | [V02-02](../validations/F-REL-01/V02-02.md) |
| V03 | Transition/arithmetic boundary table | yes | failed | [V03-01](../validations/F-REL-01/V03-01.md) |
| V03 | Cancellation trace | yes | failed | [V03-02](../validations/F-REL-01/V03-02.md) |
| V03 | Malformed JSON inspection | yes | failed | [V03-03](../validations/F-REL-01/V03-03.md) |
| V03 | Hash/time static invariants | yes | passed | [V03-04](../validations/F-REL-01/V03-04.md) |
| V04 | Core retry tests | yes | passed | [V04-01](../validations/F-REL-01/V04-01.md) |
| V04 | Core budget tests | yes | passed | [V04-02](../validations/F-REL-01/V04-02.md) |
| V04 | Core circuit tests | yes | passed | [V04-03](../validations/F-REL-01/V04-03.md) |
| V04 | Core utility tests | yes | passed | [V04-04](../validations/F-REL-01/V04-04.md) |
| V05 | Documentation/history drift | yes | failed | [V05-01](../validations/F-REL-01/V05-01.md) |
| V05 | Time-history hypothesis | yes | passed | [V05-02](../validations/F-REL-01/V05-02.md) |
| V06 | Budget overflow/invalid percentages | yes | failed | [V06-01](../validations/F-REL-01/V06-01.md) |
| V06 | Generic retry cancellation | yes | failed | [V06-02](../validations/F-REL-01/V06-02.md) |
| V06 | Half-open cancelled probe | yes | failed | [V06-03](../validations/F-REL-01/V06-03.md) |
| V06 | Malformed JSON fixtures | yes | failed | [V06-04](../validations/F-REL-01/V06-04.md) |
| V06 | Time DST and FNV known vector | yes | passed | [V06-05](../validations/F-REL-01/V06-05.md) |
| V06 | Live Agent breaker gate | yes | failed | [V06-06](../validations/F-REL-01/V06-06.md) |
| V06 | Live breaker first build attempt | yes | inconclusive | [V06-07](../validations/F-REL-01/V06-07.md) |
| V06 | Tool-overflow fixture bad import | yes | failed fixture | [V06-08](../validations/F-REL-01/V06-08.md) |
| V06 | Tool-overflow wrong unwind expectation | yes | failed fixture | [V06-09](../validations/F-REL-01/V06-09.md) |
| V06 | Tool-overflow wrong error expectation | yes | failed fixture | [V06-10](../validations/F-REL-01/V06-10.md) |
| V06 | Live tool timeout overflow | yes | failed | [V06-11](../validations/F-REL-01/V06-11.md) |
| V06 | Expanded edge consistency rerun | yes | passed | [V06-12](../validations/F-REL-01/V06-12.md) |
| V06 | Retry fixtures missing trait import | yes | failed fixture | [V06-13](../validations/F-REL-01/V06-13.md) |
| V06 | LLM overflow wrong unwind expectation | yes | failed fixture | [V06-14](../validations/F-REL-01/V06-14.md) |
| V06 | Live Agent retry cancellation | yes | failed | [V06-15](../validations/F-REL-01/V06-15.md) |
| V06 | Live LLM backoff overflow | yes | failed | [V06-16](../validations/F-REL-01/V06-16.md) |
| V06 | Second missing-trait compile attempt | yes | failed fixture | [V06-17](../validations/F-REL-01/V06-17.md) |
| V07 | Reviewer-directory isolation | yes | failed | [V07-01](../validations/F-REL-01/V07-01.md) |
| V08 | Final report/link/executor/source-state gate | yes | passed | [V08-03](../validations/F-REL-01/V08-03.md); failed attempts [V08-01](../validations/F-REL-01/V08-01.md), [V08-02](../validations/F-REL-01/V08-02.md) |
| V20 | Primary source/caller reconstruction | yes | passed | [V20-01](../validations/F-REL-01/V20-01.md) |
| V20 | Primary breaker admission source check | yes | failed invariant | [V20-02](../validations/F-REL-01/V20-02.md) |
| V20 | Primary half-open permit-drop probe | yes | failed invariant | [V20-03](../validations/F-REL-01/V20-03.md) |
| V20 | Primary budget boundary probe | yes | failed invariant | [V20-04](../validations/F-REL-01/V20-04.md) |
| V20 | Primary JSON preservation probe | yes | failed invariant | [V20-05](../validations/F-REL-01/V20-05.md) |
| V20 | Generic retry cancellation positive control | yes | passed | [V20-06](../validations/F-REL-01/V20-06.md) |
| V20 | Primary retry ownership reconstruction | yes | failed invariant | [V20-07](../validations/F-REL-01/V20-07.md) |
| V20 | Primary time/hash/core-retry positives | yes | passed | [V20-08](../validations/F-REL-01/V20-08.md) |
| V20 | Primary external-Agent fixture construction | yes | passed after corrections | [V20-12](../validations/F-REL-01/V20-12.md); immutable failed setup attempts [V20-09](../validations/F-REL-01/V20-09.md), [V20-10](../validations/F-REL-01/V20-10.md), [V20-11](../validations/F-REL-01/V20-11.md) |
| V20 | Primary live Agent breaker admission | yes | failed invariant | [V20-13](../validations/F-REL-01/V20-13.md) |
| V20 | Primary live Agent backoff overflow | yes | failed invariant | [V20-14](../validations/F-REL-01/V20-14.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| F-CORE-01: public helpers cannot panic/wrap and terminal failures must remain typed | current constraint, unmet | P2-04/P2-05; V06-01, V06-11, V06-16 |
| `echo_core::retry`: unified policy for external calls | regressed/unrealized | ReAct/provider/tool/task backoff copies; P2-03 and V01 |
| `docs/PROJECT-ANALYSIS.md:234`: `retry_llm_call` combines circuit breaker and retry | regressed | It records only; live gate fixture fails; P1-01 |
| Config references: breaker thresholds/defaults and token percentage fields | current API shape, incomplete invariant | Values are accurately listed but validation and runtime admission are absent. |
| Historical fixed local-offset concern | fixed/stale | Current `to_local` uses `chrono::Local` per instant; V05-02/V06-05 |
| FNV deterministic-content claim | current | Known vector passes; V03-04/V06-05 |

## Coverage And Uncertainty

- V07-01 records an isolation breach in the delegated pass. The primary reviewer
  therefore reconstructed every finding from source before opening the delegated
  report, used fresh core fixtures for all P2 boundaries, and independently
  reproduced both P1 findings plus the spawned-task panic through a new public
  Agent consumer (V20-01 through V20-14). The delegated evidence is retained for
  traceability but is not the sole basis for any accepted finding.
- Core targeted tests passed; no full workspace gate was run because source code
  was not changed.
- No network or external implementation research was performed. This report
  selects only framework/application ownership and a single-authority constraint;
  provider-contract implementation should be finalized in `F-LLM-01` before
  deleting a retry loop.
- Nested retry count is established for inspected OpenAI-compatible and Anthropic
  adapters, not every third-party `LlmClient`.
- The LLM/tool overflow fixtures use extreme but accepted public values. Their
  concrete public result is a silent empty success; they do not prove a process
  crash in the current spawned-stream path.
- `Duration::as_millis() as u64` in retry logging/jitter and core `now_millis`
  theoretically narrow after enormous durations/timestamps. They are retained
  as residual edge risk, not inflated into findings.
- Circuit `u32 + 1` counters can overflow only after billions of transitions;
  this review did not execute that many calls. Checked/saturating counters should
  be included when the primitive is revised.
- The temporary Cargo fixture is reproducible from source and lockfile. Its build
  output is cleaned after validation to avoid retaining gigabytes of artifacts.

## Handoff

- Primary reconstruction and acceptance are complete in V20-01 through V20-14.
- `F-LLM-01` should choose and document the one-attempt `LlmClient` contract
  before P2-03 is fixed; cancellation and breaker admission must span the chosen
  logical retry owner.
- `F-RCT-02`, `F-RCT-03`, and `F-RCT-04` may use P1-01/P1-02/P2-05 only after
  primary acceptance because they affect ReAct terminal and cancellation behavior.
- `F-CTX-01` should independently include P2-04 boundary tables while reviewing
  actual context composition; it must not create a second budget validator.
- `F-RCT-04` should verify the P2-05 timeout estimate against the authoritative
  tool retry schedule and preserve typed terminal failure.
- This report becomes stale when core retry/budget/circuit/utility files, ReAct
  retry integration, provider client retry, tool retry/timeout arithmetic, or
  critic parsing changes.
