# A-HITL-01: Multi-surface human interaction policy

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both repositories clean

## Question

Does EKO arbitrate TUI/GUI/channel approvals within one shared deadline
without gating direct user interactions as agent automation?

Answer: partially. TUI/CLI/channel approvals run through one `HitlDispatcher`
with a single shared 5-minute deadline and fail-closed rejection, and direct
user interactions (terminal, MCP, files) carry no permission-mode gate — both
AGENTS.md invariants hold on those paths. But the GUI surface bypasses the
dispatcher entirely with its own per-conversation provider (no shared
arbitration), GUI-configured permission rules are stored but never applied,
the REPL provider auto-approves on empty/EOF stdin and defeats the shared
deadline while blocking, and "approve all" on every surface installs a
session-wide `*` allow rule (framework F-HITL-01-P1-03 wildcard, of which EKO
is the primary consumer). The denial-escalation path degrades to opaque tool
errors in EKO default mode (F-HITL-01-P1-01 manifestation).

## Scope

- `echo-agent-cli/echo-agent-app-core/src/hitl/` — full read: `mod.rs`,
  `dispatcher.rs`, `repl_provider.rs`, `tui_provider.rs`, `channel_provider.rs`.
- Permission wiring: `echo-agent-app-core/src/runtime.rs:120-146`,
  `state.rs:95-202,295-345,470-485`, `agent_pool.rs:190-250,460-490,920-978`,
  `infra.rs:280-300`, `tool_exposure.rs:290-310`, `src/main.rs:245-280`,
  `src/cli/modes.rs:32-64,118-235`, `src/cli/channels.rs`,
  `src/tauri/desktop.rs:185-195`, `src/tauri/commands/chat.rs:60-435,560-730,820-870`,
  `src/tauri/commands/panels.rs:25-151`, `src/tauri/error.rs`,
  `src/tauri/terminal.rs:256-375`, `src/tauri/commands/mcp.rs:205-258`,
  `src/tui/events.rs:36-260,3570-3610`, `src/tui/mod.rs:820-845,1940-1952`,
  `src/cli/cmd_impls/coding.rs:640-690`.
- Framework decision side (consumer mapping only): `react/mod.rs:1394-1440,
  1590-1630,2040-2085`, `react/run/pipeline.rs:268-347`, `snapshot.rs:795-870`,
  `echo-orchestration/src/human_loop/service.rs:160-260,484-760,855-925`,
  `echo-core/src/tools/permission.rs` (mode/rule/decision types),
  `echo-integration/src/channels/channels/mod.rs`.
- Frontend approval consumption (contract only): `ApprovalCard.tsx`,
  `ChatPanel.tsx:310-335`, `useTauriChat.ts:260-290`.
- Executed test: `cargo test -p echo-agent-app-core --lib --locked hitl`
  (exit 0, 5 passed).

## Out Of Scope

- Framework approval semantics per se (RequireApproval/Ask no-ask, modified
  args, scope inference, timeout knobs) → F-HITL-01 (cross-referenced, not
  re-audited; see the mapping in Findings).
- Frontend store/reducer correctness for approval events → A-SRF-03, A-FE-01/02.
- Subagent provider/permission inheritance (fork construction) → F-SUB-02,
  X-AUT-01; only the reachable consequence for background runs is noted.
- Terminal write consent semantics and audit logging → A-TOOL-01, X-AUT-01.
- Permission-mode defaults of the framework classifier/Bubble → F-HITL-01.

## Inputs

- Root `AGENTS.md` (threat model, over-gating lesson, TUI/GUI parity,
  one-authority, UTF-8/panic safety), shared `README.md`, `REPORTING.md`,
  `TASKS.md` (A-HITL-01 card), `zcode-ds/README.md`, report templates.
- Dependency task reports read: zcode-ds `F-HITL-01` (full) and `A-BOOT-01`
  (full). Historical documents treated as hypotheses:
  `echo-agent-cli/docs/MASTER-PLAN.md`, `2026-07-28-app-core-full-audit.md`,
  `2026-07-17-surface-parity-closeout.md`.

## Layering Decision

- Generic mechanism (framework, correct): `PermissionMode`/`PermissionDecision`/
  `PermissionService` pipeline, `HumanLoopProvider`/`HumanLoopRequest`/
  `HumanLoopResponse` contract, `DynProviderHandler` bridge
  (service.rs:855-925), approval cache, protected paths. No movement
  recommended.
