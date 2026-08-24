# X-AUT-01: Permission and local security boundary

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0fa
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only cross-cutting synthesis; both repos
> `git status --short` empty)

## Question

Are automated Agent actions controlled while direct user terminal,
file picker, MCP configuration, and browser interactions remain usable?

## Scope

This is a **cross-cutting synthesis task**. It consumes the four
dependency reports (F-HITL-01, F-SEC-01, A-HITL-01, A-INT-01) and
re-verifies the boundary they describe against the live code at the
pinned commits. Primary source paths inspected directly (not via the
dependencies):

- `echo-agent/src/agent/snapshot.rs:1189` — `execute_tool_with_policy`
  (the single agent-tool entry to the permission pipeline).
- `echo-agent/src/agent/react/run/pipeline.rs:269, 943` — `PermissionStage`
  registration and run site.
- `echo-agent/src/agent/react/run/phases/tools.rs:154, 321` — the only
  two live callers of `execute_tool_with_policy` (ReAct tool batches).
- `echo-agent/echo-core/src/tools/permission.rs:15-130` — `ToolPermission`
  (5 variants), `PermissionMode` (8 variants), `PermissionDecision`,
  `allows_write` / `requires_interaction`.
- `echo-agent/echo-orchestration/src/human_loop/service.rs:167-185,
  484-681` — per-mode `*_confirmation_required` helpers and the 8-step
  `check_with_permissions_in_mode` pipeline.
- `echo-agent/echo-orchestration/src/human_loop/protected.rs:24-59` —
  `DEFAULT_PROTECTED_PATTERNS` (`.git/.ssh/.env/.aws/...`).
- `echo-agent/src/agent/react/mod.rs:2040-2079` — runtime
  `set_permission_mode` string → `PermissionMode` mapping + the loud
  BypassPermissions warning.
- `echo-agent-cli/src/tauri/terminal.rs:40-50, 277-365` — `create_terminal`
  (no gate), `write_terminal` (per-session consent + 64 KiB cap + audit),
  `PtySession.consented` flag.
- `echo-agent-cli/src/tauri/commands/mcp.rs:110-258` —
  `validate_ipc_mcp_stdio` (executable allowlist),
  `validate_ipc_mcp_url` (https + private-range deny),
  `connect_mcp_server` (validation only; comment explicit).
- `echo-agent-cli/src/tauri/commands/browser.rs:9-150` — browser IPC
  commands delegating to `browser_runtime.execute_main(.., None)` (no
  permission gate, no cancel token).
- `echo-agent-cli/src/tauri/ipc.rs:25-152` — `native_read_file`,
  `native_write_file` (atomic temp+rename, 5 / 10 MiB caps),
  `native_open_path` (argument-injection guard).
- `echo-agent-cli/src/tauri/path_validator.rs:1-146` — the IPC path
  validator (home-confinement + `..` reject + secret-denylist
  `~/.ssh`/`~/.aws`/history/cookies).
- `echo-agent-cli/src/tauri/commands/panels.rs:30-80` —
  `get_permissions_mode` / `set_permissions_mode` (the only IPC that
  reads/writes `permission_mode`; it manages the mode, does not gate on it).
- `echo-agent-cli/src/tauri/error.rs:1-70` — the dead `IpcAuth` /
  `IpcPermission` / `require_full_auto` / `require_not_strict`.
- `echo-agent-cli/src/tui/events.rs:1664-1696, 3576-3605` — TUI `!<shell>`
  escape (no gate), `/permission` slash command (sets mode only).
- Live redaction surfaces cross-checked:
  `echo-agent/src/security.rs:82-140`, `tools/builtin/spawn_task.rs:171,175`,
  `trace/mod.rs:434`, `snapshot.rs:885-889`, `execution.rs:229-230`.
- `echo-agent-cli/src/tauri/commands/mcp.rs:383-474` —
  `redact_mcp_config_secrets` (the direct-user secret-protection on the
  MCP panel).

## Out Of Scope

Deferred to named task IDs:

- The framework `PermissionService` 8-step composition, `SessionApprovalCache`,
  `Classifier`, `TimeoutStrategy` write-only knob, dead `ReactAgent` HITL
  block — owned by **F-HITL-01** (complete). This task consumes its
  contract and re-checks only the boundary invariants.
- Framework sandbox / guard / secret-scanner internals, the
  `ContentGuard::Redact` no-op, `sha256_hex` misnaming, parallel
  `evolution/security.rs` scanner — owned by **F-SEC-01** (complete).
- Per-surface provider arbitration, dispatcher bypass, post-turn reset —
  owned by **A-HITL-01** (complete).
- MCP / browser / LSP reachability and the IPC over-validation specifics
  — owned by **A-INT-01** (complete). This task cites A-INT-01-P1-01 as
  the canonical over-gating instance and generalises the pattern.
- Frontend DTO parity, per-mode tool visibility matrix — **A-FE-01** /
  **A-TOOL-01**.

## Inputs

Required repository documents read in full:

- Root `AGENTS.md` via system reminder. Load-bearing sections for this
  task: "产品定位与安全边界" (the local-assistant threat model, the
  explicit exclusion of the web-service XSS/SSRF model, the rule that
  terminal / file picker are user actions not gated by `permission_mode`,
  the three valid protection categories: data-loss / framework-bug /
  local-universal secret safety, the historical `require_full_auto`
  removal lesson), the framework-vs-application layering gate, the
  no-duplicate-authority rule, and the code-cleanup rule.
