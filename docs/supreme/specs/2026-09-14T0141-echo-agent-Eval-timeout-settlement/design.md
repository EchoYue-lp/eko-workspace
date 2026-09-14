---
title: echo-agent Eval timeout settlement 设计
artifact: design
carrier: markdown
---

# echo-agent Eval timeout settlement 设计

## 问题与目标

`EvalRunner::run`当前用`tokio::time::timeout`包住raw Agent stream消费。Deadline到达后该future被drop，runner只调用CancellationToken并立即读取run ID/trace、形成评分、返回结果。ReactAgent的Managed stream会在Drop后启动bounded reaper，但Eval无法等待或观察该reaper；Tool、trace和workspace effect可能尚未结算。

Plan 14已将每次Eval隔离到唯一workspace generation：timeout暂时保留目录，阻止后续case污染。本设计进一步让Eval复用唯一`AgentTurnDriver`和`TurnReceipt`，在deadline后执行有界cancel-and-settle，再决定trace读取与generation cleanup，不新增Eval私有终态。

## 目标行为

- Eval执行由`AgentTurnDriver.drive`产生唯一`TurnReceipt`；Eval不再自己从raw事件流推断FinalAnswer/Error/Cancelled/EOF。
- Eval使用无持久状态的sink消费driver envelope；Agent自己的trace producer与RunStore仍是观察事实源。
- 主`timeout_secs`到达时，runner保留同一个pinned drive future，触发同一CancellationToken，再等待framework现有6秒cancellation settlement grace。
- Grace内取得receipt：Eval结果仍判Timeout失败，不接受deadline后的FinalAnswer为成功；记录实际settled status，可读取终态trace/metrics并安全close generation。
- Grace再次超时：明确记录unsettled violation，不读取或评分非终态trace，不运行success criteria，保留generation路径；drop drive后由现有Managed stream reaper继续其独立bounded cleanup。
- 非timeout路径完全由receipt映射：Completed必须有final answer才运行criteria；Cancelled与Failed均为失败；stream EOF由driver统一变为Failed。
- Eval的result score与success保持既有独立维度；timeout或settlement failure不伪造质量成功。

## 范围与非目标

范围包括EvalRunner执行入口、通用6秒settlement grace常量复用、raw helper退出、timeout/receipt/trace/workspace顺序、双语Eval文档、ADR、语义对象与Issue #48。

以下不在本设计内：

- 不改变AgentTurnDriver、TurnReceipt或EventEnvelope的public shape和终态规则。
- 不新增Eval Run registry、JoinHandle表、background reaper、第二CancellationToken或私有状态机。
- 不让Eval拥有ReactAgent内部task handle；grace失败后继续由Managed stream既有reaper负责。
- 不改变timeout_secs public字段、EvalResult schema、grader/criteria/constraint算法或run_all串行策略。
- 不保证任意第三方Agent响应cancel；unsettled路径必须有界返回并保留资源诊断。
- 不修改echo-agent-cli或echo-website；website文档投影继续按当前冻结策略延后。

## 系统边界

`AgentTurnDriver`继续唯一拥有有限Turn的事件归约与terminal receipt；Eval只拥有deadline policy、质量判定和workspace cleanup disposition。ReactAgent owns its producer task and reaper；RunStore owns trace facts；EvalWorkspaceGeneration owns filesystem isolation。

```text
EvalRunner
  |
  +-- create unique workspace generation
  +-- build TurnRequest(Execute, invocation cwd, cancel)
  +-- pin AgentTurnDriver.drive ------------------------------+
  |                                                           |
  +-- receipt before deadline --> map terminal --> trace --> close
  |
  +-- deadline --> cancel token --> wait same drive for 6s
                         |
                         +-- receipt --> Timeout result + terminal trace + close
                         |
                         +-- grace timeout --> no trace scoring + keep workspace
                                               drop drive -> existing stream reaper
```

## 核心结构与数据流

Root crate提供一个`pub(crate)`的`AGENT_CANCELLATION_SETTLE_PERIOD`，值沿用ReactAgent stream reaper现有6秒。Stream reaper与Eval timeout共同消费该常量，不形成两个数字策略源。

Eval构造唯一EventIdentity与`TurnRequest::mode(Execute)`，注入当前workspace invocation和CancellationToken。无状态`EvalEventSink`对每个envelope返回Continue。Driver future只创建一次并pin；两次timeout都借用该future，因此主deadline不会销毁underlying drive。

Result处理使用三个事实：是否越过主deadline、是否取得TurnReceipt、receipt outcome。只有取得receipt才允许加载终态trace；只有主deadline前Completed且有final answer才运行success criteria。Workspace close只依赖settled receipt，未settled则keep。

