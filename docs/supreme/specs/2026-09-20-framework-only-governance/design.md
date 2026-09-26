---
title: Echo Agent Framework-Only 语义治理总设计
artifact: design
carrier: markdown
---

# Echo Agent Framework-Only 语义治理总设计

## 问题与目标

`echo-agent` 已经是独立发布、独立验证的通用 Rust Agent framework，但部分历史
Finding 仍把 SDK Host、EKO、三语言 SDK 或 website 的采用结果当作关闭条件。结果是
framework 生产路径、测试、文档和语义证据已经闭合，Issue 却继续保持 open；同时真正属于
外部 adapter 的缺口会反向污染 framework 的优先级和状态判断。

本设计把完成权威收回 `echo-agent` 仓库。每个语义事实只能有一个 owner；framework
Finding 只由 framework 的生产实现、测试、语义证据、门禁、文档和示例决定。外部仓库可以
依赖 framework API，也可以拥有自己的采用 Finding，但不能成为 framework Finding 的终态
条件。

目标不是建立一个覆盖所有能力的总状态机，而是让每个边界都有唯一、可核验的权威：

- 运行状态与用户投影分离，投影不反写权威；
- 生命周期从停止接纳到资源释放完整闭合；
- 持久副作用在重启后可判定为已完成、待重试、冲突或结果未知；
- 每个外部副作用由创建它的 owner 结算；
- Agent 自动执行路径的权限下限无法被 Hook、custom tool 或 fallback 放宽；
- 公共 API、双语文档、可执行示例和测试描述同一合同。

## 目标行为

### 按事实划分唯一权威

“唯一状态权威”表示同一种事实只有一个可提交 owner，不表示所有能力共享一个 store。

| 事实 | 权威 | 只读或派生投影 |
| --- | --- | --- |
| Agent Turn 终态 | `AgentTurnDriver` / canonical ReAct terminal | stream、Trace、Audit、channel event |
| TaskRun graph、claim、retry、settlement | revisioned Task store 与 `RuntimeTaskService` | Todo、Team live control、事件 |
| Subagent physical attempt | `TaskClaim` 派生 identity + canonical Subagent execution | process-local reserved/active/settled registry |
| ReAct 恢复状态 | `RuntimeStateStore` checkpoint/journal | in-memory snapshot、transcript cursor |
| 用户 transcript | `ConversationStore` committed projection；pending intent 由 runtime checkpoint 持有 | UI/history reader |
| Scheduler definition | `CronTaskStore` | runner cache、`last_run` projection |
| Scheduler occurrence delivery | framework `DeliveryLedger` | callback receipt、task definition projection |
| Tool 自动执行许可 | canonical permission pipeline 的 effective decision | prompt、Hook input、UI prompt |
| Provider request facts | provider-neutral canonical request + model capability resolver | provider wire payload、token estimate |
| Trace/Audit 持久事实 | 各自 `RunStore` / `AuditLogger` | diagnostic delivery observation |

任何 adapter 只能无损转换、注入 metadata 或施加更严格的产品策略。adapter 不得重建
claim、terminal、retry、ready frontier、permission 或 provider capability 权威。

### 生命周期与终态

所有拥有异步资源的 framework owner 遵守同一个顺序：

```text
停止接纳 -> 请求取消 -> 等待终态 -> 持久结算 -> 释放资源
```

`Drop` 只能做无阻塞的最后防线或发出未结算告警。fire-and-forget task、EOF、日志、
transport stop 和取消请求都不能证明资源已关闭。成功 close 必须等待 owner 持有的 task、child、
pending request 和 delivery lease 到达安全点；失败返回 typed debt，并由原 owner 保留重试能力。

同一执行只发布一个业务终态。资源 close、projection settlement、diagnostic delivery 和
cleanup receipt 是独立事实，不能伪装成第二个 Completed/Failed/Cancelled。

### 可恢复持久化

需要跨重启成立的副作用统一使用可识别的 operation identity 和以下边界：

