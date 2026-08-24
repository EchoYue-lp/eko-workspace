# S-APP-01: Application Review Synthesis

> **Superseded for cross-review decisions:** this independent report remains
> evidence, but the authoritative three-review reconciliation is
> [../../../application-review.md](../../../application-review.md).

> Status: complete
> Reviewer: ZCode-ds (deepseek-v4-flash)
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63 (baseline 9b0e0fa)
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5 (baseline b3b2e81)
> Worktree state: clean (both repositories; verified in V03-01)

## Question

Synthesize the completed EKO application review (all A-* task reports plus
Q-CLI-01 / Q-GUI-01 / Q-WEB-01 / Q-E2E-01) into a single canonical statement:
the A-phase P0/P1 findings clustered by theme, the multi-surface functional
parity ruling, the layering-compliance verdict, and the contradiction /
stale-handling record — each with stable IDs, file:line anchors, fix
directions, and validation-report links.

**Bottom line: the EKO application layer is architecturally sound (one chat
driver, one TaskRuntime authority, thin adapters, no SQLite, no `worker`
terminology, zero gate regressions, all submission gates green) but carries
25 canonical P1 findings. None are P0 (no data loss/corruption or secret
exposure was found; the webhook redaction gap is the closest P0-adjacent
item and is network-facing but opt-in). The dominant themes are: (1)
terminal/lifecycle facts that lie (GUI labels errors 'completed', paused
runs recorded as completed, cancel reported as agent_error, no typed
timeout terminal); (2) workspace-scope subsystems that do not follow the
workspace (config/hooks/watcher, CWD restore, plugins, subagent catalog,
task store, conversation store root, hot-memory projection); (3)
permission/HITL surfaces that silently misbehave (dead rule management,
EOF auto-approve, `*` wildcard approve-all, un-applied config); (4)
surface-parity gaps concentrated on CLI/channels/TUI (workspace, research,
evolution, browser management, task-run control, steering, cancellation,
services); (5) crash/recovery gaps in the TaskRuntime file authority and
the worktree lifecycle; (6) broken or dead management surfaces on the GUI
(double `.setup()` browser bridge, MCP config persistence) and the silent
read-only writer subagent that breaks the flagship Task capability on every
surface.**

## Scope And Inputs

- Synthesis of the 29 completed A-phase task reports
  (`docs/comprehensive-review/zcode-ds/reports/tasks/A-*.md`) and the
  declared dependencies `Q-CLI-01`, `Q-GUI-01`, `Q-WEB-01`, `Q-E2E-01` (all
  complete). `X-SRF-01` read as the parity matrix authority (A-SRF-01/04,
  A-CFG-01 conclusions consumed through it and re-anchored at their source
  reports).
- Out of scope: framework synthesis (S-FW-01), cross-repository synthesis
  (S-X-01), quality synthesis (S-QA-01), iteration roadmap (S-RDM-01);
  `codex/` and `zcode-glm/` not read per protocol. Fixes are deferred to
  S-RDM-01; this review is read-only.
- Layering classification used throughout: generic mechanism (framework) /
  EKO product policy (application) / adapter boundary, per REPORTING.md and
  AGENTS.md.

## 1. Canonical P0/P1 Finding Summary (clustered by theme)

**There are zero P0 findings in the A phase** (verified across all 29
reports; V01-01 census). The canonical P1 set is **25 findings** after
merging duplicates and re-ratings (28 filed P1 records − 4 merges + 1
re-rated P2→P1; see Section 4 and V02-01). Every row below carries the
canonical ID (the merge target), its merged duplicates as backlinks, the
tightest source anchor, a one-line statement, the fix direction, and the
canonical validation reports.

### Theme 1 — Persistence / crash consistency / recovery (5 P1)

