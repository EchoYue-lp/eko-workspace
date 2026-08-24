# F-INT-02: LSP, channels, and A2A integrations

> Status: complete
> Reviewer: Codex primary reviewer (delegated static evidence independently sampled)
> Review date: 2026-08-13
> `echo-agent` commit: `3aa7929928442aab91e4dce9c426d909a5f0a1ab`
> `echo-agent-cli` commit: `b3b2e81f2b2d9fdb319ec604a561beec5f66fea5`
> Worktree state: both source repositories clean; the external transition from
> `9b0e0faf74d35c9a432370b923acabfbb5f32d63` to the current framework commit
> changed only ReAct testing-credibility paths and did not touch this task's scope

## Question

Do LSP, IM channel, and A2A adapters isolate external protocols while preserving typed internal lifecycle, message identity, delivery, and cleanup?

## Scope

- Full framework IM channel trait/type/session/manager path, QQ and Feishu adapters, feature/export boundaries, and real EKO channel-mode composition.
- Static lifecycle, malformed input, retry/dedup, cancellation/cleanup, typed error, UTF-8, panic, and overflow inspection.
- Deduplicated LSP/A2A coverage against the canonical F-INT-01 findings.

## Out Of Scope

- Source fixes and all Cargo/rustc/test/build/dynamic fixture/network execution.
- Re-reporting F-INT-01's LSP timeout/cancel/status/restart/framing/wire/document findings or A2A cancellation/terminal/identity/client-stream findings.
- Generic Tool registry/schema/result findings owned by F-EXT-01 and canonical Agent event semantics owned by F-CORE-01.
- EKO UI polish beyond proving the channel entry and group/session identity impact.

## Inputs

- Root `AGENTS.md`; shared `README.md`, `REPORTING.md`, and exact F-INT-02 task card; Codex protocol.
- Allowed Codex dependencies [F-INT-01](F-INT-01.md), [F-CORE-01](F-CORE-01.md), and [F-EXT-01](F-EXT-01.md).
- Current framework/application source. No other reviewer directory was read.

## Layering Decision

| Classification | Decision |
|---|---|
| Generic mechanism | Protocol DTO conversion, channel identity, session lifecycle, retry/dedup, framing bounds, cancellation/cleanup, and typed delivery/error reporting are reusable framework responsibilities. |
| EKO product policy | Enabled channel selection, user configuration, full Agent bootstrap, Task/HITL projection, and presentation remain application concerns. |
| Adapter boundary | QQ/Feishu adapters translate wire identity to canonical `InboundMessage`/`OutboundMessage` and must preserve delivery truth without duplicating Agent state machines. |
| Local threat model | No cloud/multi-tenant permission gate is recommended. Raw signature correctness, secret redaction, bounded malformed input, data isolation, and cleanup remain valid local correctness protections. |
| Duplicate search | Searched channel definitions/features/exports/manager/session/plugins and all live callers in both repositories; checked LSP/A2A only against the allowed canonical ownership report. |

## Current Paths

```text
EKO --channels
  -> run_channels_mode -> ChannelManager
  -> QqChannel | FeishuChannel
  -> SessionHandler -> AppChannelMessageHandler -> AgentPool/drive_chat
  -> plugin send queue -> QQ/Feishu HTTP API

standalone framework consumer
  -> root channels facade -> ChannelManager/SessionHandler
  -> AgentChannelHandler -> ReactAgent

LSP/A2A
  -> root feature/facade paths already behavior-reviewed by F-INT-01
```

## Findings

### F-INT-02-P1-01: ChannelManager cannot identify startup failures or guarantee cleanup

