# F-TST-01: Framework test and mock utilities

> Status: complete
> Reviewer: Codex review subagent
> Review date: 2026-08-12
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: both source repositories clean at primary acceptance; the
> previously excluded changes became clean commit `3aa79299` and were re-reviewed

## Question

Do public/internal mocks and testing helpers faithfully model real streaming, Tool, usage, error, cancellation and ordering contracts?

## Scope

- `echo-agent/src/testing`: MockLlmClient, MockAgent, FailingMockAgent, MockTool and MockEmbedder.
- Root `testing` feature/module/prelude exports, required-feature examples, bilingual Mock guides and scoped history.
- Field/variant comparison with provider-neutral `LlmClient`/`ChatChunk`, `Agent`/`AgentEvent`, and `Tool`/`ToolResult`/stream contracts.
- Real framework test consumers in ReAct stream/compact, Subagent executor/team and memory stores, including dedicated local stubs that bound shared-mock limitations.
- Both repositories for duplicate/reachability search only; EKO non-use is not dead-code evidence.

## Out Of Scope

- Source fixes, test execution, Cargo/rustc/builds, dynamic fixtures, benchmarks, network/provider calls or report-index changes.
- Production provider defects owned by `F-LLM-01`, ReAct stream defects owned by `F-RCT-03`, and Tool runtime defects owned by `F-EXT-01`.
- Exhaustive correctness of embedding retrieval and individual application tests.
- Any conclusion imported from other reviewers.

## Inputs

- Root `AGENTS.md`; shared review `README.md`, `REPORTING.md`, `TASKS.md`; Codex `README.md`; report templates.
- Completed dependencies [F-LLM-01](F-LLM-01.md), [F-RCT-03](F-RCT-03.md), and [F-EXT-01](F-EXT-01.md), used only to establish canonical contracts and avoid duplicate production findings.
- Current source, docs, examples, tests and scoped history.
- Isolation disclosure: one bounded section of non-dependency Codex report `F-API-01` was accidentally read. [V90](../validations/F-TST-01/V90-01.md) excludes it and the affected V02-01; V02-02 independently reconstructs the documentation evidence from current source. No other reviewer directory was read.
- Concurrent-mutation disclosure: V00 established clean source repositories before inspection. After source evidence had been collected, `echo-agent/src/testing/mock_llm.rs`, `echo-agent/src/testing/mod.rs`, and `echo-agent/src/agent/react/run/phases/tools.rs` became externally modified, first observed at 2026-08-12 23:32:39 +0800. [V91](../validations/F-TST-01/V91-02.md) records path/status/time metadata only. Their current content/diffs were not read and cannot support a new conclusion.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Deterministic script steps for neutral LLM chunks, Agent events, Tool results/errors, timing/cancellation and strict interaction assertions are reusable framework testing capabilities and belong under `echo_agent::testing`. |
| EKO product policy | EKO may provide application fixtures for UI/storage/product DTOs. It should consume framework mocks for framework contracts and must not create a second LLM/Agent/Tool semantic model. |
| Adapter boundary | A provider protocol fixture translates bytes/wire messages to neutral chunks; a framework mock starts at the neutral chunk/event/result contract. The latter need not emulate HTTP/SSE bytes, but must express every neutral ordering/error/cancel shape consumed by framework code. |
| Duplicate search | Searched both repositories for public and local mocks, feature cfgs, examples/docs, trait implementations, response/event/result scripts, call observers and consumers. Root has one public suite. `echo_state` retains one cfg(test)-private duplicate MockEmbedder; specialized local cancellation/usage/streaming agents/tools exist because public mocks cannot express those cases. |
| Migration deletion | Extend one public strict script engine per neutral trait and migrate generic local stubs where it reduces duplication. Retain specialized protocol-byte fixtures in provider tests. Remove permissive implicit-success fallbacks and stale guide/example references after explicit permissive helpers/callers migrate. |

The APIs are reasonable framework capabilities independent of CLI use and should not be deleted for lack of an EKO caller.

## Current Path

