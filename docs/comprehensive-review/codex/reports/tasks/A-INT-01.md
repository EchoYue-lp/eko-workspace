# A-INT-01: Browser, MCP, and LSP application integration

> Status: complete
> Reviewer: Codex review subagent
> Primary acceptance: Codex primary reviewer
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: CLI clean; framework concurrent dirty transitions captured by V00/V99 and excluded; only Codex A-INT-01 reports were written by this review

## Question

Are EKO's local Browser sessions and user-configured MCP/LSP capabilities
reachable, recoverable across configuration/workspace lifecycle, and free of
irrelevant automation permission gates?

## Scope

- Browser runtime construction, IPC execution, per-conversation sessions,
  metadata restore, interrupt/shutdown, and conversation deletion cleanup.
- MCP bootstrap config selection, live Agent clients, GUI list/config state,
  connect/disconnect/toggle/update/reconnect, secret round-trip, and frontend
  reachability.
- LSP discovery/config precedence, shared manager/tool registration, plugin LSP
  merge, workspace switch/exit, and current surface reachability.
- Default-permission interactive use, existing tests, and future regressions.

## Out Of Scope

- MCP/LSP wire protocol, timeout/request lifecycle, tool payload semantics,
  URI mapping, capability negotiation, and framework cleanup defects owned by
  `F-INT-01`/`F-INT-02`.
- Generic Tool registry/sandbox/large-output behavior owned by `A-TOOL-01`.
- GUI MCP launcher/public-network over-gating and stale Browser approval
  providers, already canonical `A-HITL-01-P1-06` and `A-HITL-01-P2-07`.
- General application config persistence outside MCP, frontend visual design,
  fixes, source mutation, and shared index edits.
- Cargo, rustc, tests, builds, dynamic fixtures, application launch, and network.

## Inputs

- Root `AGENTS.md`; shared `README.md`, `REPORTING.md`, `TASKS.md`; Codex README.
- Codex dependencies `F-INT-01`, `F-INT-02`, `A-TOOL-01`, and `A-HITL-01` only.
- Current CLI source at the revision above. Concurrent framework dirty paths
  were neither read nor used; no other reviewer directory was read.
- V01 attempt 01 preserves an incorrect assumed LSP path lookup as
  inconclusive; none of its partial output supports conclusions.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | MCP transports/protocol, LSP client/manager/request semantics, generic Browser Tool execution, cancellation and typed failures belong to `echo-agent`. |
| EKO product policy | Which local config file/UI edits are authoritative, workspace rebind timing, Browser conversation ownership, frontend status, and reconnect generation policy belong to `echo-agent-cli`. |
| Adapter boundary | EKO should select one durable config generation, atomically reconcile it into framework clients/managers, and project truthful status. It must not reimplement MCP/LSP protocols. |
| Duplicate search | Both repositories were searched by Browser runtime/session, MCP config/client/command, LSP manager/config/tool, workspace, connection/reload, permission, and surface caller concepts. |
| Migration deletion | Delete `PluginState::mcp_config` as an independent mirror once one durable MCP config authority feeds bootstrap, editor, reconnect and status. Replace detached reconnect tasks rather than adding a second queue. |

This is a local personal assistant. User-selected Browser/MCP/LSP capabilities do
not need a cloud/Web permission gate. The findings below concern destructive
round trips, stale workspace capability, lifecycle leakage, and state races that
remain defects in the local threat model.

## Current Path

```text
AgentRuntime::bootstrap
  -> BrowserRuntime::start -> shared Browser Tools + Tauri Browser commands
  -> load_mcp_config(selected file) -> Agent MCP clients/tools
  -> AgentHandle
  -> register_lsp_tools(working_dir snapshot)
       -> discovery + global/project .lsp.yaml -> shared LspManager -> 5 Tools
  -> PluginRuntimeService(AgentHandle, same LspManager, base config/root snapshot)

AppState::from_shared
  -> PluginState::mcp_config = default (not bootstrap config)

GUI MCP panel
  -> list: live Agent clients + PluginState mirror
  -> get config: redacted PluginState mirror
  -> update: replace mirror -> detached disconnect-all/reconnect snapshot

workspace switch/exit
  -> Agent + Pool working_dir/context/memory update
  -> no MCP config/client reconcile; no LSP root/config rebind

Browser command/Tool -> conversation-keyed BrowserSessionManager -> persisted metadata
conversation deletion -> messages + tool executions + artifacts cleanup
  -> no Browser session/provider cleanup
process shutdown -> BrowserRuntime::shutdown -> close_all sessions/clients
```

Positive conclusions:

