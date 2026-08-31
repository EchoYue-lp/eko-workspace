---
schema_version: 3
slug: 2026-08-30-framework-boundary-and-bilingual-docs/plan
status: superseded
goal: 删除 EKO AgentRouter 已被 framework DeliveryLedger 替代的旧 projection/reducer 逻辑，同时保留最小 legacy wire/checkpoint codec 以恢复已 pruning 的旧 journal。
ships: app-core framework-backed delivery view、legacy checkpoint codec、删除 AgentInboxProjection/FoldedDelivery 的生产与重复测试代码、AgentRouter recovery/cursor/retirement 回归证据、双语状态与 Phase 3 交接记录。
verify: AgentRouter 所有读写继续由 framework DeliveryLedger 驱动；旧 journal wire 与 retained-floor checkpoint 可恢复；不存在旧 app reducer、第二 retention/validation authority 或旧 checkpoint writer；Conversation delivery、live/cold、TaskSubagent 和 retirement 行为保持不变。
design_ref: docs/supreme/specs/2026-08-30-framework-boundary-and-bilingual-docs/design.md
design_revision: sha256:7758c064650d16549ca5ca46899cb88e5ebbc0f630a089990c8f29d67ab6bb63
artifact_id: plan:7a3e9d0d-49a1-4c14-bc37-d60e7a5f7f37
lifecycle: completed
todos:
  - id: remove-eko-projection-authority
    files:
      - echo-agent-cli/echo-agent-app-core/src/agent_router/mod.rs
      - echo-agent-cli/echo-agent-app-core/src/agent_router/projection.rs
      - echo-agent-cli/echo-agent-app-core/src/agent_router/legacy.rs
      - echo-agent-cli/echo-agent-app-core/src/agent_router/inbox.rs
      - echo-agent-cli/echo-agent-app-core/src/agent_router/delivery.rs
      - echo-agent-cli/echo-agent-app-core/src/agent_router/recovery.rs
      - echo-agent-cli/echo-agent-app-core/src/agent_router/router.rs
    summary: 删除旧 AgentInboxProjection/FoldedDelivery reducer 与重复 retention/validation authority，改为 framework record 到 AgentDeliveryRecord 的只读 view，并保留 legacy journal/checkpoint codec。
    verify: production app-core 不再实现 EventReducer<AgentInboxEvent>、旧 frontier/terminal retention 或旧 checkpoint writer；framework ledger 是唯一 projection/retry authority，pruned legacy journal 有 typed checkpoint bootstrap。
  - id: migrate-router-regressions
    files:
      - echo-agent-cli/echo-agent-app-core/src/agent_router/tests.rs
    summary: 删除被替代 projection 性能/热校验测试，补齐 framework-backed facade、legacy checkpoint、restart、retirement 和 TaskSubagent 边界回归。
    verify: AgentRouter focused tests 覆盖 enqueue、FIFO claim、完整 lifecycle、stale/owner-loss、legacy replay、checkpoint recovery、retention、cursor 和 retirement；不再直接驱动旧 projection。
  - id: update-phase3-docs
    files:
      - echo-agent-cli/docs/zh/architecture/runtime.md
      - echo-agent-cli/docs/en/architecture/runtime.md
      - echo-agent-cli/docs/zh/project-status.md
      - echo-agent-cli/docs/en/project-status.md
      - echo-agent/docs/adr/0018-agent-router-legacy-wire-boundary.md
      - echo-agent/docs/en/41-delivery-ledger.md
      - echo-agent/docs/zh/41-delivery-ledger.md
    summary: 记录旧 projection 删除、legacy wire/checkpoint codec 保留和 framework authority 的最终边界。
    verify: 中英文文档一致描述 Phase 3；不宣称删除物理旧 wire 或已经完成最终 release。
  - id: record-phase3-handoff
    files:
      - docs/2026-08-30-framework-capability-placement-audit.md
      - docs/MASTER-PLAN.md
    summary: 更新跨仓 owner ledger、child SHA、Phase 3 状态和剩余 release/clean-checkout 门禁。
    verify: 顶层事实源明确旧 reducer 已删除、legacy codec 是唯一兼容残留、下一步只剩最终集成与发布门禁。
