# A-INP-01: Prepared user turn, attachments, and input artifacts

> Status: complete
> Reviewer: Codex primary reviewer (delegated static evidence independently sampled)
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: both source repositories clean before report creation; only
> Codex A-INP-01 reports were added

## Question

Do all EKO entry points normalize a user turn exactly once while preserving
Unicode, attachment identity, complete long-text artifacts, cleanup ownership,
typed errors, and equivalent display/model projections?

## Scope

- `echo-agent-app-core/src/prepared_turn.rs`, `attachments.rs`, request types,
  ChatResources, and the single `drive_chat` message merge.
- GUI regular and steer commands, TUI regular and steer dispatch, CLI REPL, and
  IM channel adapters.
- Input artifact workspace/global roots, model recovery through artifact tools,
  TaskRun/Subagent attachment projection, conversation deletion cleanup, and
  frontend draft-clearing behavior.
- Static test inventory for long/Unicode/empty input, attachment round-trip,
  error paths, cancellation/resource bounds, and deletion.

## Out Of Scope

- Composition-root/project-root divergence is already owned by
  [A-BOOT-01](A-BOOT-01.md), especially A-BOOT-01-P1-01. This report records
  the root each adapter passes but does not duplicate that finding.
- Conversation/message persistence as a whole, TaskRuntime artifact propagation
  beyond the attachment projection boundary, provider-specific multimodal wire
  encoding, UI visual design, and source fixes.
- Scheduler, cron, and background system prompts: they are not user attachment
  entry points.
- Cargo, rustc, tests, builds, dynamic fixtures, and network activity, all
  explicitly prohibited for this review.

## Inputs

- Root `AGENTS.md`; shared `README.md`, `REPORTING.md`, and exact A-INP-01 card
  in `TASKS.md`; Codex isolation protocol and report templates.
- Dependency [A-BOOT-01](A-BOOT-01.md), used only for composition/root context
  and finding deduplication.
- Current clean source at the commits above. No other reviewer directory was
  read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | The framework should continue to accept plain/multimodal `Message` values and provide artifact-read tools. No EKO upload directory, draft, conversation deletion, or UI admission policy belongs in `echo-agent`. |
| EKO product policy | Input normalization, uploads, long-paste threshold, workspace/global storage choice, draft retention, conversation ownership, mode hint, entry parity, and display projection are EKO application responsibilities. |
| Adapter boundary | GUI/TUI/CLI/channel adapters should only validate transport input, provide stable entry identities, call one PreparedUserTurn service, and translate its typed result. They must not independently skip resources, merge content, spill files, or own a second steer path. |
| Duplicate search | Searched both repositories for `PreparedUserTurn`, `UserTurnInput`, `AttachmentRef`, `InputResourceRef`, `build_message_from_refs`, multimodal construction, attachment staging, artifact spill, and every production caller. PreparedUserTurn is the entry authority; `build_message_from_refs` has a distinct reachable TaskRuntime Subagent-reconstruction role. |
| Migration deletion | After making the canonical service transactional, delete the early TUI `steer_active_turn` message constructor, batch `filter_map`/skip behavior, and caller-owned save-before-admission sequences. Do not add a second framework input model. |

`InputResourceRef` is application-specific and appropriately remains in
`echo-agent-cli`. A framework API is not considered dead because EKO does not
use it, and no framework capability is proposed for deletion here.

## Current Path

The shared application authority is:

```text
transport attachment bytes/path
  -> EKO uploads staging -> AttachmentRef(path, name, MIME, source)
  -> PreparedUserTurn::build(UserTurnInput)
       text -> inline instruction OR scoped full user-input artifact reference
       attachment -> InputResourceRef(Inline OR ToolReference)
       mode hint -> folded once into instruction
  -> inline_attachment_refs -> ChatResources -> TaskRun -> Subagent rebuild
  -> drive_chat -> PreparedUserTurn::to_message -> one framework Message
```

`PreparedUserTurn::build` is the single normalizer
(`prepared_turn.rs:251-307`) and `drive_chat_inner` performs the one normal
message projection (`chat_driver.rs:424-438`). Full long text is atomically
written beneath the shared artifact root with sha256 and a Unicode-safe preview
(`prepared_turn.rs:511-572`, `:582-635`); registered artifact tools can recover
it. This is a working positive path, not a finding.

### Entry field matrix

