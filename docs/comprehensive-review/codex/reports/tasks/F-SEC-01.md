# F-SEC-01: Guards, sandbox, secrets, and panic safety

> Status: complete
> Reviewer: Codex primary reviewer (delegated static evidence independently sampled)
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: both source repositories clean; only Codex reports written

## Question

Do generic local execution protections prevent framework bugs, accidental data loss, secret logging and sandbox escape without importing inappropriate multi-user/Web permission gates?

## Scope

- Core Guard definitions/implementations and ReAct input/tool/final integration.
- Execution sandbox policy, manager and local/Docker/K8s backend contract projection.
- Root audit/run trace sinks, secret redaction and representative tool output paths.
- Framework PathValidator output confinement and scoped panic/UTF-8/overflow safety.
- Definition, export, feature, registration, real reachability, duplicate authority, current tests and historical documentation.

## Out Of Scope

- Product permission-mode decisions, approval-cache/protected-path semantics, individual Tool contract correctness, integration transport internals and broad repository invariant audit.
- Online multi-user/Web controls. Direct user terminal/MCP/extension actions remain outside automated Agent permission gates per product boundary.
- Source fixes, index changes, Cargo/rustc/test/build/dynamic fixtures and network/provider calls.

## Inputs

- Root `AGENTS.md`; shared review `README.md`, `REPORTING.md`, `TASKS.md`; Codex `README.md`.
- Completed Codex dependencies [B-REF-01](B-REF-01.md) and [F-CORE-01](F-CORE-01.md). [F-HITL-01](F-HITL-01.md) and [F-EXT-01](F-EXT-01.md) were read only for ownership/de-duplication as authorized by primary.
- Current source/docs/tests and scoped history. No other reviewer directory was read.
- Revision transition: evidence collection began while six framework paths were externally dirty at `9b0e0faf`; they became clean external commit `3aa79299`. All adopted anchors were rebuilt from that clean commit. [V00](../validations/F-SEC-01/V00-01.md) and excluded [V90](../validations/F-SEC-01/V90-01.md) record the boundary.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Secret-safe logs/audit/trace, explicit sandbox lower bounds, resource-limit enforcement, bounded/completion-safe Guard execution, typed content transformation, path confinement and no-panic/Unicode-safe parsing are reusable framework contracts. |
| EKO product policy | EKO chooses which automated actions need approval, which local sandbox profile to inject, concrete workspace/path policy and how audit data is shown/retained. User-triggered terminal/MCP remains usable independent of automated permission mode. |
| Adapter boundary | The adapter injects permission policy and sandbox capability independently, maps product paths/config, and disables an unavailable capability. It must not redefine executor selection, Guard scheduling or path canonicalization. |
| Duplicate search | Searched both repositories by types, traits, feature/export names, constructors, directions/results, redaction/audit/trace sinks, path validators and live call paths. No second EKO guard/sandbox authority exists; two output-path algorithms exist inside echo-tools. |
| Migration deletion | Fix canonical authorities in place. Delete fallback-as-default semantics after explicit preferred-level API migration; delete the lexical output validator after callers use the ancestor-aware authority; remove unused/misleading Guard directions or wire them fully; delete stale docs/API examples. |

Framework public capabilities are retained regardless of CLI use.

## Current Path

```text
EKO app-core policy
  -> SandboxManager::local_sandbox (fallback=false; availability checked)

unrelated framework consumer
  -> SandboxManager::auto_detect/with_configs (fallback=true)
  -> SandboxPolicy evaluates a required IsolationLevel
  -> select executor -> may run below required level

user/model/tool text
  -> GuardManager::check_all
     -> capped spawned checks, but permit acquisition + ordered joins can stall
     -> Block/Warn/Pass only; no replacement content
  -> ReAct: Input guarded; successful tool text uses Output
     ToolInput/ToolOutput and final model answer are not live directions

ReAct text/tool/final/error
  -> AuditEvent / Run trace
  -> FileAuditLogger / JsonlRunStore (whole-value JSONL, no central redaction)
ToolCall trace arguments alone -> redacting constructor -> trace

tool output path
  -> lexical validate_output_file
  -> ordinary create/write follows a parent symlink
```

Positive conclusions:

- Permission policy and sandbox mechanism remain separate, matching B-REF-01; this review does not propose automated-mode gates for direct user tools.
- EKO's current adapter uses a fail-closed local sandbox profile and capability availability check.
- The live successful ToolResult path has a secret scanner/output guard and bounded output handling; ToolCall trace arguments use a redacting constructor.
- Scoped production inspection found no additional direct panic/unchecked UTF-8 truncation/overflow defect in primary paths ([V08](../validations/F-SEC-01/V08-01.md)).