---
## Historical status

本计划的删除目标已完成，但“保留 legacy wire/checkpoint codec”这一兼容性假设已被开发期
直接 typed schema reset 取代。当前 AgentRouter 没有 `AgentInboxProjection`、`FoldedDelivery`、
`LegacyDeliveryJournal` 或 legacy checkpoint codec；framework typed ledger 是唯一 durable
authority。该计划正文保留原始决策和验收背景，当前实现与公开 API 以 framework ADR 0019、
EKO ADR 0026 及其双语文档为准。

## Context

Phase 2 已将 AgentRouter 的所有生产读写切换到 framework `DeliveryLedger`，但 app-core 仍保留旧 `AgentInboxProjection`、`FoldedDelivery` 和对应 EventReducer/retention 测试。它们不再拥有生产 authority，却继续制造重复维护面。旧 `AgentInboxEvent` wire 和已 pruning journal 的 checkpoint 仍需要最小 codec，不能与旧 reducer 一起删除。

## Approach

- 删除 `projection.rs` 中的旧 reducer、frontier fold、terminal retention 和全量校验；`AgentRouter` 读取 framework `DeliveryRecord`，通过只读 `AgentDeliveryProjectionView` 映射既有应用 DTO。
- 将旧 checkpoint JSON 的反序列化缩减为 legacy codec，仅在 framework checkpoint 缺失且旧 journal retained floor 大于 1 时 bootstrap；codec 不实现 `EventReducer`、不写旧 checkpoint、不维护 retry/frontier。
- `LegacyDeliveryJournal` 继续把 framework `DeliveryEvent` 映射到既有 `AgentInboxEvent` wire，保证物理 journal、sequence、lookup/reopen 和 pruning 不变；这不是第二 mailbox。
- 删除直接操纵旧 projection 的性能测试，保留 facade-level lifecycle/recovery/retirement 测试，并让 framework 自有 tests 负责通用 retention/validation。

## Global Constraints

- framework 不引用 EKO 类型；EKO adapter 不复制通用 FIFO、retry、terminal 或 validation 语义。
- 旧 journal wire 可读写，旧 retained-floor checkpoint 可转换；codec 映射失败必须 fail closed，不删除用户数据。
- `EffectStarted`、`MailboxAccepted`、`Drained`、owner-loss、TaskSubagent 分叉和 stable IDs 语义不变。
- 不引入 SQLite、第二 mailbox、第二 reducer、worker 术语或新的产品状态机。
- 不删除物理旧 journal、groups、workspace/retirement policy 或 live/cold runtime；最终 release/clean-checkout 门禁另行记录。

## Files

- Delete: `echo-agent-cli/echo-agent-app-core/src/agent_router/projection.rs` — obsolete app reducer and FoldedDelivery authority。
- Modify: `echo-agent-cli/echo-agent-app-core/src/agent_router/mod.rs` — remove obsolete projection include。
- Modify: `echo-agent-cli/echo-agent-app-core/src/agent_router/legacy.rs` — minimal checkpoint codec and read-only framework view。
- Modify: `echo-agent-cli/echo-agent-app-core/src/agent_router/inbox.rs` — framework-backed authority type wiring。
- Modify: `echo-agent-cli/echo-agent-app-core/src/agent_router/delivery.rs` — framework checkpoint bootstrap without old reducer。
- Modify: `echo-agent-cli/echo-agent-app-core/src/agent_router/recovery.rs` — view-based facade mapping。
- Modify: `echo-agent-cli/echo-agent-app-core/src/agent_router/router.rs` — view/cursor reads through framework authority。
- Modify: `echo-agent-cli/echo-agent-app-core/src/agent_router/tests.rs` — framework-backed regression suite。
- Create: `echo-agent/docs/adr/0018-agent-router-legacy-wire-boundary.md` — final framework/application compatibility boundary。
- Modify: `echo-agent/docs/en/41-delivery-ledger.md` — framework compatibility boundary。
- Modify: `echo-agent/docs/zh/41-delivery-ledger.md` — framework compatibility boundary。
- Modify: `echo-agent-cli/docs/zh/architecture/runtime.md` — Phase 3 authority status。
- Modify: `echo-agent-cli/docs/en/architecture/runtime.md` — Phase 3 authority status。
- Modify: `echo-agent-cli/docs/zh/project-status.md` — Phase 3 authority status。
- Modify: `echo-agent-cli/docs/en/project-status.md` — Phase 3 authority status。
- Modify: `docs/2026-08-30-framework-capability-placement-audit.md` and `docs/MASTER-PLAN.md` — Phase 3 handoff。