## 异常与边界场景

- Identity构造失败：Agent未启动，记录typed setup failure并显式close generation。
- Driver在deadline前Completed无final answer：失败并close。
- Driver在deadline前Cancelled/Failed/EOF：按receipt失败并close。
- Deadline与terminal同时竞争：`tokio::timeout`先poll drive；能完成则走receipt，不把已完成Turn误标为timeout。
- Deadline后Cancel，grace内Cancelled/Failed/Completed：统一保持Timeout失败，记录settled status；trace可读取，workspace可close。
- Grace超时：记录unsettled，skip trace/criteria，keep workspace并返回；不得等待无限期。
- Third-party Agent忽略cancel：落入grace超时，不阻塞后续Eval；其effect被唯一generation隔离。
- Caller直接drop run future：Plan 14 guard继续保留workspace，driver/stream Drop沿既有owner cleanup，不产生伪receipt。
- Sink意外失败：driver生成Failed receipt；当前无状态sink不引入I/O失败。

## 关键取舍与业界依据

Tokio timeout文档说明deadline错误会取消被包装future，而取消timeout本身不执行额外cleanup；要继续观察underlying future必须保留它。CancellationToken只是请求协作取消，任务要主动等待`cancelled()`并结算。OpenAI Codex的interrupt请求直到观察到`TurnAborted`才回复，而不是把提交interrupt当作终态。Inspect AI也把interrupted cleanup作为独立生命周期处理。

- Tokio timeout cancellation: <https://docs.rs/tokio/latest/tokio/time/fn.timeout.html#cancellation>
- Tokio CancellationToken: <https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html>
- OpenAI Codex interrupt waits for TurnAborted: <https://github.com/openai/codex/blob/a505c71490885a44979df056284badbfdd75b3fb/codex-rs/app-server/src/request_processors/turn_processor.rs>
- OpenAI Codex abort integration: <https://github.com/openai/codex/blob/a505c71490885a44979df056284badbfdd75b3fb/codex-rs/core/tests/suite/abort_tasks.rs>
- Inspect AI interrupted cleanup: <https://github.com/UKGovernmentBEIS/inspect_ai/blob/b6589d81f449112bb9942b0003b6fd9b54f1c48e/docs/extensions-sandboxes.qmd#L133-L149>

直接spawn Eval-owned driver task被排除，因为`run`只借用`&dyn Agent`且不应引入' static registry。只依赖ReactAgent reaper被排除，因为Eval支持任意Agent且无法观察reaper完成。无限等待被排除，因为第三方Agent可以忽略cancel。新增可配置grace字段被排除，当前先统一复用framework既有6秒策略，避免扩大public contract。

## 公共合同与兼容

`EvalRunner`、`EvalResult`、`Agent`、`AgentTurnDriver`和SDK identity形状不变。可观察行为变化是：timeout最长可多等待6秒以取得终态；settled timeout会清理generation，unsettled timeout保留；Eval不再从raw stream重复推断终态。

双语Eval文档必须说明deadline与settlement grace的区别。SDK artifacts应保持零差异。正式ADR记录driver复用、grace authority、trace/cleanup顺序与回滚。

## 复用与实现约束

- 复用`AgentTurnDriver`、`TurnRequest`、`TurnOutcome`、`TurnReceipt`、`EventIdentity`和Plan 14 generation guard。
- 复用ReactAgent现有6秒stream cancellation settlement值，提升为root crate内部共享常量，不复制magic number。
- Driver future必须只创建一次；主timeout和grace timeout都借用同一Pin，不能在deadline后重新执行Agent。
- Grace失败时不读取RunStore、不运行criteria、不close workspace。
- 不新增依赖、public字段、wire/schema、SDK identity或应用状态。
- 不使用unwrap、expect、panic、unreachable、todo、UTF-8字节切片或Worker术语。

## 验收标准

- 旧实现red证明cancel-responsive Agent在deadline后完成副作用并发出Cancelled，但Eval已返回或保留workspace，未观察TurnReceipt。
- 新实现证明deadline后runner等待同一drive：cancel-responsive Agent结算后才返回，结果仍为Timeout，记录cancelled status，workspace已删除。
- Ignore-cancel Agent在6秒grace后有界返回，skip trace评分并保留可诊断workspace；测试自行删除。
- 正常Completed、Completed无answer、typed Failed、Cancelled与EOF均由TurnReceipt映射，现有criteria/trace结果不回归。
- Source中Eval raw `execute_for_final_answer`终态归约退出；stream reaper与Eval共用唯一6秒常量。
- Eval/Improve/TurnDriver测试、双语文档、eval feature Clippy/check、SDK零diff、semantic strict/change-evidence、Issue reconciliation和独立review全部通过。