```text
持久准备 -> 取得 owner/claim -> 标记 effect-started -> 执行副作用
         -> 持久 terminal/receipt -> 投影 -> 对账与重放
```

重启对账必须区分：

- 已有 terminal receipt：幂等返回，不重复副作用；
- prepare 已持久但 effect 未开始：可安全重试；
- effect 已开始但 terminal 丢失：记录 `OutcomeUnknown` 或领域等价状态，再按稳定 identity 重放；
- revision、generation、claim 或 payload digest 已 stale：fail closed；
- best-effort telemetry 丢失：只影响可观察性，不改写业务终态。

业务事实、可恢复 delivery debt 和 best-effort telemetry 必须使用不同类型，禁止再共用一个
“记录成功”布尔值。

### 副作用 owner

创建资源或副作用的组件拥有 admission fence、取消 token、任务句柄、持久 receipt 和 close
结果。manager 可以聚合多个 owner 的 close，但不得在第一个失败后跳过其余 owner，也不得在
close 失败时从 active map 中删除原对象。replacement 只有在旧 generation 结算后才能发布。

Task、Subagent、Workflow、Scheduler、BackgroundTask、CommandCell、MCP、LSP、Plugin、Skill、
Trace/Audit 和 provider stream 保持各自领域边界；共享的是通用 journal、delivery、cancellation、
deadline 和 typed receipt 原语，不是一个跨领域 mega-executor。

### 权限下限

Agent 自动工具调用的 effective permission 取所有适用约束的交集。protected path、readonly、
plan mode、tool allowlist、approval receipt 和 sandbox minimum isolation 都只能保持或收紧，不能由
Hook rewrite、custom tool registration、重试、provider fallback 或 sandbox fallback 放宽。

策略顺序必须在 effect 前完成，且 effect 后的 Guard 只能审查或投影结果，不能补做授权。
权限拒绝、缺少 approval、无法建立最低 sandbox 和无法分类 mutation 都是 typed failure。

本规则只覆盖 Agent 自动决策路径。用户直接操作的 terminal、file picker、MCP 配置等交互式产品
能力属于 embedding application，不由 Agent 的 `full-auto/default` 权限模式阻断。

### 公共合同一致性

公共 Rust API 是 framework 的正式合同。新增或修改 public 类型、trait、feature 或 lifecycle
语义时，同一 Outcome 必须更新：

- `docs/en` 与 `docs/zh` 的概念、生命周期和配置说明；
- `echo-agent-learning/examples` 中受影响的可执行示例；
- public facade 与 feature 编译覆盖；
- 正常、失败、取消以及适用的重启/stale 回归；
- ADR、Capability Map、Finding、repair、verification 和 rereview 引用。

没有外部消费者不是拒绝通用 framework 能力的理由；外部消费者尚未采用也不是保持 framework
Finding open 的理由。

## 范围与非目标

范围包括 `echo-agent` workspace 中的核心 Agent、Task/Subagent、memory/persistence、tool permission
与 sandbox、extension lifecycle、observation、provider request 和公共 facade。治理材料、ADR、双语
文档与 learning examples 同属 framework 交付。

以下内容不属于 framework Finding 的完成条件：

- 独立 `echo-agent-sdk` 的 Host adapter、protocol、inventory 与三语言 bindings；
- EKO CLI/GUI/TUI 的产品策略、数据根、UI 投影和安装验证；
- website 展示内容；
- 任何第二消费者的 pin 或发布节奏。

这些仓库发现的 framework 缺陷仍可进入 framework Issue；但采用、映射或产品策略缺口必须在其
owner 仓库单独追踪。SDK Host 问题 `#120` 不再阻塞 framework Finding。

A2A 保持排除。本阶段不审计、不删除 A2A，也不借治理之名移除其公共 API。

## 系统边界

### 通用机制