| Canonical ID | file:line (tight anchor) | One-liner | Fix direction | Validations |
|---|---|---|---|---|
| A-TSK-01-P1-01 (merged: A-TSK-04-P1-03) | `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_shadow.rs:362-379` (+:289-299, comment :114-117) | A torn tail line in `events.jsonl` (crash mid-append) hard-errors every read and write of that run forever; boot recovery warns and gives up; only manual file editing recovers | Truncate the final line to the last valid boundary on decode error (or length-prefix records) and rebuild; torn-tail regression fixture | [A-TSK-01 V03-02](../validations/A-TSK-01/V03-02.md), [A-TSK-04 V03-01](../validations/A-TSK-04/V03-01.md) |
| A-TSK-03-P1-01 (merged: A-TSK-04-P1-01) | `echo-agent/echo-orchestration/src/tasks/runtime_executor.rs:390-416` + `echo-agent-cli/.../executor.rs:508,643-661` | Pause requested during an active wave is silently converted into a permanent run cancellation (framework hardcodes `Cancelled` in-wave; EKO `finalize_cancelled_run_state` force-transitions the durably-Paused run) | Consult `controller.interruption_outcome` after the wave drain; `resolve_dispatch` writes Pending (not Cancelled) when durably Paused; finalize skips Paused runs | [A-TSK-03 V03-01](../validations/A-TSK-03/V03-01.md), [A-TSK-04 V03-01](../validations/A-TSK-04/V03-01.md) |
| A-TSK-03-P2-01 → re-rated P1 (merged: A-TSK-04-P1-02) | `echo-agent-cli/.../executor.rs:570-582` (Err branch) + `runtime_executor.rs:379-381,418-421` | A mid-wave store fault marks the run Failed with no sibling Running-task cleanup; same-process resume polls forever — only a restart heals (recovery error, canonical P1) | EKO: run the pause-path task reset on the Err outcome before Failed; framework: drain/resolve sibling claims on wave error | [A-TSK-03 V03-01](../validations/A-TSK-03/V03-01.md), [A-TSK-04 V03-01](../validations/A-TSK-04/V03-01.md) |
| A-STATE-01-P1-01 | `echo-agent-cli/echo-agent-app-core/src/state.rs:1078-1083` | Exiting a workspace re-roots the conversation store to the legacy `~/.eko/sessions/conversations/`; global history vanishes and post-exit writes land in a store boot never reads | Use `infra::create_conversation_store()` (user_data_dir root) in `exit_workspace`; delete the `Persistence::base_dir()`-based construction | [A-STATE-01 V02-01](../validations/A-STATE-01/V02-01.md), [A-STATE-01 V03-01](../validations/A-STATE-01/V03-01.md) |
| A-EVO-01-P1-01 | `echo-agent-cli/src/cli/repl.rs:256,340-411` | REPL session exit auto-runs an LLM "reflection" appended to `.eko/memory/PROJECT.md` — a semantic write outside the review gate, change log, security guard, and docs; CLI-only | Remove it and route session learnings through the Review Inbox (auto-memory extraction already at repl.rs:253), or make it opt-in + audited + bounded | [A-EVO-01 V02-01](../validations/A-EVO-01/V02-01.md), [A-EVO-01 V03-01](../validations/A-EVO-01/V03-01.md) |

### Theme 2 — Permission / approval semantics (4 P1)

| Canonical ID | file:line | One-liner | Fix direction | Validations |
|---|---|---|---|---|
| A-HITL-01-P1-01 | `echo-agent-cli/src/tauri/commands/panels.rs:102-151` + `echo-agent-app-core/src/state.rs:145-201` | GUI permission-rule management is behaviorally dead — rules are stored/listed but no code ever applies them to a tool call (all helpers have zero callers) | Apply rules to the shared `PermissionService` + pool agents via the framework rule API, or delete the commands/helpers | [A-HITL-01 V01-01](../validations/A-HITL-01/V01-01.md), [A-HITL-01 V03-02](../validations/A-HITL-01/V03-02.md) |
| A-HITL-01-P1-02 | `echo-agent-cli/echo-agent-app-core/src/hitl/repl_provider.rs:69-77` | REPL provider auto-approves on empty/EOF stdin (`""` → Approved) and its blocking `read_line` defeats the dispatcher's shared 5-minute deadline | Treat EOF distinctly (`Ok(0)` → Rejected), keep "enter to approve" only for a TTY, move the read to async stdin | [A-HITL-01 V01-01](../validations/A-HITL-01/V01-01.md), [A-HITL-01 V03-01](../validations/A-HITL-01/V03-01.md) |
| A-HITL-01-P1-03 (consumer of F-HITL-01-P1-03) | `echo-orchestration/src/human_loop/service.rs:898-908` + all four surface producers (tui/events.rs:242-244, repl_provider.rs:77-79, channel_provider.rs:158-160, chat.rs:335-342) | Every surface's "approve all" sends `SessionAllTools` → framework `"*"` wildcard = ALL tools allowed for the session; EKO is the sole producer | Framework: map to tool-scoped rules; EKO: map GUI "本会话同意" to per-tool Session and relabel, or expose both granularities | [A-HITL-01 V02-01](../validations/A-HITL-01/V02-01.md), [A-HITL-01 V03-02](../validations/A-HITL-01/V03-02.md) |
| A-INP-01-P1-01 | `echo-agent-cli/src/tui/events.rs:4240-4294` + `echo-agent-app-core/src/prepared_turn.rs:302,372-384` | TUI `/steer` on a non-steerable turn deletes the staged paste file during build; the queued re-send then fails and the pasted content is lost from disk | Re-queue the prepared turn (durable artifact refs) instead of raw refs, or defer `cleanup_staged_paste_files` until delivery | [A-INP-01 V02-01](../validations/A-INP-01/V02-01.md), [A-INP-01 V03-01](../validations/A-INP-01/V03-01.md) |

### Theme 3 — Terminal / final-state semantics (6 P1)

