# F-HITL-01: Human loop and permission model

> Status: complete
> Reviewer: Codex primary reviewer
> Review date: 2026-08-12
> `echo-agent` commit: `9b0e0faf74d35c9a432370b923acabfbb5f32d63`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: both source repositories clean at final source inspection; review reports only

## Question

Does the framework provide generic automated-action approval and protected-path
primitives without imposing EKO interaction policy, while preserving the exact
approved invocation through execution?

## Scope

- Portable permission modes, rules, decisions, policies, and tool permission
  metadata in `echo-core`.
- All policy, request/response, provider, cache, classifier, audit, protected-path,
  batch, and service code under `echo-orchestration/src/human_loop`.
- Root ReactAgent construction, provider/service replacement, snapshot approval
  helper, canonical ToolExecutionPipeline, and legacy approval helper.
- Narrow EKO runtime/pool/UI-provider reads only to establish layering and live
  registration/reachability.
- Definition/duplicate search, field mapping, decision/default/timeout/cache
  identity matrices, panic/UTF-8 scan, and current test inventory.

## Out Of Scope

- Source fixes or application-specific trust rules/UI design.
- Generic tool schema and post-hook validation defects owned by F-EXT-01, except
  where HITL loses the user's modified approved invocation entirely.
- Generic concurrent tool-batch scheduling owned by F-RCT-04.
- Sandbox implementation and concrete shell/file correctness owned by F-SEC-01
  and F-EXT-02.
- Cargo, rustc, builds, tests, network calls, or dynamic fixtures. Future
  regressions are specified without claiming execution.

## Inputs

- Root `AGENTS.md`; shared review protocol and F-HITL-01 task card; Codex rules.
- [B-REF-01](B-REF-01.md) for mature policy/enforcement separation.
- [F-RCT-04](F-RCT-04.md) for generic batch lifecycle boundaries.
- Current source and current test/documentation text as hypotheses. No other
  reviewer directory was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Portable permission facts, one decision service, call-scoped typed approval, provider-neutral request/response, cancellation/timeout, exact scope identity, optional protected-path primitives, and audit are reusable framework concerns. |
| EKO product policy | Whether an automated action asks, trusted path/command defaults, UI wording, and which TUI/GUI/channel provider handles a request belong to EKO. User-initiated terminal/MCP features must not be gated by automated-action mode. |
| Adapter boundary | EKO selects policy and injects a provider. The adapter must preserve request/call/session identity and final arguments but own no second policy engine, cache, or execution state machine. |
| Duplicate search | Searched both repositories for permission modes/decisions/policies, rules, services, approval scopes/cache, HumanLoop request/provider types, batch APIs, builders, provider swaps, canonical/legacy execution consumers, and tests. |
| Migration deletion | Keep PermissionService plus HumanLoopProvider as the canonical policy/transport split. Return a typed call-scoped approval result. Remove the shared modified-args side channel and obsolete parallel decision APIs/config after reasonable public use is covered. Product defaults move to EKO; the framework keeps configurable primitives. |

## Current Path

```text
EKO startup/pool
  -> HitlDispatcher provider selection
  -> ReactAgent provider swap + PermissionService construction

canonical ReAct tool call
  -> AgentRunSnapshot
  -> ToolExecutionPipeline
       ParseValidate
       PreToolUse hook (may mutate input)
       PermissionStage
         -> snapshot.check_tool_approval
         -> PermissionService(mode/rules/cache/classifier/provider)
       ReadBeforeEdit / skill policy / audit
       ExecuteStage(original or hook-mutated params)
```

The type-level split between automated-action policy and UI transport is a
sound foundation. The critical defects occur where the service's decision is
adapted into the canonical execution pipeline.

## Findings

### F-HITL-01-P0-01: Canonical execution discards user-modified approved arguments

