# S-APP-01: Application Review Synthesis (echo-agent-cli / EKO)

> **Superseded for cross-review decisions:** this independent report remains
> evidence, but the authoritative three-review reconciliation is
> [../../application-review.md](../../application-review.md).

> Synthesis task: S-APP-01
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Baseline: `echo-agent` `9b0e0fa`, `echo-agent-cli` `b3b2e81`
> Sources synthesized: 29 A-phase task reports (`A-*.md`) + `Q-STA-01.md` + `Q-DEP-01.md`
> Synthesis date: 2026-08-12

This document merges, deduplicates, and prioritizes every finding produced by
the application review phase. Canonical IDs (`APP-*`) retain backlinks to the
originating task reports. Contradictions between reports are resolved inline.
The action list at the end sequences the work.

---

## 1. Finding Count Summary

| Priority | Count | Notes |
|---|---:|---|
| P0 | 0 | — |
| P1 | 8 | includes 1 shared with framework (UTF-8 panic), 3 secret-leak class |
| P2 | 57 | parity gaps, dead code, refresh-wiring bugs, lost durability |
| P3 | 74 | test gaps, doc drift, a11y, naming/hygiene, latent edge cases |
| **Total** | **139** | |

Per-report breakdown (reports with at least one finding):

| Report | P1 | P2 | P3 | Total |
|---|---:|---:|---:|---:|
| A-CFG-01 | 2 | 5 | 2 | 9 |
| A-FE-03 | 0 | 3 | 6 | 9 |
| A-OUT-01 | 0 | 3 | 4 | 7 |
| A-SRF-02 | 0 | 3 | 4 | 7 |
| A-OBS-01 | 2 | 2 | 2 | 6 |
| A-PROJ-01 | 1 | 3 | 2 | 6 |
| A-FE-01 | 0 | 1 | 5 | 6 |
| A-BOOT-01 | 0 | 3 | 2 | 5 |
| A-DOM-01 | 0 | 1 | 4 | 5 |
| A-INT-01 | 1 | 2 | 2 | 5 |
| A-MEM-01 | 1 | 2 | 2 | 5 |
| Q-STA-01 | 1 | 3 | 1 | 5 |
| A-SRF-03 | 0 | 1 | 4 | 5 |
| A-SUB-01 | 0 | 3 | 2 | 5 |
| A-CHAT-01 | 0 | 1 | 2 | 3 |
| A-EVO-01 | 0 | 2 | 1 | 3 |
| A-FE-02 | 0 | 1 | 3 | 4 |
| A-HITL-01 | 0 | 2 | 2 | 4 |
| A-INP-01 | 0 | 2 | 1 | 3 |
| A-PLG-01 | 0 | 2 | 2 | 4 |
| A-SRF-01 | 0 | 2 | 2 | 4 |
| A-SRF-04 | 0 | 3 | 1 | 4 |
| A-STATE-01 | 0 | 2 | 2 | 4 |
| A-TSK-01 | 0 | 2 | 2 | 4 |
| A-TOOL-01 | 0 | 1 | 2 | 3 |
| A-TSK-06 | 0 | 1 | 2 | 3 |
| Q-DEP-01 | 0 | 1 | 2 | 3 |
| A-TSK-05 | 0 | 0 | 3 | 3 |
| A-TSK-03 | 0 | 0 | 2 | 2 |
| A-TSK-04 | 0 | 0 | 2 | 2 |
| A-TSK-02 | 0 | 0 | 1 | 1 |

**No P0 findings.** No unrecoverable-system-level data-corruption issues. The
worst defects are P1: a UTF-8 panic reachable from project file scanning, two
plaintext secret-leak paths to local disk + HTTP, and one regression of the
"missing API key fails fast" contract that strands new GUI users on first
message.

Reports `A-TSK-02`, `A-TSK-03`, `A-TSK-04`, `A-TSK-05` are close to clean
(only P3 hardening items). The adapter conformance they establish is the
single biggest positive conclusion of the application review (see §4 below).

---

## 2. Stale-Finding Check

All 29 A-phase reports and both Q-phase reports were generated against the
same baseline commits (`echo-agent` `9b0e0fa`, `echo-agent-cli` `b3b2e81`).
No reviewed commit changed underneath any report. **Zero findings are marked
stale.**

The following cross-report reconciliations are recorded:

1. **A-TSK-01-P2-02 (lossy `TodoStatus` round-trip) is latent, not live.**
   A-TSK-01 flagged that the authoritative event stream encodes `TodoStatus`
   which cannot represent framework `Retrying`/`Paused`. A-TSK-03-P3-02
   resolves this: EKO's executor never produces those framework statuses on
   the executor→store path. The lossiness is a deliberate projection
   boundary. The doc string at `types.rs:917-920` overstates the guarantee
   and should be narrowed.

2. **F-OPS-01 P1-03 / P2-01 are LIVE, not latent.** F-OPS-01 Coverage
   stated the framework `JsonlRunStore` was not wired into the CLI in
   production. A-OBS-01-P2-01 disproves this: `infra.rs:374-385` attaches
   the run store on the production `create_agent_with_diagnostics` path.
   The framework's secret-leak and unbounded-growth defects must be
   re-prioritized to P1 live, not P2 latent.

3. **Q-STA-01-P1-01 and A-PROJ-01-P2-03 describe the SAME defect.** Both
   identify the byte-slice `&remaining[j..]` in
   `project/gitignore.rs:178-179` that panics on non-ASCII paths. They are
   merged under `APP-BOOT-PROJ-P1-01` below.

4. **A-SRF-04-P2-03 and A-BOOT-01-P2-02 describe the SAME defect** from
   different angles (boot-lifecycle vs trigger-side): channels-only mode
   skips `start_headless_services`. Merged under `APP-BOOT-P2-02`.

5. **`A-CHAT-01-P2-01` (sink-persistence authority) is the parent of two
   related findings:** `A-SRF-02-P2-03` (subagent bridge parallel
   recorder) and `A-OBS-01-P1-01` (unredacted content). All three point
   at the same architectural fix: extract a driver-level
   `ToolExecutionRecorder`. Merged under `APP-CHAT-P2-01`.

---

## 3. Findings by Subsystem

Findings are grouped into subsystems. Within each subsystem, items are
ordered P1 → P2 → P3. Duplicate / overlapping findings from multiple reports
are merged under a canonical `APP-<SUBSYSTEM>-<priority>-<n>` ID with
backlinks to the originals.

### 3.1 Boot / Config (composition root, services, config watcher, workspace switch)

#### APP-BOOT-P1-01 — Missing provider API key does not fail fast at bootstrap
- **Priority:** P1
- **Backlinks:** `A-CFG-01-P1-02`
- **Evidence:** `echo-agent-app-core/src/infra.rs:303-320` (LlmConfig injection
  guarded by `if let Some(auth_token)`); `model_config.rs:287-293` returns
  `auth_source: "none"` when no config + no env.
- **Defect:** Bootstrap silently succeeds with no resolvable auth; first user
  message triggers opaque 401. GUI launches are the worst case (shell env
  vars absent).
- **Direction:** Surface a typed error / setup screen when both config and env
  are empty, before agent build.

#### APP-BOOT-P1-02 — Config watcher targets are not refreshed on workspace switch
- **Priority:** P1
- **Backlinks:** `A-CFG-01-P1-01`
- **Evidence:** `config_watcher.rs:199-211` (target list built once from
  cwd-at-spawn); `state.rs:853-864` mutates process CWD but not the watcher.
- **Defect:** After switching to workspace W, edits to `W/.eko/hooks.yaml`
  never hot-reload; user must restart.
- **Direction:** Give `switch_workspace` a handle to re-register the new
  workspace's hooks file (unwatch stale, watch new).

#### APP-BOOT-P2-01 — TaskRuntimeStore is constructed twice in TUI/CLI (one discarded)
- **Priority:** P2
- **Backlinks:** `A-BOOT-01-P2-01`
- **Evidence:** `main.rs:35-57` (headless helper) + `state.rs:547-566`
  (`from_shared` initializer); `modes.rs:57-60` overwrites the field.
- **Defect:** TUI/CLI builds the store and calls `recover_incomplete` twice;
  store #1 is dropped. GUI uses only `from_shared`. Two parallel "open +
  recover" implementations can drift.
- **Direction:** Make `from_shared`'s `TaskState::runtime` the single source
  of truth. Either accept an optional pre-built store, or have headless read
  it back out of `state.tasks.runtime` after `from_shared`.

#### APP-BOOT-P2-02 — Channels-only entry skips `start_headless_services` (no scheduler, no BG service, no MCP health, no dreaming)
- **Priority:** P2
- **Backlinks:** `A-BOOT-01-P2-02`, `A-BOOT-01-P2-03` (MCP health / dreaming
  GUI-only), `A-SRF-04-P2-03`
- **Evidence:** `main.rs:357-403`; `modes.rs:32-64` is the only starter for
  headless. MCP health check + dreaming also only spawn in GUI
  (`desktop.rs:243, 247`).
- **Defect:** `echo-agent-cli --channels` as a long-running IM bot gets no
  cron fires, no background tasks, no MCP health telemetry, no daily memory
  dreaming. Cron runs also never auto-resume after restart
  (cross-reference APP-SRF-CLI-P2-02).
- **Direction:** Route channels-only branch through
  `start_headless_services` (or a shared starter) before spawning
  `run_channels_mode`. Move MCP health + dreaming spawners into the shared
  starter (or `AgentRuntime::bootstrap`) so every entry gets them.

#### APP-BOOT-P2-03 — Config watcher does not refresh `AppState.app_config` (stale snapshot)
- **Priority:** P2
- **Backlinks:** `A-CFG-01-P2-03`
- **Evidence:** `config_watcher.rs:254-277` — `new_config` is consumed only by
  `webhook_emitter.reload_from_config`; `state.rs:339,478` `app_config` is
  written once.
- **Defect:** IPC handlers reading `app_config` (channels/MCP path, token
  limit for UI) return pre-edit values until restart.
- **Direction:** Decide which `app_config` fields are safe to refresh live,
  have watcher write those into `app_config`, document restart-required
  fields explicitly.

#### APP-BOOT-P2-04 — `apply_env_overrides` not re-run by hot-reload path
- **Priority:** P2
- **Backlinks:** `A-CFG-01-P3-02` (re-classified to P2 in synthesis because
  the inconsistency becomes a latent bug the moment the reload path is
  widened under APP-BOOT-P2-03).
- **Evidence:** `config_watcher.rs:254-257` reload calls `load_config` only,
  no `apply_env_overrides`.
- **Defect:** Env-overlaid fields (channel secrets, MCP path) applied once at
  bootstrap are discarded on the next reload.

#### APP-BOOT-P2-05 — No global→project config merge (first file wholly wins)
- **Priority:** P2
- **Backlinks:** `A-CFG-01-P2-01`
- **Evidence:** `echo-agent/src/config.rs:741-753` returns on first parseable
  file; `config_discovery.rs:219-240` advertises layered model.