- EKO product policy (application, correct placement): `HitlDispatcher` as
  the multi-surface arbitration composite (per MASTER-PLAN:452 and the
  app-core audit A2 "stays in app" verdict), the four leaf providers
  (repl/tui/channel/tauri-handler), per-turn provider injection,
  empty-dispatcher fail-closed default for pool/background agents,
  permission-mode state and per-surface commands, GUI rule management,
  browser approval providers, the 5-minute/300-second deadlines.
- Adapter boundary defects (application side of the framework bridge): the
  GUI "approve all" → `SessionAllTools` → `"*"` wildcard expansion is the
  framework bridge's mapping (F-HITL-01-P1-03), consumed by every EKO
  surface; the REPL provider's blocking-stdin/EOF behavior is EKO leaf
  policy.
- Duplicate search (terms + results in V01-01): `hitl`/`Hitl`/`HITL`,
  `approval`, `permission`, `require_full_auto`, `IpcAuth`, `Bubble`,
  `HumanGate`, `permission_mode`, `permission_rules`, `matches_tool`,
  `to_permission_decision`, `parse_permission_flag`, `add_rule`,
  `set_human_loop_provider`, `cancel_pending_hitl`, `human_in_loop`,
  `add_need_appeal_tool`. Results: one decision engine (framework
  `PermissionService`); four transport implementations, one of which
  (`TauriHumanLoopHandler`) is a parallel GUI transport that bypasses the
  app-core dispatcher (P2-01); `IpcAuth` gates and `PermissionRuleConfig`
  helpers are dead; no `HumanGate`; no EKO `Bubble` usage.

## Current Path

Verified call graph (V02-01):

1. Runtime bootstrap (runtime.rs:128-146) creates one `HitlDispatcher`,
   registers "repl", sets it as the primary-agent provider + browser default
   provider, and builds `PermissionService` from it.
2. TUI swaps "repl"→"tui" on the same dispatcher (main.rs:250-257); CLI
   keeps "repl"; channels never register on the dispatcher (per-handler
   `ChannelHumanLoopProvider` injected per message, channels.rs:141-146).
3. GUI chat turns set a per-conversation `TauriHumanLoopHandler` directly on
   the pool agent (chat.rs:570-582) and as the browser per-conversation
   provider (chat.rs:583-589); at turn end the agent's provider is replaced
   with an empty dispatcher (chat.rs:712-719). Pool agents are created with
   an empty dispatcher (agent_pool.rs:960-975).
4. Every tool call runs through `PermissionStage` (pipeline.rs:268-347) →
   `snapshot.check_tool_approval` (snapshot.rs:798-857) →
   `service.check_with_permissions_in_mode` (service.rs:484-681): protected
   paths deny in all modes; rules → cache → denial tracker → handler
   dispatch per mode. Default mode asks via the handler for
   Write|Execute|Network|Sensitive tools; `Allow` returns `Ok(None)`.
5. Handler responses flow back through `DynProviderHandler` (service.rs:
   855-925): Approved/Rejected/Timeout/scope rule updates; `Timeout` → deny.
6. Direct-user commands (terminal.rs:278-297, mcp.rs:211-258, files) carry
   no permission-mode gate; `IpcAuth` has zero callers (V01-01).

## Findings

### A-HITL-01-P1-01: GUI permission-rule management is behaviorally dead — `add_permission_rule`/`remove_permission_rule` store rules that no code ever applies to a tool call

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `src/tauri/commands/panels.rs:102-151` (add/remove write
  `AppState.config.permission_rules` only); the only reader is
  `panels.rs:86` (list); storage at `echo-agent-app-core/src/state.rs:343,482`;
  `PermissionRuleConfig` helpers `matches_tool` / `to_permission_decision` /
  `parse_permission_flag` (state.rs:145-201) have zero callers (V01-01);
  no `add_rule`/`apply_updates`/rule-push call exists anywhere in EKO code
  (V01-01, V03-02).
- Reachability: definition (state.rs:108-202) → registration (Tauri command
  list, src/tauri/mod.rs:230) → live caller (GUI settings UI invoking
  `add_permission_rule`/`list_permission_rules`) — the UI works, the rules
  never reach `PermissionService` (no live consumer of the stored list).
- Expected invariant: a permission rule the user configures (e.g.
  `deny tool:shell`, `ask tool:write_file`) affects the framework decision
  pipeline; the management surface is part of EKO's permission boundary.
- Observed behavior: rules are stored and listed but never applied; every
  rule is silently ineffective; `list_permission_rules` shows rules the
  agent ignores.