- Priority: P0
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-orchestration/src/human_loop/service.rs:728`,
  `echo-agent/echo-orchestration/src/human_loop/service.rs:730`,
  `echo-agent/src/agent/snapshot.rs:833`,
  `echo-agent/src/agent/react/run/pipeline.rs:328`,
  `echo-agent/src/agent/react/run/pipeline.rs:340`
- Reachability: EKO wires PermissionService; every canonical ReAct tool call runs
  PermissionStage through AgentRunSnapshot.
- Expected invariant: when the user changes dangerous arguments and approves the
  replacement, exactly that validated replacement is executed.
- Observed behavior: PermissionService stores modified input in a shared side
  channel but returns Allow. The canonical snapshot helper maps Allow to None
  without consuming the replacement, so PermissionStage leaves original params.
  Only an older non-canonical helper calls `take_modified_args`.
- Impact: the UI can show approval for a safe edit while the framework executes
  the original destructive invocation, risking local data loss.
- Root cause: final invocation data is separated from the approval result and a
  legacy/canonical split left only one path consuming it.
- Direction: return `Approved { final_input, scope, identity }` atomically; make
  the canonical pipeline own it, rerun schema/protected-path checks, and delete
  `last_modified_args` plus the legacy helper.
- Regression validation: canonical provider modifies a file/shell call; assert
  original params never reach the Tool and final params are revalidated.
- Validation reports: [V03-01](../validations/F-HITL-01/V03-01.md)

### F-HITL-01-P1-02: Session approval is widened from exact arguments to every invocation of the tool

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-orchestration/src/human_loop/policy.rs:22`,
  `echo-agent/echo-orchestration/src/human_loop/service.rs:892`,
  `echo-agent/echo-orchestration/src/human_loop/service.rs:791`,
  `echo-agent/echo-orchestration/src/human_loop/service.rs:798`,
  `echo-agent/echo-orchestration/src/human_loop/service.rs:737`
- Reachability: every ApprovedWithScope/ModifiedArgs response through the dynamic
  provider adapter reaches this mapping.
- Expected invariant: Session means same tool and same final arguments;
  SessionAllTools is the explicit broader choice.
- Observed behavior: Session emits a tool-name allow rule, and any session allow
  is inferred as SessionAllTools for cache. ModifiedArgs caches original input.
- Impact: approving one benign command/path can silently authorize different,
  more destructive arguments for the rest of the session.
- Root cause: one untyped string rule is used both as a policy update and as a
  scope classifier.
- Direction: persist exact typed scope/key using final arguments; delete scope
  inference from rule strings and keep the broader option explicit.
- Regression validation: approve benign A with Session, then dangerous B for the
  same tool; B must prompt. Repeat with ModifiedArgs and restart/provider swap.
- Validation reports: [V04-01](../validations/F-HITL-01/V04-01.md)

### F-HITL-01-P1-03: Approval cache has no logical session, conversation, or agent identity

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-orchestration/src/human_loop/approval_cache.rs:35`,
  `echo-agent/echo-orchestration/src/human_loop/approval_cache.rs:100`,
  `echo-agent/echo-orchestration/src/human_loop/permission.rs:291`,
  `echo-agent/echo-orchestration/src/human_loop/service.rs:276`
- Reachability: PermissionService may be shared/preserved while EKO swaps a
  run-scoped GUI transport.
- Expected invariant: a session-scoped approval is reusable only inside the
  logical session/conversation/agent scope shown to the user.
- Observed behavior: cache keys contain only tool and optionally args. Request
  types define session/agent/request IDs, but live service construction does not
  populate or use them.
- Impact: approval can leak across conversations or agents sharing the service.
- Root cause: “session” is a TTL label rather than a stable identity boundary.
- Direction: require call/session identity in PermissionCheckContext and cache
  keys; transport replacement must not define session ownership.
- Regression validation: two conversations and two Subagents sharing a service,
  same/different arguments, provider replacement, TTL and revocation.
- Validation reports: [V04-01](../validations/F-HITL-01/V04-01.md)

### F-HITL-01-P1-04: Canonical Ask and RequireApproval outcomes never request the HumanLoop provider

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-orchestration/src/human_loop/service.rs:564`,
  `echo-agent/echo-orchestration/src/human_loop/service.rs:580`,
  `echo-agent/echo-orchestration/src/human_loop/service.rs:643`,
  `echo-agent/src/agent/snapshot.rs:841`,
  `echo-agent/src/agent/snapshot.rs:847`
- Reachability: explicit ask rules, Bubble, missing classifier, no handler, and
  denial fallback can return these decisions on the live pipeline.
