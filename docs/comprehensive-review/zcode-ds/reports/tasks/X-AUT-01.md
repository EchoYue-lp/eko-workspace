# X-AUT-01: Permission and local security boundary

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63 (9b0e0fa)
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5 (b3b2e81)
> Worktree state: both source repositories clean

## Question

Are automated Agent actions controlled while direct user terminal, file
picker, MCP configuration, and browser interactions remain usable?

Answer: **partially controlled, correctly separated, with two new boundary
defects and one parity gap.** Automated actions on the primary and pool-agent
paths are controlled by the single framework enforcement point
(`PermissionStage` -> `PermissionService`) and the default/full-auto mode
matrix behaves as documented, with protected paths denying in every mode
including full-auto; direct user interactions (terminal, file picker, MCP
configuration, browser entry) carry no `permission_mode` gate and remain
usable (V01-01, V04-01). Local data-loss protections are in place (worktree
dirty-tree rejection, atomic conversation persistence, terminal write
consent/bound, sandbox floor) and secret redaction covers all framework
sinks, with four indexed gaps at the EKO outbound/persistence boundaries
(V03-01). Two new cross-boundary defects surface here: (P1-01) TaskRuntime
writer/readonly subagents execute automation entirely outside the permission
boundary — no `PermissionService`, no provider, no mode propagation, no
protected-path checks (this resolves the open question A-HITL-01 handed to
X-AUT-01); (P2-01) the framework `PermissionService` handler is
process-global mutable state shared by the primary and every pool agent, so
per-conversation provider installs and turn-end restores mutate the approval
channel for all agents at once. Plus (P3-01) the CLI `/permission` command
does not propagate the mode to the agent pool.

## Scope

Cross-boundary verification at the current commits, consuming (not
re-auditing) the four dependency reports:

- Agent automation enforcement: `echo-agent/src/agent/react/run/pipeline.rs`
  (`PermissionStage` :268-346, `PlanModeStage` :995-1024),
  `echo-agent/src/agent/snapshot.rs` (:227-236 plan filter, :795-870
  `check_tool_approval`), `echo-orchestration/src/human_loop/service.rs`
  (:484-681 dispatch, :167-182 confirmation sets, :250-265 provider
  replacement), `echo-core/src/tools/permission.rs` (modes :58-104).
- EKO wiring: `echo-agent-cli/echo-agent-app-core/src/runtime.rs:120-146`
  (bootstrap wiring), `agent_pool.rs` (:120-150 shared extraction, :219-250
  from_runtime, :824-975 agent creation incl. empty dispatcher and shared
  service), `infra.rs:881-1040` (subagent factories),
  `src/tauri/terminal.rs:281-375`, `src/tauri/commands/mcp.rs:169-258,476-559`,
  `src/tauri/commands/chat.rs:560-590,705-725`, `src/tauri/commands/panels.rs:30-80,102-151`,
  `src/cli/cmd_impls/coding.rs:635-690`, `src/tui/events.rs:3570-3610`,
  `src/tauri/error.rs` (IpcAuth), `browser/mod.rs:978-1018`,
  `worktree.rs:700-730`.
- Secret/data-loss anchors: `chat_driver.rs:124-157`,
  `webhook/emitter.rs:138-149`, `trace/mod.rs:349-359,425-444`,
  `echo-tools/src/web/fetch.rs:203`, `echo-tools/src/code.rs:305-345`.
- Executed commands: grep sweeps only (static review); no build/test needed
  — dependency V04 reports already establish green gates at these commits.

## Out Of Scope

- Framework approval semantics per se (ask path, modified args, scope
  inference, timeout knobs) -> F-HITL-01 (canonical findings folded in).
- EKO multi-surface arbitration internals (dispatcher, leaf providers) ->
  A-HITL-01; browser/MCP/LSP integration specifics -> A-INT-01; guards,
  sandbox, panic safety -> F-SEC-01; guard/sandbox sink inventories ->
  F-SEC-01/F-EXT-01/A-TOOL-01.