## Findings

### F-SEC-01-P0-01: WebSocket HITL logs its bearer authentication token

- Priority: P0
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-orchestration/src/human_loop/websocket.rs:105-117`, `:142-145`, `:172-189`; public human-loop/root feature exports.
- Reachability: `WebSocketHumanLoopProvider::bind*` starts the public provider and emits the `info` line before accepting clients; the same value authorizes approval/input clients.
- Expected invariant: credentials are delivered through an explicit caller-controlled interface and never enter ordinary logs.
- Observed behavior: the full random token is interpolated into an info-level startup message despite an existing `auth_token()` accessor.
- Impact: console/log collectors/diagnostic bundles or another local process/user able to read logs gain the authority represented by the token. This remains a local threat and does not depend on public-network exposure.
- Root cause: startup discoverability was coupled to logging the credential instead of secure presentation by the owning application.
- Direction: remove token material from every log; keep the accessor and let the application explicitly present/store it. Delete the plaintext-log format and add redaction assertions.
- Regression validation: capture all bind/connection logs and assert a known token never occurs while explicit retrieval/authentication still works.
- Validation reports: [V02](../validations/F-SEC-01/V02-01.md), [V10](../validations/F-SEC-01/V10-01.md), [V11](../validations/F-SEC-01/V11-01.md)

### Canonical backlink: F-OPS-01-P0-02 covers raw audit/run-trace secret persistence

- Ownership: canonical finding [F-OPS-01-P0-02](F-OPS-01.md); corroborating
  security evidence only, not a second F-SEC finding
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/src/agent/react/run/phases/prepare.rs:43-55`; `run/pipeline.rs:438-454,581-597`; `run/phases/finalize.rs:65-76,151-161`; `echo-agent/echo-state/src/audit/file.rs:39-53`; `echo-agent/src/agent/react/mod.rs:1907-1994`; `src/trace/mod.rs:423-443,667-749`.
- Reachability: configured audit logger receives these events on normal user/tool/final paths; configured run store persists started/finalized runs. The ToolCall trace constructor proves a redactor exists but covers only its argument field.
- Expected invariant: diagnostic/audit persistence applies one exhaustive sensitive-value policy before durable serialization, with raw retention only through a deliberate opt-in contract.
- Observed behavior: complete raw strings are placed in multiple audit/run variants and whole records are serialized. User prompts, failed tool inputs/errors and final model content bypass the ToolCall-argument redactor.
- Impact: API keys, credentials and private data embedded in normal local work are duplicated into long-lived files beyond the user's expected conversation/tool surface.
- Root cause: redaction is attached to one producer constructor rather than enforced at typed event construction or sink boundaries across every sensitive variant.
- Direction: define one sensitive-value/redaction contract for AuditEvent and Run persistence; cover user/tool/final/error fields exhaustively, retain structured metadata, and require explicit raw-content opt-in. Delete ad-hoc producer assumptions after migration.
- Regression validation: persist representative keys/private Unicode in every variant and assert files/queries contain redacted values only; prove metadata and error classification survive.
- Validation reports: [V02](../validations/F-SEC-01/V02-01.md), [V10](../validations/F-SEC-01/V10-01.md), [V11](../validations/F-SEC-01/V11-01.md)

