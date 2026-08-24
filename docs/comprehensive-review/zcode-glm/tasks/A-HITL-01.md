# A-HITL-01: Multi-surface human interaction policy

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa (read-only; consumes `HumanLoopProvider` /
> `PermissionService` contract from F-HITL-01)
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only review)

## Question

Does EKO arbitrate TUI/GUI/channel approvals within one shared deadline
without gating direct user interactions as agent automation?

## Scope

Primary source paths and behaviors inspected (all under
`echo-agent-cli/` at commit `b3b2e81`, application layer):

- `echo-agent-app-core/src/hitl/mod.rs` (15 lines) — module exports.
- `echo-agent-app-core/src/hitl/dispatcher.rs` (full, 167 lines) —
  `HitlDispatcher`: parallel fan-out, 5-min shared deadline,
  first-responder-wins, fail-closed default-Reject.
- `echo-agent-app-core/src/hitl/repl_provider.rs` (full, 183 lines) —
  `ReplHumanLoopProvider`: stdin/stdout blocking approval/input/selection.
- `echo-agent-app-core/src/hitl/tui_provider.rs` (full, 351 lines) —
  `TuiHumanLoopProvider`: pending-slot + oneshot, request_id-guarded
  cleanup, 300s fallback timeout.
- `echo-agent-app-core/src/hitl/channel_provider.rs` (full, 292 lines) —
  `ChannelHumanLoopProvider`: per-sender pending slot, supersede-then-reject
  previous, broadcast prompt, 300s fallback timeout.
- `echo-agent-app-core/src/runtime.rs:127-146` — bootstrap wiring of
  `HitlDispatcher` (REPL provider registered) +
  `set_human_loop_provider` + `build_permission_service` on the primary
  agent; `browser_runtime.set_default_approval_provider` hand-off.
- `echo-agent-app-core/src/agent_pool.rs:455-489, 920-979` — pool's
  per-agent `set_permission_mode`, per-agent empty-`HitlDispatcher`
  default (`agent_pool.rs:966`), `apply_permission_mode` fan-out.
- `echo-agent-cli/src/main.rs:240-345` (TUI entry) and
  `:356-435` (CLI / channels-only branches) — entry-point provider
  registration ("repl" → unregister → "tui"); headless dispatcher hand-off
  to `start_headless_services`.
- `echo-agent-cli/src/cli/modes.rs:32-90` — `start_headless_services`
  plumbing of the dispatcher into the throwaway `AppState`.
- `echo-agent-cli/src/cli/channels.rs:40-150, 195-292` — per-sender
  `ChannelHumanLoopProvider` constructed in `AppChannelMessageHandler::new`
  (`:58`), `set_human_loop_provider_preserving_approvals` per-sender
  (`:149`), prompt broadcast subscription (`:235`).
- `echo-agent-cli/src/tauri/desktop.rs:180-220` — GUI dispatcher
  hand-off to `AppState::from_shared` (`:189`).
- `echo-agent-cli/src/tauri/commands/chat.rs:210-260` (PENDING_RESPONSES
  + cancel), `:261-435` (`TauriHumanLoopHandler` — per-conversation
  provider), `:568-589` (per-turn handler install), `:700-731`
  (post-turn empty-dispatcher reset), `:829-891` (IPC resolve commands).
- `echo-agent-cli/src/tauri/error.rs` (full, 124 lines) — `IpcAuth`,
  `IpcPermission::FullAuto` / `NotStrict`, `require_full_auto`,
  `require_not_strict` (dead code; see finding A-HITL-01-P2-02).
- `echo-agent-cli/src/tauri/terminal.rs:278-365` — `create_terminal`
  (no permission gate, comment explicit), `write_terminal`
  (per-session consent + 64 KiB cap + audit log), `confirm_terminal_consent`.
- `echo-agent-cli/src/tauri/ipc.rs:1-152` — `native_read_file`,
  `native_write_file`, `native_open_path` (path validation only; no
  permission_mode gate).
- `echo-agent-cli/src/tauri/commands/mcp.rs:110-258` —
  `validate_ipc_mcp_stdio` (executable allowlist + metacharacter deny),
  `validate_ipc_mcp_url` (https-only + private-range deny),
  `connect_mcp_server` (validation only; no permission_mode gate).
- `echo-agent-cli/src/tauri/commands/panels.rs:30-80` —
  `set_permissions_mode` IPC command (writes `config.permission_mode`,
  propagates to primary + pool agents).
- `echo-agent-cli/src/tui/events.rs:1664-1694` — `run_local_shell`
  (TUI `!<shell>` direct shell escape; no PermissionStage).
- `echo-agent-cli/web-frontend/src/lib/permissionModes.ts` (full, 32 lines)
  — the four user-selectable modes + alias normalization.
- `echo-agent/echo-core/src/tools/permission.rs:55-130` (read-only) —
  `PermissionMode` variants, `allows_write` / `requires_interaction`.
- `echo-agent/src/agent/react/mod.rs:2040-2075` — runtime
  `set_permission_mode` string → `PermissionMode` mapping
  (`full-auto → BypassPermissions`, `auto-edit → AcceptEdits`,
  `strict → StrictConfirm`, default → `Default`); the loud
  bypass-disabled warning.
- `echo-agent/src/agent/react/run/pipeline.rs:269-347` (read-only;
  inherited from F-HITL-01) — `PermissionStage` is stage 6 of the
  tool-execution pipeline; runs only on agent tool calls.

## Out Of Scope

Deferred to downstream / sibling task IDs:

- The framework permission composition (`PermissionService` 8-step
  pipeline, `SessionApprovalCache`, `ProtectedPathChecker`,
  `Classifier`, `TimeoutStrategy`) — owned by **F-HITL-01** (complete).
  This task consumes F-HITL-01's contract and does not re-audit it.
- The interactive terminal PTY separation from `run_code` — owned by
  **A-TOOL-01** (complete). This task re-uses its conclusion that the
  GUI PTY and the agent `run_code` sandbox share no state.
- The chat-turn lifecycle (one entry per turn across TUI/REPL/Tauri/
  channels) — owned by **A-CHAT-01** (complete). This task inherits
  the four-caller reachability map.