```text
cfg(test) OR feature="testing"
  -> echo_agent::testing / prelude re-exports
     MockLlmClient : LlmClient
       per request VecDeque<Response|Vec<StreamChunk>>
       -> chat: one ChatResponse or folds a structural script
       -> chat_stream: immediate Delta* -> Terminal/Err script
     MockAgent : Agent
       VecDeque<String>
       -> one immediate FinalAnswer; invocation metadata recorded
     MockTool : Tool
       VecDeque<Success(String)|Failure(String)>
       -> text ToolResult; context/default stream behavior inherited
     MockEmbedder : Embedder
       byte hash -> normalized vector

test consumers
  MockLlmClient -> live ReAct create_llm_stream/run_think/direct answer/compact
  MockAgent -> Subagent registry/executor/team/manager tests
  MockTool -> ReAct registry and successful tool-cycle tests
  local specialized stubs -> cancellation propagation, usage and tool streaming
```

Positive conclusions:

- `testing` is isolated from normal builds and current Cargo example targets using mocks declare `required-features=["testing"]` (`Cargo.toml:97,251-269`).
- Queues and call histories use poison-recovering mutex access. MockLlmClient exhaustion is fail-closed with `EmptyResponse` and records messages/tool count/tool choice.
- Commit `3aa79299` adds neutral multi-delta, separate terminal usage and
  mid-stream error scripts; these structural stream-shape repairs are valid and
  should be retained.
- Dedicated `CancellationAwareStreamAgent`, `UsageAgent`, and pipeline streaming tools give valid narrow evidence where they are used; findings below do not invalidate those tests.
- Pure stream processor tests cover Unicode reasoning and tool-argument repair, but not whole-stream lifecycle permutations.

## Findings

### F-TST-01-P2-01: MockLlmClient cannot script inter-chunk waiting or cancellation

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/testing/mock_llm.rs:50`, `:280`, `:312`, `:476`,
  `:495`, `:578`; `echo-agent/echo-core/src/llm/mod.rs:53`, `:232`
- Reachability: ReAct stream/usage/tool/cancel/steer tests inject MockLlmClient through the same `Arc<dyn LlmClient>` path as production providers.
- Expected invariant: the neutral mock scripts ordered structural chunks plus
  optional pending intervals, with request cancellation live while waiting for
  any later delta/error/terminal.
- Observed behavior: the new StreamChunk API correctly supports multiple
  content/reasoning/tool deltas, separate terminal usage and an error after
  visible output. However, all values are emitted through immediately-ready
  `stream::iter`. `with_delay` and the cancellation token apply only before the
  stream is returned; after a partial delta, tests cannot hold the next chunk or
  cancel that pending stream. Cancellation is still untyped ReactError::Other.
- Impact: structural order/usage/error tests are now credible, but idle timeout,
  backpressure and cancellation-after-partial-output branches still require
  special local/provider fixtures. The existing delay test overstates this as
  proof of pending provider-stream cancellation.
- Root cause: the script models values but not timed asynchronous steps, and
  cancellation ownership is discarded when stream construction completes.
- Direction: extend the existing StreamChunk script with timed/pending steps or
  a cancel-aware scripted stream state machine; retain its corrected delta/
  terminal/error types and return a typed cancellation outcome.
- Regression validation: all future cases in V08, plus identical outcome under chunk permutations and fail on unconsumed/surplus scripts.
- Validation reports: [old V03](../validations/F-TST-01/V03-01.md),
  [current V03](../validations/F-TST-01/V03-02.md),
  [V06](../validations/F-TST-01/V06-01.md),
  [V08](../validations/F-TST-01/V08-01.md)

### F-TST-01-P2-02: MockAgent bypasses cancellation and collapses every success to FinalAnswer

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/testing/mock_agent.rs:237`, `echo-agent/src/testing/mock_agent.rs:252`, `echo-agent/src/testing/mock_agent.rs:274`, `echo-agent/src/testing/mock_agent.rs:289`, `echo-agent/echo-core/src/agent/mod.rs:549`, `echo-agent/src/agent/subagent/executor.rs:1147`, `echo-agent/src/agent/subagent/executor.rs:1197`, `echo-agent/src/agent/subagent/executor.rs:2306`
- Reachability: SubagentExecutor invokes the value-scoped text or message method on the registered trait object and consumes Token/Think/Usage/Tool/Error/Cancelled/Final events; broad orchestration tests register MockAgent.
- Expected invariant: cancel-taking overrides retain cooperative cancellation semantics and a scriptable Agent mock can emit the neutral lifecycle sequences the executor aggregates.
- Observed behavior: the value-scoped text override ignores its token and delegates to `execute_stream`; the multimodal method ignores cancellation; all successful streams contain only one FinalAnswer. Current source comments say multimodal forwarding cannot be unit-tested with MockAgent. Dedicated cancellation-aware local stubs are required for three timeout tests.
- Impact: shared-mock tests cannot validate usage aggregation, partial output/error, terminal order, Tool/verification projection, or whether the normal public mock cooperatively stops. Outer executor cancellation may pass even while child work would continue.
- Root cause: observability fields were incrementally added to a string-response stub without promoting its response queue to the AgentEvent/error/time/cancel contract.
- Direction: implement one strict Agent script of timed `Result<AgentEvent>` steps; have every cancel-taking override use the same cancellation-aware engine while recording message/invocation metadata. Delete duplicated one-shot overrides and replace generic local stubs where appropriate.
- Regression validation: text/multimodal/invocation paths; cancel before construction, during delay and after partial event; exact one terminal; usage/tool/error order; verify child work stops.
- Validation reports: [V04](../validations/F-TST-01/V04-01.md), [V06](../validations/F-TST-01/V06-01.md), [V07](../validations/F-TST-01/V07-01.md), [V08](../validations/F-TST-01/V08-01.md)