- `docs/comprehensive-review/REPORTING.md`, `templates/task-report.md`,
  `templates/validation-report.md`.
- `docs/comprehensive-review/TASKS.md` X-AUT-01 card (dependencies +
  validation list).

Dependency task reports read in full:

- `zcode-glm/tasks/F-HITL-01.md` — establishes the framework permission
  composition, protected-paths-override-Bypass invariant, per-call mode
  override non-mutation, timeout → Deny everywhere. Load-bearing for
  V01/V02: the single agent-tool permission entry
  (`PermissionService::check_with_permissions_in_mode`) and the
  protected-paths-before-bypass ordering.
- `zcode-glm/tasks/F-SEC-01.md` — establishes the four live secret-
  redaction surfaces, sandbox fail-safe fallback, no `require_full_auto`
  in framework guard/sandbox/security. Load-bearing for V03: secret
  redaction is wired and correct.
- `zcode-glm/tasks/A-HITL-01.md` — establishes the direct-user vs agent-
  automation separation (V03), the surface → provider wiring map, the
  dead `IpcAuth` (P2-02), the dispatcher bypass (P2-01). Load-bearing
  for V01/V04: the two-path classification is canonical.
- `zcode-glm/tasks/A-INT-01.md` — establishes the IPC MCP over-validation
  (P1-01: executable allowlist + private-range block make user MCP
  servers unreachable), the asymmetric on-disk-vs-IPC validators. This
  is the canonical over-gating instance; X-AUT-01 generalises it.

Historical documents treated as hypotheses:

- AGENTS.md "历史教训 ... `require_full_auto` 门控已移除" — re-verified
  in V04 for terminal / file picker / MCP / browser / TUI shell escape.
  Classified **current (literal) but the excluded threat-model narrative
  persists in comments** — see finding X-AUT-01-P2-01.
- AGENTS.md "不要套用线上 Web 服务的威胁模型:诸如'防 XSS→RCE''防 SSRF
  内网穿透'" — re-verified in V04 by grep for `XSS`/`SSRF` across the
  Tauri IPC layer. Classified **regressed in comments (5+ sites)** — the
  online-service threat model is invoked as the justification for several
  gates and guards that should be justified by the local model.

## Layering Decision

This task spans both repositories but introduces no new code; it
synthesises the boundary classification.

| Classification | Answer |
|---|---|
| Generic mechanism (framework) | `PermissionMode` × `ToolPermission` → `PermissionDecision`, `PermissionService::check_with_permissions_in_mode`, `ProtectedPathChecker`, `SessionApprovalCache`, the secret-pattern catalog (`security.rs`), `PathValidator::validate_within_base`. Any `echo-agent` consumer needs these. Correctly placed in `echo-core` / `echo-orchestration` / `echo-tools` / root `echo_agent`. None of them encode an EKO product decision. |
| EKO product policy (application) | The four user-facing mode labels (`web-frontend/src/lib/permissionModes.ts`), the `set_permissions_mode` IPC, the per-surface provider wiring, the `path_validator.rs` secret-denylist (specific to IPC file-picker), the `validate_ipc_mcp_*` input gates, the `redact_mcp_config_secrets`, the `write_terminal` per-session consent. All correctly in `echo-agent-cli`. The framework never references these. |
| Adapter boundary | `execute_tool_with_policy` (`snapshot.rs:1189`) is the single agent-tool entry; `DynProviderHandler` adapts the EKO provider into the framework handler. Both thin. The direct-user IPC commands are not adapters — they bypass `execute_tool_with_policy` entirely, which is the intended separation. |
| Duplicate search | Searched both repos for `require_full_auto`, `require_not_strict`, `IpcAuth`, `IpcPermission`, `permission_mode` reads on interactive commands, `ALLOWED_MCP_STDIO_BASES`, `validate_ipc_path`, `validate_ipc_mcp_*`. Result: one canonical permission service (framework); one canonical IPC path validator (`path_validator.rs`); one canonical MCP IPC gate pair (`validate_ipc_mcp_stdio` / `validate_ipc_mcp_url`); one dead `IpcAuth` (zero callers). No second permission gate on direct-user actions. The over-gating that exists is in the MCP IPC validator (A-INT-01-P1-01), not a duplicate of the framework path. |
| Migration deletion | X-AUT-01-P2-01 recommends a comment-/threat-model-narrowing pass across the Tauri IPC layer (no gate removal beyond what A-INT-01-P1-01 and A-HITL-01-P2-02 already propose). X-AUT-01-P3-01 narrows the `path_validator.rs` doc-comment. Neither requires code deletion in this task. |

## Current Path

### 1. The two canonical paths (V01)

The boundary EKO draws has exactly two paths, and they share no code:

