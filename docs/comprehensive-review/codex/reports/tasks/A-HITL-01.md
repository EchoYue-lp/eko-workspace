# A-HITL-01: Multi-surface human interaction policy

> Status: complete
> Reviewer: Codex primary reviewer (delegated static evidence independently sampled)
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: both source repositories clean; only Codex reports written

## Question

Does EKO arbitrate GUI/TUI/CLI/channel approval, input and selection through one identified deadline/cancellation contract while keeping direct user terminal/file/MCP interaction independent from Agent automation permission mode?

## Scope

- `HitlDispatcher`, REPL/TUI/channel/Tauri HumanLoop providers and their pending-response state.
- Primary/pool Agent PermissionService/provider injection and concurrent conversation reachability.
- GUI event filtering/store/cards and browser approval-provider lifecycle.
- GUI/TUI/CLI/channel automated permission-mode controls and propagation.
- Direct-user terminal, workspace file and MCP configuration call paths for permission-mode separation and product-boundary over-gating.
- Current tests, MASTER-PLAN claims and scoped history.

## Out Of Scope

- Framework PermissionService decision/cache/modified-args/request-envelope defects already owned by F-HITL-01.
- Tool/schema/sandbox implementation, UI visual quality, integration protocol internals and source fixes.
- Online multi-user/Web controls. The product is a local personal assistant.
- Cargo, rustc, tests, builds, dynamic fixtures, network calls and report indexes.

## Inputs

- Root `AGENTS.md`; shared review `README.md`, `REPORTING.md`, `TASKS.md`; Codex `README.md`.
- Completed Codex dependencies [F-HITL-01](F-HITL-01.md) and [A-BOOT-01](A-BOOT-01.md), used for generic contract/lifecycle ownership and de-duplication.
- Current source, frontend, tests, MASTER-PLAN and scoped git history. No other reviewer directory was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | F-HITL owns one typed, call-scoped approval result/request identity/deadline/cancellation contract and portable permission policy. |
| EKO product policy | Eligible GUI/TUI/CLI/channel provider selection, user-facing defaults, concurrent conversation projection and automated permission-mode controls are application policy. Direct terminal/file/MCP actions are user operations, not automated Agent decisions. |
| Adapter boundary | EKO must select one provider by immutable request/session/conversation context and preserve the complete framework envelope/result. It must not mutate a shared provider pointer per turn or create another cache/decision state machine. |
| Duplicate search | Searched both repositories for HumanLoop providers/dispatchers, PermissionService sharing/replacement, register/unregister, pending maps/options, request IDs/deadlines, cancel commands, browser providers, surface mode controls and IpcAuth/direct command checks. |
| Migration deletion | Keep the framework service and EKO's surface providers. Replace service-wide provider mutation with one source-scoped router/request context; delete singleton pending authorities, direct per-agent swap choreography, stale browser provider entries, inert IpcAuth and online-threat MCP allowlists after cutover. |

No framework API is classified dead because one EKO surface does not call it.

## Current Path

```text
bootstrap
  -> one PermissionService + HitlDispatcher(REPL)
  -> AgentPool extracts and shares the same Arc<PermissionService>
     -> every conversation Agent receives that Arc

TUI startup -> dispatcher removes REPL, registers TUI
GUI turn -> per-turn Tauri provider -> agent.set_provider -> shared service pointer replaced
channel sender -> shared Channel provider -> agent.set_provider -> same pointer replaced

permission check in any pooled Agent
  -> clones whichever shared handler was replaced most recently
  -> provider local pending state
     Tauri: backend HashMap, frontend singleton/filter-by-current-turn
     TUI: one Option<PendingApproval>
     channel: one Option<PendingRequest>, next unqualified message resolves it

BrowserRuntime
  -> bootstrap dispatcher default
  -> GUI inserts conversation provider per turn, never removes it

automation permission mode
  GUI/TUI -> primary + pool current/future
  CLI -> primary only
  channel -> no control

direct user terminal/file/MCP -> no permission_mode check
  MCP UI -> separate fixed launcher allowlist + public-HTTPS-only/private-host deny
```

Positive conclusions:

- Tauri backend pending requests are request-ID keyed and chat events carry message/conversation identity.
- TUI cleanup checks request identity before clearing newer state; GUI `cancel_chat` and TaskRuntime pause/cancel actively unblock matching Tauri pending futures.
- AgentPool correctly stores GUI/TUI permission-mode changes for existing and future pooled Agents.
- No live terminal/file/MCP command consults Agent automation permission mode. Workspace files use revision/path checks, and terminal confirmation is triggered by the user's first real input.
- Dispatcher snapshots providers without holding its registry lock and uses completion-order first-response with one outer deadline.

## Findings

### A-HITL-01-P1-01: Parallel conversation Agents overwrite one shared approval provider

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:120-159,183-212,824-930`; `echo-agent-cli/src/tauri/commands/chat.rs:568-582`; `echo-agent-cli/src/cli/channels.rs:131-152`; `echo-agent/src/agent/react/mod.rs:1617-1628`; `echo-agent/echo-orchestration/src/human_loop/service.rs:157,271-281,720-726`.
- Reachability: GUI explicitly supports parallel conversation Agents; channel pool creates per-sender Agents. All share one PermissionService, and every use replaces its one handler pointer.
- Expected invariant: policy may be shared, but each in-flight approval remains bound to the provider/session/conversation selected when the call began.
- Observed behavior: the most recent GUI/channel provider injection overwrites the handler for every pooled Agent. A later permission check clones that global current handler, regardless of the calling Agent.
- Impact: conversation A's dangerous action can appear in conversation B, channel approval can route to GUI/REPL or vice versa, and a response may authorize/reject the wrong visible context.
- Root cause: provider transport was treated as mutable service configuration while the service itself was promoted to a pool-wide shared resource.
- Direction: pass request/session/conversation source into one EKO provider router, or keep handler immutable per logical service while sharing only policy/rules. Delete per-turn `replace_provider_preserving_cache` choreography after cutover; do not add a second permission engine.
- Regression validation: two GUI conversations and two channel senders reach approval concurrently in adversarial injection order; assert exact provider/request/call identity and no cross-surface delivery.
- Validation reports: [V01](../validations/A-HITL-01/V01-01.md), [V09](../validations/A-HITL-01/V09-01.md)

### A-HITL-01-P1-02: GUI hides pending interaction requests outside the current turn

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/src/tauri/commands/chat.rs:114-142,210-257,283-430`; `echo-agent-cli/web-frontend/src/hooks/useTauriChat.ts:23-68`; `web-frontend/src/stores/chatStore.ts:42-44,418-423`; `components/chat/ChatPanel.tsx:290-325`.
- Reachability: backend accepts multiple request IDs from concurrent conversation Agents, but the app emits all through one global event channel and the frontend hook owns only the currently active message/conversation refs.
- Expected invariant: every pending request is stored/projected by conversation + message + request ID; switching surfaces reveals it and terminal cleanup removes that exact item.
- Observed behavior: non-current events are discarded, and the store/card surface has one approval/input/selection slot. A hidden request remains parked in the backend until timeout/cancel; a later request can replace the visible singleton.
- Impact: background/other-conversation tool execution appears hung and cannot be approved from the GUI despite the backend waiting for input.
- Root cause: identified backend concurrency was projected into current-chat singleton UI state.
- Direction: maintain a keyed pending interaction store and conversation badge/queue; deliver cleanup/timeout by request ID. Delete the three global singleton fields after migration.
- Regression validation: simultaneous approval/input/selection across active/inactive conversations, switching tabs, response/timeout/cancel, and exact card restoration.
- Validation reports: [V02](../validations/A-HITL-01/V02-01.md), [V05](../validations/A-HITL-01/V05-01.md), [V09](../validations/A-HITL-01/V09-01.md)

