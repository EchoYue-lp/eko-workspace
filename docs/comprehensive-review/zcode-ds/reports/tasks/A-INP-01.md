# A-INP-01: Prepared user turn, attachments, and input artifacts

> Status: complete
> Reviewer: ZCode-ds
> Review date: 2026-08-12
> `echo-agent` commit: 9b0e0faf74d35c9a432370b923acabfbb5f32d63
> `echo-agent-cli` commit: b3b2e81f2b2d9fdb319ec604a561beec5f66fea5
> Worktree state: both repositories clean

## Question

Do all entry points normalize user input once while preserving Unicode,
attachment identity, long text artifacts, cleanup, and display/model
projections?

## Scope

- `echo-agent-cli/echo-agent-app-core/src/prepared_turn.rs` (full,
  951 lines): `PreparedUserTurn`, `UserTurnInput`, spill decision and write,
  reference blocks, previews, cleanup helpers, path sanitization, tests.
- `echo-agent-cli/echo-agent-app-core/src/attachments.rs` (full):
  `save_attachment(s)`, `sanitize_name`, `AttachmentRef`,
  `build_message_from_refs`, MIME inference, tests.
- Entry-point adapters: `src/tauri/commands/chat.rs` (GUI send + steer),
  `src/tui/events.rs` (TUI send + /steer + /delete-session),
  `src/cli/repl.rs` (CLI REPL), `src/cli/channels.rs` (IM channel),
  `src/tauri/commands/conversations.rs` (GUI delete_conversation).
- Shared driver consumption: `echo-agent-app-core/src/chat_driver.rs`
  (`drive_chat`/`drive_chat_inner`), `chat_resources.rs`.
- Framework confinement contracts the reference depends on:
  `echo-agent/echo-tools/src/files/artifact.rs` (read_artifact root),
  `echo-agent/echo-tools/src/files/grep.rs` (allowed-root set).
- Spill-dir/artifact-root wiring: `infra.rs:61-71`, `workspace/layout.rs:93-101`,
  `state.rs:837-891` (workspace switch), `agent_pool.rs:530-559`
  (apply_working_dir), boot sites `src/main.rs:162`, `src/tauri/desktop.rs:151`.

## Out Of Scope

- Framework tool-output artifact mechanism (`ToolOutputArtifactWriter`,
  snapshot spill, read_artifact pagination internals) — F-EXT-01/F-RCT-05.
- Steering mailbox correctness — framework F-RCT-05 (its P1-02 covers the
  steer-drop class; this report only records the adapter-level identity
  divergence, P3-04).
- Frontend attachment rendering/state — A-SRF-03/A-FE-01; conversation
  persistence formats — A-STATE-01; TaskRun attachment propagation beyond the
  rebuild call sites — A-TSK-06.

## Inputs

- Root `AGENTS.md` (full), shared `README.md`, `REPORTING.md`, `TASKS.md`
  (A-INP-01 card), `zcode-ds/README.md`, report templates.
- Dependency report: `A-BOOT-01` (zcode-ds track) — used for boot/wiring
  facts (`working_dir: None` at all bootstrap call sites; workspace switch
  updates agent artifact config) and to avoid duplicating its findings.
- Historical documents treated as hypotheses: `docs/MASTER-PLAN.md` (Phase E),
  `echo-agent-cli/docs/2026-07-28-app-core-full-audit.md`,
  `echo-agent-cli/docs/2026-07-16-tool-output-artifacts.md`.

## Layering Decision

- Generic mechanism (framework): the artifact-confinement contracts that the
  user-input reference depends on — `read_artifact` root check
  (artifact.rs:86-115) and grep's candidate-root set (grep.rs:148-198) — are
  generic and already framework-owned; no movement needed. The application
  reuses them as-is (MASTER-PLAN Phase 3 did exactly this).
- EKO product policy (application): spill thresholds
  (`SPILL_THRESHOLD_BYTES`, `ESTIMATED_TOKEN_THRESHOLD`), user-input artifact
  layout, reference-block wording (including the Paste-vs-upload instruction
  semantics), mode-hint folding, 30-day TTL, conversation-scoped cleanup,
  per-entry spill-dir resolution. All live in app-core `prepared_turn.rs` /
  `attachments.rs` — correctly placed.