### F-TST-01-P2-03: MockAgent and MockTool fabricate success after script exhaustion

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/testing/mock_agent.rs:202`, `echo-agent/src/testing/mock_tool.rs:43`, `echo-agent/src/testing/mock_tool.rs:160`, `echo-agent/src/testing/mock_llm.rs:307`
- Reachability: both public mocks drive production orchestration/ReAct tests. Extra calls consume the same queues; unlike MockLlmClient, neither reports exhaustion.
- Expected invariant: strict scripted mocks fail on any unexpected extra call and expose remaining/unconsumed expectations; permissive repeating/default behavior is explicit opt-in.
- Observed behavior: exhausted MockAgent always returns `"mock agent response"`; exhausted MockTool always returns successful `"mock response"`. Neither has `remaining()` or strict verification. Some structural tests intentionally use unexecuted placeholders, but executed scripts receive silent success.
- Impact: unexpected retry, duplicate dispatch or extra tool execution can leave a test green and may even drive subsequent logic down a successful path. Call-count assertions mitigate a minority of cases but are not enforced by the mock.
- Root cause: demo-friendly defaults are also the only public execution semantics.
- Direction: make exhaustion a typed test failure/error by default, add explicit `.repeat_last()`/`.with_default_success()` for intentional permissive fixtures, expose `remaining`/`assert_consumed`, and migrate placeholder callers explicitly. Delete implicit success fallbacks after migration.
- Regression validation: zero/one/many expected calls, extra concurrent calls, unconsumed scripts, repeat/default opt-in, and exact call ordering.
- Validation reports: [V05](../validations/F-TST-01/V05-01.md), [V06](../validations/F-TST-01/V06-01.md), [V08](../validations/F-TST-01/V08-01.md)

### F-TST-01-P2-04: MockTool cannot verify the Tool contracts its guide advertises

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/testing/mock_tool.rs:37`, `echo-agent/src/testing/mock_tool.rs:79`, `echo-agent/src/testing/mock_tool.rs:139`, `echo-agent/echo-core/src/tools/mod.rs:286`, `echo-agent/echo-core/src/tools/mod.rs:739`, `echo-agent/docs/en/12-mock.md:103`, `echo-agent/docs/en/12-mock.md:311`
- Reachability: the guide recommends MockTool for parameter parsing/error behavior; ReAct tool-cycle tests inject it into the live ToolManager/pipeline path.
- Expected invariant: configured schema can reject inputs; tests can script trait-level errors, arbitrary typed `ToolResult`, context/cancellation and stream progress/terminal ordering.
- Observed behavior: `with_parameters` only changes the exposed JSON and inherited validation always succeeds. Responses are limited to successful/error text ToolResults; the mock cannot return `Err`, structured failure, typed/JSON/binary/truncated result, inspect ToolContext/cancel, select timeout/parallel policy, or emit stream progress/output.
- Impact: a green pipeline test with MockTool cannot establish schema enforcement, error classification, context isolation/cancellation, typed projection or stream terminal behavior. Dedicated local Tool implementations must be rewritten for these cases.
- Root cause: MockTool models a text callback rather than the evolved public Tool contract.
- Direction: script arbitrary `Result<ToolResult>` and `ToolStreamEvent`, capture ToolContext, accept validation/policy behavior, and keep convenience text builders layered on this canonical script. Do not add a second Tool schema authority.
- Regression validation: schema invalid/valid, infrastructure Err versus unsuccessful result, all result kinds, context identities/cancel, progress/output/complete ordering, serial/timeout flags.
- Validation reports: [V09](../validations/F-TST-01/V09-01.md), [V08](../validations/F-TST-01/V08-01.md)

