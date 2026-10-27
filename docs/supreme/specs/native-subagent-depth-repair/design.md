---
title: 原生 Subagent 深度的运行隔离与只读委派修复
artifact: design
carrier: markdown
---

# 原生 Subagent 深度的运行隔离与只读委派修复

## 问题与目标

本设计细化[GUI 产品收敛设计的 Subagent 政策](../eko-gui-product-convergence/design.md#唯一-goal-与当前有效要求)，修复两处已经由原生运行暴露的实现接缝。产品关系继续是 `TaskRun → PlanTask → SubagentRun`：主智能体为第 0 层，EKO 默认最大总层数为 1，GUI 可选 1、2、3；新配置只影响新运行，框架通用复用方保留自己的上限。

同一 conversation 的两个独立运行 `taskrun:9c54a00f-d788-4e05-b059-0a73e5f09b11` 与 `taskrun:63da81cc-6563-4512-8b00-aaaa21b47418` 已分别持久绑定上限 3，使用同一个 `native-depth-probe`、revision 1、attempt 1。前一运行的真实只读 reviewer 启动后搜索不到 `agent_tool`，没有到第二层；后一运行在派发前收到 `attempt 1 already started`，实际 child 未启动。两个 Run 的 claim/execution 均不同，因此不能以同一 attempt 重放解释第二次拒绝，也不能把问题归因于 GUI 未保存。

现有上限 1 运行 `taskrun:64916d5c-6662-489e-b779-afe1b245c502` 保留其原绑定。其成功与 HELLO/HITL 通过均不证明上限 3 或本设计已经完成。原始脱敏运行回读为 `echo-agent-cli/.echo-semantic/local/gui-native-epoch16-preparation-20261027/native-depth3-failure-journal-readback.json`，SHA-256 `444cfc180878e626e70d8dc09aedb0905ed4ed622ddb26aaabe5ba0ce247e973`；根因及 API 调查原件位于主根 `.echo-semantic/local/native-depth3-attempt-collision-20261027/`。设计不改写这些历史证据。

目标是让同一主 Agent/registry 下不同 Run 的 attempt 与下一次尝试指导互不干扰，并让声明可委派的真实只读 reviewer 在其既有工具上界内继续受限委派。运行身份、深度、只读与工具集合是同时成立的独立约束，任何一个允许都不抵消其它约束。

## 目标行为与范围

- 不同 Run 可以复用同一逻辑 task、revision 和 attempt；同一 Run 的同一逻辑 attempt 仍只准入一次，完成后的去重记录继续保留。
- 预先排队的 guidance 只能由它所属的 scope、Run、task、attempt 消费一次；应用原 revision、command 和 durable journal 校验继续生效。活消息与 interrupt 继续绑定真实 execution/attempt。
- 默认 1 的首层不能再派发；绑定 2／3 的运行可从真实内置 reviewer 工厂分别到达第二／第三层，下一层在模型、factory 或副作用开始前拒绝。每次真实 spawn 沿原 policy 增加一次深度。
- 只读父调用只向只读目标委派，所有后代继续承接只读上界与父工具上界。工具发现、模型 schema 和实际强制调用采用同一有效范围；写角色、Team 成员、后台、Fresh、同名替换与排队后注册均不能绕过。
- 泛用框架的未受限调用、旧 unscoped guidance API、普通 writer 派发与合法通用深度保留；本地交互终端与用户直接操作不新增权限门控。

范围仅含现有 control/dispatch、调用上下文、snapshot/pipeline、EKO control 适配与真实工厂消费者。不得新增 registry、mailbox、store、executor、任务 CRUD、终态或 GUI 配置；不改变调度、取消、等待、栈、超时、Schema budget 或角色名单。不得通过随机 task ID、清理 settled 账、全局将 `agent_tool` 声明只读、按工具名豁免或移除 reviewer 的只读开关制造通过。

zstd 依赖链修复、品牌源、SDK pin/接受集、A2A/LSP 外部校验工具、有效要求解释方案、发布与部署均为独立工作。本修复不以 zstd 提交或其工具合同阻塞为功能依赖，也不消费其 owner 或修改其源码。真正的依赖是本修复 Framework 机制冻结后，EKO 使用同一份机制完成消费者资格。

## 系统边界与复用依据

| 层次 | 本次归属 | 复用的当前权威 |
|---|---|---|
| Framework 通用机制 | scoped guidance、调用级只读上界、受限派发与实际执行检查 | 原 control registry、`ScopedTaskAttemptKey`、`ToolAccess`、原 registration、invocation、snapshot/pipeline |
| EKO 产品策略 | GUI 1..3 与默认 1、不可变 Run profile、内置角色/readonly 工厂、产品 journal | 原 TaskRuntime、原 control service、原 AgentPool、原工厂和 RunCreated |
| 薄适配边界 | 同一 durable identity 的 attempt 投影、guidance 转交、调用结构完整传递 | `SubagentCommandIdentity.attempt_identity()`、既有 `allowed_tools`、原四模式/后台 dispatcher |

现有机制已覆盖大部分需求：

1. EKO 的 `SubagentControlIdentity` 已直接重导出 Framework `SubagentCommandIdentity`；其 `.attempt_identity()` 已通过 `for_run` 保留 Run 与 control scope。当前 EKO helper 错用了无 Run 的 `SubagentAttemptIdentity::new`，不需要新身份类型。
2. Framework control 的实际键已是 `(control_scope_id, run_id, task_id, attempt)`；原 active、settled 与 queued guidance 共用同一 mutex 和表。原公开 queue 仅能生成 None/None 键，缺少有 scope 的入口，而不是缺一套队列。
3. Framework 已有 `ToolAccess::{ReadOnly, Mutating}`、目标 `access_mode`、不可变 resolved registration、positive `allowed_tools` 和执行锁后的工具 snapshot。当前只有工具名字上界能跨层传递，缺少调用级只读承接。
4. 内置 reviewer 源为 `readonly: true`、`can_delegate: true`，真实工厂同时开启 `.readonly_tools()` 与原 dispatch 注册。当前普通 `AgentDispatchTool` 默认 Mutating，被原 readonly snapshot/search/pipeline 正确过滤；已有自造 child 的回归没有启用真实 readonly 工厂，覆盖不足。
5. 默认 `Agent` invocation 方法保留 metadata 后调用旧接口，不实际落实只读约束；它不能被当作已支持新合同的实现。

源锚点：Framework `src/agent/subagent/control.rs:148`、`:176`、`:243`、`:992`；`executor.rs:1353`；`context.rs:115`、`:207`；`registry.rs:503`；`src/tools/builtin/agent_dispatch.rs:299`、`:350`；`src/agent/react/mod.rs:690`；`snapshot.rs:601`、`:670`；`react/run/pipeline.rs:855`、`:1747`；`echo-core/src/agent/mod.rs:418`、`:1067`、`:1177`、`:1219`；EKO `subagent_control.rs:835`、`:1933`、`infra/factory.rs:1984`、`:2013` 与 `executor/tests.rs:3984`。14 个原根因锚点按文件哈希复核后用于本次设计；后继编译必须重新冻结实际输入，行号和本设计不能替代新快照。

标准库的值传递、`Option` 与现有 mutex 即可表达缺失接缝；平台原生能力和已安装依赖不提供另一个更合适的调度/权限权威。最小自定义部分只是在原队列补 scoped 入口，在原三个 carrier 中补可选 access 上界，以及原派发实例/准入对该上界的落实；不增加依赖。

## 运行身份与指导数据流

EKO 由真实 TaskRun/TaskClaim 构造 durable control identity，再从同一个值投影 Framework attempt identity。投影必须保留 Run；不要求为尚未发出的命令虚构 command ID，不重造 execution 或 task ID。readonly 与 writer 的有／无附件分支使用相同 helper。

原 control queue 增加有 scope 的调用方式，与旧入口共用实际 enqueue。新入口明确接收 scope、Run、task 和下一次 attempt，验证各身份非空、attempt 非零、instruction 非空；错误沿原 control error 返回。旧 unscoped 入口保持原签名、None/None 键与合理外部用途。

公开追加接口为 `SubagentExecutor::queue_guidance_scoped(&self, control_scope_id: &str, run_id: &str, task_id: &str, expected_next_attempt: u32, instruction: impl Into<String>) -> Result<SubagentGuidanceQueueReceipt, SubagentControlError>`。它只选择原 scoped key，原 `queue_guidance` 与它共用一份私有 enqueue；不接收未来尚不存在的 execution，也不增加 command ledger。

EKO 原 journal 筛选完成后，从 target 的同一 attempt 投影取得 scope/Run 转交新入口；scope 缺失明确失败，不能回落无 scope。guidance 的队列键仍是未来逻辑 attempt，execution/command/revision 由应用原 durable 事实校验；未来 claim 尚不存在时不生成占位 execution。原 mailbox 接受回执、一次认领、重复命令与错误 revision 的处理保留。

真实 admission、物理 owner settle 与有界 settled 保留规则不改变。只读或取消返回 `ExecutionNotSettled` 时仍保留原 owner 和真实身份，不能当作完成释放。修复不修改 exact execution 的活消息/interrupt 路由或缩短等待。

## 只读委派与工具数据流

### 同一调用上界

复用 `ToolAccess`，在已有 `AgentInvocationContext`、`SubagentContext`、`ToolContext` 增加 `tool_access: Option<ToolAccess>`。这是调用值而非持久化 schema、产品配置、模型参数或新的许可状态。`None` 维持没有额外限制；`Some(ReadOnly)` 只能收窄；`Some(Mutating)` 不能覆盖任何已存在的只读配置或祖先只读上界。

合并后只要任一真实上游约束为 ReadOnly，后代有效值就是 ReadOnly。Fresh、空历史、无 goal、Team 只有 scope 的上下文也不能丢失它，原 `has_content()` 必须保留 access-only context。Debug 仅打印枚举，不增用户正文、密码或密钥日志。`ExternalRunContext`、Run profile、`NestedDelegationPolicy` 和事件 wire 不增加重复字段。

追加公开 Rust 字段会影响使用完整 struct literal 的复用方；仓内直接调用方必须适配，使用 Default 的既有代码不机械改写。此项不宣称完全源码兼容，也不意味着独立 SDK 已采用新字段或应自动升级 pin/accepted。

### 受限实例与同一 registration 准入

原 `AgentDispatchTool` 提供受限的只读委派实例。只有该实例的 risk/capabilities 一致表达只读；普通 `new()` 仍是原 Mutating 能力。Framework 装配时，根据既有 `readonly_tools` 与 dispatch 注册组合选择受限实例，EKO 的真实 readonly 工厂无需新增产品开关，writer 工厂保持原选择。

受限实例只展示 ReadOnly 目标，实际调用仍使用原 resolved registration。准入在同一不可变 record 上检查目标 access mode，随后沿原 dispatch/后台入口执行；不得先检查旧标签又按名称解析新 writer。Team 每个 manager/member 都按自己的实际 record 走原准入，不能只检查外层 Team 标签。

受限实例必须有原 `ToolContext` 与有效 visibility，从其不可变 available 集取得父工具上界；没有可信上下文时拒绝。此防护针对本地框架调用丢失约束，不要求 full-auto。原 `inherit_tool_scope` 求交且拒绝空交集，不把空结果转换为旧的“无限制”表示。

### 原 snapshot 与 pipeline 落实

父 pipeline 从真实有效 snapshot 将 access 与 visibility 放入原 ToolContext；dispatch 传给原 SubagentContext，再由原 Sync/Fork/Teammate 及 Team member 调用适配写入原 invocation。约束不以 task-local 或仅 prompt 传递。

子 Agent 取得原执行锁后，原 snapshot 的 `ToolRuntime::from_agent_with_tool_scope` 增加一个可选调用 access 输入，计算 `effective_readonly = config.readonly_tools || invocation_access == Some(ReadOnly)`，存入已有 `ToolRuntime.readonly_tools`；公开 `ToolRuntime::from_agent` 继续原签名并传入 None。原 snapshot 是有效值的唯一计算者，ToolContext 只承接 `snapshot.is_plan_mode() || snapshot.readonly_tools` 的结果，不读取 GUI、角色标签或模型参数重新决定。原字段继续过滤模型工具、search eligible 与实际执行。既有 plan mode 的更严格约束仍成立；向后代传递的是实际只读结果。仅有调用级只读、没有显式工具上界时，也建立原 qualified visibility，避免工具搜索回落到共享全目录；原 deferred/NoDeferred 曝光策略和预算不改变。

每层实际可用集是：当前锁后 catalog ∩ 父可用上界 W ∩ 有效只读 capabilities，并继续接受原 disabled、model、skill、permission 与深度约束。readonly 标签的目标即使错误装入写工具，实际 schema/search 也排除，强制调用仍由原 pipeline 拒绝；同名工具更换成 Mutating 实现也不能获得权限。普通不限范围的 Tool/dispatch 入口不因此全局提升为只读。

默认 `Agent` 的三个 invocation 回退不能假装落实只读：新 ReadOnly 输入在旧 execute/chat 方法开始前明确返回不支持，`None` 旧路径保持。ReactAgent 使用其真实 snapshot override；Arc 与应用 adapter 整体转交原 invocation。第三方主动 override 必须履行该公开合同，框架不能声称能阻止任意自定义 Agent 绕开自身实现的副作用。

## 官方参考与关键取舍

参考当前 [Claude Code 官方 Subagents 文档](https://code.claude.com/docs/en/sub-agents)：子角色沿主会话工具池收窄，目标限制与工具限制分别表达，达到深度边界限制曝光并拒绝越界派发。完整官方 Markdown 于 2026-10-27 保存，SHA-256 `d66f847bed2cd73de0a45f29bf9c8dff7782dc7b66dc63d8285176ae6a176cdb`，原文与 HTTP200 回执在根因主题 `references/`。本次再次调用 web reader 返回服务端 `/v1/alpha/search` 404，因此引用已保存的同日官方原文，不声称本次 reader 成功。另一次 SDK 页面跳转不是 Subagents 机制依据，不计作第二套实现。

EKO 保持自己已经批准的默认 1、GUI 1..3 与 immutable Run，不照搬外部默认深度、环境变量或终端生命周期。当前只读 reviewer 已明确可委派，因此选择让后代继承只读能力；仅隐藏工具会继续让合法深度 3 失效，仅信任角色标签会让实际工具越权。沿现有 registration 与 snapshot 落实两者，避免另一套权限服务或调度循环。

## 异常、并发与恢复

| 场景 | 必须保持的行为 |
|---|---|
| 同主 Agent 下两个 Run 复用 task/attempt，前者 active 或 settled | 按 scope/Run 区分；不清账，两次真正准入可独立结算 |
| 同 Run 同逻辑 attempt 重复或错误 execution | 原去重／exact identity 拒绝继续成立 |
| 两个 Run 的 next-attempt guidance | 各自一次消费，错 scope/revision/重复 command 不串流 |
| 强制 writer、后台或 Team 混入写成员 | 对对应不可变 record 在 factory/model/副作用前拒绝；不承诺此前合法只读成员从未执行 |
| readonly 目标实际装写工具或排队后注册同名写实现 | 锁后原 snapshot 再收窄，强制执行仍无写副作用 |
| 受限派发丢 ToolContext、有效 scope 或硬交集为空 | 明确错误，不按无上界继续 |
| 子调用请求 Mutating 或选择其它 mode | 祖先 ReadOnly 保持，策略不能提升 access/W |
| 自定义 Agent 只实现默认 invocation 回退 | 新受限调用明确不支持，旧入口计数为零；None 正控制保留 |
| GUI 改层数、进程恢复或旧运行返回 | 仍用本 Run 原 profile 与真实 owner；不读取新默认伪造许可或终态 |

当前原生 PID 56017、fixture PID 55840 与其模型脚本不在本设计执行权内。完成源码资格后由协调者准备新制品/输入快照和独立原生维护窗口；不让用户继续在旧 binary 重试，不更改固定 `native-depth-probe` 来绕开碰撞。

## 验收标准

1. 真实共享 primary/registry 的两个 Run，同 task/revision/attempt、不同 claim/execution，先后与同时 active 均可独立开始/结算；同 Run 重复 attempt 仍失败。两份原 next-attempt journal 经真实 transfer/queue/admission 各消费一次，错误身份和重复回执不串流。
2. 原内置 reviewer 经真实 discover/readonly factory、TaskRuntime 与原 Fork 路径，在运行绑定 1／2／3 下分别恰达上限，下一层 factory/model/工具计数为零；实际模型请求、ToolContext、parent/child lineage 与执行结果相互印证。旧 Run 在修改默认后保持原值。
3. Sync、Fork、Teammate、Team 与 background 的合法只读正控制成功；writer、错误标签、同名注册替换、Fresh、access-only context、空交集、排队后新写工具、强制调用与默认 Agent 回退负控制均不能获得写副作用。普通 writer、None/unscoped 与 Framework 合法 max4 对照保留。
4. 回归红必须来自旧生产行为上的断言失败，不能拿新字段缺失的编译错误或缺 API 当行为红。修复通过后，原始红/绿日志、自然退出码、选中/过滤/忽略数量、输入哈希和必要制品完整保留，测试失败与编译警告在当前单元修复。
5. Framework 与 EKO 的架构说明/ADR 按所属仓维护；示例适用性有明确记录，原示例纳入适用编译链；官网待对应源稳定后通过原镜像脚本同步。SDK pin 与接受集不自动改变，公开 inventory 仅按其独立规则核验。
6. focused、正式全量门禁、GUI build、新原生、main/CI 各自报告资格，不互相替代。新原生用同一固定 task ID 验证同 conversation 的两个 Run 及层数 1／3；尚未构建/安装时明确未验收。

## 现有设计与交付边界

本设计与总设计、EKO ADR0052 的运行级层数约定一致，是实现缺口修复；不接受或扩张仍 Proposed 的统一任务重构。Framework ADR0094 的名字上界继续有效，但需补充调用级只读继承与其默认回退限制。以往自造 child 的 focused 结果只保留原覆盖，不再据此宣称真实 readonly 工厂已通过。历史调查中“等 zstd checkpoint 后才能开始”的并发建议不属于产品依赖；协调者现在允许分离 task scope 保护现有改动，本修复据此建立独立文档和源码 epoch。

此文件为跨仓阶段设计入口，不成为长期产品/API 唯一说明。生产修复、测试与正式子仓 ADR 由对应实施计划和协调者分配的下一源码 epoch 承接；当前文档状态不表示代码、native 或整体 Goal 已完成。