- Expected invariant: a decision requiring human input creates one identified,
  cancellable HumanLoop request or explicit resumable suspension.
- Observed behavior: the snapshot helper converts these decisions into textual
  errors. The older helper contains provider routing but is not canonical.
- Impact: valid Ask/Bubble/fallback configurations make tools fail instead of
  pausing for the user, so advertised HITL behavior is unusable.
- Root cause: decision evaluation and interaction routing were split across two
  execution helpers.
- Direction: centralize both into a typed permission outcome consumed by one
  pipeline; delete the duplicate helper after migration.
- Regression validation: every decision origin must yield one request, then
  resume/deny/cancel with stable call identity.
- Validation reports: [V05-01](../validations/F-HITL-01/V05-01.md)

### F-HITL-01-P1-05: Provider adapter drops risk, identity, context, suggestions, and timeout

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent/echo-orchestration/src/human_loop/permission.rs:272`,
  `echo-agent/echo-orchestration/src/human_loop/permission.rs:287`,
  `echo-agent/echo-orchestration/src/human_loop/service.rs:864`,
  `echo-agent/echo-orchestration/src/human_loop/mod.rs:603`,
  `echo-agent/echo-orchestration/src/human_loop/mod.rs:614`
- Reachability: `PermissionService::from_provider` is the EKO convenience path.
- Expected invariant: a provider receives the same identified, bounded decision
  context the service evaluated.
- Observed behavior: DynProviderHandler reconstructs only tool and args, dropping
  request/session/agent IDs, risk, context, suggestions, and timeout. Input also
  ignores timeout in HumanLoopManager, while providers return incompatible
  timeout forms. TimeoutStrategy is never read.
- Impact: UI cannot correlate concurrent approvals reliably or enforce the
  intended deadline/default behavior.
- Root cause: two request models are bridged by a lossy constructor.
- Direction: use one request envelope or a field-complete conversion with stable
  ID and absolute deadline; remove inert TimeoutStrategy until implemented.
- Regression validation: field round-trip and timeout/cancel matrix across
  manager, console, webhook, websocket, TUI/GUI/channel adapters.
- Validation reports: [V06-01](../validations/F-HITL-01/V06-01.md)

### F-HITL-01-P1-06: Auto classifier allows malformed responses with fabricated context

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-orchestration/src/human_loop/service.rs:689`,
  `echo-agent/echo-orchestration/src/human_loop/classifier.rs:499`,
  `echo-agent/echo-orchestration/src/human_loop/classifier.rs:621`,
  `echo-agent/echo-orchestration/src/human_loop/classifier.rs:622`
- Reachability: selecting PermissionMode::Auto invokes this classifier.
- Expected invariant: automated policy consumes real invocation context and
  parse uncertainty denies or asks rather than silently allowing.
- Observed behavior: service supplies literal agent/session and no workspace,
  files, history, or risk context. Unparseable output becomes Allow(0.5).
- Impact: the automated policy can approve the exact ambiguous cases it cannot
  classify, contrary to its advertised safety role.
- Root cause: an experimental classifier was promoted to policy authority without
  typed output enforcement or invocation context wiring.
- Direction: fail closed/Ask on parse uncertainty and pass a real immutable
  context snapshot; otherwise remove Auto mode from the public live surface.
- Regression validation: malformed/partial/provider-error replies plus real
  context capture; never Allow without a typed valid decision.
- Validation reports: [V07-01](../validations/F-HITL-01/V07-01.md)

### F-HITL-01-P1-07: Batch approval erases ModifiedArgs and executes under an Approved projection

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-orchestration/src/human_loop/batch.rs:144`,
  `echo-agent/echo-orchestration/src/human_loop/batch.rs:151`,
  `echo-agent/echo-orchestration/src/human_loop/batch.rs:153`
- Reachability: public framework consumers can use BatchApprovalProvider; live
  EKO does not currently call it, which is not grounds for deletion.
- Expected invariant: each batch item preserves its final arguments, scope, and
  individual outcome.
- Observed behavior: ModifiedArgs is mapped to Approved and both fields vanish.
- Impact: a reasonable external consumer can execute original dangerous input
  after the user only approved a replacement.
- Root cause: BatchItemDecision cannot represent the full single-item contract.
- Direction: make batch decisions contain the typed final invocation or compose
  the single-call outcome type; delete the lossy enum mapping.
- Regression validation: mixed batch with modified/session/reject/timeout and
  exact call/result identity.
- Validation reports: [V08-01](../validations/F-HITL-01/V08-01.md)

### F-HITL-01-P1-08: HumanLoopManager public zero buffer panics

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-orchestration/src/human_loop/mod.rs:332`,
  `echo-agent/echo-orchestration/src/human_loop/mod.rs:334`