### A-HITL-01-P1-03: TUI and channel providers can represent only one pending request

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/hitl/tui_provider.rs:78-80,135-223`; `hitl/channel_provider.rs:24-35,76-128`; `echo-agent-cli/src/cli/channels.rs:35-68,131-152`.
- Reachability: one TUI provider serves its Agent and pooled/background work; one channel handler provider is injected into each of its per-sender pooled Agents.
- Expected invariant: concurrent tool/Subagent requests remain independently identified, visible and answerable.
- Observed behavior: TUI unconditionally overwrites its Option and drops the previous sender; channel explicitly rejects the previous request as superseded. Request-aware cleanup only prevents an old future clearing a newer singleton.
- Impact: valid parallel automated work is rejected because an unrelated request arrived later; only one approval/input/selection can be presented despite surface parity claims.
- Root cause: presentation state doubles as the authoritative pending request store.
- Direction: use a keyed/ordered pending registry and make the UI select/render requests; share a provider only when it can demultiplex by sender/conversation/request. Delete overwrite/supersede-as-normal semantics.
- Regression validation: concurrent same/different-kind requests, cancellation of one, response to another, newest/oldest display policy and no lost sender.
- Validation reports: [V03](../validations/A-HITL-01/V03-01.md), [V09](../validations/A-HITL-01/V09-01.md)

### A-HITL-01-P1-04: Provider deadlines, cancellation and default results diverge by surface and request kind

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent-cli/echo-agent-app-core/src/hitl/dispatcher.rs:89-163`; `hitl/tui_provider.rs:200-223`; `hitl/channel_provider.rs:110-128`; `echo-agent-cli/src/tauri/commands/chat.rs:236-255,330-430,805-826`.
- Reachability: every surface provider implements the same HumanLoopProvider trait and can receive approval/input/selection requests. GUI cancel and TaskRuntime pause/cancel call the shared pending cleanup.
- Expected invariant: one request deadline is honored end-to-end; cancellation, timeout, rejection and valid text are distinct typed outcomes with the same defaults across surfaces.
- Observed behavior: TUI/channel honor `req.timeout`, dispatcher adds a fixed outer five-minute deadline, and Tauri ignores the request timeout. GUI input timeout/drop/cancel fabricates empty Text while approval becomes rejection and selection becomes timeout; channel/TUI use different outcomes and cleanup mechanisms.
- Impact: an identical automated action can continue with empty user input, reject, or time out depending on UI; short caller deadlines are ignored in GUI and parked state can outlive the winning dispatcher request.
- Root cause: each adapter owns timeout/default policy instead of preserving one absolute deadline and typed terminal result.
- Direction: propagate one absolute deadline/cancellation token in the request envelope and use one application default mapping. Delete provider-local fixed sleeps and synthetic cross-kind Approval cancellation messages.
- Regression validation: every kind/surface for response, disconnect, cancel-before/during-wait and short deadline; assert one terminal, bounded wall time and exact cleanup.
- Validation reports: [V04](../validations/A-HITL-01/V04-01.md), [V05](../validations/A-HITL-01/V05-01.md), [V09](../validations/A-HITL-01/V09-01.md)

