# X-AUT-01: Permission and local security boundary

> Status: complete
> Reviewer: Codex review subagent
> Executor: Codex review subagent
> Accepted by: Codex primary reviewer
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: framework had extensive external source changes, so every adopted framework anchor came from committed `HEAD` blobs. CLI `Cargo.lock` was externally modified and excluded. No source, index, README or shared task catalog was changed; only Codex X-AUT reports were added.

## Question

Are automated Agent actions controlled while direct user terminal, file picker,
MCP configuration and Browser interactions remain usable under EKO's local
personal-assistant threat model?

## Scope

- Framework PermissionService and ReAct Tool-policy boundary at committed HEAD.
- EKO GUI/TUI/CLI/channel permission-mode propagation and direct interaction
  separation.
- Direct workspace selection, file IPC, MCP configuration and Browser command
  registration/reachability.
- Local accidental data-loss and secret-log protections.
- Existing static test coverage and dependency finding ownership.

## Out Of Scope

- Approval argument/cache/provider semantics owned by `F-HITL-01`.
- Generic guards, sandbox isolation and framework audit/trace secret behavior
  owned by `F-SEC-01`.
- Multi-surface provider routing, mode parity and GUI MCP over-gating owned by
  `A-HITL-01`.
- MCP configuration lifecycle/secret round-trip and Browser/LSP lifecycle owned
  by `A-INT-01`.
- Online/multi-user/XSS/SSRF policy, except to identify its inappropriate use
  on a direct local-user path.
- Cargo, rustc, tests, builds, dynamic fixtures, application launch and network.

## Inputs

- Root `AGENTS.md`; shared `README.md`, `TASKS.md`, `REPORTING.md`; Codex
  `README.md` and report templates.
- Exact authorized Codex dependencies used: `F-HITL-01`, `F-SEC-01`,
  `A-HITL-01`, and `A-INT-01`. One accidental read of the prior X-TOL task
  report and one failed X-TOL path attempt are disclosed in V00-09/V00-10 and
  excluded in full; no X-TOL conclusion supports this report.
- Committed framework blobs and CLI source at the revisions above. No other
  reviewer directory was read.
- Six unsuccessful/incomplete commands are preserved as immutable inconclusive
  V00 attempts. No partial output from those commands supports a conclusion.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Tool permissions, per-call policy decisions, protected-path primitives, sandbox contracts, typed approval results and secret-safe observability belong to `echo-agent` and must remain reusable by unrelated consumers. |
| EKO product policy | Which GUI/TUI/CLI/channel selection changes automation mode, how explicit local paths/extensions are trusted, revision policy for direct file editing and how configured webhook destinations are labeled belong to EKO. |
| Adapter boundary | EKO maps a surface-selected mode into the framework service for automated Tool calls and keeps direct-user IPC separate. Adapters may validate malformed input and prevent accidental overwrite/log leakage, but must not impose cloud/Web capability gates. |
| Duplicate search | Searched both repositories by PermissionMode/PermissionService/IpcAuth, Tool approval pipeline, terminal/file/MCP/Browser commands, path validators, revision/atomic write, webhook URL/secret/logging, registrations, bridge calls and adjacent tests. One canonical workspace writer has revision protection; a second registered native writer does not. |
| Migration deletion | Retain the framework permission service and EKO surface adapters. Delete inert `IpcAuth`, the online-threat MCP/path deny policy already owned by A-HITL/X-AUT, and the unused no-revision native writer after callers use the canonical revisioned service. Do not add a second permission engine. |

## Current Path

```text
automated Agent Tool
  -> ReAct policy pipeline
  -> PermissionService::check_with_permissions_in_mode
       protected-path check
       default / auto-edit / strict / full-auto decision
  -> approval provider when required
  -> Tool execution

GUI/TUI permission setting
  -> primary Agent::set_permission_mode
  -> AgentPool::apply_permission_mode
       existing pooled Agents + remembered future-Agent mode

CLI permission setting -> primary Agent only
channel -> no equivalent permission-mode command
  (existing owner: A-HITL-01-P1-05)

direct user paths
  terminal / MCP / Browser / files -> no IpcAuth or permission_mode call
  workspace picker -> create_workspace -> validate_workspace_root
                   -> HOME confinement + broad secret-name denylist
  native_write_file -> registered Tauri command -> path/content only
                    -> atomic rename, no expected revision
  workspace write_file -> expected revision -> reject stale -> atomic rename

configured webhook
  -> WebhookEmitter shared by GUI/CLI/channel/scheduler
  -> delivery error -> warning containing full endpoint URL
```

