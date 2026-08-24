# F-HITL-01: Human loop and permission model

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: not-applicable (framework-only task; EKO adapter inspected read-only for boundary classification)
> Worktree state: clean (read-only review task)

## Question

Does the framework provide generic automated-action approval and
protected-path primitives without imposing EKO interaction policy?

## Scope

Primary source paths and behaviors inspected (all under `echo-agent/`
at commit `9b0e0fa`, framework layer unless noted):

- `echo-orchestration/src/human_loop/mod.rs` (947 lines) —
  `HumanLoopProvider` trait, `HumanLoopManager` (event-driven provider),
  `HumanLoopRequest`/`HumanLoopResponse`/`ApprovalDecision`/
  `ApprovalResponder` (Drop = default-Reject), `HumanLoopHandler` trait,
  `dispatch_event`.
- `echo-orchestration/src/human_loop/service.rs` (1316 lines) —
  `PermissionService` (the unified check entry), `PermissionServiceConfig`,
  `TimeoutStrategy`, `check_with_permissions_in_mode` (8-step pipeline),
  `would_request_human_for_permissions` (pure batch-planning predictor),
  `DynProviderHandler` (trait bridge), `NullPermissionRequestHandler`,
  `PermissionServiceBuilder`.
- `echo-orchestration/src/human_loop/permission.rs` (856 lines) —
  `RiskLevel` (runtime), `PermissionRequest`/`PermissionResponse`/
  `PermissionResponseDecision`/`PermissionUpdate`, `Suggestion`/
  `SuggestedAction`, `PermissionRequestHandler` trait,
  `DefaultPermissionRequestHandler`, `build_matcher`.
- `echo-orchestration/src/human_loop/policy.rs` (137 lines) —
  `ApprovalScope` (Once/Session/SessionAllTools), `ApprovalRule`,
  `PolicyDecision`.
- `echo-orchestration/src/human_loop/approval_cache.rs` (511 lines) —
  `SessionApprovalCache` (3-scope cache, TTL, LRU eviction,
  `args_key`/`sha256_hex`).
- `echo-orchestration/src/human_loop/protected.rs` (625 lines) —
  `ProtectedPathChecker` + default patterns + bash/path extraction.
- `echo-orchestration/src/human_loop/classifier.rs:1-90` — `Classifier`
  trait (auto-mode), `ClassifierContext`, `RiskContext`.
- `echo-core/src/tools/permission.rs` (901 lines) — `ToolPermission`,
  `PermissionMode` (8 variants), `PermissionDecision` (4 variants),
  `RuleMatcher`/`RuleBehavior`/`RuleSource`/`RuleRegistry` (deny-first),
  `PermissionPolicy` trait + `DefaultPermissionPolicy`.
- `src/agent/react/run/pipeline.rs:269-347` — `PermissionStage` (the
  pipeline stage that invokes `snapshot.check_tool_approval`).
- `src/agent/snapshot.rs:795-868, 1144-1180` —
  `AgentRunSnapshot::check_tool_approval` (live per-call check path) and
  `tool_needs_approval` (live batch-planning predicate).
- `src/agent/react/run/approval.rs` (457 lines) — the legacy
  `ReactAgent` HITL impl block (`#[allow(dead_code)]`); confirmed dead
  (see finding F-HITL-01-P3-03).

EKO application adapter (read-only, for boundary classification only):

- `echo-agent-cli/echo-agent-app-core/src/hitl/dispatcher.rs` (167 lines)
  — `HitlDispatcher` (multi-provider fan-out, 5-min shared deadline,
  fail-closed).
- `echo-agent-cli/echo-agent-app-core/src/runtime.rs:127-146` and
  `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:959-975` —
  EKO wiring of `HumanLoopProvider` into framework `ReactAgent`.
- `echo-agent-cli/web-frontend/src/components/chat/ApprovalCard.tsx`,
  `web-frontend/src/lib/permissionModes.ts`, `web-frontend/src/hooks/chatEventHandler.ts`
  — React approval UI (sampled for the integration seam).

## Out Of Scope

Deferred to named task IDs:

- The 16 pipeline stage bodies beyond `PermissionStage` (hooks, audit,
  read-before-edit, skill-permission, output-guard, truncation, trace) —
  cross-cutting concerns; **F-RCT-04** owns the pipeline/batch layer.
- The dead `process_steps` parallel-batch implementation and its
  `ReactAgent::tool_needs_approval` caller — owned by **F-RCT-02-P2-02**;
  this task references it only for the dead-code conclusion
  (F-HITL-01-P3-03).
- Application-layer provider implementations (REPL/TUI/Tauri/channel) —
  **A-CHAT-01** (provider arbitration/timeout/default) and the
  `A-INT-01` / `A-TOOL-01` interactive-tool permission carve-out.
- Frontend DTO/event field-by-field contract — **A-FE-01**.
- Secret redaction in tool output (`snapshot.rs:883-889`,
  `echo_security::contains_secrets`) — **F-SEC-01**.

## Inputs

Required repository documents read:

- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/AGENTS.md` (in full via
  system reminder). Load-bearing sections: product positioning and
  security boundary ("threat model is local", "don't apply web-service
  threat model", "permission_mode controls agent automation only, not
  user-interactive tools"), the framework-vs-application layering gate,
  the "first check if it already exists" rule, dead-code cleanup rule,
  no-panic / UTF-8 safety, and the Claude Code / Codex research rule.
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/REPORTING.md`
  (in full).