### A-HITL-01-P1-05: CLI and channel cannot apply the same automation permission policy as GUI/TUI

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/src/tauri/commands/panels.rs:38-79`; `echo-agent-cli/src/tui/events.rs:3576-3606`; `echo-agent-cli/src/cli/cmd_impls/coding.rs:638-690`; `echo-agent-cli/src/cli/command.rs:83-108`; `echo-agent-cli/src/cli/channels.rs:305-330`; `echo-agent-cli/echo-agent-app-core/src/agent_pool.rs:465-491,928-932`.
- Reachability: GUI/TUI/CLI commands are user-facing; CLI ChatResources and channel both execute through pooled Agents.
- Expected invariant: default/auto-edit/full-auto/strict are application policy exposed consistently and update primary, existing pool Agents and future pool Agents.
- Observed behavior: GUI/TUI update primary and pool. CLI updates only its primary Agent because CommandContext has no pool; channel exposes interaction `/mode` but no automation permission mode. Current/future pooled work can therefore retain a different policy from the surface's displayed/selected policy.
- Impact: CLI user selecting strict/full-auto sees a policy that does not govern background/pooled execution; channel users cannot inspect/change the equivalent policy, violating TUI/GUI/channel feature parity.
- Root cause: mode mutation is implemented independently in surface handlers instead of one application service.
- Direction: centralize permission-mode state/update in app-core and project it into every surface; update primary/pool current/future exactly once. Keep it strictly scoped to Agent automation.
- Regression validation: change/query each mode from every interactive surface, then execute primary and newly/existing pooled Agent actions and compare decisions.
- Validation reports: [V06](../validations/A-HITL-01/V06-01.md), [V09](../validations/A-HITL-01/V09-01.md)

### A-HITL-01-P1-06: GUI MCP configuration blocks trusted local extensions under an online XSS/SSRF threat model

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/src/tauri/commands/mcp.rs:110-207,210-239,561-613`; root `AGENTS.md` local product boundary.
- Reachability: every GUI `connect_mcp_server` request passes through these validators before constructing the user's server config.
- Expected invariant: user-configured local extensions accept arbitrary explicit trusted executables and local/private endpoints, with lightweight validation for empty commands, obvious metacharacter mistakes and insecure remote HTTP.
- Observed behavior: a fixed launcher basename allowlist rejects custom binaries/shell wrappers, while URL validation rejects all HTTP, localhost and private ranges explicitly to prevent XSS/SSRF. Tests enshrine the online model.
- Impact: common local MCP servers and user-authored trusted launchers cannot be connected from the GUI, making a core local-assistant extension capability unusable.
- Root cause: a public/multi-user Web threat model was applied to a direct user configuration path.
- Direction: remove capability-level executable/private-network deny rules; retain parse/empty/metacharacter and remote cleartext warnings/light validation. Delete the stale XSS/SSRF rationale and fixed allowlist tests.
- Regression validation: custom executable, localhost HTTP, private HTTPS, public HTTPS, malformed URL, obvious shell-injection typo and explicit user confirmation/diagnostic behavior.
- Validation reports: [V07](../validations/A-HITL-01/V07-01.md), [V08](../validations/A-HITL-01/V08-01.md), [V09](../validations/A-HITL-01/V09-01.md)

