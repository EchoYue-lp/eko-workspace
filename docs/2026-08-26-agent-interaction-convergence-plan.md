# EKO Agent 交互与任务模型收敛迭代计划

> 状态：Approved for F0
>
> 日期：2026-08-26，执行基线更新于 2026-08-27
>
> 类型：跨仓库阶段性工程计划，不是框架公共 API 或 EKO 产品行为的长期事实源
>
> 范围：`echo-agent` 通用 turn/Subagent/task 原语与 `echo-agent-cli` 的 Conversation、
> TaskRuntime、Agent Router、surface 和 mode 删除

## 0. 2026-08-27 冻结基线

发布修复已经 child-first 完成，F0 以以下精确提交为唯一输入：

- `echo-agent@9f8d723ecb8e27a67754afe86fd7d285307866d7`；
- `echo-agent-cli@e7d9e9082df09058b7542eeb69e4cf1d5d2df387`；
- `echo-website@c25c86d96d2d4958e0c4d9424b61b3c0a8271131`；
- superproject gitlink release `b17578d`；
- 顶层文档归属规则 `dd95a9a`。

三个 child 的适用本地门禁已经完成，远端 GitHub workflow-level 调度失败按当前研发节奏延后到
F7 Final Integration Gate 统一修复和验证。原计划中的 `c6379aa`、`04092ec`、`5b29431` 和
`df049e2` 只保留为历史分析快照，不再作为实施基线。

长时和大规模门禁统一后移：10k/100k artifact/review history 全规模测试、10 分钟与 1 小时
协作 soak 均在 F7 执行；若最终仍保留 2 小时总 soak，也属于同一最终阶段。F0 日常门禁必须保留
较小规模、可快速执行的同语义正确性 fixture，不得删除性能预算或把最终门禁永久忽略。

## 1. 目标

本计划解决的不是单个 bug，而是以下概念在当前产品路径中仍存在交叠：

- `InteractionMode::{Chat, Task, Auto}` 同时影响工具可见性、TaskRun admission、UI、CLI、
  channel 和诊断投影；
- `PlanTask`、framework `Task`、`TodoStatus`、`TodoItem` 在类型注释上已经声明权威关系，
  但部分运行时路径仍直接使用 Todo 状态做调度和完成判断；
- Conversation Agent、TaskRun Subagent、一次 Subagent attempt 和后台 command handle 的身份
  容易被统一称为“任务”或“Agent”；
- 用户 steer、Agent message、Subagent guidance、follow-up 和 Task revision 修改缺少一个统一的
  产品语义表；
- Agent Router、SubagentControlService、tracked steer 和 surface command 已分别具备较完整能力，
  但模型没有一个统一、最小、无重复的协作控制面。

最终目标是形成以下正交模型：

```text
Conversation Agent
  = 长期身份 + transcript + optional goal + 多个顺序 Turn

Turn
  = 一次输入 admission + ReAct execution + typed terminal

TaskRun(revision)
  -> PlanTask / framework Task
  -> SubagentRun(attempt)

Message
  = 精确地址 + 因果关系 + 投递生命周期

TodoItem
  = Task graph 的只读 surface projection

InvocationCapabilitySnapshot
  = 本次 invocation 可见工具和资源事实
```

完成后，产品不再需要 Chat/Task/Auto mode 来决定路由。普通对话可以在需要时显式调用
`task_create/task_update/task_list/task_execute`；是否进入正式任务执行由真实 TaskRun binding 和
工具调用决定，不由 mode 标签决定。

## 2. 业界借鉴与取舍

### 2.1 Codex

借鉴：

- Thread、Turn、Item 分层；
- root-scoped 精确寻址；
- `message` 与 `follow-up` 的唤醒语义分离；
- 接收方运行时决定 start/steer；
- event/cursor wait，不依赖高频状态轮询；
- Subagent completion 是结构化事件和消息，不由 UI 文案推断。

不复制：

- 不把 Codex 当前进程内 mailbox 当作 EKO durable delivery 实现；
- 不把 Codex V2 Subagent 的 App 直接输入限制照搬成 EKO 用户权限门；
- 不把 Codex 的内部 AgentPath 当作跨 workspace 持久身份；
- 不把当前工具名和数量当作稳定产品合同。

### 2.2 Claude Code

借鉴：