- `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/comprehensive-review/templates/task-report.md`
  and `templates/validation-report.md` (in full).

Dependency task reports read:

- `docs/comprehensive-review/zcode-glm/tasks/B-REF-01.md` (in full).
  Establishes the cross-system permission constraint: permission is a
  launch-time mode/flag + isolation boundary, not a runtime approval
  state machine; privileged modes are not restored across resume
  (convergence C4, finding B-REF-01-P3-01). Directs F-HITL-01 to treat
  permission as launch mode + isolation.
- `docs/comprehensive-review/zcode-glm/tasks/F-RCT-04.md` (in full).
  Establishes: `PermissionStage` is stage 6 of the 16-stage
  `ToolExecutionPipeline`; `requires_sequential_execution` routes
  approval-needing tools to the serial batch path via
  `would_request_human_for_permissions` (which must not call the
  handler); the pipeline is the per-call middleware reached via
  `execute_tool_with_policy`. Confirms the permission check is wired
  into the live tool-batch path and that batch planning is a pure
  prediction.

Historical documents treated as hypotheses:

- `echo-orchestration/src/human_loop/service.rs:11-24` doc-comment
  describing the 7-step `check()` pipeline. Treated as **current** —
  verified step-by-step by V01.
- `echo-orchestration/src/human_loop/approval_cache.rs:1-13` doc-comment
  claiming three cache granularities (Once/Session/SessionAllTools) with
  Session = "tool_name + args_hash". Treated as **partially inaccurate**
  — the cache implements it, but the service-layer scope inference never
  selects per-args `Session` for approvals (finding F-HITL-01-P3-01,
  V03).
- `echo-core/src/tools/permission.rs:1-9` doc-comment claiming
  Claude-Code-referenced multi-level permission model. Treated as
  **current** — verified by V01/V04.

## Layering Decision

| Classification | Required answer |
|---|---|
| Generic mechanism | Yes. The entire `echo-core::tools::permission` model (`ToolPermission`, `PermissionMode`, `PermissionDecision`, `RuleRegistry` with deny-first eval, `PermissionPolicy` trait) and the `echo-orchestration::human_loop` service (`PermissionService`, `SessionApprovalCache`, `ProtectedPathChecker`, `Classifier`, `HumanLoopProvider`/`PermissionRequestHandler` traits, `HumanLoopManager`) are generic primitives any `echo-agent` consumer may need. They live correctly: the typed model in `echo-core`; the composing service + provider trait + cache + classifier in `echo-orchestration`. None of these depend on an EKO product decision. |
| EKO product policy | Confined to the application layer. EKO's multi-provider routing (`HitlDispatcher` with 5-min shared deadline and parallel first-responder-wins), its concrete providers (REPL/TUI/Tauri/channel), and the React approval UI are all in `echo-agent-cli`. The framework never references them. EKO selects `PermissionMode`, injects the provider into `ReactAgent::set_human_loop_provider` + `build_permission_service` (`runtime.rs:136-137`, `agent_pool.rs:971`); the framework does not hardcode a mode. |
| Adapter boundary | `DynProviderHandler` (`service.rs:855-908`) is the thin seam: it adapts `dyn HumanLoopProvider` → `PermissionRequestHandler`, performing lossless `HumanLoopResponse` ↔ `PermissionResponse` mapping with no scheduling or state authority. `ReactAgent::build_permission_service` (`mod.rs:1394-1399`) wraps the injected provider in `PermissionService::from_provider`. Both are thin; neither owns approval state. |
| Duplicate search | Searched names across both repos: `PermissionService`, `check_tool_approval`, `check_with_permissions`, `check_with_permissions_in_mode`, `would_request_human_for_permissions`, `tool_needs_approval`, `PermissionStage`, `HumanLoopProvider`, `PermissionRequestHandler`, `SessionApprovalCache`, `ProtectedPathChecker`, `TimeoutStrategy`, `RuleRegistry`, `request_human_approval`. Result: one canonical permission service (`echo-orchestration::human_loop::service::PermissionService`); one canonical per-call check (`AgentRunSnapshot::check_tool_approval`); one canonical batch predictor (`would_request_human_for_permissions`). The second `check_tool_approval` (`ReactAgent` in `approval.rs`) is dead (F-HITL-01-P3-03). No live duplicate. |
| Migration deletion | F-HITL-01-P3-03 proposes deleting the dead `ReactAgent` HITL block in `approval.rs` (coordinate with F-RCT-02-P2-02, which removes the last caller of `ReactAgent::tool_needs_approval`). No other deletion proposed. |

## Current Path

Verified permission check call graph at commit `9b0e0fa` (per tool call,
under `feature = "human-loop"`):