- **Defect:** User with global `~/.eko/config.yaml` + project `./echo-agent.yaml`
  silently loses the global half.

#### APP-BOOT-P2-06 — Filename mismatch between loader and discovery inventory
- **Priority:** P2
- **Backlinks:** `A-CFG-01-P2-02`
- **Evidence:** Loader reads `~/.eko/config.yaml`; discovery advertises
  `~/.eko/echo-agent.yaml`.
- **Defect:** User creates the advertised name → silently ignored.

#### APP-BOOT-P2-07 — Corrupt explicit config file silently falls back to defaults
- **Priority:** P2
- **Backlinks:** `A-CFG-01-P2-05`
- **Evidence:** `config.rs:726-738` returns `AppConfig::default()` on parse
  failure of explicit `--config`.
- **Defect:** Operator thinks config is active; reality is defaults. Bad for
  sandbox/permission tuning.

#### APP-BOOT-P2-08 — Workspace switch does not reload config (and does not say so)
- **Priority:** P2
- **Backlinks:** `A-CFG-01-P2-04`
- **Evidence:** `state.rs:844-1010` full `switch_workspace` body has no
  `load_config`/`app_config.write`/`apply_env_overrides`.
- **Defect:** User who places `echo-agent.yaml` in workspace B and switches
  is silently disappointed. Combined with APP-BOOT-P1-02, both config and
  hooks are stale after a switch.

#### APP-BOOT-P3-01 — CLI `--model` overrides only the name, not provider
- **Priority:** P3 · **Backlinks:** `A-CFG-01-P3-01`
- **Defect:** `--model gpt-4o` against an Anthropic default keeps Anthropic
  auth and sends `gpt-4o` as id — wrong account or rejection.

#### APP-BOOT-P3-02 — Headless builds AgentPool inline instead of `AgentRuntime::init_pool`
- **Priority:** P3 · **Backlinks:** `A-BOOT-01-P3-01`
- **Defect:** Two parallel pool-construction sequences (GUI uses helper,
  headless inline duplicates it) can drift.

#### APP-BOOT-P3-03 — Shutdown ordering differs between headless (flush → cancel) and GUI (cancel → flush)
- **Priority:** P3 · **Backlinks:** `A-BOOT-01-P3-02`
- **Defect:** GUI's cancel-before-flush could truncate the hook flush if a
  future hook depends on a token-gated task.

### 3.2 Chat / Input (drive_chat, sinks, attachments, prepared-turn)

#### APP-CHAT-P2-01 — `TauriChatSink` owns tool-execution persistence; sinks do not "only render"
- **Priority:** P2
- **Backlinks:** `A-CHAT-01-P2-01` (parent), `A-SRF-02-P2-03` (subagent
  bridge parallel recorder), `A-OBS-01-P1-01` (unredacted content),
  `A-STATE-01-P2-02` (TUI/GUI cascade parity)
- **Evidence:** `tauri/commands/chat.rs:1148-1156` TauriChatSink holds
  `Arc<ToolExecutionRepository>`; `:1193-1340` `handle_tool_event` writes via
  `start/append_output/finish/cancel`. `tauri/mod.rs:335-769` is a SECOND
  implementation for the subagent path that does not record `append_output`
  chunks. TUI/CLI/channels render only — no tool history persistence.
- **Defect:** (a) GUI-only durable tool history (multi-mode parity gap);
  (b) misplaced authority — a sink is a render/transport boundary, not a
  writer; (c) two parallel recorder implementations with drift; (d)
  repository writes are best-effort (errors `tracing::warn!` and continue).
- **Direction:** Extract tool-execution recording into a driver-level
  `ToolExecutionObserver` constructed inside `drive_chat_inner` from
  `ChatResources`. TauriChatSink becomes pure render; TUI/CLI/channels gain
  durable history by supplying the repository in their `ChatResources`. The
  subagent bridge in `mod.rs` shrinks to event-routing only.
- **Cross-reference:** the same extraction is the prerequisite for closing
  APP-FE-P3-01 (orphan SubagentRun type) and APP-SURF-FE-P3-03
  (`as unknown as` cast on execution channel).

#### APP-CHAT-P3-01 — `drive_chat_inner`'s `Err(e)` stream branch is dead code
- **Priority:** P3 · **Backlinks:** `A-CHAT-01-P3-01`
- **Evidence:** `chat_driver.rs:548-560` — `envelope_event_stream` only
  yields `Ok`, so the `Err` arm is unreachable.

#### APP-CHAT-P3-02 — No `drive_chat`-level cancel / steer / error fixtures
- **Priority:** P3 · **Backlinks:** `A-CHAT-01-P3-02`
- **Defect:** One-terminal invariant on cancel/error paths is guarded only
  by framework tests + static review, not by an app-level regression test.

#### APP-CHAT-INP-P2-01 — TUI `/steer` from input box bypasses PreparedUserTurn
- **Priority:** P2
- **Backlinks:** `A-INP-01-P2-01`
- **Evidence:** `tui/events.rs:1438-1470` `steer_active_turn` builds
  `Message::user(text.to_string())` directly; the slash-command path at
  `events.rs:4231` does go through `PreparedUserTurn::build`.
- **Defect:** Long `/steer` pastes not spilled; `pending_attachments`
  dropped; TUI/GUI divergence (Tauri `steer_turn` always uses
  `PreparedUserTurn::build`).

#### APP-CHAT-INP-P2-02 — Conversation-deletion cleanup assumes single shared artifact root
- **Priority:** P2
- **Backlinks:** `A-INP-01-P2-02`
- **Defect:** `delete_conversation` derives spill dir from live agent config
  (`agent.tool_output_artifacts()`), but write-time resolution uses
  `resolve_user_input_spill_dir(ws_root.as_deref())`. Workspace mismatch →
  silent no-op; orphaned dirs accumulate until 30-day TTL sweeps.

#### APP-CHAT-INP-P3-01 — `cleanup_expired_entries` return value semantics subtle
- **Priority:** P3 · **Backlinks:** `A-INP-01-P3-01`

### 3.3 State / Memory (persistence, instructions, hot memory, dreaming)

#### APP-STATE-MEM-P1-01 — Hot-layer (MEMORY.md) edits refresh the WRONG projection
- **Priority:** P1
- **Backlinks:** `A-MEM-01-P1-01`
- **Evidence:** `unified_memory.rs:28-29, 138-167` — two distinct markers
  (`eko:instruction-context` excludes MEMORY.md; `eko:hot-memory-context`
  carries it). All eight MEMORY.md-mutating sites call
  `refresh_instruction_projection` (wrong target) instead of
  `refresh_hot_memory_projection`. Sites: `infra.rs:1175-1192` (Dreaming),
  `memory.rs:126-145, 219-238` (GUI add/delete), `events.rs:2839-2857,
  2913-2927` (TUI /remember, /forget), `all.rs:123-138, 194-209` (CLI).
- **Defect:** Promoted hot memories never appear in the agent's stable prefix
  until next workspace switch or process restart. The headline capability of
  Dreaming (recall-driven promotion to a stable prefix) is silently broken
  on the primary surface it was built for. Mitigated by per-turn
  `TURN_MEMORY_CONTEXT_PROJECTION` recall, but that is the recall path, not
  the hot-layer injection the feature advertises.
- **Direction:** At every MEMORY.md-mutating site, call
  `refresh_memory_projections` (which refreshes both idempotently) and fix
  the pool helper to match. The learned-rules.md sites stay on
  `refresh_instruction_projection`.

#### APP-STATE-MEM-P2-01 — `AgentPool::refresh_instruction_context` doc lies (refreshes only instructions)
- **Priority:** P2
- **Backlinks:** `A-MEM-01-P2-01`
- **Evidence:** `agent_pool.rs:686-710` doc claims both projections; body
  refreshes only instruction.
- **Defect:** Even if a caller wanted to refresh hot memory on the pool, the
  helper does not expose it. Compounds APP-STATE-MEM-P1-01.

#### APP-STATE-MEM-P2-02 — CLI /remember, /forget, rule-promote refresh only the primary agent (pool diverges)
- **Priority:** P2
- **Backlinks:** `A-MEM-01-P2-02`
- **Evidence:** `all.rs:123-142, 194-214`, `evolution.rs:1471-1496` — no
  `pool.refresh_*` call. Contrast TUI/GUI which fan out.
- **Defect:** A background/pool agent continues with the stale projection
  until next workspace switch or restart.

#### APP-STATE-P2-01 — `Persistence` and `SessionSearchEngine` are dead application authorities
- **Priority:** P2
- **Backlinks:** `A-STATE-01-P2-01`
- **Evidence:** `persistence.rs:211,242,247,286,321` five public methods with
  no callers; `conversation_file.rs:74,99,107,138` only `reindex_all`
  invoked once at startup. Both constructed in `state.rs:493-506`.
- **Defect:** Misleading API surface; a new contributor will reasonably
  assume these are the storage/search authorities. Index/disk drift
  undetected. Live authority is the framework `FileConversationStore`.
- **Direction:** Delete the dead surface (recommended) or resurrect with real
  callers.

#### APP-STATE-P2-02 — TUI `/delete-session` does not clean up tool-execution artifacts (TUI/GUI parity gap)
- **Priority:** P2
- **Backlinks:** `A-STATE-01-P2-02`, cross-references APP-CHAT-P2-01
- **Evidence:** `tui/events.rs:3067-3102` calls `store.delete_conversation`,
  `cleanup_tool_output_scope`, `cleanup_user_input_scope`, but never
  `tool_executions.remove_conversation(id)`. Tauri does at
  `conversations.rs:600-608`.
- **Defect:** Orphaned tool-execution detail JSON + JSONL journals persist
  indefinitely after TUI deletion.

#### APP-STATE-P3-01 — Tauri `save_conversation` lost-update window (get-then-update-then-save not transactional)
- **Priority:** P3 · **Backlinks:** `A-STATE-01-P3-01`
- **Defect:** Two concurrent saves of the same conversation silently drop one
  batch. Low for the single-user model; latent on Tauri command retry.

#### APP-STATE-P3-02 — UI-only thinking segments / execution rounds do not reach agent runtime on restore
- **Priority:** P3 · **Backlinks:** `A-STATE-01-P3-02`
- **Defect:** `restore_conversation` cannot recover reasoning_content from
  UI-only saves; the LLM loses prior reasoning trace on reload via this path
  (the framework `RuntimeStateStore` is the preferred resume and is
  unaffected).

#### APP-STATE-MEM-P3-01 — Instruction and MEMORY.md files have no file watcher
- **Priority:** P3 · **Backlinks:** `A-MEM-01-P3-01`
- **Defect:** External edits invisible until workspace switch / Dreaming /
  restart. Compounded by APP-BOOT-P1-02 (watcher targets not refreshed on
  switch).

#### APP-STATE-MEM-P3-02 — Global vs workspace memory-store path layout is asymmetric
- **Priority:** P3 · **Backlinks:** `A-MEM-01-P3-02`
- **Defect:** Global warm store at `~/.eko/store.json`, workspace warm store
  at `<root>/.eko/memory/store.json`. Each internally consistent; surprising
  for tooling/backup recipes.