- Impact: a user who believes they have restricted the agent (deny/ask
  rules) has configured nothing; the rules panel is a misleading
  non-functional policy surface. For a local assistant this also removes the
  only GUI-native way to declare ask/deny policy without prompt tricks.
- Root cause: the rule-management IPC was built on a storage-only model;
  the push-to-service step (convert `PermissionRuleConfig` → framework
  `PermissionUpdate`/`RuleRegistry` and apply to the shared
  `PermissionService` and pool agents) was never implemented; the helper
  methods suggest it was planned.
- Direction: in `add_permission_rule`/`remove_permission_rule`, convert the
  rule with `to_permission_decision`/matcher parsing and apply to the shared
  `PermissionService` (and all pool agents) via the framework rule API
  (and `PermissionService::apply_updates` for session rules); or delete the
  commands and the helper methods (state.rs:139-201) if the surface is
  intentionally inert. Add a GUI-level test: configure `deny tool:shell`,
  invoke shell, expect denial.
- Regression validation: after the fix, `add_permission_rule("tool:shell",
  "deny")` → a `shell` tool call is denied and a `read_file` call is not;
  rule survives pool-agent reuse; `cargo test -p echo-agent-app-core --lib
  --locked hitl` stays green.
- Validation reports: [V01-01](../validations/A-HITL-01/V01-01.md),
  [V03-02](../validations/A-HITL-01/V03-02.md)