### A-HITL-01-P2-07: HitlDispatcher is not the canonical multi-surface authority and leaves stale browser providers

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/runtime.rs:127-145`; `echo-agent-cli/src/main.rs:247-257`; `echo-agent-cli/src/tauri/commands/chat.rs:568-589,681-719`; `echo-agent-cli/src/cli/channels.rs:145-152`; `echo-agent-cli/echo-agent-app-core/src/browser/mod.rs:145-167,978-1007`.
- Reachability: bootstrap, all production surface entry branches and BrowserRuntime confirmation use these provider owners.
- Expected invariant: one source-scoped application router owns provider registration, eligibility, first-response/cancel and removal; agents/browser consume that authority.
- Observed behavior: dispatcher owns REPL or TUI only. GUI/channel bypass it through direct Agent mutation. Browser keeps dispatcher as default plus a per-conversation provider inserted each GUI turn; the existing remove API has no production caller. After chat, Agent gets an empty provider but Browser retains the old handler/message key.
- Impact: there are multiple conflicting provider authorities, browser provider maps grow/stale, and later confirmation can emit into an obsolete turn that the frontend filters out.
- Root cause: surface integration evolved via replacement shortcuts without completing the dispatcher migration/lifecycle.
- Direction: make one EKO router own source-scoped provider leases and browser/Agent consumers; remove leases on turn/window/channel shutdown. Delete direct swaps and unowned map entries after cutover.
- Regression validation: connect/disconnect/switch every surface, first-responder and loser cleanup, browser action after turn completion, provider count returns to baseline and no stale event key.
- Validation reports: [V05](../validations/A-HITL-01/V05-01.md), [V08](../validations/A-HITL-01/V08-01.md), [V09](../validations/A-HITL-01/V09-01.md)

## Validation Matrix

| ID | Claim or execution | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Commit/source-clean snapshot | yes | passed | [report](../validations/A-HITL-01/V00-01.md) |
| V01 | Shared PermissionService/provider routing | yes | failed | [report](../validations/A-HITL-01/V01-01.md) |
| V02 | GUI pending identity/projection/cardinality | yes | failed | [report](../validations/A-HITL-01/V02-01.md) |
| V03 | TUI/channel concurrent pending capacity | yes | failed | [report](../validations/A-HITL-01/V03-01.md) |
| V04 | Deadline/cancel/default matrix | yes | failed | [report](../validations/A-HITL-01/V04-01.md) |
| V05 | Dispatcher/Browser provider registration lifecycle | yes | failed | [report](../validations/A-HITL-01/V05-01.md) |
| V06 | Surface permission-mode parity | yes | failed | [report](../validations/A-HITL-01/V06-01.md) |
| V07 | Direct-user versus Agent action boundary | yes | failed MCP; positive mode separation | [report](../validations/A-HITL-01/V07-01.md) |
| V08 | Test/history inventory and classification | yes | passed inventory | [report](../validations/A-HITL-01/V08-01.md) |
| V09 | Dynamic concurrency/deadline/surface matrix | no per instruction | not run; future | [report](../validations/A-HITL-01/V09-01.md) |
| V99-01 | Final report-integrity gate, incorrect executor predicate | yes | inconclusive; not adopted | [report](../validations/A-HITL-01/V99-01.md) |
| V99-02 | Corrected final report-integrity and source-boundary gate | yes | passed | [report](../validations/A-HITL-01/V99-02.md) |
| V30 | Primary source sampling and acceptance | yes | passed | [report](../validations/A-HITL-01/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| F-HITL: EKO selects/injects provider while framework owns portable policy | current target, violated by shared mutable transport | [V01](../validations/A-HITL-01/V01-01.md) |
| MASTER-PLAN M10: GUI/TUI/CLI/channel HITL parity complete | regressed/overbroad | Pending cardinality, routing, deadline and mode gaps remain; V02-V06. |
| MASTER-PLAN: cancel/pause during GUI HITL unblocks pending wait | current but kind-lossy | `cancel_pending_hitl` is live, but synthesizes Approval for all request kinds; [V04](../validations/A-HITL-01/V04-01.md). |
| MASTER-PLAN: TUI cancellation clears only the same request | current | Request-id Drop guard prevents clearing a newer card; [V03](../validations/A-HITL-01/V03-01.md). |
| Direct interactive terminal is not governed by Agent permission mode | current | No live mode gate; frontend confirms on first user input; [V07](../validations/A-HITL-01/V07-01.md). |
| Local-threat-model MCP restrictions still require audit | current unresolved item | Fixed launcher/private-host blocks remain; [V07](../validations/A-HITL-01/V07-01.md). |
| A-BOOT: one process lifecycle owner is missing | current dependency, not duplicated | Provider leases/cleanup should join that application lifecycle rather than add another root owner. |

## Coverage And Uncertainty

- No Cargo, rustc, tests, builds, UI launch, IM account, dynamic fixture or network call ran. Exact race timing and rendered visibility remain future V09 evidence.
- Static Arc/lock ownership is conclusive that pool Agents share one mutable handler pointer and that provider stores/UI cards are singleton.
- F-HITL-01 remains canonical for modified arguments, approval cache/session identity, Ask routing and lossy framework request conversion. This report does not duplicate those findings.
- Direct browser IPC actions with `effect=none` remain usable; the stale-provider issue applies to consequential confirmations and provider lifecycle, not every browser command.
- Changing PermissionService sharing, provider registration, pending stores, Tauri event filtering, browser provider maps, surface mode commands or MCP validation stales this report.

## Handoff

- Fix A-HITL-01-P1-01 before relying on any multi-conversation approval test; otherwise downstream surface behavior is nondeterministic by injection order.
- Build one app-core provider router with request/session/conversation identity and lease cleanup; keep framework PermissionService as the single policy authority.
- Use one keyed pending-interaction model for GUI/TUI/channel projection and one absolute deadline/typed terminal matrix.
- Centralize automation permission-mode update across primary/pool current/future Agents, while preserving the verified absence of such gates on direct terminal/file/MCP paths.
- Remove MCP online-threat over-gating under the root product boundary; retain lightweight malformed-input validation.
- V30 independently reconstructed and accepted all seven findings from current source. V09 remains future and is not a review blocker.
