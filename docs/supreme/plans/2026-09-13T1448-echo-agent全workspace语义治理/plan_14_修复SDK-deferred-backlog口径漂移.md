---
schema_version: 3
supersedes: null
slug: echo-agent全workspace语义治理/修复SDK-deferred-backlog口径漂移
goal: "修复Issue #116记录的SDK backlog语义漂移，使workspace discovery、protocol map与ADR
  0032使用同一deferred capability口径"
ships: 将全workspace discovery和protocol map中的SDK backlog统一为1441个deferred
  identity的capability级决策，并明确Host/Rust-only、language intrinsic与internal
  helper不属于语言parity backlog
verify: "Issue #116与Finding唯一映射；两处旧4076 intrinsic backlog改为1441 deferred
  capability backlog；Host/Rust-only、language intrinsic和internal
  helper明确排除；strict snapshot、task-scoped
  change-evidence、continuity与独立rereview通过"
design_ref: null
delivery_ref: docs/supreme/plans/2026-09-13T1448-echo-agent全workspace语义治理/delivery-map.md#repair-sdk-deferred-backlog-count-drift
todos:
  - id: record-finding-authority
    files:
      - echo-agent/.echo-semantic
    summary: "将Issue #116写回唯一Finding并绑定现有SDK scope权威"
    verify: Finding ID、Issue marker和URL唯一，status在修复复审前保持open
  - id: align-deferred-backlog
    files:
      - echo-agent/.echo-semantic
    summary: 将workspace discovery和protocol map中的旧intrinsic总量口径统一为deferred capability
      backlog
    verify: 两处均为1441 deferred identity且明确其它scope不是语言parity
      backlog；不修改manifest、SDK、runtime或公共文档
  - id: verify-and-rereview
    files:
      - echo-agent/.echo-semantic
    summary: 补齐repair、verification和独立rereview证据并重新验证最终continuity
    verify: "strict snapshot、HEAD change-evidence和b21
      continuity通过；reviewer零finding；Finding转resolved但Issue #116保持OPEN等待远端main"
artifact_id: plan:356f8f73-33fb-423b-a5d4-13bc6b38247c
lifecycle: completed
design_revision: null
---
## Context

- 最终独立复审发现`discovery.workspace-baseline`和`map.protocol-surfaces`仍使用4076个intrinsic identity的旧backlog口径。
- ADR 0032、SDK manifest和当前SDK map已将唯一consumer-facing backlog收敛为1441个`deferred` identity；其它scope不是语言parity backlog。
- GitHub Issue #116已建立并包含唯一Finding marker。本Plan只修语义材料，不改runtime、API、协议、manifest或语言SDK。

## Approach

- 新建`finding.sdk-deferred-backlog-count-drift`并回写Issue #116。
- 在两处受影响语义对象中使用ADR 0032的1441 deferred capability口径，保留具体capability分组和外部用户价值仍待决。
- 记录repair和verification Evidence，复用最终独立review形成rereview Audit；通过后将Finding置为resolved，Issue保持OPEN直到远端main。

## Global Constraints

- 只修改`echo-agent/.echo-semantic`，不修改Rust、Cargo、contracts、SDK、examples或正式docs。
- 不把1441个deferred identity解释为1441个逐项任务，也不把其它scope重新纳入parity backlog。
- 不关闭Issue #116，不执行push、PR、merge、release或cleanup。

## Files

- Modify: `echo-agent/.echo-semantic` — Finding、两处backlog口径、repair/verification Evidence与rereview Audit。

## Reuse

- `docs/adr/0032-sdk-contract-scope-classification.md` — identity级scope与1441 deferred权威。
- `.echo-semantic/discovery/discovery.sdk-facade-baseline.md` — 已正确使用1441 deferred capability口径的同仓参考。
- `.echo-semantic/maps/map.sdk-facade-parity.md` — 已明确其它scope不是逐identity语言任务。

## Todos

### record-finding-authority

requirements:
- Issue #116及一Finding一Issue规则

interfaces:
- consumes: reviewer finding、ADR 0032和GitHub Issue #116
- produces: 唯一Finding对象及外部Issue URL

steps:

1. 创建Finding并绑定Issue、boundary、Behavior、Rule和现有证据。
   verify: strict结构校验通过，Issue marker和URL唯一。
   expected: 新语义漂移可独立追踪且修复前保持open。

### align-deferred-backlog

requirements:
- ADR 0032的consumer-facing SDK scope分类

interfaces:
- consumes: workspace discovery和protocol map的两处4076 intrinsic旧口径
- produces: 统一的1441 deferred capability backlog表述

steps:

1. 替换两处旧总量并明确非deferred scope不是语言parity backlog。
   verify: 受影响对象不再出现4076，且1441只表达capability级待决。
   expected: discovery、protocol map、SDK map和ADR使用同一backlog权威。

### verify-and-rereview

requirements:
- Echo Semantic关闭条件和最终验收continuity

interfaces:
- consumes: 修复diff、Issue状态、最终验证日志和continuity候选
- produces: repair/verification Evidence、独立rereview Audit和resolved Finding

steps:

1. 运行strict、task-scoped change-evidence、Issue对账和b21 continuity并交给独立review。
   verify: 所有命令exit 0且reviewer Critical/Important/Minor为0。
   expected: 口径修复有可反证证据且不改变runtime合同。

2. 回写三类关闭refs并将Finding置为resolved。
   verify: Finding三类refs非空，Issue #116仍OPEN。
   expected: 本地语义层关闭，远端交付生命周期继续显式。

## Decisions

- 本问题是semantic evidence drift，不是SDK runtime或manifest缺陷。
- 1441 deferred identity只按capability进入后续合同决策；不按identity计数推进。
