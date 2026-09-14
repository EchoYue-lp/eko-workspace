---
schema_version: 3
supersedes: null
slug: echo-agent全workspace语义治理/建立全workspace语义基线
goal: 把 echo-agent 从 SDK facade 局部基线扩展为覆盖整个 11-package workspace 的可验证语义基线
ships: 建立绑定当前 revision 的全 workspace 语义基线，闭合路径库存、Capability Map、核心
  Asset、状态权威、生命周期、外部副作用和未知入口
verify: semantic-status 报告 baseline 结构有效且
  inventory_closure=closed、behavior_model_closure=closed；所有 in_scope
  区域覆盖八个风险视角，strict snapshot 与 semantic-verify 返回 0，独立 capability review
  通过，且业务代码、SDK contract 与 website 无差异
design_ref: null
delivery_ref: docs/supreme/plans/2026-09-13T1448-echo-agent全workspace语义治理/delivery-map.md#workspace-semantic-baseline
todos:
  - id: classify-workspace-inventory
    files:
      - echo-agent/.echo-semantic/baseline.md
      - echo-agent/.echo-semantic/discovery
      - echo-agent/.echo-semantic/assets
    summary: 分类全部 Git 路径、11-package crate DAG、feature topology、入口、动态注册和消费者资产
    verify: 每个非语义 Git 路径唯一命中区域分类；动态注册和不能静态解析的路径进入 unresolved 或
      needs_review；canonical entry/state/protocol/test/doc Assets 可追溯
  - id: model-core-lifecycle-boundaries
    files:
      - echo-agent/.echo-semantic/maps
      - echo-agent/.echo-semantic/behaviors
      - echo-agent/.echo-semantic/rules
      - echo-agent/.echo-semantic/evidence
    summary: 建模 Agent/Session/Context/Run/Task/Subagent/Observation/Persistence
      的状态权威与生命周期
    verify: 核心身份关系、接纳/准备/执行/effect/wait/terminal/recovery、checkpoint/clear、claim/retry/cancel/replay/retention
      场景都有 source refs 和明确处置
  - id: model-effect-and-extension-boundaries
    files:
      - echo-agent/.echo-semantic/maps
      - echo-agent/.echo-semantic/behaviors
      - echo-agent/.echo-semantic/rules
      - echo-agent/.echo-semantic/evidence
    summary: 建模 Tool/Permission/Sandbox/MCP/Hook/Skill/Plugin/LLM/Protocol/Eval
      等副作用与扩展边界
    verify: 策略优先级、权限判定、资源 owner、发布/撤销、外部副作用、失败清理和协议投影均映射到真实实现；Workspace/Device
      产品概念不被误建为 framework authority
  - id: close-workspace-baseline
    files:
      - echo-agent/.echo-semantic
      - docs/MASTER-PLAN.md
    summary: 复核所有能力场景、未知项和引用，闭合 inventory 与 behavior model 并更新阶段状态
    verify: 10 个顶层 Capability Map 及 SDK 子图形成闭合覆盖；所有场景为
      mapped/needs_review/excluded，所有引用和快照可解析，独立 reviewer 与 semantic-verify 通过
artifact_id: plan:811d3443-a224-4a65-a9e3-03445c5f1413
lifecycle: completed
design_revision: null
---
## Context

- 当前 `.echo-semantic` 只有 SDK facade 子边界：1 Map、1 Behavior、1 Rule、2 Evidence、5 resolved Findings、1 Audit、1 Discovery、0 Assets。
- `inventory_closure` 和 `behavior_model_closure` 均为 open，`coverage=[]`；严格快照通过只证明 SDK 局部材料一致，不代表全仓闭合。
- 仓库已有 30+ ADR、中英文正式文档、21 个 executable example contracts 和完整测试，可作为发现证据，不重新发明核心模型。
- SDK identity 已由 ADR 0031 降级为独立 drift telemetry；本 Plan 不继续 intrinsic 映射。

## Approach

- 使用一个现有 `.echo-semantic` 目录扩展全仓 baseline，不创建平行架构目录或第二套 inventory。
- 以语义边界而不是 API identity 建模：顶层 10 张 Capability Map，SDK facade 现有 map 作为协议/SDK 边界子图保留。
- Asset 只记录 canonical owner、入口、状态权威、协议、测试消费者和正式文档，不为每个 public 字段或方法建 Asset。
- Behavior 保存重要行为承诺；Rule 只保存唯一 authority 和不变量；Evidence 聚合当前源码、测试、example、ADR 和机器验证。
- 不确定、动态注册和跨边界歧义保持 `needs_review`/`unresolved`；discovery 不为了闭合而宣称行为正确。
- 本 Plan 只建立语义事实和 Finding 候选，不修业务代码；确认问题在后续 audit/repair outcome 单独实施。

## Global Constraints

