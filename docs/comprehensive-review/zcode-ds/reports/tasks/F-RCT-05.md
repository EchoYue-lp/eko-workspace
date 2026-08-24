# F-RCT-05: Steer, interrupt, snapshot, and resume

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: clean (both repositories; probe crate `/tmp/frct05-probe` outside both repos)

## Question

Can a running or interrupted Agent resume without replaying completed side
effects or losing canonical context?

**Answer: No.** On the EKO main path, (a) cancelling or erroring mid-tool-batch
persists a checkpoint that the resume validator rejects, and the fallback
wipes the whole conversation context (P1-01, dynamically proven); (b) the
documented same-turn steer is silently dropped because the drain key
(`current_run_id`) never matches the mailbox lease key (`turn_id`) on
invocation-driven turns (P1-02); (c) resume re-activates skills only in the
tracking registry while the skill tools consult the shared registry, so
Tier-3 skill access breaks after a fresh-process resume (P1-03, independent
confirmation of F-SKL-01-P1-02). The documented call_id-based completed-tool
skip (MASTER-PLAN) is not implemented at all (P2-01).

## Scope

- `echo-agent/src/agent/steer.rs` (full read), `echo-agent/src/agent/handle.rs`
  (full read), `echo-agent/src/agent/snapshot.rs` (full read; checkpoint
  persistence :570-716, tool policy :1189-1279, skill capture :206-222).
- `echo-agent/src/state/mod.rs` (`AgentCheckpoint` :117-237, validator
  :186-231), `echo-agent/src/state/file.rs` (`FileRuntimeStateStore`
  :151-204, atomic_write :210+).
- Resume path: `echo-agent/src/agent/react/mod.rs:1671-1763`
  (`resume_from_state_store`, `force_checkpoint`),
  `run/context.rs:230-261` (`restore_thread_context`), `direct.rs:11`,
  `run/context.rs:490-623` (prepare_stream_context* mode dispatch).
- Shared loop and phases: `run/stream_channel.rs:35-316` (wrapper, steer
  lease), :328-349 (`drain_steer_into_context`), :494-757 (`run_core_loop`),
  `run/phases/tools.rs` (batch + cancel/error checkpoint arms),
  `run/phases/compact.rs` (:21-116), `run/phases/finalize.rs`
  (terminals + checkpoints), `run/react_loop.rs:598-751` (non-streaming
  driver), `run/phases/think.rs` (cancel outcome).
- EKO side: `echo-agent-cli/echo-agent-app-core/src/chat_driver.rs:400-560`
  (invocation construction, modes), `tasks/task_runtime/task_tools.rs:178-180`
  (`formal_run_id_for_turn`), `runtime.rs:41-86` (state store wiring),
  `src/tauri/commands/chat.rs:733-795` (GUI steer), `src/tui/events.rs:1438-1474`
  (TUI steer), `src/tauri/commands/conversations.rs:712`, `src/main.rs:215`,
  `src/cli/cmd_impls/session.rs:140` (conversation-store restore via
  `load_messages` — A-STATE-01 boundary).
- Skill registries: `capabilities.rs:655-680, 835-950`,
  `echo-execution/src/skills/external/{activate_tool,resource_tool,run_script_tool}.rs`,
  `snapshot.rs:206-222` vs `react/mod.rs:1703-1704`.

## Out Of Scope

- Tool batch internals (ordering, timeout arithmetic, concurrency) → F-RCT-04
  (P1-01/P1-02); the batch checkpoint arms are referenced here only for their
  resume consequences.
- Conversation-store persistence/restore (EKO `load_messages` path) →
  A-STATE-01, F-MEM-01.
- TaskRuntime plan/pause/resume (EKO task model) → A-TSK-01..04, F-TSK-03;
  the framework `TaskNode` hydration (`snapshot.rs:763-791`) is noted only.
- Steer UI/queue policy (TUI queued follow-ups, GUI FIFO) → A-SRF-01/A-SRF-03.
- SQLite `SqliteRuntimeStateStore` semantics → F-MEM-02.
- Trace finalization gaps on terminal paths → F-RCT-02 (P2-01, P2-04),
  F-RCT-04 (P1-02).

## Inputs

- Root `AGENTS.md` (UTF-8/panic safety, one-authority, layering, local threat
  model), shared `README.md`, `REPORTING.md`, `TASKS.md` (F-RCT-05 card),
  `zcode-ds/README.md`, report templates.
