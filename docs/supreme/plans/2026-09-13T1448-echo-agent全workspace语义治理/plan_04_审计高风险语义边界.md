---
schema_version: 3
supersedes: null
slug: echo-agent全workspace语义治理/审计高风险语义边界
goal: 对全 workspace 基线中的高风险状态权威、生命周期、持久化、副作用和协议边界执行可反证审计
ships: 对运行时、持久化、并发恢复、权限与外部副作用、协议契约等高风险边界形成可反证 Audit 和当前 Finding 集合
verify: 每个审计单元绑定当前源码摘要、明确故障假设、真实入口、状态与失败路径；44 个既有开放 Finding 均有审计路由或明确 defer
  理由，新增反例形成 Finding；独立 risk review、strict snapshot、change evidence 与
  semantic-verify 全部通过，且业务代码、SDK contract、examples、tests 与 website 无差异
design_ref: null
delivery_ref: docs/supreme/plans/2026-09-13T1448-echo-agent全workspace语义治理/delivery-map.md#high-risk-boundary-audits
todos:
  - id: audit-runtime-authorities
    files:
      - echo-agent/.echo-semantic/audits
      - echo-agent/.echo-semantic/findings
      - echo-agent/.echo-semantic/maps
      - echo-agent/.echo-semantic/evidence
    summary: 审计 Agent/Turn、Context/Memory、Task/Subagent/Workflow 的状态权威与并发生命周期
    verify: 每轮最多三个边界与风险视角单元；Turn
      route、checkpoint/transcript、claim/attempt、factory、workflow/scheduler/background
      的故障假设均有源码反证与 Finding disposition
  - id: audit-persistence-and-effects
    files:
      - echo-agent/.echo-semantic/audits
      - echo-agent/.echo-semantic/findings
      - echo-agent/.echo-semantic/maps
      - echo-agent/.echo-semantic/evidence
    summary: 审计 Observation/Persistence 与 Tool/Permission/Sandbox 的事实、投影、副作用和清理责任
    verify: Journal/Projection/Trace/Delivery、event
      family、cache/validation、permission/Hook/sandbox、artifact/CommandCell
      cleanup 的权威和失败路径均有可复核 disposition
  - id: audit-extension-and-provider
    files:
      - echo-agent/.echo-semantic/audits
      - echo-agent/.echo-semantic/findings
      - echo-agent/.echo-semantic/maps
      - echo-agent/.echo-semantic/evidence
    summary: 审计 MCP/Hook/Skill/Plugin/LSP 与 LLM provider 的 generation、权限、取消和协议合同
    verify: 扩展 owner/publication/withdraw/cleanup、provider capabilities/structured
      output/cancellation/stream terminal 均以当前源码与契约证据审查，未知产品预期进入 decision 而非擅自裁决
  - id: audit-protocol-quality-frontier
    files:
      - echo-agent/.echo-semantic/audits
      - echo-agent/.echo-semantic/findings
      - echo-agent/.echo-semantic/maps
      - echo-agent/.echo-semantic/evidence
      - docs/MASTER-PLAN.md
    summary: 审计 ACP/A2A/Channels/SDK 与 Eval/Improve/Evolution，并形成 Finding 修复 frontier
    verify: 协议角色/终态/恢复、质量观察/持久 mutation/批准审计及 workspace contract 形成当前 Audit；全部
      Finding 按风险、依赖、是否需人裁决和独立 repair 单元分类
artifact_id: plan:59db3ae8-df93-4d1e-ab1e-ab99afab87af
lifecycle: completed
design_revision: null
---
## Context

- Plan 03 已在 `echo-agent@d239b02b` 建立 10 张顶层 Capability Map + SDK 子图、26 个 Asset 和 49 个 Finding，其中 44 个 open。
- `inventory_closure=closed` 与 `behavior_model_closure=closed` 只表示能力、已知问题和未知区可追踪；本 Plan 用定向 Audit 验证高风险故障假设，不修改业务代码。
- 现有 SDK facade Audit 只覆盖 SDK 子边界，不替代全 workspace 的运行时、持久化、权限、协议和质量审计。

## Approach

- 以“边界 × 风险视角 × source digest”为最小审计单元，每轮最多三个单元；不做一次覆盖全部仓库的泛化 code review。
- 每个单元从现有 Map/Finding 出发，沿真实生产入口追到状态 authority、失败恢复、外部副作用、测试/契约证据和未检查项。
- 源码已能反证的问题保留或新增 Finding；只有产品预期、行为冲突或接受风险无法由源码决定时进入 semantic-decide。
- Audit 不修代码、不关闭未经 repair/verification/rereview 证明的 Finding；它只提升证据质量、合并重复问题并形成后续独立 repair frontier。
- 业界实现只用于后续关键设计取舍的参考，本 Plan 不因外部系统做法覆盖 echo-agent 当前源码事实。