- Browser direct actions, MCP GUI actions, and Agent LSP Tools are registered and
  reachable. MCP/LSP Tool registries are shared with pooled Agents through the
  application Agent construction path; absence of a dedicated LSP settings IPC
  is not by itself evidence that the framework capability is dead.
- Direct user interaction does not consult EKO automation `permission_mode`.
- MCP list combines configured and connected states, per-server connection
  failures do not prevent later reconnect attempts, and each attempt is bounded
  by 15 seconds.
- Browser session parsing tolerates malformed metadata by warning/skipping;
  shutdown closes all sessions/clients. LSP invalid global/project config warns
  and retains the preceding discovered configuration.
- Fixed MCP launcher/private-host restrictions remain over-gating under the
  local product model, but are not duplicated here because A-HITL-01 owns them.

## Findings

### A-INT-01-P1-01: GUI MCP configuration is an unsaved mirror disconnected from bootstrap authority

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/runtime.rs:109`,
  `echo-agent-cli/echo-agent-app-core/src/infra.rs:1069`, `:1074`, `:1090`;
  `echo-agent-cli/echo-agent-app-core/src/state.rs:355`, `:489`;
  `echo-agent-cli/src/tauri/commands/mcp.rs:8`, `:377`, `:477`, `:484`, `:489`;
  `echo-agent-cli/web-frontend/src/components/mcp/McpPanel.tsx:24`, `:36`, `:50`,
  `:142`.
- Reachability: every bootstrap selects and loads a file directly into the
  primary Agent; registered GUI MCP settings read/write the separately defaulted
  `PluginState` and the panel promises that saving immediately applies config.
- Expected invariant: the editor reads and durably updates the same MCP config
  that startup/restart consumes, or explicitly identifies a live-only overlay.
- Observed behavior: AppState never receives the loaded config. GUI initially
  returns an empty config even when live clients were loaded, and update only
  swaps memory/spawns reconnect; it never writes the selected/default config.
- Impact: users can see connected servers but an empty editor; saved changes
  disappear on restart, and a save from the empty mirror can disconnect all
  configured servers for the current session.
- Root cause: EKO created a second config authority after bootstrap without a
  load/save/revision contract.
- Direction: introduce one application-owned, atomically persisted MCP config
  service selected by the documented precedence; make bootstrap, GUI, CLI/TUI,
  status and reconnect consume its generation. Delete `PluginState::mcp_config`
  as an independent mirror.
- Regression validation: boot from default, override and environment files;
  assert editor equality, atomic save, live reconcile, restart reconstruction,
  invalid-write last-known-good behavior, and GUI/TUI/CLI parity.
- Validation reports: [V02](../validations/A-INT-01/V02-01.md).

### A-INT-01-P1-02: Redacted MCP secrets are destructively round-tripped as literal credentials

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `echo-agent-cli/src/tauri/commands/mcp.rs:377`, `:388`, `:402`,
  `:412`, `:420`, `:429`, `:477`, `:481`, `:490`;
  `echo-agent-cli/web-frontend/src/components/mcp/McpPanel.tsx:24`, `:28`, `:36`,
  `:46`, `:50`.
- Reachability: opening the registered MCP settings panel fetches the redacted
  complete document; Save sends that document back to whole-object replacement
  and reconnection.
- Expected invariant: masking credentials for display must preserve unchanged
  secret values through an unrelated edit.
- Observed behavior: env values and credential header/query values become the
  literal `<redacted>` placeholder. No opaque identity, presence-aware patch, or
  merge with the secret-bearing source restores them on update.
- Impact: editing a server name/argument can replace API keys, authorization
  headers and URL tokens, causing immediate connection failure and permanent
  credential loss once P1-01 adds real persistence.
- Root cause: one serialization shape serves incompatible read-redaction and
  write-replacement contracts.
- Direction: return an editable DTO with explicit masked/unchanged fields and
  apply a field-level patch against the authoritative config, or use opaque
  credential references. Reject a new literal placeholder where no prior secret
  exists.
- Regression validation: round-trip stdio env, bearer/unknown headers and secret
  query parameters while editing each non-secret field; verify clear/replace are
  explicit operations.
- Validation reports: [V03](../validations/A-INT-01/V03-01.md).

### A-INT-01-P1-03: Detached MCP reconciliation has no generation guard, so stale saves can win

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/src/tauri/commands/mcp.rs:477`, `:489`, `:496`,
  `:502`, `:505`, `:509`, `:519`, `:533`, `:552`, `:555`.
- Reachability: every GUI full-config save returns after spawning this task;
  settings can be saved again before prior reconciliation acquires the Agent
  write lock or finishes.
