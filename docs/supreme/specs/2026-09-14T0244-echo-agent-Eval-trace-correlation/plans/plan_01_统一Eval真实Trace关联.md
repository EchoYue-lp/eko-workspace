---
schema_version: 3
supersedes: null
slug: echo-agent-Eval-trace-correlation/统一Eval真实Trace关联
goal: 让Eval以invocation correlation解析并返回真实trace identity，消除product run ID与trace
  run ID混用
ships: Eval以每次invocation唯一run/turn/execution correlation从RunStore解析真实trace
  identity，使EvalResult、trace criteria、constraints和metrics消费同一个已验证Run
verify: 真实ReactAgent red/green证明legacy product run不再被当作trace
  ID，唯一/缺失/歧义/存储失败和unsettled
  timeout边界可观察；Eval/React/TurnDriver、双语文档、feature、SDK零diff、semantic/Issue与独立review门禁全部通过
design_ref: docs/supreme/specs/2026-09-14T0244-echo-agent-Eval-trace-correlation/design.md
delivery_ref: null
todos:
  - id: establish-eval-trace-correlation
    files:
      - echo-agent/src/eval/runner.rs
    summary: 以唯一value-scoped runtime identity关联并加载真实trace Run
    verify: 真实ReactAgent携带独立legacy product run时，EvalResult返回RunStore真实trace
      ID和metrics；零候选合法，歧义或存储不一致失败，unsettled零查询
  - id: verify-and-document-trace-identity
    files:
      - echo-agent/docs/en/24-eval-system.md
      - echo-agent/docs/zh/24-eval-system.md
      - echo-agent/docs/adr/0038-eval-trace-correlation-identity.md
    summary: 以确定性测试、双语文档和ADR冻结trace correlation合同
    verify: product/correlation/trace三类identity与正常、失败、timeout边界在测试和文档中一致，SDK public
      contract保持零差异
  - id: close-eval-trace-finding
    files:
      - echo-agent/.echo-semantic
      - docs/MASTER-PLAN.md
    summary: "写入repair、verification和独立复审并只关闭Issue #49对应Finding"
    verify: 目标Finding具备三类关闭refs；92个Finding与Issue一对一；其它75个open Finding不变；semantic
      gates通过
artifact_id: plan:d6ebad7f-e505-49ba-a2fa-a3f484201f70
lifecycle: completed
design_revision: sha256:b2677bff910f31cdaf59bda326ea330d7dc9556cd43090a8225cb64c1487bd28
---
## Approach

- 复用Eval现有每次run唯一identity value，以`EventIdentity::for_run`赋予同一formal run、turn和execution correlation，并通过`AgentInvocationContext.runtime`值传给Agent。
- ReactAgent继续创建真实trace `Run.run_id`；Eval只在TurnReceipt settled后调用既有`RunStore::list_by_parent_run`，再按parent/turn/execution tuple筛出唯一summary并load真实Run。
- `EvalResult.run_id`只保存已成功load的真实trace ID；同一Run实例用于trace criteria、constraints和metrics。零候选保持无trace语义，歧义/list/load/dangling summary形成明确失败。
- 删除Eval对`Agent::current_run_id()`的trace依赖；legacy product run仍属于Agent外部上下文，不被修改或误投影。
- 增加真实ReactAgent red/green与可控RunStore边界测试，同步双语Eval文档和Accepted ADR 0038；TurnReceipt、AgentEvent、RunStore trait及SDK artifacts不变。
- 独立review后只关闭`finding.eval-trace-identity`；GitHub Issue #49保持open直到提交进入远端main。

## Global Constraints

- Product/business run ID、Eval correlation和真实trace run ID是三种限定身份，不得复用同一个getter或字段含义。
- Correlation必须来自每次Eval invocation唯一值，并同时进入run、turn和execution；不得按case ID、时间、列表顺序或agent-wide共享状态猜trace。
- 只有唯一精确summary且`RunStore::load`成功时才能设置`EvalResult.run_id`并评分trace；歧义和存储不一致必须失败关闭。
- 零trace候选对不生产trace的Agent合法；依赖trace的criteria按既有缺失语义处理。
- Unsettled timeout不得调用RunStore list/load；settled timeout、Cancelled或Failed可读取诊断trace但不能改变原终态。
- 不新增public API、状态类型、依赖、wire/schema或SDK identity，不公开ReactAgent具体getter，不修改echo-agent-cli或echo-website。
- docs/en与docs/zh保持对等；ADR记录行业依据、identity authority、兼容和回滚。
- 不使用unwrap/expect/panic/unreachable/todo、UTF-8字节切片或Worker术语。
- Issue #49已存在且唯一；本Plan只关闭其本地Finding，远端Issue等待main交付。

## Files

- Modify: `echo-agent/src/eval/runner.rs` — correlation装配、真实trace解析、失败投影和测试。
- Modify: `echo-agent/docs/en/24-eval-system.md` — product/correlation/trace identity与缺失/歧义行为。
- Modify: `echo-agent/docs/zh/24-eval-system.md` — 同步中文合同。
- Create: `echo-agent/docs/adr/0038-eval-trace-correlation-identity.md` — 唯一correlation与RunStore解析决策。
- Modify: `echo-agent/.echo-semantic` — current snapshot、repair/verification/rereview Evidence/Audit和Finding状态。
- Modify: `docs/MASTER-PLAN.md` — 更新修复提交、Finding计数与下一frontier。