### 3.4 Tasks / Worktree (TaskRuntime, plan, claims, worktree, review)

The task subsystem is the **strongest positive finding** of the application
review: the framework/application layering is clean, the adapter is thin,
claim identity is sound, and recovery is fail-closed. The findings below are
P2/P3 hardening; no P1 in this subsystem.

#### APP-TSK-P2-01 — Single malformed line in `events.jsonl` bricks every read
- **Priority:** P2
- **Backlinks:** `A-TSK-01-P2-01`
- **Evidence:** `file_shadow.rs:362-379` `read_events` returns fatal
  `ShadowError::Decode` on any non-empty malformed line; `file_store.rs:37-55`
  `load` eagerly reads events so `get_run`/`get_plan` (which need only
  projections) also fail.
- **Defect:** A truncated tail (crash mid-append) or external edit takes the
  whole run offline; intact projections cannot be used to recover.
- **Direction:** Make `read_events` skip-and-log malformed lines (or truncate
  partial trailing line). Decouple projection reads from the event file.

#### APP-TSK-P2-02 — No cleanup cascade for the task-runtime artifact tree
- **Priority:** P2
- **Backlinks:** `A-TSK-06-P2-01`, cross-references APP-STATE-P2-02
- **Evidence:** `~/.eko/tasks/{run_id}/` and `~/.eko/runtime/{run_id}/` have
  no `delete_run`/cleanup path; the conversation-deletion cascade does not
  reach them.
- **Defect:** Multi-MB artifact trees accumulate indefinitely after
  conversation deletion; privacy angle (sensitive content persists after
  user believes they deleted).
- **Direction:** Add `TaskRuntimeStore::delete_runs_for_conversation(conv_id)`;
  wire into both Tauri `delete_conversation` and TUI `/delete-session`.

#### APP-TSK-P3-01 — `commit_eko_task_plan` uses `DefaultTaskToolPolicy`, skipping capability validation
- **Priority:** P3 · **Backlinks:** `A-TSK-01-P3-01`
- **Defect:** Planner path bypasses `validate_candidate`; a plan referencing
  unknown capabilities accepted here would be rejected by `task_create`.

#### APP-TSK-P3-02 — `recover_incomplete` is not atomic and not idempotent for Paused runs with stuck Running tasks
- **Priority:** P3 · **Backlinks:** `A-TSK-04-P3-02`, cross-references
  APP-TSK-P3-05
- **Defect:** A crash during `recover_incomplete` leaves the run Paused with
  stuck Running tasks; subsequent boots skip it (status not Running). The
  user-driven resume then soft-locks.

#### APP-TSK-P3-03 — `create_complex_task` synthetic conversation-id fallback
- **Priority:** P3 · **Backlinks:** `A-TSK-02-P3-01`

#### APP-TSK-P3-04 — `execute_run` drain loop missing run-state guard (microsecond race soft-lock)
- **Priority:** P3 · **Backlinks:** `A-TSK-03-P3-01`
- **Defect:** If the run transitions to non-Running terminal during the
  quiescent-completion window, the loop spins forever. Two-line check fix.

#### APP-TSK-P3-05 — `set_task_status` is non-claim-guarded with no `TodoStatus` monotonicity check
- **Priority:** P3 · **Backlinks:** `A-TSK-04-P3-01`
- **Defect:** Defense-in-depth gap — the invariant holds today only because
  every caller self-restricts.

#### APP-TSK-P3-06 — `worktree.rs` module doc still references removed `panels.rs`
- **Priority:** P3 · **Backlinks:** `A-TSK-05-P3-01`

#### APP-TSK-P3-07 — No executor-level test exercises cancellation during `spawn_blocking { integrate_fork_worktree }`
- **Priority:** P3 · **Backlinks:** `A-TSK-05-P3-02`

#### APP-TSK-P3-08 — `acquire_fork` reuses existing worktree path without re-running `validate_worktree_target`
- **Priority:** P3 · **Backlinks:** `A-TSK-05-P3-03` (defense-in-depth)

#### APP-TSK-P3-09 — `archive_trace` duplicates `full_output` and writes to CWD-derived path
- **Priority:** P3 · **Backlinks:** `A-TSK-06-P3-01`

#### APP-TSK-P3-10 — Artifact `metadata.retention` field is write-only
- **Priority:** P3 · **Backlinks:** `A-TSK-06-P3-02`

#### APP-TSK-P3-11 — A-TSK-01-P2-02 doc string overstates "lossless" guarantee
- **Priority:** P3 · **Backlinks:** `A-TSK-03-P3-02`
- **Defect:** `types.rs:917-920` should be narrowed: `Retrying`/`Paused` are
  never produced on the executor path; the lossiness is a deliberate
  projection boundary.

### 3.5 Surfaces (TUI / GUI / CLI / Channels parity)

This subsystem holds the largest cluster of parity gaps. AGENTS.md mandates
"TUI 与 GUI 是功能完全一样的 Agent 完全体"; the findings below are the gaps
toward that mandate.

#### APP-SURF-CLI-P1-01 — CLI `/workspace switch` does not switch state
- **Priority:** P1
- **Backlinks:** `A-PROJ-01-P1-01`
- **Evidence:** `cli/cmd_impls/workspace.rs:114-146` opens the registry and
  prints; never obtains `AppState` and never calls
  `AppState::switch_workspace`. Tauri IPC `workspace.rs:131-137` is the only
  caller.
- **Defect:** User runs `/workspace switch B` in CLI/TUI → success printed →
  nothing changes: CWD, agent working_dir, persistence, conversation store,
  memory store, project-context projection all keep pointing at bootstrap
  workspace. Combined with APP-BOOT-P1-02, the CLI workspace model is
  substantially non-functional.
- **Direction:** Thread `AppState` (or a `switch_workspace` capability) into
  the CLI workspace command; have `ws_switch` invoke the same orchestration
  as the Tauri command.

#### APP-SURF-CLI-P2-01 — Chat/Auto turns on REPL and channels have no externally reachable cancel handle
- **Priority:** P2
- **Backlinks:** `A-SRF-04-P2-01`
- **Evidence:** `repl.rs:533`, `channels.rs:244` — fresh CancellationToken
  per turn, never registered. Only Task mode registers via
  `register_run_cancellation`. Contrast TUI `app.active_cancel`.
- **Defect:** A long-running turn on a channel blocks that sender's pool
  agent until completion; the IM user have no escape hatch. Runaway agent
  loops on a channel tie up per-sender agents until bot kill.
- **Direction:** Register chat-lane cancel handle; add `/cancel` slash
  command for channels; install `tokio::signal::ctrl_c()` for REPL.

#### APP-SURF-CLI-P2-02 — Cron runs recovered to Paused on restart but never auto-resumed
- **Priority:** P2
- **Backlinks:** `A-SRF-04-P2-02`
- **Evidence:** `tasks/service.rs:541, 552, 563, 616, 910` filters on
  `background:`; a cron run's `cron:` prefix fails the filter.
  `recover_incomplete` reconciles to Paused but nothing wakes them up.
- **Defect:** Interrupted cron runs pile up Paused forever; next cron tick
  fires a NEW run duplicating work. The cron promise "runs on schedule" is
  broken across any restart.

#### APP-SURF-CLI-P3-01 — REPL slash commands (cron/tasks) unavailable on channels
- **Priority:** P3 · **Backlinks:** `A-SRF-04-P3-01`
- **Defect:** Only `/trace /analysis /papers /skills /mode` wired on
  channels. Operationally important `/cron` unavailable to channels-only
  bots.

#### APP-SURF-TUI-P2-01 — `parallel_tasks` Vec and `TaskStrip` widget are scaffolded but never populated
- **Priority:** P2
- **Backlinks:** `A-SRF-01-P2-01`
- **Evidence:** `tui/mod.rs:334, 830` field never reassigned; repo-wide grep
  zero producers.
- **Defect:** Dead UI scaffold; bottom progress strip never renders.

#### APP-SURF-TUI-P2-02 — Subagent internal lifecycle collapses to a counter; 11/16 framework SubagentEvent variants dropped
- **Priority:** P2
- **Backlinks:** `A-SRF-01-P2-02`
- **Evidence:** `events.rs:5343-5434` matches only 5 variants; catch-all
  `_ => {}` for the rest. `DispatchToolStarted` only increments counter.
- **Defect:** No live indicator of subagent activity, no token usage for
  subagent LLM calls, no thinking trace. Context-window indicator
  undercounts during subagent-heavy turns.
- **Direction:** Short-term: extend `update_subagent_runs` to handle
  `DispatchLlmUsage`, `DispatchThinkingStarted/Delta/Ended`. Long-term:
  inherit per-tool detail from the unified `ToolExecutionObserver` once
  APP-CHAT-P2-01 lands.

#### APP-SURF-TUI-P3-01 — TUI `/permission` alias set reduced relative to GUI/CLI
- **Priority:** P3 · **Backlinks:** `A-SRF-01-P3-01`, `A-SRF-02-P2-02`
- **Defect:** User types `/permission autoedit` in TUI → "Unknown";
  same alias works in GUI/CLI. Cross-surface drift.
- **Direction:** Lift canonicalization into `PermissionMode::from_alias` in
  app-core.

#### APP-SURF-TUI-P3-02 — No fixture drives TUI reducer's terminal event arms (Cancelled / Error / Interrupt)
- **Priority:** P3 · **Backlinks:** `A-SRF-01-P3-02`

#### APP-SURF-TUI-P3-03 — TUI has no interactive terminal pane (parity gap)
- **Priority:** P3 · **Backlinks:** `A-TOOL-01-P3-02`
- **Defect:** Cross-reference to A-BOOT-01 / B-PATH-01; not a fix target
  for this synthesis but tracked for parity accounting.

#### APP-SURF-GUI-P2-01 — `TerminalManager.close_all()` never called on window close (PTY shells orphaned)
- **Priority:** P2
- **Backlinks:** `A-SRF-02-P2-01`
- **Evidence:** `desktop.rs:256-268` post-`.run()` cleanup has no terminal
  cleanup; `mod.rs:69-310` registers no `on_window_event`. `close_all` at
  `terminal.rs:256-267` has zero callers.
- **Defect:** On every GUI window close, every open PTY shell is left
  running (reparented to launchd on macOS). A `npm run dev` started in an
  EKO terminal keeps serving after the app is "closed".
- **Direction:** Register `on_window_event(CloseRequested)` →
  `terminal_manager.close_all()`, or expose terminal_manager via AppState.

#### APP-SURF-GUI-P2-02 — Permission-mode alias normalization triplicated with drift
- **Priority:** P2
- **Backlinks:** `A-SRF-02-P2-02`, `A-SURF-TUI-P3-01`
- **Evidence:** Three independent match blocks: `panels.rs:43-58`,
  `coding.rs:663-667`, `events.rs:3583-3591`. TUI is reduced.