- Task CRUD 与后台 execution handle 分离；
- Explore、Plan、Implementer、Reviewer 等角色由 prompt、工具集、模型配置和输出合同组合；
- typed user question、review finding 和 worktree 生命周期；
- Plan 是 artifact 和行为约束，不是复杂运行时状态机；
- Tool、Skill、Subagent、Plugin 和 MCP 是不同扩展层。

不复制：

- 不重新引入 `TodoWrite` 或第二套 Task store；
- 不保留 `EnterPlanMode/ExitPlanMode` 作为 EKO 产品 mode；
- 不硬编码某一版本的工具数量或角色数量；
- 不让 TaskOutput/TaskStop 成为第二套任务状态权威。

### 2.3 证据边界

本计划基于以下已完成调研和当前 checkout：

- [Agent 协同 ADR](../echo-agent-cli/docs/adr/0001-agent-collaboration.md)
- [Codex 工具能力目录](../echo-agent-cli/docs/adr/0002-codex-tool-capability-catalog.md)
- [Claude Code 能力目录](../echo-agent-cli/docs/adr/0003-claude-code-capability-catalog.md)
- [tracked steer ADR](../echo-agent/docs/adr/0003-tracked-steer-lifecycle.md)
- 当前执行基线见第 0 节；`c6379aa`/`04092ec` 仅是本计划最初调研快照

2026-08-26 本次刷新中，OpenAI Docs 搜索接口返回 404，页面返回 HTTP 403；Anthropic 官方
页面在前一轮刷新中连接超时。因此外部工具名称仍视为版本快照，本计划的架构结论只依赖跨
实现共性和本地可核验源码。

## 3. 当前基线

### 3.1 已有且必须复用

| 能力                    | 当前权威                                                           | 结论                                                         |
| ----------------------- | ------------------------------------------------------------------ | ------------------------------------------------------------ |
| 普通与 Task turn 驱动   | `AgentTurnDriver` + EKO `drive_chat`/turn lifecycle                | 继续收敛，不新增第二个 stream loop                           |
| 用户运行中追加          | `Agent::steer_input_tracked`                                       | 作为 accepted/drained/settled 唯一实时事实                   |
| framework Task graph    | `TaskRevisionService`、`PlanValidator`、`RuntimeDagExecutor`       | 保留唯一 DAG、revision 和 ready frontier                     |
| EKO TaskRun             | TaskRuntime journal、store、completion gate                        | 保留产品 goal、review、worktree、surface 策略                |
| Task Subagent 控制      | framework `SubagentControlRegistry` + EKO `SubagentControlService` | 保留 exact attempt、guidance、interrupt                      |
| Conversation Agent 消息 | EKO `AgentRouter`                                                  | 保留 durable queue、group、correlation 和 cold/live delivery |
| Subagent 执行           | `SubagentRegistry` + `SubagentExecutor`                            | 保留唯一 dispatch、hook、timeout、isolation                  |
| 工具可见性              | invocation `visible_tools/disabled_tools` + ToolSearch             | 保留 snapshot 原语，删除 mode 选择器                         |
| Todo                    | `PlanTask -> TodoItem` surface projection                          | 收紧为单向投影，不再反向拥有运行时语义                       |

### 3.2 当前已观察到的交叠

以下计数只描述 `echo-agent-cli/main@04092ec` 历史快照，F0 必须在 `e7d9e90` 重新生成：

- `InteractionMode`、`requested_mode`、`observed_path` 相关引用约 288 处，分布约 31 个
  Rust/TypeScript/文档文件；
- Todo 相关引用约 599 处，分布约 28 个核心 Rust/TypeScript 文件；
- `PlanTask.status` 当前仍保存 `TodoStatus`，executor、completion gate、recovery 和 store 多处直接
  读取或写入它；
- `AgentRouter` live delivery 在当前基线已经使用 `steer_input_tracked` 并等待 `Drained`；剩余缺口是
  cold/live receipt vocabulary、其它 surface 的 legacy steer/follow-up 策略和 Subagent control drain；
- Conversation Agent 控制主要暴露给 CLI/Tauri，Task Subagent 控制主要暴露给 GUI/TUI/CLI，
  模型侧只有初始 `agent_tool` dispatch，没有统一 list/inspect/message/wait 控制面；
- `tool_exposure` 仍按 Chat/Task/Auto 生成可见工具集合，Task mode 还会在模型调用前预构造 formal
  run identity。

