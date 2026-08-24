# A-INP-01: Prepared user turn, attachments, and input artifacts

> Status: complete
> Reviewer: ZCode (builtin:bigmodel-coding-plan/GLM-5.2)
> Review date: 2026-08-12
> `echo-agent` commit: not-applicable (application-layer task; framework not modified)
> `echo-agent-cli` commit: b3b2e81
> Worktree state: clean (read-only review)

## Question

Do all entry points normalize user input once while preserving Unicode,
attachment identity, long text artifacts, cleanup, and display/model projections?

## Scope

Primary source paths and behaviors inspected:

- `echo-agent-cli/echo-agent-app-core/src/prepared_turn.rs` (full, 950 lines) —
  `PreparedUserTurn`, `UserTurnInput`, `InputResourceRef`, `Delivery`,
  `ResourceKind`, `should_spill`, `spill_to_artifact`, `cleanup_user_input_scope`,
  `cleanup_user_input_older_than`, `resolve_user_input_spill_dir`,
  `build_original_message_reference`, `build_data_reference`, `path_component`.
- `echo-agent-cli/echo-agent-app-core/src/attachments.rs` (full, 341 lines) —
  `AttachmentRef`, `AttachmentData`, `save_attachment`, `save_attachments`,
  `stage_attachment_data`, `stage_local_attachment`, `sanitize_name`,
  `infer_mime_type`, `is_image_mime`, `build_message_from_refs`,
  `resolve_uploads_dir`.
- Entry-point adapters:
  - `echo-agent-cli/src/tui/events.rs` lines 1294-1435 (`handle_enter` /
    `send_to_agent`) and 4231-4304 (`SlashCommand::Steer`) and 1438-1470
    (`steer_active_turn`) and 3067-3091 (`SlashCommand::DeleteSession` cleanup).
  - `echo-agent-cli/src/tauri/commands/chat.rs` lines 625-680 (`send_chat`)
    and 750-803 (`steer_turn`).
  - `echo-agent-cli/src/tauri/commands/conversations.rs` lines 585-642
    (`delete_conversation` cleanup).
  - `echo-agent-cli/src/cli/repl.rs` lines 495-535 (`run_repl_turn`).
  - `echo-agent-cli/src/cli/channels.rs` lines 195-260 (channel message
    handler).
- Supporting types and layout:
  - `echo-agent-cli/echo-agent-app-core/src/types/request.rs:150-173`
    (`AttachmentSource`, `AttachmentData`).
  - `echo-agent-cli/echo-agent-app-core/src/workspace/layout.rs:88-143`
    (`WorkspaceLayout::artifacts`, `user_input_artifacts`, `uploads`).
  - `echo-agent-cli/echo-agent-app-core/src/infra.rs:55-74`
    (`tool_output_artifact_config` — shares the artifact root with
    user-input).
  - `echo-agent/echo-core/src/tools/artifact.rs:28-67`
    (`ToolOutputArtifactConfig::root_dir`, used by Tauri/TUI delete paths to
    derive the cleanup dir).

## Out Of Scope

Deferred to downstream tasks:

- **A-CHAT-01**: `drive_chat` / `drive_chat_inner` ownership, sink
  responsibility split, terminal-event invariants. This task only verifies
  that every entry point produces a `PreparedUserTurn` and hands it to
  `drive_chat`; it does not audit `drive_chat_inner` itself.
- **A-MEM-01 / A-MEM-02**: instruction/memory projection and conversation
  file round-trip (only the framework `project_message` /
  `restore_message` round-trip at line 863-882 is touched, as a smoke test
  of artifact preservation).
- **A-TSK-***: `TaskRun.attachments` lifecycle, `set_run_attachments`
  scheduling semantics (only the call site at `chat_driver.rs:315` is
  cited).
- **A-SRF-01 / A-SRF-02**: full TUI / Tauri surface capability matrices.

## Inputs

Required repository documents read in full:

- Repository root `AGENTS.md` (UTF-8 safety rule, multi-mode parity rule,
  "only Subagent, no Worker" terminology, framework-vs-application
  layering gate, "check whether it already exists before adding").
- `docs/comprehensive-review/REPORTING.md` (finding and validation
  contract).
