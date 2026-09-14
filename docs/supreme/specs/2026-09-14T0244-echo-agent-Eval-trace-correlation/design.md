---
title: echo-agent Eval trace correlation identity 设计
artifact: design
carrier: markdown
---

# echo-agent Eval trace correlation identity 设计

## 问题与目标

`EvalRunner`在Turn结束后调用`Agent::current_run_id()`并把返回值同时当作`EvalResult.run_id`和`RunStore::load`键。对ReactAgent，这个getter返回外部product/business run ID；真正的trace invocation ID由`start_scoped_trace_run`内部生成并保存在`Run.run_id`。因此Eval可能得到一个看似有效但无法加载的ID，导致ToolUsed、trace metrics和constraints看不到刚完成的真实Run。

目标是让每个Eval invocation用已有typed runtime identity建立唯一correlation，在Turn settled后从RunStore解析真实trace invocation，并把真实ID投影到EvalResult。修复不得读取ReactAgent共享可变trace字段，不得把product run与trace run重新合并，也不得为此扩张TurnReceipt或多语言SDK合同。

## 目标行为

- 每次`EvalRunner::run`生成唯一eval correlation，并用`EventIdentity::for_run`形成同一formal run、turn和execution identity。
- 同一identity通过`AgentInvocationContext.runtime`值传入Agent；working directory、cancel和其它既有invocation字段保持原权威。
- ReactAgent继续唯一创建真实trace `Run.run_id`；它把Eval correlation保存为`parent_run_id`，并把turn/execution identity保存到同一Run。
- Turn settled后，Eval只通过配置的RunStore和该correlation查询；不再调用`Agent::current_run_id()`获取trace link。
- 候选必须同时满足parent run、turn和execution identity。唯一候选成功load后，其真实`Run.run_id`写入`EvalResult.run_id`，同一个Run用于criteria、constraints和metrics。
- 没有候选表示Agent没有在该RunStore生产trace，保持`run_id=None`；依赖trace的criteria沿既有规则失败，不依赖trace的criteria仍可执行。
- 多个精确候选、list失败、唯一summary随后无法load或load失败属于trace authority不一致：Eval记录violation并失败，不任意选择一个Run。
- 未settled timeout继续完全跳过RunStore correlation、trace criteria和workspace cleanup，保持ADR 0037合同。

## 范围与非目标

范围包括Eval invocation identity装配、RunStore correlation与唯一性判定、EvalResult真实trace ID、正常/timeout/error测试、双语Eval文档、ADR、语义对象和Issue #49。

以下不在本设计内：

- 不给`TurnReceipt`、`AgentEvent`、`EventEnvelope`或`Agent` trait新增trace字段/getter。
- 不公开ReactAgent的`current_trace_run_id`，不让Eval向下转型具体Agent。
- 不改变`Run.run_id`生成策略、RunStore持久格式、trace事件、retention或exporter delivery。
- 不把Eval correlation伪装成真实trace ID；它只作为parent/turn/execution关联键。
- 不修复其它调用方的trace identity、terminal commit/projection顺序或外部telemetry协议。
- 不修改echo-agent-cli或echo-website；website继续按当前冻结策略延后。

## 系统边界

EvalRunner拥有本次评估的correlation和质量投影；AgentTurnDriver拥有Turn identity与terminal receipt；ReactAgent/其它Agent拥有是否生产trace及真实trace ID；RunStore拥有trace事实和correlation查询；EvalResult只投影已从RunStore验证存在的真实trace ID。

```text
EvalRunner
  |
  +-- eval correlation C
  +-- EventIdentity(run=C, turn=C, execution=C)
  +-- AgentInvocationContext.runtime(C) --> AgentTurnDriver --> ReactAgent
                                                        |
                                                        +-- Run.run_id = T
                                                        +-- parent/turn/execution = C
  |
  +-- TurnReceipt settled
  +-- RunStore.list_by_parent_run(C)
  +-- exact turn/execution match --> RunStore.load(T)
  +-- EvalResult.run_id = T --> criteria / constraints / metrics
```

## 核心结构与数据流

Eval已有每次执行唯一的`identity_value`。该值升级为正式correlation，不新增第二UUID。`EventIdentity::for_run`使transport identity显式包含run、turn和execution；`ExternalRunContext`使用同一值填入对应字段。它们是调用方分配的关联身份，不替代React trace producer分配的`run_<uuid>`。

Trace解析只在receipt存在时运行。RunStore先按`parent_run_id=C`返回summary，再要求`turn_id=C`与`execution_id=C`；只有一个候选才按summary中的真实trace ID加载Run。加载完成后，EvalResult和所有评分逻辑共用该Run对象，避免result ID与评分数据来自不同事实。