### 3.3 与正在进行的五任务重构关系

本计划不是第六套并行 runtime 重构，而是现有 Task 2、Task 4、Task 5 的收敛覆盖层：

| 既有工作流                  | 本计划复用内容                                                       | 本计划不应抢占的文件/职责                             |
| --------------------------- | -------------------------------------------------------------------- | ----------------------------------------------------- |
| Task 2 EKO Control Surface  | workspace/conversation resolver、surface parity、typed IPC           | 当前 active 的 `state.rs`/channel/surface 收口        |
| Task 4 Task/Subagent Kernel | generic Task graph、Subagent identity、isolation/result              | framework Task schema 与 DAG authority                |
| Task 5 Runtime-State Kernel | AgentTurnDriver、journal、safe point、boot recovery、durable mailbox | TaskRuntime store/executor 和 chat journal 主路径迁移 |

生产改动必须等待“架构重构总负责人”发布 Task 5 当前阶段的冻结基线。冻结前只允许完成本计划
的 characterization tests、schema/引用清单和设计 review，不允许在旧基线上开始 mode 删除。

## 4. 不可违反的权威规则

1. **一个 turn driver**：所有 surface 和 TaskRun 最终进入同一个 framework turn driver。
2. **一个任务图**：单 Task、Todo 列表、依赖 DAG 都属于同一个 revisioned TaskRun graph。
3. **一个 Subagent executor**：tool dispatch、TaskRun dispatch 和 Team 都经过
   `SubagentExecutor`。
4. **两个持久消息域但不重复事实**：Conversation Agent 消息由 AgentRouter journal 拥有；
   TaskRun exact-attempt control 由 TaskRuntime journal 拥有。
5. **Todo 只读投影**：UI 不得直接把 Todo 状态写回运行时；修改通过 `task_update` 或明确控制
   命令进入权威 Task graph。
6. **接收和消费分离**：Persisted、MailboxAccepted、Drained、TurnSettled、BusinessAccepted
   是不同边界。
7. **没有 mode 替身**：不得把 `InteractionMode` 重命名成 `ExecutionStyle`、`RouteMode` 等继续
   承担相同职责。
8. **普通 chat 不伪造 TaskRun**：只有 `task_create/task_execute` 或明确产品 trigger 才建立 formal
   run。
9. **TaskRun Subagent attempt 不变成长对话**：长期多 turn 协作属于 Conversation Agent；
   SubagentRun 保持 revision/attempt 可核验。
10. **surface 不是权威**：GUI/TUI/CLI/channel 只渲染 typed event/receipt。

## 5. 目标通信合同

### 5.1 用户输入

```text
用户输入到 Conversation Agent
  -> active regular turn: tracked steer
  -> no active turn: start new turn
  -> active non-steerable boundary: durable queue, terminal 后 start new turn
```

判定由接收方 turn runtime 完成，surface 不实现“steer 失败后自己猜 follow-up”的竞态逻辑。

### 5.2 Agent 消息

```text
agent_message
  -> 只表达信息投递
  -> 不为普通 idle Conversation Agent 自动制造模型调用

agent_followup
  -> 表达新的 Conversation Agent 工作
  -> idle 时启动新 turn
  -> running 时在安全边界进入，或等待当前 turn terminal
```

TaskRun Subagent 不接受模糊 follow-up：

- active attempt 的纠偏使用 exact-attempt message；
- 尚未开始的已声明 next attempt 使用现有 guidance；
- 新增或改变正式工作使用 `task_update(base_revision)`；
- retry/resume 使用 TaskRuntime 控制，不通过自由文本偷偷改图。

### 5.3 Receipt

统一语义投影：

```text
Persisted
  -> Claimed
  -> MailboxAccepted
  -> Drained
  -> TurnSettled(completed | failed | cancelled | dropped)
  -> BusinessAccepted(optional completion gate)
```

底层 store 可以保留领域特定事件名，但所有 surface 和模型工具必须能映射到这套无歧义合同。

## 6. 迭代计划

## Iteration 0：冻结基线与 characterization

**工期**：2-3 人日

**依赖**：第 0 节列出的 child 与 superproject 基线已经发布。该依赖已满足。

**工作**：