framework 拥有跨产品成立的 cancellation、deadline、journal、delivery ledger、revision/claim、
typed terminal、effect classification、permission minimum、sandbox contract、provider-neutral request
和 public lifecycle API。这些机制必须独立编译、测试和文档化。

### 产品策略

重试次数、交互式 approval UI、reviewer 策略、worktree 规则、文件权威、具体 data root、资源配额、
GUI/TUI/CLI 投影和外部 protocol 映射由 embedding application 或 SDK Host 决定。framework 只暴露
表达这些策略所需的通用接口与 typed receipt。

### 适配边界

adapter 只能：

1. 在 framework 类型与外部 wire 类型之间做可验证的无损转换；
2. 注入 framework 已定义的 metadata、deadline 和 policy；
3. 把 framework typed result 投影给消费者。

adapter 不拥有第二套执行循环、状态迁移、持久 command authority 或恢复算法。需要跨进程保证的
Host command journal 属于 Host；需要当前进程 exact attempt control 的 primitive 属于 framework。

## 核心数据流

```text
调用入口
  -> 读取 canonical durable state / revision
  -> 计算 effective policy 与 capability
  -> 持久化 intent / claim / operation identity
  -> admission fence
  -> 执行一次 owned effect
  -> 产生唯一业务 terminal
  -> 持久化 receipt / delivery debt
  -> 更新派生投影
  -> awaited close / reconcile
```

所有恢复入口重新读取 durable authority，不从 cache、UI projection、日志或遗留 checkpoint 字段推断
当前事实。stale caller 携带的 revision、claim、generation 和 receipt identity 必须在副作用前核验。

## 异常和边界场景

- caller 取消 close future：cleanup owner 继续可寻址，后续 close 可以等待同一 single-flight 结果；
- 副作用成功但 receipt 写入失败：标记结果未知，不能宣称失败后盲目重做，也不能静默成功；
- terminal 已提交但 projection 失败：保留业务 terminal，并记录可恢复 projection debt；
- projection 冲突会影响业务可见正确性：在原 terminal 发布前 fail closed；
- Hook/custom tool 请求越权路径：保留原权限下限并返回拒绝；
- sandbox backend 不可用：若无法满足 minimum isolation，返回 typed failure，不降级裸执行；
- late response、late terminal 或 stale control：只匹配 exact identity，不能作用于新 attempt/generation；
- cache 与 store 不一致：以 store 为准刷新或拒绝，cache 不得覆盖 durable state；
- observer、telemetry 或日志失败：通过独立 failure fact/counter 可见，不改写 producer terminal；
- public API 有可选实现：不能因 EKO 未使用而删除；删除必须证明 framework 与合理复用方均不需要；
- A2A 路径被相邻改动触达：只保持现有编译和行为，不扩大本阶段范围。

## 关键取舍与业界依据

1. Claude Code 把 permission 作为 host enforcement：`deny -> ask -> allow`，prompt 不能改变
   host 允许的动作；subagent 有独立 context 与工具集，但仍受 session permission/sandbox 约束。
   本 framework 因而把 Hook、custom tool 和 Subagent 视为受同一下限约束的执行入口。
2. OpenAI Codex 把 session、active turn、task 与 cancellation token 分开，并通过 typed item/turn
   events表达运行结果。这里保持 Task、Turn、Subagent 与资源 close 的终态分离，不构造包含 plan
   approval 的总状态机。
3. Cursor 把 Plan 作为可审阅 artifact，并把执行角色与 live task 分开。`TaskPlan` 因此只能是
   revisioned TaskRun graph 的 artifact，不成为第二 store 或状态机。
4. MCP lifecycle 把 shutdown 定义为正式阶段；stdio 需要关闭输入、等待、再有界升级终止，request
   timeout 后发送 cancellation 并停止等待。MCP transport 必须由 transport owner awaited close。
5. Kubernetes CronJob 与 Temporal Activity 都承认 effect/receipt crash window，并要求 stable
   identity 与幂等处理。Scheduler 与其它 durable effect 采用 at-least-once delivery 和显式
   `OutcomeUnknown`，不虚构 exactly-once。