- Subagent lifecycle correctness (team modes, cancellation) -> F-SUB-02;
  the permission wiring question here was explicitly delegated to X-AUT-01
  by A-HITL-01 and is resolved in P1-01.
- Dynamic end-to-end approval runs -> Q-E2E-01 (read-only review).

## Inputs

- Root `AGENTS.md` (local threat model; interactive vs automation; when to
  add protection; historical `require_full_auto` lesson), shared
  `README.md`, `REPORTING.md`, `TASKS.md` (X-AUT-01 card), `zcode-ds/README.md`,
  report templates.
- Dependency task reports read (all four, in full):
  `F-HITL-01.md`, `F-SEC-01.md`, `A-HITL-01.md`, `A-INT-01.md`, plus their
  validation inventories; `A-TOOL-01.md` (P1-01 writer plan-mode collision)
  and `F-SUB-01.md`/`F-SUB-02.md` (permission terms) were consulted for the
  P1-01 duplicate check.
- Historical documents treated as hypotheses: AGENTS.md security lesson
  claims and the A-HITL-01 "unresolved" subagent-inheritance question.

## Layering Decision

- Generic mechanism (framework, correctly placed): `PermissionMode` /
  `PermissionService` pipeline / `HumanLoopProvider` bridge /
  protected-path checks. The framework's only enforcement point is inside
  the agent tool pipeline (`PermissionStage`) — it never gates direct user
  interactions (V01-01; zero `require_full_auto` in echo-agent).
- EKO product policy (application, correct placement): mode surfaces
  (GUI/TUI/CLI normalization), shared `PermissionService` across the pool,
  empty-dispatcher fail-closed for background agents, per-conversation
  GUI provider, terminal consent, MCP light validation.
- Adapter boundary defects (new): (1) the subagent factories omit service/
  provider/mode wiring (P1-01) — an EKO wiring omission; the framework
  default (`permission_service: None` -> allow-all, react/mod.rs:535 +
  snapshot.rs:857-862) is opt-in by design but its documented fail-safe
  ("no handler -> RequireApproval") only exists when a service exists;
  (2) `set_human_loop_provider*` mutates the shared service's handler in
  place (framework semantics) combined with EKO's pool-wide service sharing
  (P2-01).
- Duplicate-search terms (both repositories): `build_permission_service`,
  `set_permission_service`, `set_human_loop_provider`,
  `require_full_auto`, `IpcAuth`, `permission_mode`, `PermissionService`,
  `replace_provider`, `apply_permission_mode`, `permission_service: None`,
  `subagent.*permission`. Results: one decision engine; one service
  instance per process (primary + pool share it); zero `permission` wiring
  inside the subagent factories; zero `IpcAuth` callers; `permission_mode`
  only on agent paths. No parallel decision authority exists — the findings
  below are wiring/boundary defects, not duplicates of earlier reports.

## Current Path

Verified cross-boundary call graph (V01-01, V02-01):

1. Direct user surfaces are gate-free: `create_terminal`
   (terminal.rs:281-295, explicit local-model comment) with per-session
   consent + 64 KiB bound + audit for writes (terminal.rs:301-369); native
   OS file dialog (default.json:10-14, tauri-bridge.ts:91); MCP
   connect/config with light input validation only (mcp.rs:169-258);
   browser entry via agent tools (permission pipeline) + `confirm_action`
   risk gate (browser/mod.rs:978-1018).
2. Automated agent tool calls (primary and pool agents) flow through
   `PermissionStage` (pipeline.rs:268-346) -> `check_tool_approval`
   (snapshot.rs:798-857) -> `check_with_permissions_in_mode`
   (service.rs:484-681): protected paths deny in all modes (service.rs:505-521);
   Default asks for Write|Execute|Network|Sensitive via the configured
   handler (service.rs:167-172); full-auto (BypassPermissions) allows with
   a loud warn (react/mod.rs:2066-2073); mode switches clear the approval
   cache (react/mod.rs:2074-2076).