- Subagent approval inheritance — owned by **F-SUB-01 / F-SUB-02**.
  This task notes one cross-cutting observation (Coverage) but does not
  audit subagent semantics.
- The `TauriChatSink` persistence asymmetry — owned by **A-CHAT-01-P2-01**.

## Inputs

Required repository documents read in full:

- Repository root `AGENTS.md` via system reminder. Load-bearing
  sections: product positioning and security boundary ("threat model
  is local", "don't apply web-service threat model",
  "permission_mode controls agent automation only, not user-interactive
  tools", "the historical `require_full_auto` gates on
  `create_terminal`/`connect_mcp_server` are gone"), the framework-vs-
  application layering gate, the "first check if it already exists" rule,
  the dead-code cleanup rule, multi-mode functional parity (TUI/GUI/
  CLI/channel must be feature-equivalent), and the Claude Code / Codex
  research rule.
- `docs/comprehensive-review/REPORTING.md`.
- `docs/comprehensive-review/templates/task-report.md`,
  `templates/validation-report.md`.

Dependency reports read:

- `zcode-glm/tasks/F-HITL-01.md` (complete). Establishes: the framework
  provides generic `PermissionService` + `HumanLoopProvider` primitives;
  the `HitlDispatcher` is named as EKO's multi-provider fan-out with a
  5-min shared deadline and fail-closed default; timeout always maps to
  Deny at the framework seam; the framework `TimeoutStrategy` knob is
  write-only (dead). Load-bearing for V01/V02: this task sharpens the
  "shared deadline" claim — it is shared only within a single
  dispatcher fan-out, and GUI/Channels bypass the dispatcher entirely.
- `zcode-glm/tasks/A-BOOT-01.md` (complete). Establishes: every entry
  point constructs `AgentRuntime::bootstrap` once; the
  `HitlDispatcher` lives on `AppState`/`AgentRuntime` and is handed to
  `start_headless_services`. Load-bearing for V01: the dispatcher is
  constructed exactly once per process at `runtime.rs:129`.
- `zcode-glm/tasks/A-TOOL-01.md` (complete). Establishes: the only
  interactive terminal is the GUI PTY in `tauri/terminal.rs`; it shares
  no state with the agent's `SandboxManager` or `run_code`. `create_terminal`
  has no permission gate (comment explicit). Load-bearing for V03: the
  interactive-terminal carve-out is already audited; this task
  generalizes the conclusion to all direct-user IPC commands.
- `zcode-glm/tasks/A-CHAT-01.md` (complete). Establishes the four
  production `drive_chat` callers (TUI/REPL/Tauri/channels) and the
  sink-responsibility diff. Load-bearing for V01: the per-conversation
  agent wiring in `send_chat_message` (chat.rs:570-582) is where the
  GUI bypass is installed.

Historical documents treated as hypotheses:

- `echo-agent-app-core/src/hitl/dispatcher.rs:1-6` doc-comment —
  claims the dispatcher "routes approval/input requests to the
  currently active interface (WebSocket, TUI, REPL, Tauri)". Treated
  as **partially overstated**: it routes only when a surface actually
  registers a provider; the GUI and Channels never register with the
  dispatcher (they install per-session handlers bypassing it) — see
  finding A-HITL-01-P2-01.
- `echo-agent-app-core/src/hitl/dispatcher.rs:18-23` doc-comment —
  claims "Providers are tried in registration order. The first to
  respond wins." Treated as **current for the in-process fan-out path**,
  but operationally each entry point registers at most one provider, so
  the multi-provider path is unexercised — see A-HITL-01-P3-01.
- `echo-agent-cli/src/tauri/error.rs:1-10` module doc-comment — claims
  "Commands that spawn processes, write files outside the workspace, or
  execute arbitrary code are gated behind `IpcAuth::require_full_auto()`"
  and "`native_read_file` targeting `~/.ssh` are gated behind
  `IpcAuth::require_not_strict()`". Treated as **stale / falsified** —
  no IPC command calls either method (V03). See A-HITL-01-P2-02.

## Layering Decision

This is an **application-layer** task. All inspected code lives in
`echo-agent-cli` / `echo-agent-app-core` (the EKO product). The
framework side is consumed read-only via `HumanLoopProvider`,
`HumanLoopRequest`/`HumanLoopResponse`, `PermissionService`, and
`set_human_loop_provider` / `set_human_loop_provider_preserving_approvals`.

| Classification | Required answer |
|---|---|
| Generic mechanism | The framework supplies the right primitives: `HumanLoopProvider` trait, `PermissionRequestHandler` trait, `PermissionService`, `ReactAgent::set_human_loop_provider` and `set_human_loop_provider_preserving_approvals` (in-place provider swap with optional cache preservation), `PermissionMode` × `ToolPermission` → `PermissionDecision`. None of these depend on an EKO product decision. F-HITL-01 already classified this layering as clean. |
| EKO product policy | The `HitlDispatcher` (multi-provider fan-out + 5-min deadline), the per-surface provider implementations (`ReplHumanLoopProvider`, `TuiHumanLoopProvider`, `ChannelHumanLoopProvider`, `TauriHumanLoopHandler`), the per-conversation/per-sender provider install points, the `PENDING_RESPONSES` map + IPC resolver commands, the `IpcAuth` (dead) gate, the `permissionModes.ts` user-facing labels, and the choice of which surfaces use the dispatcher vs bypass it — all EKO product policy, correctly in `echo-agent-cli`. The framework never references any of these. |
| Adapter boundary | `ReactAgent::set_human_loop_provider_preserving_approvals` (`react/mod.rs:1617-1630`) is the thin seam: it calls `PermissionService::replace_provider_preserving_cache` (swap handler in place, keep mode/cache/classifier). The dispatcher and per-session handlers all implement `HumanLoopProvider`, so the framework treats them uniformly. No thickness, no scheduling authority on either side beyond what the trait contract specifies. |
| Duplicate search | Searched both repos for: `HitlDispatcher`, `TuiHumanLoopProvider`, `ReplHumanLoopProvider`, `ChannelHumanLoopProvider`, `TauriHumanLoopHandler`, `set_human_loop_provider`, `set_human_loop_provider_preserving_approvals`, `PENDING_RESPONSES`, `cancel_pending_hitl`, `register.*Provider`, `IpcAuth`, `require_full_auto`, `require_not_strict`, `apply_permission_mode`, `set_permission_mode`. Result: one canonical dispatcher (`hitl/dispatcher.rs:22`); four canonical per-surface provider implementations; one framework trait method for in-place swap (`react/mod.rs:1617`); one per-conversation pending map (`chat.rs:212`). No duplicate `HumanLoopProvider` impl, no second dispatcher. The `IpcAuth::require_full_auto` is the only duplicate-shaped artifact — it reimplements a "mode satisfies required-level" check that the framework's `PermissionMode` already expresses, but is unreachable (V03). |
| Migration deletion | A-HITL-01-P2-02 proposes deleting `IpcAuth`, `IpcPermission`, `require_full_auto`, `require_not_strict`, and the misleading module doc-comment in `src/tauri/error.rs`. A-HITL-01-P3-01 proposes simplifying `HitlDispatcher` (or actually registering multiple surfaces) to match operational usage. Neither is implemented in this review. |

## Current Path

### Verified surface → provider wiring (V01)

```text
AgentRuntime::bootstrap (runtime.rs:73)
   │  dispatcher = HitlDispatcher::new()                                  [:129]
   │  dispatcher.register("repl", ReplHumanLoopProvider)                  [:130-131]
   │  agent.set_human_loop_provider(dispatcher.clone())                   [:136]
   │  agent.build_permission_service()                                    [:137]
   │  browser_runtime.set_default_approval_provider(dispatcher.clone())  [:144-146]
   ↓
   ├─ REPL entry (run_cli_mode / run_repl_turn)
   │     dispatcher keeps the "repl" provider; no swap; approval reaches REPL stdin.
   │
   ├─ TUI entry (run_tui_or_cli_entry, feature="tui")                     [main.rs:240-345]
   │     dispatcher.unregister("repl")                                    [:253]
   │     dispatcher.register("tui", TuiHumanLoopProvider)                 [:254]
   │     tui_pending_handle → ratatui event loop poller                   [:252, 280]
   │     dispatcher.clone() → start_headless_services                     [:260]
   │     single pending slot keyed by request_id; cleanup on drop.
   │
   ├─ Channels entry (run_channels_mode, feature="channels")              [main.rs:357-405]
   │     AppChannelMessageHandler.hitl = ChannelHumanLoopProvider::new()  [channels.rs:58]
   │     PER INBOUND MESSAGE:
   │        pooled_agent.set_human_loop_provider_preserving_approvals(hitl)  [channels.rs:149]
   │        prompt_rx = hitl.subscribe_prompts()                          [channels.rs:235]
   │     THE GLOBAL DISPATCHER IS NEVER CONSULTED BY CHANNEL AGENTS.
   │
   └─ GUI entry (run_desktop, feature="gui")                              [desktop.rs:160-220]
         runtime.hitl_dispatcher.clone() → AppState::from_shared         [:189]
         PER CHAT TURN (send_chat_message):
            handler = TauriHumanLoopHandler::new(app, conv_id, message_key)  [chat.rs:570-574]
            pooled_agent.set_human_loop_provider_preserving_approvals(handler)  [chat.rs:575-582]
            browser_runtime.set_conversation_approval_provider(key, handler)   [chat.rs:586-589]
            AFTER drive_chat returns:
               empty = HitlDispatcher::new()                              [chat.rs:715]
               pooled_agent.set_human_loop_provider_preserving_approvals(empty)  [chat.rs:716]
         THE GLOBAL DISPATCHER IS NEVER CONSULTED BY GUI AGENTS during a turn.
```

**Three distinct surface patterns:**

1. **REPL / TUI — registered in dispatcher.** REPL registers "repl";
   TUI replaces it with "tui". The dispatcher is the live path. Single
   provider in the dispatcher at any time.

2. **Channels — bypass dispatcher; per-sender install.** Each inbound
   message re-installs the `ChannelHumanLoopProvider` on the pooled
   agent via `set_human_loop_provider_preserving_approvals`
   (`channels.rs:149`). The global dispatcher on `AppState` is not
   consulted by channel agents.

3. **GUI — bypass dispatcher; per-conversation-per-turn install.**
   `send_chat_message` builds a fresh `TauriHumanLoopHandler` keyed to
   `conversation_id` + `message_key` (`chat.rs:570`) and installs it on
   the pooled agent for that one turn. After the turn, an empty
   `HitlDispatcher::new()` is installed (`chat.rs:715-716`) — fail-closed.

The pooled-agent default at `agent_pool.rs:966` is also an empty
`HitlDispatcher::new()` — so any pooled agent that has not yet been
swapped (e.g. a subagent run that does not install its own provider)
auto-rejects ("No HITL provider available", `dispatcher.rs:82-87`).

### Agent-tool approval path (V03)

```text
agent_tool_call
   ↓ execute_tool_with_policy                                       [snapshot.rs:1189]
   ↓ ToolExecutionPipeline::default_pipeline().stages                [pipeline.rs:943]
   ↓ ...
   ↓ PermissionStage (stage 6)                                       [pipeline.rs:269]
   │   permission_hook → block / mode_override / decision            [:294-326]
   │   snapshot.check_tool_approval(name, input, mode_override)      [:330]
   ↓ AgentRunSnapshot::check_tool_approval                           [snapshot.rs:798]
   │   tokio::select! { cancel → Cancelled ; service.check_with_permissions_in_mode(..) }
   ↓ PermissionService::check_with_permissions_in_mode               [service.rs:484]
   │   STEP 0  protected_paths → Deny (overrides Bypass)
   │   STEP 1  BypassPermissions? → Allow (or Deny if bypass_disabled)
   │   STEP 4  rules.check (deny-first)
   │   STEP 5  cache hit → Allow
   │   STEP 6  mode dispatch → needs handler?
   ↓ check_with_handler (only when mode requires confirmation)       [service.rs:707]
   │   handler.handle(PermissionRequest) → PermissionResponse
   │     ↳ DynProviderHandler → HumanLoopProvider::request
   │           ↳ HitlDispatcher (REPL/TUI)  OR
   │           ↳ TauriHumanLoopHandler (GUI per-turn)  OR
   │           ↳ ChannelHumanLoopProvider (per-sender)
   │   map Allowed/Denied/Timeout → PermissionDecision
```

PermissionStage runs **only inside the agent tool pipeline**, reached
via `execute_tool_with_policy`. The framework side of this path is
owned by F-HITL-01; this task confirms the application wiring hands
the right provider to that path at the right time.

### Direct-user interaction path (V03)

Direct-user commands do **not** enter `execute_tool_with_policy`. They
go through Tauri IPC commands (GUI) or ratatui key handlers (TUI),
each with input validation only:

| Direct-user action | Location | Gate |
|---|---|---|
| GUI `create_terminal` | `tauri/terminal.rs:278` | None (comment explicit: "no permission gate here", `:286-289`) |
| GUI `write_terminal` | `tauri/terminal.rs:300` | Per-session consent flag (`confirm_terminal_consent`) + 64 KiB cap + audit log; **not** a permission_mode gate |
| GUI `connect_mcp_server` | `tauri/commands/mcp.rs:211` | `validate_ipc_mcp_stdio` (executable allowlist) + `validate_ipc_mcp_url` (https-only + private-range deny); **not** a permission_mode gate (comment `:217-221`) |
| GUI `native_read_file` | `tauri/ipc.rs:28` | `path_validator::validate_ipc_path(.., true)` only |
| GUI `native_write_file` | `tauri/ipc.rs:47` | `validate_ipc_path(.., false)` + 10 MiB cap |
| GUI `native_open_path` | `tauri/ipc.rs:119` | `validate_ipc_path` + argument-injection check |
| TUI `!<shell>` | `tui/events.rs:1664` (`run_local_shell`) | None; spawns `sh -lc` directly |
| TUI `$EDITOR` | `tui/events.rs:1723` (`open_external_file_editor`) | None; invokes user's `$EDITOR` |

The two paths (agent automation → PermissionStage; direct-user → input
validation) are cleanly **separate**. No direct-user command calls
`IpcAuth::require_full_auto` or `require_not_strict` (V03 confirms
zero call sites for both).

### Timeout behavior (V02)

| Path | Default timeout | On timeout |
|---|---|---|
| `HitlDispatcher` (REPL/TUI) | `TIMEOUT_DURATION = 5 min` single shared deadline (`dispatcher.rs:101-102`) | `HumanLoopResponse::Rejected { reason: "All HITL providers failed or timed out (...)" }` (fail-closed) |
| `TuiHumanLoopProvider` own timer | `req.timeout.unwrap_or(300s)` (`tui_provider.rs:201`) | `HumanLoopResponse::Timeout` (mapped to Deny at framework seam per F-HITL-01) |
| `ChannelHumanLoopProvider` | `req.timeout.unwrap_or(300s)` (`channel_provider.rs:110-112`) | `HumanLoopResponse::Timeout` |
| `TauriHumanLoopHandler` (GUI per-turn) | `tokio::time::sleep(300s)` race against oneshot (`chat.rs:350, 385, 426`) | `HumanLoopResponse::Timeout` |

All four paths converge on 300s / 5 min default. All map timeout →
`HumanLoopResponse::Timeout` or `Rejected` → framework's
`DynProviderHandler` maps both to `PermissionResponse::denied` →
`PermissionDecision::Deny`. Fail-closed everywhere; consistent with
F-HITL-01's V02 conclusion.

Framework never sets `HumanLoopRequest::timeout` (F-HITL-01 V02), so
the 300s default is provider-owned — same value across all four
provider implementations (likely convergent imitation rather than a
shared constant; no `const` is extracted).

### Permission-mode matrix (V04)

User-selectable modes (`web-frontend/src/lib/permissionModes.ts`):

| String | Framework `PermissionMode` | Behaviour (per F-HITL-01 V01 + V04) |
|---|---|---|
| `default` | `Default` | Read auto-allow; Write/Execute/Network/Sensitive → handler (approval required) |
| `auto-edit` / `accept-edits` | `AcceptEdits` | Read + Write auto-allow; Execute/Network/Sensitive → handler |
| `full-auto` / `bypass` | `BypassPermissions` | All tools auto-allow; **protected paths override** (`.git/.ssh/.env/.aws/...` still denied); loud `tracing::warn!` on activation (`react/mod.rs:2069-2073`) |
| `strict` / `strict-confirm` | `StrictConfirm` | Read auto-allow; Write/Execute/Network/Sensitive → handler (same surface as Default, slightly broader ask set) |

Legacy aliases `plan` and `auto` normalize to `default`
(`react/mod.rs:2048-2051`); read-only planning and Auto routing are
controlled by separate runtime modes (comment at `:2042-2045`).

**No tool requires `full-auto` for direct-user actions.** `full-auto`
only affects the agent-automation path (PermissionStage step 1). The
direct-user commands listed above don't read `permission_mode` at all
(verified by V03). The dead `IpcAuth::require_full_auto` would have
made some IPC commands require `full-auto`, but it is never called
(A-HITL-01-P2-02).

## Findings

### A-HITL-01-P2-01: GUI and Channels bypass the `HitlDispatcher` — multi-surface "single shared deadline" is not implemented in practice

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - GUI per-chat-turn install bypasses the dispatcher:
    `echo-agent-cli/src/tauri/commands/chat.rs:570-582` constructs a
    `TauriHumanLoopHandler` and calls
    `agent.set_human_loop_provider_preserving_approvals(handler)` on
    the pooled agent for that conversation. The dispatcher on
    `AppState.hitl_dispatcher` (stored at `desktop.rs:189`) is never
    consulted for agent approvals during a chat turn.
  - Channels per-sender install bypasses the dispatcher:
    `echo-agent-cli/src/cli/channels.rs:149` calls
    `agent.set_human_loop_provider_preserving_approvals(hitl)` on each
    pooled per-sender agent for every inbound message. The handler's
    own `ChannelHumanLoopProvider` (constructed at `channels.rs:58`)
    is the live approval channel.
  - The dispatcher's parallel fan-out + first-responder-wins + shared
    5-min deadline logic (`dispatcher.rs:101-155`) therefore runs only
    on the REPL/TUI path. Each surface has its **own** 300s timeout
    (`tui_provider.rs:201`, `channel_provider.rs:110-112`,
    `chat.rs:350/385/426`) and its **own** fail-closed mapping — there
    is no cross-surface coordination.
  - F-HITL-01's handoff describes the `HitlDispatcher` as EKO's
    multi-provider fan-out with a 5-min shared deadline. That is true
    *structurally*; operationally, the dispatcher's
    `FuturesUnordered` always contains at most one future because no
    entry point registers more than one provider simultaneously
    (`runtime.rs:130-131` registers "repl"; `main.rs:253-254` swaps it
    for "tui"; GUI/Channels never register).
- Reachability: every GUI chat turn and every channel inbound message.
  The bypass is the production path, not a corner case.
- Expected invariant: the task question — "Does EKO arbitrate
  TUI/GUI/channel approvals within one shared deadline". The answer
  is: only REPL/TUI share a deadline (single active surface);
  GUI/Channels do not arbitrate *across* surfaces — each surface /
  conversation / sender has its own isolated provider with its own
  300s timeout. The dispatcher's "single shared deadline" claim is
  true only *within* a single dispatcher fan-out, which in practice
  contains one provider.
- Observed behavior: per-session isolation. Each pooled agent gets
  the provider that matches its surface (GUI → TauriHumanLoopHandler,
  Channels → ChannelHumanLoopProvider, REPL/TUI → dispatcher with one
  registered provider). No cross-surface collision is possible
  because each pooled agent has exactly one provider at a time, and
  the framework calls only that provider.
- Impact: this is **correct** for the local-assistant positioning
  (AGENTS.md: "no multi-user / online threat model"; each conversation
  is a separate session). The cost is that the
  `HitlDispatcher`'s multi-provider fan-out logic is misleading: a
  future maintainer reading `dispatcher.rs:18-23` ("routes
  approval/input requests to the currently active interface")
  expects the dispatcher to be the unified surface arbiter, but it is
  not. The downstream F-HITL-01 V01 handoff propagates the same
  framing. A reviewer asking "what happens if TUI and GUI are open at
  the same time and both want approval?" would search the dispatcher
  in vain — the answer is "they're separate processes, each with its
  own dispatcher / handler, and they don't coordinate".
- Root cause: the dispatcher was designed early as a unified arbiter
  for an imagined multi-surface coexistence (the original doc-comment
  lists "WebSocket, TUI, REPL, Tauri" as concurrently active). The
  actual product model evolved toward per-session isolation (one
  process per surface, one provider per pooled agent), but the
  dispatcher's interface and doc-comment were not narrowed to match.
- Direction: pick one.
  (a) **Document and narrow (preferred under YAGNI)**: rewrite the
  `HitlDispatcher` doc-comment to state it is the REPL/TUI
  single-surface arbiter (single provider at a time), and that GUI /
  Channels install per-session providers via
  `set_human_loop_provider_preserving_approvals` and do not consult
  the dispatcher. Update F-HITL-01's V01 handoff wording
  accordingly. The 5-min shared deadline then reads as "bounds the
  REPL/TUI fan-out" rather than "bounds all surfaces".
  (b) **Actually unify (only if cross-surface arbitration is wanted)**:
  register the per-conversation Tauri handler and the per-sender
  Channel handler into the dispatcher instead of bypassing it. This
  requires the dispatcher to key pending requests by `(conv_id,
  request_id)` and route resolver commands accordingly — significant
  rework. Not justified by the current product model.
  Prefer (a). The product positioning (AGENTS.md) does not require
  cross-surface arbitration.
- Regression validation: under (a), no behaviour change — re-run
  `cargo test -p echo-agent-app-core --lib hitl::` (5 tests, V04) and
  add a doc-test asserting the new doc-comment wording. Under (b), a
  test that two registered providers resolve against the shared
  deadline (one fast-approve, one slow-reject → first wins) — but (b)
  is not the recommended path.
- Validation reports: [V01-01](../validations/A-HITL-01/V01-01.md),
  [V02-01](../validations/A-HITL-01/V02-01.md).

### A-HITL-01-P2-02: `IpcAuth` / `IpcPermission` / `require_full_auto` / `require_not_strict` is dead code with a misleading doc-comment

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/tauri/error.rs:39-70` defines `pub struct
    IpcAuth`, `pub fn require_full_auto(mode: &str) -> Result<...>`,
    `pub fn require_not_strict(mode: &str) -> Result<...>`, and
    `pub enum IpcPermission { FullAuto, NotStrict }` with its
    `is_satisfied_by` impl.
  - The module doc-comment (`error.rs:1-10`) claims: "Commands that
    spawn processes, write files outside the workspace, or execute
    arbitrary code are gated behind `IpcAuth::require_full_auto()`.
    ... Commands that read sensitive files (e.g. `native_read_file`
    targeting `~/.ssh`) are gated behind `IpcAuth::require_not_strict()`."
  - Repository-wide grep for `IpcAuth::` / `IpcPermission::` outside
    `error.rs` returns **zero** call sites (V03). Concretely:
    - `create_terminal` (`terminal.rs:278-297`) — comment at `:286-289`
      explicitly states "no permission gate here".
    - `connect_mcp_server` (`commands/mcp.rs:211-258`) — comment at
      `:217-221` explicitly states "we don't gate it behind a
      permission mode". Only input validation
      (`validate_ipc_mcp_stdio` / `validate_ipc_mcp_url`) runs.
    - `native_read_file` (`ipc.rs:28`) — only
      `path_validator::validate_ipc_path(.., true)`.
    - `native_write_file` (`ipc.rs:47`) — only
      `validate_ipc_path(.., false)` + 10 MiB cap.
    - `native_open_path` (`ipc.rs:119`) — only
      `validate_ipc_path` + argument-injection check.
  - The historical lesson recorded in AGENTS.md ("曾有一批安全 commit
    给 `create_terminal` / `connect_mcp_server` 加了
    `require_full_auto` 门控, 导致默认 `default` 权限下终端打不开、
    MCP 连不上 ... 这类门控已移除") confirms these gates were
    deliberately removed. The dead code in `error.rs` is the leftover.
- Reachability: definition → `pub` items on a `pub` module → zero
  callers anywhere in `echo-agent-cli` or `echo-agent`. Live in the
  API surface, dead at every call site.
- Expected invariant: a `pub` authorization gate advertised in the
  module doc-comment as the gate for "commands that spawn processes
  ... execute arbitrary code" should actually gate those commands.
  AGENTS.md code-cleanup rule: "delete superseded code, don't leave
  dead paths".
- Observed behavior: the gate is never invoked. The actual gates are
  input validation (path validator / executable allowlist /
  metacharacter deny / private-IP deny), per-session consent
  (`confirm_terminal_consent`), payload-size caps, and audit logs.
  None of these read `permission_mode`. The interactive-terminal
  carve-out is correct per AGENTS.md; the dead gate is the residue.
- Impact: misleading documentation (REPORTING.md P2 category
  "misleading public API / documentation"). A reviewer or security
  auditor reading `error.rs:1-10` will conclude that the codebase
  enforces a `full-auto` requirement on process-spawning IPC commands
  and a `not-strict` requirement on `native_read_file`. That is
  false. This cost is more serious than the dead-code maintenance
  burden alone — the misleading doc-comment actively wastes audit
  time. (The behaviour itself is correct under AGENTS.md's local-
  assistant positioning.)
- Root cause: the gate was added (the original "batch of security
  commits" referenced in AGENTS.md), then removed from the call sites
  (deliberately, per the historical lesson) but the gate definition
  and its doc-comment were left behind.
- Direction: delete `IpcAuth`, `IpcPermission`, `require_full_auto`,
  `require_not_strict`, and rewrite the `error.rs:1-10` module
  doc-comment to state the actual policy — "Tauri IPC commands that
  touch the filesystem or spawn processes use input validation (path
  validator, executable allowlist, URL scheme) and per-session user
  consent, not a `permission_mode` gate. The agent-automation path
  uses `PermissionStage` on `execute_tool_with_policy`; the two are
  intentionally separate (AGENTS.md security boundary)." Also drop
  the dead `IpcPermission` import wherever it leaked.
- Regression validation: `cargo build -p echo-agent-cli` after
  deletion; `cargo test --workspace --all-features --locked`; grep
  confirms zero remaining `IpcAuth` / `IpcPermission` references.
- Validation reports: [V03-01](../validations/A-HITL-01/V03-01.md),
  [V04-01](../validations/A-HITL-01/V04-01.md).

### A-HITL-01-P3-01: `HitlDispatcher`'s parallel-fan-out + first-responder-wins logic is over-engineered for current single-provider usage

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-app-core/src/hitl/dispatcher.rs:108-155` implements a
    `FuturesUnordered`-based parallel broadcast: each registered
    provider gets its own clone of the request, all race against a
    single shared 5-min deadline, the first `Ok(Ok(response))` wins,
    the remaining futures are dropped (cancelling them), and the
    `failures: Vec<String>` accumulates per-provider errors for the
    fail-closed message.
  - `runtime.rs:130-131` registers exactly one provider ("repl");
    `main.rs:253-254` replaces it with exactly one provider ("tui");
    GUI and Channels never register. The `FuturesUnordered` therefore
    always contains exactly one future on the live path.
  - The "first responder wins" semantics only differ from "the one
    provider responds" when there are two or more providers — a case
    that has never occurred in production.
- Reachability: every REPL/TUI approval request, but always with one
  future in the fan-out.
- Expected invariant: AGENTS.md YAGNI / "delete superseded code"
  rules — the multi-provider machinery is unused complexity.
- Observed behavior: the parallel fan-out behaves correctly as a
  single-provider pass-through. The `failures.join("; ")` message
  always contains at most one entry.
- Impact: low (no runtime defect). Maintenance cost: a future
  maintainer reading `dispatcher.rs` will infer the product supports
  multi-surface coexistence (which it does not — see A-HITL-01-P2-01).
  Combined with the misleading doc-comment (P2-01), the dispatcher
  reads as more general than it is.
- Root cause: the dispatcher was written for an imagined multi-surface
  scenario; the product evolved to per-session isolation.
- Direction: either (a) simplify to a single-provider wrapper (or
  remove the dispatcher entirely and install the REPL/TUI provider
  directly, mirroring GUI/Channels), or (b) actually register
  multiple surfaces (e.g. when the desktop app simultaneously hosts a
  TUI control panel — if such a use case ever materializes). Under
  (a), the 5-min shared deadline and fail-closed default still need
  to live somewhere (perhaps inside each per-surface provider, which
  already have their own 300s timeout — see V02).
- Regression validation: `cargo test -p echo-agent-app-core --lib
  hitl::dispatcher` (currently none — the dispatcher has no unit
  tests); add a test asserting the chosen semantics end-to-end.
- Validation reports: [V01-01](../validations/A-HITL-01/V01-01.md).

### A-HITL-01-P3-02: The GUI's post-turn empty-`HitlDispatcher` reset is an undocumented fail-closed safety net

- Priority: P3
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/tauri/commands/chat.rs:712-719` — after
    `drive_chat` returns, the spawned task installs an empty
    `HitlDispatcher::new()` on the pooled agent via
    `set_human_loop_provider_preserving_approvals(empty)`. No comment
    explains why.
  - The pooled-agent default at `agent_pool.rs:966` is also an empty
    `HitlDispatcher::new()`. The `agent_pool.rs:960-964` comment
    explains *that* default ("auto-reject instead of blocking on
    terminal stdin which hangs GUI"), but `chat.rs:712-719` does not
    reference the same rationale.
  - Net effect: between chat turns, a pooled GUI agent has an empty
    dispatcher. Any approval request from a background SubagentRun
    that shares the same pooled agent would auto-reject with "No HITL
    provider available" (`dispatcher.rs:82-87`).
- Reachability: every GUI chat turn exit. The empty dispatcher
  persists until the next `send_chat_message` installs a new
  `TauriHumanLoopHandler`.
- Expected invariant: safety-net code should be documented (AGENTS.md
  "本地场景下为何仍需要" rule for any added friction; symmetric for
  any added fail-closed behaviour).
- Observed behavior: fail-closed between turns. This is the correct
  default (better than silently letting a stale handler accumulate),
  but the rationale is implicit.
- Impact: low (the behaviour is correct). A maintainer changing the
  reset (e.g. to preserve the last handler for late subagent
  approvals) has no comment to read and may break the safety net
  unintentionally.
- Root cause: the reset was added when `send_chat_message` started
  installing per-turn handlers, to prevent the previous turn's
  handler from absorbing the next turn's approvals (which would route
  them to a dead `message_key`); the rationale was not written down.
- Direction: add a comment at `chat.rs:712-719` referencing
  `agent_pool.rs:960-964`'s rationale and stating: "Reset to an empty
  dispatcher so any approval request that races ahead of the next
  `send_chat_message` (e.g. a late subagent run on the same pooled
  agent) fails closed instead of routing to a stale `message_key`
  handler whose oneshot is already dropped." No code change required.
- Regression validation: documentation-only change; no test impact.
- Validation reports: [V01-01](../validations/A-HITL-01/V01-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Provider arbitration: surface → provider wiring map; one shared deadline claim; race-condition inventory | yes | passed (with finding) | [V01-01](../validations/A-HITL-01/V01-01.md) |
| V02 | Timeout/default behaviour: default 300s; on-timeout → Deny across all surfaces | yes | passed | [V02-01](../validations/A-HITL-01/V02-01.md) |
| V03 | Direct-user vs agent-action permission path separation; `IpcAuth` is dead code | yes | passed (with finding) | [V03-01](../validations/A-HITL-01/V03-01.md) |
| V04 | Default vs full-auto mode: what requires approval in each mode; targeted executable check | yes | passed | [V04-01](../validations/A-HITL-01/V04-01.md) |
| V05 | Historical-document drift | conditional | n/a | No prior A-HITL-01 report exists in this reviewer directory; the three doc-comments treated as hypotheses are classified inline in the Inputs section (two partially overstated → A-HITL-01-P2-01/P3-01, one stale → A-HITL-01-P2-02). |

Executed cargo command (exit 0):

```text
cd echo-agent-cli
cargo test -p echo-agent-app-core --lib hitl::     (5 passed, 0 failed)
```

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `hitl/dispatcher.rs:1-6` — "routes approval/input requests to the currently active interface (WebSocket, TUI, REPL, Tauri)" | partially overstated | V01 confirms the dispatcher routes only REPL/TUI requests; GUI installs `TauriHumanLoopHandler` per turn and Channels install `ChannelHumanLoopProvider` per sender — neither consults the dispatcher. Finding A-HITL-01-P2-01. |
| `hitl/dispatcher.rs:18-23` — "Providers are tried in registration order. The first to respond wins." | current (single-provider in practice) | V01 confirms the fan-out logic is correct but only one provider is ever registered at a time, so "first responder wins" is operationally "the one provider responds". Finding A-HITL-01-P3-01. |
| `tauri/error.rs:1-10` — "Commands that spawn processes ... are gated behind `IpcAuth::require_full_auto()`" | stale (regressed) | V03 confirms zero call sites for `IpcAuth::require_full_auto` / `require_not_strict` outside `error.rs`. The actual gates are input validation only. Finding A-HITL-01-P2-02. |
| AGENTS.md — "the historical `require_full_auto` gates on `create_terminal`/`connect_mcp_server` are gone" | current (supported) | V03 confirms `create_terminal` (`terminal.rs:278-297`) and `connect_mcp_server` (`commands/mcp.rs:211-258`) have no permission_mode gate; only the dead `IpcAuth` residue remains. |
| AGENTS.md — "permission_mode controls agent automation only, not user-interactive tools" | current (supported) | V03 confirms `PermissionStage` runs only inside `execute_tool_with_policy`; direct-user commands (`create_terminal`, `connect_mcp_server`, `native_read_file`, `native_write_file`, TUI `!<shell>`, `$EDITOR`) bypass permission_mode entirely. |
| F-HITL-01 handoff — "`HitlDispatcher` (5-min shared deadline, fail-closed)" | current (within dispatcher) | V01/V02 confirm the 5-min shared deadline + fail-closed apply *inside* the dispatcher fan-out (REPL/TUI only). GUI/Channels bypass and use their own 300s timeout. Sharpened in A-HITL-01-P2-01. |
| A-TOOL-01 handoff — "GUI PTY and agent `run_code` sandbox share no state" | current (adopted) | V03 generalizes A-TOOL-01's PTY-vs-`run_code` separation to all direct-user IPC commands vs the agent PermissionStage. |

## Coverage And Uncertainty

Inspected in full: every file under
`echo-agent-cli/echo-agent-app-core/src/hitl/`; the bootstrap wiring
in `runtime.rs:120-150`; the entry-point dispatcher hand-off in
`main.rs:240-435`, `desktop.rs:180-220`, `modes.rs:32-90`; the
channels per-sender install in `channels.rs:40-150, 195-292`; the
Tauri per-turn install + resolver commands in `chat.rs:210-260,
261-435, 568-589, 700-731, 829-891`; the full `error.rs` (124 lines);
the relevant slices of `terminal.rs`, `ipc.rs`, `commands/mcp.rs`,
`commands/panels.rs`, `tui/events.rs:1664-1694`; the framework
permission-mode mapping in `react/mod.rs:2040-2075`; the framework
`PermissionStage` slice in `pipeline.rs:269-347`.

Not inspected (out of scope or deferred):

- **Subagent approval inheritance.** A pooled subagent created by
  `create_complex_task` inherits its parent's `HumanLoopProvider` by
  default (framework default at `ReactAgent` build time), but the
  application may override. Whether subagent approvals reach the
  parent's `TauriHumanLoopHandler` (GUI) or auto-reject against the
  empty dispatcher (post-turn reset, A-HITL-01-P3-02) is owned by
  **F-SUB-01 / F-SUB-02**. Noted as a Coverage uncertainty because the
  post-turn reset (P3-02) interacts with subagent approval routing.
- **The `path_validator` internals.** Consumed as the actual gate for
  `native_read_file` / `native_write_file` / `native_open_path`. Its
  correctness (home-confinement, secret-denylist, symlink escape)
  belongs to a Tauri-path-security task, not the multi-surface HITL
  policy.
- **The browser-runtime approval-provider map**
  (`browser_runtime.set_default_approval_provider` /
  `set_conversation_approval_provider`). Read enough to confirm it
  mirrors the agent's provider install (default + per-conversation),
  but its internal routing is out of scope.
- **The `WebhookTurnObserver`** — consumed as a precedent for the
  correct cross-cutting-observer pattern (A-CHAT-01-P2-01). Not
  re-audited here.

Environmental constraints:

- `cargo test -p echo-agent-app-core --lib hitl::` ran against the
  existing incremental cache; 5 tests passed, exit 0. The
  `human-loop` feature is on the root `echo_agent` package, not on
  `echo-agent-app-core`; the app-core HITL module is feature-
  unconditional.
- No `cargo clean` was needed (disk pressure well below threshold).

Uncertain claims:

- Whether any future product configuration will want multi-surface
  coexistence (e.g. a desktop app simultaneously hosting a TUI
  control panel and a GUI conversation). If so, the dispatcher's
  parallel fan-out becomes useful and A-HITL-01-P2-01/P3-01 should
  be re-evaluated. The current product positioning (AGENTS.md) does
  not call for it.
- Whether the `IpcAuth` dead code is referenced by any external
  consumer of `echo-agent-cli` (it is `pub`, so external forks could
  in principle call it). The in-repo grep is clean; the misleading
  doc-comment is the stronger reason to delete.

## Handoff

Conclusions downstream tasks may rely on:

1. **Direct-user actions are correctly NOT gated by `permission_mode`.**
   Every interactive-user IPC command (`create_terminal`,
   `write_terminal`, `connect_mcp_server`, `native_read_file`,
   `native_write_file`, `native_open_path`) and every TUI direct
   action (`!<shell>`, `$EDITOR`) bypass `PermissionStage` and use
   input validation / per-session consent only. AGENTS.md's security
   boundary holds. Downstream tasks auditing "what gates a command"
   must classify each command as **agent automation** (goes through
   `PermissionStage` via `execute_tool_with_policy`) or **direct
   user** (input validation only) — these are the two canonical
   categories.
2. **`permission_mode` only affects the agent-automation path.**
   `full-auto` (BypassPermissions) auto-allows every agent tool call
   except protected paths (`.git/.ssh/.env/.aws/...`); `default`
   requires approval for Write/Execute/Network/Sensitive. The modes
   do not change direct-user command behaviour.
3. **Multi-surface approval arbitration is per-session, not cross-
   surface.** Each surface (REPL, TUI, GUI conversation, channel
   sender) has its own provider with its own 300s timeout. The
   `HitlDispatcher` is the REPL/TUI single-surface arbiter (one
   registered provider at a time), not a cross-surface coordinator.
   Tasks must not assume a shared deadline across surfaces.
4. **Timeout behaviour is uniformly fail-closed.** All four provider
   implementations (Dispatcher, TUI, Channel, Tauri) default to 300s
   and map timeout → Deny at the framework seam. This matches
   F-HITL-01's V02 conclusion and extends it to the application layer.
5. **`IpcAuth` is dead.** The `require_full_auto` /
   `require_not_strict` methods are never called; the module
   doc-comment in `tauri/error.rs:1-10` is false. Tasks touching
   Tauri security must not cite `IpcAuth` as an active gate.

Reports they must read:

- This report (A-HITL-01) for the surface → provider wiring map, the
  four findings, and the direct-user-vs-agent-action classification.
- `tasks/F-HITL-01.md` for the framework `PermissionService` /
  `HumanLoopProvider` composition, the dead `TimeoutStrategy` knob,
  and the protected-paths-override-Bypass invariant.
- `tasks/A-TOOL-01.md` for the PTY-vs-`run_code` sandbox separation
  and the per-mode visible/disabled tool matrix.
- `tasks/A-CHAT-01.md` for the four `drive_chat` callers and the
  per-conversation agent wiring in `send_chat_message`.

Conditions that make this report stale:

- Any change to `HitlDispatcher`'s fan-out logic or its registered
  providers (especially registering more than one provider
  simultaneously) invalidates V01 / A-HITL-01-P2-01 / P3-01.
- Any change that wires GUI or Channels through the dispatcher
  (resolving P2-01 direction (b)) invalidates V01's bypass claim.
- Any change that calls `IpcAuth::require_full_auto` /
  `require_not_strict` (resurrecting the gate) invalidates V03 /
  A-HITL-01-P2-02. Conversely, deleting them (P2-02 direction)
  invalidates V03's "dead code" claim — the validation would update
  to "deleted, classification: resolved".
- Any change to the four provider timeouts (300s) or their
  timeout → response mapping invalidates V02.
- Any new direct-user command that does go through `permission_mode`
  would invalidate V03's separation claim.

Follow-up task IDs (no fixes implemented in this review):

- A **Tauri security cleanup task** should action A-HITL-01-P2-02
  (delete `IpcAuth` / `IpcPermission` and rewrite the misleading
  module doc-comment). Same task can pick up A-HITL-01-P3-02 (add
  the missing rationale comment on the post-turn dispatcher reset).
- A **dispatcher scope-narrowing task** should action A-HITL-01-P2-01
  (rewrite the dispatcher doc-comment to match operational usage) and
  decide A-HITL-01-P3-01 (simplify the dispatcher or actually
  register multiple surfaces). The two should be resolved together.
- **F-SUB-01 / F-SUB-02** should confirm whether pooled subagent
  approval requests reach the parent's per-turn handler or auto-reject
  against the empty dispatcher (the P3-02 uncertainty).
