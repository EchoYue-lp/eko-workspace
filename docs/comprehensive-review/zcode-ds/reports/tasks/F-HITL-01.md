# F-HITL-01: Human loop and permission model

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both source repositories clean

## Question

Does the framework provide generic automated-action approval and
protected-path primitives without imposing EKO interaction policy?

Answer: the decision/policy/provider separation is architecturally sound and
EKO-neutral (V03-01), but the live execution path does not honor the
`RequireApproval`/`Ask` contract as "ask the human" (P1-01), silently drops
user-modified tool arguments (P1-02), and widens approval scopes beyond their
documented granularity on the live EKO-used bridge (P1-03). Several parallel,
mostly dead approval implementations remain from the pre-refactor era
(P2-03) and are the only code that implements the intended behavior.

## Scope

- `echo-orchestration/src/human_loop/` — full read: `mod.rs` (manager,
  events, responders, provider/handler traits), `permission.rs`
  (PermissionRequest/Response/handler + bridge), `policy.rs`, `service.rs`
  (pipeline, TimeoutStrategy, builder, DynProviderHandler),
  `approval_cache.rs`, `protected.rs`, `pattern.rs`, `batch.rs`,
  `classifier.rs`, `console.rs`, `webhook.rs`, `audit.rs`, `websocket.rs`
  (skims where noted).
- `echo-core/src/tools/permission.rs` — full read (PermissionMode/Decision/
  RuleMatcher/RuleBehavior/RuleRegistry/PermissionPolicy).
- React approval integration — `src/agent/react/run/approval.rs` (full),
  `src/agent/react/subsystems/approval.rs`, `src/agent/snapshot.rs`
  (`check_tool_approval` :798-868, `tool_needs_approval` :1151-1180,
  `execute_tool_with_policy` :1189-1279), `src/agent/react/run/pipeline.rs`
  `PermissionStage` :268-347, `src/agent/react/run/phases/tools.rs:32`
  (batch split), `src/agent/react/run/react_loop.rs:177-502` (dead
  `process_steps`), `src/agent/react/mod.rs` (provider/service wiring
  :439, :531-538, :1386-1400, :1580-1630), `src/agent/react/capabilities.rs`
  (`add_need_appeal_tool` :88-129), `src/agent/react/run/context.rs`
  (`flush_pending_permission_rules` :140-157), `src/tools/builtin/human_in_loop.rs`,
  `src/tools/permission.rs` (re-export), `src/lib.rs` re-exports,
  `src/config.rs` + `src/agent/config.rs` (enable_human_in_loop),
  `examples/demo05_compressor.rs` (:150-230), `docs/zh/05-human-loop.md`.
- EKO boundary only: `echo-agent-cli/echo-agent-app-core/src/runtime.rs:120-145`
  (provider/service wiring), `hitl/` module inventory (dispatcher timeout
  policy noted for boundary classification), `src/tauri/error.rs`
  (`IpcAuth::require_full_auto`), `web-frontend/src/components/chat/ApprovalCard.tsx`
  + `types/api.ts` (approval_request event consumption).
- Executed tests: `cargo test -p echo_orchestration --lib --locked human_loop`
  (119 passed), `cargo test -p echo_agent --lib --locked "permission"` (7
  passed), `cargo test -p echo_core --lib --locked "permission"` (23 passed),
  `cargo test -p echo_agent --lib --locked "react::run::approval"` (0 tests),
  `cargo test -p echo_agent --lib --locked "pipeline"` (14 passed).

## Out Of Scope

- EKO multi-surface approval arbitration (HitlDispatcher provider selection,
  per-surface rendering, GUI/TUI/channel providers, the 5-minute shared
  deadline as product policy) → A-HITL-01.
- EKO permission-mode defaults and `IpcAuth::require_full_auto` IPC gating →
  A-HITL-01, A-INT-01, X-AUT-01 (over-gating search).
- Frontend approval store/rendering correctness → A-SRF-03, A-FE-01/02.
- Subagent Bubble semantics end-to-end → F-SUB-01/F-SUB-02 (the framework
  side is covered here: the mode degrades to `RequireApproval`).
- Guard system, sandbox, secrets → F-SEC-01 (only the protected-path and
  permission boundary interplay is referenced).

## Inputs

- Root `AGENTS.md` (threat model, over-gating lesson, HumanGate cleanup
  promise, UTF-8/panic safety, layering, one-authority), shared `README.md`,
  `REPORTING.md`, `TASKS.md` (F-HITL-01 card), `zcode-ds/README.md`,
  report templates.
- Dependency task reports read: zcode-ds `F-RCT-04` (batch serial split by
  approval; snapshot `execute_tool_with_policy`; `tool_needs_approval`
  anchor) and `B-REF-01` (convergence matrix: sandbox/approval separation,
  subagents excluded from interactive approval where possible, plans as
  artifacts — no run-level approval state machines). Both cross-referenced,
  not re-audited.