## Global Constraints

- 审计对象绑定 Plan 03 的源码摘要 `source:8b3972e1d2bc92f4ad59f511b6674eaaf243c1760f21973caf9e96558f71db90`；若业务源码变化则停止并刷新基线。
- 每轮最多三个“boundary × risk lens × revision”单元，且必须包含可证伪假设、入口、状态、恢复、副作用、证据、Finding、残余风险和未检查项。
- 审计阶段只修改 `.echo-semantic/{audits,findings,maps,evidence}` 与顶层阶段状态，不修改 Rust、Cargo、SDK source/contract、examples、tests 或 website。
- Framework public capability 不因 echo-agent-cli 未采用而判死；EKO Workspace/Device/UI 策略不进入 framework authority。
- 只有 Subagent 术语；TaskRun、PlanTask、SubagentRun、SubagentAttempt 与内部 retry ordinal 必须保持限定 identity。
- 本地个人助理边界继续区分 Agent 自动 effect、caller-owned framework primitive、trusted 用户扩展与 direct-user 产品操作，不引入线上多租户式门控。
- Audit 通过不等于 Finding resolved；每个独立修复必须先插入 delivery map，并创建单独 Plan。

## Files

- Create: `echo-agent/.echo-semantic/audits` — 为高风险边界和风险视角建立当前 revision 的可反证审计。
- Modify: `echo-agent/.echo-semantic/findings` — 补充证据、合并重复候选、记录新反例和 audit 引用，不提前关闭问题。
- Modify: `echo-agent/.echo-semantic/maps` — 将 Audit 和审计后 residual risk 回链到对应场景。
- Modify: `echo-agent/.echo-semantic/evidence` — 记录当前源码、测试、契约和运行证据的真实覆盖与限制。
- Modify: `docs/MASTER-PLAN.md` — 更新审计阶段的真实进度、Finding frontier 和下一入口。

## Reuse

- `echo-agent/.echo-semantic/baseline.md` — 全 workspace 路径、边界和八风险视角覆盖权威。
- `echo-agent/.echo-semantic/maps` — 11 张 Capability Map 提供审计入口和已知状态权威。
- `echo-agent/.echo-semantic/findings` — 44 个 open Finding 是故障假设种子，不作为未经复核的结论。
- `echo-agent/.echo-semantic/evidence` — 当前源码、ADR、tests/examples 与 contract 的结构化引用。
- `echo-agent/.echo-semantic/audits/audit.sdk-facade-plan08-final.md` — 复用 Audit 结构，不复用其 SDK 局部结论。
- `docs/supreme/plans/2026-09-13T1448-echo-agent全workspace语义治理/delivery-map.md` — 限定本 Plan 只交付 high-risk-boundary-audits outcome。

## Todos

### audit-runtime-authorities

requirements:
- 用户确认：项目级治理以 Capability、Behavior、Rule、State Authority、Lifecycle、Finding、Evidence 为主单位。
- Plan 03 基线：Agent/Turn、Context/Memory、Task/Subagent/Workflow 的 needs_review 与 open Finding 必须经高风险审计。

interfaces:
- consumes: `map.agent-session-turn`、`map.context-memory`、`map.task-subagent-workflow`、相关 Assets/Findings 与当前源码摘要。
- produces: runtime authority/time/failure audit objects，更新后的 Finding evidence/audit refs 和 residual risk。

steps:

1. 审计 Agent/Turn state authority、Context/Memory data durability、Task/Subagent state authority 三个单元。
   verify: 每个单元给出可证伪假设、raw/driven route、scope/generation、claim/attempt 或 checkpoint producer 的实际路径。
   expected: Turn route、transcript settlement、Task claim/attempt 等 Finding 被确认、收窄、拆分或标记需裁决，且不凭名称归并 authority。

2. 审计 Task/Workflow failure concurrency、Workflow/Scheduler time lifecycle、BackgroundTask/CommandCell terminal ownership 三个单元。
   verify: race、cancel/abort、claim crash、cache/store、wait/panic 和 shutdown 场景都有真实 source/test evidence 与未检查项。
   expected: 运行时修复 frontier 能按独立行为切片，不把 Workflow、Scheduler、BackgroundTask、CommandCell 合成一个 executor。

### audit-persistence-and-effects

requirements:
- 用户要求：明确 Journal、Projection、Trace、Delivery Ledger 谁是事实源，以及 Tool/Permission/Sandbox/Effect 的策略和清理责任。
- AGENTS.md：direct-user 与 Agent 自动权限分离，只有本地仍成立的数据保护才可成为强制门禁。