```text
AGENT AUTOMATION PATH (permission_mode applies)
  agent_tool_call (ReAct loop)
    ↓ phases/tools.rs:154 / :321   .execute_tool_with_policy(..)
    ↓ snapshot.rs:1189             execute_tool_with_policy
    ↓ pipeline.rs:943              ToolExecutionPipeline::default_pipeline().stages[5]
    ↓ pipeline.rs:269, 330         PermissionStage → snapshot.check_tool_approval
    ↓ snapshot.rs:798              AgentRunSnapshot::check_tool_approval
    ↓ service.rs:484               PermissionService::check_with_permissions_in_mode
       STEP 0  protected_paths.check → Protected? → Deny (overrides Bypass)
       STEP 1  BypassPermissions? → Allow (or Deny if bypass_disabled)
       STEP 2  Plan? → Write/Exec/Sensitive → Deny
       STEP 4  rules.check (deny-first)
       STEP 5  cache.is_approved → Allow
       STEP 6  denial_tracker.should_fallback → RequireApproval
       STEP 5.5 needs_handler && !has_real_handler → RequireApproval
       STEP 6  mode dispatch → handler / classifier / Allow / Deny
       STEP 7-8 post-process + audit
    ↓ (only when mode needs confirmation) check_with_handler → HumanLoopProvider
       ↳ HitlDispatcher (REPL/TUI) / TauriHumanLoopHandler (GUI) / ChannelHumanLoopProvider

DIRECT-USER PATH (permission_mode does NOT apply)
  GUI:  #[tauri::command] → input validation only
        create_terminal      terminal.rs:278   no gate (comment explicit)
        write_terminal       terminal.rs:300   per-session consent + 64 KiB cap + audit
        connect_mcp_server   mcp.rs:211        validate_ipc_mcp_stdio / validate_ipc_mcp_url
        disconnect_mcp_server mcp.rs:261       no gate
        toggle_mcp_server    mcp.rs:293        no gate
        update_mcp_config    mcp.rs:477        serde validation + bg reconnect
        native_read_file     ipc.rs:28         validate_ipc_path + 5 MiB cap
        native_write_file    ipc.rs:47         validate_ipc_path + 10 MiB cap + atomic rename
        native_open_path     ipc.rs:119        validate_ipc_path + arg-injection guard
        browser_navigate/... browser.rs:24+    browser_runtime.execute_main(.., None)
        set_permissions_mode panels.rs:39      writes mode, propagates to agents (not a gate)
  TUI:  !<shell>             events.rs:1664    sh -lc <command> directly
        $EDITOR              events.rs:1723    user's $EDITOR directly
        /mcp load            events.rs:3425    agent.load_mcp_from_file directly
        /permission          events.rs:3576    sets mode (not a gate)
```

The two paths are disjoint. No direct-user command calls
`execute_tool_with_policy`. No agent tool call reaches the Tauri IPC
input validators. The `permission_mode` is read by exactly two IPC
commands — `get_permissions_mode` and `set_permissions_mode` in
`panels.rs` — which manage the mode and propagate it to the primary and
pool agents; they do not gate any direct-user action on the current mode
value.

### 2. The permission-mode matrix (V02)

Framework `PermissionMode` dispatch (verified at
`service.rs:167-185, 603-648`), with the user-facing aliases mapped at
`react/mod.rs:2048-2065`:

| `ToolPermission` | `Default` | `AcceptEdits` (auto-edit) | `StrictConfirm` (strict) | `BypassPermissions` (full-auto) | `Plan` | `DontAsk` |
|---|---|---|---|---|---|---|
| Read | Allow | Allow | Allow | Allow¹ | Allow | rule-only² |
| Write | **handler** | Allow | **handler** | Allow¹ | **Deny** | rule-only² |
| Network | **handler** | **handler** | **handler** | Allow¹ | Allow | rule-only² |
| Execute | **handler** | **handler** | **handler** | Allow¹ | **denied** | rule-only² |
| Sensitive | **handler** | **handler** | **handler** | Allow¹ | **denied** | rule-only² |

¹ `BypassPermissions` Allow happens at STEP 1, **after** STEP 0
(`ProtectedPathChecker`). Protected paths — `.git`, `.ssh`, `.env`,
`.aws/credentials`, `.gnupg`, private keys (`.pem`/`.key`/`.p12`),
`.docker/config.json`, `.pgpass`, `.my.cnf`, shell rc files
(`protected.rs:24-59`) — are denied even in full-auto. Activation is
loud: `tracing::warn!` at `react/mod.rs:2069-2073`.

² `DontAsk` silently rejects any tool without an explicit allow-rule
match (no user prompt). Only `Allow` rules that match pass; the cache
and handler are bypassed.

**`Default` and `StrictConfirm` have identical confirmation surfaces**
(`default_confirmation_required` and `strict_confirmation_required`
both return true for Write/Execute/Network/Sensitive — `service.rs:167-185`).
The two user-facing labels are behaviourally the same today; the
distinction is reserved for future divergence. (Note: A-HITL-01 V04
described StrictConfirm as "slightly broader ask set" — the code shows
they are identical; that characterisation is a doc drift in the
dependency, not a load-bearing claim here.)

What requires approval in the default mode: any agent tool declaring
`Write`, `Execute`, `Network`, or `Sensitive`. What is auto-allowed in
default: `Read`. What is auto-allowed in full-auto: everything except
protected paths. **No direct-user command in any mode requires
approval** — the modes do not consult the direct-user path.

### 3. Local data-loss / secret protections (V03)

Provisions that AGENTS.md explicitly endorses (data-loss / framework-
bug / local-universal secret safety), all verified live:

| Protection | Path | Mechanism | AGENTS.md category |
|---|---|---|---|
| Protected paths override Bypass | agent (STEP 0) | `protected.rs:514-528` denies `.git/.ssh/.env/.aws/...` before BypassPermissions | (1) data loss + (3) secret safety |
| Secret redaction in tool output | agent | `spawn_task.rs:171,175`, `execution.rs:229-230`, `snapshot.rs:885-889` redact via `security::redact_secrets` (19 patterns) | (3) "不把密钥打进日志" |
| Secret redaction in trace | agent | `trace/mod.rs:434` redacts every trace string | (3) |
| MCP config secret redaction | direct-user (GUI panel) | `mcp.rs:388 redact_mcp_config_secrets` strips `env`/`headers`/url-credentials before returning to frontend | (3) |
| IPC path secret-denylist | direct-user (file picker) | `path_validator.rs:18-60, 107-112` denies `~/.ssh`, `~/.aws`, `.config/gh`, `.docker`, `.gnupg`, `.kube`, `.netrc`, `.npmrc`, `.pypirc`, history/cookies | (3) |
| Atomic file write | direct-user (file picker) | `ipc.rs:64-72` temp + rename, cleanup on failure | (1) prevent torn file on crash |
| Payload size caps | direct-user | `native_read_file` 5 MiB, `native_write_file` 10 MiB, `write_terminal` 64 KiB | (2) framework-bug containment |
| Per-session terminal write consent | direct-user (terminal) | `terminal.rs:316-321` requires `confirm_terminal_consent` before programmatic writes; audit log at `:344-355` | (1) prevent stray write driving user's shell |
| Bypass-disabled admin switch | agent | `service.rs:532-540` denies BypassPermissions when `bypass_disabled = true` | (2) framework safety |
| DenialTracker fallback | agent | `service.rs:576-585` escalates to RequireApproval after repeated denials | (2) framework safety |

All ten protections are appropriate under AGENTS.md's three valid
categories. None of them is a web-service-style gate (no XSS sanitiser,
no CSRF token, no multi-tenant isolation). The secret protections in
particular satisfy AGENTS.md's "本地也成立的通用安全 (如不把密钥打进日志)"
clause literally.

### 4. Over-gating inventory (V04)

Repository-wide search for the historical over-gating patterns and the
excluded threat-model narrative:

| Pattern | Sites | Classification |
|---|---|---|
| `require_full_auto` / `require_not_strict` / `IpcAuth` / `IpcPermission` | `tauri/error.rs:17-70` only; zero callers anywhere | dead code (A-HITL-01-P2-02). The literal permission gate is gone. |
| `permission_mode` read as a gate on a direct-user action | zero. The only readers are `panels.rs:34 (get)`, `panels.rs:59 (set)`, both mode-management. | clean separation. |
| `execute_tool_with_policy` called from the app layer | zero. Only `phases/tools.rs:154, 321` and `pipeline.rs:1609` (test). | clean separation. |
| `ALLOWED_MCP_STDIO_BASES` executable allowlist | `mcp.rs:117-160`; blocks any binary not in `[npx,node,uvx,uv,python,python3,pipx,docker,java]` | **over-gating** (A-INT-01-P1-01). Makes user MCP binaries unreachable via GUI. |
| `validate_ipc_mcp_url` private-range / loopback reject | `mcp.rs:169-208`; rejects `localhost`, `127.0.0.1`, `::1`, `169.254.*`, `10.*`, `192.168.*`, `172.16-31.*` | **over-gating** (A-INT-01-P1-01). Makes locally-served MCP servers unreachable via GUI. |
| `XSS` / `SSRF` threat-model narrative in comments | `terminal.rs:48,312`, `path_validator.rs:9`, `mcp.rs:112,165,203,385` | **regressed in documentation** (X-AUT-01-P2-01). The excluded online-service threat model is re-invoked as the justification for gates and guards across the Tauri IPC layer. |

The literal `require_full_auto` gate is gone (matching the AGENTS.md
historical lesson). But the **XSS/SSRF threat-model narrative** that
AGENTS.md explicitly excludes has been re-introduced as the stated
justification for several gates and guards in the same files the
lesson was applied to. This is the cross-cutting pattern this task
surfaces.

## Findings

### X-AUT-01-P2-01: The web-service XSS/SSRF threat model excluded by AGENTS.md is re-invoked as the justification for gates and guards across the Tauri IPC layer