Positive conclusions:

- Agent automation does pass the framework permission service. `full-auto`
  maps to bypass, default mode applies permission classification/approval, and
  mode changes clear approval cache.
- No live user terminal/file/MCP/Browser command calls the stale `IpcAuth` mode
  gate. Direct Browser command defaults allow all domains and malformed URLs
  are rejected structurally.
- GUI/TUI update primary plus current/future pooled Agents. The remaining
  CLI/channel mismatch is already `A-HITL-01-P1-05`.
- Workspace editor writes use containment, expected revision, atomic rename and
  cleanup. That is the application authority to reuse.
- Existing framework protected-path, WebSocket token, audit persistence,
  sandbox and HITL defects retain their dependency owners; this report does not
  duplicate them.

## Findings

### X-AUT-01-P0-01: Webhook delivery failures write credential-bearing endpoint URLs to local logs

- Priority: P0
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/webhook/emitter.rs:20`, `:67`, `:73`, `:128`, `:159`, `:164`, `:166`, `:170`; `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:135`, `:148`, `:154`, `:168`, `:446`, `:461`; `echo-agent-cli/src/tauri/desktop.rs:135`, `:169`, `:196`
- Reachability: canonical app config -> shared live WebhookEmitter -> every subscribed chat/scheduler delivery -> initial or retry request failure -> warning log interpolates the complete configured URL.
- Expected invariant: observability may identify an endpoint but never persist URL userinfo, signed query parameters or webhook access tokens.
- Observed behavior: endpoint URLs are cloned verbatim from config and both failure warnings print `{url}` without redaction. The distinct optional HMAC secret is not logged, but token-bearing webhook URLs are supported ordinary inputs.
- Impact: a transient delivery error copies durable access credentials into local logs, where support bundles, backups or later Tool reads can expose them.
- Root cause: the URL is used simultaneously as a request destination and a human-readable endpoint identity.
- Direction: assign a safe endpoint ID/label and log only that plus a redacted origin/path. Centralize URL redaction and delete full-URL interpolation from both retry warnings.
- Regression validation: configure URL userinfo plus Unicode path and signed/token query, force first and retry failures, and assert captured logs contain neither credentials nor raw query values while retaining a useful endpoint label.
- Validation reports: [V03](../validations/X-AUT-01/V03-01.md), [V04](../validations/X-AUT-01/V04-01.md)

### X-AUT-01-P1-02: Explicit workspace selection is rejected by HOME and secret-name rules derived from an XSS threat model

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/web-frontend/src/lib/tauri-bridge.ts:89`; `echo-agent-cli/web-frontend/src/components/workspace/NewTaskDialog.tsx:75`, `:78`, `:109`; `echo-agent-cli/web-frontend/src/api/endpoints.ts:1404`; `echo-agent-cli/src/tauri/commands/workspace.rs:26`, `:37`; `echo-agent-cli/src/tauri/path_validator.rs:6`, `:9`, `:18`, `:30`, `:47`, `:72`, `:96`, `:99`, `:107`, `:130`, `:139`
- Reachability: the GUI directory picker returns the user's explicit selection; create-and-switch sends it as `root`; every custom root passes `validate_workspace_root` before registry creation.
- Expected invariant: an explicitly selected readable local directory is usable as a workspace, including external/mounted paths. Validation should reject empty/malformed/traversing paths and obvious mistakes, not infer that the trusted user is an XSS attacker.
- Observed behavior: the validator requires every root to resolve inside HOME and rejects any relative path containing `history`, `cookie` or `cookies`, plus a fixed credential-directory list. Its module rationale explicitly protects against XSS exfiltration. Thus `/Volumes/work`, another local home, and benign roots such as `~/projects/browser-history-analyzer` are refused after selection.
- Impact: a core local-assistant workflow cannot open common project locations or benignly named directories, despite the user directly selecting and trusting them.
- Root cause: one IPC validator conflates lexical containment/error validation with a cloud/Web capability policy, then applies it to user intent after the native picker.
- Direction: separate direct-user selected-path validation from automated Agent file authority. For workspace selection, resolve/canonicalize, verify accessibility/type and preserve the explicit root; remove HOME and substring deny policy plus its XSS rationale/tests. Keep workspace-relative containment for later editor operations.
- Regression validation: create/switch workspaces on external volume, sibling home/mount, `.ssh` only when explicitly selected, and benign Unicode names containing `history`/`cookie`; also retain empty, nonexistent, file-not-directory, traversal/symlink and permission-error cases.
- Validation reports: [V01](../validations/X-AUT-01/V01-01.md), [V04](../validations/X-AUT-01/V04-01.md)