### F-TST-01-P3-05: Dedicated Mock guides omit the feature and reference a missing example

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/lib.rs:59`, `echo-agent/Cargo.toml:97`, `echo-agent/docs/en/12-mock.md:39`, `echo-agent/docs/en/12-mock.md:327`, `echo-agent/docs/zh/12-mock.md:39`, `echo-agent/docs/zh/12-mock.md:332`
- Reachability: these bilingual public guides are the canonical testing chapters and directly import `echo_agent::testing`.
- Expected invariant: external users are told to enable `testing` and linked commands name an existing required-feature example.
- Observed behavior: neither guide mentions feature enablement; both end with `cargo run --example demo16_testing`, but no such file/target exists. Current Cargo examples using mocks are correctly feature-gated.
- Impact: following the dedicated guide under default features yields an unresolved module, and its full-example command cannot run.
- Root cause: feature gating and example renumbering were not propagated to the guide.
- Direction: add dependency/CLI feature instructions and replace the stale target with one maintained testing example; delete the demo16 reference.
- Regression validation: external minimal crate with/without feature and a checked command extracted from both guides.
- Validation reports: [V02 corrected](../validations/F-TST-01/V02-02.md), [V07](../validations/F-TST-01/V07-01.md)

### F-TST-01-P3-06: Public MockEmbedder constructor panics on zero dimension

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/testing/mock_embedder.rs:32`, `echo-agent/src/testing/mock_embedder.rs:38`, `echo-agent/src/testing/mock_embedder.rs:49`
- Reachability: any framework consumer enabling `testing` can call `MockEmbedder::new(0)` with an ordinary numeric input.
- Expected invariant: public helpers obey the repository no-panic policy and reject malformed configuration through a typed result or nonzero type.
- Observed behavior: `assert!(dimension > 0)` panics. This protects the later modulo but converts invalid configuration into process panic. The cfg(test)-private echo_state duplicate does the same.
- Impact: parameterized/property test harnesses can abort instead of reporting an invalid fixture; the public API teaches a panic pattern forbidden by project policy.
- Root cause: constructor validation uses an assertion and a raw `usize` instead of a fallible/nonzero contract.
- Direction: accept `NonZeroUsize` or return a typed error; share the corrected implementation with the private duplicate if useful.
- Regression validation: dimensions 0, 1 and large bounded values; empty and multilingual text; no panic.
- Validation reports: [V05](../validations/F-TST-01/V05-01.md), [V08](../validations/F-TST-01/V08-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Commit/source-clean snapshot | yes | passed | [report](../validations/F-TST-01/V00-01.md) |
| V01 | Definition/export/feature/duplicate/reachability inventory | yes | passed | [report](../validations/F-TST-01/V01-01.md) |
| V02 | Testing feature/docs/example entry | yes | 01 excluded; 02 failed source invariant | [01](../validations/F-TST-01/V02-01.md), [02](../validations/F-TST-01/V02-02.md) |
| V03 | MockLlm versus neutral streaming contract | yes | 01 failed old invariant; 02 failed narrowed current invariant | [01](../validations/F-TST-01/V03-01.md), [02](../validations/F-TST-01/V03-02.md) |
| V04 | MockAgent event/cancellation fidelity | yes | failed | [report](../validations/F-TST-01/V04-01.md) |
| V05 | Exhaustion and panic/UTF-8/overflow safety | yes | failed; positive subchecks passed | [report](../validations/F-TST-01/V05-01.md) |
| V06 | Production-module test reliance and dedicated stubs | yes | failed/partial | [report](../validations/F-TST-01/V06-01.md) |
| V07 | History/document drift | yes | failed | [report](../validations/F-TST-01/V07-01.md) |
| V08 | Executable scripted fixtures | no per instruction | not run; future matrix | [report](../validations/F-TST-01/V08-01.md) |
| V09 | MockTool versus public Tool contract | yes | failed | [report](../validations/F-TST-01/V09-01.md) |
| V90 | Non-dependency report read disclosure | evidence integrity | inconclusive; excluded | [report](../validations/F-TST-01/V90-01.md) |
| V91 | Concurrent source-mutation boundary | evidence integrity | 01 inconclusive; 02 passed boundary only | [01](../validations/F-TST-01/V91-01.md), [02](../validations/F-TST-01/V91-02.md) |
| V99 | Final report/link/executor/evidence-boundary integrity | yes | passed | [report](../validations/F-TST-01/V99-01.md) |
| V30 | Primary current-commit rebase/acceptance | yes | passed | [report](../validations/F-TST-01/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| June Phase-3 mock delay proves running LLM cancellation propagation | stale/overbroad | It proves ReAct passes a token to a cooperative mock, not pending provider I/O; F-LLM-01 owns the production defect; [V03](../validations/F-TST-01/V03-01.md), [V06](../validations/F-TST-01/V06-01.md) |
| June multimodal MockAgent can record message forwarding but could not verify trait-object dispatch | current limitation | Source comment remains and shared mock cannot script cancellation/events; [V04](../validations/F-TST-01/V04-01.md), [V07](../validations/F-TST-01/V07-01.md) |
| Mock guides: utilities cover most unit/integration scenarios and are fully controlled | current for deterministic happy paths; stale for evolved streaming/Tool/Agent contracts | [V03](../validations/F-TST-01/V03-01.md), [V04](../validations/F-TST-01/V04-01.md), [V09](../validations/F-TST-01/V09-01.md) |
| Mock guides: full example is `demo16_testing` | stale | No current file or target; [V02-02](../validations/F-TST-01/V02-02.md) |
| Mutex poison hardening | current | Shared mock state uses poison recovery; [V05](../validations/F-TST-01/V05-01.md) |

## Coverage And Uncertainty

- No Cargo, rustc, tests, build, doctest, dynamic fixture, timing probe, network or provider call was run. V08 defines future regression cases.
- Static branch/type evidence is conclusive for what each public mock can express. Runtime scheduler interleavings and exact cancellation latency remain unmeasured.
- This report does not invalidate all mock-based tests. Message capture, request count, happy text/tool cycles, explicit pre-stream error and dedicated local-stub tests retain their stated narrow evidence.
- The accidental non-dependency Codex report read is fully excluded. Primary must independently de-duplicate the guide issue during acceptance/synthesis.
- V01-V09 evidence was collected before the concurrent source mutation. Primary
  V30 reconstructed all affected anchors after those changes became clean commit
  `3aa79299`; V03-02 supersedes old stream semantics while preserving V03-01.

## Handoff

- Primary reconstruction is complete in V30 at clean commit `3aa79299`.
- Keep F-LLM-01, F-RCT-03 and F-EXT-01 as production-defect owners; F-TST findings concern verification capability and false confidence, not duplicate runtime findings.
- Preserve `echo_agent::testing` as an independent framework API. CLI non-use is not deletion evidence.
- Prefer one strict neutral script model per trait, with convenience builders; retain byte-level provider fixtures and truly specialized local stubs.
- This report becomes stale if mock response/event/result scripts, exhaustion defaults, cancel overrides, testing feature exports, or guides/examples change.