- Adapter boundary (entry-point adapters): six `UserTurnInput` constructions
  are thin and uniform (V02-01). Two adapters violate the thin-adapter rule:
  (1) TUI `/steer` re-uses already-consumed attachment refs after build
  (P1-01 — consume-then-reuse instead of re-queueing the prepared turn);
  (2) CLI REPL resolves the spill dir from `config.project` while the agent
  artifact root stays global (P2-01 — the adapter decides a workspace scope
  the runtime does not have).
- Duplicate search (terms + results in V01-01): `PreparedUserTurn`,
  `UserTurnInput`, `resolve_user_input_spill_dir`, `cleanup_user_input_scope`,
  `cleanup_user_input_older_than`, `user-input`, `spill`, `[Mode:`,
  `build_message`, `match multimodal`, `ContentPart`, `AttachmentRef`,
  `AttachmentData`, `AttachmentSource`, `path_component`, byte-slice `[..`.
  One definition per concept; no parallel normalization implementation in
  either repository; framework "spill" hits are the separate tool-output
  mechanism. No `worker` terminology.

## Current Path

Verified data flow (V02-01):

1. Each surface stages attachments to disk first: GUI/steer via
   `save_attachments` -> `{ws}/.eko/uploads/` or `~/.eko/uploads/`
   (chat.rs:465-467, 753-756); TUI via `stage_attachment`/`handle_pasted_text`
   (events.rs:1510-1543, 2540-2557); channel via `stage_channel_attachments`
   (channels.rs:472-489); CLI via `stage_local_attachment`.
2. Every surface then calls the single `PreparedUserTurn::build`
   (6 call sites) with the uniform six-field `UserTurnInput`; build:
   (a) converts each attachment to an `InputResourceRef` (image -> inline;
   text -> inline unless source==Paste or over threshold -> spill), (b) spills
   long instruction text to `{spill}/{conversation}/{turn}/{nonce}-paste.txt`
   (atomic .partial->rename, SHA-256, 400-char UTF-8-safe preview), (c) folds
   the mode hint, (d) removes the staged paste copy after a successful spill
   (`cleanup_staged_paste_files`).
3. `to_message()` is the single merge point into one framework `Message`
   (inline resources as `ContentPart::ImageUrl`/`File`; spilled text as a
   reference block inside the instruction).
4. `drive_chat` consumes the turn: `turn.instruction` becomes the Task-mode
   goal; `res.attachments` (= `inline_attachment_refs`) is bound to
   TaskRun.attachments; subagents rebuild the message from refs
   (executor.rs:2913/3073); steer paths call `steer_input` instead.
5. Deletion: GUI `delete_conversation` (conversations.rs:597-649) and TUI
   `/delete-session` (events.rs:3077-3095) remove tool-output scope and
   user-input scope; spills also expire after 30 days.

## Findings

### A-INP-01-P1-01: TUI `/steer` on a non-steerable turn destroys the staged paste file; the queued re-send then fails and the pasted content is lost from disk