- Dependency task reports read (complete): zcode-ds `F-RCT-02` (loop terminal
  inventory, non-streaming wrapper gap), `F-RCT-04` (batch exit paths,
  checkpoint sites :429 etc., sanitize repair), `F-MEM-01` (file-store
  durability precedent, FileConversationStore hardened pattern). `F-SKL-01`
  read for the P1-02 cross-reference; the resume divergence was independently
  re-verified in this task (V02-01, finding P1-03).
- Historical documents treated as hypotheses: root `docs/MASTER-PLAN.md`,
  `docs/PROJECT-ANALYSIS.md`, `echo-agent-cli/docs/2026-07-11-running-input-interrupt-design.md` —
  classified in the Historical Claim Status section.

## Layering Decision

- Generic mechanism (framework): `TurnSteerMailbox`/`ActiveTurnLease`,
  `AgentCheckpoint` + `RuntimeStateStore` trait (file impl default, sqlite
  feature option), `save_runtime_checkpoint`/`resume_from_state_store`,
  `validate_tool_message_pairing`, `hydrate_running_nodes`, and the loop's
  checkpoint arms are all correctly placed in `echo-agent`. Findings P1-01,
  P1-02, P1-03, P2-01, P2-02, P3-01 are framework defects; no repository
  movement is recommended.
- EKO product policy (application): the decision to drive every turn through
  `execute_stream_message_with_invocation_context` (Execute mode + runtime
  `turn_id`/`run_id` split + per-mode `run_id`), `formal_run_id_for_turn`,
  GUI/TUI steer entry points, and conversation restore via `load_messages`
  are application policy. The steer mismatch (P1-02) and the checkpoint-poison
  wipe (P1-01) become user-visible exactly because of these choices.
- Adapter boundary: none new; EKO calls framework APIs directly
  (`steer_input`, `execute_stream_message_with_invocation_context`,
  `load_messages`), no scheduling/state authority in adapters.
- Duplicate-search terms (both repositories, V01-01): `steer`, `Steer`,
  `TurnSteerMailbox`, `drain_steer`, `steer_input`, `checkpoint`,
  `Checkpoint`, `AgentCheckpoint`, `RuntimeCheckpoint`,
  `save_runtime_checkpoint`, `resume_from_state_store`, `restore_thread_context`,
  `save_checkpoint`, `get_checkpoint`, `completed_tool_call_ids`,
  `interrupt`, `RuntimeStateStore`, `TaskNode`. Results: one mailbox, one
  checkpoint type, one resume entry, one live batch-cancel authority; no
  parallel implementation in either repository; EKO `load_messages` is a
  distinct conversation-restore mechanism (A-STATE-01), not a second runtime
  checkpoint authority.

## Current Path

Verified data flow (anchors in V02-01):

- Turn start (EKO): `drive_chat_inner` (chat_driver.rs:425) builds the
  invocation — `runtime.run_id = Some("taskrun:{turn_id}")` only for Task mode
  (chat_driver.rs:451-452, task_tools.rs:178-180), `runtime.turn_id =
  Some(turn_id)` always (chat_driver.rs:498), `conversation_id` set
  (chat_driver.rs:493) — then
  `execute_stream_message_with_invocation_context` (react/mod.rs:2913-2925) →
  `run_stream_message_entry(Execute, Some(invocation))` →
  `run_stream_channel` (stream_channel.rs:35-316).
- Steer: mailbox lease `begin(turn_id)` where
  `turn_id = runtime.turn_id.or(runtime.run_id).or(agent.current_run_id).or(uuid)`
  (stream_channel.rs:111-122); `set_steerable(true)` after snapshot creation
  (:300). `drain_steer_into_context` reads `self.current_run_id`
  (= `runtime.run_id` for invocation paths, snapshot.rs:461-469) and drains
  that id (stream_channel.rs:333-336); consumed at iteration start (:538),
  before final text (:663), and at Finish (:709). EKO steer callers:
  `steer_chat_message` (tauri/commands/chat.rs:735-795), `/steer`
  (tui/events.rs:1438-1474) → `AgentHandle::steer_input` (handle.rs:187-194)
  → `ReactAgent::steer_input` (react/mod.rs:271-276).
- Checkpoint: `save_runtime_checkpoint` (snapshot.rs:570-631) persists
  `AgentCheckpoint{messages_json, current_plan, active_skills (union of both
  skill registries, snapshot.rs:206-222), blocked_reason, working_dir,
  timestamp}` into `RuntimeStateStore` (default file store, state/file.rs).
  Call sites: compact.rs:35 (every iteration, pre-prepare), tools.rs:429
  (batch completion), tools.rs:438-440 (periodic, interval default 0),
  tools.rs:203/258/296/309/336/414/419 (cancel/error mid-batch),
  finalize.rs:79/164/251 (terminals), force_checkpoint (react/mod.rs:1758-1763).