### F-SEC-01-P1-03: Default sandbox constructors execute below a policy's required isolation

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-execution/src/sandbox/policy.rs:89-104,146-199`; `sandbox/manager.rs:173-211,290-403,471-479,526-546`.
- Reachability: public `auto_detect`/`with_configs` set fallback true; `execute*` evaluates policy then routes through this selection. This is a framework consumer path independent of EKO.
- Expected invariant: a level named/evaluated as required is a lower bound; downgrade occurs only through a separate explicitly advisory request.
- Observed behavior: absent a matching executor, the manager selects the strongest lower backend, warns and runs. Default Strict can therefore execute locally rather than reject.
- Impact: callers reasonably relying on policy/minimum isolation execute code under weaker containment without an error or explicit per-call consent.
- Root cause: required and preferred isolation are represented by one value plus a manager-wide fallback default.
- Direction: separate `minimum_required` from preferred isolation; fail closed for minimum requirements, make downgrade an explicit call/config choice with an observable result. Preserve EKO's current `local_sandbox` fail-closed adapter.
- Regression validation: each required level with absent/unavailable backends; explicit advisory fallback; limits/cancellation parity; assert the executed backend level.
- Validation reports: [V03](../validations/F-SEC-01/V03-01.md), [V10](../validations/F-SEC-01/V10-01.md), [V11](../validations/F-SEC-01/V11-01.md)

### F-SEC-01-P1-04: GuardManager can stall before observing a completed Block and streaming fails open on manager errors

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/guard/mod.rs:118-187`; `echo-agent/src/agent/react/run/react_loop.rs:518-542`; `run/stream_channel.rs:136-179`.
- Reachability: non-streaming and streaming public Agent input paths both call the same manager, but handle its error differently.
- Expected invariant: bounded checks are launched without producer deadlock, Block is observed in completion order, cancellation stops peers, and fail-open/fail-closed behavior is one explicit policy across modes.
- Observed behavior: permit acquisition happens before spawning, so 17+ guards can block construction while the first 16 wait. Handles are then joined in registration order. Guard `Err` becomes Warn, task panic becomes manager `Err`; non-streaming propagates it while streaming ignores every non-`Ok(Block)` result and continues.
- Impact: an early slow guard can make a later fast Block ineffective or hang input; identical input has mode-dependent protection after guard panic/error.
- Root cause: concurrency admission and completion collection are owned by the producer/ordered vector, while failure policy is implicit in callers.
- Direction: acquire permits inside spawned tasks, consume a bounded completion-order set, add cancellation/timeout, and centralize explicit failure policy. Delete caller-specific error interpretation.
- Regression validation: 17+ pending checks, later immediate Block, task panic/error, timeout/cancel, streaming/non-streaming parity and no orphan work.
- Validation reports: [V04](../validations/F-SEC-01/V04-01.md), [V10](../validations/F-SEC-01/V10-01.md), [V11](../validations/F-SEC-01/V11-01.md)

### F-SEC-01-P1-05: ContentGuard Redact discards the transformed content on live Agent paths

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/guard/content.rs:124-159,202-234`; `echo-agent/src/agent/react/builder.rs:758-763`; live Input callers above.
- Reachability: ContentGuard implements the Guard trait consumed by GuardManager. ReAct proceeds with the original user/tool string on Warn.
- Expected invariant: Redact replaces content before it reaches storage, the model or downstream tools, or the API rejects Redact as unsupported.
- Observed behavior: standalone ContentGuard computes a redacted String; the trait adapter drops it and returns a warning claiming redaction, while the original text continues.
- Impact: applications can configure advertised redaction yet transmit/persist the exact PII/secret they intended to remove.
- Root cause: GuardResult has no typed replacement and checking was bolted onto a boolean/block pipeline.
- Direction: add a canonical preprocessing/transformation result and thread it through every caller, or remove Redact from Guard integration and documentation until supported. Delete misleading warning text.
- Regression validation: multilingual PII through user/tool/final paths; assert transformed content in model request, context, audit/trace and emitted output.
- Validation reports: [V05](../validations/F-SEC-01/V05-01.md), [V09](../validations/F-SEC-01/V09-01.md), [V10](../validations/F-SEC-01/V10-01.md), [V11](../validations/F-SEC-01/V11-01.md)

### F-SEC-01-P1-06: Public Guard directions do not match runtime enforcement points

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/guard/mod.rs:16-27`; ReAct Input call sites; `run/pipeline.rs:691-713`; `run/phases/finalize.rs:65-87,141-168`.
- Reachability: Input is live; Output is applied to successful ToolResult text. No live ToolInput/ToolOutput call exists, and final model answers are persisted/emitted without GuardManager.
- Expected invariant: direction names describe distinct live boundaries: user input, model output, tool arguments and tool results.
- Observed behavior: model Output is absent, tool output is mislabeled Output, and two public variants are unused.
- Impact: callers configure direction-specific rules that silently never execute or execute against the wrong content class; final model output bypasses advertised output protection.
- Root cause: the enum evolved independently of pipeline registration/cardinality.
- Direction: wire each boundary exactly once with truthful names, or remove unsupported variants until implemented; migrate Output semantics explicitly rather than aliasing tool output.
- Regression validation: unique marker per direction across normal/streaming/tool/final paths, asserting exact once and correct payload.
- Validation reports: [V05](../validations/F-SEC-01/V05-01.md), [V10](../validations/F-SEC-01/V10-01.md), [V11](../validations/F-SEC-01/V11-01.md)

