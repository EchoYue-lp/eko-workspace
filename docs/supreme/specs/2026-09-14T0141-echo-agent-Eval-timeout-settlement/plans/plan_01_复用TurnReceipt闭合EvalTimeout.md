---
schema_version: 3
supersedes: null
slug: echo-agent-Eval-timeout-settlement/复用TurnReceipt闭合EvalTimeout
goal: 让Eval timeout在取消后有界等待唯一TurnReceipt，再按settled状态决定trace评分与workspace回收
ships: Eval超时后取消并有界等待唯一TurnReceipt；已结算才读取终态trace并清理generation，未结算则跳过评分并保留隔离目录
verify: cancel-responsive与ignore-cancel两类timeout red/green证明同一drive
  future、6秒grace、receipt/trace/cleanup顺序；TurnDriver/Eval/Improve测试、双语文档、eval
  feature Clippy/check、SDK零diff、semantic/Issue门禁与独立review全部通过
design_ref: docs/supreme/specs/2026-09-14T0141-echo-agent-Eval-timeout-settlement/design.md
delivery_ref: null
todos:
  - id: drive-eval-through-turn-receipt
    files:
      - echo-agent/src/eval/runner.rs
      - echo-agent/src/agent/mod.rs
      - echo-agent/src/agent/react/run/mod.rs
      - echo-agent/src/agent/react/run/stream_channel.rs
    summary: 以唯一AgentTurnDriver和共享6秒grace替换Eval raw stream timeout归约
    verify: 主deadline借用pinned
      drive；cancel后继续等待同一future；receipt前不清理，grace失败不读取trace且保留generation
  - id: verify-and-document-timeout-settlement
    files:
      - echo-agent/docs/en/24-eval-system.md
      - echo-agent/docs/zh/24-eval-system.md
      - echo-agent/docs/adr/0037-eval-timeout-turn-settlement.md
    summary: 覆盖responsive/unresponsive取消、终态映射并同步双语与ADR
    verify: settled timeout仍失败但记录receipt且cleanup；unsettled
      timeout在6秒后有界返回并retain；正常terminal行为不回归，文档对等
  - id: close-eval-timeout-finding
    files:
      - echo-agent/.echo-semantic
      - docs/MASTER-PLAN.md
    summary: "写入repair、verification和独立复审并只关闭Issue #48对应Finding"
    verify: 目标Finding具备三类关闭refs；92个Finding与Issue一对一；其它76个open Finding不变；semantic
      gates通过
artifact_id: plan:a140f8d3-d18b-44c6-a1e6-cd9cde8d065d
lifecycle: completed
design_revision: sha256:4529bd7e0951ea27bc8e907e5cc2cd9741c70744b45b1b14e40b83e05d805732
---
## Approach

- 把ReactAgent stream reaper现有6秒值提升为root crate内部AGENT_CANCELLATION_SETTLE_PERIOD，Eval与stream reaper共同使用，不新增public配置。
- Eval构造TurnRequest Execute和无状态EvalEventSink，唯一AgentTurnDriver负责EventEnvelope与TurnReceipt；删除raw execute_for_final_answer归约。
- AgentTurnDriver.drive只创建一次并pin。主timeout借用该future；deadline后cancel，再以共享6秒grace继续借用同一future。
- Grace内receipt仍输出Timeout失败，记录settled status；只有receipt存在才加载终态trace并close generation。Grace超时skip trace/criteria并keep workspace。
- 增加cancel-responsive与ignore-cancel可控Agent；前者在cancel后发Cancelled并证明Eval等待receipt，后者证明6秒有界返回与retain。正常Completed/Failed/EOF保持driver合同。
- 同步双语Eval文档与ADR 0037；public shape不变，SDK artifacts零diff；website按冻结策略不动。
- 独立review后只关闭finding.eval-timeout-settlement；Issue #48保持open直到远端main交付。

## Global Constraints

- AgentTurnDriver/TurnReceipt是唯一Turn终态；Eval不得保留第二套AgentEvent terminal reducer。
- 主deadline和settlement grace必须poll同一个pinned drive future，不得重启Agent或spawn Eval私有driver registry。
- CancellationToken只表示请求；receipt之前不得把cancel调用视为settled。
- 共享grace固定沿用现有6秒值；stream reaper与Eval只能有一个root crate内部常量。
- Deadline后的Completed也保持Timeout失败，不运行success criteria；只有settled receipt允许读取终态trace并cleanup。
- Grace失败不得读取RunStore或close generation，必须记录unsettled并保留路径；caller-drop继续沿用Plan14 guard。
- 不新增public字段、状态机、依赖、wire/schema或SDK identity，不改变EvalResult格式、grader和run_all策略。
- docs/en与docs/zh保持对等；echo-agent-cli与echo-website不修改。
- 不使用unwrap/expect/panic/unreachable/todo、UTF-8字节切片或Worker术语。
- Issue #48已存在且唯一；只关闭该Finding，其它open Finding不顺带处理。

## Files