- `docs/comprehensive-review/templates/task-report.md`,
  `templates/validation-report.md`.
- `docs/comprehensive-review/TASKS.md` (A-INP-01 card).

Dependency reports read:

- `zcode-glm/tasks/A-BOOT-01.md` (complete) — confirms
  `AgentRuntime::bootstrap` is the single composition root that installs
  `tool_output_artifact_config` on every agent. This is load-bearing for
  the cleanup-dir derivation in V03.
- `zcode-glm/tasks/B-PATH-01.md` (complete) — entry-point inventory used
  as the cross-reference for the five-entry matrix.

Historical documents treated as hypotheses: none (A-INP-01 has no prior
report under `zcode-glm/`).

## Layering Decision

This is an **application-layer** task. `PreparedUserTurn`,
`InputResourceRef`, `AttachmentRef`, the spill-to-artifact strategy, the
mode-hint prefix folding, the 32 KiB / 4 000-token thresholds, the
paste-vs-upload instruction/data distinction, and the
per-workspace-vs-global spill dir resolution are all EKO product policy
(local personal assistant that spills long pastes to local files for
`grep` / `read_artifact` recovery). None of them belong in the framework.

Adapter boundary is **thin and lossless**:

- `to_message` (`prepared_turn.rs:328-369`) is the single authoritative
  merge point that converts the turn into a framework
  `echo_agent::llm::types::Message`. It performs no scheduling, no state
  authority, no cancellation — it only flattens `instruction` + inline
  resources into `ContentPart`s.
- `inline_attachment_refs` (`prepared_turn.rs:312-323`) is a pure
  projection used to populate `ChatResources.attachments` for the
  TaskRuntime / subagent rebuild path; tool-reference resources are
  dropped because their content is already carried losslessly inside
  `instruction`.

Duplicate-search terms used across both repositories (full
`echo-agent-cli` tree and `echo-agent` framework):

- `PreparedUserTurn`, `UserTurnInput`, `InputResourceRef`, `Delivery`,
  `ResourceKind`, `SPILL_THRESHOLD_BYTES`, `should_spill`,
  `spill_to_artifact`, `build_original_message_reference`,
  `build_data_reference`, `resolve_user_input_spill_dir`,
  `cleanup_user_input_scope`.
- `AttachmentRef`, `AttachmentData`, `AttachmentSource`,
  `save_attachment(s)`, `stage_attachment_data`, `stage_local_attachment`,
  `build_message_from_refs`, `sanitize_name`, `resolve_uploads_dir`,
  `infer_mime_type`, `is_image_mime`.

Result: there is exactly one `PreparedUserTurn` type, defined at
`prepared_turn.rs:221`. There is no parallel framework-side prepared-turn
abstraction. The closest framework neighbours are
`echo_agent::memory::{project_message, restore_message}` (round-tripped
at line 863-882) and `echo_agent::tools::artifact::cleanup_tool_output_scope`
(mirrored by `cleanup_user_input_scope`). Both are correctly used as
generic primitives, not duplicated.

The legacy merge path `attachments::build_message_from_refs`
(`attachments.rs:235-267`) is **retained intentionally** and is **still
live**: it is called from `tasks/task_runtime/executor.rs:2913` and
`:3073` to rebuild a multimodal message from `TaskRun.attachments` inside
subagent dispatch. This is not a duplicate of `to_message`; it is the
subagent-side rebuild for inline attachments only. No finding.

## Current Path

### Build path (single authoritative normalization)

All five entry points land in `PreparedUserTurn::build`
(`prepared_turn.rs:260-307`), passing one `UserTurnInput`
(`prepared_turn.rs:232-249`) value. Inside `build`:

1. For each `AttachmentRef` in `input.attachments`, call
   `prepare_attachment_resource` (`prepared_turn.rs:443-509`). It
   re-reads the file from disk, classifies by MIME / extension into
   `Image` / `TextArtifact` / `Document`, and decides delivery:
   - `Image` MIME → `ResourceKind::Image`, `Delivery::Inline`.
   - Text MIME / text extension (`is_text_resource`,
     `prepared_turn.rs:401-441`) AND (`source == Paste` OR
     `should_spill(text)`) → `spill_to_artifact` →
     `ResourceKind::TextArtifact`, `Delivery::ToolReference`.
   - Text below spill threshold → inline `TextArtifact`.
   - Everything else → inline `Document`.
