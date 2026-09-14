---
schema_version: 3
supersedes: null
slug: echo-agent全workspace语义治理/修复Improve迭代配置传递
goal: 让EvalDrivenImprovement的public max_iterations配置真实控制唯一ImprovementLoop的执行次数
ships: EvalDrivenImprovement 将 public max_iterations 配置无损传入唯一
  ImprovementLoop，包含零迭代语义，并使实际成本与调用方请求一致
verify: 旧实现配置2却执行5次的red与新实现2次/0次green闭合；focused test、Improve feature
  Clippy/check、SDK artifact零diff、语义Finding和独立review全部通过
design_ref: null
delivery_ref: docs/supreme/plans/2026-09-13T1448-echo-agent全workspace语义治理/delivery-map.md#repair-improve-iteration-config
todos:
  - id: wire-iteration-config
    files:
      - echo-agent/src/improve/eval_improvement.rs
    summary: 把public max_iterations直接传入唯一ImprovementLoop并覆盖2次与0次边界
    verify: 配置2产生2个iterations，配置0不构造Agent且返回空iterations，既有disabled/empty行为不变
  - id: close-iteration-finding
    files:
      - echo-agent/.echo-semantic
      - docs/MASTER-PLAN.md
    summary: 写入repair、verification和独立复审并只关闭Improve迭代配置Finding
    verify: 目标Finding resolved具备三类关闭refs；其它Finding不变；semantic gates通过
artifact_id: plan:b826c8f0-2bd8-4d47-a08d-fbe2270662c5
lifecycle: completed
design_revision: null
---
## Context

- `finding.improve-iteration-config`确认public `max_iterations` setter只更新`EvalDrivenImprovement`字段，`run`却使用`ImprovementLoop::new()`默认5次。
- docs/en与docs/zh已有`.max_iterations(5)`公开用法，本切片使实现符合现有说明，不改变调用形状或新增文档概念。
- Plan 09已在`2bac0580`修复singleton split；iteration wiring是独立行为，不改变train/holdout策略。

## Approach

- 先增加真实`EvalDrivenImprovement::run`测试：一个始终失败的case配`max_iterations(2)`，旧实现返回5个iterations并red。
- 用`ImprovementLoop { max_iterations: self.max_iterations, ..Default::default() }`把字段直接传给唯一loop，不新增builder、adapter或第二配置。
- 增加`max_iterations(0)`边界，要求返回`Some(LoopResult)`且iterations为空、Agent factory未调用；0明确表示调用方请求不运行迭代。
- 保持`enabled=false`和empty cases返回None、report generation、threshold、holdout与workspace行为不变。
- Public类型、setter签名与SDK route不变，contracts/sdk与sdks/shared必须零diff。

## Global Constraints

- `self.max_iterations`是EvalDrivenImprovement到ImprovementLoop的唯一迭代次数输入，不复制或重命名字段。
- 0不归一化为1或默认值；调用方显式0必须产生零迭代且不构造Agent。
- 不使用unwrap/expect/panic/直接索引，不新增依赖、状态权威、wire/schema或SDK identity。
- 只关闭`finding.improve-iteration-config`；singleton已解决，workspace generation、Eval timeout及其它81个非目标open Finding保持状态。
- docs/examples/echo-agent-cli/echo-website不修改：公开用法与合同文本已经正确，本次仅修复内部wiring。

## Files

- Modify: `echo-agent/src/improve/eval_improvement.rs` — max_iterations传递与真实pipeline边界测试。
- Modify: `echo-agent/.echo-semantic` — current snapshot、repair/verification/rereview Evidence/Audit及Finding关闭。
- Modify: `docs/MASTER-PLAN.md` — 更新修复提交、Finding计数与下一Frontier。

## Reuse

- `EvalDrivenImprovement.max_iterations`与`ImprovementLoop.max_iterations` — 同类型现有字段直接传递。
- `EvalDrivenImprovement::run`、`EvalCase`、`SuccessCriteria`与testing `MockAgent` — 复用真实执行链，不增加测试专用production accessor。
- `audit.eval-evolution.failure-concurrency`、目标Finding与`evidence.provider-protocol-quality` — 故障基线和关闭对象。

## Todos

### wire-iteration-config

requirements:
- 用户要求按Finding修复重复或断裂语义，public配置必须真实控制执行。
- 0值必须保持调用方明确语义，不能静默回退default。

interfaces:
- consumes: `EvalDrivenImprovement.max_iterations`与`ImprovementLoop`现有public field。
- produces: run pipeline实际iteration count等于configured max。

steps:

1. 添加配置2次的真实run回归并记录旧实现5次red。
   verify: enabled engine、非空失败case和max_iterations(2)最终iterations长度必须为2。
   expected: 旧实现断言失败并显示actual 5。

2. 直接传递配置并增加0边界。
   verify: 2产生2次；0产生Some空iterations且factory调用数为0；disabled/empty原行为不变。
   expected: 执行成本与调用方请求一致，无第二配置owner。

3. 运行EvalDrivenImprovement focused tests、Improve feature Clippy/check和SDK artifact零diff检查。
   verify: 全部exit 0且public contract artifacts不变。
   expected: 仅内部wiring变化。

### close-iteration-finding

requirements:
- Echo Semantic resolved Finding必须有repair、verification和独立rereview证据。

interfaces:
- consumes: red/green日志、最终diff与独立review。
- produces: resolved iteration Finding与current Eval/Improve semantic evidence。

steps:

1. 刷新Eval/Improve对象并写repair/verification Evidence。
   verify: before绑定`2bac0580`，after绑定current source digest；相邻Finding不变。
   expected: public输入、唯一consumer、边界行为与限制可追踪。

2. 独立review后写rereview Audit并关闭唯一目标Finding。
   verify: reviewer无blocker；Finding具备三类关闭refs；semantic strict/change-evidence通过。
   expected: repair可独立提交和停止。

## Decisions

- 直接构造唯一ImprovementLoop并填现有字段，不新增setter/getter/helper层。
- 0表示零迭代而非default或minimum one；它与public usize字段和Rust range语义一致。
- 不与workspace cleanup或Eval timeout合并，它们有独立生命周期与验证面。