- Reachability: any external framework consumer can pass zero to the public
  constructor.
- Expected invariant: public configuration returns a typed error or normalizes
  invalid input, never panics.
- Observed behavior: zero is forwarded to Tokio bounded mpsc channel creation,
  which requires a positive capacity.
- Impact: a valid Rust call can terminate the process during setup.
- Root cause: capacity invariant is neither encoded nor validated.
- Direction: return Result or use NonZeroUsize; audit sibling zero capacities.
- Regression validation: zero, one, and maximum configured values without panic.
- Validation reports: [V08-01](../validations/F-HITL-01/V08-01.md)

### F-HITL-01-P1-09: Default protected-path matching blocks `.gitignore` in every mode

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-orchestration/src/human_loop/protected.rs:24`,
  `echo-agent/echo-orchestration/src/human_loop/protected.rs:140`,
  `echo-agent/echo-orchestration/src/human_loop/protected.rs:157`,
  `echo-agent/echo-orchestration/src/human_loop/service.rs:512`
- Reachability: PermissionService::new/from_provider installs the checker and
  evaluates it before BypassPermissions.
- Expected invariant: `.git` means that complete directory segment, not ordinary
  developer files sharing the prefix; product policy remains configurable.
- Observed behavior: contains/starts-with lacks a trailing segment boundary, so
  `.gitignore` matches `.git` and is denied even in bypass mode.
- Impact: normal agent edits to a common repository file are unusable.
- Root cause: string containment substitutes for normalized path-segment matching,
  while a product default is installed as unconditional framework behavior.
- Direction: keep a generic canonical-path checker with exact segment semantics;
  move concrete defaults/exceptions to product policy. Do not add gates to
  user-interactive terminal/MCP paths.
- Regression validation: `.git`, `.git/config`, `.gitignore`, `.github`, relative,
  symlink/canonical, Unicode, and disabled/custom policy cases.
- Validation reports: [V09-01](../validations/F-HITL-01/V09-01.md)

### F-HITL-01-P2-10: Public permission surface retains parallel authorities and inert configuration

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/tools/permission.rs:505`,
  `echo-agent/echo-orchestration/src/human_loop/policy.rs:11`,
  `echo-agent/echo-orchestration/src/human_loop/service.rs:102`,
  `echo-agent/echo-orchestration/src/human_loop/batch.rs:119`
- Reachability: PermissionService is live; DefaultPermissionPolicy,
  ApprovalRule/PolicyDecision, BatchApprovalProvider, builder, TimeoutStrategy,
  enable_classifier, max_total_denials and several identity fields are public or
  configurable but disconnected or partially consumed.
- Expected invariant: public optional framework capabilities are retained when
  coherent, but overlapping concepts have explicit composition and each config
  field changes behavior.
- Observed behavior: old policy types remain after ownership was declared moved;
  batch is a separate lossy outcome model; several service fields are never read.
- Impact: consumers cannot identify the real authority, and changes/tests can
  target surfaces that do not affect canonical execution.
- Root cause: staged migrations added a unified service without completing
  field/API deletion or composition.
- Direction: converge decisions on PermissionService + one typed outcome, retain
  only composable low-level primitives, and delete inert fields/duplicate APIs
  after reasonable external use is covered.
- Regression validation: one end-to-end public entry exercises mode, rule,
  provider, cache, timeout, modification, audit, and terminal result; repository
  search proves replaced authorities are gone.
- Validation reports: [V01-01](../validations/F-HITL-01/V01-01.md),
  [V06-01](../validations/F-HITL-01/V06-01.md),
  [V07-01](../validations/F-HITL-01/V07-01.md)

## Validation Matrix