### A-HITL-01-P1-02: The REPL provider auto-approves approvals on empty/EOF stdin and its blocking `read_line` defeats the dispatcher's shared deadline

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-app-core/src/hitl/repl_provider.rs:74-77`
  (`"y" | "yes" | "" => Ok(HumanLoopResponse::Approved)` — empty input and
  EOF both produce `""`); the blocking read at `repl_provider.rs:69-72`
  (`std::io::stdin().read_line` inside the async request) cannot be
  interrupted by `tokio::time::timeout_at`, so the dispatcher's "total
  wall-clock wait is bounded by TIMEOUT_DURATION" claim (dispatcher.rs:96-101)
  is false for this provider; the REPL provider is registered on the shared
  dispatcher at bootstrap (runtime.rs:130-131) and never unregistered in the
  GUI path (only TUI unregisters, main.rs:253); GUI browser-default provider
  = the same dispatcher (runtime.rs:144-146, browser/mod.rs:980-987).
- Reachability: definition → registration (runtime.rs:131) → live caller:
  (a) CLI/REPL mode with piped/scripted stdin — when the input stream hits
  EOF, every subsequent approval auto-approves; (b) GUI: any approval routed
  to the primary-agent dispatcher or the browser default provider (before a
  per-conversation provider is set) hits the REPL provider — with stdin at
  /dev/null (Finder launch) EOF auto-approves, with a terminal attached it
  blocks a worker thread indefinitely (deadline unenforceable).
- Expected invariant: an approval with no user response fails closed
  (deny/reject), and the shared deadline bounds the wait for every provider
  (dispatcher's own documented contract; AGENTS.md fail-closed doctrine).
- Observed behavior: EOF or an empty line approves; a blocked stdin read
  suspends the timeout entirely.
- Impact: scripted/piped CLI runs silently approve destructive tools after
  input exhaustion (rm/format-class commands execute without consent);
  GUI-side approvals routed to the dispatcher can either auto-approve
  (Finder launch) or hang the request past the 5-minute deadline (terminal
  launch). Directly contradicts the "approval must not silently pass"
  invariant and the fail-closed comment in dispatcher.rs:82-87 (which only
  covers the no-provider case, not the EOF case).
- Root cause: `read_line` conflates EOF and empty input, and the empty
  branch was chosen as "approve" (probably mirroring "enter to approve" for
  interactive use); the provider was written as a blocking sync read inside
  async code without a non-blocking alternative.
- Direction: treat EOF distinctly from empty input (check `read_line`
  return: `Ok(0)` = EOF → `Rejected` with an EOF reason; `Ok(n)` with empty
  trimmed text → keep the current interactive "enter to approve" only when
  stdin is a TTY), and move the read to `tokio::io::stdin()` (async) or a
  dedicated blocking task so the dispatcher deadline can fire; add a
  provider test with a closed stdin asserting `Rejected`.
- Regression validation: provider test where stdin is closed → approval
  returns `Rejected`; piped CLI scenario `(echo -e 'y\nn' | eko ...)` with
  more approvals than inputs → remaining approvals reject, not approve.
- Validation reports: [V01-01](../validations/A-HITL-01/V01-01.md),
  [V03-01](../validations/A-HITL-01/V03-01.md)

### A-HITL-01-P1-03: "Approve all" on every EKO surface installs a session-wide `*` allow rule covering ALL tools — EKO is the primary consumer of F-HITL-01-P1-03's wildcard mapping

- Priority: P1
- Confidence: high
- Layer: adapter (EKO surfaces → framework bridge)
- Evidence: all four surfaces send `ApprovedWithScope { SessionAllTools }`
  for "approve all": TUI `'a'` (`src/tui/events.rs:242-244`), REPL `'a'`
  (`repl_provider.rs:77-79`), channel `"a"/"全部同意"`
  (`channel_provider.rs:158-160`), GUI "本会话同意" →
  scope `"session_all_tools"` (`web-frontend/src/components/chat/ChatPanel.tsx:321-322`
  → `src/tauri/commands/chat.rs:335-342`); the framework bridge maps
  `SessionAllTools` → `add_session_rule("*")`
  (`echo-orchestration/src/human_loop/service.rs:898-908`); `RuleMatcher::Pattern("*")`
  matches every tool (`echo-core/src/tools/permission.rs:253-255`); rules are
  evaluated before cache and handler (service.rs:561-573).
- Reachability: definition (framework bridge) → live caller: any user
  pressing "approve all" / "本会话同意" / `a` on any surface — the GUI button
  label "本会话同意" reads as session-scoped approval of the current tool,
  but unlocks every tool.
- Expected invariant: "approve all" covers the requested granularity
  (F-HITL-01-P1-03's expected invariant: session = same tool + same args;
  SessionAllTools = same tool, any args); the user's chosen scope is what is
  enforced.
- Observed behavior: one "approve all" auto-approves every subsequent tool
  call (any tool, any args, including destructive shell) for the whole
  session, bypassing cache, denial tracker and handler.
- Impact: a single approval silently removes the approval boundary for ALL
  tools for the session on every surface; the GUI label understates the
  scope. Same safety-boundary widening class as F-HITL-01-P1-03, now with
  the EKO surfaces as the (only) producers.
- Root cause: framework-side (see F-HITL-01-P1-03 — the two bridges
  disagree and the live bridge emits `"*"`); EKO-side, every surface
  uniformly chooses the `SessionAllTools` scope for "approve all" with no
  per-tool session option.
- Direction: framework fix first (service.rs:898-908 → tool-scoped rules via
  `build_matcher`-style mapping, per F-HITL-01-P1-03); EKO-side: consider
  mapping the GUI "本会话同意" to `ApprovalScope::Session` (per-tool) and
  relabel it, or expose both granularities; add a service-level regression
  test asserting a `SessionAllTools` response on `Bash` does NOT allow
  `Read`/`agent_tool`.
- Regression validation: after both fixes, "approve all" on `Bash` allows
  other `Bash` calls but `read_file`/`agent_tool` still ask; GUI end-to-end
  run of the same sequence.
- Validation reports: [V02-01](../validations/A-HITL-01/V02-01.md),
  [V03-02](../validations/A-HITL-01/V03-02.md)

### A-HITL-01-P2-01: GUI approval arbitration bypasses the app-core `HitlDispatcher` — a parallel `TauriHumanLoopHandler` with a global static pending map lives in the Tauri command layer, and the hitl module doc claims a Tauri route that does not exist

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `TauriHumanLoopHandler` defined at `src/tauri/commands/chat.rs:261-435`
  with the module-global `PENDING_RESPONSES` map (chat.rs:210-213); installed
  per conversation (chat.rs:570-582) instead of registering on the
  dispatcher; no Tauri/WebSocket provider is ever registered on
  `HitlDispatcher` (registration sites: runtime.rs:131 "repl", main.rs:254
  "tui" only — V01-01); `hitl/mod.rs:1-4` documents routing to "WebSocket,
  TUI, REPL, Tauri".
- Reachability: definition → registration (chat.rs:570-582 per chat turn,
  chat.rs:583-589 browser) → live caller: every GUI chat turn.
- Expected invariant: one arbitration composite owns the shared-deadline and
  first-response semantics for all surfaces (the module's own doc); the
  Tauri path participates in the same arbitration as TUI/CLI/channel.
- Observed behavior: GUI conversations get a direct per-conversation
  provider with its own hard-coded 300 s timeout and its own response map;
  the dispatcher's arbitration (shared deadline, first-response-wins,
  fail-closed aggregation) never sees GUI requests. The two transports
  diverge in timeout signaling, scope mapping, and cancellation semantics
  (GUI keeps stale pending entries only until timeout/cancel).
- Impact: the "one shared deadline across surfaces" property is not uniform
  (GUI is per-request 300 s); two parallel implementations of the same
  transport semantic (emit event, await response, timeout, scope mapping)
  must be kept in sync; the doc misleads maintainers. The audit
  (2026-07-28:29) and MASTER-PLAN:452 already agreed the dispatcher stays in
  app — the GUI provider should feed it rather than replace it.
- Root cause: the GUI provider was written before/independently of the
  app-core hitl unification and never migrated onto the dispatcher; the
  comment at chat.rs:568-569 ("keep concurrent GUI conversations isolated
  instead of racing through the global dispatcher") documents the deliberate
  bypass.
- Direction: register a per-conversation (or conversation-keyed) provider on
  the shared dispatcher instead of direct `set_human_loop_provider` on the
  agent, or move `TauriHumanLoopHandler` into app-core hitl as a leaf
  provider and have `set_human_loop_provider_preserving_approvals` operate
  through the dispatcher; fix the hitl/mod.rs doc; align the GUI scope
  mapping with the other providers.
- Regression validation: a GUI turn and a TUI turn issuing the same approval
  sequence produce identical provider semantics (scope, timeout, cancel
  cleanup); dispatcher unit tests covering first-response-wins and the
  shared deadline (currently absent — P3-01).
- Validation reports: [V01-01](../validations/A-HITL-01/V01-01.md),
  [V02-01](../validations/A-HITL-01/V02-01.md)

### A-HITL-01-P2-02: Channel approvals are not per-sender isolated — one handler-global `ChannelHumanLoopProvider` serves all senders and `resolve_message` matches by content only

- Priority: P2
- Confidence: medium
- Layer: application
- Evidence: `AppChannelMessageHandler` holds a single `hitl:
  Arc<ChannelHumanLoopProvider>` (`src/cli/channels.rs:40,58`) shared by all
  senders while per-sender isolation is claimed via the pool key
  (channels.rs:5-7, 65-68); the provider's pending slot is a single
  `Mutex<Option<PendingChannelRequest>>` (`channel_provider.rs:11-15,24-27`);
  a new request supersedes the previous one with "Superseded by a newer
  request" rejection (channel_provider.rs:94-106); `resolve_message` checks
  only the message text, not the sender (channel_provider.rs:42-67,
  channels.rs:114-119).
- Reachability: definition → registration (channels.rs:141-146 injects the
  shared provider on every message) → live caller: concurrent or interleaved
  messages from two senders while one sender's approval is pending.
- Expected invariant: per-sender isolation (documented in channels.rs:5-7
  "per-sender 隔离由 pool key 承载", aligning with "one session per user");
  a sender can only answer its own pending approval.
- Observed behavior: sender B's message text can resolve sender A's pending
  approval ("y" approves A's request); a second concurrent request from
  sender B rejects A's pending request outright.
- Impact: cross-sender approval interference — one IM user can approve or
  (via supersede) cancel another user's pending tool approval; with
  concurrent transport delivery this violates the documented per-sender
  isolation of the channel surface.
- Root cause: the provider was designed as a single-slot transport per
  handler; the pool-key isolation covers agents but not the shared provider
  state, and message resolution never binds to a sender.
- Direction: make the provider per-sender (keyed by conversation/sender, or
  include `conversation_id` in `PendingChannelRequest` and check it in
  `resolve_message`); reject resolution from a different sender with
  `ChannelHumanLoopResolution::Invalid`.
- Regression validation: unit test: sender A has a pending approval; sender
  B's "y" message returns `Invalid` and does not resolve A's request;
  concurrent two-sender scenario keeps both pending slots independent.
- Validation reports: [V01-01](../validations/A-HITL-01/V01-01.md),
  [V02-01](../validations/A-HITL-01/V02-01.md)

### A-HITL-01-P2-03: The denial-escalation path degrades to opaque tool errors on the EKO default path — after 3 consecutive denials (or 3 auto-rejections in background runs) `RequireApproval` fails the tool without asking

- Priority: P2
- Confidence: high
- Layer: adapter (EKO consumer of framework F-HITL-01-P1-01)
- Evidence: denial tracker returns `RequireApproval` after
  `max_consecutive_denials` (`echo-orchestration/src/human_loop/service.rs:575-585`,
  `record_denial` at service.rs:639-644); the live consumer maps
  `RequireApproval` to `Err("Tool '...' requires user approval")`
  (`src/agent/snapshot.rs:841-846`); no ask is issued on this path
  (F-HITL-01-P1-01). EKO-reachable contexts: (a) default mode, a user
  rejecting 3 consecutive approval-worthy tools; (b) background/cron runs on
  pool agents whose empty dispatcher auto-rejects every approval
  (agent_pool.rs:960-975, dispatcher.rs:82-87, V02-01) — the 3rd
  auto-rejection upgrades the 4th tool to the opaque error.
- Reachability: definition (service) → registration (PermissionService on
  all EKO agents) → live caller: any tool call after 3 consecutive denials.
- Expected invariant: "escalate to human approval" escalates — the human is
  asked (F-HITL-01-P1-01's expected invariant); background runs fail closed
  with a comprehensible reason.
- Observed behavior: the tool call fails with an opaque error; the user is
  never prompted; in background runs the error reason ("No HITL provider
  available") hides the underlying policy state.
- Impact: after repeated rejections the agent's turn errors instead of
  asking; background run outcomes misattribute the failure. Same defect
  class as F-HITL-01-P1-01, surfaced on the EKO default-mode path.
- Root cause: framework-side (RequireApproval/Ask never reach a provider on
  the live path — F-HITL-01-P1-01); EKO-side, background/pool agents have no
  provider by design (fail-closed), which feeds the tracker.
- Direction: fix F-HITL-01-P1-01 (route RequireApproval/Ask to the provider)
  and, EKO-side, either give background runs a deterministic "no-approval
  available → deny with explicit reason" path that does not pollute the
  denial tracker, or suppress tracker escalation for auto-rejections.
- Regression validation: EKO default-mode test: 3 rejections then a 4th
  write tool → provider receives an ApprovalRequest (after fix); background
  run with write tools → explicit denial reason, no tracker escalation.
- Validation reports: [V02-01](../validations/A-HITL-01/V02-01.md),
  [V03-02](../validations/A-HITL-01/V03-02.md)

### A-HITL-01-P2-04: `IpcAuth::require_full_auto`/`require_not_strict` are dead APIs whose module doc claims a permission gate that does not exist

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: definitions at `src/tauri/error.rs:40-76` (`IpcPermission` +
  `IpcAuth`), module doc at error.rs:6-10 ("commands that execute arbitrary
  code are gated behind `IpcAuth::require_full_auto()`"); zero callers in
  either repository (V01-01).
- Reachability: none — definition exists, never registered/called.
- Expected invariant: no dead security-gate API with a misleading doc; the
  local-threat-model rationale (AGENTS.md) is the documented reason gates
  were removed.
- Observed behavior: the API compiles and its doc claims a gating model that
  nothing uses; `create_terminal`/`connect_mcp_server` are gated only by
  input validation and the write-consent mechanism (V03-02).
- Impact: maintainers may believe the desktop IPC is permission-gated (or
  reintroduce the historical over-gating by "using" the API); the
  AGENTS.md historical lesson ("require_full_auto 门控 … 已移除") is
  documented as fixed but the scaffolding survives.
- Root cause: the over-gating removal deleted the call sites but left the
  gate API and its doc in place.
- Direction: delete `IpcAuth`/`IpcPermission` (error.rs:14-76) and rewrite
  the module doc to state that direct-user IPC carries no permission-mode
  gate by design (local threat model); keep only the validation-based
  guards.
- Regression validation: grep for `IpcAuth|require_full_auto` after removal
  (zero hits); `cargo check --no-default-features --features gui --bin
  echo-agent-tauri --locked` (baseline A-BOOT-01 V04-02, exit 0).
- Validation reports: [V01-01](../validations/A-HITL-01/V01-01.md),
  [V03-02](../validations/A-HITL-01/V03-02.md)

### A-HITL-01-P3-01: `HitlDispatcher` arbitration has zero unit tests; the REPL provider is untested

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `dispatcher.rs` contains no `#[cfg(test)]`; the 5 hitl tests
  cover only the TUI and channel leaf providers (V04-01); the REPL provider
  has no tests (its EOF/blocking behaviors, P1-02, are untested).