| Canonical ID | file:line | One-liner | Fix direction | Validations |
|---|---|---|---|---|
| A-CHAT-01-P1-01 | `echo-agent/echo-core/src/agent/event_envelope.rs:134-191` + `echo-agent-cli/src/tauri/commands/chat.rs:690-696` + `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:540-561` | `drive_chat`'s Result is decoupled from the agent-stream terminal (envelope never yields Err): GUI labels error-terminated turns "completed" and user cancels render a fabricated error | Return a typed `TurnOutcome` from `drive_chat` (from the last envelope payload + cancel token) and derive `TurnStatus` from it | [A-CHAT-01 V02-01](../validations/A-CHAT-01/V02-01.md), [A-CHAT-01 V03-01](../validations/A-CHAT-01/V03-01.md) |
| A-SRF-03-P1-01 | `echo-agent-cli/src/tauri/commands/chat.rs:511-535` + `web-frontend/src/hooks/useTauriChat.ts:188-190,253-259` | The interrupt prompt (message while a TaskRuntime run is active) strands the frontend turn state — no terminal event ever arrives for the ghost key, so the chat input queues forever until reload | On `kind:'interrupt_prompt'` roll back the optimistic turn (clear refs, restore status, drain queue) or have the backend emit a terminal for the interrupted key | [A-SRF-03 V02-01](../validations/A-SRF-03/V02-01.md), [A-SRF-03 V03-02](../validations/A-SRF-03/V03-02.md) |
| A-SRF-03-P1-02 | `web-frontend/src/stores/chatStore.ts:354-362` + `hooks/chatEventHandler.ts:140-150` | Error/cancel turns end with `runStatus 'completed'` ("就绪") and wipe the streamed partial answer — the frontend re-produces the backend lie | Split content finalization from status; keep partial content on Error/Cancel; make terminal transitions monotone | [A-SRF-03 V03-01](../validations/A-SRF-03/V03-01.md), [A-SRF-03 V05-01](../validations/A-SRF-03/V05-01.md) |
| A-OBS-01-P1-01 | `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/executor.rs:562-568,3521-3526,570-582` | `save_trace` records a paused (resumable) run as `Completed` and writes no terminal record on executor faults — diagnostics report false lifecycle facts | Pass a typed terminal outcome to `save_trace`; write a Failed record on the Err branch with the real cause | [A-OBS-01 V02-01](../validations/A-OBS-01/V02-01.md), [A-OBS-01 V03-01](../validations/A-OBS-01/V03-01.md) |
| A-OBS-01-P1-03 (webhook arm of A-CHAT-01-P1-01) | `echo-agent-cli/echo-agent-app-core/src/webhook/events.rs:9-33` + `chat_driver.rs:153-175` | Webhook channel reports user cancellations as a fabricated `agent_error` and has no cancel/failure chat terminal variants — external consumers cannot distinguish outcomes | Add `chat_cancelled`/`chat_failed` variants emitted from the driver's typed outcome; suppress `AgentError` for cancel-driven terminals | [A-OBS-01 V02-01](../validations/A-OBS-01/V02-01.md), [A-OBS-01 V03-01](../validations/A-OBS-01/V03-01.md) |
| A-SRF-04-P1-01 | `echo-agent-cli/src/cli/repl.rs:533,234-237` + `src/cli/channels.rs:244` + `src/cli/modes.rs:228` | REPL and IM-channel turns are not cancellable (fresh tokens, zero producers, inline-await input loop) and the CLI path has no signal handler — mid-turn Ctrl+C kills the process, skipping shutdown hooks | Retain the token; install a signal handler in CLI mode; race the turn await against input/signal; cancel per-turn token on channel stop | [A-SRF-04 V03-01](../validations/A-SRF-04/V03-01.md), [A-SRF-04 V01-01](../validations/A-SRF-04/V01-01.md) |

### Theme 4 — Mode parity / workspace & surface scope (7 P1)