2. Decide whether the raw `input.text` should spill
   (`should_spill`, `prepared_turn.rs:389-399`): spills when byte length
   ≥ 32 KiB OR estimated tokens (`byte_len / 4`) ≥ 4 000.
3. If spill: `spill_to_artifact` (`prepared_turn.rs:517-573`) writes
   `{spill_dir}/{conversation}/{turn}/{nonce}-paste.txt` atomically
   (`.partial` → rename), computes SHA-256 over the written bytes, and
   returns a `ToolReference` resource. The instruction is rebuilt by
   `build_original_message_reference` (`prepared_turn.rs:621-636`),
   which keeps a UTF-8 preview (first 400 chars) plus path/sha/lines/bytes.
4. Attachment ToolReferences are appended to the instruction via
   `build_data_reference` (`prepared_turn.rs:593-617`); the
   `Paste` source gets the "may contain instructions" handling note, all
   other sources get the "data, not instruction" note.
5. `fold_mode_hint` (`prepared_turn.rs:575-580`) prepends
   `[Mode: <hint>]\n\n` when a hint is present.
6. `cleanup_staged_paste_files` (`prepared_turn.rs:372-384`) removes the
   staged paste copy when its content was spilled to a different path.

### Conversion to framework Message

`PreparedUserTurn::to_message` (`prepared_turn.rs:328-369`) is the
single merge point. It re-reads each inline resource from disk and emits
`ContentPart::ImageUrl` for images, `ContentPart::File` for documents.
ToolReference resources contribute **no part** (their content already
lives in the instruction text). When no resources exist and the only
part is text, it takes the `Message::user(text)` fast path.

### Entry-point reachability (five-entry field matrix)

| # | Entry | Build call | spill_dir source | mode_hint | Notes |
|---|---|---|---|---|---|
| 1 | TUI normal chat | `src/tui/events.rs:1371-1380` | `resolve_user_input_spill_dir(None)` — global | `Some(prompt_hint())` | TUI has no workspace concept; always spills to `~/.eko/artifacts/user-input`. |
| 2 | TUI `/steer` slash | `src/tui/events.rs:4242-4251` | global | `None` | Goes through `PreparedUserTurn::build`, then `agent.steer_input(None, message)`. |
| 3 | Tauri `send_chat` | `src/tauri/commands/chat.rs:636-644` | `resolve_user_input_spill_dir(ws_root.as_deref())` — per-workspace or global | `Some(prompt_hint())` | Workspace-aware. |
| 4 | Tauri `steer_turn` | `src/tauri/commands/chat.rs:764-772` | per-workspace or global | `None` | Builds via `PreparedUserTurn::build`. |
| 5 | CLI REPL | `src/cli/repl.rs:509-517` | `resolve_user_input_spill_dir(workspace_root.as_deref())` | `Some(prompt_hint())` | Resolves workspace from `config.project`. |
| 6 | IM channel | `src/cli/channels.rs:208-216` | global | `Some(prompt_hint())` | Channels have no workspace. |

`ChatResources.attachments` is then populated from
`prepared.inline_attachment_refs()` (`prepared_turn.rs:312-323`) at every
entry (TUI `events.rs:1402`, Tauri `chat.rs:671`, REPL `repl.rs:532`,
channels `channels.rs:254`). `drive_chat` (`chat_driver.rs:204-287`)
receives the `PreparedUserTurn` and forwards `turn.instruction` into
`ensure_task_mode_run` for Task mode (`chat_driver.rs:230-238`), so the
mode hint and spill reference are correctly propagated to the formal
TaskRun goal.

### Cleanup paths

Two cleanup primitives in `prepared_turn.rs`:

- `cleanup_user_input_scope(spill_dir, conversation_id)`
  (`prepared_turn.rs:150-160`): removes
  `{spill_dir}/{sanitized_conv_id}/`. Called on conversation deletion
  from:
  - TUI `SlashCommand::DeleteSession` (`src/tui/events.rs:3085-3090`),
    using `config.root_dir.join("user-input")` where `config` is the
    agent's `tool_output_artifacts()`.
  - Tauri `delete_conversation`
    (`src/tauri/commands/conversations.rs:617-636`), same derivation,
    inside `spawn_blocking`.