- **Direction:** Introduce `PermissionMode::from_alias(&str)` in app-core.

#### APP-SURF-GUI-P2-03 — `execution://event` channel untyped while `chat://event` is typed
- **Priority:** P2 (raised from P3 by synthesis: the asymmetry is
  load-bearing for the frontend contract)
- **Backlinks:** `A-SRF-02-P3-02`, `A-SRF-03-P3-03` (receive-side cast),
  `A-FE-01-P3-04` (no contract test)
- **Evidence:** `chat.rs:153-183` `emit_execution_event` hand-builds
  `serde_json::Map`; `mod.rs:703-752` subagent bridge hand-builds same.
- **Defect:** Frontend treats payload as `Record<string, unknown>` cast via
  `as unknown as ExecutionEvent`. Typo in field name compiles cleanly on
  both sides and silently breaks grouping.
- **Direction:** Introduce `ExecutionEvent` enum mirroring `ChatEvent`;
  refactor `emit_execution_event` to take `&ExecutionEvent`. Pair with the
  APP-CHAT-P2-01 recorder extraction.

#### APP-SURF-GUI-P3-01 — Conversation UI-projection merge algorithm lives in Tauri command module
- **Priority:** P3 · **Backlinks:** `A-SRF-02-P3-01`
- **Defect:** 140-line merge algorithm cannot be reused by TUI/CLI save
  paths.

#### APP-SURF-GUI-P3-02 — `send_chat_message` is ~290-line fat orchestration command
- **Priority:** P3 · **Backlinks:** `A-SRF-02-P3-03`

#### APP-SURF-GUI-P3-03 — `save_conversation` holds `conversation_store` read lock across multiple awaits
- **Priority:** P3 · **Backlinks:** `A-SRF-02-P3-04`

#### APP-SURF-GUI-P3-04 — `useToolExecutionStore.ingest` (live path) is direct overwrite (cross-link)
- **Priority:** P3 · **Backlinks:** `A-SRF-03-P2-01`, also APP-FE-P3-01
- **Defect:** A late `started` event clobbers a terminal. The hydrate path
  uses status-rank merge; the live path does not.
- **Direction:** Route live `ingest` through `mergeToolExecution`.

#### APP-SURF-GUI-P3-05 — `done` / `final_answer` ordering brittle (latent)
- **Priority:** P3 · **Backlinks:** `A-SRF-03-P3-01`

#### APP-SURF-GUI-P3-06 — `chatEventHandler` `cancelled` branch doesn't finalize streaming message
- **Priority:** P3 · **Backlinks:** `A-SRF-03-P3-02`

#### APP-SURF-GUI-P3-07 — `useBrowserEvents` listener-setup race unfixed
- **Priority:** P3 · **Backlinks:** `A-SRF-03-P3-04`

### 3.6 Tools / MCP / Browser / HITL / Plugins / Subagents

#### APP-TOOL-INT-P1-01 — IPC MCP URL / stdio validation rejects legitimate local servers (over-gating)
- **Priority:** P1
- **Backlinks:** `A-INT-01-P1-01`
- **Evidence:** `tauri/commands/mcp.rs:117-160` `validate_ipc_mcp_stdio`
  rejects any base-name not in `ALLOWED_MCP_STDIO_BASES`;
  `:169-208` `validate_ipc_mcp_url` rejects loopback / private-range URLs.
  The on-disk path (`config_loader.rs:229-261`) only applies a denylist.
- **Defect:** Same class of regression as the historical `require_full_auto`
  gate that AGENTS.md explicitly removed. GUI users cannot configure
  locally-served MCP servers (`http://localhost:8100/mcp`) or non-allowlisted
  binaries (`/usr/local/bin/my-custom-mcp`) through the panel; the same
  content works via `~/.eko/mcp.json`. Existing tests *lock in* the
  over-gating.
- **Direction:** Drop the executable allowlist (keep denylist + shell-meta +
  traversal guards); drop the loopback/private-range rejection (keep
  `https://` for non-localhost). Update existing tests. Aligns with AGENTS.md
  "保留对明显错误输入的轻量校验即可,不要做权限级拦截".

#### APP-TOOL-INT-P2-01 — No graceful MCP / LSP shutdown on app exit
- **Priority:** P2
- **Backlinks:** `A-INT-01-P2-01`, cross-references APP-SURF-GUI-P2-01
  (terminal cleanup parallel)
- **Evidence:** `main.rs:334, 399, 443`, `desktop.rs:267` only call
  `browser_runtime.shutdown()`. No equivalent for MCP/LSP. Framework relies
  on best-effort `Drop`.
- **Defect:** MCP stdio subprocesses killed via `Drop`-spawned tasks that may
  not run if runtime is shutting down; LSP subprocesses get `kill_on_drop`
  SIGKILL with no graceful `shutdown` request.
- **Direction:** Add `async fn shutdown(&self)` on `AgentRuntime` that
  iterates `agent.disconnect_mcp(name)` and calls
  `lsp.manager.write().await.shutdown_all()`.

#### APP-TOOL-INT-P2-02 — Framework `LspManager::restart_server` has no application caller
- **Priority:** P2
- **Backlinks:** `A-INT-01-P2-02`
- **Evidence:** `grep -rn "restart_server|restart_lsp" echo-agent-cli/`
  returns zero hits. No Tauri `lsp.rs`, no TUI `/lsp`.
- **Defect:** A crashed/hung LSP server cannot be recovered without
  restarting EKO. Compounds F-INT-02-P2-01/P2-02.
- **Direction:** Add Tauri `restart_lsp_server(language)` + TUI
  `/lsp restart <lang>`, both delegating to the framework primitive.

#### APP-TOOL-INT-P3-01 — `disconnect_mcp_server` always returns success even when nothing disconnected
- **Priority:** P3 · **Backlinks:** `A-INT-01-P3-01`

#### APP-TOOL-INT-P3-02 — Browser `interrupt()` does not cancel in-flight tool calls (silently rebuilds sidecar)
- **Priority:** P3 · **Backlinks:** `A-INT-01-P3-02`

#### APP-TOOL-HITL-P2-01 — GUI and Channels bypass `HitlDispatcher`; multi-surface shared deadline not implemented in practice
- **Priority:** P2
- **Backlinks:** `A-HITL-01-P2-01`
- **Evidence:** `chat.rs:570-582` (Tauri per-turn install),
  `channels.rs:149` (per-sender install). Dispatcher fan-out runs only on
  REPL/TUI; GUI/Channels each have own 300s timeout.
- **Defect:** Correct for local-assistant positioning, but the dispatcher
  doc-comment is misleading; a future maintainer expects unified surface
  arbitration.
- **Direction:** Document the dispatcher as REPL/TUI single-surface arbiter;
  update F-HITL-01 V01 wording.

#### APP-TOOL-HITL-P2-02 — `IpcAuth` / `require_full_auto` / `require_not_strict` is dead code with misleading doc-comment
- **Priority:** P2
- **Backlinks:** `A-HITL-01-P2-02`
- **Evidence:** `tauri/error.rs:1-10` module doc claims these gate
  process-spawning IPC commands. Grep returns zero call sites.
- **Defect:** A security auditor reading the module doc will conclude
  `full-auto` is required for `create_terminal`/`connect_mcp_server`. False.
- **Direction:** Delete the dead surface; rewrite the module doc to state
  the actual policy (input validation + per-session consent, not
  permission_mode gate).

#### APP-TOOL-HITL-P3-01 — `HitlDispatcher` parallel fan-out over-engineered for single-provider usage
- **Priority:** P3 · **Backlinks:** `A-HITL-01-P3-01`

#### APP-TOOL-HITL-P3-02 — Post-turn empty-dispatcher reset is undocumented fail-closed safety net
- **Priority:** P3 · **Backlinks:** `A-HITL-01-P3-02`

#### APP-TOOL-P2-01 — `SandboxConfigData.security_level` (Low/Medium/High) is cosmetic for `run_code`
- **Priority:** P2
- **Backlinks:** `A-TOOL-01-P2-01`
- **Evidence:** `state.rs:262` field; only write site is default initializer.
  Agent's `SandboxManager::local_sandbox()` never consults it.
- **Defect:** User raises tier to "High" believing they harden agent
  execution — no effect. Same "config that doesn't do what it says" class as
  the historical `require_full_auto`.
- **Direction:** Remove `security_level` and GUI selector, keeping numeric
  limits; or wire SandboxTier → SandboxPolicy with documented mapping.

#### APP-TOOL-P3-01 — `Auto` interaction mode exposes `shell` but not `run_code`
- **Priority:** P3 · **Backlinks:** `A-TOOL-01-P3-01` (low confidence)

#### APP-TOOL-PLG-P2-01 — Single malformed application component in any plugin aborts entire reload
- **Priority:** P2
- **Backlinks:** `A-PLG-01-P2-01`
- **Evidence:** `plugin_components.rs:217-221` returns `Err(Vec<String>)` if
  any error non-empty.
- **Defect:** One plugin's bad theme blocks reload for all other plugins.
  Asymmetric with the resilient skill loader.

#### APP-TOOL-PLG-P2-02 — Plugin component files have no filesystem-watch integration
- **Priority:** P2
- **Backlinks:** `A-PLG-01-P2-02`, cross-references APP-BOOT-P1-02 / P2-03
  (config watcher pattern)
- **Defect:** Plugin edits require manual `/plugins reload`; asymmetry with
  user hooks (hot-reloaded).

#### APP-TOOL-PLG-P3-01 — Plugin shutdown relies on `Drop` with no ordered async teardown
- **Priority:** P3 · **Backlinks:** `A-PLG-01-P3-01`

#### APP-TOOL-PLG-P3-02 — `validate()` does not apply plugin variables while `prepare()` does
- **Priority:** P3 · **Backlinks:** `A-PLG-01-P3-02`

#### APP-TOOL-SUB-P2-01 — Plugin subagents register only on the primary agent (pool agents never see them)
- **Priority:** P2
- **Backlinks:** `A-SUB-01-P2-01`
- **Evidence:** `agent_pool.rs:93-113` `SharedResources` lacks
  `SubagentRegistry`; pool's `create_agent` constructs fresh registry;
  `plugin_runtime.rs:833-867` registers plugins only on primary.
- **Defect:** In multi-conversation GUI sessions, plugin subagents silently
  unavailable in conversation B. LLM call to `agent_tool(agent_name=...)`
  rejected as unknown.
- **Direction:** Share the primary registry via `SharedResources`, or add
  `pool.apply_subagent_definitions(definitions)` mirroring
  `refresh_skill_descriptors`.

#### APP-TOOL-SUB-P2-02 — Subagent definitions have no reload mechanism
- **Priority:** P2
- **Backlinks:** `A-SUB-01-P2-02`
- **Defect:** `.md` edits require restart for primary; pool agents lazily
  re-read on `acquire`, creating primary-vs-pool divergence.