6. OpenTelemetry 把 telemetry export failure 与业务结果分离，并要求 failure 可观察。Trace/Audit
   直接 store API 保持 fallible；Agent 集成的 diagnostic delivery 另行报告失败。

参考：

- <https://code.claude.com/docs/en/permissions>
- <https://code.claude.com/docs/en/sub-agents>
- <https://github.com/openai/codex/tree/main/codex-rs/core/src/state>
- <https://github.com/openai/codex/tree/main/codex-rs/core/src/tasks>
- <https://cursor.com/docs/agent/plan-mode>
- <https://modelcontextprotocol.io/specification/2025-06-18/basic/lifecycle>
- <https://modelcontextprotocol.io/specification/2025-06-18/basic/utilities/cancellation>
- <https://kubernetes.io/docs/concepts/workloads/controllers/cron-jobs/#job-creation>
- <https://docs.temporal.io/activity-definition#idempotency>
- <https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/error-handling.md>

## 复用与实现约束

- 复用现有 Task graph、`RuntimeTaskService`、`AgentTurnDriver`、`RuntimeStateStore`、
  `ConversationStore`、`DeliveryLedger`、Journal、permission pipeline、sandbox policy、Trace/Audit
  store 和 provider-neutral request；禁止平行实现。
- 标准库与 Tokio primitives 只解决 mutex、channel、task join 和 cancellation plumbing；业务
  identity、durability 和 terminal 继续由现有领域类型拥有。
- 复用已安装的 `tokio-util::CancellationToken`、现有 atomic file/journal backend 和 feature
  topology，不因治理引入新的数据库、scheduler 或 protocol。
- 迁移可以短暂保留薄 adapter，但每个 Outcome 必须切换真实生产入口并删除已被替代的权威逻辑。
- 触及遗留执行角色命名时统一使用 `subagent`；第三方固定 wire name 只在最小边界保留。
- 所有文本截断使用 UTF-8 安全字符迭代；生产路径不新增 panic API。

## Finding 生命周期与验收

一个 framework Finding 只有在以下条件同时满足时才进入 `resolved` 并关闭对应 Issue：

1. 修复已 squash merge 到远端 framework `main`，并核对实际 main SHA；
2. focused tests 覆盖正常、失败、取消以及该能力适用的重启/stale 场景；
3. repair、verification、independent rereview 三类语义证据绑定最终源码快照；
4. `./scripts/verify.sh` 与本次改动适用的独立 feature matrix 全部通过；
5. framework 双语文档、注释、public facade 和仓库内 executable examples 已同步；
6. Finding 状态、Capability Map、Evidence/Audit 引用与 GitHub Issue 关闭说明一致。

外部 adapter 未采用、SDK inventory 未刷新、CLI 未 pin、website 未同步，都不能否定以上六项。
相反，framework 只有接口而生产路径、失败语义或验证尚未闭合时，也不能用外部消费者的绿色测试
代替 framework 完成。

Phase 0 的五个历史 Issue 必须按这套口径重新审查：`#55/#106/#46/#84/#99`。满足六项的立即
补齐当前快照证据并关闭；未满足的只保留真实 framework 缺口。`#120` 只属于 SDK Host。

## 总体验收标准

- Capability Map 中每个高风险场景都能指向唯一 owner、状态 authority、失败语义和验证；
- 生命周期路径没有用 `Drop`、detached task、日志或 cache 代替 awaited settlement；
- restart/replay 测试能证明稳定 identity、stale fence 和 terminal 收敛；
- Hook、custom tool、plan/readonly 和 sandbox fallback 都不能放宽权限下限；
- provider-specific adapter 只翻译 wire，不拥有平行 model policy；
- framework public API、双语文档、examples、tests 和语义材料在同一 main snapshot 上一致；
- A2A 保持现状且不进入本治理完成口径；
- framework Issue 的 open/closed 状态只反映 framework 事实。
