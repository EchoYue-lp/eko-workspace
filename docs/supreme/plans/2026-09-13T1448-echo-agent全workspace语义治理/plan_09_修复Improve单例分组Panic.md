---
schema_version: 3
supersedes: null
slug: echo-agent全workspace语义治理/修复Improve单例分组Panic
goal: 让ImprovementLoop对合法单例criteria分组给出确定train/holdout disposition而不触发panic
ships: ImprovementLoop 对单例 criteria 分组采用 train-only disposition，消除合法单 case 输入的
  panic，同时保持多例分组的独立 holdout
verify: 旧实现单例输入稳定panic的red与新实现单例/混合group/ratio边界green闭合；Improve focused
  test、Clippy、no-default feature check、SDK artifact零diff、语义Finding和独立review全部通过
design_ref: null
delivery_ref: docs/supreme/plans/2026-09-13T1448-echo-agent全workspace语义治理/delivery-map.md#repair-improve-singleton-split
todos:
  - id: repair-singleton-split
    files:
      - echo-agent/src/improve/loop.rs
    summary: 修复单例criteria分组并覆盖train-only与多例holdout边界
    verify: 合法单例不panic且只进入train，多例group保持train/holdout非空，ratio边界确定
  - id: close-singleton-finding
    files:
      - echo-agent/.echo-semantic
      - docs/MASTER-PLAN.md
    summary: 写入repair、verification和独立复审并只关闭Improve单例panic Finding
    verify: 目标Finding resolved具备三类关闭refs；其它Finding不变；semantic gates通过
artifact_id: plan:b4f1627e-9895-490b-a5e6-74dc20a08112
lifecycle: completed
design_revision: null
---
## Context

- `finding.improve-single-case-panic`确认`split_idx.clamp(1, 0)`可由合法单例criteria分组稳定触发panic，违反仓库对外部输入的硬约束。
- `stratified_split`是`ImprovementLoop::run/run_async`的唯一分组实现；没有第二splitter、store或状态机需要新增。
- 现有注释要求stratification防过拟合，test阶段blind且不向suggestion生成器暴露，因此单例不能同时复制到train与holdout。

## Approach

- 先加入直接调用私有`stratified_split`的单例回归测试，在旧实现上取得非零panic证据。
- 每个criteria group只有一个case时将其放入train，holdout为空；两项及以上继续按`holdout_ratio`计算，并保证train与holdout各至少一项。
- 使用`into_iter().enumerate()`按`split_idx`push到两侧，不使用`group[..index]`直接切片。
- 增加单例、混合criteria group、0/1及越界ratio的边界断言，证明全部输入均有确定disposition且多例holdout独立。
- 不修改`run/run_async`签名、LoopResult、EvalReport、SDK route、持久化或workspace cleanup；相邻`improve-iteration-config`与`eval-workspace-generation-isolation`继续open。

## Global Constraints

- 合法EvalCase输入不得panic，不使用unwrap/expect/panic/直接索引或字节切片。
- 单例只进入train，绝不同时进入holdout；group长度>=2时两侧都非空。
- 不新增依赖、public API、wire/schema、SDK identity、状态权威或近义概念。
- 只关闭`finding.improve-single-case-panic`；其它82个非目标open Finding保持状态。
- 正式文档、examples、echo-agent-cli与echo-website不修改：公开调用形状和已描述的analysis-only行为不变，具体边界由源码注释、测试与语义Evidence覆盖。

## Files

- Modify: `echo-agent/src/improve/loop.rs` — 单例/多例分组算法与边界测试。
- Modify: `echo-agent/.echo-semantic` — current snapshot、repair/verification/rereview Evidence/Audit及Finding关闭。
- Modify: `docs/MASTER-PLAN.md` — 更新修复提交、Finding计数与下一Frontier。

## Reuse

- `src/improve/loop.rs` — `ImprovementLoop::criteria_variant`与唯一`stratified_split`继续拥有分组语义。
- `EvalCase`与`SuccessCriteria` — 复用真实输入类型构造边界测试，不建测试DTO。
- `audit.eval-evolution.failure-concurrency`、`finding.improve-single-case-panic`与`evidence.provider-protocol-quality` — 故障基线和关闭目标。

## Todos

### repair-singleton-split

requirements:
- 用户要求按Finding驱动统一全workspace语义，并消除合法输入panic。
- 当前blind holdout约束禁止把单例同时用于train与test。

interfaces:
- consumes: `EvalCase`、`SuccessCriteria`、`holdout_ratio`与唯一`stratified_split`。
- produces: singleton train-only与multi-case nonempty holdout disposition。

steps:

1. 添加单例criteria回归并在旧实现上记录panic red。
   verify: 一个合法EvalCase调用stratified_split时旧实现非零失败，失败点为clamp下界大于上界。
   expected: red直接证明Finding，不以静态推断替代。

2. 用长度分支和迭代分配修复split，覆盖单例、混合group与ratio边界。
   verify: 单例只在train；每个长度>=2的group在train/holdout各有case；0、1、负数与大于1 ratio不panic且disposition确定。
   expected: 不复制holdout、不直接slice、不改变公开结果类型。

3. 运行Improve focused test、Clippy、no-default improve feature check与SDK artifact零diff检查。
   verify: 相关命令全部exit 0且contracts/sdk、sdks/shared无diff。
   expected: 修复仅影响私有split行为。

### close-singleton-finding

requirements:
- Echo Semantic resolved Finding必须有repair、verification和独立rereview证据。

interfaces:
- consumes: red/green日志、最终diff与独立review。
- produces: resolved singleton Finding与current Eval/Improve semantic evidence。

steps:

1. 刷新Eval/Improve Behavior、Rule、Asset、Map与repair/verification Evidence。
   verify: before绑定`6d66479f`，after绑定current source digest；相邻Improve/Eval Finding不变。
   expected: 故障、disposition、验证、限制与回滚可追踪。

2. 独立review后写rereview Audit并关闭唯一目标Finding。
   verify: reviewer无blocker；Finding具备三类关闭refs；semantic strict/change-evidence通过。
   expected: repair可独立提交和停止。

## Decisions

- singleton选择train-only而非复制到holdout或返回新错误：现有public API不返回Result，analysis loop需要该case产生critique，同时blind holdout不能包含训练样本。
- 不与max_iterations、workspace generation或Eval timeout合并；它们具有独立结果、风险和验证面。