- Resume: Execute mode → `restore_thread_context` (context.rs:502/571,
  direct.rs:11) → `resume_from_state_store` (react/mod.rs:1680-1752):
  `get_checkpoint` → `restore_messages` (pairing validation, state/mod.rs:157-168,
  186-231) → `set_messages` + plan + `mark_activated` (tracking registry only,
  :1703-1704) + `hydrate_running_nodes` + working_dir; on `Err` the fallback
  is `reset_messages()` — context cleared to system prompt (context.rs:245-248,
  216-228).
- Terminal/abnormal exits: cancel during think → `ThinkOutcome::Cancelled` →
  `Ok(())` (stream_channel.rs:605-610), no checkpoint; cancel/error during a
  tool batch → grace 5 s → checkpoint with blocked reason → `Abandoned` →
  `Ok(())` (tools.rs:203-205, 295-300, 309-312, 336-339, 418-423, 258-262,
  414-415); batch timeout → `try_send(Err)` only, no checkpoint
  (tools.rs:284-292); consumer channel close drops the loop task with no
  terminal checkpoint.

## Findings

### F-RCT-05-P1-01: Interrupting mid-tool-batch persists an unresumable checkpoint, and the next restore wipes the entire conversation context

- Priority: P1
- Confidence: high (static chain fully verified AND dynamically reproduced by
  probe V04-04)
- Layer: framework
- Evidence: assistant-with-tools message is pushed before execution
  (`phases/tools.rs:101-113`); cancel arms checkpoint at that moment
  (`tools.rs:203`, `:296`, `:309`, `:336`, `:419`) and error arms at
  `tools.rs:258-262`, `:414` — in-flight concurrent tools / not-yet-run serial
  tools have no result in context; restore-side validator rejects any
  result-less call (`state/mod.rs:186-231`, error at `:221-230`); the
  rejection propagates (`react/mod.rs:1689`) to `restore_thread_context`
  (`run/context.rs:245-248`), which calls `reset_messages()` (`context.rs:216-228`)
  clearing the context to the system prompt; `restore_thread_context` runs at
  the start of every Execute-mode turn (context.rs:502/571), and EKO drives
  every chat turn in Execute mode (chat_driver.rs:512-514 →
  react/mod.rs:2913-2925). Probe: cancel mid-batch →
  `blocked_reason="Tool batch cancelled"`, `unpaired = ["write-1"]`,
  `restore_messages() = Err(... checkpoints has tool calls without results:
  write-1)` (V04-04).
- Reachability: definition (`AgentCheckpoint`/validator) → registration
  (EKO `state_store` + `conversation_id` on every agent, runtime.rs:80-86,
  infra.rs:126) → live caller: any user stop (cancel) or tool error during a
  multi-call batch, then any subsequent turn (same process or after restart).
  The poisoned checkpoint additionally survives one more save: the next
  turn's compact checkpoint (compact.rs:35) is taken before
  `ContextManager::prepare` runs `sanitize_tool_call_pairing`, so the dangling
  calls are re-serialized; only a second post-prepare checkpoint heals.
- Expected invariant: interrupt at a checkpointed "safe point" yields a
  restorable checkpoint; a rejected checkpoint must never silently discard
  prior state (AGENTS.md: prevent unintended data loss; MASTER-PLAN:148's
  pairing validation is a recovery aid, not a wipe trigger).
- Observed behavior: cancel/error mid-batch → checkpoint saved with unpaired
  calls → next turn's resume rejects it → warn + full context wipe (all
  messages, plan, tool results) to `[system]`; user sees a normal-looking turn
  that forgot the conversation; on restart the same wipe happens at the first
  turn. The GUI history pane (conversation store) still shows old messages,
  but the model's canonical context is gone, and the next checkpoint
  overwrites the only checkpoint file with the wiped state.
- Impact: conversation-context data loss on the flagship resume path exactly
  at the points the framework explicitly checkpoints for recovery (cancel and
  tool-error arms); silent (warning-only) so EKO surfaces cannot distinguish
  "resumed" from "forgot everything".
- Root cause: checkpoint save is not guarded by the pairing invariant — the
  validator exists only on the restore side; the restore failure path was
  designed as "start fresh" without preserving prior in-process state, and
  the cancel/error arms save at a moment the context is structurally invalid
  for resume.