- Historical documents treated as hypotheses: root `AGENTS.md` HumanGate
  promise, `docs/zh/05-human-loop.md`, `echo-agent-cli/docs/MASTER-PLAN.md:13`
  — classified in Historical Claim Status.

## Layering Decision

- Generic mechanism (framework, correctly placed — no movement recommended):
  `PermissionMode`/`PermissionDecision`/`RuleRegistry`/`RuleMatcher`/
  `PermissionRule` (echo-core/src/tools/permission.rs), the unified
  `PermissionService` pipeline (rules → cache → denial → handler/classifier),
  `SessionApprovalCache`, `ProtectedPathChecker`, `DenialTracker`/classifiers,
  audit sinks, `HumanLoopManager` + `ConsoleHumanLoopProvider` +
  `WebhookHumanLoopProvider` + `WebSocketHumanLoopProvider`, `HumanLoopHandler`,
  `BatchApprovalProvider`, the `human_in_loop` builtin tool. The framework's
  only enforcement point is `PermissionStage` inside the agent tool pipeline
  (pipeline.rs:268-346) — it gates agent automation, never direct user
  interactions, and contains no EKO-specific gate (V01: zero
  `require_full_auto`-style gates in echo-agent). The AGENTS.md "permission
  modes apply only to agent auto paths" invariant holds at the framework
  level.
- EKO product policy (application, owned by A-HITL-01): provider
  arbitration (`HitlDispatcher` broadcast/first-response-wins, single
  5-minute shared deadline, fail-closed rejection), TUI/GUI/channel/REPL
  providers, `IpcAuth::require_full_auto` (EKO Tauri IPC only),
  `permission_mode` state wiring, frontend `ApprovalCard`.
- Adapter boundary: `DynProviderHandler` (service.rs:855-908, the bridge EKO
  uses via `build_permission_service` → `from_provider`) and
  `DefaultPermissionRequestHandler` (permission.rs:643-689) are thin
  conversions BUT disagree with each other on `ApprovalScope::SessionAllTools`
  → rule mapping, and `infer_scope_from_updates` collapses the args-keyed
  granularity (P1-03). Boundary defect, not EKO policy.
- Duplicate-search terms and results (see V01-01): `HumanGate` (zero in
  code), `PermissionPolicy`/`permission_policy` (zero runtime callers),
  `tool_needs_approval` (2 defs, 1 live), `check_tool_approval` (3 defs,
  1 live), `request_human_approval`/`handle_ask_decision` (dead),
  `take_modified_args` (1 caller, dead), `flush_pending_permission_rules`
  (dead callers only), `timeout_strategy`/`enable_classifier` (written,
  never read), `PermissionServiceBuilder` (dead, drops cache TTL),
  `PermissionMode::Bubble` (no bubbling implementation), `requires_confirmation`
  (no production callers), `approval` across echo-agent-cli (providers only —
  no parallel decision engine).

## Current Path

Verified call graph (V02-01):

1. Every tool call on the live path (`run_stream_channel` → `run_core_loop`
   → `run_tools`, phases/tools.rs:50) executes through the 15-stage
   `ToolExecutionPipeline`; `PermissionStage` (pipeline.rs:268-346) runs the
   permission hook, then calls `snapshot.check_tool_approval`
   (snapshot.rs:798-857) → `PermissionService::check_with_permissions_in_mode`
   (service.rs:484-681): protected path (deny, all modes) → bypass → plan →
   rules (deny-first, allow/ask) → cache → denial-tracker → no-handler →
   mode dispatch (`Auto` classifier / `Default`/`AcceptEdits`/`StrictConfirm`
   handler / `DontAsk` deny / `Bubble` RequireApproval) → post-processing +
   audit. `Allow` → tool runs; `Deny` → blocked; `RequireApproval`/`Ask` →
   `Err("… requires user approval")` (snapshot.rs:841-852) — **no human
   request is made on the live path**.
2. The only human ask on the live path happens inside the service's
   `check_with_handler` (service.rs:707-754), reachable only in
   Default/AcceptEdits/StrictConfirm when the permission set requires
   confirmation and a real handler is configured (EKO satisfies this:
   runtime.rs:130-141 wires `HitlDispatcher` + `build_permission_service`).
   Handler responses write `last_modified_args` (service.rs:729-731) and the
   approval cache (service.rs:735-738) — but the live consumer never calls
   `take_modified_args` (only caller is the dead run/approval.rs:143).
3. Batch split (phases/tools.rs:32) calls the live `snapshot.tool_needs_approval`
   → `would_request_human_for_permissions` (service.rs:371-405; rules + mode
   only, no cache, no handler — conservative).