- Reachability: test gap only.
- Expected invariant: the arbitration composite (first-response-wins, shared
  deadline, fail-closed aggregation, empty-provider reject, cancellation of
  remaining futures) is covered by unit tests; Q-TST-01 can rely on it.
- Observed behavior: no coverage; a regression in the arbitration would pass
  the suite silently.
- Impact: low today, but the P1-02/P2-01 findings would have been caught by
  tests (e.g., a fake provider that never resolves → assert the 5-minute
  deadline fires; a fake provider returning `Err` → assert others still win).
- Root cause: the dispatcher was added with leaf-provider tests only.
- Direction: add dispatcher tests (fake providers via a test double):
  first-response-wins, single Err does not fail the request, all-Err →
  `Rejected`, empty registry → `Rejected`, deadline firing → `Rejected`;
  add a REPL provider test with closed stdin.
- Regression validation: the new dispatcher tests are green and the existing
  hitl tests stay green.
- Validation reports: [V04-01](../validations/A-HITL-01/V04-01.md)

## F-HITL-01 Finding Mapping (independent verification)

| F-HITL-01 finding | EKO-side verification | Verdict |
|---|---|---|
| P1-01 live path never asks (RequireApproval/Ask → opaque error) | Confirmed reachable on EKO default mode via the denial-tracker escalation and in background/pool runs (empty-dispatcher auto-reject feeds the tracker) → A-HITL-01-P2-03 | EKO does not mitigate; P2-03 records the surfaced manifestation |
| P1-02 user-modified args discarded | EKO surfaces never emit `ModifiedArgs` — all "modify" flows are feedback rejections (TUI events.rs:53-81, ChatPanel.tsx onModify → `approved:false`, repl/channel "m" → `Rejected`) | EKO mitigates by not using the feature; framework bug remains latent (F-HITL-01-P1-02) |
| P1-03 `SessionAllTools` → `"*"` wildcard | All four EKO surfaces + browser map "approve all" to `SessionAllTools`; GUI label "本会话同意" understates the all-tools scope → A-HITL-01-P1-03 | EKO is the primary consumer of the wildcard; aggravates by uniform surface choice |
| P2-01 TimeoutStrategy dead / unbounded default | EKO imposes its own deadlines (dispatcher 5 min; TUI/channel/GUI 300 s) so the framework knob's deadness is not EKO-visible; GUI/REPL timeout behavior covered by P1-02/P2-01 | Framework-side only |
| P2-02 `add_need_appeal_tool` flush dead | EKO never calls `add_need_appeal_tool`; `.enable_human_in_loop()` only gates tool registration (infra.rs:289) | Not EKO-reachable |
| P2-03 duplicate dead approval implementations | Not EKO-visible (framework dead code) | Framework-side only |

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search (hitl/permission/approval/gates across both repos) | yes | passed | [V01-01](../validations/A-HITL-01/V01-01.md) |
| V02 | Registration and runtime reachability (provider map per surface, dead APIs, mode propagation) | yes | passed | [V02-01](../validations/A-HITL-01/V02-01.md) |
| V03 | Provider arbitration and cancellation; timeout/default behavior (dispatcher, TUI/channel/REPL/GUI timeouts, fail-closed mapping) | yes | passed | [V03-01](../validations/A-HITL-01/V03-01.md) |
| V03 | Direct-user vs Agent call paths; default permission mode scenarios (terminal/MCP/files gates, mode propagation, rules dead, escalation, scope mapping) | yes | passed | [V03-02](../validations/A-HITL-01/V03-02.md) |
| V04 | `cargo test -p echo-agent-app-core --lib --locked hitl` (exit 0, 5 passed) | yes | passed | [V04-01](../validations/A-HITL-01/V04-01.md) |
| V05 | Historical-document drift (MASTER-PLAN, app-core audit A2, surface-parity closeout, AGENTS.md, module docs) | yes | passed | [V05-01](../validations/A-HITL-01/V05-01.md) |