#### APP-TOOL-SUB-P2-03 — System-prompt-baked subagent catalog diverges from live registry after plugin registration
- **Priority:** P2
- **Backlinks:** `A-SUB-01-P2-03`
- **Defect:** Plugin subagents land in `agent_tool` schema enum but not in
  the `## Available Subagents` system-prompt section (frozen at bootstrap).
  Cross-surface: same plugin subagent invisible in system prompt but
  visible in `task_execute` capability catalog.

#### APP-TOOL-SUB-P3-01 — `validate_default_subagent_routes` only validates bootstrap snapshot
- **Priority:** P3 · **Backlinks:** `A-SUB-01-P3-01`

#### APP-TOOL-SUB-P3-02 — No documented precedence between plugin subagents and file-based scopes
- **Priority:** P3 · **Backlinks:** `A-SUB-01-P3-02`

### 3.7 Frontend (TypeScript / React / Tauri IPC)

#### APP-FE-P2-01 — Manual `ToolInfo` type and `ToolsPanel` field access drift from wire; tool parameters never rendered
- **Priority:** P2
- **Backlinks:** `A-FE-01-P2-01`
- **Evidence:** `types/response.rs:39-47` Rust DTO sends `parameters`,
  `need_approval`, `source: ToolSource`. Manual TS `types/api.ts:179-185`
  declares `input_schema?` and omits `need_approval`. `ToolsPanel.tsx:89,94`
  reads `tool.input_schema` which is always undefined on the wire.
- **Defect:** Documented feature (viewing tool parameter schemas in GUI)
  broken silently. The tool name/description/enable-toggle work; only the
  schema-viewing feature is dead.
- **Direction:** Switch consumers to the generated `ToolInfo` (preferred) or
  fix the manual field name. Pair with APP-FE-P3-04 contract test.

#### APP-FE-P2-02 — `MessageBubble` wide-subscribes to `chatStore.messages`; every token re-renders all 500 bubbles
- **Priority:** P2
- **Backlinks:** `A-FE-03-P2-01`
- **Evidence:** `MessageBubble.tsx:185-187` three subscriptions;
  `chatStore.ts:241-249` `appendToken` returns brand-new messages array on
  every token. `MAX_MESSAGES = 500` (`chatStore.ts:104`). `memo` at
  `:153` does not block Zustand-initiated re-renders.
- **Defect:** O(N·T) per turn. On 500-bubble conversation + 1000-token
  answer ≈ 500K bubble-function invocations.
- **Direction:** Lift `lastAssistantMessageId` / `messageIds` into
  `ChatPanel` and pass as props; switch to narrower selector.

#### APP-FE-P2-03 — `InterruptPromptDialog` is a non-interactive modal (no role, focus, Escape, backdrop click)
- **Priority:** P2
- **Backlinks:** `A-FE-03-P2-02`
- **Evidence:** `TaskRuntimePanel.tsx:850-922` — no `role="dialog"`, no
  `aria-modal`, no Escape, no autofocus, no focus trap, backdrop not
  clickable.
- **Defect:** A primary HITL gate for the complex-task flow. Screen-reader
  users get no announcement; keyboard users have no obvious path to buttons.
- **Direction:** Extract shared `Modal`/`Dialog` primitive with focus trap,
  Escape, autofocus; migrate all three modals.

#### APP-FE-P2-04 — `chatStore` ↔ `conversationStore` runtime circular import
- **Priority:** P2
- **Backlinks:** `A-FE-03-P2-03`
- **Evidence:** `chatStore.ts:3` imports conversationStore;
  `conversationStore.ts:2` imports chatStore. Both call `getState()` on the
  other.
- **Defect:** Architecturally smelly; no TDZ today but prevents independent
  testing and makes future extraction hard.

#### APP-FE-P3-01 — Vestigial Rust DTOs (`SkillInfo`/`McpServerInfo`/`McpToolInfo`/`ConversationRecord`) shadow generated TS
- **Priority:** P3 · **Backlinks:** `A-FE-01-P3-01`

#### APP-FE-P3-02 — Five generated TS files orphaned (not re-exported from `index.ts`)
- **Priority:** P3 · **Backlinks:** `A-FE-01-P3-02`
- **Defect:** `SubagentRun` orphan is structurally important — the durable
  subagent record has no frontend contract.

#### APP-FE-P3-03 — `FileEntry` / `DiffLine` TS types declare `undefined` but wire sends `null`
- **Priority:** P3 · **Backlinks:** `A-FE-01-P3-03`

#### APP-FE-P3-04 — No contract test guards manual DTO shapes against wire drift
- **Priority:** P3 · **Backlinks:** `A-FE-01-P3-04`
- **Defect:** The APP-FE-P2-01 drift went undetected; the next wire-side
  rename will regress silently.

#### APP-FE-P3-05 — `SubagentRunEventKind` includes dead `'artifact'` variant
- **Priority:** P3 · **Backlinks:** `A-FE-01-P3-05`

#### APP-FE-P3-06 — `PlanTask.execution_checks` / `acceptance_criteria` never displayed or editable
- **Priority:** P3 · **Backlinks:** `A-FE-02-P3-01`

#### APP-FE-P3-07 — `SubagentResultView` flattens verification `source` (observed/reported) with no semantics
- **Priority:** P3 · **Backlinks:** `A-FE-02-P3-02`

#### APP-FE-P3-08 — Tool-execution live-ingest overwrite test gap
- **Priority:** P3 · **Backlinks:** `A-FE-02-P3-03`

#### APP-FE-P3-09 — Reviewer verdict (`ReviewResult`) never fetched or rendered
- **Priority:** P3 (raised from P2 by synthesis because the parent
  capability is multi-mode parity)
- **Backlinks:** `A-FE-02-P2-01`
- **Defect:** `listReviews` endpoint defined (`endpoints.ts:559-562`) and
  never called. Backend command registered and implemented. A user staring
  at a `blocked` task sees only "评审未通过" — no review issues, no severity.

#### APP-FE-P3-10 — `TasksPanel` SSE re-subscribes on every tasks update (close-skip defect)
- **Priority:** P3 · **Backlinks:** `A-FE-03-P3-01`
- **Defect:** SSE effectively dead — opens connection, closes on next
  render, never reopens (closed EventSource remains truthy in ref). Polling
  fallback masks the defect.

#### APP-FE-P3-11 — Module-scope timers/listeners in `chatStore`/`authStore` not HMR-disposed (dev leak)
- **Priority:** P3 · **Backlinks:** `A-FE-03-P3-02`

#### APP-FE-P3-12 — `subagentRunStore` reaches into `components/compress/` for utility (layering inversion)
- **Priority:** P3 · **Backlinks:** `A-FE-03-P3-03`

#### APP-FE-P3-13 — ESLint actually runs nowhere — `eslint.config.js` is dead config; `eslint-plugin-jsx-a11y` not configured
- **Priority:** P3 · **Backlinks:** `A-FE-03-P3-04`
- **Defect:** Future stale-closure bugs and a11y regressions won't be
  caught. The a11y gaps in APP-FE-P2-03 persisted because no lint enforces.

#### APP-FE-P3-14 — Primary chat `<textarea>` has no accessibility label
- **Priority:** P3 · **Backlinks:** `A-FE-03-P3-05`

#### APP-FE-P3-15 — `AppLayout` mobile sidebar overlay not keyboard-dismissable
- **Priority:** P3 · **Backlinks:** `A-FE-03-P3-06`

### 3.8 Evolution (skills, plugins, hooks, memory auto-write, dreaming cadence)

#### APP-EVO-P2-01 — `auto_memory::run_auto_memory_extraction` is dead public function
- **Priority:** P2
- **Backlinks:** `A-EVO-01-P2-01`
- **Evidence:** `auto_memory/mod.rs:17-24` `pub fn` with zero callers.
- **Defect:** If anyone discovers and calls it from a workspace-switched
  session, it writes to the wrong workspace's inbox.

#### APP-EVO-P2-02 — Pre-compaction flush is only Review-Inbox-bypassing auto-write that runs in EKO
- **Priority:** P2
- **Backlinks:** `A-EVO-01-P2-02`, inherits `F-EVO-01-P2-01`
- **Evidence:** Framework `context.rs:676-798` calls
  `MemoryLayerManager::write_memory` directly per extracted fact; EKO
  installs layer manager on every agent.