```text
ToolExecutionPipeline::default_pipeline().stages[5] = PermissionStage   [pipeline.rs:269, 943]
   │  (stages run in order; each sees ctx.blocked from prior stage)
   ↓
PermissionStage::run(ctx, snapshot)                                      [pipeline.rs:277]
   │  1. ctx.permission_decision preset? (Allow→pass, Deny→block, Ask/RequireApproval→continue)
   │  2. run permission_request lifecycle hook  → block / mode_override / decision
   │  3. snapshot.check_tool_approval(name, input, mode_override)
   ↓
AgentRunSnapshot::check_tool_approval(name, input, mode_override)        [snapshot.rs:798]
   │  permissions = tool_manager.get_tool(name).permissions()
   │  tokio::select! { cancel_token → Cancelled ; service.check_with_permissions_in_mode(..) }
   ↓
PermissionService::check_with_permissions_in_mode(name, input, perms, mode_override)  [service.rs:484]
   │  effective_mode = mode_override.unwrap_or(config.mode)   ← call-scoped, not written to config
   │  STEP 0  protected_paths.check(name, input)  → Protected? → Deny  (runs even in Bypass!)
   │  STEP 1  effective_mode == BypassPermissions? → (bypass_disabled? → Deny : Allow)
   │  STEP 2  effective_mode == Plan? → Write/Execute/Sensitive → Deny ; else Allow
   │  STEP 4  rules.check(name, perms) → behavior.to_decision()   (deny-first)
   │  STEP 5  cache.is_approved(name, input) → Allow (cache hit)
   │  STEP 6  denial_tracker.should_fallback() → RequireApproval
   │  STEP 5.5 needs_handler && !has_real_handler() → RequireApproval  (fail-open to UI, not silent deny)
   │  STEP 6  mode dispatch:
   │            Auto           → check_with_classifier  (no classifier → RequireApproval)
   │            Default        → confirm_required? → check_with_handler : Allow
   │            AcceptEdits    → confirm_required? (Exec/Net/Sensitive) → handler : Allow
   │            StrictConfirm  → confirm_required? (Write/Exec/Net/Sensitive) → handler : Allow
   │            DontAsk        → Deny (no allow rule matched)
   │            Bubble         → RequireApproval
   │            Plan/Bypass    → (handled above)
   │  STEP 7  Allow → denial_tracker.reset() ; Deny → record_denial()
   │  STEP 8  audit_sink.record(entry)   (fire-and-forget spawn)
   ↓  (only inside check_with_handler, service.rs:707)
check_with_handler(name, input, perms)                                   [service.rs:707]
   │  request = PermissionRequest::new(..).with_permissions(..).with_risk_based_suggestions()
   │  handler = self.request_handler.read().clone()   (RwLock — hot-swappable)
   │  response = handler.handle(request).await?
   │    ↳ DynProviderHandler::handle → HumanLoopProvider::request(approval req)
   │       ↳ EKO HitlDispatcher → GUI/TUI/REPL → user → HumanLoopResponse
   │  updated_input? → last_modified_args side-channel
   │  Allowed? → cache.record_approval(name, input, infer_scope_from_updates(rule_updates))
   │  rule_updates? → apply_updates (AddRule/RemoveRule/SetMode)
   │  map Allowed→Allow, Denied→Deny, NeedMoreInfo→Ask
```

Composing types (the V01 answer):

- **`ToolPermission`** (`echo_core::tools::permission`, 5 variants:
  Read/Write/Network/Execute/Sensitive) — declared per-tool at definition
  time; the input axis to every check.
- **`PermissionMode`** (8 variants: Default/Plan/AcceptEdits/
  BypassPermissions/Auto/Bubble/DontAsk/StrictConfirm) — launch-time
  policy selected by the application; stored in
  `PermissionServiceConfig.mode` and overridable per-call via
  `mode_override` (hook-supplied, not written to config).
- **`PermissionDecision`** (4 variants: Allow/Deny/RequireApproval/Ask)
  — the output of `check_with_permissions_in_mode`; consumed by the
  snapshot path (Allow→proceed, Deny/RequireApproval/Ask→Err→tool
  blocked) when no inline handler resolves it.
- **Composition**: `PermissionMode` × `ToolPermission[]` →
  `RuleRegistry::check` (deny-first) → `SessionApprovalCache` →
  `DenialTracker` → mode dispatch (`Classifier` for Auto,
  `PermissionRequestHandler` for Default/AcceptEdits/StrictConfirm) →
  `PermissionDecision`. `ProtectedPathChecker` is a step-0 override that
  can Deny regardless of mode.

Key invariants verified (full evidence in V01–V04):

- **Protected paths override BypassPermissions.** Step 0 runs before
  step 1 (`service.rs:512-528`); `.git/.ssh/.env/.aws/...` are denied
  even in BypassPermissions mode. Matches Claude Code (B-REF-01 C4).
- **Per-call mode override does not mutate service config.** Verified by
  `call_scoped_mode_override_does_not_mutate_service_mode`
  (`service.rs:1070-1085`); concurrent tool calls cannot leak a
  hook-selected mode.
- **Batch planning never calls the handler.**
  `would_request_human_for_permissions` is pure prediction
  (`service.rs:371-405`); verified by
  `test_batch_prediction_does_not_call_handler` (handler counter stays
  0). This is the F-RCT-04 invariant from the permission side.