1. 记录 `echo-agent`、`echo-agent-cli`、generated DTO 和顶层 gitlink 精确基线；
2. 生成 mode、Todo、Agent Router、SubagentControl、Task tool 的定义/注册/主路径引用清单；
3. 建立现状行为 fixture：
   - idle user input；
   - active steer；
   - non-steerable input；
   - live/cold Agent message；
   - active/future Subagent guidance；
   - Todo/Task status rebuild；
   - GUI/TUI/CLI/channel tool visibility；
4. 冻结当前 schema budget、event sequence、restart 和 long-horizon 基准；
5. 为 artifact/review history 保留快速规模 fixture，覆盖结果一致性、restart 和 bounded projection；
   10k/100k 完整性能预算只登记为 F7 显式门禁；
6. 在实施分支记录 framework、application、adapter 三层判定和重复搜索结果。

**退出门**：

- 每个待删除机制都有真实生产调用图；
- 所有 characterization tests 在未修改行为前通过；
- 与活跃 Task 5 文件没有未协调的并行写 owner。

## Iteration 1：统一 turn admission 与 delivery receipt

**工期**：4-6 人日

**主要所有者**：Task 5 runtime owner；framework 和 AgentRouter adapter 各一个明确 owner。

**工作**：

1. 保留当前 AgentRouter live delivery 的 `steer_input_tracked` + `wait_for_drained` 主路径，先用
   characterization 证明其真实边界，不再重建 tracker/mailbox；
2. GUI/TUI/CLI/channel 用户追加和 active Task Subagent message 迁到同一 tracked admission 合同；
3. 只有真实 mailbox accepted 后发布 MailboxAccepted；
4. 只有 ContextManager/turn input drain 后发布 Drained；
5. turn lease terminal 发布 TurnSettled，并保留 completed/failed/cancelled/dropped；
6. cold delivery 经统一 turn driver 返回同一 receipt 语义；
7. shutdown、retry、duplicate message 和 stale turn incarnation 使用同一幂等结算；
8. UI 旧 `Injected/Delivered` 映射迁移到新 typed projection。

**同阶段删除**：

- AgentRouter 对 legacy steer 成功即视为 injected 的推断；
- surface 从 foreground status 或 transcript 最后一条消息推断 drain/settled 的代码；
- 同一消息的重复 terminal reducer。

**退出门**：

- accepted-before-drain、terminal-before-drain、shutdown、ABA、duplicate delivery、cold/live parity
  全部有测试；
- restart 后每个非终态 receipt 要么恢复，要么明确进入 retryable/terminal failure；
- 没有新增应用侧 mailbox。

## Iteration 2：Task/Plan/Todo 单一权威

**工期**：6-9 人日

**主要所有者**：Task 4/Task 5 集成 owner；`task_runtime/types.rs`、`store.rs`、`executor.rs` 独占。

**工作**：

1. `PlanTask` 拆为 specification + canonical `TaskExecution`，内部状态统一使用 framework
   `TaskStatus`；
2. executor、completion gate、recovery、retry 和 dependency 判断停止读取 `TodoStatus`；
3. `TodoStatus/TodoItem` 只在 projection builder 和 generated DTO 边界出现；
4. 删除 Todo -> TaskStatus 的通用反向写入路径；用户动作转换为明确的 `task_update` operation、
   retry、skip、cancel 或 recovery decision；
5. `TaskPlan` 继续作为版本化 artifact，TaskRun journal 保存 revision facts；
6. `task_create/task_update/task_list/task_execute` 保持唯一模型任务 API；不新增 TaskGet/TodoWrite/
   Workflow store；
7. bounded `task_list` 通过 detail level/cursor 或 artifact reference 控制体积，不复制 store。

**同阶段删除**：

- runtime 内部直接以 `TodoStatus` 做 ready frontier、terminal 或 completion 判断；
- `TodoUpdated` 被当成 Task 状态 authority 的路径；
- 重复的 plan/task status conversion 和有损 adapter。

**退出门**：

- Todo DTO 删除后端字段不会改变 TaskRuntime replay 结果；
- graph revision round-trip 字段无损；
- dynamic patch、retry、cancel、timeout、skip、blocked、recovery 和 completion gate 回归全绿；
- 全仓库只有一个 DAG validator、ready frontier 和 Task status owner。

## Iteration 3：统一模型可调用的 Agent 协作控制面

**工期**：5-7 人日

**主要所有者**：EKO application；framework 只复用已有原语。

