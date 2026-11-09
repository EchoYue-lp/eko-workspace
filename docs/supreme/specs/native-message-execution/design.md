# 原生消息与统一执行终态设计

## 背景

当前 Framework 以 `ThinkTool` 和 `FinalAnswerTool` 作为可调用控制面，同时以 `ThinkStart`/`ThinkEnd` 推断文字块阶段。该模型把普通进度文字与 reasoning 混在同一事件序列中，并使“回答完成”依赖一个特殊工具。Provider 已经提供原生 reasoning、content、tool call、停止原因和签名/opaque block；Framework 应保留这些事实并让既有 ReAct、Guard、Critic、Stop Hook、持久化和结算继续拥有执行权威。

## 参考实现与取舍

- Anthropic 原生 extended thinking 以内容块表达 thinking、text、tool use，并要求签名 thinking 在后续请求中原样回传；本设计将其归一为带身份和 phase 的 assistant block，不把签名内容当普通进度。
- Codex/OpenAI Responses 以有序 response item/delta 和明确完成信号区分 reasoning、message、function call；本设计保留 provider item/request 身份、顺序和增量完成状态。
- Claude Code/Codex 的计划与执行由提示和运行时边界协作，不把“计划已批准”塞进额外运行时状态机；本设计复用现有 ReAct 主循环，不新建 executor 或结束权威。

## 分层与权威

Framework 只提供 provider-neutral 消息块、工具调用/结果身份、响应完整性判定和最终候选的通用生命周期。EKO 负责进度汇报阈值、GUI/IM/cron 呈现；provider 负责协议解析；SDK 负责无损 facade。`TaskRun`、`Agent` 运行结束和产品目标完成继续由各自现有权威分别判定。

## Canonical 合同

### AssistantMessageBlock

每个 assistant 输出块至少包含：

- `block_id`：provider item/block 或 Framework 生成的稳定回退身份；
- `request_id`：所属模型请求/响应身份；
- `sequence`：单次请求内的单调顺序；
- `phase`：`Reasoning`、`Commentary`、`PartialAnswer`、`FinalAnswer`、`Unknown`；
- `delta`：本次增量文字或空值；
- `complete`：该块是否已由 provider/运行时明确结束；
- `provider`：来源标识；
- `reasoning`：签名、redacted 或 opaque 原生块，不降格为普通文字。

### ExecutionStatus

运行时状态事件保留 `execution_id`、`request_id`、`sequence`、`phase`、`status`（`Started`/`Waiting`/`ToolPending`/`Candidate`/`Settled`/`Cancelled`/`Failed`）和可选原因。它是观察事实，不拥有 TaskRun 目标完成权。

### 完整响应与终态

1. 有待执行 tool call：先按 stable call id 发出/执行/回填结果，再请求模型。
2. 只有明确进度或 partial answer：保存并继续，不产生终态。
3. 明确 final 且无待执行工具，或无阶段 provider 明确正常完成且有正文：形成最终候选。
4. 最终候选依次进入既有结构化输出、Guard/Critic、Stop Hook、transcript persistence 和 `finalize_run`；Stop Hook 要求继续时保留同一 execution。
5. 截断、异常断流、空响应和未结算 owner 不能发布成功；成功终态和最终发布各自幂等。

## 迁移边界

- 具名退役 `ThinkTool`、`FinalAnswerTool` 的默认注册、曝光和特殊终态分支；保留 `thinking` 配置、provider 签名/opaque block、`AgentEvent::FinalAnswer` 作为运行时确认后的兼容投影。
- 新执行不调用控制工具；纯文字候选沿既有 `verify_final_text`/`emit_final_text` 路径。
- 旧日志和断点只走读取适配：旧 thinking/工具前说明标为历史执行说明，旧控制工具不重新注册、不重放副作用、不补成功。

## 阶段验收

Framework focused 测试覆盖纯文字、reasoning+文字、文字+tool、未知阶段、截断/断流、Stop Hook 继续和单次终态；provider/EKO/SDK 线路分别证明无损转换与同一事实消费。真实原生 host、GUI、IM/cron、SSH、三语言和 main CI 单列，不由源码测试替代。