- **Drop-without-respond default is Reject (fail-closed).**
  `ApprovalResponder::Drop` sends `Rejected{..}`
  (`mod.rs:168-177`); `HumanLoopManager::request` timeout returns
  `HumanLoopResponse::Timeout` (`mod.rs:424-437`), which both
  `DynProviderHandler` and `DefaultPermissionRequestHandler` map to
  `PermissionResponse::denied` → `PermissionDecision::Deny`
  (`service.rs:879-881`, `permission.rs:678-680`). So timeout/drop =
  Deny. See V02.
- **Deny-first rule evaluation.** `RuleRegistry::check`
  (`permission.rs:437-461`) returns the first matching Deny regardless
  of source priority; Ask and Allow are tracked and returned by highest
  source priority only if no Deny matched. Documented and tested
  (`test_rule_registry_deny_first_ordering`).
- **No handler configured → RequireApproval, not silent deny.**
  `has_real_handler()` gate (`service.rs:250-255, 594-600`) short-circuits
  to `RequireApproval`; `NullPermissionRequestHandler::is_null_handler()`
  returns true. Verified by `test_default_mode_requires_handler_for_execute`.

## Findings

### F-HITL-01-P2-01: `TimeoutStrategy` config is dead — stored and settable but never consulted; effective timeout behavior is hardcoded to Deny regardless of the configured strategy

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-orchestration/src/human_loop/service.rs:84-96` defines
    `TimeoutStrategy { Reject, AutoApprove { reason }, Escalate }`.
  - `service.rs:112` stores it as
    `PermissionServiceConfig.timeout_strategy`; `service.rs:126` defaults
    it to `TimeoutStrategy::Reject`; `service.rs:221-225`
    (`with_timeout_strategy`) and `service.rs:949-950` (builder
    `timeout_strategy`) are setters.
  - Repository-wide grep for non-test, non-comment reads of
    `timeout_strategy` / `TimeoutStrategy` returns **only** those six
    write/define sites in `service.rs` (V02). There is no read in
    `check_with_permissions_in_mode`, `check_with_handler`,
    `HumanLoopManager`, or any provider.
  - The actual timeout behavior is decided one layer below, at the
    provider→handler seam, and is hardcoded: `HumanLoopManager::request`
    returns `HumanLoopResponse::Timeout` on elapsed
    (`mod.rs:424-437`); `DynProviderHandler::handle`
    (`service.rs:879-881`) and `DefaultPermissionRequestHandler::handle`
    (`permission.rs:678-680`) both map `Timeout →
    PermissionResponse::denied(Some("请求超时"))`; `check_with_handler`
    then maps `Denied → PermissionDecision::Deny`. So timeout = Deny,
    independent of `TimeoutStrategy`.
  - `PermissionService` itself never imposes a timeout on the approval
    request: both `check_with_handler` (`service.rs:715-718`) and
    `DynProviderHandler::handle` (`service.rs:864`) build
    `HumanLoopRequest::approval(name, input)`, whose `timeout` field is
    `None` (`mod.rs:628-642`). Any timeout comes from the provider
    implementation (e.g. EKO `HitlDispatcher`'s 5-min shared deadline,
    `dispatcher.rs:101-102`).
- Reachability: definition → `pub` field on a `pub` config struct →
  settable via two `pub` builder methods → never read. Live in the API
  surface, dead at the decision site. Any consumer that calls
  `.with_timeout_strategy(TimeoutStrategy::AutoApprove { .. })` believing
  a timed-out approval will be auto-approved is mistaken — it will be
  denied.
- Expected invariant: a `pub` config field named `timeout_strategy`
  with variants `AutoApprove` and `Escalate` implies those behaviors are
  honored when a timeout occurs. The framework does not honor them.
- Observed behavior: the field is write-only. Effective behavior on
  timeout is always Deny (fail-closed), which happens to match the
  `Reject` default — so the default is correct but the configurability
  is an illusion.
- Impact: misleading public API (REPORTING.md P2 category: "misleading
  public API"). A reviewer or consumer reading the enum cannot infer
  real behavior from it. The `AutoApprove` variant is especially
  dangerous as a silent expectation: a deployment that opts into it
  thinking it fails open on timeout would in fact fail closed. (Note:
  AGENTS.md warns against over-engineering security for the local
  threat model, so the *fixed* Deny-on-timeout is the right call — the
  defect is the unused knob, not the behavior.)
- Root cause: `TimeoutStrategy` was designed as a framework-level
  policy knob, but the timeout actually originates inside the provider
  (which the framework does not own), and the handler-to-decision
  mapping was written independently and hardcoded to Deny. The two
  halves were never connected.
- Direction: pick one.
  (a) **Wire it in (preferred if the knob is real)**: when
  `timeout_strategy != Reject`, have `PermissionService` impose the
  timeout itself by building `HumanLoopRequest::approval_with_timeout`
  (`mod.rs:666-684`) with a service-configured deadline, and on
  `HumanLoopResponse::Timeout` branch on `timeout_strategy`:
  `AutoApprove → Allow` (with audit reason), `Escalate →
  RequireApproval`, `Reject → Deny`. Add a regression test per branch.
  (b) **Delete it (preferred under YAGNI / AGENTS.md dead-code rule)**:
  remove `TimeoutStrategy`, `PermissionServiceConfig.timeout_strategy`,
  `with_timeout_strategy`, and the builder method; document that
  timeout is provider-owned and always maps to Deny at the framework
  seam. This is the smaller change and matches the local-assistant
  positioning (no need for a fail-open knob).
  Prefer (b) unless a concrete consumer requires fail-open-on-timeout.
- Regression validation: under (a), one test per `TimeoutStrategy`
  variant asserting the mapped `PermissionDecision`; under (b), grep
  confirms zero remaining references and `cargo test -p echo_orchestration
  --lib -- human_loop::` stays green.
- Validation reports: [V02-01](../validations/F-HITL-01/V02-01.md),
  [V01-01](../validations/F-HITL-01/V01-01.md).

### F-HITL-01-P3-01: Per-args `ApprovalScope::Session` cache granularity is implemented but unreachable from the service path — session approvals collapse to tool-wide (`SessionAllTools`)

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-orchestration/src/human_loop/approval_cache.rs:6-8, 99-169`
    documents and implements three scopes: `Once` (no cache), `Session`
    (key = `tool_name + args_key`, per-args), `SessionAllTools`
    (key = `tool_name`, ignores args). The `Session` (per-args) path is
    real in `is_approved`/`record_approval`.
  - `echo-orchestration/src/human_loop/service.rs:791-807`
    `infer_scope_from_updates` returns `SessionAllTools` for any
    `AddRule { source=="session", behavior=="allow" }`, and `Session`
    only for `behavior=="deny"`. But `record_approval` is called only
    when the decision is `Allowed` (`service.rs:735-738`). So for an
    allow decision, the inferred scope is either `SessionAllTools` (a
    session allow rule exists) or `Once` (no session rule). The per-args
    `Session` scope is **never selected** for an approval through the
    standard handler path.
  - `echo-orchestration/src/human_loop/permission.rs:536-543`
    `build_matcher` collapses any non-null/non-empty `args` to
    `"{tool_name}(*)"` — so even the RULE installed by an
    `AllowForSession` suggestion is tool-wide, not per-args
    (`permission.rs:499-504`).
  - Net: the user clicking "allow for session" (`Suggestion::
    allow_for_session`) on `Bash({command:"ls"})` installs rule
    `Bash(*)` (allow, session) AND records `SessionAllTools` for `Bash`;
    every subsequent `Bash({command:"rm -rf /"})` hits the cache at
    step 5 and returns `Allow` without re-asking. Confirmed by
    `test_session_all_tools_scope` (`approval_cache.rs:359-369`), which
    explicitly asserts `rm -rf /` is cached after `{}`.