**工作**：

1. 在现有 AgentRouter 和 SubagentControlService 之上建立薄 routing adapter；
2. 定义带 discriminator 的目标：

```text
ConversationTarget(workspace_id, conversation_id)
TaskSubagentTarget(run_id, task_id, plan_revision, execution_id, attempt)
```

3. 提供最小模型工具：
   - `agent_list`
   - `agent_inspect`
   - `agent_message`
   - `agent_followup`
   - `agent_wait`
   - `agent_interrupt`
4. 现有 `agent_tool` 继续承担一次有界 Subagent dispatch；本阶段不同时新增同义
   `agent_spawn`；
5. `agent_wait` 使用 event cursor，等待 mailbox、Subagent terminal 或 needs-attention，不拥有
   任务终态；
6. 工具结果全部有界，长历史、输出和 evidence 使用 artifact/reference/cursor；
7. CLI/Tauri/GUI/TUI/channel 全部调用同一 service，不重复解析自由文本目标。

**同阶段删除**：

- surface-local Agent 地址拼接和状态映射；
- 模型通过 Shell/CLI 命令间接调用 Agent Router 的路径；
- 同语义的第二套 Agent list/message/wait store。

**退出门**：

- Conversation 与 TaskSubagent 两类目标错用时 typed rejection；
- stale attempt、wrong revision、wrong workspace generation、duplicate message 全部 fail closed；
- 模型工具不能绕过 TaskRun graph 修改正式任务关系；
- schema budget 不通过时先压缩描述或 deferred load，不提高预算掩盖问题。

## Iteration 4：删除 Chat/Task/Auto mode

**工期**：6-9 人日

**依赖**：Iteration 1-3 已合并；这是单一集成 owner 阶段，不与核心 TaskRuntime 改动并行。

**工作顺序**：

1. **工具可见性切换**：由 invocation capability snapshot、registered tool、DomainProfile、
   workspace resource 和明确 TaskRun binding 计算；
2. **run admission 切换**：普通 turn 默认无 run；只有 task tool 或明确 scheduler/background trigger
   创建/绑定 TaskRun；
3. **路由诊断切换**：保留 observed execution facts，但删除 `requested_mode` 作为决策输入；
4. **surface 切换**：删除 GUI segmented mode、CLI `/mode`、channel session mode 和 TUI mode 状态；
5. **wire contract 切换**：删除 `InteractionModeRequest` 及 generated TypeScript；
6. **prompt 切换**：删除 Chat/Task/Auto prompt 分支，改为稳定工具说明和当前 capability facts；
7. **持久化清理**：删除 mode 字段、reducers、默认值和兼容解析。本项目无兼容负担，不留 shim。

**禁止替代实现**：

- 不新增 `ExecutionStyle::{Direct,Formal,Auto}`；
- 不通过字符串 `route=chat|task|auto` 继续决定行为；
- 不让 UI 选择“复杂任务”后在后台偷偷预创建 formal run；
- 不因删除 mode 隐藏 Task tools，普通 Agent 始终可显式创建正式 graph。

**退出门**：

- `rg InteractionMode` 和生成 DTO 引用为零；
- 普通聊天不产生 TaskRun；
- 模型调用 `task_create` 后能建立 formal run 并继续 `task_execute`；
- 所有 surface 对相同输入获得相同 capability snapshot；
- tool schema budget、provider prompt snapshot 和五入口合同全绿。

## Iteration 5：长期 Agent 与 attempt-scoped Subagent 语义收口

**工期**：4-6 人日

**工作**：

1. Conversation Agent 明确支持多 turn、durable message、follow-up 和 optional goal；
2. TaskRun Subagent 保持一个 PlanTask execution attempt，不复用为无界对话；
3. `agent_followup` 只对 ConversationTarget 表达“必要时启动新 turn”；
4. active TaskSubagent 的纠偏走 exact attempt message；future guidance 必须绑定已声明 next attempt；
5. 新工作或验收变化通过 `task_update(base_revision)`，避免 message 静默改图；
6. Subagent terminal result 保持 typed summary/artifact/evidence/remaining_work；
7. 角色保持动态配置，不把 Explore/Reviewer/Implementer 写死为 runtime enum。

**同阶段删除**：