| Canonical ID | file:line | One-liner | Fix direction | Validations |
|---|---|---|---|---|
| A-CFG-01-P1-01 | `echo-agent-cli/echo-agent-app-core/src/state.rs:844-1032,854` + `config_watcher.rs:199-211` + `hook_config_loader.rs:184-185` | Workspace switch mutates the process CWD but leaves watcher targets, hook registry, and AppConfig bound to the pre-switch scope — new project hooks never load, old-scope edits reload the new scope | Rebuild watcher targets + re-merge hooks for the new cwd on switch (or document config global and stop chdir) | [A-CFG-01 V02-01](../validations/A-CFG-01/V02-01.md), [A-CFG-01 V03-01](../validations/A-CFG-01/V03-01.md) |
| A-CFG-01-P1-02 | `echo-agent-cli/echo-agent-app-core/src/state.rs:1053-1185` (vs :854) | `exit_workspace` never restores the process CWD — tools/config/hooks keep resolving inside the exited (often deleted) workspace | Capture the pre-switch CWD and restore it on exit (guarded); add a real GUI exit command for the dead frontend `exit()` | [A-CFG-01 V02-01](../validations/A-CFG-01/V02-01.md), [A-CFG-01 V03-01](../validations/A-CFG-01/V03-01.md) |
| A-CFG-01-P1-03 (merged: A-SRF-01-P1-01) | `echo-agent-cli/src/cli/cmd_impls/workspace.rs:114-146` + `src/tui/commands.rs:58` (no workspace variant) + `src/tui/events.rs:1368/3266` ("TUI has no workspace concept" comments) | Workspace switching is GUI-only; REPL `/workspace switch` and `/model` are print-only stubs that claim success; the TUI has no workspace surface at all | Wire `/workspace switch|exit` on REPL/TUI to `AppState::switch_workspace`/`exit_workspace` (or a TUI workspace tab); replace stubs with behavior; delete the positioning comments | [A-CFG-01 V02-01](../validations/A-CFG-01/V02-01.md), [A-SRF-01 V01-01](../validations/A-SRF-01/V01-01.md), [A-SRF-01 V02-01](../validations/A-SRF-01/V02-01.md) |
| A-SRF-04-P1-02 (re-rated from A-BOOT-01-P2-02) | `echo-agent-cli/src/main.rs:365,389-403` + `src/cli/modes.rs:118-235` | Channels-only mode omits the scheduler and the background task service — cron never exists and background runs never resume on that surface (parity invariant violation) | Call the shared headless-service assembly inside `run_channels_mode`; unify the channels-only branch with the CLI branch | [A-SRF-04 V02-01](../validations/A-SRF-04/V02-01.md), [A-SRF-04 V03-01](../validations/A-SRF-04/V03-01.md), [A-SRF-04 V04-06](../validations/A-SRF-04/V04-06.md) |
| A-PLG-01-P1-01 | `echo-agent-cli/echo-agent-app-core/src/state.rs:844-1032` (zero plugin refs) + `plugin_runtime.rs:608,966,986-991` | Workspace switch leaves project-scope plugin components live and the LSP root boot-frozen — the previous workspace's plugin hooks/monitors/Subagents keep firing in the new workspace | Invoke `plugin_runtime.reload()` on switch/exit; derive the LSP project root per apply instead of boot-fixed | [A-PLG-01 V02-01](../validations/A-PLG-01/V02-01.md), [A-PLG-01 V03-01](../validations/A-PLG-01/V03-01.md) |
| A-MEM-01-P1-01 (exposure: A-EVO-01-P2-02) | `echo-agent-cli/echo-agent-app-core/src/unified_memory.rs:154-167` (dead helper) + 8 wrong-target refresh sites (infra.rs:1175-1192, memory.rs:126-145/221-239, events.rs:2838-2852/2913-2927, all.rs:120-141/195-205, agent_pool.rs:687-710) | Every in-session hot-layer (MEMORY.md) mutation refreshes the instruction projection instead of the hot-memory projection — active memory stays stale in live context until boot/switch/exit | Replace the eight sites with `refresh_memory_projections`; make `refresh_instruction_context` refresh both; wire `on_memory_layer_change` | [A-MEM-01 V02-01](../validations/A-MEM-01/V02-01.md), [A-MEM-01 V03-01](../validations/A-MEM-01/V03-01.md) |

### Theme 5 — Security / outbound (1 P1)

| Canonical ID | file:line | One-liner | Fix direction | Validations |
|---|---|---|---|---|
| A-OBS-01-P1-02 | `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:124,148-157` + `webhook/emitter.rs:138-149` | Webhook payloads carry up to 240 chars of raw tool arguments and raw error text to external endpoints with zero secret redaction (contrast: framework redacts the same class in trace events) | Apply `echo_agent::security::redact_secrets` (+ bounded truncation) at a single choke point in the observer/emitter before serialization | [A-OBS-01 V01-01](../validations/A-OBS-01/V01-01.md), [A-OBS-01 V03-01](../validations/A-OBS-01/V03-01.md) |

### Theme 6 — Broken / dead surfaces (3 P1)

| Canonical ID | file:line | One-liner | Fix direction | Validations |
|---|---|---|---|---|
| A-SRF-02-P1-01 | `echo-agent-cli/src/tauri/mod.rs:40-68` vs `:311-772` (Tauri Builder::setup overwrite, vendored app.rs:1765-1769) | `build_tauri_app` registers two `.setup()` closures; Tauri runs only the last — the `browser://event` forwarder and DevTools auto-open never execute, so the GUI browser workspace panel is dead | Merge both closures into a single `.setup()`; add a builder-level test asserting exactly one setup closure | [A-SRF-02 V02-01](../validations/A-SRF-02/V02-01.md), [A-SRF-02 V03-02](../validations/A-SRF-02/V03-02.md) |
| A-INT-01-P1-01 | `echo-agent-cli/src/tauri/commands/mcp.rs:476-559` + `echo-agent-app-core/src/state.rs:357,490` | GUI MCP config editor never persists to disk and is never seeded from the on-disk file — every GUI-created server and disabled flag silently disappears on restart while the UI says "配置已保存并应用" | Persist `update_mcp_config` to `~/.echo-agent/mcp.json` (atomic) and seed `plugins.mcp_config` from the same path at boot | [A-INT-01 V01-01](../validations/A-INT-01/V01-01.md), [A-INT-01 V02-01](../validations/A-INT-01/V02-01.md) |
| A-TOOL-01-P1-01 (F-EXT-01-P1-01; arm: A-SUB-01-P2-01; scenario: Q-E2E-01-P1-02) | `echo-agent-cli/echo-agent-app-core/src/infra.rs:963` (`set_plan_mode(true)` in `build_writer_subagent_agent`) + framework filter snapshot.rs:227-236/282-285, pipeline.rs:1004-1018 | Writer subagents (Implementation/Debugging tasks) are silently read-only — write tools are invisible to the model and blocked at three layers, so Task runs "complete" with pristine worktrees on every surface | Remove `set_plan_mode(true)` from the writer builder (keep infra.rs:1040); add a write-tool visibility test | [A-TOOL-01 V02-01](../validations/A-TOOL-01/V02-01.md), [A-TOOL-01 V03-01](../validations/A-TOOL-01/V03-01.md), [A-SUB-01 V03-01](../validations/A-SUB-01/V03-01.md) |