- Reachability: every session-scoped allow approval through
  `PermissionRequestHandler`. This is the default UX path for
  Default/AcceptEdits/StrictConfirm modes when the user chooses
  "allow for session".
- Expected invariant: `ApprovalScope::Session` is documented
  (`policy.rs:23-29`) as "本次会话内，相同工具 + 相同参数不再请求审批"
  (same tool + same args). The cache implements this. But the service
  never selects it, so the documented per-args granularity is not
  delivered.
- Observed behavior: session approval is always tool-wide. This matches
  Claude Code's "allow for session" semantics (B-REF-01 C4) and is
  acceptable for a local dev tool (AGENTS.md: don't over-engineer the
  local threat model). The defect is that the finer-grained scope is
  implemented, documented, and tested at the cache layer but
  circumvented by the service layer — a maintainability and
  truth-in-advertising issue, not a behavior regression.
- Impact: low. Behavior aligns with mature references. Cost is two
  layers of cache granularity that disagree, which will mislead future
  maintainers and any consumer who reads the cache API expecting
  per-args scoping.
- Root cause: `infer_scope_from_updates` keys only off
  `source`/`behavior`, not off whether the matcher is arg-specific; and
  `build_matcher` discards args early. The two were written without a
  shared notion of "per-args vs tool-wide".
- Direction: pick one.
  (a) **Make per-args real**: have `build_matcher` preserve a specific
  args signature (or have `infer_scope_from_updates` return `Session`
  when the matcher is arg-specific and `SessionAllTools` when it is
  bare `tool_name`), and add a test that `Bash(ls)` session-approval
  does not cache `Bash(rm)`.
  (b) **Delete the dead granularity (preferred under YAGNI)**: remove
  `ApprovalScope::Session` (per-args) handling from
  `SessionApprovalCache` and `infer_scope_from_updates`, rename the
  remaining session scope to `Session` (tool-wide), and update the
  doc to state session approval is tool-wide, matching Claude Code.
- Regression validation: `cargo test -p echo_orchestration --lib --
  human_loop::approval_cache` (currently green) and a new test
  asserting the chosen semantics end-to-end through
  `PermissionService::check_with_permissions`.
- Validation reports: [V03-01](../validations/F-HITL-01/V03-01.md).