- 以 Subagent 名称或 UI title 作为唯一地址的路径；
- settled attempt 接收 late message 的宽松 fallback；
- 把 Conversation completion、Subagent terminal 和 Task completion 混为一个 `completed`。

**退出门**：

- 同一个 Conversation Agent 可连续完成多个 turn；
- stale Subagent attempt 永远不能影响新 attempt；
- follow-up 不改变 Task graph revision；
- Task graph 更新不会伪造消息 delivered receipt。

## Iteration 6：事件等待、恢复与 surface 对等

**工期**：5-8 人日

**工作**：

1. 为 AgentRouter 和 TaskRuntime 投影统一 cursor-based wait response；
2. wait 支持 timeout、cancel、needs-attention、terminal 和 restart cursor；
3. GUI/TUI/CLI/JSONL/channel 展示相同 Agent/Task/Todo/receipt 语义；
4. UI “Agent updated”只由 typed activity event 产生，不从文本猜测；
5. boot reconciliation 覆盖 persisted/claimed/accepted/drained/settled 的未完成组合；
6. cold Agent、unloaded conversation、workspace switch/delete 和 app shutdown 有 typed result；
7. long output、review findings 和 task evidence 使用 bounded projection。

**退出门**：

- wait 不需要高频 list polling；
- cursor 不重复交付已确认 terminal；
- workspace generation 切换不会错投；
- 五入口同 fixture parity 全绿；
- bounded functional probe 无 stranded receipt、active turn 或 Subagent handle；本阶段不运行长时 soak。

## Iteration 7：删除旧路径、文档和最终门禁

**工期**：3-5 人日

**工作**：

1. 全仓库搜索并删除 mode、旧 Todo authority、旧 Agent command adapter、重复 reducer 和死测试；
2. 删除已被 task tools 取代的 `create_complex_task/check_run_status/cancel_run` 等平行产品入口；
3. 更新所属子仓库 ADR、architecture、features、configuration 和 examples；
4. 更新 `echo-website` 中受影响的产品说明；若无相关内容，在交付说明写明不适用；
5. 重新生成 TypeScript DTO 并执行 path hygiene；
6. 完成全部提交门禁、fault matrix、10k/100k 全规模性能测试，以及 10 分钟与 1 小时 soak；
   若最终验收仍保留 2 小时 soak，也在本阶段执行；
7. 修复并验证所有 deferred 远端 CI；
8. 先提交/push `echo-agent`，再提交/push `echo-agent-cli`，同步 `echo-website`，最后更新
   superproject gitlink。

**退出门**：

- 没有双 driver、双 Task graph、双 mailbox、双 status reducer；
- docs/examples/website 与实现一致；
- `git diff --check`、Rust/GUI/frontend 全部门禁通过；
- `Cargo.lock --locked` 和 clippy `-D warnings` 在 CI 环境可复现。

## 7. 依赖和并行调度

```text
当前 Task 5 冻结基线
        |
        v
Iteration 0 characterization
        |
        v
Iteration 1 receipt/admission
        |
        +------------+
        |            |
        v            v
Iteration 2      Iteration 3
Task/Todo        Agent tools
        |            |
        +------v-----+
               |
        Iteration 4 mode removal
               |
        Iteration 5 identity semantics
               |
        Iteration 6 recovery/parity
               |
        Iteration 7 cleanup/gates
```

### 可并行范围

- Iteration 2 与 Iteration 3 可以并行，但不得同时修改 `task_runtime/store.rs`、`executor.rs`、
  `state.rs` 或 generated DTO；
- surface 团队可以在 Iteration 1-3 期间只做 fixture/renderer 准备，不能提前删除 mode；
- docs/examples 可以随所属实现阶段更新，最终由 coordinator 统一复核；
- mode 删除必须单 owner 完成，避免各 surface 留下不同 fallback。

### 文件所有权建议

| Owner           | 独占范围                                                                            |
| --------------- | ----------------------------------------------------------------------------------- |
| Runtime receipt | framework tracked steer、`state.rs` live/cold delivery、AgentRouter receipt adapter |
| Task/Todo       | `task_runtime/types.rs`、`store.rs`、`executor.rs`、completion/recovery             |
| Agent tools     | `agent_router.rs`、`subagent_control.rs` 的薄 adapter 和 tool registration          |
| Surface         | Tauri/TUI/CLI/channel/frontend renderer 和 generated consumers                      |
| Coordinator     | `MASTER-PLAN.md`、Cargo.lock、generated snapshot、最终 merge 和 gitlink             |