- 治理主单位为 Capability、Behavior、Rule、State Authority、Lifecycle、Finding、Evidence。
- Framework 与应用分层遵循 ADR 0014；EKO Workspace、DomainProfile、review/worktree policy、UI/TUI/CLI projection 和 Device sync 不进入 framework authority。
- `TaskRevisionService`/`RuntimeTaskService`、`AgentTurnDriver`/`TurnReceipt`、`RuntimeStateStore`、`ConversationStore`、`EventJournal`、`RunStore`、`DeliveryLedger`、`SubagentExecutor` 等现有权威先复用，不凭名称另建抽象。
- Plan 是版本化 artifact，Todo 是 projection；禁止新增 plan approval runtime state 或第二 Task CRUD/DAG loop。
- 全项目统一使用 Subagent，不新增 Worker 术语。
- 不修改 Rust、Cargo feature、SDK language source、contract、schema、examples、tests 或 `echo-website`。
- 历史综合审计只作为候选输入；每项结论必须重锚当前 revision。
- 不使用 SDK identity 完成率表示全仓治理进度。

## Files

- Modify: `echo-agent/.echo-semantic` — 承载本 Plan 的全部全仓语义对象和快照，不创建第二治理目录。
- Modify: `echo-agent/.echo-semantic/baseline.md` — 扩展区域、边界、覆盖矩阵和闭合状态。
- Create: `echo-agent/.echo-semantic/discovery` — 保存全仓扫描范围、候选事实、动态未知入口和归并结果。
- Create: `echo-agent/.echo-semantic/assets` — 登记 canonical 入口、状态权威、协议、测试与文档资产。
- Create: `echo-agent/.echo-semantic/maps` — 新增 10 个顶层 Capability Map，复用现有 SDK 子图。
- Create: `echo-agent/.echo-semantic/behaviors` — 保存跨边界的重要行为承诺。
- Create: `echo-agent/.echo-semantic/rules` — 保存唯一权威和不变量。
- Create: `echo-agent/.echo-semantic/evidence` — 聚合当前源码、ADR、测试和验证证据。
- Modify: `echo-agent/.echo-semantic/README.md` — 说明全仓模型、对象范围和维护入口。
- Modify: `docs/MASTER-PLAN.md` — 仅更新本阶段真实闭合状态和下一入口。

## Reuse

- `echo-agent/.echo-semantic/maps/map.sdk-facade-parity.md` — 保留为协议/SDK 顶层边界的子图。
- `echo-agent/docs/adr/0001-channel-session-sender-scope.md` 与 `0006-runtime-state-scope-lineage.md` — Session、Conversation、runtime incarnation 证据。
- `echo-agent/docs/adr/0005-invocation-resource-lifetime.md` 与 `0010-canonical-turn-receipt-accounting.md` — Invocation resource 和 TurnReceipt 权威。
- `echo-agent/docs/adr/0008-canonical-runtime-task-authority.md` — TaskRevision/TaskRun/claim/retry/cancel/settlement 权威。
- `echo-agent/docs/en/41-persistence-concepts.md` 与 ADR 0007/0019 — Store/Journal/Checkpoint/Trace/Delivery 分层。
- `echo-agent/docs/adr/0012-immutable-plugin-preparation.md` 与 `0030-versioned-subagent-event-envelope.md` — Plugin generation 与 Subagent event/replay。
- `echo-agent/echo-agent-learning/tests/example_contracts`、各 crate tests 和 `tests/` — 可执行消费者与失败场景证据。
- `docs/comprehensive-review` — 仅作为需要重新验证的历史 Finding 候选清单。

## Todos

### classify-workspace-inventory

requirements:
- 用户确认：第一层先做全 workspace 语义发现，只盘点和建模，不改业务代码。
- Echo Semantic：全部 Git 路径必须分类，动态入口和未知事实不得静默遗漏。

interfaces:
- consumes: 当前 child commit、Cargo metadata、root facade、crate module trees、feature definitions、ADR、正式文档、tests/examples、现有 SDK semantic objects。
- produces: 全仓 Discovery、区域分类、crate/feature/entry inventory、canonical Asset 集合。

steps:

1. 固定当前 Git revision 和源码摘要，枚举全部 tracked/unignored 路径、11-package DAG、feature topology、public/后台入口和动态注册点。
   verify: inventory 能解释每个路径的分类，并把配置路由、插件发现、注册表和反射式入口记录为已映射或 unresolved。
   expected: 不再用 `src`/`docs` 大目录概括整个仓库；每个 in_scope 区域有明确 owner 和复查边界。

2. 为 canonical 入口、状态权威、协议、测试消费者和文档创建稳定 Asset，并连接候选重复。
   verify: 每个 Asset 的 code/consumer/behavior/rule/evidence 引用可解析；API identity 不被逐项复制为 Asset。
   expected: 后续审计可以从任一 authority 追到生产入口、持久化、副作用和验证消费者。