### F-HITL-01-P3-02: `SessionApprovalCache::sha256_hex` is misnamed — it computes a `DefaultHasher` (SipHash) digest, not SHA-256

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-orchestration/src/human_loop/approval_cache.rs:280-290`:
    ```rust
    fn sha256_hex(input: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        // 使用 DefaultHasher + 十六进制格式作为轻量级哈希方案
        // 避免引入额外依赖；DefaultHasher 在本进程内是确定的
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        let hash = hasher.finish();
        format!("{hash:016x}")
    }
    ```
    The name promises SHA-256 (128-bit hex, 64 chars). The body
    computes `DefaultHasher` (SipHash 1-3, 64-bit) formatted as 16-char
    hex. The body comment documents the substitution, but the method
    name does not.
- Reachability: only active when `SessionApprovalCache::hash_args` is
  true (`with_hash_args()` / `set_hash_args(true)`). The default
  (`new()`, `with_ttl()`) uses raw JSON as the key, so this path is opt-
  in. No live caller in `echo-agent`/`echo-agent-cli` sets
  `hash_args = true` (V03 grep).
- Expected invariant: a function named `sha256_hex` computes SHA-256.
- Observed behavior: it computes SipHash. The 64-bit output space has
  materially higher collision probability than SHA-256's 128-bit space
  (though still negligible at session-cache volumes).
- Impact: low. For an in-memory, session-scoped, non-persisted cache,
  SipHash is functionally fine and `DefaultHasher` avoids a dependency.
  The cost is a naming trap: a future maintainer who reads the method
  name may rely on SHA-256's collision/stability properties, or may
  hesitate to persist the key believing SHA-256 is stable across Rust
  versions (it is not guaranteed for `DefaultHasher`).
- Root cause: deliberate dependency avoidance, but the name was not
  updated to match the chosen algorithm.
- Direction: rename to `args_hash_hex` (or `siphash_hex`) and update
  the doc-comment to state the algorithm and why it is acceptable for
  an in-memory session cache (not stable across Rust versions, do not
  persist). Alternatively, if genuine SHA-256 is wanted, pull `sha2`
  (likely already transitively available). Cheapest: rename.
- Regression validation: `cargo test -p echo_orchestration --lib --
  human_loop::approval_cache`.
- Validation reports: [V03-01](../validations/F-HITL-01/V03-01.md).

### F-HITL-01-P3-03: The `ReactAgent` HITL approval impl block in `src/agent/react/run/approval.rs` is dead code — superseded by the `AgentRunSnapshot` + `PermissionService` path

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence:
  - `echo-agent/src/agent/react/run/approval.rs:11` annotates the entire
    `impl ReactAgent` block with `#[allow(dead_code)]`.
  - Repository-wide grep for callers of `check_tool_approval`
    (excluding the definitions and tests) returns exactly two sites,
    both in `pipeline.rs:330` and `pipeline.rs:336`, and both call
    `snapshot.check_tool_approval(..)` — the 3-argument
    `AgentRunSnapshot` method (`snapshot.rs:798`), not the 2-argument
    `ReactAgent` method (`approval.rs:65`). The `ReactAgent` method has
    no external caller.
  - The methods `request_human_approval` (`approval.rs:295`),
    `handle_ask_decision` (`approval.rs:229`), and
    `flush_pending_permission_rules` are referenced only *inside* the
    dead `ReactAgent::check_tool_approval` (`approval.rs:123, 161, 164`).
  - `ReactAgent::tool_needs_approval` (`approval.rs:17`) has a single
    caller at `react_loop.rs:223`, which is inside the dead
    `process_steps` (owned by F-RCT-02-P2-02). So it is live only via a
    dead caller.
  - The live per-call path is `PermissionStage` (`pipeline.rs:329`) →
    `AgentRunSnapshot::check_tool_approval` (`snapshot.rs:798`) →
    `PermissionService::check_with_permissions_in_mode`, with the
    injected handler resolving approvals inline. The live batch
    predicate is `AgentRunSnapshot::tool_needs_approval`
    (`snapshot.rs:1152`), called from `phases/tools.rs:32`.