interfaces:
- consumes: `map.observation-persistence-delivery`、`map.tool-permission-sandbox`、相关 Findings、ADR 0002/0005/0007/0019/0025/0030。
- produces: persistence authority 与 effect lifecycle audit objects，backend-specific retention/redaction 与 cleanup disposition。

steps:

1. 审计 Observation state authority、data durability 与完整 event-family classification 三个单元。
   verify: 每类 Journal/Envelope/live bus/projection/checkpoint/trace/delivery 的 ordering、retention、replay、gap 和 terminal 属性均从 producer/consumer 反证。
   expected: 不再把所有 event family 宣称为 journal-backed；trace/audit secret retention 按具体 backend 记录。

2. 审计 Tool permission_external、failure_concurrency 与 result_side_effect 三个单元。
   verify: ReactAgent policy、programmatic ToolManager、trusted Hook、cache/validation、approval、sandbox fallback、artifact/CommandCell cleanup 分别追到真实 effect。
   expected: 找到的重复 authority、bypass 或 cleanup debt 各自进入 Finding；不把 direct-user surface纳入 Agent permission。

### audit-extension-and-provider

requirements:
- 用户要求：治理 MCP 生命周期、工具调用、Hook、Outbox、资源传输和 LLM/provider 合同。
- Plan 03 基线：extension 与 provider 当前均为 needs_review，并已有 owner、capability、cancel、cleanup 与文档漂移 Finding。

interfaces:
- consumes: `map.extension-lifecycle`、`map.llm-provider-runtime`、extension/provider Assets、Findings、协议与测试证据。
- produces: extension owner/lifecycle/permission 与 provider contract/failure audit objects。

steps:

1. 审计 Extension state_authority、time_lifecycle 与 permission_external 三个单元。
   verify: MCP/Hook/Skill/Plugin/LSP 的 discover/prepare/publish/replace/withdraw/close、owner/generation、credential 与 trusted effect 均有 source/test disposition。
   expected: 双 registry、Hook precedence、Plugin/MCP collision、LSP/SSE cleanup 等 Finding 具有准确 owner 与独立修复边界。

2. 审计 LLM contract_evidence、failure_concurrency 与 time_lifecycle 三个单元。
   verify: structured output、capability/config/profile、non-stream/stream cancel、timeout、malformed/truncated terminal 与 usage 的 provider 对等性均被反证。
   expected: provider adapter 与 harness policy 不再混为一层；快速变化模型事实保留明确 override/刷新责任。

### audit-protocol-quality-frontier

requirements:
- 用户要求：治理 SDK/Contracts、Product boundary、Delivery/Observability 以及完整语义优化进度。
- Plan 03 基线：Protocol 与 Eval/Evolution 有 A2A/Channel/trace/timeout/panic/mutation/audit 等 open Finding。

interfaces:
- consumes: `map.protocol-surfaces`、`map.eval-evolution`、`map.workspace-architecture`、SDK 子图和全部审计产物。
- produces: protocol/quality/workspace Audit，去重后的当前 Finding 集合及可插入 delivery map 的独立 repair frontier。

steps:

1. 审计 Protocol state_authority、time_lifecycle 与 contract_evidence 三个单元。
   verify: ACP/A2A/Channel/Headless/SDK 的 session/run/task/turn identity、terminal、cancel/drop/close/replay 与 attachment projection 均沿 wire 和内部 adapter 验证。
   expected: 固定协议 wire name 与内部 authority 清楚分离；A2A/Channel 不再被描述为已共享 driven Turn。

2. 审计 Eval/Evolution data_durability、failure_concurrency 与 permission_external 三个单元。
   verify: trace identity、eval timeout、improve split/temp、memory/Skill mutation、ChangeLog、approval/security/rollback 均有 failure path 与证据。
   expected: 质量 observation 与持久 mutation 继续分离；需人裁决的 promotion/authorization 进入 semantic-decide。

3. 审计 Workspace contract_evidence，并归并全部 Audit/Finding frontier。
   verify: 44 个既有 open Finding 每项都有 current audit ref 或明确 defer 理由；新增 Finding 不重复已有语义；每个 repair 候选可独立合并、验证和停止。
   expected: MASTER-PLAN 能报告真实审计覆盖、Finding 分类和下一批独立 repair，不使用 SDK identity 数量作为项目完成度。

## Decisions

- 本 Plan 只形成 Audit 与 Finding frontier，不实施任何 repair。
- 审计顺序按状态权威与生命周期依赖组织，不按文件、crate 或 Finding 数量排序。
- 只有无法从源码/契约/测试确定的产品预期才进入 semantic-decide；实现反例不要求用户先裁决。
- 当前源码摘要未变化，因此 Audit revision 继续绑定 Plan 03 的 source digest；semantic-only commit SHA 不替代源码摘要。