## 8. 工期与人员

| Iteration                 |           人日 |
| ------------------------- | -------------: |
| 0 基线与 characterization |            2-3 |
| 1 admission/receipt       |            4-6 |
| 2 Task/Todo authority     |            6-9 |
| 3 Agent control tools     |            5-7 |
| 4 mode 删除               |            6-9 |
| 5 Agent/Subagent identity |            4-6 |
| 6 recovery/surface parity |            5-8 |
| 7 cleanup/gates           |            3-5 |
| **总计**                  | **35-53 人日** |

建议排期：

- 1 名熟悉全链路的高级工程师：约 8-11 周；
- 2 名工程师 + 1 名协调/reviewer：约 5-7 周；
- 3 条以上生产改动并行不建议，核心文件冲突和双权威风险会抵消收益。

以上不包含当前 Task 5 尚未完成部分；如果其冻结基线推迟，本计划日历时间等量顺延。工期也不
包含 provider/network 不稳定导致的外部等待，但包含本地完整门禁和 fault tests。

## 9. 验收矩阵

| 场景                                | 必须结果                                           |
| ----------------------------------- | -------------------------------------------------- |
| idle Conversation 用户输入          | 启动一个新 turn，无 TaskRun                        |
| active regular turn 用户输入        | tracked steer，receipt 经 accepted/drained/settled |
| active non-steerable turn           | durable queue，不能丢失或伪造 drain                |
| idle queue-only Agent message       | 持久等待，不制造无意模型调用                       |
| idle Agent follow-up                | 启动同一 Conversation Agent 的新 turn              |
| active Task Subagent message        | 精确 execution/attempt，安全边界注入               |
| settled/stale Subagent message      | typed rejection，不路由到新 attempt                |
| Task graph patch                    | base revision CAS，Todo 自动投影                   |
| UI Todo 点击                        | 明确 task operation，不直接改 projection store     |
| ordinary Agent 创建复杂任务         | 显式 `task_create`，随后 `task_execute`            |
| restart before drain                | 恢复或可解释重试，不能标 Delivered                 |
| restart after drain before terminal | receipt 保留 Drained，等待 owning turn settlement  |
| workspace generation change         | 旧地址/claim 不得写入新 generation                 |
| GUI/TUI/CLI/channel                 | 同一输入、事件和 receipt 语义                      |

## 10. 提交门禁

每个 iteration 必须完成相关专项测试和所属 workspace 提交门禁。最终至少包括：

```bash
cd echo-agent
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo clippy --workspace --lib --bins --all-features --locked -- \
  -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::unreachable
cargo test --workspace --all-targets --all-features --locked
cargo check --workspace --lib --no-default-features --locked

cd ../echo-agent-cli
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo clippy --workspace --lib --bins --all-features --locked -- \
  -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::unreachable
cargo test --workspace --all-features --locked
cargo check -p echo-agent-app-core --no-default-features --locked
cargo check --no-default-features --features gui --bin echo-agent-tauri
cargo test --no-default-features --features gui

cd web-frontend
npx prettier --check "src/**/*.{ts,tsx}"
npm test
npm run build
```

任何失败都必须修复后再提交。不得提高 schema/test budget、使用 `--no-verify`、跳过 `--locked`
或把失败标记为“与本次无关”。

## 11. 完成定义

本计划只有同时满足以下条件才算完成：

1. Chat/Task/Auto mode 从 Rust、IPC、generated TS、GUI/TUI/CLI/channel 和持久化中删除；
2. Todo 只作为 Task graph 投影存在，不参与权威调度和完成判断；
3. Conversation Agent 与 Task Subagent attempt 有不同且精确的地址合同；
4. 模型可以 list/inspect/message/followup/wait/interrupt，但这些工具只调用现有 authority；
5. Agent message 和 user steer 都能证明 accepted、drained 和 turn settled；
6. 普通 chat、TaskRun、background、cron 和 channel 共用一个 turn driver；
7. 所有 surface 行为对等；
8. 旧代码、旧 DTO、旧命令、旧文档和重复测试被删除；
9. restart、ABA、cancel、timeout、duplicate、workspace switch 和长时间 soak 全绿；
10. 子仓库提交、远端可达 commit 和顶层 gitlink 顺序正确。