- Expected invariant: live clients converge to the latest accepted config
  generation, and callers can observe success/failure/cancellation terminally.
- Observed behavior: each task owns a cloned snapshot, disconnects all current
  clients, and reconnects its snapshot. Tasks are untracked and have no
  generation/cancellation comparison; lock scheduling can let an older snapshot
  execute last. The returned success covers only the mirror swap, not reconcile.
- Impact: rapid edits or close/reopen flows can restore removed servers, remove
  newly added ones, and show success while all connects failed; a stale task may
  overwrite a newer live capability set.
- Root cause: connection reconciliation is modeled as fire-and-forget work
  rather than one latest-generation state machine.
- Direction: use one application reconciliation owner with monotonically
  increasing generation, latest-wins cancellation/coalescing, a single terminal
  status, and shutdown ownership. Delete per-request `tokio::spawn` after it is
  replaced.
- Regression validation: control task/lock order for A then B saves, inject slow
  transports/timeouts and shutdown, and assert only B can commit/project status.
- Validation reports: [V04](../validations/A-INT-01/V04-01.md).

### A-INT-01-P1-04: Workspace transitions leave LSP processes, root, and base configuration bound to bootstrap

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/runtime.rs:499`, `:504`,
  `:509`, `:531`, `:559`, `:573`, `:591`;
  `echo-agent-cli/echo-agent-app-core/src/plugin_runtime.rs:25`, `:946`, `:950`,
  `:964`, `:966`, `:986`;
  `echo-agent-cli/echo-agent-app-core/src/state.rs:844`, `:872`, `:887`,
  `:1022`, `:1064`, `:1164`.
- Reachability: LSP tools are registered on the primary shared ToolManager and
  used by chat/task Agents; GUI workspace switch and exit execute the inspected
  AppState transitions.
- Expected invariant: after workspace switch/exit, diagnostics/navigation use
  the active workspace root and its nearest `.lsp.yaml`; old processes shut down
  through the same generation transition.
- Observed behavior: bootstrap snapshots `base_config`/`project_root`.
  `PluginRuntimeService::project_root` is dynamic only for plugin discovery;
  `prepare_lsp` still applies the captured root/base. Workspace transitions
  update Agent/Pool working directories and memory but never rebind LSP.
- Impact: diagnostics, definition, references and hover can operate on the wrong
  project or keep servers/config from the previous workspace, violating mode and
  workspace correctness for a core coding capability.
- Root cause: LSP lifecycle is attached to process/plugin bootstrap instead of
  EKO workspace generation.
- Direction: add one serialized workspace-capability transition that re-runs
  discovery/precedence, prepares a new manager for the active root, atomically
  swaps the shared handle, then shuts down old processes. Preserve framework LSP
  protocol ownership.
- Regression validation: switch A->B->global with distinct configs/languages;
  verify root, server set, in-flight cancellation, old shutdown, primary/pool
  tools, and invalid B config last-known-good behavior.
- Validation reports: [V05](../validations/A-INT-01/V05-01.md).

### A-INT-01-P2-05: Conversation deletion leaves Browser session metadata alive until process shutdown

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `echo-agent-cli/echo-agent-app-core/src/browser/session.rs:168`,
  `:181`, `:193`, `:495`, `:504`;
  `echo-agent-cli/echo-agent-app-core/src/browser/mod.rs:137`, `:149`, `:161`,
  `:185`, `:194`;
  `echo-agent-cli/src/tauri/commands/conversations.rs:585`, `:595`, `:600`,
  `:609`, `:639`.
- Reachability: Browser GUI/Tools allocate sessions keyed by conversation and
  persist them; the registered delete command follows the inspected cleanup
  path.
- Expected invariant: deleting a conversation removes or terminally closes all
  conversation-owned Browser state and metadata.
- Observed behavior: only process shutdown closes all sessions; there is no
  per-conversation session removal/close API. Delete removes conversation,
  Tool-execution and artifact data but does not touch Browser state. Approval
  provider leakage is separately owned by A-HITL-01-P2-07.
- Impact: deleted conversation IDs, URLs/titles and tabs remain resident and on
  disk, can appear in restored metadata, and accumulate independently of the
  user's deletion intent.
- Root cause: conversation cleanup is a partial command-local checklist rather
  than a lifecycle receipt over registered conversation-scoped resources.
- Direction: add an idempotent BrowserRuntime `close_conversation` that closes
  tabs, emits terminal events, deletes metadata and removes provider/diagnostic
  state; call it from canonical deletion alongside pool/session cleanup. Avoid a
  second Browser session store.
- Regression validation: delete active/failed/closed managed and extension
  sessions, repeat deletion, restart, and assert no metadata/provider/session
  entry remains while unrelated conversations continue.
- Validation reports: [V06](../validations/A-INT-01/V06-01.md).

## Validation Matrix

| ID | Claim | Required | Status | Report |
|---|---|---:|---|---|
| V00 | Revision and concurrent-dirty isolation | yes | passed | [report](../validations/A-INT-01/V00-01.md) |
| V30 | Primary source-anchor and reachability sample | yes | passed | [report](../validations/A-INT-01/V30-01.md) |
| V01-01 | Assumed dedicated LSP command path | retained failure | inconclusive | [report](../validations/A-INT-01/V01-01.md) |
| V01-02 | Definition, duplicate, layering, corrected reachability | yes | passed | [report](../validations/A-INT-01/V01-02.md) |
| V02 | MCP bootstrap/editor durable authority | yes | failed -> finding | [report](../validations/A-INT-01/V02-01.md) |
| V03 | Redacted secret edit round-trip | yes | failed -> finding | [report](../validations/A-INT-01/V03-01.md) |
| V04 | MCP reconnect latest-generation invariant | yes | failed -> finding | [report](../validations/A-INT-01/V04-01.md) |
| V05 | LSP workspace root/config lifecycle | yes | failed -> finding | [report](../validations/A-INT-01/V05-01.md) |
| V06 | Browser conversation deletion cleanup | yes | failed -> finding | [report](../validations/A-INT-01/V06-01.md) |
| V07 | Local interaction permission boundary/de-dup | yes | passed with inherited deviations | [report](../validations/A-INT-01/V07-01.md) |
| V08 | Existing test inventory | yes | passed with gaps | [report](../validations/A-INT-01/V08-01.md) |
| V09 | Dynamic lifecycle/race regression matrix | future | not_run by direction | [report](../validations/A-INT-01/V09-01.md) |
| V99 | Static report integrity gate | yes | passed | [report](../validations/A-INT-01/V99-01.md) |

## Historical Claim Status

| Dependency claim | Classification | Current evidence |
|---|---|---|
| `F-INT-01` MCP/LSP protocol, timeout, malformed response and URI findings | current framework issues; not duplicated | V01-02, V08 |
| `F-INT-02` LSP/A2A duplicate and protocol integration conclusions | current; EKO lifecycle only deepened | V01-02, V05 |
| `A-TOOL-01` direct interactive terminal is not permission-mode gated | current pattern; Browser/MCP/LSP corroborated | V07 |
| `A-HITL-01-P1-06` fixed GUI MCP launcher/private-host validators over-gate local extension | current and canonical; not duplicated | V07 |
| `A-HITL-01-P2-07` stale Browser conversation approval providers | current and canonical; Browser session cleanup is separately deepened | V06, V07 |

## Coverage And Uncertainty

- No dynamic command, build, test, fixture, application launch or network access
  was used. Exact race schedules and external process behavior remain future V09.
- The incorrect `lsp.rs` lookup remains immutable/inconclusive. Corrected
  searches proved LSP Tool reachability, so absence of direct LSP UI controls was
  not promoted to a finding.
- Concurrent framework dirty paths expanded during the final integrity gate and
  were excluded without reading their content/diffs. CLI remained clean.
  Framework MCP/LSP correctness relies only on the allowed completed Codex
  dependencies and unaffected source.
- MCP workspace switching was not separately reported: default MCP config is
  intentionally user-scoped (`~/.eko/mcp.json`) and the current loader avoids
  project CWD injection. P1-01 covers its actual selected-file authority defect.
- Browser `interrupt` deliberately closes client connections but retains session
  continuity; this was not treated as a defect. P2-05 concerns explicit
  conversation deletion.

## Handoff

- Establish one durable MCP config generation first; its API must preserve
  masked credentials and own latest-wins live reconciliation. Then delete the
  in-memory mirror and detached per-save task.
- Attach LSP manager/config/process replacement to the canonical EKO workspace
  generation while keeping generic LSP mechanics in `echo-agent`.
- Make conversation deletion invoke one registered resource-cleanup lifecycle,
  including Browser session metadata and the already canonical A-HITL provider
  cleanup.
- Do not reintroduce automation permission gates. Remove the over-broad MCP GUI
  restrictions under A-HITL-01 while retaining lightweight malformed input and
  cleartext-warning behavior suitable for a trusted local extension.
- Primary must independently sample source anchors and verify V99 before moving
  A-INT-01 from `needs_evidence` to `complete`.