4. `add_need_appeal_tool` (capabilities.rs:88-129) buffers an `Ask` session
   rule into `pending_permission_rules`; the flush
   (`flush_pending_permission_rules`, context.rs:140-157) is invoked only by
   the dead ReactAgent code (run/approval.rs:20/:123) and a test — the live
   path never installs the rule.
5. Dead branch: `ReactAgent::tool_needs_approval`/`check_tool_approval`/
   `request_human_approval`/`handle_ask_decision` (run/approval.rs, whole
   file `#[allow(dead_code)]`) — used only by the uncalled `process_steps`
   (react_loop.rs:177-502, :223). This is the ONLY code that maps
   `RequireApproval` to a real provider request, records scope into cache via
   `take_modified_args`, and handles `ModifiedArgs` end-to-end.

## Findings

### F-HITL-01-P1-01: The live tool-approval path cannot ask the human — `RequireApproval`/`Ask` decisions become opaque tool errors; the only implementation that requests human approval (run/approval.rs) is dead code; Bubble mode's documented "bubble up to parent" semantics are unimplemented

- Priority: P1
- Confidence: high
- Layer: framework (`AgentRunSnapshot::check_tool_approval` consumer +
  `PermissionService` mode dispatch)
- Evidence: `src/agent/snapshot.rs:841-852` (`RequireApproval`/`Ask` →
  `Err("Tool '{}' requires user approval")`, no provider call);
  `src/agent/react/run/pipeline.rs:268-346` (`PermissionStage` is the live
  consumer); the ask-capable implementation lives only in the dead file
  `src/agent/react/run/approval.rs:159-165` (RequireApproval →
  `request_human_approval` :295-455) whose only caller is the uncalled
  `process_steps` (`react_loop.rs:177-502`, `#[allow(dead_code)]`);
  `service.rs:643` (Bubble → RequireApproval), `service.rs:575-585`
  (denial-tracker fallback → RequireApproval), `service.rs:700-703`
  (Auto without classifier → RequireApproval), `service.rs:561-568` (Ask
  rules → `Ask` decision returned before any handler call);
  `echo-core/src/tools/permission.rs:165-171` (`RequireApproval|Ask =
  requires user approval` contract).
- Reachability: definition → registration (pipeline default composition,
  F-RCT-01) → live caller: any tool call whose service decision is
  `RequireApproval`/`Ask`. With EKO's real-handler wiring these arise from:
  (a) any Ask rule — the exact mechanism `add_need_appeal_tool` creates;
  (b) Bubble mode (documented for subagents); (c) denial-tracker fallback
  after 3 consecutive denials; (d) Auto mode without classifier. With no
  handler configured, every write/execute tool errors this way.
- Expected invariant: `PermissionDecision::RequireApproval`/`Ask` result in a
  human approval request (the enum's own contract, echo-core:165-171, and
  the RuleBehavior::Ask doc "Require user confirmation", echo-core:315-316).
- Observed behavior: on the live path the tool call fails with an opaque
  error; the human is never asked; the only code that asks is dead.
- Impact: Ask rules (`add_need_appeal_tool`) and Bubble mode silently
  degrade to hard tool failures — the model sees an error, no prompt appears,
  the turn loses the tool's work. The denial-tracker fallback ("升级为人工
  审批", service.rs:583) ends the tool instead of escalating to a human.
  EKO's main path (Default + handler) is unaffected, which is why this has
  shipped unnoticed.
- Root cause: during the streaming refactor the ask was moved inside the
  service's `check_with_handler` (mode dispatch), and the live consumer's
  `RequireApproval`/`Ask` arms were implemented as errors without porting the
  asking behavior from the pre-refactor ReactAgent code; the old
  implementation was left as dead code instead of being deleted.
- Direction: in `AgentRunSnapshot::check_tool_approval` (snapshot.rs:798),
  map `RequireApproval`/`Ask` to a provider request via the agent's
  `approval_provider` (mirror `request_human_approval` semantics incl. audit,
  scope recording, modified args — see P1-02), or route these decisions
  through the configured handler in all modes; document that Bubble's
  parent-bubbling requires EKO wiring or implement it; delete
  `run/approval.rs` together with `process_steps` (F-RCT-02-P3-01) only after
  porting.
- Regression validation: live-path test where the service returns
  `RequireApproval` (Bubble mode / no handler / Ask rule) → the approval
  provider receives an `ApprovalRequest` event; tool executes on Approved,
  is blocked on Rejected; `cargo test -p echo_orchestration --lib --locked
  human_loop` stays green.
- Validation reports: [V01-01](../validations/F-HITL-01/V01-01.md),
  [V02-01](../validations/F-HITL-01/V02-01.md),
  [V03-01](../validations/F-HITL-01/V03-01.md)