- Priority: P1; confidence: high; layer: framework.
- Evidence/reachability: `echo-agent/echo-integration/src/channels/manager.rs:11`, `:33`, `:47`, `:95`, `:133`; live EKO caller `echo-agent-cli/src/cli/modes.rs:217`.
- Expected: identity-keyed lifecycle results, explicit partial-start policy, observable stop failures, safe replacement, and truthful Drop behavior.
- Observed/impact: unordered start results have no channel IDs, partial successes remain live, duplicate registration silently replaces a plugin, `stop_all` swallows all errors, and the promised auto-stop Drop only logs. EKO cannot name failures and can report all channels started/closed when that is false.
- Root cause/direction: the manager stores plugins but no authoritative lifecycle record or owned cleanup state. Return identity-keyed results/state, reject or explicitly stop replacement, define rollback/partial-success policy, aggregate stop failures, and remove the false Drop promise unless an owned supervisor can fulfill it.
- Regression: duplicate ID, deterministic mixed start result, rollback policy, double start, mixed stop failure, Drop after start, and ID-bearing EKO diagnostics.
- Validation: [V02](../validations/F-INT-02/V02-01.md)

### F-INT-02-P1-02: Group chat identity is flattened to sender identity

- Priority: P1; confidence: high; layer: framework/application adapter.
- Evidence/reachability: `echo-agent/echo-integration/src/channels/types.rs:74`, `session.rs:158`, `:295`, `:306`; `echo-agent/src/channels.rs:118`; `echo-agent-cli/src/cli/channels.rs:65`, `:85`, `:121`.
- Expected: session/cache identity includes the platform conversation, and outbound group messages target `chat_id` while direct messages target the peer.
- Observed/impact: sessions and EKO pool/cache keys omit `chat_id`, so one sender shares Agent context across different groups. Reset and normal replies use `sender_id` as `OutboundMessage.to` even for Group, while QQ/Feishu group senders require a group/chat ID. This risks cross-chat context disclosure and failed/misdirected group responses.
- Root cause/direction: a per-sender assumption replaced canonical message identity at multiple adapters. Define one lossless conversation/reply-target conversion from `InboundMessage`; include group/thread context and reuse it for session, cache, HITL, normal reply, reset, and streaming chunks.
- Regression: same sender in two groups and direct chat, cache/session separation, reset and multi-chunk reply targets, QQ and Feishu group endpoints.
- Validation: [V03](../validations/F-INT-02/V03-01.md)

### F-INT-02-P1-03: Session replacement and callbacks race with in-flight Agent work

- Priority: P1; confidence: high; layer: framework.
- Evidence/reachability: `echo-agent/echo-integration/src/channels/session.rs:259`, `:297`, `:315`, `:318`, `:343`, `:351`; every configured framework/EKO session uses this handler.
- Expected: one conversation has defined execution order; reset/timeout cancels or joins old work before cleanup; idle sessions end observably.
- Observed/impact: the mutex protects only handler lookup, then is released before `handle` or stream consumption. Same-session messages run concurrently; reset/timeout can remove/replace and fire cleanup while an old `Arc` still executes. Idle entries never expire without another message and Drop emits no end callback.
- Root cause/direction: session map ownership is mistaken for execution ownership. Add a per-session owned queue/supervisor with cancellation and generation identity; make reset/timeout/drop transition once and join/drain according to a documented policy.
- Regression: concurrent message ordering, reset/timeout during model/tool stream, old completion after replacement, exactly-once callback, idle eviction, drop.
- Validation: [V04](../validations/F-INT-02/V04-01.md)

### F-INT-02-P1-04: QQ stop creates an unowned hot-loop sender task

- Priority: P1; confidence: high; layer: adapter.
- Evidence/reachability: `echo-agent/echo-integration/src/channels/channels/qq/channel.rs:93`, `:105`, `:108`, `:192`, `:198`, `:225`; selected by live EKO QQ registration.
- Expected: all spawned tasks are owned, channel closure terminates receive loops, stop awaits cleanup, and health covers all background work.
- Observed/impact: the send task handle is discarded, and its loop does not break when `recv()` returns `None`. Stop drops the sender and aborts only the gateway, so the detached sender can poll a closed queue forever at high CPU. Repeated start can orphan additional gateway/send tasks, and health sees only the newest gateway.
- Root cause/direction: receive/send tasks are not one plugin lifecycle. Store both handles under one cancellation owner; use `while let Some`, close/drain explicitly, await termination, and reject or idempotently handle double start.
- Regression: stop/drop/double-start, closed queue, sender panic/failure, gateway failure, no live task after stop.
- Validation: [V05](../validations/F-INT-02/V05-01.md)