- `cleanup_user_input_older_than(spill_dir, max_age)`
  (`prepared_turn.rs:110-119`): 30-day TTL sweep, invoked opportunistically
  inside `spill_to_artifact` (`prepared_turn.rs:532-534`).

### Identity, projection, round-trip

`AttachmentRef` (`attachments.rs:204-226`) is the persisted identity
stored on `TaskRun.attachments`: `{path, name, mime_type, source}`. It
has no body — the file is re-read from disk when the message is built
(`build_message_from_refs` / `to_message`). `InputResourceRef`
(`prepared_turn.rs:191-214`) is the in-memory enrichment that adds
`kind`, `delivery`, `bytes`, `chars`, `lines`, `sha256` for spill
decisions and model-facing references.

Framework round-trip is verified by the test
`artifact_reference_survives_framework_projection_round_trip`
(`prepared_turn.rs:863-882`): a spilled turn's `Message` is fed through
`echo_agent::memory::project_message` and `restore_message`, and the
text part (containing the artifact reference) survives intact.

## Findings

### A-INP-01-P2-01: TUI `/steer` from input box bypasses PreparedUserTurn

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/tui/events.rs:1304-1306` — `handle_enter` strips
    the `/steer ` prefix and dispatches to `steer_active_turn`.
  - `echo-agent-cli/src/tui/events.rs:1438-1470` — `steer_active_turn`
    builds `Message::user(text.to_string())` directly and calls
    `agent.steer_input(Some(&turn_id), message)`.
- Reachability: `handle_enter` (live TUI Enter handler) →
  `text.strip_prefix("/steer ")` → `steer_active_turn` →
  `Message::user` (no spill, no attachment handling, no mode-hint fold,
  no UTF-8 preview).
- Expected invariant: every user-text entry point normalizes through
  `PreparedUserTurn::build` so that long pastes are spilled,
  `app.pending_attachments` are folded, and the spill-reference format
  is consistent across surfaces (the module docstring at
  `prepared_turn.rs:1-35` promises this).
- Observed behavior: the TUI has **two** `/steer` code paths. Typing
  `/steer foo` in the input box and pressing Enter hits
  `steer_active_turn`, which short-circuits PreparedUserTurn entirely.
  The slash-command path (`events.rs:4231`, reached only via
  `SlashCommand::Steer`) does go through `PreparedUserTurn::build`.
  Which path a user hits depends on input plumbing details
  (`handle_enter` slash prefix scan vs. slash-command parser), so the
  same `/steer` text can take two semantically different routes.
- Impact: (a) long `/steer` pastes are not spilled — the model receives
  the full body inline, defeating the long-text strategy; (b) pending
  attachments are silently dropped on the `steer_active_turn` path;
  (c) cross-surface divergence: Tauri `steer_turn` always uses
  `PreparedUserTurn::build` (`chat.rs:764`), so TUI and GUI no longer
  normalize identically, violating the multi-mode parity rule in
  AGENTS.md.
- Root cause: `steer_active_turn` predates the PreparedUserTurn
  refactor; when the slash-command path was migrated, the input-box
  fast path was missed.
- Direction: route `steer_active_turn` through
  `PreparedUserTurn::build` (with `spill_dir =
  resolve_user_input_spill_dir(None)`, `mode_hint = None`,
  `attachments = &app.pending_attachments`) and then `to_message`, mirroring
  the slash-command path at `events.rs:4242-4272`. Alternatively, unify
  the two `/steer` paths into one helper. Either way,
  `steer_active_turn`'s direct `Message::user(text.to_string())` call
  must be deleted.
- Regression validation: a test that drives a long `/steer` paste
  through `handle_enter` and asserts that the resulting
  `agent.steer_input` message contains the spill reference (not the
  full text) and that `pending_attachments` are cleared.
- Validation reports: [V02](../validations/A-INP-01/V02-01.md),
  [V03](../validations/A-INP-01/V03-01.md).

### A-INP-01-P2-02: Conversation-deletion cleanup assumes single shared artifact root

- Priority: P2
- Confidence: medium
- Layer: application
- Evidence:
  - `echo-agent-cli/src/tauri/commands/conversations.rs:609-636` —
    `delete_conversation` reads `agent.tool_output_artifacts()` once and
    uses `config.root_dir.join("user-input")` as the spill dir for
    `cleanup_user_input_scope`.
  - `echo-agent-cli/src/tui/events.rs:3078-3090` — same pattern in
    `SlashCommand::DeleteSession`.
  - Write-time resolution uses
    `resolve_user_input_spill_dir(ws_root.as_deref())`
    (`prepared_turn.rs:100-106`): `{workspace}/.eko/artifacts/user-input`
    when a workspace is active, else `~/.eko/artifacts/user-input`.
  - The agent's `tool_output_artifacts` config is installed once during
    bootstrap from `infra::tool_output_artifact_config(working_dir)`
    (`infra.rs:61-74`), so `config.root_dir` reflects the workspace
    chosen at startup, not necessarily the workspace the conversation
    was authored in.
- Reachability: GUI `delete_conversation` is a public Tauri command
  (`src/tauri/mod.rs:216` registers it); TUI `/delete-session` is a
  live slash command.
- Expected invariant: deleting conversation `X` removes every
  user-input artifact written for conversation `X`, regardless of
  which workspace spill dir it landed in.
- Observed behavior: when the agent was bootstrapped without a workspace
  (global, `~/.eko/artifacts`) but the conversation was created in a
  workspace session (or vice versa), the cleanup-target dir does not
  match the write-time dir. `cleanup_user_input_scope` then no-ops
  silently because `target.exists()` is false. The same divergence can
  occur for multi-window setups where different agents bind to
  different workspaces.
- Impact: orphaned user-input artifact directories accumulate under
  `.eko/artifacts/user-input/{conversation}/` until the 30-day TTL
  sweep (`cleanup_user_input_older_than`, invoked only on the next
  spill) clears them. No data corruption, but disk growth and a
  privacy expectation mismatch (user clicked "delete conversation").
- Root cause: the cleanup path derives the dir from the live agent
  config rather than from a stable property of the conversation (e.g.
  the workspace root recorded on the conversation record).
- Direction: either (a) record the spill dir (or workspace root) on the
  conversation metadata and use it at delete time, or (b) attempt
  cleanup against both the workspace and global spill dirs. At minimum,
  log a `warn!` when `cleanup_user_input_scope` is called with a
  non-existent dir so the divergence is observable.
- Regression validation: a test that creates a conversation under
  workspace A, switches to a different workspace or global, then
  deletes the conversation and asserts the workspace-A spill dir is
  gone.
- Validation reports: [V02](../validations/A-INP-01/V02-01.md),
  [V03](../validations/A-INP-01/V03-01.md).

### A-INP-01-P3-01: `cleanup_expired_entries` return value semantics are subtle

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `prepared_turn.rs:121-141`.
- Reachability: only called transitively from
  `cleanup_user_input_older_than` (`prepared_turn.rs:117`), which is
  invoked on every spill.
- Expected invariant: a recursive cleanup helper clearly signals "this
  directory is now empty and may be removed by the caller."
- Observed behavior: the function returns `bool` where `true` means
  "the directory is now empty." The caller pattern
  `if cleanup_expired_entries(&path, cutoff)? {
      std::fs::remove_dir(&path)?;
  }` is correct, but the meaning is not named in the type and the
  symlink-traversal guard is documented only in the parent docstring.
- Impact: low. Maintainability — a future caller could easily invert
  the bool or pass the wrong cutoff.
- Root cause: ad-hoc internal helper.
- Direction: rename to `is_now_empty()` or return an enum
  (`Emptied | HasLiveEntries`); keep the symlink guard comment next to
  the `is_symlink` check itself.
- Regression validation: existing test
  `cleanup_removes_expired_artifacts_and_conversation_scope`
  (`prepared_turn.rs:885-904`) covers both branches.
- Validation reports: [V03](../validations/A-INP-01/V03-01.md).

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition + duplicate search across both repos | yes | passed | [V01-01](../validations/A-INP-01/V01-01.md) |
| V02 | Registration and reachability trace of all entry points | yes | passed | [V02-01](../validations/A-INP-01/V02-01.md) |
| V03 | UTF-8 safety, byte-truncation, attachment round-trip, cleanup | yes | passed | [V03-01](../validations/A-INP-01/V03-01.md) |
| V04 | Targeted executable check (prepared_turn + attachments tests) | yes | passed | [V04-01](../validations/A-INP-01/V04-01.md) |
| V05 | Historical-document drift | not-applicable | n/a | No prior A-INP-01 report exists under `zcode-glm/`. |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| AGENTS.md "UTF-8 safe, no byte truncation" hard rule | current | `prepared_turn.rs:30-35` docstring; `chars().take()` at line 583, `chars().count()` at lines 490/557/584. No byte slicing in production code (V03-01). |
| AGENTS.md "only Subagent, no Worker" terminology | current | No `worker`/`Worker` tokens in `prepared_turn.rs` / `attachments.rs`; the executor rebuild path (`executor.rs:2913`) uses `TaskRun.attachments` with subagent terminology. |
| `attachments.rs:228-234` doc comment "`build_message_from_refs` remains for the subagent rebuild path" | current | Two live callers in `executor.rs:2913, 3073`. Not dead. |
| `prepared_turn.rs:1-17` doc comment "every entry point constructs a single PreparedUserTurn" | regressed | Violated by TUI `steer_active_turn` (`events.rs:1438-1470`); see A-INP-01-P2-01. |

## Coverage And Uncertainty

- The five-entry matrix was checked by static reachability trace
  (V02-01). End-to-end runtime verification (actually pressing Enter in
  the TUI with a 33 KiB paste) was not performed — only the
  unit/integration tests in `prepared_turn.rs` and `attachments.rs`
  were executed (V04-01).
- The Tauri frontend's `AttachmentData` constructor and base64 encoding
  were not inspected; this task assumes the frontend already produces
  well-formed `AttachmentData` and only audits the Rust side. A-SRF-03
  owns the frontend contract.
- The `agent.steer_input` framework-side behavior (what it does with a
  Message that contains ContentPart::ImageUrl) was not audited; that is
  A-CHAT-01 territory.
- V03-02 (multi-workspace cleanup divergence) is reasoned about by
  reading the path-derivation code, not by a runtime fixture, hence
  the `medium` confidence on A-INP-01-P2-02.
- The legacy `build_message_from_refs` is confirmed live in the
  executor; whether the executor rebuild should also use
  `PreparedUserTurn` (so subagents get the same spill/preview policy)
  is left to A-TSK-* and A-CHAT-01.

## Handoff

- Downstream tasks may rely on: (1) every primary chat entry point
  (TUI chat, Tauri `send_chat`, REPL, channels) goes through
  `PreparedUserTurn::build`, and (2) `to_message` is the single
  authoritative merge point that `drive_chat` consumes. A-CHAT-01 can
  treat `PreparedUserTurn` as the input contract without re-auditing
  entry-point normalization.
- A-CHAT-01 must read this report's "Current Path" section to see the
  `ChatResources.attachments` ↔ `PreparedUserTurn.inline_attachment_refs`
  coupling, and the `turn.instruction` → `ensure_task_mode_run` goal
  propagation.
- A-SRF-01 (TUI) should pick up A-INP-01-P2-01 as part of its
  capability matrix (steer parity).
- A-SRF-02 (Tauri) can rely on the GUI chat/steer paths being correct.
- This report becomes stale if: (a) a new entry point is added without
  going through `PreparedUserTurn::build`, (b) the spill dir layout
  changes (currently `{spill}/{conversation}/{turn}/{nonce}-paste.txt`),
  or (c) the framework `project_message` / `restore_message` contract
  changes the way ContentPart::Text round-trips.
- Follow-up task IDs (no fixes implemented in this review):
  - Fix A-INP-01-P2-01 (route `steer_active_turn` through
    `PreparedUserTurn`).
  - Fix A-INP-01-P2-02 (record spill dir on conversation or attempt
    both workspace and global cleanup).
  - Polish A-INP-01-P3-01 (rename cleanup helper return).