- Direction: (a) on cancel/error mid-batch, checkpoint only the paired prefix
  (or sanitize/clear pending calls before save, mirroring
  `sanitize_tool_call_pairing` semantics at save time); (b) on restore
  rejection, fall back to the *previous* checkpoint or keep the in-process
  context instead of `reset_messages()`; (c) surface the rejection as an
  error event so EKO can inform the user; (d) move the compact checkpoint
  after `ContextManager::prepare` (or re-save post-prepare). Regression: the
  probe scenario as a permanent fixture — cancel mid-batch, rebuild agent,
  resume, assert context contains the pre-cancel messages and no wipe.
- Regression validation: (1) `state::checkpoint` test asserting a
  cancel-shaped checkpoint (assistant call + partial results) is either
  rejected-with-preservation or accepted with a synthesized "interrupted"
  result; (2) loop-level test: slow tool batch, cancel, next turn — assert
  context retained; (3) EKO `drive_chat` smoke after cancel.
- Validation reports: [V02-01](../validations/F-RCT-05/V02-01.md),
  [V03-03](../validations/F-RCT-05/V03-03.md),
  [V03-04](../validations/F-RCT-05/V03-04.md),
  [V04-04](../validations/F-RCT-05/V04-04.md),
  [V04-01](../validations/F-RCT-05/V04-01.md)

### F-RCT-05-P1-02: Same-turn steer is silently dropped on the EKO main path — drain key (`current_run_id`) never matches the mailbox lease key (`turn_id`)

- Priority: P1
- Confidence: high (static chain unambiguous; the only test uses equal ids)
- Layer: framework (wiring) with EKO exposure
- Evidence: lease keyed by `turn_id = runtime.turn_id.or(runtime.run_id).or(...)`
  (stream_channel.rs:111-122); drain keyed by `self.current_run_id`
  (stream_channel.rs:333) which equals `runtime.run_id` on invocation paths
  (snapshot.rs:461-469); EKO Chat/Auto sets `run_id = None`
  (chat_driver.rs:451-452 → drain early-returns 0), Task mode sets
  `run_id = "taskrun:{turn_id}"` (task_tools.rs:178-180 →
  `mailbox.drain` finds no active turn, steer.rs:117-119); `steer` itself
  validates against the lease id (steer.rs:88-107) so the API returns `Ok`
  and the UI reports "accepted"/"已补充到当前任务" (tauri/commands/chat.rs:783-786,
  tui/events.rs:1454-1460); the framework integration test sets
  `run_id == turn_id == "turn-steer-1"` (stream_channel.rs:2106-2107) and
  passes, masking the mismatch (V04-02/V04-03).
- Reachability: definition (`TurnSteerMailbox`, steer.rs) → registration
  (agent construction, react/mod.rs:544; handle.rs:187-194) → live caller:
  every EKO GUI `steer_chat_message` or TUI `/steer` during an active
  invocation-driven turn — the primary chat surface.
- Expected invariant: a steer accepted by `steer_input` is injected at the
  next safe point (iteration start / before final text / at Finish —
  stream_channel.rs:538/663/709; documented contract
  echo-agent-cli/docs/2026-07-11-running-input-interrupt-design.md:9,98).
- Observed behavior: the message is pushed into the mailbox (Ok), drained
  under a key that matches nothing, and silently discarded when the lease
  drops (steer.rs:143-147).
- Impact: user corrections/mid-turn instructions are lost without any error;
  both TUI and GUI claim success; the steer capability — a documented product
  feature (MASTER-PLAN:157, design doc) — is dead on the main path.
- Root cause: two different identity derivations for the same turn — the
  mailbox uses the product `turn_id` while the drain uses the snapshot's
  `current_run_id` (product `run_id`); the non-invocation paths agree only
  because both fall back to `agent.current_run_id`, and the test hardcodes
  equality.
- Direction: make `drain_steer_into_context` use the same id the lease used —
  carry the lease id in the snapshot (e.g. store `turn_steer_turn_id` from
  `stream_channel.rs:111-122` into the snapshot, or drain by
  `runtime.turn_id.or(runtime.run_id)`), or key the mailbox by `run_id` for
  invocation paths; add a test with `run_id != turn_id` and with
  `run_id = None` asserting the steer message reaches the context.