### F-INT-02-P1-05: Feishu Webhook signs reconstructed JSON and permanently acknowledges failed work

- Priority: P1; confidence: high; layer: adapter.
- Evidence/reachability: `echo-agent/echo-integration/src/channels/channels/feishu/webhook.rs:80`, `:100`, `:115`, `:141`, `:152`, `:189`; selected by configured Feishu Webhook mode.
- Expected: HMAC consumes exact raw body bytes; successful acknowledgement means work is durable or completed; handler failure remains retryable; shutdown owns event work.
- Observed/impact: the extractor parses JSON before verification and reserializes it for HMAC, changing valid wire representations. Message IDs are marked processed before validation/handler/reply; the endpoint returns 200 and a detached task only logs failure. Valid signed calls can be rejected, while accepted calls can be lost and all retries suppressed.
- Root cause/direction: protocol acknowledgement, dedup, and processing ownership are split. Verify raw bytes first; model dedup as in-progress/completed with durable or owned queue semantics, and make shutdown drain/cancel explicitly.
- Regression: valid signatures over reordered/whitespace JSON, invalid signature, handler/reply failure, concurrent duplicate, retry after failure, stop during event.
- Validation: [V06](../validations/F-INT-02/V06-01.md)

### F-INT-02-P1-06: Feishu long-poll trusts unbounded fragment counts and acknowledges before recoverable processing

- Priority: P1; confidence: high; layer: adapter.
- Evidence/reachability: `echo-agent/echo-integration/src/channels/channels/feishu/long_poll.rs:374`, `:408`, `:416`, `:434`, `:465`, `:482`; `feishu/channel.rs:207`, `:289`.
- Expected: peer-controlled framing is bounded before allocation; acknowledgements and dedup retain retry/recovery facts; stop owns receive, event, and send tasks.
- Observed/impact: positive peer `sum` is cast to `usize` and allocated directly with no count/byte cap. Response and dedup occur before detached processing, whose errors only log, so retry can be skipped after failure. Stop aborts only the top receive task; event and send tasks are detached.
- Root cause/direction: framing, delivery acknowledgement, and plugin task ownership are independent helpers. Add strict fragment/count/byte limits and one owned delivery supervisor; preserve failure/retry state and await all plugin work on stop.
- Regression: extreme/missing/duplicate fragments, aggregate byte cap, malformed JSON, handler/reply failure and replay, reconnect, stop during receive/send.
- Validation: [V07](../validations/F-INT-02/V07-01.md)

### F-INT-02-P1-07: Channel send success ends at enqueue and hides actual delivery failure

- Priority: P1; confidence: high; layer: framework/adapter contract.
- Evidence/reachability: `echo-agent/echo-integration/src/channels/types.rs:206`; QQ `channel.rs:108`, `:210`; Feishu `channel.rs:207`, `:311`.
- Expected: a `Result` named `send` reports transport outcome, or the API explicitly returns a durable delivery receipt/state whose later failure remains observable.
- Observed/impact: both implementations return `Ok` after MPSC enqueue. Later token/API/network failure is log-only in detached consumers, so callers and cleanup logic cannot distinguish delivery from loss.
- Root cause/direction: queue admission is exposed as protocol completion. Either await a per-message oneshot delivery result, expose typed queued/delivered/failed state, or rename/document enqueue semantics and provide an observable delivery stream.
- Regression: token failure, HTTP non-success, queue close/full/cancel, stop with queued messages, ordered per-message receipts.
- Validation: [V08](../validations/F-INT-02/V08-01.md)