All required validations executed; every reported command has a known exit
code (V04-01 exit 0; GUI compile baseline exit 0 at the same commit,
A-BOOT-01 V04-02); no validation is pending.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| AGENTS.md — `create_terminal`/`connect_mcp_server` `require_full_auto` gates were removed; direct user interactions must not be gated | current | no gate on either command (terminal.rs:278-297, mcp.rs:211-258); `IpcAuth` dead (P2-04); [V03-02](../validations/A-HITL-01/V03-02.md) |
| MASTER-PLAN.md:69 — dispatcher snapshots providers, broadcasts under one shared deadline, drops remaining futures after first response | current (with REPL caveat) | dispatcher.rs:74-137; REPL blocking read defeats the deadline (P1-02); [V03-01](../validations/A-HITL-01/V03-01.md) |
| MASTER-PLAN.md:452 — HITL dispatcher stays in the app layer | current | dispatcher in app-core hitl (V02-01) |
| 2026-07-28-app-core-full-audit.md:29 — A2 fix list (shared deadline, cancel remaining, snapshot before await) | fixed/current | all three implemented (dispatcher.rs:74-137) |
| 2026-07-17-surface-parity-closeout.md:63 — TUI free-form Input auto-approved, Selection silently picks first option | fixed | tui_provider input_mode + passing tests (V04-01) |
| 2026-07-17-surface-parity-closeout.md:65 — channel agents have empty dispatcher and reject every request | fixed | channels.rs:141-146 injects the channel provider per message (V02-01) |
| `hitl/mod.rs:1-4` — routes to "WebSocket, TUI, REPL, Tauri" | stale | no WebSocket/Tauri provider in the module; GUI provider bypasses the dispatcher (P2-01) |
| `error.rs:6-10` — IPC commands "gated behind `IpcAuth::require_full_auto()`" | stale (false) | zero callers (P2-04) |