### F-SEC-01-P1-07: K8s executes despite an unenforced `network=false` resource limit

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-execution/src/sandbox/k8s.rs:194-267`; public `ResourceLimits` execution path.
- Reachability: callers choosing K8s and `execute_with_limits` reach this branch before `kubectl run` pod construction.
- Expected invariant: a requested no-network limit is enforced or rejected as unsupported before execution.
- Observed behavior: the backend warns it cannot fully disable network and continues with cluster defaults; generated pod fields do not establish network isolation.
- Impact: code explicitly requested to run without network can access whatever the namespace/cluster permits, violating the caller's containment contract.
- Root cause: ResourceLimits mixes enforceable backend controls with best-effort hints without capability negotiation.
- Direction: advertise backend limit capabilities and reject unsupported hard limits, or require/verify an isolated namespace/NetworkPolicy before launch. Do not silently downgrade.
- Regression validation: fake/isolated cluster manifest inspection and denial/allow cases; no pod launch when hard network isolation is unavailable.
- Validation reports: [V03](../validations/F-SEC-01/V03-01.md), [V10](../validations/F-SEC-01/V10-01.md), [V11](../validations/F-SEC-01/V11-01.md)

### F-SEC-01-P1-08: Output path validation can escape allowed roots through a parent symlink

- Priority: P1
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-tools/src/security.rs:265-398`; `echo-tools/src/text.rs:435-448`; `echo-tools/src/data.rs:1001-1024`.
- Reachability: representative export tools call `validate_output_file`, create parents and write/create the returned path.
- Expected invariant: an allowed output resolves inside a canonical allowed root at open time, including non-existent suffixes.
- Observed behavior: `validate_output_file` only normalizes lexical components/prefixes. An allowed-root symlink can resolve outside it; ordinary writes follow it. The same type already has an ancestor-aware canonical validator.
- Impact: an automated framework tool can overwrite/create data outside its configured workspace despite allowed-root policy; symlink replacement leaves a validation/open race.
- Root cause: a second weaker path authority was retained for output files.
- Direction: converge output callers on canonical-nearest-ancestor validation and use no-follow/dirfd-style open semantics where overwrite integrity matters. Delete the lexical implementation after migration.
- Regression validation: existing/nonexistent nested outputs through inside/outside symlinks, symlink replacement between validation/open, Unicode paths and denied roots.
- Validation reports: [V06](../validations/F-SEC-01/V06-01.md), [V10](../validations/F-SEC-01/V10-01.md), [V11](../validations/F-SEC-01/V11-01.md)