| Entry | Text | Attachments | Mode | Spill scope | Conversation / turn | Terminal merge |
|---|---|---|---|---|---|---|
| GUI regular | command `message` | base64 batch -> uploads refs | active mode | current workspace/global | optional conversation / message key | `drive_chat` -> `to_message` (`chat.rs:443-477`, `:633-680`) |
| TUI regular | queued text | pending durable refs | TUI mode | global | session conversation / UUID | `send_to_agent` -> `drive_chat` (`events.rs:1326-1435`) |
| CLI REPL | REPL text | locally staged refs | REPL mode | canonicalized project/global | configured/new conversation / UUID | spawned `drive_chat` (`repl.rs:475-544`) |
| IM channel | inbound text | inbound bytes -> uploads refs | shared channel mode | global | sender conversation / UUID | channel `drive_chat` (`channels.rs:190-229`) |
| GUI steer | command `message` | base64 batch -> uploads refs | none | current workspace/global | conversation / active turn | direct `to_message` -> `steer_input` (`chat.rs:733-780`) |
| TUI steer (live) | stripped text | **dropped/not taken** | none | **not used** | active turn | direct `Message::user` (`events.rs:1294-1306`, `:1438-1469`) |

The later TUI `SlashCommand::Steer` branch does build a PreparedUserTurn and
preserve attachments (`events.rs:4231-4303`), but the normal `/steer ` prefix is
intercepted before the slash dispatcher, so it is not the live typed route.

## Findings

### A-INP-01-P1-01: GUI and channel batches silently discard failed attachments while accepting the turn

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/attachments.rs:145`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/attachments.rs:174`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/chat.rs:458`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/chat.rs:751`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/cli/channels.rs:472`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/web-frontend/src/components/chat/ChatInput.tsx:633`
- Reachability: GUI regular/steer call `save_attachments`; channel handling
  calls `stage_channel_attachments`; both helpers log and omit failures, then
  the surviving refs or text-only turn reaches the live agent.
- Expected invariant: every user-selected attachment is either bound to the
  accepted turn or returned as an identified typed failure while the draft is
  retained.
- Observed behavior: invalid base64/name or disk failure is skipped. Callers do
  not compare requested/saved identities. GUI can return accepted, after which
  the frontend clears every pending file; channels send no item-level failure.
  TUI/CLI local staging instead propagates an error.
- Impact: a user can receive a plausible answer to an incomplete prompt while
  believing every image/file was analyzed; surface behavior is not equivalent.
- Root cause: the shared batch API encodes partial failure as a shorter success
  vector with logging as its only error channel.
- Direction: make the one staging API atomic by default or return an explicit
  batch result keyed by stable input ID; require a surface to retain/retry
  failures and visibly confirm deliberate partial send. Delete the skip/filter
  adapters after cutover.
- Regression validation: first/middle/last and all-item failures across
  GUI/steer/channel, Unicode names, disk errors, draft retention, and exact
  accepted-ID display/model equality.
- Validation reports: [V04](../validations/A-INP-01/V04-01.md)

### A-INP-01-P1-02: Attachment projections do not preserve content identity across repeated reads

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/attachments.rs:199`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/attachments.rs:228`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/prepared_turn.rs:309`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/prepared_turn.rs:328`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/prepared_turn.rs:443`
- Reachability: every attached normal turn is read during preparation, read
  again by `to_message`, and its downgraded AttachmentRef may be persisted on a
  TaskRun and read later by a Subagent.
- Expected invariant: one logical attachment denotes one immutable byte payload
  for display, the primary agent, persistence/restart, and all Subagents.
- Observed behavior: `AttachmentRef` has path/name/MIME/source but no stable ID,
  size, or hash. Inline `InputResourceRef` leaves sha256 empty. Its projection
  back to AttachmentRef drops bytes/kind/delivery/hash, while each consumer
  independently re-reads the mutable path.
- Impact: file mutation/deletion between stages can make the primary agent and
  Subagent see different bytes under the same name, or fail a previously
  accepted turn; persisted metadata cannot prove which content was analyzed.
- Root cause: filesystem location is treated as content identity and the
  richest transient type is projected into a poorer persisted type.
- Direction: stage once into immutable, content-addressed application storage;
  retain typed ID/hash/actual size through PreparedUserTurn, TaskRun, display,
  and Subagent adapters, verifying every later read. Remove lossy parallel
  projections after migration.
- Regression validation: mutate/delete/replace between prepare, `to_message`,
  restart, and Subagent reconstruction; assert exact hash/bytes or a typed
  identity error, never silent drift.
- Validation reports: [V01](../validations/A-INP-01/V01-01.md),
  [V05](../validations/A-INP-01/V05-01.md)

### A-INP-01-P1-03: The live TUI `/steer` route bypasses PreparedUserTurn and drops pending attachments

- Priority: P1
- Confidence: high
- Layer: adapter
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tui/events.rs:1294`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tui/events.rs:1438`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tui/events.rs:4231`
- Reachability: Enter dispatch checks `strip_prefix("/steer ")` before the
  general slash-command dispatcher, so normal typed steer always calls the
  early helper.
- Expected invariant: TUI steer uses the same long-text, attachment, identity,
  and typed error path as GUI steer, and preserves all resources when queued.