## Coverage And Uncertainty

- All conclusions are static except the V04 test run; no live approval event
  was exercised end-to-end on any surface (read-only review; Q-E2E-01 owns
  dynamic scenarios).
- Whether TaskRuntime plan subagents inherit the primary agent's
  provider/permission service is unresolved in this slice (framework fork
  construction — F-SUB-02, X-AUT-01). If they do not, the empty-dispatcher
  auto-reject also applies to in-turn complex runs' subagents; if they do,
  approvals flow through the Tauri/TUI provider. The P2-03/P1-01 mapping is
  independent of this question for background/pool runs (confirmed
  pool-acquired agents).
- Channel cross-sender interference (P2-02) is proven structurally; whether
  the QQ/Feishu transports deliver concurrently (which triggers it) is a
  transport-level dynamic question (F-INT-02, Q-E2E-01).
- The GUI "repl still registered" path (P1-02 context b) depends on a
  browser action or primary-agent request occurring without a per-conversation
  provider; the primary-agent and browser-default wiring is confirmed, the
  concrete GUI flows that trigger it were not exhaustively enumerated.
- The frontend approval store and `ApprovalCard` rendering correctness are
  A-SRF-03/A-FE-01 scope; only the scope string and callback contract were
  checked here.

## Handoff

- Downstream tasks may rely on: the per-surface provider map and
  registration sites (V02-01); dispatcher arbitration semantics as
  documented for non-blocking providers (V03-01); the direct-user command
  surfaces being gate-free and mode propagation being complete (V03-02); the
  dead GUI rule management (P1-01); the REPL EOF auto-approve and
  deadline-defeating blocking read (P1-02); the all-surface `SessionAllTools`
  wildcard consumption (P1-03); the GUI dispatcher bypass (P2-01); the
  channel per-sender gap (P2-02); the denial-escalation opaque error on EKO
  paths (P2-03); the dead `IpcAuth` API (P2-04); the missing dispatcher test
  coverage (P3-01).