**Cross-references for the roadmap (P2 canonicals that gate P1 fixes or
surface as families):** A-TSK-01-P2-01 (read-side rebuild gap — same crash
window as P1-01), A-TSK-04-P2-01 (unguarded `set_task_status` block writes),
A-TSK-05-P2-01..04 (stale fork lock, missing fork sweep, non-isolated writer
routing, panels.rs worktree duplication), A-CFG-01-P2-01..05 (path map
contradiction, task-store workspace binding, orphan `web_config`, silent
save failures, boot/switch window divergence), A-HITL-01-P2-01..04 (GUI
dispatcher bypass, channel cross-sender, denial escalation, dead `IpcAuth`),
A-SRF-02-P2-01 (double subagent-tool producer — canonical for the frontend
A-FE-02-P2-01 / A-SRF-03-P2-01 duplicate-card family), A-OBS-01-P2-01
(`save_trace` second trace authority), A-INT-01-P2-01..05 (CLI /mcp stubs,
all-or-nothing boot MCP, SSRF-style URL over-gating, no MCP auto-recovery,
browser confirmation inheritance), A-PROJ-01-P2-01..04 (dead ProjectIndex,
stale TUI completion, three diff engines, write-only change tracker),
A-DOM-01-P2-01/02 (undisclosed auto-ingest side effect, run-artifact
destruction on rerun), A-FE-01-P2-01/02 (ToolInfo and SkillInfo/McpServerInfo
dual wire shapes), A-FE-02-P2-01..03 (live-ingest identity, revision-blind
latest-attempt selector, acceptance/check/artifact surface gap), A-FE-03-P2-01/02
(render-bounded streaming, two-mirror turn lifecycle), A-STATE-01-P2-01/02
(workspace store root off-by-one, no cross-process serialization),
A-EVO-01-P2-01..03 (unaudited L3 exposure, wrong-projection refresh, TUI
evolution surface gap), A-OUT-01-P2-01..04 (unfiltered session export,
TUI table byte/char mismatch, decorative format machinery, TUI research
surface gap), A-SUB-01-P2-02..04 (plugin catalog invisibility, plugin prompt
compiler bypass, no subagent reload on switch), A-TSK-06-P2-01 (dead runtime
artifact projection), Q-E2E-01-P2-01 (REPL zero conversation persistence),
Q-DEP-01-P2-01 (6 active RUSTSEC advisories in the shipped binary).

## 2. Multi-Surface Functional Parity Ruling

**Ruling: the shared-core parity claim holds at the agent-capability layer
and fails at the management/control layer.** All six entry classes (GUI,
TUI, CLI REPL, channels, cron, background) reach the same shared core — one
`AgentRuntime::bootstrap`, one `drive_chat` application driver (4 live call
sites), one TaskRuntime store + `task_create/update/list` + `task_execute`,
one AgentPool, one `PreparedUserTurn`, one tool executor, one browser
runtime, one HITL dispatcher (X-SRF-01 V01-01/V02-01; re-anchored by
Q-E2E-01 V01-V46). Every deviation below is a surface adapter gap or a
shared-core defect that surfaces on a subset of surfaces — never a deliberate
"mode doesn't use Y" policy (AGENTS.md historical lesson). Matrix rows cite
the canonical source (A-SRF-01/04, A-CFG-01, X-SRF-01, Q-E2E-01).