## Reuse

- `echo-agent/src/eval/runner.rs` — Eval现有unique identity、TurnReceipt settlement gate和单一Run评分路径。
- `echo-agent/echo-core/src/agent/event_envelope.rs` — `EventIdentity::for_run`同时建立run、turn和execution关联。
- `echo-agent/echo-core/src/tools/mod.rs` — `ExternalRunContext`提供spawn-safe value-scoped runtime identity。
- `echo-agent/src/agent/react/mod.rs` — `start_scoped_trace_run`生成真实trace ID并保存parent/turn/execution，不改变producer authority。
- `echo-agent/src/trace/mod.rs` — `RunStore::list_by_parent_run/load`与`RunSummary`提供既有correlation查询和真实ID。
- `echo-agent/src/agent/react/run/stream_channel.rs` — 现有value-scoped child trace测试证明生产路径可达。
- Issue #49、OpenTelemetry、OpenAI Agents SDK与Codex typed identity实现 — 故障基线和设计依据。

## Todos

### establish-eval-trace-correlation

requirements:
- § 目标行为
- § 系统边界
- § 核心结构与数据流
- § 异常和边界场景

interfaces:
- consumes: Eval unique identity、EventIdentity、ExternalRunContext、TurnReceipt、RunStore和RunSummary。
- produces: 唯一eval correlation、可选真实trace Run、明确trace lookup failure和真实EvalResult.run_id。

steps:

1. 添加真实ReactAgent red，设置与真实trace不同的legacy product run并附带usage，证明旧Eval返回/查询错误identity。
   verify: 失败必须来自EvalResult没有RunStore真实trace ID或metrics，而非mock编译、网络或手工伪造结果。
   expected: Issue #49具有可重复的生产路径反例。

2. 将Eval identity装配为同一run/turn/execution correlation，并在settled后按精确tuple解析唯一Run。
   verify: 真实trace ID进入EvalResult且对应Run可load；legacy product run不参与匹配；其它child summary不被误选。
   expected: producer trace ID与caller correlation分离，一个已验证Run成为评分唯一输入。

3. 闭合缺失、歧义、list/load/dangling与timeout分支。
   verify: 零候选不阻断无trace criteria；歧义和存储不一致产生violation并失败；unsettled list/load均为0；settled失败终态可保留诊断trace。
   expected: 不按顺序猜测，不吞掉authority冲突，也不把可选trace变成所有Agent强制能力。

### verify-and-document-trace-identity

requirements:
- § 关键取舍与业界依据
- § 范围与非目标
- § 复用与实现约束
- § 验收标准

interfaces:
- consumes: correlation解析行为、真实EvalResult.run_id与trace lookup failure。
- produces: deterministic回归测试、双语Eval说明和Accepted ADR 0038。

steps:

1. 覆盖真实ReactAgent、精确匹配、其它child、零候选、歧义、存储错误及settled/unsettled终态。
   verify: 每个场景观察result run_id、Run identity、metrics、violation和RunStore调用次数，测试无sleep和全局状态猜测。
   expected: 正常、失败和并发污染候选都有直接反证，不依赖静态阅读。

2. 更新双语Eval文档并创建ADR 0038。
   verify: en/zh对三类identity、lookup顺序、缺失/失败、兼容与回滚描述一致，documentation contract通过。
   expected: 正式文档不再暗示Agent current run等于trace run。

3. 运行Eval/React/TurnDriver、feature、SDK零diff和语义门禁。
   verify: 所有适用命令exit 0；contracts/sdk与sdks/shared无差异。
   expected: 修复不扩公共API，不回归Plan15 settlement与其它Eval/Improve行为。

### close-eval-trace-finding

requirements:
- § 验收标准
- § 复用与实现约束
- 用户要求语义修复严格执行一Finding一Issue。

interfaces:
- consumes: red/green日志、ADR 0038、最终diff、Issue #49和独立review。
- produces: resolved trace identity Finding、repair/verification Evidence、rereview Audit和MASTER-PLAN状态。

steps:

1. 刷新Eval、Agent Turn与Observation/Persistence语义对象，写repair/verification Evidence。
   verify: before绑定`8332345a`，after绑定current source digest；Issue #49 URL唯一；其它Finding状态不变。
   expected: product run、correlation、trace run和评分数据流可追踪。

2. 独立review后写rereview Audit并执行最终语义与Issue门禁。
   verify: reviewer无blocker；Finding具备repair/verification/rereview refs；92/92 Issue映射、strict snapshot、change evidence和git diff check通过。
   expected: Finding本地resolved，Issue #49仍open等待远端main交付。

## Decisions

- 使用调用方correlation查询producer生成的真实trace ID，不给TurnReceipt添加可选观测字段。
- Eval correlation同时占用formal run、turn和execution identity；真实trace ID只来自RunStore中的Run。
- 零候选合法；多个精确候选或存储不一致失败，不按时间或列表顺序选择。
- 本Plan只关闭Issue #49，SDK与website保持不变。