- Regression validation: mocked invocation turn with `turn_id = "t1"`,
  `run_id = None` (Chat/Auto shape) — steer mid-LLM-call → assert
  `steer correction` appears in the context messages before the final
  answer; same with `run_id = "taskrun:t1"` (Task shape).
- Validation reports: [V02-01](../validations/F-RCT-05/V02-01.md),
  [V04-02](../validations/F-RCT-05/V04-02.md),
  [V04-03](../validations/F-RCT-05/V04-03.md),
  [V05-01](../validations/F-RCT-05/V05-01.md)

### F-RCT-05-P1-03: Resume marks only the tracking SkillRegistry while the skill tools consult the shared registry — "not activated" after fresh-process resume (independent confirmation of F-SKL-01-P1-02)

- Priority: P1
- Confidence: medium (static chain complete; fresh-process dynamic resume not
  executed — same limit as F-SKL-01)
- Layer: framework
- Evidence: checkpoint `active_skills` is the UNION of tracking and
  progressive registries (`snapshot.rs:206-222`); resume marks only
  `self.tools.skill_registry.mark_activated(skill_name)`
  (`react/mod.rs:1703-1704`); the three progressive tools are constructed with
  the shared registry (`capabilities.rs:857-865`) and gate on
  `registry.is_activated(...)` (`resource_tool.rs:98-103`,
  `run_script_tool.rs:191`, `activate_tool.rs:150,191`); reachability: EKO
  resume path is live (V02-01) and EKO persists `AgentCheckpoint`s including
  `active_skills` (infra.rs:1242).
- Reachability: definition (`SkillRegistry::mark_activated/activated_names/
  is_activated`, echo-execution/src/skills/registry.rs:295,342) → registration
  (both registries fed at capabilities.rs:660-680) → live caller: any
  fresh-process resume (app restart) of an agent with model-activated skills,
  then `read_skill_resource`/`run_skill_script`/`activate_skill`.