3. One `PermissionService` per process: extracted from the primary agent at
   pool creation (agent_pool.rs:120-150,224) and installed on every pool
   agent (agent_pool.rs:928-929); `set_human_loop_provider*` replaces the
   service handler in place (react/mod.rs:1592-1603,1617-1625;
   service.rs:258-265). GUI installs a per-conversation
   `TauriHumanLoopHandler` per turn (chat.rs:570-582) and an empty
   dispatcher at turn end (chat.rs:712-719); pool agent creation installs
   an empty dispatcher (agent_pool.rs:955-975). Consequence: the handler
   slot is process-global — see P2-01.
4. TaskRuntime writer/readonly subagents are built by factories
   (infra.rs:881-966, 968-1010) with no permission service, no provider, no
   mode; `set_plan_mode(true)` (infra.rs:963) blocks write tools/shell/
   delete_file via visibility filter (snapshot.rs:227-236) and
   `PlanModeStage` (pipeline.rs:1004-1018); `run_code`, network, browser,
   git, and data tools are not in the blocklist and execute with no
   permission decision (snapshot.rs:857-862 `Ok(None)` when no service) —
   see P1-01.

## Findings

### X-AUT-01-P1-01: TaskRuntime writer/readonly subagents execute automation entirely outside the permission boundary — no PermissionService, no provider, no mode propagation; protected-path checks do not run; the default/full-auto mode matrix silently does not apply to subagent automation

- Priority: P1
- Confidence: high
- Layer: adapter (EKO subagent factory wiring; framework opt-in default
  `permission_service: None` is the mechanism)
- Evidence: `echo-agent-cli/echo-agent-app-core/src/infra.rs:881-966`
  (`build_writer_subagent_agent`) and `:968-1010`
  (`build_readonly_subagent_agent`) never call
  `build_permission_service`/`set_permission_service`/`set_human_loop_provider`
  (grep: zero `permission` hits in infra.rs beyond the worktree comment at
  :398-400); `subagent.set_plan_mode(true)` at infra.rs:963; framework
  default `permission_service: None` at
  `echo-agent/src/agent/react/mod.rs:535`; allow-all fallback
  `snapshot.rs:857-862` (`if let Some(service)` -> else `Ok(None)`);
  protected paths live inside the service (`service.rs:505-521`) and do not
  run without it; plan-mode blocklist `snapshot.rs:227-236` +
  `pipeline.rs:1004-1018` covers only write tools/shell/delete_file —
  `run_code`, web/network, browser, git, and data tools pass; contrast:
  pool agents DO get the shared service and mode (`agent_pool.rs:928-932`).
- Reachability: definition (infra.rs factories) -> registration
  (`register_subagent_factory`, infra.rs:851; plugin_components.rs:472) ->
  live caller (TaskRuntime fork/isolated dispatch
  `subagent/executor.rs:1148+`, `registry.rs:318-407`; `isolated.rs:52-64`
  builds a bare `ReactAgentBuilder` with no service at all) — every
  Implementation/Debugging task run and every forked subagent.
- Expected invariant: AGENTS.md "permission modes apply to agent automated
  paths" and the task question's "automated Agent actions controlled":
  subagent automation is subject to the same decision pipeline as primary
  automation (or a documented equivalent), and protected paths always deny.