### F-HITL-01-P1-02: User-modified tool arguments are silently discarded on the live path — the modified args are stored in the service's side channel but only the dead code reads them, so the tool executes with the ORIGINAL arguments

- Priority: P1
- Confidence: high
- Layer: framework (snapshot.rs consumer + service side channel)
- Evidence: `service.rs:729-731` (handler response `updated_input` →
  `last_modified_args`); the only reader `take_modified_args` is called
  solely at `run/approval.rs:143` (dead); the live
  `AgentRunSnapshot::check_tool_approval` returns `Ok(None)` on `Allow`
  (snapshot.rs:834) and never reads the side channel; `PermissionStage`
  applies only the returned `Option` (pipeline.rs:340-344); the dead
  `request_human_approval` returns `Ok(Some(args))` for `ModifiedArgs`
  (run/approval.rs:418-421). EKO has no other reader (grep, V01-01).
- Reachability: definition → registration → live caller: any approval where
  the user edits the tool input before approving (console provider 'e' key,
  console.rs:78/116-171; EKO providers returning `ModifiedArgs`).
- Expected invariant: `check_tool_approval`'s documented contract — "Returns
  modified input if approval modified the tool call" (snapshot.rs:795-796) —
  and the ModifiedArgs feature semantics: the edited input executes.
- Observed behavior: the modified args are written to the side channel and
  never consumed; the tool runs with the original arguments.
- Impact: a user who edits a dangerous command during approval (removing a
  destructive flag, changing a path) believes the edited version executes —
  the original executes instead. Directly contradicts the AGENTS.md
  local data-loss protection intent and the ModifyInput suggestion feature
  (permission.rs:167, 520-525).
- Root cause: the side-channel mechanism was wired to the pre-refactor
  (dead) approval entry point; the live consumer was rewritten without it.
- Direction: in the live `check_tool_approval` `Allow` arm
  (snapshot.rs:834-835), call `service.take_modified_args().await` and return
  it as `Ok(Some(modified))` (PermissionStage already applies it);
  add a live-path test with a handler returning `updated_input`; remove the
  side channel if EKO confirms it uses the framework path only.
- Regression validation: mocked handler returning `PermissionResponse {
  decision: Allowed, updated_input: Some({"cmd": "safe"}) }` → executed tool
  receives `{"cmd": "safe"}`; console-provider edit flow end-to-end.
- Validation reports: [V01-01](../validations/F-HITL-01/V01-01.md),
  [V02-01](../validations/F-HITL-01/V02-01.md),
  [V04-02](../validations/F-HITL-01/V04-02.md)

### F-HITL-01-P1-03: Approval scope is widened beyond its documented granularity — session approvals collapse to per-tool global cache entries, and the live EKO-used bridge converts `SessionAllTools` into a session-wide "*" allow rule covering ALL tools

- Priority: P1
- Confidence: high
- Layer: framework (service.rs inference + DynProviderHandler bridge)
- Evidence: `infer_scope_from_updates` (service.rs:791-807) returns
  `SessionAllTools` for ANY session-source allow rule — the args-keyed
  `Session` cache branch (approval_cache.rs:133-150) is unreachable through
  the service flow (record path service.rs:735-738); `DynProviderHandler::response_with_scope`
  (service.rs:892-908) maps `SessionAllTools` → `add_session_rule("*")`
  while `DefaultPermissionRequestHandler::permission_response_with_scope`
  (permission.rs:706-711) maps it → `add_session_rule(tool_name)`; the
  documented semantics are per-tool ("该工具的所有调用",
  policy.rs:26-28; "相同工具 + 相同参数", policy.rs:24-25; approval_cache.rs:6);
  `RuleMatcher::Pattern("*")` matches every tool (echo-core/src/tools/permission.rs:253-255)
  and rules are evaluated before the cache (service.rs:561-573);
  `build_matcher` never emits `tool_name(key:*)`-specific matchers
  (permission.rs:536-543).
- Reachability: definition → registration → live caller: any consumer
  responding `ApprovedWithScope{Session}` or `{SessionAllTools}` through the
  live bridge — the framework's own console provider produces both ('s'/'a'
  keys, console.rs:66-77), and EKO's GUI/TUI/channel providers use the same
  framework types; the wildcard path needs no EKO involvement at all.
- Expected invariant: `Session` = same tool + same args only; `SessionAllTools`
  = same tool, any args; both bridges install equivalent rules; cache and
  rules agree with the user's chosen scope.
- Observed behavior: "allow for this session" on one (tool, args) auto-approves
  every argument set of that tool for the session (cache global entry);
  "approve all" installs a wildcard allow rule covering every tool, bypassing
  cache, denial tracker and handler for the rest of the session.