### F-INT-02-P2-01: Channel duration arithmetic can underflow or overflow

- Priority: P2; confidence: high; layer: adapter/framework.
- Evidence/reachability: `echo-agent/echo-integration/src/channels/channels/qq/api.rs:155`, `feishu/api.rs:281`, `channels/session.rs:87`.
- Expected: server/public numeric values are range-validated and use checked/saturating arithmetic.
- Observed/impact: token refresh calculates `now + expires_in - 300`; short expiry underflows and extreme expiry overflows. Public session minutes multiply by 60 unchecked. Debug builds can panic and release builds can wrap into incorrect long-lived state.
- Root cause/direction: safety margin and unit conversion are applied after unchecked arithmetic. Use checked/saturating operations with explicit minimums and typed invalid-config/protocol errors.
- Regression: expiry 0/299/300/maximum and timeout-minute multiplication boundary.
- Validation: [V08](../validations/F-INT-02/V08-01.md)

## Validation Matrix

| ID | Claim | Status | Report |
|---|---|---|---|
| V01 | feature/export/layering and real reachability | passed | [V01](../validations/F-INT-02/V01-01.md) |
| V02 | manager identity/lifecycle/cleanup | failed | [V02](../validations/F-INT-02/V02-01.md) |
| V03 | session and group reply identity | failed | [V03](../validations/F-INT-02/V03-01.md) |
| V04 | session concurrency/reset/timeout cleanup | failed | [V04](../validations/F-INT-02/V04-01.md) |
| V05 | QQ background lifecycle | failed | [V05](../validations/F-INT-02/V05-01.md) |
| V06 | Feishu Webhook signature/delivery | failed | [V06](../validations/F-INT-02/V06-01.md) |
| V07 | Feishu long-poll framing/delivery/cleanup | failed | [V07](../validations/F-INT-02/V07-01.md) |
| V08 | typed send failure and numeric safety | failed | [V08](../validations/F-INT-02/V08-01.md) |
| V09 | LSP/A2A canonical dedup coverage | passed | [V09](../validations/F-INT-02/V09-01.md) |
| V10 | existing tests and future fixtures | not_run | [V10](../validations/F-INT-02/V10-01.md) |
| V11 | report integrity and source isolation | passed | [V11](../validations/F-INT-02/V11-01.md) |
| V30 | primary current-commit source sampling and acceptance | passed | [V30](../validations/F-INT-02/V30-01.md) |

## LSP And A2A Deduplication

F-INT-01 remains canonical for LSP request timeout/cancel/process status/restart,
Content-Length, wire enum, and URI/document/version/position behavior, plus A2A
cancel, Error-to-Completed, duplicate task ID, token cleanup, client UTF-8/SSE
errors, typed errors, and timeout. Current sampling found no separate F-INT-02
root cause; V09 records coverage without duplicating those findings.

## Coverage Gaps And Residual Uncertainty

- No dynamic validation ran by explicit instruction. V10 and every finding's
  regression list are future work, not blockers to the source-conclusive review.
- Platform-specific QQ reconnect/resume and delivery semantics were not promoted
  to a separate finding without a protocol fixture; the clearly owned task leak
  is sufficient and independently actionable.
- Feishu media is explicitly reported unsupported by channel capabilities; this
  review does not treat unused generic attachment DTOs as dead framework API.
- The six externally modified files observed during delegated capture were later
  committed as `3aa79299`. Primary review confirmed that commit does not touch
  channel/LSP/A2A integration scope; both source repositories were clean at final
  acceptance.

## Handoff

First fix plugin and manager lifecycle ownership, then canonical group/session
identity, then Feishu acknowledgement/dedup/framing. Keep protocol machinery in
the framework and EKO bootstrap/presentation policy in the application. Do not
add cloud-style permission gates. Primary current-commit sampling is recorded in
V30; future dynamic work is regression validation, not missing review evidence.