| Capability | GUI | TUI | CLI/REPL | Channels | Cron | Background | Canonical source |
|---|---|---|---|---|---|---|---|
| Chat (drive_chat) | ✓ (broken terminals) | ✓ | ✓ (no cancel) | ✓ (no cancel) | manual only | ✓ | A-CHAT-01, A-SRF-04-P1-01, F-OPS-01-P1-01 |
| Task execution | ✓ (writer read-only) | ✓ (same) | ✓ (same) | ✓ (same) | manual only | ✓ (same) | A-TOOL-01-P1-01 / Q-E2E-01-P1-02 |
| Task-run management (list/pause/cancel/resume/retry) | ✓ | ✓ | ✓ | ✗ none | resume via TUI/GUI only | resume via TUI/GUI only | X-SRF-01-P2-02, A-SRF-04-P2-01 |
| Workspace switching | ✓ | ✗ | ✗ stub prints success | n/a | n/a | n/a | A-CFG-01-P1-03 / A-SRF-01-P1-01 |
| Research/export workbench | ✓ | ✗ | ✓ | partial (NL papers) | n/a | n/a | A-OUT-01-P2-04, A-SRF-01-P2-02 |
| Evolution mutations (rule-promote/curator/skill lifecycle) | ✓ | ✗ | ✓ | n/a | n/a | n/a | A-EVO-01-P2-03 |
| Browser management | ✗ panel dead (browser://event) | ✓ | ✗ | ✗ | n/a | n/a | A-SRF-02-P1-01, X-SRF-01-P2-01, Q-E2E-01-P2-03 |
| MCP management | partial (config non-durable) | ✓ | ✗ connect/disconnect stubs | ✗ | n/a | n/a | A-INT-01-P1-01/P2-01 |
| Steering | ✓ | ✓ | ✗ | ✗ | n/a | n/a | X-SRF-01-P3-01 |
| Turn cancellation | ✓ | ✓ | ✗ | ✗ | n/a | n/a | A-SRF-04-P1-01 |
| Interrupt prompt (run in progress) | ✓ (broken: ghost turn) | ✗ silent 2nd turn | ✗ | ✗ | n/a | n/a | A-CHAT-01-P2-01, A-SRF-03-P1-01 |
| Conversation persistence / restart | ✓ | ✓ | ✗ | ✗ | n/a | n/a | Q-E2E-01-P2-01, A-SRF-04-P1-02 |
| Cron scheduling | ✓ | ✓ | ✓ | ✗ no service | — (trigger dead) | n/a | A-SRF-04-P1-02, F-OPS-01-P1-01 |
| Background service | ✓ | ✓ | ✓ | ✗ no service | n/a | ✓ | A-SRF-04-P1-02 |
| Session-end memory review | ✗ | ✓ | ✓ | n/a | n/a | n/a | A-BOOT-01-P2-03 |
| Dreaming | ✓ | ✓ | ✓ | ✗ | n/a | n/a | A-MEM-01-P3-01 |
| HITL approval transport | partial (bypasses dispatcher; rules dead) | ✓ cleanest | ✗ EOF auto-approve | ✗ cross-sender | ✗ auto-reject | ✗ auto-reject + opaque | A-HITL-01-P1-01/P1-02/P2-01/P2-02/P2-03 |
| Truthful terminal facts | ✗ error→completed, cancel→fabricated error | ✓ | ✓ (but no cancel) | ✓ (but no cancel) | ✗ no typed failure terminal | ✗ same | A-CHAT-01-P1-01, A-SRF-03-P1-02, A-OBS-01-P1-01/P1-03 |
| LSP tools | ✓ primary only | ✓ primary only | ✓ primary only | ✗ | ✗ | ✗ | A-INT-01-P3-02 |

Scenario-level verdicts (Q-E2E-01 V01-V46): Chat GUI fails (P1-01), Task
fails on all surfaces (P1-02), cron fails at the trigger on all surfaces
(P1-03), HITL fails on CLI/channel/cron/background (P2-02), Browser/MCP
connect fails on GUI/CLI/channel (P2-03), Restart fails on CLI/channel
(P2-01 + A-SRF-04-P1-02); attachment, tool rendering, subagent dispatch/
result, and large-output fold/cursor pass on all applicable surfaces. TUI is
the only surface that is simultaneously complete on browser/MCP management
and truthful on terminals — the parity baseline the other surfaces must
converge to.

## 3. Layering Compliance Conclusions (positives)

These are the affirmative answers the A phase produced; they are load-bearing
for S-RDM-01 (protect them while fixing the P1s):

1. **TaskRuntime thin-shell compliance (AGENTS.md gates 1-6).** Exactly one
   live DAG execution loop (`echo_orchestration::tasks::RuntimeDagExecutor`);
   EKO's `EkoRuntimeDagController` injects only product policy (ownership-safe
   wave filter, review/acceptance policy, worktree integration, durable-result
   reuse, drain completion gate). No second ready frontier, retry state
   machine, cancellation loop, or stall detection exists in EKO (A-TSK-03
   V01-01/V02-01; A-TSK-01 V01-01; A-TSK-02 V01-01 — zero forbidden
   `todo_write`/`plan_*` CRUD). The adapters are thin and lossless:
   `EkoRevisionedTaskStore` implements only `load`/`compare_and_commit`
   (revisioned_adapter.rs:26-56) and field-level round-trips are tested
   (A-TSK-01 V03-01, V04-01..06).
2. **Worktree protection (A-TSK-05).** No P0/P1 data-loss vector in the
   isolated-writer pipeline: staged-index refusal, dirty-overlap refusal,
   `git merge-tree` preflight, failure-preserving abort, execution-id trailer
   idempotency, repo-level merge lock, per-logical-task worktree reuse, and
   hard-fail when isolation is declared but no factory exists. 25+9 green
   tests (V04-01..05).
3. **Thinking zero-leakage (A-TSK-06).** Thinking tokens are routed to
   realtime events only and never enter `output`, review prompts, summaries,
   capsule, memory, or hooks at either dispatch path
   (framework subagent/executor.rs:1200-1222; EKO executor.rs:3155-3182;
   prompt.rs:161-163). Review consumes the complete output (not the bounded
   summary) and restart recovery reuses identical evidence (V02-01, V04-04).