- Impact: for a local assistant, a single approval can silently remove the
  approval gate for a whole tool family or for ALL tools (any destructive
  command) for the session — a material widening of the permission boundary
  the user believes they configured; the two framework bridges also disagree,
  so behavior depends on which constructor EKO happens to use.
- Root cause: scope was modeled twice (cache granularity vs rule matcher) and
  the service infers cache scope from rule_updates text instead of carrying
  the actual `ApprovalScope` from the response; the live bridge was written
  with a different (broader) mapping than the older bridge.
- Direction: carry the response's scope through `check_with_handler` and
  record cache entries with the true scope; make `response_with_scope` use
  `build_matcher`-style tool-scoped rules (drop the "*" mapping); make
  `infer_scope_from_updates` inspect the matcher (`tool_name` vs `*`);
  add service-level tests asserting the three granularities end-to-end.
- Regression validation: service test where a session-allow response with
  matcher `"Bash"` is followed by `Bash` calls with different args — the
  different-args call must still ask (currently auto-approved); a
  `SessionAllTools` response on `Bash` must NOT allow `Read`/`agent_tool`
  (currently allowed via "*").
- Validation reports: [V01-01](../validations/F-HITL-01/V01-01.md),
  [V03-03](../validations/F-HITL-01/V03-03.md)

### F-HITL-01-P2-01: `TimeoutStrategy` (and `enable_classifier`) are dead configuration — the documented default `Reject` is never applied; the framework default approval has no timeout (unbounded wait; only responder-drop yields the implicit Reject), and the console provider reports timeout as an `Err` instead of the `Timeout` variant

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `timeout_strategy` written at service.rs:112/:126/:221-226/:949-950,
  never read (V01-01); `enable_classifier` written at :110/:125/:944-945,
  never read (`check_with_classifier` keys on `Option<Arc<dyn Classifier>>`,
  service.rs:689); `HumanLoopRequest::approval` sets `timeout: None`
  (mod.rs:638) and both bridges forward it unset (permission.rs:660,
  service.rs:864); `HumanLoopManager` waits indefinitely when None
  (mod.rs:433-437); console provider blocks on stdin indefinitely
  (console.rs:287-288) and on timeout returns `Err("Approval timeout")`
  (console.rs:275-289) rather than `HumanLoopResponse::Timeout`;
  responder Drop → `Rejected { "No response provided" }` (mod.rs:168-177).
  EKO imposes its own 5-minute deadline at the dispatcher (hitl/dispatcher.rs)
  — product policy (A-HITL-01), not framework.
- Reachability: any consumer configuring `with_timeout_strategy` or the
  builder's `timeout_strategy`, or relying on the documented default.
- Expected invariant: the framework's documented timeout policy knob
  (default Reject) is honored; approval waits are bounded; provider timeout
  signaling is uniform.
- Observed behavior: the knob does nothing; default behavior is an unbounded
  wait with implicit drop-deny; console timeout surfaces as an error while
  manager/webhook surface `Timeout`.
- Impact: misleading public API (consumers believe a timeout policy exists);
  a hung UI/provider hangs the agent turn indefinitely on default wiring;
  consumers that handle the `Timeout` variant are bypassed when the console
  provider is used.
- Root cause: the timeout policy was designed into the config but the
  pipeline branch that should consult it (wrapping `handler.handle`) was
  never implemented; provider timeout handling evolved independently.
- Direction: either wrap `handler.handle` with the configured timeout and
  apply `TimeoutStrategy` (Reject/AutoApprove/Escalate), or remove the knob
  and document the default (unbounded wait + drop-deny); align the console
  provider to return `HumanLoopResponse::Timeout`; add a service test with a
  non-responding handler.
- Regression validation: service test with a never-responding handler and
  `TimeoutStrategy::Reject` → `Deny`; `AutoApprove` → `Allow`; console
  provider timeout returns the `Timeout` variant.
- Validation reports: [V01-01](../validations/F-HITL-01/V01-01.md),
  [V03-02](../validations/F-HITL-01/V03-02.md)

### F-HITL-01-P2-02: `add_need_appeal_tool`'s approval requirement is silently dropped on the live path — the Ask rule is buffered but never flushed by the live entry points; the framework doc and demo05 claim the chain works and can pass falsely

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: buffer write at `capabilities.rs:109-117` (pending_permission_rules);
  flush `context.rs:140-157` called only from run/approval.rs:20/:123 (dead)
  and `src/agent/react/tests.rs:1082` (test); live `snapshot.rs:1151-1173`
  (`tool_needs_approval`) and `snapshot.rs:797-857` (`check_tool_approval`)
  never flush (divergence documented at snapshot.rs:1145-1150 as
  "preserved intentionally"); framework doc `docs/zh/05-human-loop.md:378-390`
  promises the flush at "the first async permission check
  (`tool_needs_approval`/`check_tool_approval`)"; demo05
  (`examples/demo05_compressor.rs:197-198`, executed via `agent.execute` —
  the live `run_stream_channel` path, react/mod.rs:1839) claims to validate
  the approval chain.