- Reachability: definition → `#[allow(dead_code)]` → no live caller.
- Expected invariant: there should be one authoritative per-call
  approval path (AGENTS.md: "same semantics can only have one
  authoritative implementation"). There are two `check_tool_approval`
  methods with divergent semantics (the snapshot one takes a mode
  override and errors on `RequireApproval`; the ReactAgent one runs
  hooks, calls `request_human_approval`, and resolves `RequireApproval`
  interactively). The ReactAgent one is dead; leaving it breeds
  confusion about which is authoritative.
- Observed behavior: the dead block compiles (via the allow), shadows
  the live path in greps, and its `request_human_approval`/`handle_ask_decision`
  logic gives a false impression of an interactive flow that no longer
  runs.
- Impact: low (no runtime effect). Cost is maintenance burden and the
  two-implementation confusion flagged by AGENTS.md's "no parallel
  implementation of the same semantics" rule.
- Root cause: the HITL path was migrated from `ReactAgent` to
  `AgentRunSnapshot` + `PermissionService`, but the old impl was
  retained under `#[allow(dead_code)]` instead of deleted. AGENTS.md's
  code-cleanup rule ("delete superseded code, don't leave dead paths")
  was not applied.
- Direction: delete the `impl ReactAgent { ... }` block in
  `approval.rs:11-456` after confirming `log_permission_denied`
  (`approval.rs:190`) has no live caller outside the block (grep shows
  it is called only within the dead methods; if so, delete it too).
  `ReactAgent::tool_needs_approval` should be removed together with
  F-RCT-02-P2-02's removal of `process_steps` (its only caller). Verify
  `cargo test --workspace --all-features` stays green and that the
  `#[allow(dead_code)]` attribute disappears with the block.
- Regression validation: `cargo build -p echo_agent --features
  human-loop`; `cargo test --workspace --all-features --locked`; grep
  confirms zero remaining `ReactAgent::check_tool_approval`/
  `request_human_approval` references.
- Validation reports: [V01-01](../validations/F-HITL-01/V01-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Decision/policy/provider mapping: `PermissionMode` × `ToolPermission` → `PermissionDecision` composition; single live check path; duplicate search | yes | passed | [V01-01](../validations/F-HITL-01/V01-01.md) |
| V02 | Timeout/default behavior: `TimeoutStrategy` is write-only; effective timeout → Deny; drop-without-respond → Reject | yes | passed | [V02-01](../validations/F-HITL-01/V02-01.md) |
| V03 | Approval cache identity: 3 scopes; `Session` (per-args) unreachable for approvals; `sha256_hex` misnaming; cross-context scoping | yes | passed | [V03-01](../validations/F-HITL-01/V03-01.md) |
| V04 | Local-vs-generic boundary: framework primitives vs EKO adapter; `ProtectedPathChecker` overrides Bypass; permission modes gate agent automation only | yes | passed | [V04-01](../validations/F-HITL-01/V04-01.md) |
| V05 | Historical-document drift check | conditional | n/a | No prior F-HITL-01 report exists in this reviewer directory; the three docstrings treated as hypotheses are classified inline in the Inputs section (two current, one partially inaccurate → F-HITL-01-P3-01). |

Executed cargo command (exit 0):

```text
cargo test -p echo_orchestration --lib -- human_loop::     (119 passed)
```

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| AGENTS.md: "permission_mode controls agent automation only, not user-interactive tools (terminal, file picker)" | current (supported) | V04 confirms the permission check runs only inside `PermissionStage` on agent tool calls; interactive terminal/file picker are direct UI actions that never enter the tool pipeline. The historical `require_full_auto` gates on `create_terminal`/`connect_mcp_server` are gone (no `require_full_auto` reference in the permission path). |
| AGENTS.md (via B-REF-01): "permission is launch mode + isolation, not a runtime approval state machine" | current (supported) | V01/V04 confirm `PermissionMode` is a launch-time config (or call-scoped override) and `PermissionDecision` is a per-call result; no approval state lives in the run state machine. The approval "state" is the in-flight `oneshot` + optional `SessionApprovalCache` entry, both cache/artifact-shaped, not run-state columns. Matches B-REF-01 C4. |
| `echo-orchestration/src/human_loop/service.rs:11-24` — the 7-step `check()` pipeline doc | current | V01 walks all 8 steps (step 0 protected-paths + the documented 7) in `check_with_permissions_in_mode` (`service.rs:484-681`). |
| `echo-orchestration/src/human_loop/approval_cache.rs:6-8` — "Session = tool_name + args_hash (per-args)" | partially inaccurate | V03 confirms the cache implements per-args `Session`, but `infer_scope_from_updates` never selects it for approvals. Finding F-HITL-01-P3-01. |
| `echo-core/src/tools/permission.rs:4-9` — "multi-level permission model referenced from Claude Code" | current | V01/V04 confirm the mode/rule/decision/provider layering matches the Claude Code reference (B-REF-01 V01-02 A,E; C4). |
| B-REF-01 handoff — "F-HITL-01 should treat permission as launch mode + isolation" | current (adopted) | V01/V04 frame the model as launch mode + isolation boundary; the approval loop is provider-owned, framework-agnostic. |

## Coverage And Uncertainty

Inspected in full: every `.rs` under `echo-orchestration/src/human_loop/`
except `audit.rs`, `batch.rs`, `console.rs`, `webhook.rs`,
`websocket.rs`, `pattern.rs` (these were read enough to confirm their
pub surface via `mod.rs` exports but not line-by-line); the full
`echo-core/src/tools/permission.rs`; `pipeline.rs:269-347` (PermissionStage
and SkillPermissionStage); `snapshot.rs:795-868, 1144-1230`; the full
`approval.rs`; the EKO `hitl/dispatcher.rs`.

Not inspected (out of scope or deferred):

- `audit.rs` internals (the `PermissionAuditSink` trait is consumed by
  `service.rs:427-452` via fire-and-forget `tokio::spawn` — confirmed
  non-blocking; the sink implementations are not load-bearing for the
  permission *decision*). Belongs to a logging/audit task.
- `batch.rs` (`BatchApprovalProvider`) and `console.rs`/
  `webhook.rs`/`websocket.rs` providers — alternate provider
  implementations; their correctness is not the framework permission
  model. The provider trait contract is what matters and it is
  inspected.
- Application-layer provider implementations (REPL/TUI/Tauri/channel) —
  A-CHAT-01.
- Whether the primary agent created in `runtime.rs` is reused across
  multiple GUI conversations (which would make its
  `SessionApprovalCache` accumulate across conversations). The agent
  pool creates per-conversation agents (`agent_pool.rs:953-977`), but
  the primary agent's reuse pattern is an application concern; if
  reused, "approve for session" on one conversation could auto-approve
  another's same-tool calls. This is a potential cross-context cache
  consideration for an A-* task, not a framework defect.

Environmental constraints:

- `cargo test -p echo_orchestration --lib -- human_loop::` ran against
  the existing incremental cache; 119 passed, 0 failed. The
  `human-loop` feature is on the root `echo_agent` package
  (`Cargo.toml:73`), not on `echo_orchestration`, so the test used
  `echo_orchestration`'s default features (the `human_loop` module is
  compiled unconditionally; only the `websocket` provider sub-module is
  gated by the `websocket` feature re-exported via `human-loop`).
- No `cargo clean` was needed (disk pressure well below threshold).

Uncertain claims:

- Whether any third-party `echo-agent` consumer or EKO currently calls
  `with_timeout_strategy` expecting it to take effect. The grep shows
  no caller in `echo-agent`/`echo-agent-cli`, but external consumers
  cannot be ruled out. The defect (write-only field) is real
  regardless; the severity is bounded by the lack of in-repo callers.
- Whether `ApprovalScope::Session` (per-args) was ever intended to be
  reachable. Its presence in the cache, the doc, and the tests suggests
  yes, but the service layer may have deliberately collapsed it. The
  finding is "dead/misleading granularity"; the resolution (make-it-real
  vs delete) is a product call. F-HITL-01-P3-01 offers both.

## Handoff

Conclusions downstream tasks may rely on:

1. **Generic primitives, clean boundary (the primary answer).** The
   framework provides a complete, generic automated-action approval +
   protected-path stack in `echo-core` (typed model) and
   `echo-orchestration` (composing service), without imposing EKO
   interaction policy. EKO policy is confined to
   `echo-agent-cli/echo-agent-app-core/src/hitl/` and the React UI.
   Downstream tasks can treat
   `PermissionService::check_with_permissions_in_mode` as the single
   authoritative permission entry point and `HumanLoopProvider`/
   `PermissionRequestHandler` as the extension seam.
2. **Protected paths override BypassPermissions.** Step 0 of the
   pipeline (`service.rs:512-528`) denies `.git/.ssh/.env/.aws/...`
   before the BypassPermissions short-circuit. Any task reasoning about
   "what bypass actually bypasses" must account for this: bypass skips
   mode/rules/cache/handler, NOT protected paths.
3. **Per-call mode override is non-mutating.** Hooks can supply a
   `permission_mode_override` per call without racing concurrent tool
   calls. Tasks designing hook-driven permission (e.g. plan-mode
   toggling) can rely on this.
4. **Timeout/drop = Deny (fail-closed), always.** Regardless of the
   (dead) `TimeoutStrategy` knob. Tasks must not assume fail-open on
   timeout.
5. **Session approval is tool-wide in practice.** "Allow for session"
   caches approval for all args of that tool (matches Claude Code). The
   per-args `Session` scope is dead in the service path
   (F-HITL-01-P3-01). Tasks must not assume per-args session caching.
6. **One dead approval implementation.** The `ReactAgent` HITL block in
   `approval.rs` is dead; the live path is `snapshot.check_tool_approval`
   → `PermissionService`. Tasks touching approval code should edit the
   snapshot/service path, not the `ReactAgent` block.

Reports they must read:

- This report (F-HITL-01) for the permission-model composition and the
  four findings.
- `tasks/B-REF-01.md` (V06-01 C4, finding P3-01) for the cross-system
  constraint that permission is launch mode + isolation.
- `tasks/F-RCT-04.md` for how `PermissionStage` sits in the tool-batch
  pipeline and why `would_request_human_for_permissions` must not call
  the handler.
- `tasks/F-RCT-02.md` (P2-02) for the dead `process_steps` that is the
  last caller of `ReactAgent::tool_needs_approval` (coordinate deletion
  with F-HITL-01-P3-03).
- `validations/F-HITL-01/V01-01.md` through `V04-01.md` for per-claim
  evidence.

Conditions that make this report stale:

- Any change to the step ordering in
  `check_with_permissions_in_mode` (especially the protected-paths-
  before-bypass ordering) invalidates V01/V04.
- Wiring `timeout_strategy` into the decision path (fixing F-HITL-01-
  P2-01) invalidates V02's "write-only" claim.
- Changing `infer_scope_from_updates` or `build_matcher` to deliver
  per-args `Session` scope (fixing F-HITL-01-P3-01) invalidates V03's
  "unreachable" claim.
- Deleting the dead `ReactAgent` HITL block (fixing F-HITL-01-P3-03)
  invalidates V01's duplicate-search dead-code note.
- Adding a `require_full_auto`-style gate on user-interactive tools
  would regress the AGENTS.md security-boundary claim in V04.

Follow-up task IDs (no fixes implemented in this review):

- A **framework robustness/cleanup task** should action F-HITL-01-P2-01
  (wire-or-delete `TimeoutStrategy`) — highest value, as it removes a
  misleading public API. Same task can pick up F-HITL-01-P3-02
  (`sha256_hex` rename) and F-HITL-01-P3-01 (session-scope resolution).
- A **dead-code removal task** (coordinated with F-RCT-02-P2-02) should
  delete the `ReactAgent` HITL block in `approval.rs`
  (F-HITL-01-P3-03).
- **A-CHAT-01** should confirm whether the primary agent is reused
  across GUI conversations and, if so, whether per-conversation cache
  isolation is needed (the cross-context cache uncertainty above).