- Priority: P2
- Confidence: high
- Layer: application (documentation / threat-model consistency)
- Evidence:
  - AGENTS.md "产品定位与安全边界" states: "不要套用线上 Web 服务的
    威胁模型:诸如'防 XSS→RCE''防 SSRF 内网穿透'...这类线上服务的
    安全闸,**默认不适用于 EKO**." and "默认不加权限门控;要加必须在
    注释里写明'本地场景下为何仍需要'."
  - `echo-agent-cli/src/tauri/terminal.rs:46-49` — `PtySession.consented`
    doc-comment: "...interactive-shell injection channel reachable from
    any page JS; we require the user to explicitly confirm a session
    before any programmatic write is accepted, so a **background XSS
    can't silently drive a shell** the user opened."
  - `echo-agent-cli/src/tauri/terminal.rs:310-315` — `write_terminal`
    comment: "...reachable from any page JS; without an explicit confirm
    step, a **background XSS could silently drive the shell**."
  - `echo-agent-cli/src/tauri/path_validator.rs:7-9` — module doc:
    "The secret-denylist logic ... stays here as a thin wrapper layer,
    since it is specific to the IPC threat model (**XSS exfiltrating
    credentials via `native_read_file`**)."
  - `echo-agent-cli/src/tauri/commands/mcp.rs:110-119` —
    `validate_ipc_mcp_stdio` doc: "The frontend must not be able to
    spawn an arbitrary process (**any XSS would then be a one-hop RCE**)."
  - `echo-agent-cli/src/tauri/commands/mcp.rs:162-168` —
    `validate_ipc_mcp_url` doc: "...the **SSRF pivot where a compromised
    page forces the app to issue authenticated POSTs to internal
    services**."
  - `echo-agent-cli/src/tauri/commands/mcp.rs:203-204` — rejection
    reason string: "...private/loopback address; refused to prevent
    **SSRF**."
  - `echo-agent-cli/src/tauri/commands/mcp.rs:383-385` —
    `get_mcp_config` redaction comment: "...returning them verbatim
    means **any page (or XSS)** can read the configured secrets."
  - `echo-agent-cli/src/tauri/error.rs:1-10` — the (dead) `IpcAuth`
    module doc claims the gates exist for "commands that spawn processes,
    write files outside the workspace, or execute arbitrary code" under
    the same XSS-RCE framing (A-HITL-01-P2-02).
- Reachability: every reader of these comments — a reviewer, a security
  auditor, a future maintainer, or an agent doing a security pass — is
  told the Tauri IPC layer defends against an online multi-user threat
  model. That framing is the one AGENTS.md has codified as inapplicable
  to EKO, and its presence makes the next change in this area likely to
  preserve or extend the over-gating rather than align with the local
  model.
- Expected invariant: AGENTS.md "何时该加防护 ... 默认不加权限门控;
  要加必须在注释里写明'本地场景下为何仍需要'." Any gate or guard in
  the IPC layer should be justified by one of the three local-valid
  categories (data-loss / framework-bug / local-universal secret safety),
  not by the excluded online threat model.
