---
schema_version: 3
supersedes: null
slug: echo-agent全workspace语义治理/统一StreamingTool参数校验
goal: 让ToolManager的streaming与non-streaming执行在任何effect前复用同一参数校验语义
ships: ToolManager 的 streaming 与 non-streaming 执行在 cache、permit 和外部 effect 前复用同一
  schema 与 custom parameter validation kernel
verify: 旧stream路径schema/custom
  invalid仍调用Tool的red与统一kernel后的对等green闭合；ToolManager focused test、echo_execution
  Clippy/check、SDK artifact零diff、语义Finding和独立review全部通过
design_ref: null
delivery_ref: docs/supreme/plans/2026-09-13T1448-echo-agent全workspace语义治理/delivery-map.md#repair-streaming-tool-validation
todos:
  - id: unify-stream-validation
    files:
      - echo-agent/echo-execution/src/tools.rs
    summary: 抽取并接入stream/non-stream/public validator共用的schema与custom validation kernel
    verify: 无效schema或custom参数在cache/permit/effect前同类失败且calls为0；valid stream不回归
  - id: close-stream-validation-finding
    files:
      - echo-agent/.echo-semantic
      - docs/MASTER-PLAN.md
    summary: 写入repair、verification和独立复审并只关闭Streaming Tool validation Finding
    verify: 目标Finding resolved具备三类关闭refs；其它Finding不变；semantic gates通过
artifact_id: plan:98b72d17-752f-4970-9d87-e49b34d1c6a6
lifecycle: completed
design_revision: null
---
## Context

- `finding.streaming-tool-validation`确认non-stream execute与public validator执行schema和custom validation，stream inner取得Tool后直接进入cache/permit/execute。
- 相同非法参数因此只在stream路径到达Tool effect，违反同一ToolManager的输入合同与permission-effect顺序。
- 现有`validate_parameters_against_schema`与`Tool::validate_parameters`足以复用，不新增validation类型或策略。

## Approach

- 先添加stream路径schema invalid与custom validator拒绝测试，记录旧实现实际调用Tool的red。
- 抽取私有async `validate_tool_input(tool, parameters)`，顺序固定为JSON Schema再custom validation。
- `execute_tool_inner`、`execute_tool_stream_with_context_inner`与`validate_tool_parameters_async`全部调用同一kernel；校验位于cancel、cache、semaphore、retry与任何Tool future之前。
- 保持stream timeout/retry/output/backpressure、non-stream cache与permission owner不变。
- Public API、Tool trait、error类型、SDK identity和文档说明不变，contracts/sdk与sdks/shared必须零diff。

## Global Constraints

- schema与custom validation失败不得取得permit、读写cache或调用execute/execute_stream。
- stream与non-stream必须返回同一现有typed error分类，不增加fallback或成功文本。
- 不使用unwrap/expect/panic/直接索引，不新增依赖、public API、状态权威、协议或第二validator。
- 只关闭`finding.streaming-tool-validation`；cache scope/invalidation、sandbox、permission及其它80个非目标open Finding保持状态。
- docs/examples/echo-agent-cli/echo-website不修改：现有Tool contract已经要求执行前validation，本次仅使stream实现符合合同。

## Files

- Modify: `echo-agent/echo-execution/src/tools.rs` — 单一validation kernel、stream接入与对等测试。
- Modify: `echo-agent/.echo-semantic` — current snapshot、repair/verification/rereview Evidence/Audit及Finding关闭。
- Modify: `docs/MASTER-PLAN.md` — 更新修复提交、Finding计数与下一Frontier。

## Reuse

- `validate_parameters_against_schema`与`Tool::validate_parameters` — 现有两级输入合同。
- `ToolManager::execute_tool_inner`、`execute_tool_stream_with_context_inner`、`validate_tool_parameters_async` — 三个真实consumer。
- `SchemaTool`、`ToolStreamEvent`与现有AtomicUsize test pattern — 复用真实stream执行与调用观测。
- `audit.tool-permission-sandbox.failure-concurrency`、目标Finding、`evidence.effects-extensions` — 故障基线与关闭对象。

## Todos

### unify-stream-validation

requirements:
- 用户要求同一语义只有一个authority，stream不得绕过non-stream输入合同。
- 无效参数必须在任何外部effect之前失败。

interfaces:
- consumes: `Tool`、`ToolParameters`、JSON Schema validator与custom validate future。
- produces: 三入口共用的私有async validation kernel。

steps:

1. 添加schema/custom invalid的stream red并观测Tool调用计数。
   verify: 旧实现stream返回成功或进入Tool，断言invalid error与calls=0失败。
   expected: red直接证明stream bypass。

2. 抽取kernel并接入三个入口。
   verify: schema先于custom，二者先于cancel/cache/permit/effect；valid stream仍转发并完成。
   expected: stream/non-stream对同一输入同分类且无第二validator。

3. 运行ToolManager focused tests、echo_execution Clippy/check与SDK artifact零diff检查。
   verify: 全部exit 0，无新增public identity或生成artifact。
   expected: 只改变stream invalid-input路径。

### close-stream-validation-finding

requirements:
- Echo Semantic resolved Finding必须有repair、verification与独立rereview证据。

interfaces:
- consumes: red/green日志、最终diff与独立review。
- produces: resolved stream validation Finding与current Tool semantic evidence。

steps:

1. 刷新Tool Behavior、Rule、Asset、Map并写repair/verification Evidence。
   verify: before绑定`57066461`，after绑定current source digest；cache/sandbox Finding不变。
   expected: input、ordering、error与验证限制可追踪。

2. 独立review后写rereview Audit并关闭唯一目标Finding。
   verify: reviewer无blocker；Finding具备三类关闭refs；semantic strict/change-evidence通过。
   expected: repair可独立提交和停止。

## Decisions

- 私有async kernel同时拥有schema和custom validation顺序；不把其中任一步复制到stream。
- 保持既有validation-before-cancellation顺序以实现non-stream对等，本Plan不改变permission/cancel策略。
- 不与cache scope/invalidation合并，它们位于validation之后且具有独立状态authority与并发验证。