- Reachability: any consumer calling the documented `add_need_appeal_tool`
  (public API; the only current caller is demo05).
- Expected invariant: a tool registered via `add_need_appeal_tool` requires
  human approval before execution on the main path.
- Observed behavior: the Ask rule never reaches `PermissionService` on the
  live path; the tool executes without approval; demo05's "approval
  validation" can pass without any approval event (the tool succeeds, so the
  demo's emptiness check passes).
- Impact: silent loss of an explicit approval requirement for a documented
  public API; the demo gives false confidence ("真实路径示例：…验证
  PermissionService / 审批事件链路", docs/zh/05-human-loop.md:375).
- Root cause: the flush was attached to the pre-refactor ReactAgent approval
  entry points; the streaming refactor created new live entry points
  (snapshot.rs) without porting the flush.
- Direction: flush pending rules at the start of the live
  `snapshot.tool_needs_approval`/`check_tool_approval` (or in
  `PermissionStage`); make demo05 assert that an approval event actually
  occurred (fail loudly otherwise); delete the flush calls together with
  `run/approval.rs`.
- Regression validation: live-path test: `add_need_appeal_tool(tool)` then
  execute the tool → provider receives an ApprovalRequest; a demo05 run with
  an assertion that the Ask event fired.
- Validation reports: [V01-01](../validations/F-HITL-01/V01-01.md),
  [V02-01](../validations/F-HITL-01/V02-01.md),
  [V05-01](../validations/F-HITL-01/V05-01.md)

### F-HITL-01-P2-03: Divergent duplicate approval implementations remain — the whole `run/approval.rs` file (tool_needs_approval/check_tool_approval/request_human_approval/handle_ask_decision) is dead code reachable only through the uncalled `process_steps`, and it is the ONLY code that implements the intended ask/scope/modified-args behavior

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `run/approval.rs:11-12` (`#[allow(dead_code)]` impl block);
  callers of `ReactAgent::check_tool_approval`: none in production (V01-01);
  `tool_needs_approval` caller: react_loop.rs:223 inside `process_steps`
  (react_loop.rs:177, `#[allow(dead_code)]`, zero callers); the live
  equivalents snapshot.rs:798/:1152 diverge (no rule flush, no provider ask,
  no modified-args read); `take_modified_args` reader only in the dead file;
  `PermissionPolicy`/`DefaultPermissionPolicy` (echo-core/src/tools/permission.rs:507-602,
  re-exported src/lib.rs:165, macro generator echo-macros/src/lib.rs:518-560)
  is a second decision authority with zero runtime callers and a stale doc
  claim in run/approval.rs:57-60 ("then PermissionPolicy check" — the code
  does not do it).
- Reachability: not-applicable for the dead code itself (no live caller);
  the PermissionPolicy menu API is consumer-opt-in.
- Expected invariant: one approval authority with one ask path; dead
  divergent implementations are removed (AGENTS.md code-cleanup rule).
- Observed behavior: two parallel approval semantics exist — the dead one
  works as documented, the live one does not (P1-01/P1-02); `PermissionPolicy`
  is a second decision API that nothing calls.
- Impact: maintainers reading `run/approval.rs` believe the framework asks
  humans and honors modified args (it does not, on the live path); the file's
  deletion (planned with `process_steps`, F-RCT-02-P3-01) risks silently
  discarding the only correct implementation if not ported first; the stale
  doc comment misleads.
- Root cause: the streaming refactor left the pre-refactor approval file as
  dead code instead of migrating and deleting it (partially completed
  cleanup of the "old vs new approval path" — AGENTS.md promise).
- Direction: port the asking behavior to the live path (P1-01/P1-02),
  then delete `run/approval.rs` with `process_steps`; fix or remove the
  PermissionPolicy doc claim and either document
  `PermissionPolicy`/`#[permission_policy]` as the opt-in extension point or
  delete them (framework public-API rule: retain only with a documented
  purpose).
- Regression validation: after deletion, `cargo test -p echo_agent --lib
  --locked pipeline` and the new live ask-path tests stay green;
  `cargo build --examples` succeeds (demo68 references only `human_loop`
  re-exports).
- Validation reports: [V01-01](../validations/F-HITL-01/V01-01.md),
  [V02-01](../validations/F-HITL-01/V02-01.md),
  [V04-02](../validations/F-HITL-01/V04-02.md)

### F-HITL-01-P3-01: Minor dead-code and doc issues — `PermissionServiceBuilder` drops `cache_ttl`; cache `args_key` is JSON-order-sensitive and the "SHA-256" hash helper is not SHA-256; `PermissionRequest::requires_confirmation` has no production callers; `would_request_human_for_permissions` ignores the approval cache (conservative batch serialization)

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `PermissionServiceBuilder` `#[allow(dead_code)]` (service.rs:913,:922)
  and `build()` constructs `SessionApprovalCache::new()` without TTL
  (service.rs:979), dropping `config.cache_ttl`; `args_key` uses
  `serde_json::to_string` (approval_cache.rs:270-277) — JSON key order
  changes the key (semantically identical args re-ask; safe direction);
  `sha256_hex` uses `DefaultHasher`, not SHA-256 (approval_cache.rs:280-290),
  and hash mode is never enabled by the service (service.rs:191-193);
  `PermissionRequest::requires_confirmation` (permission.rs:433-435) has zero
  production callers and its "suggestions non-empty → confirm" rule is
  misleading; `would_request_human_for_permissions` (service.rs:371-405)
  never checks the cache, so cache-hit tools are still serialized in batches.
- Reachability: dead code / edge behavior only.
- Expected invariant: no misleading API surface or dead builders; cache keys
  canonical; batch split matches execution decisions.
- Observed behavior: as listed.
- Impact: cosmetic-to-small: latent TTL bug in a dead builder; cache misses
  on arg key-order changes; a documented-but-unused helper with misleading
  semantics; slightly conservative batch serialization.
- Root cause: accumulated minor drift during the approval refactor.
- Direction: delete `PermissionServiceBuilder` or honor `cache_ttl`;
  canonicalize `args_key` (sorted-key serialization) or enable hash mode;
  rename `sha256_hex` or implement real SHA-256; delete
  `requires_confirmation` or wire it; consult the cache in the batch
  prediction.
- Regression validation: unit tests for canonical args keys and for the
  builder's TTL (if retained); existing service tests stay green.
- Validation reports: [V01-01](../validations/F-HITL-01/V01-01.md),
  [V03-03](../validations/F-HITL-01/V03-03.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search (approval/permission/HumanGate/decision authorities across both repos) | yes | passed | [V01-01](../validations/F-HITL-01/V01-01.md) |
| V02 | Registration and runtime reachability trace (live pipeline → PermissionStage → service → handler; dead ReactAgent branch; EKO wiring) | yes | passed | [V02-01](../validations/F-HITL-01/V02-01.md) |
| V03 | Decision/policy/provider mapping + local-vs-generic boundary classification | yes | passed | [V03-01](../validations/F-HITL-01/V03-01.md) |
| V03 | Timeout/default behavior (`timeout_strategy` reads, provider timeout signaling) | yes | passed | [V03-02](../validations/F-HITL-01/V03-02.md) |
| V03 | Approval cache identity and scope granularity (infer_scope, bridges, wildcard rule) | yes | passed | [V03-03](../validations/F-HITL-01/V03-03.md) |
| V04 | `cargo test -p echo_orchestration --lib --locked human_loop` | yes | passed (exit 0; 119 passed) | [V04-01](../validations/F-HITL-01/V04-01.md) |
| V04 | `cargo test -p echo_agent --lib --locked "permission"` + `-p echo_core --lib --locked "permission"` + `"react::run::approval"` + `"pipeline"` | yes | passed (exit 0; 7 + 23 + 0 + 14) | [V04-02](../validations/F-HITL-01/V04-02.md) |
| V05 | Historical-document drift (AGENTS.md HumanGate promise, 05-human-loop.md claims, CLI MASTER-PLAN) | conditional | passed | [V05-01](../validations/F-HITL-01/V05-01.md) |

All required validations executed; every reported command has a known exit
code; no validation is pending.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| AGENTS.md — "旧 HumanGate 路径 vs 新审批路径", old path deleted once the new one covers it | current (code cleanup done; partially completed in spirit — P2-03) | zero `HumanGate` symbols in both repos; `examples/demo68_human_gate.rs` doc-comment keeps the legacy name; [V01-01](../validations/F-HITL-01/V01-01.md), [V05-01](../validations/F-HITL-01/V05-01.md) |
| AGENTS.md — permission modes (`full-auto`/`default` etc.) apply only to agent automated paths, not direct user interactions; over-gating removed | current (framework side) | framework enforcement only in `PermissionStage` (pipeline.rs:268-346); no `require_full_auto` in echo-agent (EKO Tauri `IpcAuth` is application scope); [V03-01](../validations/F-HITL-01/V03-01.md) |
| `docs/zh/05-human-loop.md:56` — Bubble: "子 Agent 权限上溯到父级处理" | stale (unimplemented) | Bubble → `RequireApproval` → hard error on live path (service.rs:643, snapshot.rs:841-846); no bubbling code (V01-01); P1-01 |
| `docs/zh/05-human-loop.md:375/:378-390` — demo05 validates `add_need_appeal_tool` chain; rules flushed at first async permission check | regressed (live path) | flush callers are dead code + a test only; live entry points never flush (V01-01, V02-01); P2-02 |
| `docs/zh/05-human-loop.md:92/:107-108` — Session cache = same tool + args; SessionAllTools = per tool | regressed (service path) | `infer_scope_from_updates` collapses session approvals to per-tool global; live bridge maps SessionAllTools to "*" (V03-03); P1-03 |
| `docs/zh/05-human-loop.md:363` — cache TTL configurable | current | `PermissionServiceConfig.cache_ttl` default 30 min (service.rs:115-129, 191-193) |
| CLI MASTER-PLAN:13 — call-scoped `permission_mode_override` without global mode mutation | current | `check_with_permissions_in_mode` override (service.rs:484-493) + dedicated test (service.rs:1070-1085); [V05-01](../validations/F-HITL-01/V05-01.md) |
| B-REF-01 V05 — sandbox/approval policy separation; subagents excluded from interactive approval where possible | current (separation) / gap (bubbling) | `PermissionService` (policy) vs providers (transport) separated; Bubble parent-approval is unimplemented (P1-01) — a convergence gap for the subagent case |

## Coverage And Uncertainty

- All conclusions are static except the four test commands (V04); no live
  run exercised an approval event with a real provider on the EKO path
  (read-only review; A-HITL-01/Q-E2E-01 cover dynamic behavior).
- EKO-side details (which GUI button sends which scope; provider arbitration
  edge cases) were inspected only enough to classify the boundary; they are
  A-HITL-01 scope.
- `websocket.rs` and `audit.rs` were skimmed, not line-audited (no
  F-HITL-01-relevant contract was suspected there beyond what V01/V03
  confirmed).
- The `globset` feature-gated branch of `RuleMatcher::Pattern` (echo-core
  `permission` feature) was noted; the "*"/prefix fallbacks — the paths
  actually reached by session rules — were fully verified.
- P1-03's "which UI action maps to which scope" on EKO surfaces is not
  proven (A-HITL-01); the framework-side contract defect is proven
  independent of EKO (console provider alone reaches it).
- The exact commit where `process_steps` was superseded was not bisected;
  F-RCT-02-P3-01 already owns its deletion.

## Handoff

- Downstream tasks may rely on: one live decision authority
  (`PermissionService`) with the exact consumer mapping (V02-01); the
  RequireApproval/Ask no-ask defect and its only correct-but-dead
  implementation (P1-01); the dropped modified-args side channel (P1-02);
  the scope-widening conversions (P1-03); dead config knobs (P2-01);
  the broken `add_need_appeal_tool` flush (P2-02); the dead
  `run/approval.rs` file and its deletion coupling to `process_steps`
  (P2-03); test green state at the reviewed commits (V04).
- `A-HITL-01` should treat P1-01/P1-02/P1-03 as framework-side facts when
  designing EKO's arbitration: the EKO handler wiring is the only working ask
  path today, so any surface that expects `RequireApproval`-driven prompts
  (Bubble/Ask rules) must either rely on the framework fix or compensate
  EKO-side; the "*" session rule from "approve all" must be treated as a
  known framework behavior until fixed.
- `X-AUT-01` should use P1-01/P1-03 as permission-boundary evidence and check
  EKO's `IpcAuth::require_full_auto` (src/tauri/error.rs) against the
  over-gating invariant (framework side is clean — V03-01).
- `F-RCT-02` (P3-01: delete `process_steps`) must coordinate with P2-03:
  deleting `process_steps` deletes `run/approval.rs`'s only caller, so the
  asking behavior must be ported to the live path first (P1-01/P1-02).
- `X-BND-01` should record: `PermissionPolicy` as a zero-consumer framework
  menu API (P3-01), the two scope-converting bridges (P1-03), and the
  `PermissionServiceBuilder` dead code.
- Reports to read: this report + [V01-01](../validations/F-HITL-01/V01-01.md)
  through [V05-01](../validations/F-HITL-01/V05-01.md); dependency reports
  F-RCT-04 and B-REF-01.
- Stale triggers: any change to `snapshot.rs` `check_tool_approval` /
  `tool_needs_approval`, `pipeline.rs` `PermissionStage`, `service.rs`
  pipeline/infer_scope/bridges, `approval_cache.rs` key/scope semantics,
  `capabilities.rs` `add_need_appeal_tool`, `run/approval.rs` or
  `react_loop.rs` `process_steps` (deletion or revival), or
  `echo-core/src/tools/permission.rs` rule matching invalidates the
  corresponding claims.
- Follow-up task IDs (fixes are not implemented in this review): A-HITL-01,
  X-AUT-01, X-BND-01, F-RCT-02 (P3-01 coordination), Q-TST-01 (approval
  fixture gap), S-RDM-01.