- Expected invariant: activation state is a single authority; after resume all
  previously activated skills behave identically and no content is injected
  twice (framework's own save/restore contract: save both, restore both).
- Observed behavior: after fresh-process resume the shared registry is never
  marked → `read_skill_resource`/`run_skill_script` return "Skill 'X' has not
  been activated"; if the model re-activates, the skill instructions are
  injected a second time (tool result + projections).
- Impact: the flagship resume path silently breaks Tier-3 skill access and can
  duplicate instructions in context.
- Root cause: the shared registry was introduced for async tool access
  (capabilities.rs:659-677) without reconciling the resume writer; the
  "All activation paths should use this method" comment
  (capabilities.rs:962-964) is not enforced for the resume path.
- Direction: single authority — mark both registries on resume (or have the
  tools consult a merged activated set, or drop the tracking registry);
  add a save/restore round-trip test asserting both registries agree.
  Cross-reference: F-SKL-01-P1-02 (canonical ID); fix belongs with it.
- Regression validation: unit test — activate via `ActivateSkillTool`,
  snapshot, rebuild agent + restore checkpoint, assert
  `run_skill_script`/`read_skill_resource` succeed without re-activation and
  context contains one instruction copy.
- Validation reports: [V02-01](../validations/F-RCT-05/V02-01.md)

### F-RCT-05-P2-01: The documented call_id-based completed-tool skip does not exist — resume re-issues completed side effects whenever the checkpoint predates them

- Priority: P2
- Confidence: high
- Layer: framework
- Evidence: `completed_tool_call_ids` has exactly one production consumer site,
  `resume_from_state_store`, and it is used only for logging — a trace event
  and a `tracing::info` line (react/mod.rs:1731-1741); no
  skip/replay gate exists in `run_tools` (tools.rs:50-443) or the resume path
  (react/mod.rs:1680-1752); checkpoint timing: batch-completion checkpoint
  only at tools.rs:429 (plus periodic, default interval 0), while a
  batch-timeout channel-close (tools.rs:284-292 → consumer drops stream →
  loop task dropped) or process kill mid-batch leaves the previous (pre-batch)
  checkpoint; `validate_tool_message_pairing` (state/mod.rs:186-231) rejects
  rather than partially resumes; MASTER-PLAN:67/96/147/185/373 promise
  "resume 跳过已完成副作用"/"恢复按 call_id 跳过已完成工作"/"已完成结果不重放".
- Reachability: any kill/timeout during a batch after a write tool completed
  but before the :429 checkpoint; the resumed model sees a history without the
  completed call and can re-issue it (the framework's test only asserts the
  trace event, stream_channel.rs:1634-1711).
- Expected invariant: resume must not re-execute tools whose completion is
  persisted; incomplete writes must be surfaced for retry/skip decision
  (MASTER-PLAN:70).
- Observed behavior: no resume-side skip mechanism; the only protection is
  checkpoint freshness, and the "incomplete" class is handled by whole-
  checkpoint rejection (P1-01), not by partial recovery with a warning.
- Impact: completed side effects (file edits, shell commands) can be replayed
  after an interrupted batch; the M3 acceptance is unmet.
- Root cause: checkpoint-per-batch-completion + restore-whole-history design
  was implemented, but the promised call_id skip and partial-side-effect
  classification were never wired into resume.
- Direction: either (a) persist per-call completion records
  (call_id → completed/failed/interrupted) in `AgentCheckpoint` and inject
  them into the resumed context so the model sees "already done" facts
  (idempotency keys for write tools), or (b) explicitly reject the
  at-most-once claim and document the guarantee; delete the misleading
  MASTER-PLAN claim. Regression: two-phase fixture — tool completes, kill
  before :429 checkpoint, resume, assert the model is informed the call
  already ran (or the tool is not re-invoked by the scripted mock).
- Regression validation: mock LLM that re-emits the same tool call after
  resume from a pre-batch checkpoint; assert the framework injects an
  "already completed" note or blocks re-execution.
- Validation reports: [V03-02](../validations/F-RCT-05/V03-02.md),
  [V05-01](../validations/F-RCT-05/V05-01.md)

### F-RCT-05-P2-02: No test exercises any resume-relevant failure shape — cancel→resume, error→resume, invocation steer, skill round-trip, corrupt file

- Priority: P2
- Confidence: high
- Layer: framework (test infrastructure)
- Evidence: repository-wide test inventory (V04-03, V03-02): resume tests
  cover only paired-history happy paths (`completed_tool_batch_is_checkpointed_before_next_model_call`
  stream_channel.rs:1600-1632; `resume_records_checkpoint_origin_and_completed_tools_in_trace`
  :1634-1711); steer test hardcodes `run_id == turn_id` (:2106-2107); no test
  saves a cancel/error checkpoint and resumes; no test asserts the skill
  save/restore round-trip; no test corrupts a checkpoint file; the
  `state::checkpoint` tests cover only 2 of 4 validator branches
  (V04-01).
- Reachability: not-applicable (test gap).
- Expected invariant: task card requires fixtures for snapshot field
  round-trip, completed-tool skip, interrupt at each safe point, and
  corrupted/incomplete snapshot handling; none of the failure shapes exist.
- Observed behavior: the suite is green while three P1 defects (P1-01, P1-02,
  P1-03) survive unobserved.
- Impact: Q-FLT-01 has no fixtures to reuse; future resume changes have no
  regression net.
- Root cause: checkpoint/resume tests were written for the happy path only
  when the feature landed.
- Direction: add the fixture family: (a) cancel-mid-batch → resume (must fail
  today, per P1-01); (b) tool-error-mid-batch → resume; (c) invocation steer
  with `run_id=None` and `run_id="taskrun:..."` (must fail today, per P1-02);
  (d) skill activation save/restore round-trip (must fail today, per P1-03);
  (e) corrupt checkpoint file → explicit error + preserved state; (f) name
  mismatch / orphan-result validator branches.
- Regression validation: the fixtures themselves; `cargo test -p echo_agent
  --lib` green after the P1 fixes.
- Validation reports: [V04-03](../validations/F-RCT-05/V04-03.md),
  [V04-01](../validations/F-RCT-05/V04-01.md),
  [V04-02](../validations/F-RCT-05/V04-02.md)

### F-RCT-05-P3-01: Corrupt checkpoint files are handled with warn+reset and silently overwritten on the next save — no backup, no explicit error

- Priority: P3
- Confidence: high
- Layer: framework
- Evidence: `FileRuntimeStateStore::get_checkpoint` returns `Err` on JSON
  parse failure (state/file.rs:160-161); `restore_thread_context` warn +
  `reset_messages` (context.rs:245-248); `save_checkpoint` atomically
  overwrites the path (state/file.rs:170-187) without moving the corrupt file
  aside; contrast the hardened `FileConversationStore` precedent
  (F-MEM-01, explicit corrupt-file errors, file_conversation.rs:20-22,153-157).
- Reachability: any truncation/manual edit/downgrade of the per-conversation
  checkpoint file, then the next turn (Execute mode) or next save.
- Expected invariant: corrupt runtime-state file surfaces explicitly and is
  recoverable (AGENTS.md: prevent unintended data loss).
- Observed behavior: warn + context wipe; the corrupt file is then replaced by
  the next checkpoint, destroying the only copy; EKO sees a normal stream
  (error never propagates past `restore_thread_context`).
- Impact: silent loss of the last recoverable snapshot on disk; inconsistent
  with the project's own hardened-file pattern.
- Root cause: the restore fallback predates the hardening and was never
  updated; the checkpoint file has no backup/rename-on-corrupt step.
- Direction: on parse error, move the file to `<path>.corrupt` (or surface a
  typed error to the turn) before the next save overwrites it; keep the
  in-process context on rejection (shared fix with P1-01).
- Regression validation: unit test — write truncated checkpoint file,
  `get_checkpoint` errors, save_checkpoint writes a new one, assert the
  corrupt bytes are preserved (renamed) rather than overwritten.
- Validation reports: [V03-04](../validations/F-RCT-05/V03-04.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate search across both repositories (steer/checkpoint/resume/safe-point authorities) | yes | passed | [V01-01](../validations/F-RCT-05/V01-01.md) |
| V02 | Registration and runtime reachability trace (EKO invocation path → steer keying, resume chain, skill save/restore divergence) | yes | passed | [V02-01](../validations/F-RCT-05/V02-01.md) |
| V03 | Snapshot field round-trip (save vs restore field-by-field; pairing validator) | yes | passed | [V03-01](../validations/F-RCT-05/V03-01.md) |
| V03 | Completed-tool skip (call_id consumers; replay gap) | yes | passed (defect found) | [V03-02](../validations/F-RCT-05/V03-02.md) |
| V03 | Interrupt at each safe point (checkpoint-site inventory; clean vs poisoned) | yes | passed (defect found) | [V03-03](../validations/F-RCT-05/V03-03.md) |
| V03 | Corrupted/incomplete snapshot handling (file parse error; pairing rejection; fallback) | yes | passed (defect found) | [V03-04](../validations/F-RCT-05/V03-04.md) |
| V04 | `cargo test -p echo_agent --lib --locked 'state::checkpoint'` | yes | passed, exit 0 (2/2) | [V04-01](../validations/F-RCT-05/V04-01.md) |
| V04 | `cargo test -p echo_agent --lib --locked steer` | yes | passed, exit 0 (3/3) | [V04-02](../validations/F-RCT-05/V04-02.md) |
| V04 | `cargo test -p echo_agent --lib --locked 'react::run::stream_channel'` | yes | passed, exit 0 (23/23) | [V04-03](../validations/F-RCT-05/V04-03.md) |
| V04 | Probe: cancel mid-batch → saved checkpoint restorability | yes | failed (P1-01 evidence; probe exit 0) | [V04-04](../validations/F-RCT-05/V04-04.md) |
| V05 | Historical-document drift (MASTER-PLAN / PROJECT-ANALYSIS / steer design doc) | conditional | passed (3 claim families regressed) | [V05-01](../validations/F-RCT-05/V05-01.md) |

All required validations executed; every command has a known exit code;
the one failed execution (V04-04) became finding P1-01.

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| MASTER-PLAN:67/96/147/185 — resume "按 call_id 跳过已完成工作", "不重新发射已完成副作用" | regressed (not implemented) | `completed_tool_call_ids` only logged (react/mod.rs:1731-1741); no skip gate in tools.rs/resume (V03-02) |
| MASTER-PLAN:148 — "恢复时先校验 tool_call/tool_result 配对" | current (validation exists) / regressed (failure handling) | validator state/mod.rs:186-231; rejection → full wipe (V03-04, P1-01) |
| MASTER-PLAN:373 M3 — "已完成结果不重放,写副作用不确定时阻塞自动恢复并要求 retry/skip 决策" | regressed | no retry/skip decision path; whole-checkpoint rejection instead (P1-01/P2-01) |
| MASTER-PLAN:146 — "cancel 是不可继续的终态" | stale | cancel arms save a "Tool batch cancelled" resume checkpoint (tools.rs:203/296/…) that then fails restore (P1-01) |
| MASTER-PLAN:157 + design doc 2026-07-11:9/98 — same-turn steer injected at the next safe point, consumed before model call / after batch / before final text | regressed (EKO main path) | drain keyed on `current_run_id` vs lease keyed on `turn_id` (stream_channel.rs:333 vs 111-122; P1-02) |
| PROJECT-ANALYSIS:166-167 — checkpoint/transcript mechanism (anchor "snapshot.rs:312") | current (mechanism); stale (anchor) | `save_runtime_checkpoint` at snapshot.rs:570; transcript at :648 (V05-01) |
| PROJECT-ANALYSIS:129 — runtime checkpoint table (`FileRuntimeStateStore`) | current | state/file.rs; EKO per-workspace dir (infra.rs:126, state.rs:924) |
| F-SKL-01-P1-02 — resume marks only tracking registry; tools check progressive registry | current (re-confirmed independently) | snapshot.rs:206-222 vs react/mod.rs:1703-1704 vs resource_tool.rs:98-103 (V02-01, P1-03) |

## Coverage And Uncertainty

- All conclusions are static except four test runs (V04-01..03) and one
  dynamic probe (V04-04). Not executed: fresh-process resume end-to-end in
  EKO (would require GUI/CLI restart harness — Q-E2E-01 scope); the tool-error
  checkpoint arm dynamically (V04-04 covers cancel only; the message-state
  logic is identical); Task-mode resume through EKO's executor.
- P1-01's wipe claim follows from the probe (checkpoint rejected) + the
  restore fallback chain (context.rs:245-248); the in-process next-turn wipe
  is static evidence only.
- The steer drain mismatch (P1-02) is verified for the invocation shape EKO
  uses; framework standalone (non-invocation) steer works — the finding is
  scoped to the EKO main path.
- The batch-timeout no-checkpoint path (tools.rs:284-292) and its
  side-effect-replay consequence (P2-01) were assessed statically; the
  channel-close drop depends on consumer behavior (chat_driver drops the
  stream on the error event — F-RCT-04-P1-02).
- `sanitize_tool_call_pairing` interplay (poison persists one extra
  checkpoint) is static evidence; its full semantics belong to F-RCT-04.
- F-RCT-02-P1-01 (non-streaming silent `Ok("")` on loop error) and
  F-RCT-04-P1-02 (terminal-less timeout/cancel) interact with resume only in
  that those turns also produce no terminal checkpoint — noted, not re-filed.
- The `react_checkpoint_interval` default (0 = only batch/compact/terminal
  checkpoints) means the periodic arm is dormant for default EKO config.

## Handoff

- Downstream tasks may rely on: single-authority steering/checkpoint/resume
  (V01); the EKO invocation identity split `turn_id` vs `run_id` and its two
  defects (P1-01, P1-02); the poisoned-checkpoint wipe chain with the probe
  reproduction (V04-04); skill save/restore asymmetry (P1-03); absent
  completed-tool skip (P2-01); green test baseline + fixture gaps (V04-01..03,
  P2-02); corrupt-file handling (P3-01).
- `A-CHAT-01`/`A-SRF-01`/`A-SRF-03`: the steer contract on GUI/TUI must
  account for P1-02 (UI reports acceptance that never lands); the
  restore-wipe (P1-01) is silent to the chat surface.
- `A-STATE-01`: EKO's `load_messages` restore coexists with the framework
  checkpoint restore (Execute mode); the two must be reconciled after P1-01's
  fix so the conversation-store path cannot mask the checkpoint wipe.
- `X-STA-01`: use P1-01/P1-02/P1-03 as crash-point recovery matrix inputs;
  the identity table must record turn_id vs run_id divergence.
- `Q-FLT-01`: convert V04-04's probe into a permanent fixture; add the
  P2-02 fixture list.
- `Q-TST-01`: coverage gaps from P2-02.
- Reports to read: this report + V01-01 through V05-01; dependency reports
  F-RCT-02, F-RCT-04, F-MEM-01; cross-reference F-SKL-01 (P1-02 canonical).
- Stale triggers: any change to `steer.rs`, `snapshot.rs`
  (save_runtime_checkpoint/ToolRuntime skill capture), `state/mod.rs`
  (AgentCheckpoint/validator), `state/file.rs`, `react/mod.rs` resume path,
  `stream_channel.rs` steer lease/drain or loop exits, `phases/tools.rs`
  checkpoint arms, `run/context.rs` restore fallback, `capabilities.rs`
  registry wiring, or EKO `chat_driver.rs` invocation construction
  invalidates the corresponding claims.
- Follow-up task IDs (fixes are not implemented in this review): A-CHAT-01,
  A-STATE-01, X-STA-01, Q-FLT-01, Q-TST-01; fix ownership: framework
  (echo-agent) for P1-01/P1-02/P1-03/P2-01/P2-02/P3-01.