4. **Single chat driver and one-terminal-per-turn envelope (A-CHAT-01).** One
   `drive_chat` definition, four live call sites, three stateless sinks; the
   envelope enforces exactly one agent terminal per turn (V01-01/V02-01).
5. **No SQLite, zero `worker` terminology, panic-safe surface (Q-CLI-01;
   X-INV-01).** `cargo tree` shows zero sqlite crates in the full reachable
   graph under all features (V06-01); all six CLI gate commands exit 0
   including the panic-safety clippy pass (V03-01); the A-phase reports
   independently grep-verified zero `worker` terms in every touched module.
6. **Direct-user interactions are gate-free by design (A-HITL-01 V03-02,
   A-TOOL-01).** `create_terminal`/`connect_mcp_server` carry no
   `permission_mode` gate (AGENTS.md historical lesson holds); the only
   gate remnants (`IpcAuth`/`IpcPermission`, error.rs) are dead and are
   deletion targets (A-HITL-01-P2-04, A-SRF-02-P3-01, A-TOOL-01-P3-01).
7. **Sandbox fail-closed (A-TOOL-01).** `run_code` enforces
   `IsolationLevel::OsSandbox` as a hard floor with no bare fallback; EKO
   probes and removes run_code when no local sandbox exists (infra.rs:265-272,
   533-543).
8. **Submission gates all green (Q-CLI-01, Q-GUI-01, Q-WEB-01).** Rust
   workspace gate 6/6 exit 0; GUI feature matrix check + 48 tests exit 0;
   frontend prettier/vitest(26 files/101 tests)/build exit 0 at the reviewed
   commits. (Caveat recorded: the gates certify compile/unit health, not
   boot composition — Q-GUI-01-P3-01; the A-SRF-02-P1-01 double-setup defect
   is invisible to them.)
9. **Evolution stays explicit diagnostics + review gate (A-EVO-01).** Trigger
   capture, Review Inbox, rule promotion, and Dreaming are user-gated or
   deterministic; the single live violation is A-EVO-01-P1-01 (REPL
   reflection). Framework evolution APIs are consumed, not deleted.
10. **File authority steady state is sound (A-TSK-01/04).** `events.jsonl`
    is the sole write authority; `plan.json`/`run-state.json` are derived
    projections; claim CAS gates every executor terminal write; stale
    revisions rejected at tool boundary and CAS; boot recovery is
    claim-aware and heals orphaned Running tasks.

## 4. Contradiction And Uncertainty Handling (V02)

All identified conflicts were resolved by re-opening the smallest relevant
validation at the reviewed commits (details and commands in V02-01):

| Conflict | Filed priorities | Resolution (canonical) |
|---|---|---|
| Mid-wave store fault (A-TSK-03-P2-01 vs A-TSK-04-P1-02) | P2 vs P1 | **P1** — recovery-error class per REPORTING.md; A-TSK-03's P2 was likelihood-weighted (trigger probability medium), preserved as a note |
| Channels-only services (A-BOOT-01-P2-02 vs A-SRF-04-P1-02) | P2 vs P1 | **P1** — violates the AGENTS.md surface-parity product invariant; A-BOOT-01 retained as backlink |
| Scheduler/background tokens never cancelled (A-BOOT-01-P3-02 vs A-SRF-04-P2-02) | P3 vs P2 | **P2** — material consequence (process-exit-only stop → Paused-at-boot orphans of A-SRF-04-P2-01); A-BOOT-01-P3-02 retained as backlink |
| Live tool-ingest keyed by `detail_ref` (A-SRF-03-P2-01 vs A-FE-02-P2-01) | P2 both | **Merged under A-FE-02-P2-01** (adds the status-rank monotonicity gap); backlink to A-SRF-03-P2-01 retained |
| Writer plan_mode (A-TOOL-01-P1-01 vs A-SUB-01-P2-01) | P1 vs P2 | **A-TOOL-01-P1-01 canonical**; A-SUB-01-P2-01 kept as the plugin-vs-md divergence arm (P2, distinct fix surface) |
| Hot-projection refresh (A-MEM-01-P1-01 vs A-EVO-01-P2-02) | P1 vs P2 | **A-MEM-01-P1-01 canonical** (as the source reports already declared); A-EVO-01-P2-02 is the evolution-surface exposure |
| zcode-ds/README A-phase count "20 P1 across 12 tasks" | — | Stale preliminary summary (predates A-TSK-03/A-TSK-04; internally inconsistent — names 14 tasks summing to 21); superseded by this synthesis's census-derived **25 canonical P1s**. README not modified (not this task's deliverable) |
| A-SRF-04-P2-02 header cross-ref "A-BOOT-01-P3-03" | — | ID slip; correct target is A-BOOT-01-P3-02 (P3-03 is the PTY `close_all` finding). Recorded as a factual correction |