- Modify: `echo-agent/src/eval/runner.rs` — TurnDriver、pinned timeout/grace、receipt映射、trace/cleanup顺序与测试。
- Modify: `echo-agent/src/agent/mod.rs` — root crate内部共享6秒settlement常量。
- Modify: `echo-agent/src/agent/react/run/mod.rs` — 删除局部重复常量。
- Modify: `echo-agent/src/agent/react/run/stream_channel.rs` — 使用共享settlement常量。
- Modify: `echo-agent/docs/en/24-eval-system.md` — deadline/settlement/cleanup合同。
- Modify: `echo-agent/docs/zh/24-eval-system.md` — 同步中文合同。
- Create: `echo-agent/docs/adr/0037-eval-timeout-turn-settlement.md` — driver复用、grace和失败边界。
- Modify: `echo-agent/.echo-semantic` — source snapshot、repair/verification/rereview和Finding关闭。
- Modify: `docs/MASTER-PLAN.md` — 更新修复进度与下一frontier。

## Reuse

- `AgentTurnDriver`、`TurnRequest`、`TurnOutcome`、`TurnReceipt`与`EventIdentity` — 唯一finite Turn驱动与终态。
- ReactAgent`STREAM_CANCELLATION_SETTLE_PERIOD=6s` — 提升为共享内部策略，不复制magic number。
- Plan14 `EvalWorkspaceGeneration` — settled close、unsettled keep和caller-drop retain。
- `CancellationToken`与Tokio对pinned future的borrowed timeout — cancel request与continued settlement observation。
- Codex TurnAborted、Inspect interrupted cleanup与Issue #48 — 设计依据和关闭目标。

## Todos

### drive-eval-through-turn-receipt

requirements:
- § 目标行为
- § 系统边界
- § 核心结构与数据流
- § 复用与实现约束

interfaces:
- consumes: raw Agent、Eval invocation、AgentTurnDriver、共享settlement grace和EvalWorkspaceGeneration。
- produces: 单一pinned drive、optional TurnReceipt、settled flag与trace/cleanup disposition。

steps:

1. 添加cancel-responsive Agent red：deadline后Agent完成取消副作用并发Cancelled，但旧Eval不等待receipt。
   verify: 旧实现返回时无settled receipt且generation保持；失败来自driver future被drop而非测试sleep。
   expected: Issue #48有确定性producer未结算证据。

2. 提升共享6秒常量并把Eval切到AgentTurnDriver；主timeout与grace借用同一Pin。
   verify: responsive Agent在runner返回前settled，结果仍Timeout且workspace关闭；ignore-cancel Agent在grace后返回unsettled并retain。
   expected: 一个driver、一个receipt、一个grace authority，不重启Agent。

3. 按receipt重排final output、trace、criteria和cleanup。
   verify: Completed/Cancelled/Failed/EOF映射由driver唯一处理；unsettled不加载trace/criteria；settled timeout可读终态trace但不转成功。
   expected: 评分不领先于终态，cleanup不领先于producer settlement。

### verify-and-document-timeout-settlement

requirements:
- § 异常与边界场景
- § 关键取舍与业界依据
- § 公共合同与兼容
- § 验收标准

interfaces:
- consumes: Eval timeout settlement结果与共享6秒grace。
- produces: deterministic tests、双语docs与ADR 0037。

steps:

1. 覆盖responsive/unresponsive timeout及非timeout terminal回归。
   verify: receipt status、结果failure、trace读取资格、workspace存在性和bounded elapsed分别可观察。
   expected: cancel不被误作terminal，grace成功/失败分支互斥且闭合。

2. 更新双语Eval文档并写Accepted ADR 0037。
   verify: 两种deadline、TurnReceipt、trace/criteria与workspace顺序在en/zh一致；文档合同通过。
   expected: 行为变化、延迟上界、兼容、残余与回滚可追踪。

3. 运行TurnDriver/Eval/Improve tests、eval feature Clippy/check和SDK零diff。
   verify: 全部exit 0；contracts/sdk与sdks/shared无差异。
   expected: 唯一Turn authority接入不改变public identity或其它Eval行为。

### close-eval-timeout-finding

requirements:
- § 验收标准
- 用户要求语义修复严格执行一Finding一Issue。
- Echo Semantic resolved Finding关闭合同。

interfaces:
- consumes: red/green日志、ADR 0037、最终diff、Issue #48与独立review。
- produces: resolved timeout Finding、repair/verification Evidence、rereview Audit与MASTER-PLAN。

steps:

1. 刷新Agent Turn/Eval Map、Behavior、Rule、Asset与source snapshot，写repair/verification Evidence。
   verify: before绑定29a00f66，after绑定current source digest；Issue #48 URL唯一；其它Finding状态不变。
   expected: deadline、cancel、receipt、trace与cleanup顺序可追踪。

2. 独立review后写rereview Audit并执行最终语义与Issue门禁。
   verify: reviewer无blocker；strict snapshot、change evidence、Issue reconciliation和git diff check通过。
   expected: Finding本地resolved，Issue #48仍open等待远端main交付。

## Decisions

- Eval从raw Agent stream consumer升级为AgentTurnDriver consumer，不复制终态归约。
- 6秒grace复用stream reaper现有值，不新增public tuning knob。
- Timeout是deadline事实，即使grace内receipt Completed也不改判成功。
- Grace超时是显式unsettled结果：skip trace/criteria、retain workspace，不无限等待。
- 本Plan只关闭Issue #48。