- Observed behavior: the early helper creates `Message::user(text)` directly.
  It neither takes pending attachments nor spills long input; NotSteerable
  queues `attachments: Vec::new()`. The later canonical branch is shadowed.
- Impact: TUI users cannot steer with attachments equivalently to GUI, long
  steer bypasses the full artifact path, and a queued fallback changes the
  submitted turn.
- Root cause: two TUI steer authorities exist at different dispatcher levels.
- Direction: route all steer commands through the PreparedUserTurn branch and
  delete `steer_active_turn`'s parallel message/queue construction.
- Regression validation: real key dispatch for typed and palette steer with
  long CJK/emoji, attachment-only/mixed input, accepted/not-steerable/mismatch,
  and queued exact identity.
- Validation reports: [V02](../validations/A-INP-01/V02-01.md),
  [V06](../validations/A-INP-01/V06-01.md)

### A-INP-01-P1-04: GUI persists uploads before turn admission and loses their ownership on early return

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/chat.rs:458`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/chat.rs:511`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/chat.rs:536`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/chat.rs:630`
- Reachability: every GUI regular send stages attachments before in-progress
  TaskRun detection, active-turn admission, cancellation registration, and
  PreparedUserTurn construction.
- Expected invariant: staging commits only with an admitted turn, or every
  rejection returns/refunds exact artifact ownership to a retained draft.
- Observed behavior: InterruptPrompt returns only new text; busy returns a
  validation error; preparation failure clears turn bookkeeping. None removes
  or returns the already-written upload refs.
- Impact: rejected/interrupted attempts leak files and lose the attachment
  association needed for faithful resume/retry.
- Root cause: persistence is a precondition step outside the turn admission and
  lifecycle transaction.
- Direction: admit/reserve the turn first, then stage under a turn-owned
  transaction with commit/rollback; alternatively return an explicit retained
  draft artifact set. Delete save-before-admission caller sequencing.
- Regression validation: interrupt prompt, busy collision, preparation/read
  failure, cancellation, process restart, and success with exact file/owner
  assertions.
- Validation reports: [V07](../validations/A-INP-01/V07-01.md)

### A-INP-01-P1-05: Conversation deletion cannot identify or remove uploaded attachment files

- Priority: P1
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/attachments.rs:52`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/attachments.rs:145`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/conversations.rs:585`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tui/events.rs:3067`
- Reachability: all saved GUI/channel/local attachments use the flat workspace
  or global uploads directory. GUI and TUI production deletion clean tool and
  user-input artifact scopes only.
- Expected invariant: deleting a conversation removes its private input
  artifacts, or reports a durable typed cleanup-pending/failure state.
- Observed behavior: upload filenames contain only a UUID and sanitized name;
  AttachmentRef carries no conversation/turn owner. Deletion therefore cannot
  enumerate them. GUI also detaches best-effort artifact cleanup and returns
  success before its outcome.
- Impact: original user-uploaded documents/images remain on local disk
  indefinitely after the user deletes the conversation, causing storage growth
  and violating deletion expectations.
- Root cause: upload storage has no lifecycle identity/manifest aligned with
  the conversation and turn scopes used by long-input artifacts.
- Direction: store uploads under stable conversation/turn identities or use an
  atomic ownership manifest/reference count; make deletion completion truthful
  and retryable. Delete the flat unowned layout once migrated.
- Regression validation: delete active/inactive conversations, shared/deduped
  content, restart, missing file, partial cleanup failure, and GUI/TUI parity.
- Validation reports: [V08](../validations/A-INP-01/V08-01.md)

### A-INP-01-P2-06: Synchronous unbounded preprocessing precedes effective cancellation