- Reports to read: this report + V01-01 through V05-01; dependency reports
  F-HITL-01 and A-BOOT-01.
- `X-AUT-01` should use P1-02/P1-03/P2-03 as EKO-side permission-boundary
  evidence and P2-04 as the over-gating-cleanup completion item; the
  "direct-user gate-free" verification (V03-02) answers its call-path
  classification for terminal/MCP/files.
- `X-SRF-01` should add rows: GUI approval transport (bypasses dispatcher),
  channel approval transport (per-handler, not per-sender), background-run
  approval behavior (auto-reject), GUI rules management (dead).
- `Q-TST-01`/`Q-E2E-01` should exercise: piped-stdin CLI approvals (P1-02),
  browser-default-provider approval in GUI (P1-02 context b), concurrent
  two-sender channel approvals (P2-02), GUI approval round-trip scope
  mapping (P1-03).
- Stale triggers: changes to `hitl/*` (dispatcher semantics, REPL provider
  EOF handling), `src/tauri/commands/chat.rs` provider injection,
  `panels.rs` rule commands, `agent_pool.rs` empty-dispatcher default,
  `channels.rs`/`channel_provider.rs` per-sender handling, framework
  `service.rs` bridge scope mapping, `snapshot.rs` RequireApproval/Ask arms
  invalidate the corresponding claims.
- Follow-up task IDs (fixes are not implemented in this review): X-AUT-01,
  X-SRF-01, X-BND-01, Q-TST-01, Q-E2E-01, S-RDM-01.