- Observed behavior: subagent tool calls return `Ok(None)` (no decision)
  for every tool outside the plan-mode blocklist; the mode the user set
  (default/strict/full-auto) has zero effect on subagents; protected-path
  checks (.git/.ssh/.env) do not run; approvals never reach a provider
  (browser consequential actions instead fail closed with "no HITL provider
  is available", browser/mod.rs:991-995). This also resolves A-HITL-01's
  open question: subagents inherit NEITHER the provider NOR the
  empty-dispatcher auto-reject — they have no service at all.
- Impact: in default and strict modes the user expects write/execute/
  network to ask before running; on subagents `run_code` (sandboxed at
  OsSandbox with worktree cwd), web fetches, and browser navigation execute
  with no approval and no denial. Physical containment (plan mode for file/
  shell writes, worktree checkout, sandbox floor) bounds the data-loss
  surface, so no P0; but the mode matrix — the product's stated automated-
  action control — silently does not govern a major automation surface.
- Root cause: the subagent factories were written before/independently of
  the permission wiring and never gained a service; the framework's
  allow-when-no-service fallback converts "no wiring" into "no control"
  rather than "no permission granted" (its documented fail-safe
  "no handler -> RequireApproval" only applies once a service exists).
- Direction: give writer/readonly subagents the shared `PermissionService`
  (and the current mode) at factory construction — mirroring
  `agent_pool.rs:928-932` — with an explicit provider policy (fail-closed
  empty dispatcher for background runs); or, framework-side, make the
  no-service fallback in `check_tool_approval` deny instead of allow when
  tools carry Write/Execute/Network/Sensitive permissions. Coordinate with
  A-TOOL-01-P1-01 (writer subagents are currently silently read-only due to
  plan mode; permission wiring must not be masked by that collision).
- Regression validation: a writer-subagent fixture calling `run_code` (or
  web fetch) in default mode must produce an approval request (or denial),
  not `Ok(None)`; a `write_file` to `.git/config` path must be denied on
  the subagent; mode change on the primary must be observable inside
  subagents.
- Validation reports: [V01-01](../validations/X-AUT-01/V01-01.md),
  [V02-01](../validations/X-AUT-01/V02-01.md)

### X-AUT-01-P2-01: The PermissionService handler is process-global mutable state — per-conversation provider installs, turn-end restores, and pool-agent creation mutate the approval channel for every agent at once, defeating per-conversation isolation and the background fail-closed design

- Priority: P2
- Confidence: high (mechanism fully traced; user-visible impact depends on
  flow timing)
- Layer: adapter (framework `replace_provider` semantics + EKO pool-wide
  service sharing)
- Evidence: one service Arc shared by primary + every pool agent
  (`agent_pool.rs:136,224,928-929`; `from_runtime` extracts from the
  primary, :224); `set_human_loop_provider*` -> in-place
  `service.replace_provider[_preserving_cache]`
  (`echo-agent/src/agent/react/mod.rs:1592-1603,1617-1625`;
  `echo-orchestration/src/human_loop/service.rs:258-265`); GUI installs
  `TauriHumanLoopHandler` per conversation turn (chat.rs:570-582) and an
  empty `HitlDispatcher` at turn end (chat.rs:712-719); every pool agent
  creation installs an empty dispatcher (agent_pool.rs:955-975); GUI chat
  routes to pool agents (`agent_for` -> `pool.acquire`, state.rs:309-321).
- Reachability: any GUI conversation turn (install -> all agents' approvals
  route to that conversation's handler for the turn duration; turn end ->
  all agents' approvals hit the empty dispatcher and reject) and any pool
  acquire in TUI/CLI processes (empty dispatcher replaces the primary's
  handler -> subsequent primary approvals reject).
- Expected invariant: per-conversation approval isolation (comment at
  chat.rs:568-569) and the background/pool fail-closed empty-dispatcher
  default (agent_pool.rs:955-975; A-HITL-01-P2-03's mitigation claim) —
  provider semantics are per agent/conversation; the service handler
  matches the agent that owns the request.
- Observed behavior: the handler slot is one per process. During an active
  GUI conversation, a background TaskRuntime approval is emitted to that
  conversation's window (mis-attributed consent card; 300 s hang if
  ignored); after any GUI turn ends, all non-turn approvals in the process
  reject (empty dispatcher) even though the primary agent's own
  `approval_provider` field still names the bootstrap dispatcher; in
  TUI/CLI processes the first pool acquire clobbers the primary's handler
  with an empty dispatcher.
- Impact: consent can be granted in the wrong conversation for a background
  action (defeating the documented fail-closed background design during
  active turns); approvals can silently reject after turn end or first pool
  acquire. Bounded (single user, fail-closed direction dominates), hence P2
  rather than P1.
- Root cause: EKO deliberately shares one `PermissionService` across the
  pool (mode/rules consistency) while the framework's provider API replaces
  the handler in place; nothing reconciles "one shared service" with
  "per-conversation providers" — the per-agent `approval_provider` field is
  not what the service consults.
- Direction: key the handler per request owner (e.g., route through the
  dispatcher with per-conversation provider registration, per A-HITL-01-P2-01's
  direction) or make `PermissionService` handler selection per-agent
  (handler registry keyed by agent/conversation); restore the previous
  handler after a turn ends instead of installing an empty dispatcher;
  assert in tests that two conversations' providers never share a slot.
- Regression validation: dispatcher/service fixture with two conversations:
  conversation A installs a provider, conversation B's approval must not be
  emitted through A's provider; after A's turn ends, B's approval must
  still reach B's provider; TUI fixture: pool acquire then a primary
  approval must reach the TUI provider.
- Validation reports: [V01-01](../validations/X-AUT-01/V01-01.md)

### X-AUT-01-P3-01: CLI `/permission` does not propagate the mode to the agent pool — GUI and TUI do; CLI/headless background and channel agents keep "default" while the REPL runs in another mode

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/src/cli/cmd_impls/coding.rs:675`
  (`ctx.agent.write(|a| a.set_permission_mode(normalized))` — no
  `apply_permission_mode` call); GUI `panels.rs:59-80` and TUI
  `events.rs:3593-3602` both call `pool.apply_permission_mode`; the pool
  exists in headless/CLI mode (`main.rs:183-200`) and is used by channels
  (`main.rs:355-370`); `apply_permission_mode` exists and propagates to
  existing and future pool agents (`agent_pool.rs:467-483`).
- Reachability: CLI REPL `/permission full-auto` then a channel message or
  background task -> pool agent still in "default" -> write/execute asks
  (REPL provider) or rejects, while the REPL agent auto-approves.
- Expected invariant: multi-surface functional equality (AGENTS.md) —
  mode changes are a product-level setting that applies to all agents of
  the session, as GUI/TUI do.
- Observed behavior: the CLI surface changes only the primary agent.
- Impact: inconsistent automation control between the REPL and its
  background/channel agents in CLI/headless runs; low severity (no data
  loss; fail-closed direction), parity defect.
- Root cause: the CLI command predates pool propagation; GUI/TUI were
  updated, CLI was not.
- Direction: add `pool.apply_permission_mode(normalized)` to
  `cmd_permission` (mirroring panels.rs:76 / events.rs:3600) when a pool is
  present; add a REPL-level test asserting pool agents observe the mode.
- Regression validation: CLI fixture: `/permission full-auto` -> a pool
  agent's `get_permission_mode` returns "full-auto".
- Validation reports: [V02-01](../validations/X-AUT-01/V02-01.md)

## Canonical Findings Folded Into This Task

Already-filed findings re-verified at the current commits and carried by
canonical ID (full matrix in [V05-01](../validations/X-AUT-01/V05-01.md)):

- Automation-control defects (axis A): `F-HITL-01-P1-01` (RequireApproval/
  Ask never ask on the live path), `F-HITL-01-P1-03` + `A-HITL-01-P1-03`
  (`SessionAllTools` -> `"*"` all-tools unlock; EKO is the sole producer),
  `A-HITL-01-P1-02` (REPL EOF auto-approves; defeats the shared deadline),
  `A-HITL-01-P2-03` (denial escalation -> opaque errors on the EKO default
  path), `A-INT-01-P2-05` (browser confirmations inherit the REPL EOF and
  wildcard defects), `A-HITL-01-P1-01` (GUI rule management behaviorally
  dead), `F-HITL-01-P1-02` (user-modified args discarded — the original
  command executes).
- Secret/data-loss gaps (axis B): `A-OBS-01-P1-02` (webhook raw args/error
  text to external endpoints), `F-OPS-01-P2-01` (trace JSONL stores raw
  user input/output/error), `F-OPS-01-P2-04` (audit path unredacted),
  `F-SEC-01-P3-11` (raw URL logged in `web_fetch`).
- Over-gating residue (axis B): `A-HITL-01-P2-04` / `F-SEC-01-P3-04` (dead
  `IpcAuth::require_full_auto`), `A-INT-01-P2-03` (GUI MCP dialog
  private-range/https over-validation), `F-SEC-01-P3-03` (web_fetch SSRF
  over-gating), `F-SEC-01-P3-01` (eval `TestPass` unvalidated `sh -c`).
- Related: `A-TOOL-01-P1-01` / `F-EXT-01-P1-01` (writer subagents silently
  read-only via plan mode — the same subagent path as X-AUT-01-P1-01),
  `A-HITL-01-P2-01` (GUI provider bypasses the dispatcher — mechanism
  extended by X-AUT-01-P2-01).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Call-path classification per permission-sensitive operation (agent-auto vs direct user) | yes | passed | [V01-01](../validations/X-AUT-01/V01-01.md) |
| V02 | Default/full-auto mode matrix (surface normalization, framework dispatch, protected paths, propagation) | yes | passed | [V02-01](../validations/X-AUT-01/V02-01.md) |
| V03 | Local data-loss/secret protections (AGENTS.md three "when to protect" rules; indexed gaps) | yes | passed | [V03-01](../validations/X-AUT-01/V03-01.md) |
| V04 | Over-gating search (require_full_auto/permission_mode on user paths; residue inventory) | yes | passed | [V04-01](../validations/X-AUT-01/V04-01.md) |
| V05 | Cross-reference with canonical findings (F-HITL-01, A-HITL-01, A-OBS-01, F-OPS-01, A-INT-01, F-SEC-01) | yes | passed | [V05-01](../validations/X-AUT-01/V05-01.md) |

All required validations executed; all conclusions are static
(definition/reachability traces plus dependency-report test evidence);
no validation is pending. No executable command was required for this
cross-boundary slice — dependency reports F-HITL-01 V04 and A-HITL-01 V04
already establish green tests at these exact commits.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| AGENTS.md — permission modes (`full-auto`/`default` etc.) apply only to agent automated paths, not direct user interactions; over-gating removed | current on primary/pool and direct-user paths; **violated on subagent automation** | `PermissionStage` sole enforcement point; `create_terminal`/`connect_mcp_server` gate-free (terminal.rs:281-295, mcp.rs:211-216); zero `require_full_auto` callers (V04-01); X-AUT-01-P1-01 (subagents have no service) |
| AGENTS.md — `create_terminal`/`connect_mcp_server` `require_full_auto` gates removed | current (call sites gone); residue = dead `IpcAuth` API + stale doc | `src/tauri/error.rs:5-12,45-56`, zero callers (V04-01); A-HITL-01-P2-04 / F-SEC-01-P3-04 |
| AGENTS.md — protect (1) user data loss, (2) framework bugs, (3) secrets in logs | current for (1)(2) and framework sinks; partial for (3) at EKO boundaries | worktree.rs:705-724, terminal.rs:301-369, code.rs:305-345, redaction sinks (snapshot.rs:885, trace/mod.rs:434, spawn_task.rs:171/175); gaps A-OBS-01-P1-02, F-OPS-01-P2-01/04, F-SEC-01-P3-11 (V03-01) |
| A-HITL-01 — "Whether TaskRuntime plan subagents inherit the primary agent's provider/permission service is unresolved (F-SUB-02, X-AUT-01)" | resolved: they inherit neither; they have no service at all | infra.rs:881-1010 (no wiring) + snapshot.rs:857-862 (allow when None); X-AUT-01-P1-01 |
| A-HITL-01-P2-03 — background/pool runs fail closed via the empty dispatcher | current only when no GUI turn is active; defeated during active turns and by the shared handler slot | agent_pool.rs:955-975 vs chat.rs:570-582/712-719 + shared service (agent_pool.rs:928-929); X-AUT-01-P2-01 |
| A-HITL-01-P2-01 — GUI provider bypasses the dispatcher with per-conversation isolation (comment chat.rs:568-569) | regressed in part: "isolation" cannot hold on a shared handler slot | react/mod.rs:1592-1603,1617-1625 + service.rs:258-265; X-AUT-01-P2-01 |

## Coverage And Uncertainty

- All conclusions are static; no live approval or subagent run was executed
  (read-only review; Q-E2E-01 owns dynamic scenarios). P2-01's concrete
  user-visible impact (which flows actually collide in practice) is inferred
  from the fully traced mechanism; the mechanism itself is a code fact.
- The exact set of tools installed on writer/readonly subagents (MCP tools
  included or not) was not exhaustively enumerated; the finding's scope
  covers whatever non-blocklisted tools exist (run_code/web/browser/git/
  data verified as not in the plan-mode blocklist; shell/file/delete_file
  blocked by plan mode).
- Whether EKO ever runs TaskRuntime subagents with a provider in some
  configuration was not found (grep shows no wiring site); any future
  wiring must be tested against P1-01's regression list.
- Frontend approval rendering (ApprovalCard routing per message_key) is
  A-SRF-03/A-FE-01 scope; P2-01's "card in the wrong conversation"
  consequence assumes the emitted event carries the owning handler's
  message_key (verified at chat.rs:570-582 construction).
- InteractionMode (GUI routing mode, chat.rs:587-591) is separate from the
  permission mode and was not re-audited here (A-HITL-01/A-SRF scope).

## Handoff

- Downstream tasks may rely on: the per-surface call-path classification
  (V01-01); the four-mode matrix and its propagation differences (V02-01);
  the verified protection inventory and the four indexed secret gaps
  (V03-01); the over-gating inventory (V04-01); the canonical matrix
  (V05-01); the resolution of A-HITL-01's subagent question (P1-01);
  the process-global handler slot (P2-01); the CLI pool-propagation gap
  (P3-01).
- Reports to read: this report + the five validation reports in
  `validations/X-AUT-01/`; dependency reports F-HITL-01, F-SEC-01,
  A-HITL-01, A-INT-01.
- `X-SRF-01` should add rows: subagent permission surface (absent),
  CLI mode propagation (missing), GUI/background approval routing through
  the shared handler.
- `S-RDM-01` ordering: P1-01 (subagent boundary) before P2-01 (handler
  slot); both before the mode-matrix cleanup; P3-01 is a one-line parity
  fix. Framework-side fix option (no-service -> deny) belongs to F-HITL-01
  follow-up and must be coordinated with A-TOOL-01-P1-01.
- `Q-TST-01`/`Q-E2E-01` should exercise: subagent `run_code`/web calls in
  default and strict modes (P1-01); two-conversation GUI approval
  isolation and post-turn approvals (P2-01); CLI `/permission` then a
  channel/background run (P3-01); piped-stdin approvals (A-HITL-01-P1-02).
- Stale triggers: changes to `infra.rs` subagent factories (permission
  wiring), `agent_pool.rs` shared-resource extraction or `apply_permission_mode`,
  `react/mod.rs` `set_human_loop_provider*` / `set_permission_mode`,
  `service.rs` `replace_provider` or the mode dispatch,
  `snapshot.rs` `check_tool_approval` no-service fallback, `chat.rs`
  provider install/restore, `coding.rs` `cmd_permission`.
- Follow-up task IDs (fixes are not implemented in this review):
  X-SRF-01, Q-TST-01, Q-E2E-01, S-RDM-01.