- Priority: P1
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/tui/events.rs:4240` — `/steer` takes the pending
    attachments (`mem::take(&mut app.pending_attachments)`).
  - `echo-agent-cli/src/tui/events.rs:4242-4256` — builds a
    `PreparedUserTurn` from those refs; on success
    `prepared_turn.rs:302` runs `cleanup_staged_paste_files`, which deletes
    the staged Paste copy when the paste was spilled to an artifact
    (prepared_turn.rs:372-384, `std::fs::remove_file` at :380).
  - `echo-agent-cli/src/tui/events.rs:4286-4294` — when
    `steer_input(None, message)` returns NoActiveTurn / NotSteerable /
    TurnMismatch, the handler re-queues the RAW refs (now dangling) plus the
    raw instruction into `app.queued_turns`.
  - `echo-agent-cli/src/tui/events.rs:1371-1400` — the queued turn later
    re-enters `dispatch_turn`, `build` fails on the deleted file
    (`PreparedTurnError::Read`), the turn is dropped with a system message,
    and `app.pending_attachments` keeps the dangling refs so every retry
    fails until the user clears them.
- Reachability: `Event::Paste` for text >= 1,000 chars
  (`PASTE_ATTACHMENT_CHAR_THRESHOLD`, events.rs:32) stages a Paste-source
  attachment (events.rs:1510-1543); `/steer` while the agent is mid-turn and
  not steerable is a normal live state; the queued re-dispatch executes on
  the next Enter. The paste file is deleted during the failed steer's build,
  before the failure is known.
- Expected invariant: user input, once staged, must survive a failed steer and
  be re-deliverable; preparation over the same refs must be idempotent
  (or refs must only be consumed when delivery succeeds).
- Observed behavior: the staged paste is deleted during the steer attempt; the
  queued follow-up fails with "failed to prepare user turn ... failed to read
  input resource", the follow-up instruction is dropped, and the pasted body
  exists afterwards only in the orphaned user-input artifact (referenced by
  the never-delivered steer message).
- Impact: user content loss on a live TUI path — a >=1,000-char paste plus a
  `/steer` follow-up while the turn is not steerable silently destroys the
  staged copy and drops the queued message; the user must re-paste.
- Root cause: the steer handler re-queues pre-build attachment refs after
  `build` has already spilled and deleted them; the queue path assumes `build`
  is idempotent over the same refs, but it is consume-once.
- Direction: on steer failure, re-queue the PREPARED turn (instruction +
  `InputResourceRef`s, which include the durable artifact path) instead of the
  raw refs, or defer `cleanup_staged_paste_files` until the message is
  actually delivered; alternatively re-spill from the artifact. TUI only — GUI
  steer returns an error without re-queuing and the artifact retains the
  content.
- Regression validation: TUI integration test — paste >=1,000 chars, `/steer`
  while NotSteerable, then send a follow-up and assert (a) no
  "failed to prepare user turn" error, (b) the pasted content is reachable via
  the artifact, (c) no dangling pending attachments.
- Validation reports: [V02-01](../validations/A-INP-01/V02-01.md),
  [V03-01](../validations/A-INP-01/V03-01.md)

### A-INP-01-P2-01: CLI REPL with `--project` spills long pastes outside the agent's artifact root — read_artifact/grep refuse the reference, long-text recovery silently broken

- Priority: P2
- Confidence: high
- Layer: application
- Evidence:
  - `echo-agent-cli/src/cli/repl.rs:504-510` — `chat_with_agent` resolves the
    spill dir from `config.project` (workspace_root), and
    `repl.rs:118-127` loads the explicit `--project` path.
  - `echo-agent-cli/src/main.rs:162` (also desktop.rs:151, main.rs:493) —
    `AgentRuntime::bootstrap` is called with `working_dir: None`, so the agent
    artifact config root is the global `~/.eko/artifacts`
    (echo-agent-app-core/src/infra.rs:63-71). Nothing in the CLI boot path
    calls `set_tool_output_artifacts` / `apply_working_dir` (production callers
    are GUI-only: state.rs:879-881/890/1167; agent_pool.rs:534-559 is otherwise
    test-only).
  - `echo-agent/echo-tools/src/files/artifact.rs:86-115` —
    `resolve_artifact_path` rejects absolute paths outside the configured
    root ("outside the configured artifact root").
  - `echo-agent/echo-tools/src/files/grep.rs:148-198` — grep's allowed roots
    are base_dir, working_dir, output_artifacts.root_dir; the same absolute
    path is rejected ("outside the allowed directory scope").
- Reachability: `eko --cli --project <dir>` is a live CLI flag
  (repl_config_for, modes.rs:12-19); any text >= 32 KiB or >= 4,000 estimated
  tokens (prepared_turn.rs:389-399) is spilled to
  `{project}/.eko/artifacts/user-input/...`; the model then receives only the
  reference block (path + sha256 + 400-char preview) and cannot read the file.
- Expected invariant: spilled user-input artifacts are always inside the
  agent's artifact root so `read_artifact`/`grep` can recover them — the
  invariant the GUI maintains by updating both together on workspace switch
  (state.rs:876-881) and the design comment at infra.rs:66.
- Observed behavior: on the REPL+`--project` path the spill dir is
  workspace-scoped while the artifact root stays global; every recovery tool
  rejects the reference path.
- Impact: long-paste handling silently degrades to a 400-char preview on the
  CLI surface when a project is set; the model cannot analyze the full paste,
  and a `read_artifact` attempt returns an error mid-turn. No error is
  surfaced at send time.
- Root cause: the REPL adapter decides a workspace-scoped spill dir without
  the runtime knowing a workspace (bootstrap gets `working_dir: None`); the
  two settings are updated together only in the GUI's `switch_workspace`.
- Direction: in the CLI boot path, when `--project` is set, apply the project
  root to the agent (working_dir + `tool_output_artifact_config(Some(root))`)
  using the same helper as state.rs:879-881 — this also fixes file-tool CWD
  semantics — or, failing that, spill to the global dir when the agent artifact
  root is global. Coordinate with A-CFG-01 (project/workspace lifecycle).
- Regression validation: CLI REPL with `--project`, paste >= 32 KiB; assert
  `read_artifact` on the reference path succeeds and `grep` of the spilled
  content succeeds (compare with GUI workspace behavior).
- Validation reports: [V02-01](../validations/A-INP-01/V02-01.md),
  [V03-01](../validations/A-INP-01/V03-01.md)

### A-INP-01-P3-01: uploads directory has no retention or deletion cleanup

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `attachments.rs:150-172` writes `{uuid}_{name}` files with no
  conversation scope; GUI/TUI deletion cleans only tool-output and user-input
  scopes (conversations.rs:617-628, events.rs:3078-3095); repository-wide grep
  for uploads cleanup (remove/clean/delete) — zero hits; no TTL equivalent of
  `cleanup_user_input_older_than`.
- Reachability: every attachment upload/paste staging on every surface; files
  persist for the app lifetime and after conversation deletion.
- Expected invariant: staged upload files are either conversation-scoped for
  deletion or TTL-expired like user-input artifacts (prepared_turn.rs:110-119).
- Observed behavior: `~/.eko/uploads/` and `{ws}/.eko/uploads/` accumulate
  uuid-named copies forever; attachments of deleted conversations are never
  removed.
- Impact: unbounded disk growth for attachment-heavy use; orphaned copies of
  possibly sensitive pasted/uploaded content persist after deletion.
- Root cause: uploads were designed as a shared staging area without a
  retention owner; the 30-day cleanup was only added for the user-input scope.
- Direction: add a TTL cleanup for the uploads dir mirroring
  `cleanup_user_input_older_than`, or scope upload paths per conversation so
  the deletion cascade (A-STATE-01) can remove them.
- Regression validation: upload -> delete conversation -> assert upload file
  removed (or TTL fixture: file older than TTL is removed).
- Validation reports: [V03-01](../validations/A-INP-01/V03-01.md)

### A-INP-01-P3-02: `path_component` collapses non-ASCII conversation ids to `_` — cross-conversation spill collision and cross-conversation deletion

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `prepared_turn.rs:642-658` (`path_component` replaces every
  non-ASCII-alphanumeric char with `_` and collapses all-`_`/all-dot results to
  a single `_`); test `path_component_sanitizes_separators`
  (prepared_turn.rs:907-916) asserts `path_component("会话一") == "_"`;
  `cleanup_user_input_scope` (prepared_turn.rs:150-160) removes the whole
  `{spill}/{conv}` subtree for that id.
- Reachability: latent — current conversation ids are ASCII (uuid,
  `channel:{channel}:{sender}`), so the `_` bucket is unused today; any
  future user-supplied non-ASCII id (e.g., CLI `--conversation-id` or channel
  names) enters the shared bucket.
- Expected invariant: spill artifacts are namespaced per conversation id
  (identity preservation); deleting one conversation removes only its
  artifacts.
- Observed behavior: two distinct non-ASCII conversation ids both map to `_`;
  their spills are mixed, and deleting either conversation deletes the other's
  artifacts.
- Impact: wrong deletion cascade and mixed artifact scopes once non-ASCII ids
  occur; silent today.
- Root cause: sanitize-to-nothing collapses to a constant instead of
  preserving uniqueness (e.g., hash/hex suffix).
- Direction: append a short hash of the raw id when the cleaned component
  collapses, so identity is preserved while remaining path-safe; add a test
  with two CJK ids asserting distinct directories and isolated deletion.
- Regression validation: unit test — two non-ASCII ids produce distinct spill
  dirs; `cleanup_user_input_scope(id1)` leaves id2's artifacts intact.
- Validation reports: [V03-01](../validations/A-INP-01/V03-01.md)

### A-INP-01-P3-03: GUI `send_chat_message` and the channel path have no backend empty-input gate

- Priority: P3
- Confidence: high
- Layer: application
- Evidence: `src/tauri/commands/chat.rs:444-460` — `send_chat_message` never
  checks `message.trim().is_empty()` (the steer command does, chat.rs:741);
  `src/cli/channels.rs:208` builds the turn regardless of text; TUI and CLI
  REPL skip empty lines (events.rs:1322, repl.rs:221-223). Frontend gates
  empty sends (web-frontend/src/components/chat/ChatInput.tsx:621-622, :765).
- Reachability: any Tauri IPC caller (or future non-frontend client) can send
  an empty message; the turn then delivers `Message::user("")` to the model.
- Expected invariant: the backend rejects empty turns at every entry point
  (defense in depth; the other surfaces already do).
- Observed behavior: an empty GUI message produces an empty user message,
  a wasted model call, and an empty transcript entry; empty channel text is
  forwarded as a turn.
- Impact: minor — degraded transcripts and wasted calls; frontend mitigates the
  GUI case today.
- Root cause: the empty-input guard was added to the steer commands and the
  TUI/REPL loops but not to the GUI send command or the channel handler.
- Direction: mirror the steer guard (chat.rs:741) in `send_chat_message`
  (return `IpcError::Validation` when text is empty and no attachments) and
  skip empty channel messages in channels.rs.
- Regression validation: direct IPC call with empty message -> Validation
  error; empty channel message -> no turn, no model call.
- Validation reports: [V03-01](../validations/A-INP-01/V03-01.md)

### A-INP-01-P3-04: steer identity propagation diverges between surfaces — GUI passes `Some(expected_turn_id)`, TUI passes `None`

- Priority: P3
- Confidence: medium
- Layer: adapter
- Evidence: `src/tauri/commands/chat.rs:771-777` —
  `steer_input(Some(&expected_turn_id), steer_message)` (turn id from
  `active_chat_turns`); `src/tui/events.rs:4271` —
  `steer_input(None, message)`; framework entry
  `echo-agent/src/agent/handle.rs:187` forwards the id to the turn-steer
  mailbox (react/mod.rs:271-274). Framework steering defect class already
  tracked as F-RCT-05-P1-02 (mailbox lease keyed by turn_id vs drained by
  current_run_id).
- Reachability: both steer paths are live; the TUI path can reach the mailbox
  with no expected turn.
- Expected invariant: same input semantics per surface (AGENTS.md surface
  parity); steering targets the intended turn identically.
- Observed behavior: the GUI constrains the mailbox lookup to the expected
  turn; the TUI leaves the mailbox to fall back on its own current-turn
  resolution — two different steering contracts for the same product action.
- Impact: steering behavior (which turn accepts the injected message) can
  differ between GUI and TUI; interacts with F-RCT-05-P1-02.
- Root cause: TUI `/steer` predates the GUI's expected-turn plumbing and was
  never aligned.
- Direction: pass the active turn id in the TUI `/steer` call
  (`app.active_turn_id`), mirroring the GUI; validate against the F-RCT-05
  fix before landing.
- Regression validation: TUI steer during an active turn delivers to the
  active turn only; GUI steer behavior unchanged.
- Validation reports: [V02-01](../validations/A-INP-01/V02-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition and duplicate normalization search (both repos) | yes | passed | [V01-01](../validations/A-INP-01/V01-01.md) |
| V02 | Registration and runtime reachability (six-call-site call graph) | yes | passed | [V02-01](../validations/A-INP-01/V02-01.md) |
| V03 | Invariant/edge cases (field matrix, long/Unicode/empty, round-trip, deletion cleanup, UTF-8 truncation) | yes | passed | [V03-01](../validations/A-INP-01/V03-01.md) |
| V04 | Targeted executable check | yes | passed | [V04-01](../validations/A-INP-01/V04-01.md) (`prepared_turn` tests, exit 0), [V04-02](../validations/A-INP-01/V04-02.md) (`attachments` tests, exit 0), [V04-03](../validations/A-INP-01/V04-03.md) (`cargo check -p echo-agent-app-core --locked`, exit 0) |
| V05 | Historical-document drift | yes | passed | [V05-01](../validations/A-INP-01/V05-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `MASTER-PLAN.md`:291-297 Phase E layering + Phase 1 (PreparedUserTurn, InputResourceRef, 32 KiB threshold, UTF-8-safe preview, atomic write, SHA-256) | current | prepared_turn.rs matches (V01-01, V04-01) |
| `MASTER-PLAN.md`:302-306 Phase 2 — five entries + steer switched, `to_message` single merge, `build_message` deleted, `build_message_from_refs` kept for executor rebuild | current | six build call sites (V02-01); executor.rs:2913/3073 |
| `MASTER-PLAN.md`:310-311 Phase 3 — grep candidate-root extension (framework 8fd9b6a), session deletion cleanup of user-input scope | current | grep.rs:148-198; conversations.rs:617-628, events.rs:3078-3095 |
| `MASTER-PLAN.md`:313-314 deferred — persistence de-data-URL | current (deferred as documented) | frontend still renders data URLs; A-STATE-01 scope |
| `docs/2026-07-28-app-core-full-audit.md`:182 attachment data URLs as separate concern | current | consistent with the deferral |
| `docs/2026-07-16-tool-output-artifacts.md` spill design (framework) | current | generic mechanism unchanged (F-EXT-01) |
| No document claims uploads cleanup or REPL project-scoped spill | n/a (new observations) | A-INP-01-P2-01, P3-01 are new defects, not regressions |

## Coverage And Uncertainty

- No process was launched end to end; the P1-01 chain (steer -> queue ->
  re-dispatch -> Read error) and the P2-01 chain (spill outside root ->
  read_artifact rejection) are static call-graph proofs; both need dynamic
  confirmation (Q-E2E-01 scenarios for TUI steer and CLI --project long
  paste).
- The GUI deletion cleanup assumes the agent artifact config matches the
  workspace the conversation was spilled under; a conversation deleted after a
  workspace switch is cleaned against the current workspace root — the
  previous workspace's user-input files then expire only via the 30-day TTL.
  Edge case, left in uncertainty (A-STATE-01 owns the deletion cascade).
- GUI conversations across workspaces, and whether conversation ids are ever
  non-ASCII in practice, were not verified against the frontend — P3-02
  remains latent until A-SRF-03/A-STATE-01 confirm the id generators.
- The channel empty-message behavior and IM transport text normalization are
  framework channel surfaces (F-INT-02); only the adapter entry was checked.
- `prepared_turn`/`attachments` tests (18 total) all pass; the GUI bin and
  channels feature were not compiled in this task (Q-GUI-01/Q-CLI-01).

## Handoff

- Downstream tasks may rely on: single `PreparedUserTurn::build` normalization
  with six live call sites (one per send/steer surface); single
  `to_message` merge point; UTF-8-safe truncation end to end; GUI/TUI deletion
  cleanup covering tool-output + user-input scopes; TaskRun attachment
  propagation via `inline_attachment_refs` + executor rebuild.
- Reports to read: this report + A-BOOT-01 (boot wiring) + V01-V05.
- Findings to own elsewhere: A-INP-01-P2-01 should be confirmed/merged with
  A-CFG-01 (project/workspace lifecycle); P3-01 deletion cascade with
  A-STATE-01; P3-04 steering with F-RCT-05-P1-02; X-SRF-01 should add rows for
  per-surface spill-dir/artifact-root consistency and steer identity
  propagation; Q-E2E-01 should include the TUI paste+steer scenario (P1-01)
  and the CLI --project long-paste scenario (P2-01).
- This report becomes stale if `prepared_turn.rs`, the six adapter call sites,
  `cleanup_user_input_*`, `tool_output_artifact_config` wiring
  (infra.rs:61-71, state.rs:876-881), or the framework confinement
  (artifact.rs:86-115, grep.rs:148-198) change.