- **Defect:** Warm-layer typed-memory writes happen without per-write user
  action. Bounded (warm only, security-scanned, dedup'd, no auto-promote)
  but violates the strict reading of the AGENTS.md anchor.
- **Direction:** Do NOT implement here — pick a F-EVO-01-P2-01 direction
  (document boundary / route through sink / opt-in-out config).

#### APP-EVO-P3-01 — Doc still calls the rule file `AGENTS.md`, but writes `learned-rules.md`
- **Priority:** P3 · **Backlinks:** `A-EVO-01-P3-01`
- **Defect:** CLI/GUI strings say "AGENTS.md"; reality is `learned-rules.md`.

### 3.9 Output (formats, export, file delivery)

#### APP-OUT-P2-01 — `output::OutputFormat` / `FormatContext` / `format_response` are dead; `--output`/`-o` flag never existed
- **Priority:** P2
- **Backlinks:** `A-OUT-01-P2-01`
- **Evidence:** `output/format.rs:8` docstring claims the flag;
  `args.rs:12-68` no such flag. `output/mod.rs:9` `#![allow(dead_code)]`.

#### APP-OUT-P2-02 — `LatexExporter` / `tasks::ResearchOutputFormat::Latex` / `Profile.output_format` all dead
- **Priority:** P2
- **Backlinks:** `A-OUT-01-P2-02`
- **Evidence:** `export/latex.rs` zero production callers;
  `ResearchOutputFormat::Latex` never constructed; `Profile.output_format`
  zero read sites.

#### APP-OUT-P2-03 — Live `export_conversation` Markdown drops tool calls/results/attachments/reasoning
- **Priority:** P2
- **Backlinks:** `A-OUT-01-P2-03`
- **Evidence:** `tauri/commands/conversations.rs:663-670` emits only `role`
  + `content`. Tool-heavy conversations produce a narrative summary, not a
  record.
- **Defect:** Asymmetric with the dead `Persistence::export_conversation_markdown`
  which was richer. A user who exports to archive or share loses provenance.

#### APP-OUT-P3-01 — `parse_export_format` duplicated in `research_tool.rs` and `tauri/commands/research.rs`
- **Priority:** P3 · **Backlinks:** `A-OUT-01-P3-01`

#### APP-OUT-P3-02 — `ReviewExportArtifact` lacks content hash (inconsistent with `ToolOutputArtifactRef` / `AnalysisOutputArtifact`)
- **Priority:** P3 · **Backlinks:** `A-OUT-01-P3-02`

#### APP-OUT-P3-03 — `print_tool_result` mixes char-count and byte-length checks (UTF-8 inconsistency)
- **Priority:** P3 · **Backlinks:** `A-OUT-01-P3-03`

#### APP-OUT-P3-04 — `research::atomic_write` omits parent-directory fsync (recurring atomic-write defect, 4th instance)
- **Priority:** P3 · **Backlinks:** `A-OUT-01-P3-04`
- **Defect:** Same class as F-MEM-01 P2-01/P2-02 and A-STATE-01 V02-01.
  Application's 4th copy of the recipe.

### 3.10 Observability (diagnostics, webhooks, run store)

#### APP-OBS-P1-01 — `ToolExecutionRepository` persists tool args / output / failure in plaintext
- **Priority:** P1
- **Backlinks:** `A-OBS-01-P1-01`, cross-references APP-CHAT-P2-01
- **Evidence:** `tool_execution.rs:217, 226, 232, 260-291, 293-358` writes
  raw args_full, output chunks, failure struct. `TauriChatSink.handle_tool_event`
  calls start/append/finish on every tool event before rendering.
- **Defect:** Tool args potentially `{"api_key": "sk-..."}`, output potentially
  `.env` content or stack trace with secrets — written verbatim to
  `~/.echo-agent/tool-executions/.../manifest.json` and `.jsonl`. Framework
  redacts in `RunEvent::ToolCall.args`; the application parallel persistence
  does not.
- **Direction:** Apply `echo_agent::security::redact_secrets` to
  args_preview/args_full, output chunks, failure fields before writing.

#### APP-OBS-P1-02 — `WebhookTurnObserver` ships raw tool args and error messages to external HTTP endpoints
- **Priority:** P1
- **Backlinks:** `A-OBS-01-P1-02`
- **Evidence:** `chat_driver.rs:124` (`args.to_string().chars().take(240)`),
  `:135-139` emit ToolCalled, `:148-152` emit ToolFailed (raw error),
  `:154-157, :521-525, :554-558` emit AgentError (raw). `webhook/emitter.rs:180-209`
  POSTs body to user-configured URL.
- **Defect:** As soon as a webhook endpoint is configured, args and error
  messages leave the machine in plaintext JSON over HTTP (URL scheme not
  validated; HMAC protects integrity not confidentiality).
- **Direction:** Run `redact_secrets` over args_summary and error/message
  clones before constructing each WebhookEvent. Optionally validate URL is
  `https://` or `http://localhost` at config-load.

#### APP-OBS-P1-03 — Framework RunStore IS wired in production (F-OPS-01 P1-03/P2-01 are LIVE)
- **Priority:** P1 (re-classification)
- **Backlinks:** `A-OBS-01-P2-01`
- **Evidence:** `infra.rs:374-385` attaches `JsonlRunStore` on
  `create_agent_with_diagnostics`; `runtime.rs:103` bootstrap calls it;
  `main.rs:168` + `desktop.rs:160` reach bootstrap; `agent_pool.rs:895-897`
  re-injects on pooled agents.
- **Defect:** Every EKO user with default config has `~/.echo-agent/runs/`
  growing without bound on every chat turn, with plaintext prompts and tool
  output (including any secrets the user pasted).
- **Direction:** This finding is a re-prioritization, not a fix. The fixes
  remain F-OPS-01's: apply `redact_secrets` at `JsonlRunStore::save`, add
  `max_file_bytes`/`max_runs` eviction.

#### APP-OBS-P2-01 — Webhook coverage has misleading-success gaps on cron failure and entire background TaskRun lifecycle
- **Priority:** P2
- **Backlinks:** `A-OBS-01-P2-02`
- **Evidence:** `scheduler/runner.rs:111-127` Err arm emits nothing; zero
  `WebhookEvent::` references in `tasks/task_runtime/*`. No `ChatCancelled`
  variant.
- **Defect:** Operator monitoring "is my agent working" sees
  `chat_completed`/`cron_task_completed` but never cron failures, background
  TaskRun lifecycle, or chat cancellations.

#### APP-OBS-P3-01 — Webhook module has zero unit tests
- **Priority:** P3 · **Backlinks:** `A-OBS-01-P3-01`

#### APP-OBS-P3-02 — `aggregate_status` priority orders Running above Failed
- **Priority:** P3 · **Backlinks:** `A-OBS-01-P3-02`

### 3.11 Cross-Cutting (Q-STA-01 + Q-DEP-01, framework touchpoints)

#### APP-CROSS-P1-01 — `gitignore::globstar_match` byte-slices `&str` mid-UTF-8 (latent panic)
- **Priority:** P1
- **Backlinks:** `A-PROJ-01-P2-03`, `Q-STA-01-P1-01` (same defect)
- **Evidence:** `project/gitignore.rs:178-181` `for j in 0..=remaining.len()
  { let candidate = &remaining[j..]; … }`. `simple_glob` at `:125-156` is
  byte-indexed and produces wrong matches across multibyte chars.
- **Reachability:** Latent today — `should_ignore_path` (the only caller)
  has zero callers outside `project/` module. Live the moment any caller
  (file-browser filter, tree-summary pruner, workspace link) consults it.
  Empirically reproduced: `globstar_match("z**", "中文模块")` panics.
- **Direction:** Rewrite `simple_glob`/`globstar_match` over `Vec<char>`.
  Add Chinese-path `**` regression test.

#### APP-CROSS-P2-01 — ~50 production `#[allow(dead_code)]` annotations in framework
- **Priority:** P2
- **Backlinks:** `Q-STA-01-P2-01`
- **Defect:** Maintenance burden; misleading fields. (Framework-side; flagged
  here for cross-cutting visibility.)

#### APP-CROSS-P2-02 — 25 source files exceed 1000 lines; 2 exceed 5000
- **Priority:** P2
- **Backlinks:** `Q-STA-01-P2-02`
- **Defect:** Top offenders: `executor.rs` 6272, `tui/events.rs` 5746,
  `data.rs` 3751, `subagent/executor.rs` 3672, `task_runtime/store.rs`
  3496. Maintainability/review difficulty.

#### APP-CROSS-P2-03 — Duplicate crate versions (38 framework / 76 CLI; high-impact: hashbrown x5, rand x3, thiserror/syn/reqwest x2)
- **Priority:** P2
- **Backlinks:** `Q-STA-01-P2-03`, `Q-DEP-01-P2-01`

#### APP-CROSS-P2-04 — `hashbrown` resolves to 5 major versions
- **Priority:** P2 · **Backlinks:** `Q-DEP-01-P2-01` (duplicate of
  APP-CROSS-P2-03, listed for backlink clarity)

#### APP-CROSS-P3-01 — No clippy guard for numeric `as` casts
- **Priority:** P3 · **Backlinks:** `Q-STA-01-P3-01`

#### APP-CROSS-P3-02 — `quick-xml` 4-5 versions across lockfiles
- **Priority:** P3 · **Backlinks:** `Q-DEP-01-P3-01`

#### APP-CROSS-P3-03 — `@tailwindcss/vite` declared in both `dependencies` and `devDependencies`
- **Priority:** P3 · **Backlinks:** `Q-DEP-01-P3-02`

### 3.12 Domain (data analysis, research, evidence)

#### APP-DOM-P2-01 — `AutoIngestResearchTool` swallows ingestion failures; agent sees successful search while sources fail to persist
- **Priority:** P2
- **Backlinks:** `A-DOM-01-P2-01`
- **Evidence:** `research_connectors.rs:306-337` — on `Err(error)` logs
  `warn!` and returns original successful search `ToolResult` unchanged.
  Records that fail normalization dropped without even a log line.
- **Defect:** In batch literature-review workflows the agent will not revisit
  failed ingestions; user discovers missing sources only by manual diff.

#### APP-DOM-P3-01 — `enrich_from_europe_pmc` stamps `enriched_at = now()` even when all sub-requests fail
- **Priority:** P3 · **Backlinks:** `A-DOM-01-P3-01`

#### APP-DOM-P3-02 — `export_review` silently drops missing sources while `audit_review` flags them in same artifact
- **Priority:** P3 · **Backlinks:** `A-DOM-01-P3-02`

#### APP-DOM-P3-03 — BibTeX export produces duplicate citation keys + no special-char escaping
- **Priority:** P3 · **Backlinks:** `A-DOM-01-P3-03`

#### APP-DOM-P3-04 — `find_matching_source` rescans entire library per `ingest_source` call (O(N·M) reads)
- **Priority:** P3 · **Backlinks:** `A-DOM-01-P3-04`

### 3.13 Project (index, diff, coding loop, workspace state)

#### APP-PROJ-P2-01 — `ProjectIndex` is 488 lines of dead code with designed-but-unused cache
- **Priority:** P2
- **Backlinks:** `A-PROJ-01-P2-01`, cross-references APP-CROSS dead-code
- **Evidence:** Whole-repo grep returns matches only inside `project/index.rs`
  (struct + 2 `#[cfg(test)]` call sites).

#### APP-PROJ-P2-02 — `FileChangeTracker` / `CodingLoop` change tracking is empty second "what changed" authority
- **Priority:** P2
- **Backlinks:** `A-PROJ-01-P2-02`
- **Evidence:** Grep for `record_file_write` etc. returns zero matches
  outside `project/`. `/review` fallback branch in non-git projects always
  returns "No file changes".

#### APP-PROJ-P3-01 — `ProjectIndex` module doc advertises cache that never exists
- **Priority:** P3 · **Backlinks:** `A-PROJ-01-P3-01` (folded into
  APP-PROJ-P2-01 deletion)

#### APP-PROJ-P3-02 — Workspace registry index/manifest writes non-atomic
- **Priority:** P3 · **Backlinks:** `A-PROJ-01-P3-02`

---

## 4. Cross-Cutting Patterns

These are the recurring failure shapes the synthesis highlights for
prioritized remediation.

### 4.1 Parity gaps (TUI / GUI / CLI / Channels)

The AGENTS.md mandate "TUI 与 GUI 是功能完全一样的 Agent 完全体" is the most
violated invariant in the application layer. Concrete gaps:

| Gap | Surfaces affected | Backlinks |
|---|---|---|
| Channels-only mode skips scheduler/BG service/MCP-health/dreaming | channels | APP-BOOT-P2-02 |
| Cron runs never auto-resume after restart | cron / channels | APP-SURF-CLI-P2-02 |
| Chat/Auto turns have no externally reachable cancel on REPL/channels | REPL / channels | APP-SURF-CLI-P2-01 |
| TUI `parallel_tasks` scaffold dead | TUI | APP-SURF-TUI-P2-01 |
| TUI subagent detail collapses to counter (11/16 events dropped) | TUI | APP-SURF-TUI-P2-02 |
| TUI `/permission` alias set reduced | TUI | APP-SURF-TUI-P3-01 / APP-SURF-GUI-P2-02 |
| TUI has no interactive terminal pane | TUI | APP-SURF-TUI-P3-03 |
| GUI-only durable tool-execution persistence | GUI vs TUI/CLI/channels | APP-CHAT-P2-01 |
| TUI `/delete-session` cascade incomplete (no tool_executions removal) | TUI vs GUI | APP-STATE-P2-02 |
| CLI `/workspace switch` is a no-op for live state | CLI vs GUI | APP-SURF-CLI-P1-01 |
| CLI /remember no pool fan-out | CLI vs TUI/GUI | APP-STATE-MEM-P2-02 |
| Plugin subagents only on primary agent | GUI multi-conversation | APP-TOOL-SUB-P2-01 |
| LSP restart available in framework, no app surface | all | APP-TOOL-INT-P2-02 |
| REPL slash commands largely unavailable on channels | channels | APP-SURF-CLI-P3-01 |
| Hot-memory refresh broken on every surface except workspace switch | all | APP-STATE-MEM-P1-01 |

### 4.2 Dead code / over-engineered authorities

Application layer carries substantial dead surface that misleads readers:

| Dead surface | Lines | Backlinks |
|---|---|---|
| `ProjectIndex` (488 lines incl. unused cache) | 488 | APP-PROJ-P2-01 |
| `FileChangeTracker` / `CodingLoop.record_file_*` | ~150 | APP-PROJ-P2-02 |
| `Persistence::{5 methods}` + `SessionSearchEngine` | ~600 | APP-STATE-P2-01 |
| `output::OutputFormat` / `FormatContext` / `format_response` | ~140 | APP-OUT-P2-01 |
| `export/latex.rs` `LatexExporter` + `ResearchOutputFormat::Latex` + `Profile.output_format` | ~160 | APP-OUT-P2-02 |
| `auto_memory::run_auto_memory_extraction` | ~10 | APP-EVO-P2-01 |
| `IpcAuth` / `require_full_auto` / `require_not_strict` | ~30 | APP-TOOL-HITL-P2-02 |
| `SandboxConfigData.security_level` (Low/Medium/High) | field | APP-TOOL-P2-01 |
| `parallel_tasks` Vec + `TaskStrip` widget | ~120 | APP-SURF-TUI-P2-01 |
| TUI `TaskProgressEntry` / `TaskStripStatus` types | ~30 | APP-SURF-TUI-P2-01 |
| Artifact `metadata.retention` field | field | APP-TSK-P3-10 |
| `SubagentRunEventKind` `'artifact'` variant | line | APP-FE-P3-05 |
| Generated TS `AttachmentSource` / `AttendedMode` / `SubagentRun` / `SubagentRunUsage` / `UnattendedWriteMode` (orphans) | 5 files | APP-FE-P3-02 |
| `eslint.config.js` (no ESLint installed) | file | APP-FE-P3-13 |
| Framework `#[allow(dead_code)]` (~50 sites) | ~50 sites | APP-CROSS-P2-01 |

### 4.3 Refresh-wiring bugs

A recurring pattern: a mutation lands but the corresponding projection /
propagation is wired to the wrong target or missing entirely.

- **APP-STATE-MEM-P1-01** — All 8 MEMORY.md-mutating sites refresh the wrong
  projection (instruction instead of hot-memory).
- **APP-STATE-MEM-P2-01** — Pool helper doc claims both, body refreshes only
  one.
- **APP-STATE-MEM-P2-02** — CLI /remember forgets pool fan-out.
- **APP-BOOT-P1-02** — Config watcher targets not refreshed on workspace
  switch.
- **APP-BOOT-P2-03** — Hot-reload doesn't refresh `AppState.app_config`.
- **APP-BOOT-P2-04** — `apply_env_overrides` not re-run by hot-reload.
- **APP-TOOL-SUB-P2-02** — Subagent definitions have no reload mechanism.
- **APP-TOOL-PLG-P2-02** — Plugin component files have no fs-watch.

### 4.4 Secret leakage

The application layer persists/emits raw content at three boundaries; none
invoke `redact_secrets`. The framework redactor is `pub`, UTF-8-safe, ~18
patterns.

- **APP-OBS-P1-01** — `ToolExecutionRepository` plaintext on disk.
- **APP-OBS-P1-02** — `WebhookTurnObserver` raw over HTTP.
- **APP-OBS-P1-03** — Framework RunStore IS wired → F-OPS-01 P1-03/P2-01
  become LIVE P1 defects (prompts + tool output on disk in plaintext).

### 4.5 Adapter conformance (positive conclusion)

The single biggest positive of the application review: **the task subsystem
adapter is thin and faithful.** A-TSK-01 / A-TSK-02 / A-TSK-03 / A-TSK-04 /
A-TSK-05 / A-TSK-06 all corroborate:

- **One framework `RuntimeDagExecutor` construction** (`executor.rs:1645`);
  one `RuntimeDagController` impl (`EkoRuntimeDagController`); one production
  `execute_run` entry with five callers all funneling through
  `execute_runtime_plan`. No second ready-frontier / wave / claim authority
  / retry loop / stall detector / DAG validator in EKO. (A-TSK-03 V01/V02)
- **One `RevisionedTaskStore` impl** (`EkoRevisionedTaskStore`); one
  `TaskToolPolicy` impl (`EkoTaskToolPolicy`); framework
  `task_create/update/list` replaced in-place by name. Zero parallel
  `todo_write`/`plan_create`/`plan_patch`/`plan_execute`. (A-TSK-02 V01/V04)
- **Single file authority** (`events.jsonl` + deterministic `plan.json` /
  `run-state.json` projections, no SQL). (A-TSK-01 V01)
- **Single writer-isolation authority** (`planner::FileOwnership` +
  `select_ownership_safe_wave` + `file_write_locks` +
  `repo_merge_lock` + `integrate_fork_worktree`). Framework's broken
  worktree tools (`F-EXT-02`) are listed in `UNATTENDED_DIRECT_MUTATION_TOOLS`
  precisely to disable them; EKO routes around them at the application
  layer. (A-TSK-05 V01)
- **Two-gate completion assessment sound** (`assess_task_execution` reads
  only execution_checks + required_artifacts; `review_task` reads only
  acceptance_criteria). (A-TSK-06 V02)
- **Claim identity deterministic and attempt-scoped**
  (`{run_id}:{task_id}:{revision}:{attempt}`); stale claims rejected before
  any event append. (A-TSK-04 V01/V02)
- **Crash recovery fail-closed** for mutating in-doubt work; durable-result
  reuse keyed on exact execution_id. (A-TSK-04 V04)

### 4.6 Atomic-write recipe duplicated 6+ times

The framework's canonical recipe (uuid-tmp + fsync + rename + parent-dir
fsync + temp cleanup) lives in `FileConversationStore::atomic_write`. The
application has at least four parallel copies, three of which omit
parent-dir fsync:

- `research::atomic_write` (`research.rs:1988-1999`) — missing parent-dir
  fsync, no temp cleanup on error (APP-OUT-P3-04)
- `Persistence::write_json` (dead, but cited in A-STATE-01 V02-01)
- `task_runtime/file_shadow::atomic_write` (`file_shadow.rs:405-422`) —
  correct (unique-tmp + fsync + rename); the one good application copy
- `workspace/registry.rs` `RegistryIndex::save` (`registry.rs:43-47`) —
  uses raw `fs::write`, not atomic at all (APP-PROJ-P3-02)
- `PluginRuntimeService` atomic preference persistence (correct,
  tmp+rename+0o600)

Direction: extract one shared `echo_agent::fs::atomic_write` helper (or
publish the framework's) and migrate the application copies.

### 4.7 Sink-vs-observer layering

The `WebhookTurnObserver` is the canonical correct pattern for cross-cutting
per-turn work: it lives INSIDE `drive_chat_inner`, runs for every sink, and
holds no per-surface state. Three other cross-cutting concerns violate this
pattern:

- **`TauriChatSink` owns tool-execution persistence** (APP-CHAT-P2-01) —
  should be a driver-level observer.
- **Subagent bridge in `tauri/mod.rs` is a SECOND persistence authority**
  (APP-CHAT-P2-01 / A-SRF-02-P2-03) — should call the same observer.
- **Per-surface permission-mode canonicalization** (APP-SURF-GUI-P2-02) —
  should be one app-core helper.

---

## 5. Contradictions Resolved

| Contradiction | Resolution |
|---|---|
| A-TSK-01-P2-02 says event stream is lossy for `Retrying`/`Paused`; types.rs:917-920 says "lossless" | **A-TSK-03-P3-02 resolves:** EKO executor never produces those framework statuses on the executor→store path. Lossiness is a deliberate projection boundary. The doc string must be narrowed (APP-TSK-P3-11). |
| F-OPS-01 says RunStore not wired in CLI ("latent hazard"); A-OBS-01 says it IS wired | **A-OBS-01-P2-01 wins:** `infra.rs:374-385` attaches JsonlRunStore on the production path. F-OPS-01 P1-03/P2-01 are LIVE P1 defects (APP-OBS-P1-03). |
| A-SRF-02-P3-02 (emit-side) and A-SRF-03-P3-03 (receive-side) both flag `execution://event` channel | **Same defect, two ends.** Merged under APP-SURF-GUI-P2-03. Fix requires both the `ExecutionEvent` enum and removing the `as unknown as` casts. |
| Q-STA-01-P1-01 and A-PROJ-01-P2-03 describe `gitignore::globstar_match` byte-slicing | **Same defect, two reports.** Merged under APP-CROSS-P1-01. |
| A-BOOT-01-P2-02 and A-SRF-04-P2-03 describe channels-only missing `start_headless_services` | **Same defect, two angles.** Merged under APP-BOOT-P2-02. |
| A-CHAT-01-P2-01 vs A-SRF-02-P2-03 (which authority owns tool-execution persistence?) | **Two parallel implementations** (TauriChatSink + subagent bridge). Merged under APP-CHAT-P2-01; the unified `ToolExecutionObserver` extraction subsumes both. |
| A-STATE-01 says `Persistence`/`SessionSearchEngine` are dead; A-OUT-01 says `export_conversation` is lossy and `Persistence::export_conversation_markdown` was richer | **No contradiction** — the dead code WAS richer; the live code is lossy. Recommendation: fix the live exporter before deleting the dead one (or do both in one PR). |

---

## 6. Prioritized Action List

Ordered by leverage (impact ÷ effort), then priority. Sequencing
recommendations: tackle the secret-leak cluster first (cheap, high-impact),
then the parity gaps that block real product usage, then the dead-code
sweep (cheap, high-maintenance-burden reduction).

### Tier 0 — Critical correctness / safety (do first)

1. **APP-OBS-P1-01 / APP-OBS-P1-02 / APP-OBS-P1-03** — Apply
   `redact_secrets` at the three persistence/emission boundaries
   (`tool_execution.rs`, `chat_driver.rs::WebhookTurnObserver`,
   `JsonlRunStore::save`). Single shared helper, ~10 call sites. Eliminates
   the entire secret-leak class.
2. **APP-CROSS-P1-01 / APP-PROJ-P2-03 (same defect)** — Rewrite
   `gitignore::simple_glob`/`globstar_match` over `Vec<char>`. Two-function
   fix, one regression test. Prevents the latent panic from going live.
3. **APP-STATE-MEM-P1-01** — Replace 8 wrong-target
  `refresh_instruction_projection` calls with `refresh_memory_projections`.
   Mechanical, well-scoped. Restores the headline Dreaming/`/remember`
   capability.
4. **APP-BOOT-P1-02** — Wire `switch_workspace` to re-register config
   watcher targets. One method extension + state threading.
5. **APP-SURF-CLI-P1-01** — Wire CLI `/workspace switch` through
   `AppState::switch_workspace`. Required for the CLI workspace model to
   function at all.
6. **APP-BOOT-P1-01** — Add fast-fail gate for missing API key in
   `create_agent_with_diagnostics` / `bootstrap`. Improves first-run GUI
   experience materially.

### Tier 1 — Parity gaps blocking real product use

7. **APP-BOOT-P2-02 / APP-SURF-CLI-P2-02 / APP-SURF-CLI-P2-01** —
   Channels-only mode cluster: route through `start_headless_services`,
   extend `resume_pending` to cron runs, register chat-lane cancel handle +
   `/cancel` slash command.
8. **APP-CHAT-P2-01** — Extract `ToolExecutionObserver` to driver level;
   unify TauriChatSink + subagent bridge. Unblocks TUI/CLI/channels durable
   tool history (multi-mode parity) and the receive-side `ExecutionEvent`
   enum (APP-SURF-GUI-P2-03).
9. **APP-TOOL-INT-P1-01** — Align IPC MCP validators with on-disk path
   (drop allowlist + private-range rejection). Update locked-in tests.
   Restores GUI MCP panel usability for local servers.
10. **APP-TOOL-SUB-P2-01 / APP-TOOL-SUB-P2-02** — Pool/subagent refresh
    parity. Either share registry via `SharedResources` or add
    `pool.apply_subagent_definitions`.
11. **APP-STATE-P2-02 / APP-TSK-P2-02** — Unify conversation-deletion
    cascade across Tauri/TUI and extend to `~/.eko/tasks/`. Single helper
    called from both surfaces.

### Tier 2 — High-frequency UX papercuts + significant dead code

12. **APP-FE-P2-01** — Switch `ToolsPanel` to generated `ToolInfo` (or fix
    manual field name). Pair with APP-FE-P3-04 contract test.
13. **APP-FE-P2-02** — Lift `lastAssistantMessageId`/`messageIds` out of
    `MessageBubble` into `ChatPanel`. Single highest-impact perf fix.
14. **APP-FE-P2-03** — Shared `Modal` primitive with focus trap/Escape;
    migrate three modals. Pair with APP-FE-P3-13 (install ESLint +
    jsx-a11y).
15. **APP-SURF-GUI-P2-01** — Wire `on_window_event(CloseRequested)` →
    `terminal_manager.close_all()`. Pair with APP-TOOL-INT-P2-01 (graceful
    MCP/LSP shutdown).
16. **APP-SURF-GUI-P2-02** — Lift permission-mode canonicalization into
    `PermissionMode::from_alias` in app-core.
17. **APP-OUT-P2-03** — Extend `export_conversation` to emit
    tool_calls/tool_results/attachments/reasoning. Then delete dead
    `Persistence::export_conversation_markdown` (APP-STATE-P2-01).
18. **APP-PROJ-P2-01 / APP-PROJ-P2-02 / APP-OUT-P2-01 / APP-OUT-P2-02 /
    APP-EVO-P2-01 / APP-TOOL-HITL-P2-02 / APP-TOOL-P2-01** — Dead-code
    sweep. Each is small and self-contained. Most can ride along with
    nearby changes per AGENTS.md "随手清理".
19. **APP-TSK-P2-01** — Make `read_events` skip-malformed and decouple
    projection reads from event file. Localized to two functions.
20. **APP-TOOL-INT-P2-02** — Add LSP restart surface (Tauri command + TUI
    slash command). Pairs with F-INT-02 framework fixes.

### Tier 3 — Test gaps, a11y, latent edge cases, doc drift

21. **APP-FE-P3-04** — Add contract tests (ts-rs expansion or paired
    Rust-serialize / TS-consume fixtures) for every manual DTO. Prevents
    the next APP-FE-P2-01-class drift.
22. **APP-FE-P3-13** — Install ESLint + plugins, wire into `npm test`/CI.
    Catches future a11y and exhaustive-deps regressions.
23. **APP-TSK-P3-04 / APP-TSK-P3-02 / APP-TSK-P3-07** — Task-runtime
    robustness: drain-loop guard, crash-during-recovery idempotency,
    mid-merge cancel test.
24. **APP-SURF-TUI-P2-02 / APP-SURF-TUI-P2-01** — TUI subagent detail
    extension and `parallel_tasks` resolution (populate or delete).
25. **APP-FE-P3-09** — Call `taskRuntimeApi.listReviews` in
    TaskRuntimePanel for blocked tasks; render `ReviewResult` issues.
26. **APP-OBS-P2-01** — Add `CronTaskFailed` / `TaskRunCompleted` /
    `TaskRunFailed` webhook variants.
27. **All remaining P3 doc/naming/hygiene items** — defer to opportunistic
    "ride-along" cleanup per AGENTS.md "随手清理是强制要求".

### Tier 4 — Cross-cutting quality (framework coordination)

28. **APP-CROSS-P2-03 / APP-CROSS-P2-04 / APP-CROSS-P3-02** — Dependency
    convergence (`hashbrown`, `quick-xml`, etc.). Coordinate with framework
    task list; `cargo tree -d` before/after.
29. **APP-CROSS-P2-02** — Split 5000+ line `executor.rs` and `events.rs`
    along sub-responsibility boundaries. Behavior-preserving.
30. **APP-CROSS-P2-01** — Incremental `#[allow(dead_code)]` cleanup in
    framework per AGENTS.md "随手清理".

---

## 7. Coverage And Uncertainty

### 7.1 Scope of this synthesis

- **29 A-phase task reports** (`A-BOOT-01` through `A-TSK-06`) — all
  read in full.
- **2 Q-phase reports** (`Q-STA-01`, `Q-DEP-01`) — read in full;
  application-relevant findings promoted into the synthesis, framework-only
  findings cross-referenced.
- **Cross-cutting B/F reports** referenced via backlinks where load-bearing
  (B-PATH-01, B-REF-01, B-BASE-01, F-OPS-01, F-EXT-02, F-INT-01/02,
  F-SUB-01/02, F-TSK-01/02/03, F-EVO-01, F-HITL-01, F-CMP-01, F-MEM-01,
  F-RCT-02/03/05).

### 7.2 Synthesis-level uncertainty

- **Confidence on APP-CROSS-P1-01 reachability:** the panic is empirically
  reproducible; whether ANY production code path triggers it today is
  uncertain (only caller is `should_ignore_path` with zero live callers
  outside the `project/` module). Marked P1 because the next contributor
  who wires `should_ignore_path` into a file-browser or tree-pruner makes
  it live.
- **APP-OBS-P1-02 user-frequency:** whether EKO users in the field actually
  configure webhook endpoints is unknowable from the repo. The leak path
  is deterministic; whether it fires today is usage-dependent.
- **APP-BOOT-P1-01 GUI env-var inheritance:** the impact claim relies on
  the comment at `infra.rs:305`. Actual env-var inheritance on each target
  OS was not measured.
- **APP-TSK-P3-02 probability:** requires crash during a narrow window in
  `recover_incomplete`. The drain-loop soft-lock (APP-TSK-P3-04) is the
  higher-leverage fix and bounds the worst case even without P3-02.
- **Cross-references into framework review (`S-FW-01`):** the framework
  synthesis carries the framework-side counterparts of APP-OBS-P1-03,
  APP-CROSS-P2-01..P2-03. The two syntheses are intentionally complementary;
  neither subsumes the other.

### 7.3 What this synthesis does NOT do

- Does not re-audit any finding — all evidence is backlinked to the
  originating report's "Evidence" and "Validation reports" sections.
- Does not promote or demote any finding's priority except where explicitly
  noted (and the rationale is given).
- Does not address B-phase inventory findings, F-phase framework-internal
  findings, or X-phase cross-cutting synthesis findings; those belong to
  their own syntheses.
- Does not modify `TASKS.md` or `README.md`.

---

## 8. Conditions That Invalidate This Synthesis

- Any baseline commit change underneath `echo-agent` `9b0e0fa` or
  `echo-agent-cli` `b3b2e81` invalidates the source reports; re-run the
  affected A-task validations before re-trusting the synthesis.
- Specific invalidation triggers per finding are documented in each
  originating task report's "Conditions that make this report stale"
  section. The most load-bearing ones for Tier 0:
  - Applying `redact_secrets` at the three leak boundaries
    (APP-OBS-P1-01/02/03) — resolves Tier 0 item 1.
  - Rewriting `gitignore::globstar_match` over `Vec<char>`
    (APP-CROSS-P1-01) — resolves Tier 0 item 2.
  - Replacing the 8 wrong-target refresh calls
    (APP-STATE-MEM-P1-01) — resolves Tier 0 item 3.
  - Wiring CLI `/workspace switch` through `AppState::switch_workspace`
    (APP-SURF-CLI-P1-01) — resolves Tier 0 item 5.
  - Adding the API-key fast-fail gate (APP-BOOT-P1-01) — resolves Tier 0
    item 6.
- Adding any new `#[tauri::command]`, `SubagentEvent` variant,
  `AgentEvent` variant, `ChatEvent` variant, or non-GUI trigger requires
  re-evaluating the parity matrix in §4.1.
- Adding any new persistence boundary (new repository, new emit channel)
  requires re-evaluating the secret-leak cluster in §4.4.

---

## 9. Handoff

Downstream syntheses (e.g. `S-X-01` cross-cutting, `S-CLOSE-01` closure)
may rely on:

1. **Application adapter conformance is the strongest positive finding.**
   The task-runtime adapter is thin, the file authority is single, claim
   identity is sound, and recovery is fail-closed. AGENTS.md rule 6 holds.
2. **The four highest-leverage fixes** (Tier 0 items 1–6) address every P1
   finding and are all small, localized changes. They should be sequenced
   first.
3. **The APP-CHAT-P2-01 recorder extraction is the keystone for surface
   parity.** It unblocks TUI/CLI/channels durable tool history, the
   receive-side `ExecutionEvent` enum, and the subagent bridge cleanup.
4. **The dead-code sweep (Tier 2 item 18) is opportunistic.** Each item is
   small; most can ride along with nearby changes per AGENTS.md "随手清理".
5. **The framework review synthesis (S-FW-01) carries the framework-side
   counterparts** of APP-OBS-P1-03, APP-CROSS-P2-01..P2-03, and the
   F-EXT-02 / F-INT-01/02 / F-TSK-03 framework-level gaps this synthesis
   references. The two syntheses are complementary; fix coordination
   across the framework/application boundary (especially for the
   pre-compaction flush, MCP transport, and DAG kernel gaps) requires
   reading both.

Reports downstream tasks must read:
- This document for the merged, deduplicated, prioritized application
  finding set.
- The originating task reports (backlinked from each `APP-*` finding) for
  full evidence and per-finding regression-validation proposals.
- `S-FW-01` (framework review synthesis) for the framework-side
  counterparts of the cross-cutting findings.