- Priority: P2
- Confidence: high
- Layer: application
- Evidence: `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/attachments.rs:65`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/attachments.rs:145`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/prepared_turn.rs:260`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/echo-agent-app-core/src/prepared_turn.rs:328`;
  `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/src/tauri/commands/chat.rs:556`
- Reachability: every attachment surface performs some combination of these
  helpers before the model call; GUI creates its CancellationToken only after
  save/admission work.
- Expected invariant: count/size-heavy input work is bounded or streamed and
  observes cancellation without monopolizing an async executor thread.
- Observed behavior: local staging fully reads and base64-encodes, then decodes
  and writes; preparation and `to_message` synchronously re-read and encode.
  No shared attachment count/byte limit or cancellation check covers these
  copies and disk operations.
- Impact: a large or many-file local input can cause high peak memory, delay the
  event loop, and remain uninterruptible until preprocessing completes.
- Root cause: synchronous transport conversion, persistence, normalization,
  and provider projection are fused without a bounded/cancellable I/O service.
- Direction: establish product-level count/size budgets with typed errors and
  use cancellable streaming/blocking I/O under the one turn lifecycle. This is
  local reliability control, not a cloud-style permission gate.
- Regression validation: bounded peak memory and cancel latency for one huge
  file, many files, CJK text, slow disk, and cancellation at every phase.
- Validation reports: [V09](../validations/A-INP-01/V09-01.md)

## Validation Matrix

| ID | Claim or command | Required | Status | Report |
|---|---|---:|---|---|
| V01 | Definition, duplicate search, and layering | yes | passed | [V01](../validations/A-INP-01/V01-01.md) |
| V02 | Five-entry field matrix and real reachability | yes | failed | [V02](../validations/A-INP-01/V02-01.md) |
| V03 | Long/Unicode/path-safe artifact behavior | yes | passed | [V03](../validations/A-INP-01/V03-01.md) |
| V04 | Attachment batch typed-error/partial-failure contract | yes | failed | [V04](../validations/A-INP-01/V04-01.md) |
| V05 | Attachment identity and projection round-trip | yes | failed | [V05](../validations/A-INP-01/V05-01.md) |
| V06 | TUI steer dispatcher and queue behavior | yes | failed | [V06](../validations/A-INP-01/V06-01.md) |
| V07 | Admission transaction and early-return ownership | yes | failed | [V07](../validations/A-INP-01/V07-01.md) |
| V08 | Conversation deletion cleanup | yes | failed | [V08](../validations/A-INP-01/V08-01.md) |
| V09 | Resource bounds and cancellation | yes | failed | [V09](../validations/A-INP-01/V09-01.md) |
| V10 | Existing-test coverage inventory | yes | passed | [V10](../validations/A-INP-01/V10-01.md) |
| V11 | Targeted executable regressions | conditional | not_run | [V11](../validations/A-INP-01/V11-01.md) |
| V12 | Exact-ID/link/header/source-clean integrity gate | yes | passed | [V12](../validations/A-INP-01/V12-01.md) |
| V30 | Primary source sampling and acceptance | yes | passed | [V30](../validations/A-INP-01/V30-01.md) |

## Historical Claim Status

| Source claim | Classification | Current evidence |
|---|---|---|
| `prepared_turn.rs:3-6`: every GUI/TUI/CLI/channel/steer entry constructs PreparedUserTurn | regressed | Live TUI prefix interception bypasses the later prepared steer branch; [V02](../validations/A-INP-01/V02-01.md) |
| `prepared_turn.rs:18-28`: full long text is referenced and recoverable through artifact tools | current | Shared artifact root and tool reachability; [V03](../validations/A-INP-01/V03-01.md) |
| `prepared_turn.rs:30-35`: reviewed previews/truncation are UTF-8 safe | current | Character-based previews and CJK/emoji test inventory; [V03](../validations/A-INP-01/V03-01.md) |
| `attachments.rs:174-177`: attachment failures are skipped so the message proceeds | current defect | Caller behavior confirms silent partial acceptance; [V04](../validations/A-INP-01/V04-01.md) |
| `prepared_turn.rs:143-149`: conversation deletion removes user-input scope | current but incomplete | Scoped long-input cleanup exists; flat uploads are outside it; [V08](../validations/A-INP-01/V08-01.md) |

## Coverage And Uncertainty

- Static evidence is source-conclusive for the call order, types, fields, and
  missing cleanup ownership. No runtime timing, memory peak, filesystem fault,
  or UI interaction was measured.
- Existing unit tests were inspected only. Per explicit instruction, no Cargo,
  rustc, frontend, or fixture command was run; V11 is `not_run`. Primary static
  sampling is recorded in V30 and the task is accepted as `complete`.
- MIME-content validation and provider-specific file support were observed but
  not expanded into findings; they belong with provider/wire reviews unless an
  entry-level user impact is reproduced.
- Backend `PreparedUserTurn` can represent empty text with resources, which is
  necessary for attachment-only turns. Surface-specific blank-text policy was
  not treated as a defect without a concrete failed path.
- GUI/TUI project-root selection can change where artifacts land, but that
  causal root is already A-BOOT-01-P1-01 and is deliberately not duplicated.

## Handoff

- V30 independently sampled V02, V04, V05, V07, and V08 and accepted the six
  findings from current source.
- Iteration order should be: establish stable immutable attachment identity and
  transaction ownership; make batch errors explicit; unify steer dispatch;
  scope cleanup; then stream/bound preprocessing. This keeps one application
  authority and avoids a parallel migration path.
- Surface-parity synthesis should consume A-INP-01-P1-01 and P1-03. State/data
  lifecycle synthesis should consume P1-04 and P1-05. Task artifact propagation
  should consume P1-02 without creating a second TaskRun attachment model.
- This report becomes stale if PreparedUserTurn/AttachmentRef fields, any entry
  constructor, uploads layout, conversation deletion, TaskRun attachment
  projection, or `drive_chat` merge behavior changes.