RunStore查询与唯一性错误必须返回结构化内部结果，由Eval投影为明确violation。零候选不是错误，因为Agent trait允许不支持trace；配置RunStore只提供读取能力，不强迫任意Agent生产记录。

## 异常和边界场景

- EventIdentity构造失败：沿现有setup failure路径close workspace，不查询trace。
- Agent不生产trace：correlation结果为空，`run_id=None`；OutputContains等无trace criteria仍按输出判断。
- ReactAgent生产一个trace：精确匹配并load真实ID，metrics与trace criteria使用该Run。
- 外部legacy product run已设置：value-scoped Eval correlation覆盖本invocation，getter值不参与trace解析，也不被修改。
- 同一parent下有其它turn/execution的child traces：精确tuple过滤，不误选。
- 两个summary具有相同parent/turn/execution：标记ambiguous并失败，不按时间或列表顺序选取。
- list/load I/O失败或summary指向不存在Run：记录trace lookup violation并失败；不伪造run_id。
- Turn Failed/Cancelled但已settled且有trace：仍可加载诊断Run，原Turn结果保持失败。
- Deadline后grace内settled：可解析terminal trace，Eval仍保持Timeout。
- Grace后未settled：RunStore调用数保持0，workspace保留。

## 关键取舍与业界依据

OpenTelemetry把Trace ID定义为Span Context中的独立不可变身份，并通过context propagation建立关联；它不等于上层业务请求或会话ID。OpenAI Agents SDK同样区分自动/显式`trace_id`与用于关联同一conversation多条trace的`group_id`，并通过上下文管理trace生命周期。OpenAI Codex在协议和rollout trace中持续使用分型的thread、turn、runtime operation和telemetry trace identity，而非一个裸run ID覆盖所有语义。

- OpenTelemetry Traces: <https://opentelemetry.io/docs/concepts/signals/traces/>
- OpenAI Agents SDK Tracing: <https://openai.github.io/openai-agents-python/tracing/>
- OpenAI Codex Turn protocol: <https://github.com/openai/codex/blob/main/codex-rs/app-server-protocol/src/protocol/v2/turn.rs>
- OpenAI Codex W3C trace context: <https://github.com/openai/codex/blob/main/codex-rs/otel/src/trace_context.rs>

扩展TurnReceipt被排除：trace是可选observation，不是每个Agent terminal都必须拥有的控制事实；新增字段还会无必要扩大公共SDK合同。公开ReactAgent getter被排除：共享mutable current值无法证明属于刚完成的invocation。扫描全部Run并按时间猜测被排除：顺序不是identity。现有`ExternalRunContext`与`RunStore::list_by_parent_run`已经提供调用方correlation和producer trace分离的完整机制，仓库测试`value_scoped_direct_answer_records_usage_in_child_trace`也证明该路径真实可达。

## 复用与实现约束

- 复用`EventIdentity::for_run`、`ExternalRunContext`、`AgentInvocationContext`、`TurnReceipt`和`RunStore::list_by_parent_run/load`。
- 复用`RunSummary.parent_run_id/turn_id/execution_id`，不新增索引、registry、store或公共trait方法。
- `EvalResult.run_id`只保存已成功load的真实`Run.run_id`；correlation不进入该字段。
- 精确匹配必须检查parent、turn和execution，不使用case ID、时间、agent-wide getter或列表首项作为权威。
- 未settled timeout不得执行list/load；所有查询只使用同一配置RunStore。
- 不新增依赖、public API、wire/schema、SDK identity或应用状态。
- 不使用unwrap、expect、panic、unreachable、todo、UTF-8字节切片或Worker术语。

## 验收标准

- 真实ReactAgent red同时设置legacy product run和独立RunStore trace；旧Eval把product ID当trace ID，无法返回真实Run或trace metrics。
- 修复后EvalResult.run_id等于RunStore中真实`run_<uuid>`，不等于legacy product ID；loaded Run的parent/turn/execution都等于本次Eval correlation。
- 带provider usage的真实ReactAgent证明tokens/metrics来自同一真实Run。
- 零候选保持可执行；多精确候选、list/load failure与dangling summary有明确失败投影；其它child trace不被误选。
- Unsettled timeout的RunStore list/load均为0；settled failed/cancelled/timeout仍可加载诊断trace且不改变原终态。
- 源码不再从`agent.current_run_id()`获取Eval trace；TurnReceipt与SDK artifacts保持零差异。
- Eval/React/TurnDriver测试、双语文档、eval feature Clippy/check、SDK零diff、semantic strict/change-evidence、Issue reconciliation和独立review全部通过。