### F-SEC-01-P2-09: RuleGuard silently drops malformed caller regexes

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/echo-core/src/guard/rule.rs:102-109,136-145`.
- Reachability: every external caller using public `blocked_pattern(...).build()` receives an apparently valid Guard even when the pattern was invalid.
- Expected invariant: malformed security configuration is a typed construction error.
- Observed behavior: invalid regex logs a warning, is omitted, and infallible build succeeds.
- Impact: intended blocking rules are absent while startup/configuration reports success; logs may not be visible to embedding applications.
- Root cause: parsing errors are erased to preserve a fluent infallible builder.
- Direction: make pattern insertion/build fallible and preserve source/index in the error; delete ignore-and-continue behavior.
- Regression validation: invalid/empty/Unicode/large patterns and mixed valid-invalid batches must fail without producing a partially protected guard.
- Validation reports: [V07](../validations/F-SEC-01/V07-01.md), [V10](../validations/F-SEC-01/V10-01.md), [V11](../validations/F-SEC-01/V11-01.md)

### F-SEC-01-P3-10: Guard guides describe removed APIs and transformations runtime cannot perform

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `echo-agent/docs/en/18-guard-system.md:5,133-185,214-228`; corresponding Chinese guide; current `echo-core/src/guard/mod.rs` and ReAct builder.
- Reachability: bilingual public documentation is the framework user's construction/behavior entry point.
- Expected invariant: docs use current constructors and accurately describe evaluation and mutation.
- Observed behavior: they show removed `add_input_guard`/`add_output_guard` methods, sequential short-circuit and content modification; current manager/API does not provide those semantics.
- Impact: users cannot follow examples and may believe sensitive content has been modified when it has not.
- Root cause: API/concurrency/redaction changes did not update both guides.
- Direction: document one current construction path, honest direction cardinality/failure policy, and only supported transformation semantics; delete stale snippets/claims.
- Regression validation: checked docs example plus source-linked behavior assertions in both languages.
- Validation reports: [V09](../validations/F-SEC-01/V09-01.md), [V11](../validations/F-SEC-01/V11-01.md)

## Validation Matrix

| ID | Claim or execution | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Revision/clean-state transition | yes | passed | [report](../validations/F-SEC-01/V00-01.md) |
| V01 | Definition/export/duplicate/layering inventory | yes | passed | [report](../validations/F-SEC-01/V01-01.md) |
| V02 | Secret logging and durable persistence trace | yes | failed | [report](../validations/F-SEC-01/V02-01.md) |
| V03 | Isolation fallback and resource-limit enforcement | yes | failed | [report](../validations/F-SEC-01/V03-01.md) |
| V04 | Guard scheduling/error/mode parity | yes | failed | [report](../validations/F-SEC-01/V04-01.md) |
| V05 | Guard transformation/direction reachability | yes | failed | [report](../validations/F-SEC-01/V05-01.md) |
| V06 | Output path confinement | yes | failed | [report](../validations/F-SEC-01/V06-01.md) |
| V07 | Malformed RuleGuard configuration | yes | failed | [report](../validations/F-SEC-01/V07-01.md) |
| V08 | Scoped panic/UTF-8/overflow static audit | yes | passed | [report](../validations/F-SEC-01/V08-01.md) |
| V09 | History/document drift | yes | failed | [report](../validations/F-SEC-01/V09-01.md) |
| V10 | Existing test coverage inventory | yes | passed inventory | [report](../validations/F-SEC-01/V10-01.md) |
| V11 | Targeted executable regression matrix | no per instruction | not run; future | [report](../validations/F-SEC-01/V11-01.md) |
| V12 | Earlier immutable executable-case declaration | no per instruction | not run; future; retained | [report](../validations/F-SEC-01/V12-01.md) |
| V90 | Accidental dirty-source search disclosure | integrity | inconclusive; excluded | [report](../validations/F-SEC-01/V90-01.md) |
| V99 | Final links/executor/source-state integrity | yes | passed | [report](../validations/F-SEC-01/V99-01.md) |
| V30 | Primary deduplication, current-source sampling and acceptance | yes | passed | [report](../validations/F-SEC-01/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| Mature systems separate action permission policy from execution sandbox | current | Framework types and EKO adapter remain separate; [V01](../validations/F-SEC-01/V01-01.md), B-REF-01 |
| Direct user terminal/MCP should not be gated by automated permission mode | current product boundary | This review adds no such gate and evaluates only framework protection contracts. |
| ToolCall trace arguments are redacted before serialization | current but narrow | `RunEvent::new_tool_call` is live; it does not cover raw audit/run variants; [V02](../validations/F-SEC-01/V02-01.md) |
| Guard system can modify content and sequentially short-circuit | regressed/stale | Current GuardResult cannot carry replacement content and manager behavior is concurrent/ordered-join; [V05](../validations/F-SEC-01/V05-01.md), [V09](../validations/F-SEC-01/V09-01.md) |
| `run_code` requires native local isolation in EKO | current/fixed positive | EKO injects `local_sandbox`, checks availability and disables the tool otherwise; [V01](../validations/F-SEC-01/V01-01.md), [V03](../validations/F-SEC-01/V03-01.md) |

## Coverage And Uncertainty

- No Cargo, rustc, tests, builds, dynamic fixtures, network or provider calls ran. Scheduler timing, OS symlink behavior and emitted/persisted byte content remain executable future validation in V11.
- Static source establishes control/data-flow and absent transform/enforcement branches at the reviewed commits. It cannot measure exact cancellation latency or backend platform behavior.
- The P0 persistence finding does not require redacting canonical user-owned conversation storage by default; it is limited to diagnostic/audit/trace duplication and demands an explicit raw-retention contract where needed.
- Integration-specific MCP/LSP/channel transport issues remain with F-INT tasks; approval/protected-path authority remains F-HITL-01; Tool schema/cancel/result correctness remains F-EXT-01.
- Any changes to guard result/directions, manager scheduling, sandbox fallback/capabilities, AuditEvent/Run serialization, PathValidator or the reviewed adapters stale this report.

## Handoff

- Keep F-SEC-01-P0-01 plus canonical F-OPS-01-P0-02 as the secret-safety synthesis;
  do not duplicate the durable-persistence finding or approval semantics from F-HITL-01.
- Treat “required isolation” and “preferred isolation” as separate contracts. EKO's current local fail-closed adapter is a positive example, not a reason to delete general sandbox backends.
- Converge on one ancestor-aware path authority and one explicit Guard execution/transformation contract; deletion targets are named in findings.
- Primary reconstruction, deduplication and acceptance are recorded in V30.
  Dynamic V11 remains future regression evidence and does not block completion.