## Reuse

- `echo-agent/echo-state/src/delivery.rs` — `DeliveryLedgerProjection`, `DeliveryRecord`, `from_records`, lifecycle and retention authority。
- `echo-agent/echo-state/src/journal/mod.rs` — checkpoint store, prepared identity, journal lookup/reopen and replay。
- `echo-agent-cli/echo-agent-app-core/src/agent_router/legacy.rs` — existing typed event wire adapter。
- `echo-agent-cli/echo-agent-app-core/src/agent_router/router.rs` and `recovery.rs` — shared facade call paths and retirement guards。

## Todos

### remove-eko-projection-authority

requirements:
- § Framework 能力归属
- § Framework 迁移数据流
- § 异常与边界场景
- § 关键取舍

interfaces:
- consumes: framework `DeliveryLedgerProjection` and legacy checkpoint JSON。
- produces: app-owned read-only delivery view and minimal legacy codec。

steps:

1. Replace old projection callback types with a framework-backed view exposing only the fields required by AgentRouter facade adapters.
   verify: recovery/router code compiles without `AgentInboxProjection` or `FoldedDelivery`, and no app reducer owns lifecycle transitions.
   expected: app layer has no duplicate lifecycle state machine.
2. Move retained-floor checkpoint bootstrap to the minimal codec and remove old checkpoint/reducer writes.
   verify: full old journal and already-pruned journal with a valid legacy checkpoint both recover; invalid/missing checkpoint fails closed when required.
   expected: old data remains recoverable without retaining the obsolete reducer.

### migrate-router-regressions

requirements:
- § Framework 迁移数据流
- § 异常与边界场景
- § 验收标准

interfaces:
- consumes: existing AgentRouter facade and framework delivery tests。
- produces: focused app-core regression evidence。

steps:

1. Remove tests whose only authority is the deleted app projection and add facade-level checks for framework records, wire replay, checkpoint, retirement and TaskSubagent isolation.
   verify: no test calls old `apply_checked`, `full_validation_count` or old frontier fields; focused router suite remains green.
   expected: tests protect the actual authority rather than a dead implementation.

### update-phase3-docs

requirements:
- § 当前候选边界
- § Framework 迁移数据流
- § 异常与边界场景

interfaces:
- consumes: final child implementation and Phase 2 handoff。
- produces: bilingual architecture/status pages and framework ADR。

steps:

1. Document the deleted app reducer, retained wire/checkpoint codec, and final framework/EKO ownership.
   verify: zh/en parity and website source-aware sync remain valid.
   expected: no stale claim that legacy projection is an active authority.

### record-phase3-handoff

requirements:
- § 当前候选边界
- § 候选交付结果与依赖
- § 验收标准

interfaces:
- consumes: framework, CLI and website commits。
- produces: top-level phase ledger and release residuals。

steps:

1. Record exact symbols deleted, compatibility codec retained, child SHAs and remaining final integration gates.
   verify: top-level owner ledger is consistent with child docs and website manifest.
   expected: Phase 3 can stop without misreporting release readiness.

## Diagram

```mermaid
flowchart LR
  W[Legacy AgentInboxEvent wire] --> A[LegacyDeliveryJournal codec]
  A --> F[Framework DeliveryLedger]
  F --> V[Read-only AgentDeliveryProjectionView]
  V --> R[AgentRouter facade]
  R --> E[EKO AppState live/cold]
  C[Legacy checkpoint codec] --> F
  T[TaskSubagent] --> S[SubagentControlService]
```

## Decisions

- Delete the obsolete app reducer now that framework owns all lifecycle and retention semantics.
- Retain only the minimal legacy wire/checkpoint codec required for persisted EKO data; it cannot mutate lifecycle state.
- Do not delete physical journal files or claim final release readiness in this phase.