**Preserved open questions (minority/uncertain conclusions, not erased):**
A-INT-01-P2-03 (GUI MCP URL rejection: SSRF-style over-gating vs deliberate
choice — medium confidence); A-TOOL-01-P2-01 (tool toggle: gate vs
display-only — product decision); A-SUB-01-P2-04 and A-PLG-01-P1-01
(refresh-on-switch vs document-boot-only — product decision); A-TSK-06-P2-01
(wire vs delete the runtime artifact projection); A-CHAT-01-P2-01 (route
interrupt through the shared sink vs delete the dead variant); A-SRF-01-P2-02
(research workbench placement on TUI — product decision); channels
concurrent-turn interleaving on one pool agent (X-SRF-01 residual,
statically unresolvable); REPL-provider EOF context (b) concrete GUI flows
(A-HITL-01 residual).

**Stale handling (V03):** both repositories are at the baseline commits
(9b0e0fa / b3b2e81) with clean worktrees — no A-phase finding is stale on
commit grounds; every finding is carried as `current`. The 79-file
`web-frontend/src/generated/` formatting drift recorded by A-FE-01/A-FE-02/
A-FE-03/A-SRF-03 was external churn, already restored before Q-WEB-01; the
mechanism conclusion (regeneration writes unformatted output, A-FE-01-P3-02
= Q-WEB-01-P3-01) stays `current` as a latent build-hygiene item.

## 5. Coverage And Uncertainty

- All conclusions are syntheses of static task reports plus the gate
  executions recorded there; no process was launched in this synthesis and
  none was re-run (V01-V03 are static checks over the reports and the two
  repositories' HEADs).
- Dynamic verification (real LLM turns, GUI launch, IM traffic, browser
  sidecar) remains `not_run` for environmental reasons (Q-E2E-01 V47-V49).
- The parity matrix rows reflect the surface capabilities as of the reviewed
  commits; a product decision documented for any row (e.g., research
  workbench placement) may legitimately close a gap without a code fix — the
  matrix requires a documented decision, not necessarily an implementation.
- Priority is not inflated for the P1-adjacent items: A-INT-01-P1-01 borders
  P0 (silent loss of user configuration with a false success message) but is
  local-only and recoverable; A-OBS-01-P1-02 is network-facing but opt-in
  (webhooks configured) and local-threat-model-framed — both stay P1.

## 6. Handoff

- **S-X-01 may rely on:** the canonical merge table (Section 4) for
  cross-repository duplicate reconciliation (F-* ↔ A-* families:
  F-HITL-01-P1-01→A-HITL-01-P2-03, F-HITL-01-P1-03→A-HITL-01-P1-03,
  F-EXT-01-P1-01→A-TOOL-01-P1-01, F-RCT-03-P1-01/P1-02→A-CHAT-01-P1-01/
  A-SRF-03-P1-02, F-OPS-01-P1-01→A-SRF-04 family, F-TSK-03-P2-01/P2-02→
  A-TSK-03-P1-01/P2-01/P2-02); the boundary-gate classifications recorded
  per report (all findings above are application or adapter; none recommend
  framework movement except the framework-side halves of the pairs above).
- **S-QA-01 may rely on:** the A-phase validation matrix statuses recorded
  per report (all A tasks complete; every required validation has an
  immutable report; V03-01 of A-OBS-01 failed by design with findings).
- **S-RDM-01 roadmap order implied by this synthesis:** (1) correctness and
  data integrity first — A-TSK-01-P1-01 torn-tail repair, A-TSK-03-P1-01
  pause fix, A-STATE-01-P1-01 store root, A-TOOL-01-P1-01 writer capability
  (breaks the flagship Task feature on all surfaces), A-SRF-03-P1-01/P1-02
  truthful terminals; (2) authority convergence — save_trace deletion
  (A-OBS-01-P2-01), single diff engine (A-PROJ-01-P2-03), MCP config
  persistence (A-INT-01-P1-01), single `.setup()` (A-SRF-02-P1-01); (3)
  surface parity — workspace on TUI/REPL (A-CFG-01-P1-03), channels services
  (A-SRF-04-P1-02), REPL/channel cancel + steer (A-SRF-04-P1-01, X-SRF-01-
  P3-01); (4) permission/HITL leaves (A-HITL-01-P1-02/P1-03, rules wiring);
  (5) security — webhook redaction choke point (A-OBS-01-P1-02) and RUSTSEC
  advisories (Q-DEP-01-P2-01); (6) maintainability/dead-code deletions
  (IpcAuth, ProjectIndex, OutputFormat cluster, SessionSearchEngine,
  TasksPanel/PlanEditor/ResultFullView, `/critiques clear`, advanced.rs cron
  stub). Every item above carries acceptance/regression criteria and
  deletion targets in its source finding (Sections 1-3 links).
- **Stale triggers for this synthesis:** any change to the anchors listed in
  Section 1 (events.jsonl read/append, runtime_executor wave/cancel,
  exit_workspace/switch_workspace, repl.rs exit sequence, repl/channel
  providers, chat.rs interrupt/terminal, chatStore/chatEventHandler,
  unified_memory refresh sites, mod.rs setup closures, mcp.rs
  update_mcp_config, infra.rs writer builder), or a change of either
  repository HEAD.
- **Reports to read:** this report + its three validation reports
  (validations/S-APP-01/V01-01, V02-01, V03-01) + the 29 A-* task reports +
  Q-CLI-01/Q-GUI-01/Q-WEB-01/Q-E2E-01/X-SRF-01.