| ID | Claim or inspection | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Task/dependency boundary | yes | passed on attempt 02 | [V00-01](../validations/F-HITL-01/V00-01.md), [V00-02](../validations/F-HITL-01/V00-02.md) |
| V01 | Definition/export/duplicate-authority search | yes | failed invariant | [V01-01](../validations/F-HITL-01/V01-01.md) |
| V02 | Registration and canonical runtime reachability | yes | passed on attempt 02 | [V02-01](../validations/F-HITL-01/V02-01.md), [V02-02](../validations/F-HITL-01/V02-02.md) |
| V03 | Modified-argument execution preservation | yes | failed invariant | [V03-01](../validations/F-HITL-01/V03-01.md) |
| V04 | Approval scope/cache identity | yes | failed invariant | [V04-01](../validations/F-HITL-01/V04-01.md) |
| V05 | Ask/RequireApproval provider routing | yes | failed invariant | [V05-01](../validations/F-HITL-01/V05-01.md) |
| V06 | Request/provider field and timeout mapping | yes | failed invariant | [V06-01](../validations/F-HITL-01/V06-01.md) |
| V07 | Auto classifier context/default behavior | yes | failed invariant | [V07-01](../validations/F-HITL-01/V07-01.md) |
| V08 | Batch semantics and panic/UTF-8/config scan | yes | failed invariant | [V08-01](../validations/F-HITL-01/V08-01.md) |
| V09 | Protected-path semantic/product boundary | yes | failed invariant | [V09-01](../validations/F-HITL-01/V09-01.md) |
| V10 | Existing-test and external-reference classification | yes | passed | [V10-01](../validations/F-HITL-01/V10-01.md) |
| V99 | Report/link/source integrity | yes | passed | [V99-01](../validations/F-HITL-01/V99-01.md) |

Future fix regressions, not executed during review: concurrent ModifiedArgs;
session/agent cache matrix; Ask/Bubble/no-classifier routing; field-complete
provider round-trip; provider timeout/cancel; malformed classifier responses;
mixed batch; zero capacities; protected-path canonicalization.

## Historical Claim Status

| Claim | Classification | Current evidence |
|---|---|---|
| B-REF-01: approval policy and enforcement/sandbox are separate | current | types are separated, but adapters lose decision identity/data |
| AGENTS: automated-action policy must not gate user-interactive terminal/MCP | current | this report recommends no such gate; concrete defaults remain EKO policy |
| `policy.rs`: PermissionService is the unified permission entry | current but incomplete migration | V01/V02 identify live service plus parallel remnants |
| PermissionService modified input is returned to caller | regressed on canonical path | V03: only legacy helper consumes side channel |
| ApprovalScope::Session means same tool and args | regressed | V04: service widens to tool/all arguments |
| TimeoutStrategy config controls timeout behavior | stale | V06: no reader |
| Default protected `.git` pattern protects the directory | overbroad | V09: also blocks `.gitignore` |

## Coverage And Uncertainty

- All production files in the assigned human_loop directory and the canonical
  root integration sites were statically inspected. Tests were inventoried but
  not executed.
- P0/P1 findings are source-conclusive about field/control flow. Exact concurrent
  timing and provider behavior remain future regression tests.
- This report deliberately does not treat local user extensions as hostile
  tenants and does not propose cloud-style permission gates. Findings prevent
  framework-caused execution mismatch, data loss, unusable approval, or panic.
- Public optional providers/policies are not classified dead merely because EKO
  does not use them; deletion is limited to replaced/inert duplicate authority.

## Handoff

- First iteration item: replace the shared modified-args side channel with a
  call-scoped typed approval outcome and make the canonical pipeline the only
  consumer; revalidate final args and delete the legacy helper.
- Second: make scope/session identity explicit and losslessly map one request
  envelope through every provider. Then repair Ask/Require routing and deadlines.
- Third: make Auto parse uncertainty Ask/deny, repair batch and zero-capacity
  contracts, and move concrete protected-path defaults to EKO after exact segment
  matching is fixed.
- F-EXT-01 owns general schema/hook mutation validation; F-RCT-04 owns batch
  scheduling; F-SEC-01 owns sandbox/enforcement. Do not duplicate them.
- This report becomes stale if PermissionService/outcome/request types, snapshot
  approval helper, pipeline order, cache key, providers, or protected defaults change.