### X-AUT-01-P2-03: A registered native file writer bypasses the canonical revision contract

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/src/tauri/ipc.rs:22`, `:47`, `:58`, `:60`, `:64`, `:66`, `:67`, `:68`; `echo-agent-cli/src/tauri/mod.rs:69`, `:72`; `echo-agent-cli/web-frontend/src/lib/tauri-bridge.ts:62`, `:73`, `:75`; `echo-agent-cli/src/tauri/commands/files.rs:153`, `:164`, `:171`, `:182`, `:183`, `:196`, `:202`
- Reachability: `native_write_file` is registered in the Tauri invoke handler and exposed by the frontend `fileSystem.writeFile` bridge. No current production component calls that bridge method, while the active workspace editor uses the separate revisioned command.
- Expected invariant: EKO has one direct file-write authority; overwriting an existing file requires its observed revision or another explicit replace contract so concurrent/local edits cannot be silently lost.
- Observed behavior: the native command accepts only path/content, creates missing parents and atomically renames over any target. Atomicity prevents torn bytes but not lost updates. The canonical workspace writer already hashes current content and rejects a stale `expected_revision`.
- Impact: any present/future bridge caller or direct IPC use can overwrite newer local content without warning, and maintaining two writers invites a UI to choose the weaker contract.
- Root cause: legacy low-latency IPC survived after the revisioned workspace service became the application authority.
- Direction: delete `native_write_file` and `fileSystem.writeFile` if still unused; otherwise route it through the canonical revisioned writer with explicit create/replace semantics. Do not copy the revision algorithm into a third adapter.
- Regression validation: two clients read revision A, one writes B, the second attempts C; require stale rejection and preserved B. Cover create, missing parent, symlink/root containment, Unicode path and rename failure cleanup through the one authority.
- Validation reports: [V02](../validations/X-AUT-01/V02-01.md), [V04](../validations/X-AUT-01/V04-01.md)

## Dependency Backlinks (Not Duplicated)

| Canonical finding | X-AUT observation |
|---|---|
| `F-HITL-01-P0-01`, `P1-02..09` | Approval edits/cache/provider and protected-path false positives affect automated Tools; retain framework owner. |
| `F-SEC-01-P0-01` plus its F-OPS backlink; `P1-03..08` | Framework bearer/audit secrets, sandbox and guard behavior remain canonical there. X-AUT-P0-01 is a separate EKO outgoing-webhook log site. |
| `A-HITL-01-P1-01..06`, `P2-07` | Surface providers/mode parity and GUI MCP online-threat blocking remain application HITL owners. |
| `A-INT-01-P1-01..04`, `P2-05` | MCP config/secret round-trip and Browser/LSP lifecycle remain integration owners. |

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V00-01 | Commit and dirty-source isolation | yes | passed | [report](../validations/X-AUT-01/V00-01.md) |
| V00-02 | Broad `rg -E` attempt | disclosure | inconclusive, excluded | [report](../validations/X-AUT-01/V00-02.md) |
| V00-03 | Incorrect surface path search | disclosure | inconclusive, excluded | [report](../validations/X-AUT-01/V00-03.md) |
| V00-04 | Incorrect prior-report glob | disclosure | inconclusive, excluded | [report](../validations/X-AUT-01/V00-04.md) |
| V00-05 | Incorrect app-core config/type paths | disclosure | inconclusive, excluded | [report](../validations/X-AUT-01/V00-05.md) |
| V00-06 | Incorrect framework validator path | disclosure | inconclusive, excluded | [report](../validations/X-AUT-01/V00-06.md) |
| V00-07 | Incorrect/masked channel path search | disclosure | inconclusive, excluded | [report](../validations/X-AUT-01/V00-07.md) |
| V00-08 | Incorrect dependency report glob | disclosure | inconclusive, excluded | [report](../validations/X-AUT-01/V00-08.md) |
| V00-09 | Unauthorized prior X-TOL report read | disclosure | inconclusive, excluded | [report](../validations/X-AUT-01/V00-09.md) |
| V00-10 | Missing unauthorized X-TOL validation path | disclosure | inconclusive, excluded | [report](../validations/X-AUT-01/V00-10.md) |
| V01 | Call-path classification and over-gating | yes | failed -> finding | [report](../validations/X-AUT-01/V01-01.md) |
| V02 | Local data-loss/write-authority trace | yes | failed -> finding | [report](../validations/X-AUT-01/V02-01.md) |
| V03 | Secret-log trace | yes | failed -> finding | [report](../validations/X-AUT-01/V03-01.md) |
| V04 | Static tests and edge-case matrix | yes | failed -> coverage/finding support | [report](../validations/X-AUT-01/V04-01.md) |
| V05 | Dependency ownership/drift classification | yes | passed | [report](../validations/X-AUT-01/V05-01.md) |
| V09 | Dynamic default/full-auto/direct-user/data-loss/secret matrix | future | not_run by instruction | [report](../validations/X-AUT-01/V09-01.md) |
| V99 | Report integrity and source-write isolation | yes | passed | [report](../validations/X-AUT-01/V99-01.md) |
| V30 | Primary committed-source acceptance | yes | passed | [report](../validations/X-AUT-01/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| Root AGENTS: permission mode governs Agent automation, not direct terminal/file/MCP | current for absence of mode calls | V01 confirms no live direct `IpcAuth`; GUI/TUI automation propagation remains current. |
| Root AGENTS: local user extensions should receive only light obvious-input validation | regressed for GUI MCP, current dependency | `A-HITL-01-P1-06`; not duplicated. |
| `A-HITL-01`: no direct terminal/file/MCP command consults permission mode | current | V01. |
| `A-HITL-01`: remove inert IpcAuth and online-threat MCP allowlists | current unresolved | `IpcAuth` has no caller; MCP over-gate remains dependency-owned. |
| MASTER-PLAN: common webhook lifecycle works across GUI/TUI/CLI/channel | current for reachability, incomplete for secrecy | V03 confirms the shared emitter is live; its failure log leaks URL credentials. |

## Coverage And Uncertainty

- This is static review. No permission decision, file write, webhook failure or
  direct interaction was executed; V09 records the future runtime matrix.
- Framework current dirty bodies were excluded. A primary reviewer must sample
  committed blobs before acceptance.
- CLI `Cargo.lock` was not read, so dependency-version conclusions are absent.
- `native_write_file` has a registered IPC/bridge path but no current production
  component caller. The finding is intentionally P2; new caller registration
  would raise the risk and make this report stale.
- The webhook finding requires a configured URL containing a credential. The
  code logs every URL verbatim, but this review did not inspect user config.
- Tauri capabilities/configuration outside source registration were not
  dynamically exercised. They do not negate the registered command contract.

## Handoff

- Primary review should independently reconstruct V01 picker -> workspace
  validator, V02 command -> bridge plus canonical writer comparison, and V03
  configured emitter -> failure log before changing status to `complete`.
- Downstream synthesis may rely on three candidate findings only after primary
  acceptance; retain dependency findings under their original IDs.
- Fix order: redact webhook logs first; restore explicit workspace selection;
  then delete/converge the unused writer. Permission-provider and MCP work must
  be coordinated with the dependency owners rather than adding new policy.
- This report becomes stale when either reviewed commit changes, when workspace
  root validation is separated, when webhook logging is redacted, when
  `native_write_file` is removed/revisioned, or when a new frontend caller is
  added.