- Observed behavior: the gates and guards split into two groups:
  (a) **Legitimate protections mis-justified** — `write_terminal`
  per-session consent (legitimate as "prevent stray/accidental IPC write
  driving the user's shell"; AGENTS.md category 1/2), `path_validator.rs`
  secret-denylist (legitimate as "don't pull credentials into agent
  context"; AGENTS.md category 3), `redact_mcp_config_secrets`
  (legitimate category 3). The gates are defensible; the comments
  invoke the wrong model.
  (b) **Actual over-gating inherited from A-INT-01-P1-01** —
  `ALLOWED_MCP_STDIO_BASES` executable allowlist and
  `validate_ipc_mcp_url` private-range/loopback rejection. These make
  user-configured MCP servers unreachable via the GUI panel; the on-disk
  config path accepts the same content. Both the gate and the comment
  are wrong under AGENTS.md.
- Impact: two-layer. (1) The over-gating (group b) is a P1 user-
  capability regression tracked under A-INT-01-P1-01. (2) The narrative
  drift (groups a + b) is a P2 documentation/threat-model consistency
  defect that will keep producing over-gated gates if not corrected,
  because every future contributor reading these comments will reason
  from the wrong model. The cost is not just reading time — it is the
  next over-gated gate being added "to match the existing XSS defense".
- Root cause: the historical "batch of security commits" that added
  `require_full_auto` were partially reverted (the permission_mode gate
  was removed, and the literal `IpcAuth` was orphaned), but the
  threat-model narrative in the comments and the parallel input-
  validation gates (executable allowlist, private-range block) were
  added or retained in the same files without being re-evaluated
  against the local-assistant positioning that AGENTS.md later codified.
  The AGENTS.md rule ("write why it's needed locally") was not applied
  to these comments.
- Direction: a single threat-model-narrowing pass across the Tauri IPC
  layer, coordinated with A-INT-01-P1-01 (which fixes the actual over-
  gating) and A-HITL-01-P2-02 (which deletes the dead `IpcAuth` and its
  misleading doc). For each comment:
  - `terminal.rs:46-49, 310-315` — rewrite to: "Per-session consent
    prevents a stray IPC call (frontend bug, accidental frontend logic)
    from driving the user's interactive shell without an explicit user
    action. The user clicks to enable writes for this terminal session."
    Drop the XSS framing.
  - `path_validator.rs:7-9` — rewrite to: "The secret-denylist prevents
    `native_read_file` from slurping credential files (`~/.ssh`,
    `~/.aws`, ...) into the agent context or trace, where they could be
    logged or sent to the model. Local-universal secret safety
    (AGENTS.md)." Drop the XSS framing.
  - `mcp.rs:110-119, 162-168, 203-204, 383-385` — handled by
    A-INT-01-P1-01's fix direction (drop the executable allowlist and
    the private-range block; keep only typo / shell-injection /
    https-for-non-localhost guards). Rewrite the comments to cite
    "lightweight misconfiguration guard" (AGENTS.md "明显错误输入的
    轻量校验") instead of XSS/SSRF.
  - `error.rs:1-10` — handled by A-HITL-01-P2-02 (delete the module
    and rewrite the doc to describe the actual input-validation policy).
  The legitimate protections (consent, secret-denylist, config
  redaction) stay; only the justifying narrative changes.
- Regression validation: documentation-only change; the underlying
  gates are addressed by A-INT-01-P1-01 / A-HITL-01-P2-02. After the
  pass, `grep -rn "XSS\|SSRF" echo-agent-cli/src/tauri --include="*.rs"`
  returns zero hits (or only hits that explicitly say "not applicable
  to EKO's local model").
- Validation reports: [V04-01](../validations/X-AUT-01/V04-01.md),
  [V03-01](../validations/X-AUT-01/V03-01.md).

### X-AUT-01-P3-01: `path_validator.rs` secret-denylist doc-comment invokes the excluded XSS threat model; the gate itself is an appropriate local secret-protection

- Priority: P3
- Confidence: high
- Layer: application (documentation)
- Evidence:
  - `echo-agent-cli/src/tauri/path_validator.rs:7-9` module doc-comment:
    "The secret-denylist logic (`.ssh`, `.aws`, cookie/history files) —
    which `PathValidator` does not provide — stays here as a thin
    wrapper layer, since it is specific to the IPC threat model (**XSS
    exfiltrating credentials via `native_read_file`**)."
  - The denylist itself (`path_validator.rs:18-28`):
    `.ssh`, `.aws`, `.config/gh`, `.docker`, `.gnupg`, `.kube`, `.netrc`,
    `.npmrc`, `.pypirc`, plus filenames containing `history`/`cookies`
    (`:31`). Applied at `:107-112` inside `validate_ipc_path`, which is
    the gate for `native_read_file` (`ipc.rs:29`), `native_write_file`
    (`ipc.rs:58`), and `native_open_path` (`ipc.rs:128`).
- Reachability: every file-picker IPC read/write/open. The denylist
  blocks any attempt to read `~/.ssh/id_rsa`, `~/.aws/credentials`,
  `~/.zsh_history`, etc. through the GUI file picker.
- Expected invariant: AGENTS.md "本地也成立的通用安全 (如不把密钥打进
  日志)" — secret protection IS one of the three valid categories. The
  denylist correctly prevents the agent (or a user via the file picker)
  from pulling credentials into the agent context where they could be
  logged or sent to the model. The defect is the doc-comment's
  justification: it cites "XSS exfiltrating credentials", which is the
  excluded online threat model.
- Observed behavior: the denylist denies the right paths for the right
  practical reason, but tells the reader the wrong reason. A reviewer
  reading the comment cannot infer the local-valid justification; they
  will either accept the XSS framing (propagating X-AUT-01-P2-01) or
  propose deleting the denylist as "online-service overreach" (which
  would be wrong — the denylist is a legitimate local secret-protection).
- Impact: low (documentation only). The gate is correct; the comment
  is misleading. The cost is the same narrative-drift cost as X-AUT-01-
  P2-01, localised to one file.
- Root cause: the denylist was added under the same online-threat-model
  framing as the rest of the Tauri IPC layer (X-AUT-01-P2-01), and its
  comment was never re-anchored to the local secret-safety rationale.
- Direction: rewrite `path_validator.rs:7-9` to: "The secret-denylist
  prevents `native_read_file` / `native_write_file` / `native_open_path`
  from pulling credential files (`~/.ssh`, `~/.aws`, `.kube`, browser
  cookies/history, etc.) into the agent context, where they could be
  logged or sent to the model. This is local-universal secret safety
  (AGENTS.md '不把密钥打进日志'); the underlying `PathValidator`
  handles only home-confinement and traversal, so the denylist is
  layered here." Drop the "XSS exfiltrating credentials" framing.
- Regression validation: documentation-only; re-read the file after
  the rewrite and confirm the denylist behaviour is unchanged
  (`test_denied_secret_paths_match` at `:152-160` stays green).
- Validation reports: [V03-01](../validations/X-AUT-01/V03-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Call-path classification: agent automation vs direct-user; correctly separated | yes | passed | [V01-01](../validations/X-AUT-01/V01-01.md) |
| V02 | Default / full-auto mode matrix: what is allowed, what requires approval | yes | passed | [V02-01](../validations/X-AUT-01/V02-01.md) |
| V03 | Local data-loss / secret protections: present and appropriate per AGENTS.md | yes | passed (with one comment finding) | [V03-01](../validations/X-AUT-01/V03-01.md) |
| V04 | Over-gating search: `require_full_auto`, permission gates on interactive tools, XSS/SSRF narrative | yes | passed (with findings) | [V04-01](../validations/X-AUT-01/V04-01.md) |
| V05 | Historical-document drift | conditional | n/a | No prior X-AUT-01 report exists in this reviewer directory. The historical claims audited come from AGENTS.md itself, classified under "Historical Claim Status". |

No cargo command was executed for this task: it is a static cross-
cutting synthesis that consumes the executable evidence of its four
dependencies (F-HITL-01 V01–V04, F-SEC-01 V01–V04, A-HITL-01 V01–V04,
A-INT-01 V01–V04) and adds only static reachability grep + doc-comment
inspection at the pinned commits.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| AGENTS.md: "permission_mode controls agent automation only, not user-interactive tools (terminal, file picker)" | current (supported) | V01 confirms `execute_tool_with_policy` and `PermissionStage` are reached only from the ReAct tool loop (`phases/tools.rs:154, 321`); every direct-user IPC command and TUI direct action bypasses them. `permission_mode` is read by `panels.rs:34, 59` (mode management) only. |
| AGENTS.md historical lesson: "`require_full_auto` gates on `create_terminal`/`connect_mcp_server` removed" | current (literal); parallel narrative persists | V04 confirms the literal `require_full_auto` / `IpcAuth` has zero callers (`error.rs` only). But the XSS/SSRF threat-model narrative that motivated those gates persists in 5+ comments across the same files — finding X-AUT-01-P2-01. |
| AGENTS.md: "不要套用线上 Web 服务的威胁模型 (XSS/SSRF)" | regressed (in comments) | V04 grep finds `XSS`/`SSRF` invocations at `terminal.rs:48,312`, `path_validator.rs:9`, `mcp.rs:112,165,203,385`. The excluded threat model is the stated justification for several gates/guards. X-AUT-01-P2-01. |
| AGENTS.md: "何时该加防护 ... 写明'本地场景下为何仍需要'" | partially followed | The ten legitimate protections (V03) are appropriate, but the comments justifying them cite the excluded model rather than the local-valid category. X-AUT-01-P2-01 / X-AUT-01-P3-01. |
| AGENTS.md: "保留对明显错误输入的轻量校验即可,不要做权限级拦截" | regressed (in MCP IPC) | On-disk `validate_stdio_command` is correctly lightweight; IPC `validate_ipc_mcp_stdio` (allowlist) + `validate_ipc_mcp_url` (private-range block) go beyond lightweight typo-catching. A-INT-01-P1-01 (inherited). |
| F-HITL-01: "Protected paths override BypassPermissions" | current (supported) | V02 confirms STEP 0 (`service.rs:514-528`) runs before STEP 1 (Bypass short-circuit); `.git/.ssh/.env/.aws/...` denied even in full-auto. |
| F-HITL-01: "one canonical permission entry" | current (supported) | V01 confirms `execute_tool_with_policy` (`snapshot.rs:1189`) is the single agent-tool entry; no app-layer caller. |
| F-SEC-01: "secret redaction wired into all four main runtime surfaces" | current (supported) | V03 confirms `spawn_task.rs:171,175`, `trace/mod.rs:434`, `snapshot.rs:885-889`, `execution.rs:229-230` all redact via `security::redact_secrets`. |
| A-HITL-01: "direct-user actions correctly NOT gated by permission_mode" | current (supported) | V01 confirms the two-path separation; V04 confirms zero `permission_mode`-read-as-gate sites. |
| A-HITL-01: "IpcAuth is dead" | current (supported) | V04 confirms `IpcAuth` / `IpcPermission` / `require_full_auto` / `require_not_strict` confined to `error.rs:17-70`; zero callers. |
| A-INT-01: "IPC MCP validators over-gate (reject localhost)" | current (supported) | V04 confirms `validate_ipc_mcp_url` rejects `localhost`/`127.0.0.1`/`::1`/private ranges; `validate_ipc_mcp_stdio` restricts to a 9-binary allowlist. Inherited as the canonical over-gating instance. |
| A-INT-01: "on-disk path accepts what IPC path rejects" | current (supported) | The asymmetry is the evidence that the IPC gates exceed the local-valid lightweight-check bar. |

## Coverage And Uncertainty

Inspected in full (directly, not via dependencies):

- All `#[tauri::command]` sites in `terminal.rs`, `ipc.rs`,
  `commands/mcp.rs`, `commands/browser.rs`, `commands/panels.rs` (the
  permission-relevant command surface).
- The TUI direct actions `!<shell>` (`events.rs:1664-1696`) and
  `/permission` (`events.rs:3576-3605`).
- The framework permission pipeline `service.rs:484-681`, the per-mode
  helpers `:167-185`, the protected-paths catalog `protected.rs:24-59`,
  the `PermissionMode` / `ToolPermission` model
  `echo-core/src/tools/permission.rs:15-130`, and the runtime mode
  mapping `react/mod.rs:2040-2079`.
- The `path_validator.rs` (full, 200 lines) and the four live secret-
  redaction call sites.
- The `execute_tool_with_policy` / `PermissionStage` reachability via
  grep across both repos.

Not inspected (out of scope or deferred):

- **The ~200 remaining Tauri commands** outside the permission-relevant
  surface (analysis, research, tasks, session, config, files beyond the
  native_* trio, panels beyond mode management). They were counted
  (219 total) and sampled for `permission_mode` reads; none gates on
  permission_mode. A full audit of every command's input validation is
  out of scope for the boundary question (which is about agent-vs-user
  separation, not per-command validation quality).
- **The `path_validator` symlink-escape and canonicalisation internals.**
  `PathValidator::validate_within_base` (framework, `echo_tools::security`)
  is consumed as a black box. Its correctness belongs to a framework
  path-security task.
- **Frontend enforcement of the consent / mode flows.** Whether the
  React layer always calls `confirm_terminal_consent` before
  `write_terminal`, or how the permission-mode selector renders, is
  A-FE-01.
- **Subagent approval routing.** Whether a pooled subagent inherits the
  per-turn GUI handler or auto-rejects against the empty dispatcher is
  F-SUB-01 / F-SUB-02.
- **No executable cargo run.** This is a static synthesis task; the
  executable evidence is inherited from the four dependency tasks.

Environmental constraints:

- Both repos verified at the pinned commits (`echo-agent` 9b0e0fa,
  `echo-agent-cli` b3b2e81), both clean.
- No `cargo clean` needed (disk pressure well below threshold; no build
  performed).

Uncertain claims:

- Whether the `Default` vs `StrictConfirm` identity (identical
  confirmation surfaces at `service.rs:167-185`) is intentional future-
  proofing or a drift. The code comment at `permission.rs:79` says
  StrictConfirm "reads are allowed, mutating or external operations ask"
  — which is exactly what Default does too. Not load-bearing for the
  boundary question; flagged here for a future mode-semantics task.
- Whether any external `echo-agent-cli` consumer calls the dead
  `IpcAuth` (it is `pub`). The in-repo grep is clean; the misleading
  doc-comment is the stronger reason for A-HITL-01-P2-02.

## Handoff

Conclusions downstream tasks may rely on:

1. **The agent-vs-user boundary is correct.** Agent automation goes
   through `execute_tool_with_policy` → `PermissionStage` →
   `PermissionService`; direct-user terminal / file picker / MCP /
   browser / TUI-shell actions bypass that path entirely and use input
   validation + per-session consent. `permission_mode` is read by mode-
   management IPC only, never as a gate on a direct-user action. This
   matches AGENTS.md's positioning and is confirmed across four
   dependency tasks plus this task's V01/V04.
2. **The mode matrix is as documented.** Default/auto-edit/strict/full-
   auto map to framework Default/AcceptEdits/StrictConfirm/BypassPermissions
   with the exact confirmation surfaces at `service.rs:167-185`. Full-auto
   (BypassPermissions) auto-allows everything **except protected paths**
   (`.git/.ssh/.env/.aws/...`), which override Bypass at STEP 0.
   `Default` and `StrictConfirm` are behaviourally identical today.
3. **The ten local data-loss / secret protections are appropriate per
   AGENTS.md** (V03). All map to one of the three valid categories
   (data-loss / framework-bug / local-universal secret safety). None is
   a web-service-style gate.
4. **The one genuine over-gating defect is A-INT-01-P1-01** (MCP IPC
   executable allowlist + private-range/loopback URL block). It is the
   only place where a direct-user capability (configuring a local or
   non-allowlisted MCP server) is unreachable via the GUI panel while
   being accepted by the on-disk config path. Downstream tasks citing
   "MCP is user-configured and reachable" must qualify that the GUI
   panel over-gates until A-INT-01-P1-01 lands.
5. **The XSS/SSRF narrative drift is systematic (X-AUT-01-P2-01).** The
   excluded online threat model is re-invoked as the justification for
   gates/guards at 5+ Tauri IPC sites. Downstream security-review tasks
   must not cite these comments as evidence that the codebase defends
   against XSS/SSRF — the comments reflect a threat model AGENTS.md has
   rejected; the underlying protections are local-valid (or, for the MCP
   pair, over-gated and pending fix).

Reports they must read:

- This report (X-AUT-01) for the boundary synthesis and the two
  cross-cutting findings.
- `tasks/F-HITL-01.md` for the framework permission composition and the
  protected-paths-override-Bypass invariant.
- `tasks/F-SEC-01.md` for the four secret-redaction surfaces and the
  sandbox fail-safe fallback.
- `tasks/A-HITL-01.md` for the surface → provider wiring map, the dead
  `IpcAuth` (P2-02), and the direct-user-vs-agent-action classification.
- `tasks/A-INT-01.md` for the MCP IPC over-gating (P1-01) — the
  canonical over-gating instance that X-AUT-01 generalises.

Conditions that make this report stale:

- Any change that routes a direct-user IPC command through
  `execute_tool_with_policy` (or that makes a direct-user command read
  `permission_mode` as a gate) invalidates V01 / V04.
- Any change to the per-mode `*_confirmation_required` helpers
  (`service.rs:167-185`) invalidates V02.
- Any change to the protected-paths catalog or its ordering relative to
  the Bypass short-circuit invalidates V02/V03.
- Resolution of A-INT-01-P1-01 (dropping the executable allowlist and
  private-range block) and A-HITL-01-P2-02 (deleting `IpcAuth`) will
  resolve the over-gating half of X-AUT-01-P2-01; the comment-narrowing
  pass should follow in the same patch.
- Any new gate added to a direct-user IPC command should be checked
  against AGENTS.md's three local-valid categories and documented with
  the local rationale, not the XSS/SSRF framing.

Follow-up task IDs (no fixes implemented in this review):

- A **Tauri threat-model-narrowing task** should action X-AUT-01-P2-01
  (rewrite the comments across `terminal.rs`, `path_validator.rs`,
  `mcp.rs`, `error.rs` to cite local-valid categories instead of
  XSS/SSRF). This bundles cleanly with A-INT-01-P1-01 (which fixes the
  actual MCP over-gating) and A-HITL-01-P2-02 (which deletes `IpcAuth`
  and its misleading doc). The three should land together so the
  narrative and the gates are consistent in one pass.
- X-AUT-01-P3-01 (`path_validator.rs` comment) can be folded into the
  same pass or done as a standalone doc-only change.