### model-core-lifecycle-boundaries

requirements:
- 用户要求：统一 Agent、Revision、Session、Context、Run、Observation、TaskRun 与 SubagentAttempt 的关系。
- 用户要求：确认创建、发布、接纳、准备、领取、Reply、Effect、Wait、终态、恢复、压缩、检查点和 clear。

interfaces:
- consumes: inventory Assets、现有 ADR 0001/0005/0006/0007/0008/0010/0019/0030、生产源码和 executable contracts。
- produces: Agent identity、ReAct execution、Context persistence、Task/Subagent、Observation persistence 等 Capability Maps 及关联 Behavior/Rule/Evidence。

steps:

1. 建模 AgentDefinition/AgentInstance、ACP/Channel Session、Conversation、Invocation、Turn、runtime incarnation 和各限定 Revision 的身份关系。
   verify: 裸 `Session`、`Run`、`Revision` 的多义性被显式拆开；不存在凭空新增的统一 AgentRevision。
   expected: 每种 identity 都有 owner、创建/失效条件和跨边界映射。

2. 建模 ReAct、Context/Memory、Task/Subagent/Workflow 和 Observation/Persistence 的正常、失败、取消、超时、重试、恢复与终态场景。
   verify: 状态决策来自 canonical service/store/driver；projection、trace 和 live event 不被误当 durable fact。
   expected: 每个高风险生命周期都有可反证 source/test Evidence 或明确 needs_review。

### model-effect-and-extension-boundaries

requirements:
- 用户要求：统一 Workspace/Device、Permission/Tools、MCP/Hook/Library、SDK/Contracts、Delivery/Observability 方向。
- 仓库约束：本地个人助理安全边界不能把应用策略硬塞进通用 framework。

interfaces:
- consumes: Tool/Permission/Sandbox/MCP/Hook/Skill/Plugin/LSP/LLM/ACP/A2A/Channels/Eval/Evolution 入口和对应 tests/docs。
- produces: Effect/permission、extension lifecycle、LLM/provider、protocol/SDK、eval/evolution Capability Maps 及关联规则和证据。

steps:

1. 沿真实调用链记录策略合成、tool rewrite、permission decision、sandbox/process/file/network effect、资源 owner 和清理责任。
   verify: 交互式用户工具与 Agent 自动权限路径被区分；每个 terminal 前的 cleanup 义务可追到实现。
   expected: 不新增面向公网或多租户的假权限门禁，真实本地数据/资源风险保持可审计。

2. 建模 MCP/Hook/Skill/Plugin/LSP 加载、协商、发布、热更新、撤销、rollback、generation 和外部协议投影；补充 LLM/provider/budget/timeout/stream 与 Eval/Evolution 边界。
   verify: 动态配置/插件入口全部显式列出；ACP、A2A、MCP、SDK 的职责不互相替代；Product Backend/Frontend/Desktop/Device 只记录边界证据。
   expected: 扩展与协议失败不会被描述成第二套 Agent、Session、Run 或状态权威。

### close-workspace-baseline

requirements:
- Echo Semantic：库存闭合与行为模型闭合必须分别给出真实状态，结构和严格快照校验通过。
- 用户要求：形成全项目语义统一、优化的真实进度和后续 Finding 集合。

interfaces:
- consumes: 全部 Maps、Assets、Behaviors、Rules、Evidence、unresolved 和现有 Findings。
- produces: closed baseline、十边界覆盖矩阵、可执行下一阶段 Audit frontier、更新后的顶层进度。

steps:

1. 对每个顶层边界完成八风险视角覆盖，复核每个场景的 mapped/needs_review/excluded 状态和全部引用。
   verify: baseline 无重复/冲突路径分类，所有 in_scope 区域有八视角覆盖，未知项都有 next_step。
   expected: inventory_closure 和 behavior_model_closure 只在合同真实满足时改为 closed。

2. 执行独立 capability review、semantic-status、strict snapshot 和 semantic-verify，并更新 MASTER-PLAN 的当前状态。
   verify: reviewer 返回 pass；机器校验返回 0；状态视图显示闭合 baseline 且下一入口为 high-risk semantic audit。
   expected: 全仓语义基线成为后续 Finding 驱动修复的唯一输入，不宣称仓库没有缺陷。

## Decisions

- 顶层使用 10 个 Capability Map；SDK facade 现有 map 是协议/SDK map 的子图。
- Workspace identity、Device sync、产品 backend 和 frontend/desktop 不进入 echo-agent framework baseline，只记录适配边界。
- Discovery 发现的问题只形成 Finding/unknown，不在本 Plan 修业务代码。
- 闭合表示库存和行为模型可追踪，不表示所有 Finding 已解决或系统没有缺陷。