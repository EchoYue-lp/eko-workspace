# EKO 总纲(MASTER-PLAN)

> **跨上下文单一事实源**。新窗口先读本文，再按本文链接读取专项 spec/plan。
> 本文件只保留：当前状态、前向路线、活跃决策、验收标准和代码入口。逐轮调试记录不进入本文。
> **最后更新**：2026-08-16
>
> 专项实施账本不是第二份产品事实源。EKO 的 coding、数据分析、学术研究与医学研究实施证据记录在
> [`echo-agent-cli/docs/MASTER-PLAN.md`](../echo-agent-cli/docs/MASTER-PLAN.md)，其范围与结论受本文的产品边界、术语和验收标准约束。

## 一、产品定位与当前结论

EKO 是运行在用户本机的个人超级智能助理。`echo-agent` 是独立、可复用的通用 Agent 框架；`echo-agent-cli` 是 EKO 应用层；`echo-website` 是官网。

当前产品迭代收敛为五个方向，优先顺序固定为：

1. **Agent 主流程**
2. **工具可靠性**
3. **任务完成率**
4. **上下文效率**
5. **Tauri / TUI / CLI / channel / cron 对等**

| 路线 | 优先级 | 状态 | 当前断点 |
|---|---|---|---|
| Agent 主流程 | P0 | M1、M2、M3、M11 已完成 | 主流程与专业域 Subagent 编排进入持续回归门禁 |
| 工具可靠性 | P0 | M4、M5、M6、M12 已完成 | 探索性统计与正式推断边界进入持续回归门禁 |
| 任务完成率 | P1 | M7、M8 已完成 | ownership/dependency 与 writer merge 进入持续回归门禁 |
| 上下文效率 | P1 | M9 已完成 | 单 run usage/cache/protected context 进入持续回归门禁 |
| 五入口对等 | P1 | M10 已完成 | 共享事件合同与能力矩阵进入持续回归门禁 |

**不再建设自进化指标平台。** 现有 evolution 能力只保留按需诊断和用户显式触发的 review/inbox，不扩展为后台评分、自动建议或自动修改系统。

### 明确不规划

- 自动改善建议系统。
- EKO 本地 EvalRunner 或 benchmark loop。
- 自动重写基础 prompt。
- 微调数据生成与 TrajectorySaver 产品路径。
- 指标驱动自动修改 memory、rule 或 skill。
- EKO 引入 SQLite。

### 低优先候选

- 飞书 webhook 支持图片、文件等多模态消息。
- 删除确认无调用的 `isolated.rs` 死路径。
- Evolution hook fire 站点补齐。当前没有消费者，不为“事件完整性”单独建设。
- Hosted Agent Service。继续等待真实消费需求。

---

## 二、业界调研与架构取舍

本轮路线参考了以下成熟实现：

- [OpenAI Codex app-server](https://github.com/openai/codex/blob/main/codex-rs/app-server/README.md)：交互主模型是 `Thread -> Turn -> Item`；thread 可 start/resume/fork，turn 可 start/steer/interrupt；进度通过 `item/started`、`item/completed` 和 delta 流表达，最终由 `turn/completed` 收敛状态和 token usage。
- [OpenAI Codex exec JSONL](https://github.com/openai/codex/blob/main/codex-rs/exec/src/exec_events.rs)：非交互消费者读取稳定事件流，而不是重新推断运行状态。
- [Claude Code common workflows](https://code.claude.com/docs/en/common-workflows)：会话可 continue/resume；plan mode 在批准前只读；并行写任务使用 worktree 隔离。
- [Claude Code subagents](https://code.claude.com/docs/en/sub-agents)：Subagent 使用独立上下文、独立工具/权限配置，主会话只接收汇总；写入型 subagent 可使用 worktree 隔离。
- [Claude Code checkpointing](https://code.claude.com/docs/en/checkpointing)：checkpoint 与会话一起持久化，恢复后仍可 rewind；checkpoint 是局部恢复手段，不替代 Git。
- [Claude Code hooks](https://docs.anthropic.com/en/docs/claude-code/hooks)：工具前后、工具失败、Subagent start/stop 和 session resume 使用独立事件边界，取消与恢复不靠一个模糊 terminal 推断。
- [Temporal Activity definition](https://docs.temporal.io/activity-definition)：Activity 可能因重试而重复执行，外部写入需要幂等键或可验证 postcondition；“durable execution”不等于任意副作用天然 exactly-once。
- [LangGraph persistence](https://docs.langchain.com/oss/python/langgraph/persistence)：以 checkpoint/thread identity 保存执行进度，恢复从持久化事实继续，而不是重放整段对话推断已完成节点。
- [Claude Code session storage](https://code.claude.com/docs/en/agent-sdk/session-storage.md)：本地会话是主存储，sidecar/subagent 使用 session 子路径，主 session 删除时级联清理，并由宿主显式定义 retention。
- [OpenAI Codex rollout recorder](https://github.com/openai/codex/blob/main/codex-rs/core/src/rollout.rs) 与 [thread rollout truncation](https://github.com/openai/codex/blob/main/codex-rs/core/src/thread_rollout_truncation.rs)：完整 rollout 独立持久化，模型使用按稳定 turn 边界派生的有界历史，不破坏原始记录。

跨系统共性与 EKO 取舍：

1. **一个生命周期，多种触发器。** Chat、Task、Auto、后台、cron 和 channel 不各建状态机；它们只决定输入、策略和渲染，统一进入同一 run/turn 驱动。
2. **事件是事实，UI 是投影。** 工具、Subagent、任务和终态以稳定 identity 的事件记录；GUI/TUI/CLI 从同一事件合同投影，不自行猜状态。
3. **Plan 是 artifact，不是审批状态机。** 计划可编辑、可审阅；批准由 prompt/权限/HITL 驱动，不把 Planning/AwaitingApproval/Ready 等塞进运行时主状态机。
4. **恢复依赖持久化事实。** 已完成 tool call、task node、Subagent 结果和终态必须可辨认；resume 跳过已完成副作用，不靠模型“记得做过”。
5. **Subagent 隔离上下文和写入所有权。** 主 Agent 只接收结构化结果；并行写入必须有文件所有权或 worktree 隔离。
6. **准确 usage 优先，估算只作诊断。** provider/API usage 是记账权威；本地估算用于预算预警、prompt assembly 和缺失 usage 时的观察，不伪装成准确值。
7. **副作用采用分级恢复，不宣称通用 exactly-once。** 已持久化完成事实直接复用；可安全重放的只读操作允许重试；开始但无终态的写入/执行/网络操作必须阻塞自动恢复，由用户检查真实状态后选择 retry 或 skip。

### 框架与应用边界

| 能力 | 归属 | 原因 |
|---|---|---|
| EventEnvelope、tool identity、通用 checkpoint、cancel/timeout 原语 | `echo-agent` | 任意 Agent 都需要的执行合同 |
| provider usage/cache 归一、通用压缩/投影原语 | `echo-agent` | 与 EKO 产品形态无关 |
| 探索性统计原语、沙箱化 `run_code` | `echo-agent` | 通用数据探索与代码执行能力，不承载 EKO 领域策略 |
| Chat/Task/Auto 路由、后台/cron 生命周期接入 | `echo-agent-cli` | EKO 产品策略 |
| TaskRun、plan artifact、Subagent 结果投影、三端 DTO | `echo-agent-cli` | EKO 的任务与 UI 模型 |
| 正式分析脚本合同、lineage、Notebook/报告 UI | `echo-agent-cli` | coding-first 数据分析的产品工作流 |
| 文件所有权、writer semaphore、worktree 合并策略 | `echo-agent-cli` | 编码型本地助理的产品约束 |
| 超长日志展示、诊断面板、Review Inbox | `echo-agent-cli` | 用户体验与本地持久化策略 |

拿不准时先放应用层；确认多个复用方都需要后再下沉。不得因 EKO 没调用就删除 `echo-agent` 的合理公开能力。

---

## 三、当前基线

以下能力已经存在，后续应复用和收敛，不得平行重建：

- GUI/TUI/channel 已通过 `drive_chat` 统一驱动，差异主要在 `ChatSink`。
- 框架已有 versioned `EventEnvelope`、稳定 sequence/event identity、tool parent identity 和 terminal exactly-once 校验。
- 对话与 runtime checkpoint 使用文件持久化；CLI 不启用 SQLite。
- ReAct 工具批次后可保存安全 checkpoint，恢复时可识别已完成 tool call。
- TaskRuntime 已有 plan/task/run、暂停/恢复/取消、cron 和后台执行基础。
- 工具已有 call_id、流式 stdout/stderr/log、明确成功/失败/取消终态、UTF-8 安全截断和大结果 spill。
- Shell/file/search/browser/MCP/subagent 已有 GUI/TUI 专属或通用 renderer 基础。
- Subagent 已有 Sync/Fork/Teammate/Team、独立上下文、timeout、checkpoint、worktree/tmpdir 隔离和 writer 文件锁基础。
- 框架 `agent_tool` 的子取消令牌以当前 invocation 的 `ToolContext.cancel` 为权威；主运行被停止时，后台 Subagent 必须同步取消，不得继续脱离运行。
- 框架以 `TaskSpec + TaskExecution + TaskStatus` 作为唯一动态任务模型，`RuntimeDagExecutor` 是唯一 DAG 遍历内核；框架富 hooks/verifier/store 记录明确命名为 `ManagedTask`，LLM authoring artifact 明确命名为 `PlanTaskSpec`，二者都不拥有第二套调度或结构校验。
- EKO 的 TaskRun 文件权威、统一 `task_create/task_update/task_list/task_execute` 工具壳和产品 policy 留在应用层，Rust 文件/UI DTO 明确命名为 `EkoTaskSpec/EkoTaskExecution` 并通过 checked round-trip adapter 接入框架；不启用框架 SQLite/background-task store，隐藏其 `spawn/check/list` 工具，并在 TaskRuntime 注册边界移除全局 `todo_write`，避免两套 task ID、状态和执行循环。
- Prompt assembly 已有模块预算、protected token 估算、稳定 projection、provider usage/cache 观测。
- GUI 会把旧版单模型配置投影为默认 configured model，并补齐 effective context window；上下文环不再因 `configured_models` 为空而显示未知。
- GUI 工具调用折叠态只显示单行语义摘要，完整参数仅在展开后展示和复制；`task_create` 固定显示为 `task_create + title`。主消息投影与会话恢复必须补回尚未进入最终 execution round 的稳定 tool call，不能让计划创建行在 TaskRuntime handoff 后消失。
- TUI 已接任务、plan、Subagent、HITL、记忆、附件、Browser/MCP 和会话恢复基础。
- Evolution 已收缩为按需诊断；EKO 不再启用 eval/improve 产品链路。
- 统计能力已拆成两层：`exploratory_statistics` 只做描述性摘要；正式检验、回归和建模由数据 Subagent 生成可审阅的 SciPy/statsmodels/R 脚本，经 `run_code` 执行并保存 artifact。
- `echo-agent` 根 manifest 同时是 `echo_agent` package 与 Cargo workspace root；7 个 split crate、
  `echo-rust-learning` 和 `echo-agent-examples` 与根 package 共享 `Cargo.lock`、`target` 和统一门禁。

### F2-F5 收口里程碑（2026-08-28）

状态：plan_03 完成并合入本地 `main`。framework producer 为
`302453b174086c3795dc026d16eeb668ecc66bed`，CLI consumer 为
`d09f11c7878474d0e01ba2562309d5890e369554`；CLI 继续通过相对路径 `../echo-agent`
消费 framework。F5 保持单一 squash，260 项测试 panic lint 已由独立 hygiene 提交清零。

framework、CLI、GUI 与 frontend 完整适用门禁均有真实 exit 0 证据；generated DTO 已纳入
标准 Prettier 检查。2k/10k、10k/100k 性能门、长时 soak 和 Final Gate 仍明确后置，ADR 0016
记录的 R1/P0 bounded-query residual 也未宣称解决。本里程碑未 push、未 release；下一阶段是
F6/R1 与剩余 lifecycle/persistence 收口。

### Task 3：Public Framework Boundary（2026-08-23）

状态：实现完成，提交门禁执行中。权威分支为两个独立仓库的
`refactor/public-framework-boundary` worktree；合并顺序固定为 framework 先、EKO adapter 后。

- framework 通用机制：`ToolCapabilities/ToolPack/CommandPolicy`、`StandardToolPack`、typed
  `PermissionMode`、显式 `DataRoot` 和完整 `echo_agent` facade。
- EKO 产品策略：`EkoConfig`、产品 prompt/model catalog、channel/TUI/server 配置、permission
  DTO、Theme/Monitor/OutputStyle 与 coding auto-memory policy。
- 适配边界：CLI/app-core 不再直接依赖 `echo_core`；EKO 把 typed config、permission 和显式
  路径传给 framework，`tool_exposure` 继续作为 invocation 可见性的产品权威。
- crate DAG：`FileSystemSkill/ShellSkill` 已归 `echo-tools`，`echo-execution -> echo-tools` 为 0。
- consumer gate：新增不发布的 `echo-agent-examples`，依赖项只有 `echo_agent`；smoke/acceptance
  断言归 `tests/`；Rust 教程归 `echo-rust-learning/docs`，内部知识归 `docs/internal`。

本次边界取舍参考了 Cargo 官方 examples 契约（example 可使用 package public API）、Codex 的
sandbox/approval 分离，以及 Claude Code 的 tools/permission 分离：能力描述、工具组合与审批策略
是三个独立合同，不能继续由工具名列表或产品字符串同时承担。

### 已知需要校正的事实

- `run_code`：框架已删除 bare fallback，命令携带最低隔离要求；EKO 启动时探测本机 OS sandbox，不可用时从主 Agent 与 Writer Subagent 移除该工具。交互式 terminal 仍是用户主动能力，不受此合同或 agent `permission_mode` 门控。
- 超长工具结果已统一为完整 artifact + 有界模型/会话投影；GUI/TUI/CLI 共享路径、大小、SHA-256 和 retention，conversation 删除级联清理。
- 三端共享事件源已经具备，但各端仍有独立 match/reducer，新增事件存在漏接风险，需要合同测试和能力矩阵持续审计。

---

## 四、已完成路线 1：Agent 主流程回归合同(P0)

### 目标

从用户输入到最终结果只存在一条权威生命周期；模式、触发方式和前端只改变策略或渲染，不改变状态语义。

### Phase A：生命周期盘点与契约固化

- 画出 GUI/TUI/CLI/channel/cron/background 从入口到 `drive_chat`、`launch_cron_run`、TaskRuntime 的真实调用图。
- 列出所有 run/turn/task 状态、转换发起者、持久化位置和终态写入点。
- 找出重复状态机、同义状态、平行数据源和 UI 自行推断逻辑。
- 固化最小权威状态：`pending/running/paused/completed/failed/cancelled`。等待批准、计划编辑等作为 interaction/artifact 状态，不扩张主运行状态机。

**验收**：审计明确每个状态的当前写入路径和竞争点；每个入口都有对应的契约测试用例定义；文档中能回答“谁创建、谁推进、谁结束、谁恢复”。修复后的单一写入路径与可执行 contract tests 在 M2 同提交落地。

### Phase B：统一 trigger 到 Run 的适配层

- 将 interactive chat、formal task、background、cron、channel 统一映射为 `RunRequest + TriggerContext + ExecutionPolicy`。
- `Chat/Task/Auto` 只决定工具面、是否鼓励 plan、是否允许后台化，不复制 executor。
- 普通短 Chat 可以没有正式 TaskRun，但一旦创建复杂任务或后台执行，必须懒创建并绑定唯一 run identity。
- 删除旧入口的特殊完成逻辑和重复 terminal 写入。

**验收**：同一输入通过三种前端和不同 trigger 执行时，生命周期事件序列与终态语义一致。

### Phase C：暂停、恢复、取消与进程重启

- 明确 pause 是可恢复挂起，cancel 是不可继续的终态；两者不混用。
- 每个有副作用的工具批次完成后持久化 checkpoint；恢复按 call_id/task node 跳过已完成工作。
- interrupted/stream error/process crash 后恢复时，先校验 tool_call/tool_result 配对、task node 和 artifact 完整性。
- 取消必须传播到模型流、工具、Subagent、审批等待和 TaskRuntime，并最终只产生一个 cancelled terminal。

**验收**：建立 crash/resume、pause/resume、cancel-during-tool、cancel-during-HITL、cancel-during-subagent 场景测试；写工具不重复执行。

### Phase D：模式路由稳定性

- 为 Chat/Task/Auto 建立明确、短小、可测试的路由规则和解释字段。
- Auto 的选择结果进入 trace，但不成为新的运行状态。
- 路由错误允许用户同 turn steer 或显式切换，不要求重建会话。

**验收**：覆盖短问答、单工具操作、多步编码、需审批、需并行、需后台化等 fixture；路由结果稳定且可解释。

---

## 五、已完成路线 2：工具可靠性回归合同(P0)

### Phase A：统一错误分类与恢复决策

建立跨 shell、file、search、Browser、MCP 的通用错误分类：

- `invalid_arguments`：参数形状或明显输入错误，可让模型修正后重试。
- `unavailable`：依赖、server、browser session 或能力不可用，先恢复连接或降级。
- `timeout`：记录是否产生部分副作用，再决定 retry/continue/fail。
- `cancelled`：不自动重试，收敛为取消。
- `transient`：有限重试，使用 backoff/jitter。
- `permanent`：停止重试，给出简短诊断。
- `partial_side_effect`：必须先核对真实状态，禁止盲重放。

恢复策略属于工具执行合同；“连续多个 run 出现同类错误”的简短诊断属于 EKO 应用层观测。

**验收**：相同错误在流事件、最终 ToolResult、trace、checkpoint 和三端 UI 中分类一致。

### Phase B：幂等、超时、取消和重复调用

- 为可重试工具定义 idempotency key 或可验证的 postcondition。
- 写文件采用 revision/content hash；shell 根据命令副作用分类；Browser/MCP 根据 tool contract 判断能否重试。
- call_id 全链路稳定，重复 terminal 幂等；resume 不重新发射已完成副作用。
- timeout 与 cancel 分开记录，保留 partial output 和实际退出原因。

**验收**：参数纠错重试、网络瞬断、MCP 重连、Browser tab 丢失、shell timeout、重复事件、进程重启均有专项测试。

### Phase C：日志与 artifact

- 保持对话和会话文件只保存有界最终投影。
- 超过阈值的完整 stdout/stderr/tool payload 写独立 artifact，事件只保存摘要、路径、大小、hash 和保留策略。
- GUI/TUI/CLI 都能打开同一 artifact；artifact 缺失时显示明确状态，不把它误判为工具失败。
- 定义按 run/session 清理策略，避免无限占盘。

**验收**：10MB+ 日志不膨胀 conversation；三端摘要一致且可访问完整 artifact。

### Phase D：`run_code` 真沙箱闭环

- 先证明 EKO 所有 `run_code` 注册路径都在执行前获得 `SandboxExecutor`。
- EKO 启动时缺 sandbox 应构建失败或禁用 `run_code` 并明确报错，不允许静默 bare 执行。
- 评估框架 `RunCodeTool` 无 sandbox 时改为 `Unavailable`，还是提供显式 opt-in 的 trusted local executor；禁止用 EKO 的 `permission_mode` 卡用户交互工具。
- 覆盖 Python/R、working_dir、timeout、cancel、stdout/stderr、sandbox unavailable。

**验收**：EKO 生产路径不存在 bare fallback；测试能证明命令经过 sandbox executor。

---

## 六、已完成路线 3：任务完成率回归合同(P1)

### Phase A：结构化 Subagent 结果合同

每个 Subagent 结束时必须返回统一结果：

- `status`：completed/failed/cancelled/timed_out。
- `summary`：完成了什么。
- `artifacts`：路径、类型、hash、生成者 execution_id。
- `verification`：运行了什么验证、结果如何、哪些未验证。
- `remaining_work`：未完成项和阻塞原因。
- `touched_files`：实际读写集合，用于所有权核验和汇总。

不得仅凭自然语言出现“完成”就把 task 标为 completed。

**验收**：缺少必要产物或验证时，父任务不能进入 completed；三端展示同一结果。

### Phase B：失败、超时和合理降级

- 区分可恢复失败、能力不可用、超时、取消和部分完成。
- retry 必须复用 checkpoint 和已完成 artifact；达到上限后由主 Agent选择降级、重分配或带缺口结束。
- Subagent 失败不得被主 Agent 汇总文本掩盖；最终 task 状态以结构化结果为准。

**验收**：Subagent timeout、单 Subagent 失败、多 Subagent 部分成功、synthesis 失败、resume 后成功均有测试。

### Phase C：并行文件所有权与依赖

- 计划阶段为写入任务声明 owner scope 和依赖；未知所有权默认串行或 worktree 隔离。
- 不相交文件可并行；相交文件必须串行、重新分配 owner 或通过显式 merge 阶段收敛。
- 调度器只依据显式依赖和 ownership 放开并行，不靠任务描述猜测。

**验收**：相交写入不覆盖，依赖未完成时下游不启动；worktree 产物合并失败能回报真实状态。

### Phase D：主 Agent 汇总与真实终态

- 主 Agent 汇总使用结构化 Subagent result，不重新扫描聊天文本猜结论。
- 完成判定同时检查 task node、required artifacts、verification 和 unresolved failures。
- UI 的 completed/failed/cancelled 与 runtime 权威结果严格一致。

**验收**：消除“显示完成但实际未完成”；加入 terminal consistency property tests。

---

## 七、已完成路线 4：上下文效率回归合同(P1)

### Phase A：准确 usage 与诊断统一

- provider/API 返回 usage 时作为权威；缺失时标记 unknown，并单独展示估算值。
- 统一 prompt、cached、cache creation、output、total 的 provider 归一语义。
- trace/诊断面板显示本轮 usage 来源：`provider` 或 `estimated`。

**验收**：不会把估算值写成准确 usage；各 provider fixture 覆盖 cache token 语义。

### Phase B：稳定前缀与重复注入治理

- 固定 system/canonical 前缀顺序和序列化格式，动态内容只进入 tail projection。
- workspace、skill catalog、memory、task brief 使用 marker/latest-wins 更新，不重复 append。
- 对每个模块记录 token、hash、是否命中稳定前缀、为何失效。

**验收**：相同 workspace/skills 下连续 turn 的稳定前缀 hash 不变；重复模块为零。

### Phase C：protected context 与压缩保真

- 限制 protected message 数量和体积，保护“任务状态、关键决策、已完成工具事实、artifact 引用”，不保护冗长原始日志。
- 压缩后校验 tool_call/tool_result 配对、run/task identity、已完成副作用和 pending interaction。
- 对超长工具结果继续使用截断 + artifact spill。

**验收**：压缩前后恢复同一任务不会重复已完成工具；protected token 超预算有明确诊断。

### Phase D：Prompt Observability

- 将 `PromptAssembly` 模块报告、provider usage、cache diagnostics、protected tokens 串成单个 run 诊断视图。
- 只做可观察性和人工优化依据，不重建 EvalRunner、自动评分或自动改 prompt。

**验收**：能从一个 run 回答“token 花在哪里、cache 为什么失效、哪些内容被保护/截断/溢出”。

### Phase E：用户输入归一化与长文本 artifact 化（已完成，保留为回归合同）

**背景**：用户粘贴或上传长日志时，纯文本此前全文内联进 LLM，后续 ReAct 回合和会话恢复还会重复携带。业界(Claude Code 50K 字符落盘、Codex goal_files)收敛到"引用 + 搜索 + 按需分页读"，而非全文内联。

**分层归属**（已确认）：
- **应用层**（echo-agent-app-core）：`PreparedUserTurn` 归一化、`InputResourceRef`、32 KiB 阈值判定、user-input artifact 落盘目录、五入口 + steer 统一预处理、会话持久化去 data URL。
- **框架层**（echo-agent）：仅扩展 `grep` 让它能解析 `ToolContext.output_artifacts.root_dir`（字段已存在），复用已有的 artifact 落盘/SHA-256/分页读基础设施（`ToolOutputArtifactWriter` / `read_artifact`），不新建第二套。

**Phase 1（2026-08-03 已完成，纯新增无切换）**：
- 新建 `echo-agent-app-core/src/prepared_turn.rs`：`PreparedUserTurn`（instruction + resources + mode_hint）、`InputResourceRef`（扩展 AttachmentRef，加 kind/delivery/元数据）、`UserTurnInput` 构造器、`to_message()` 单一合流点、阈值判定（32 KiB 或估算 4000 tokens）、UTF-8 安全落盘（`chars().take` preview、原子写 `.partial`→rename、SHA-256）。
- `attachments.rs`：`is_image_mime` 提升为 `pub(crate)`、新增 `AttachmentRef::to_input_resource()` 转换。
- `workspace/layout.rs`：新增 `WorkspaceLayout::user_input_artifacts()`（`.eko/artifacts/user-input/`，按需创建不进 ensure_dirs，与 tool-logs 一致）。
- `lib.rs`：注册 `prepared_turn` 模块。
- 10 个单元测试全绿（阈值/CJK/emoji/落盘/to_message/路径 sanitize）；clippy + 四条 panic lint + fmt 通过；attachments/persistence 回归无影响。

**Phase 2（2026-08-03 已完成——权威主路径已切换）**：
- `drive_chat`/`drive_chat_inner`（`chat_driver.rs:197`）签名改收 `&PreparedUserTurn`，`match multimodal` 合流块删除，`to_message()` 成为唯一合流点；mode_hint 拼接移入 `PreparedUserTurn::build`。
- 五入口 + steer 全部切换：GUI send（`chat.rs`）、GUI steer（`steer_chat_message`）、TUI send（`events.rs` send 路径 + `send_to_agent`）、TUI steer（`/steer`）、CLI REPL（`chat_with_agent`）、channel。各自通过 `UserTurnInput` + `resolve_user_input_spill_dir` 构造 `PreparedUserTurn`。
- `ensure_task_mode_run` 的 goal 改用 `turn.instruction`（spill 后是引用块，比原文更适合做任务目标）；attachments 仍来自 `ChatResources.attachments`。
- 双实现消除：内存版 `build_message` 已删除，其 3 个测试迁移到 `build_message_from_refs`（后者保留给 `executor.rs:2790/2948` subagent 重建路径）。
- 附件传播链不变：`ChatResources.attachments`、`TaskRun.attachments`、`executor.rs:2790/2948` 继续用 `AttachmentRef`；不变量 SubagentRun 不携带附件、实时从父 TaskRun 读，保持成立。
- 门禁全绿：fmt + clippy（`-D warnings`）+ 四条 panic lint + workspace 测试（app-core 608 / cli 83+9+5，全过，0 失败）。

**Phase 3（2026-08-04 已完成——grep artifact root + 会话清理）**：
- **grep artifact root 扩展**（框架层，`echo-agent` 提交 `8fd9b6a`）：`echo-tools/src/files/grep.rs` 的 confinement 改为候选根集合（base_dir + working_dir + `ctx.output_artifacts.root_dir`）。模型可用 `read_artifact`/spill 返回的绝对路径直接 grep spilled 内容。候选根与 resolved path 都先 canonicalize（照搬 read_artifact），防止 symlink/`..` 逃逸。不动 ToolContext/schema/pipeline。3 集成测试覆盖（artifact root 可搜 / 越界拒绝 / 相对路径仍走 working_dir）。
- **会话删除清理**：`delete_conversation` 现也删除 user-input artifact 的 per-conversation 子树（新增 `prepared_turn::cleanup_user_input_scope`，镜像已有的 `cleanup_tool_output_scope`）。
- **暂缓**：持久化去 data URL（`SavedAttachment.url` → `artifact_path`）。这是前端跨面改动——data URL 是前端自己的持久化契约（`conversationStore.ts` 写、`MessageBubble.tsx:241` 渲染 `img.url`）。移除需新增 Tauri command 按 path 读文件 + 前端异步渲染。不阻塞功能（只影响会话文件体积），作为独立任务跟踪。

**两个仓库门禁全绿**：echo-agent（660+ 测试，0 失败，clippy + 四条 panic lint + feature 隔离）；echo-agent-cli（app-core 608 / cli 83+9+5 / e2e 5，全过 0 失败）。

**验收标准**：粘贴十万行日志初始请求不含全文；模型可用 grep(artifact root)+read_artifact 搜索分页读；后续 ReAct 回合不重复携带；中文/emoji/超长单行 JSON 不 panic；重启可读、删会话清理；五入口 + steer + Subagent 同语义；诊断准确率不降、输入 tokens 显著下降。

---

## 八、已完成路线 5：Tauri / TUI / CLI / channel / cron 对等回归合同(P1)

五入口对等不是末尾补 UI，而是每个功能的 Definition of Done。

### 能力矩阵

持续审计以下能力：

- Chat/Task/Auto 路由。
- plan 创建、编辑、执行、暂停、恢复、取消。
- foreground/background/cron run。
- Subagent/Team 生命周期、结果、artifact 和错误。
- HITL approval/input/selection。
- memory、skills、MCP、Browser、附件和多模态。
- tool streaming、timeout、cancel、retry、artifact。
- usage/cache/protected context 诊断。

### 实施规则

- 新增 `AgentEvent`、`ExecEvent`、TaskRun 字段或 terminal 状态时，同一提交更新 GUI/TUI/CLI/channel reducer 或明确证明不适用。
- 建立共享事件 wire contract test；各入口只做渲染差异，不重建事实或自行推断业务终态。
- 交互入口消费同一 `ChatDriverEvent`，cron/background 消费持久化 `RuntimeTaskEvent`；测试比较生命周期、错误、identity 和 artifact 是否完整保留。
- TUI 不得以“终端不需要”为由省略能力；GUI 也不得维护只在前端存在的权威运行状态。

**验收**：能力矩阵无无理由缺口；同一事实经过交互事件或持久事件传输后不丢生命周期、错误、identity 和 artifact。

---

## 九、历史执行顺序与当前门禁

下表只记录已经完成的历史实施顺序，不再产生“下一步”。当前产品任务只以
[`2026-08-16-eko-long-horizon-runtime-implementation-plan.md`](../echo-agent-cli/docs/2026-08-16-eko-long-horizon-runtime-implementation-plan.md)
为执行入口；本轮跨层质量修复以
[`comprehensive-review/cross-quality-remediation.md`](comprehensive-review/cross-quality-remediation.md)
为证据账本。

| 阶段 | 内容 | 依赖 | 主要交付 |
|---|---|---|---|
| M1 | Agent 生命周期审计与契约 | 无 | 调用图、状态表、重复路径清单、contract test 矩阵 |
| M2 | trigger/run 统一与 terminal 收敛 | M1 | 统一适配层、删除平行终态写入 |
| M3 | pause/resume/cancel/crash conformance | M2 | 恢复矩阵、exactly-once 副作用测试 |
| M4 | 工具错误分类与恢复策略 | M2 | 通用错误合同、retry/idempotency 测试 |
| M5 | `run_code` sandbox 闭环 | M4 | 无 bare 生产路径、sandbox conformance tests |
| M6 | 超长工具日志 artifact | M4 | 有界会话投影、完整日志入口和清理策略 |
| M7 | Subagent 结构化结果与完成判定 | M3 | result contract、真实终态 |
| M8 | ownership/dependency 并行调度 | M7 | 安全并行、merge/失败收敛 |
| M9 | usage/cache/protected observability | 可与 M7 并行 | 单 run 上下文诊断视图 |
| M10 | 五入口差异总审计 | 贯穿 M1-M9 | 能力矩阵、共享事件与 wire contract tests |
| M11 | 专业域 Subagent 编排闭环 | M7、M10 | DomainProfile 传播、领域路由、复杂 Run 计划物化 |
| M12 | coding-first 统计推断正确性 | M5、M11 | 探索/推断分层、成熟库脚本、可复现 artifact |
| M13 | framework/application TaskRuntime 收敛 | M8、M11 | 唯一 runtime DAG executor、模型与 validator 归一 |

当前进度：

- **M1 已完成**：审计见 [2026-07-16-agent-lifecycle-audit.md](../echo-agent-cli/docs/2026-07-16-agent-lifecycle-audit.md)。
- **M2 已完成**：turn/TaskRun terminal 已分离，CLI 已进入 `drive_chat`，run cancel/pause/recovery 已收敛，background pipeline 已迁移到 TaskRuntime，模式合同记录 requested/observed path。
- M2 未新增主运行状态，未给 CLI 引入 SQLite，修改归属 `echo-agent-cli`；`echo-agent` 的通用 EventEnvelope、checkpoint、framework Task API 保持不动。
- **M3 已完成**：模型审批等待、Subagent、TaskRuntime Subagent/tool、GUI/TUI/CLI 的 pause/resume/cancel/crash 合同已收敛；已完成结果不重放，写副作用不确定时阻塞自动恢复并要求 retry/skip 决策。
- **M4 已完成**：shell/file/search/Browser/MCP 共用结构化失败与恢复合同；有限重试、幂等键、postcondition、trace/TaskRuntime 持久化和三端投影已接通。
- **M5 已完成**：`run_code` 无 bare 生产路径，最低隔离、sandbox availability、timeout/cancel/output-limit/cleanup/error 分类和 EKO capability gating 已收敛。
- **M6 已完成**：shell 与通用工具超长结果写完整 artifact，模型/会话/trace 只保留有界投影和引用；三端入口、缺失语义、hash、retention 与 conversation 清理已接通。
- **M7 已完成**：Subagent 统一结构化终态、observed/reported evidence、artifact 完整性、恢复复用和 task/run 真实完成门禁已接通。
- **M8 已完成**：task ownership/dependency 安全波次、隔离 writer worktree、Git merge 前置核验、幂等集成和 merge 失败收敛已接通。
- **M9 已完成**：provider usage、cache fingerprint、context breakdown、protected context 与 compression 已进入同一 durable run 诊断；GUI/TUI/CLI 共用 DTO/formatter，旧内存 collector、usage ledger 和趋势面板已删除。
- **M10 已完成**：Tauri/TUI/CLI/channel/cron 共享事件与持久事实合同已建立，HITL、mode、memory、附件、trace、scheduler 和 cron terminal 缺口已收敛。
- **M11 已完成**：DomainProfile 从 TaskRun 传播到 PlanTask 和执行提示；数据任务默认路由 `analyst`，复杂任务由独立主 Agent 物化正式 DAG 后再执行，`direct_execute` 保留轻量直达路径。
- **M12 已完成**：删除自研正式统计推断，框架只保留 `exploratory_statistics`；正式检验和建模由数据 Subagent 保存并执行 SciPy/statsmodels/R 脚本，记录输入哈希、版本、种子、缺失值规则、诊断和结果 artifact。
- **M13 Phase 1 已完成**：`echo-orchestration::RuntimeDagExecutor` 已成为 EKO 动态计划的唯一执行循环；revision safe point、ready frontier、Subagent wave、取消、失败传播、外部 in-flight 等待和 stall 判断已从应用层删除。EKO 只保留 review、worktree、文件持久化、事件与 attended/unattended policy adapter。专项记录见 [2026-07-27-runtime-dag-kernel-convergence.md](../echo-agent-cli/docs/2026-07-27-runtime-dag-kernel-convergence.md)。
- **M13 Phase 2 已完成**：框架 runtime 已原位拆成 immutable `RuntimeTaskSpec` + mutable `RuntimeTaskExecution`；required artifacts、execution checks、acceptance criteria 不再压平；EKO 专属字段只经 `EkoTaskMetadata` 注入。现有 framework `PlanValidator` 成为 revisioned runtime 结构校验唯一权威，EKO 重复的 dependency/DFS validator 已删除。
- **M13 Phase 3 已完成**：旧 framework `TaskExecutor::execute_all` 已通过私有 controller 复用 `RuntimeDagExecutor`，旧 ready/deadlock 主循环与 round-timeout 配置已删除；hooks/verifier/replanner/TaskStore/scheduler/per-task retry/timeout 保留为单任务 pipeline。authoring `PlanSpec`、`TaskManager` 拓扑查询与结构校验统一编译/投影到 canonical runtime specs，不再各自维护 Kahn/深度 validator。
- **M13 Phase 4 已完成**：框架公开模型最终归一为 immutable `TaskSpec` + mutable `TaskExecution` + shared `TaskStatus` + composed `Task`；rich hooks/verifier/store record 明确为 `ManagedTask`，authoring artifact 明确为 `PlanTaskSpec`。EKO 文件/UI DTO 明确为 `EkoTaskSpec/EkoTaskExecution`，通过 checked round-trip adapter 接入框架；不再存在第二套动态 PlanTask DAG executor、结构 validator、状态模型或同名 runtime model。最终唯一性审计又将 `TaskManager` cycle query 收归 `PlanValidator` 的 canonical dependency analysis，并删除 manager-local DFS/`VisitState`。后续只保留长运行观测与性能调优，不再保留架构迁移阶段。
- **M13 Phase 5 已完成**：EKO 对外任务关系 API 统一为 `task_create/task_update/task_list/task_execute`；单 Task、批量 Task 和依赖 DAG 使用同一个 revisioned TaskRun graph、store transaction、TaskStatus 和 executor。旧 `plan_create/plan_patch/plan_execute`、GUI/Tauri patch-plan API 与 EKO 可见的全局 `todo_write` 已删除；`TaskPlan` 仅是版本化 graph artifact，`TodoItem` 仅是 UI 投影，不再存在第二套 Todo/Plan 生命周期。

每个阶段必须：

1. 动手前 grep 框架与应用，确认现有定义、调用点和运行时可达性。
2. 先写专项 spec，记录业界依据、框架/应用归属、删除哪些旧路径。
3. 涉及生命周期、状态机、API 或编排时先调研成熟实现。
4. 新路径接通并覆盖测试后，删除被替代的旧路径，不保留双系统。
5. 同一阶段完成三端接入和验证，不留“以后补 TUI”。
6. 全量验证、按 `AGENTS.md` 的磁盘阈值决定是否 `cargo clean`，提交后更新本文状态。

---

## 十、第一阶段归档：M1 Agent 生命周期审计

> **归档边界**：本节起至“验证规范摘要”之前都是按日期保留的实施历史。
> 其中“下一步”“待做”“进行中”和旧 file:line 只描述当时状态，不是当前任务指令；当前状态以上文路线表和唯一执行入口为准。

M1 已于 2026-07-16 完成，没有修改核心状态机。完整审计、代码证据、P0/P1/P2 清单、M2 测试矩阵和提交拆分见 [2026-07-16-agent-lifecycle-audit.md](../echo-agent-cli/docs/2026-07-16-agent-lifecycle-audit.md)。

### 审计范围

- 入口：GUI `send_chat_message`、TUI send、CLI REPL、channel、cron、background。
- 驱动：`drive_chat`、`drive_run_async`、`launch_cron_run`、`execute_plan`、TaskRuntime executor。
- 身份：conversation_id、turn_id、run_id、task_id、execution_id、call_id、event_id。
- 状态：TaskRun、PlanTask、SubagentRun、tool execution、approval/HITL。
- 持久化：conversation files、runtime state files、run store、artifact/spill。
- 投影：Tauri events、TUI reducer、CLI/channel renderer、前端 Zustand store。

### 已交付

- `echo-agent-cli/docs/2026-07-16-agent-lifecycle-audit.md`。
- 当前调用图和状态转换表。
- 重复状态源/不可达路径/终态竞争清单，按 P0/P1/P2 排序。
- M2 的精确改动计划，包含删除项、测试矩阵和回滚点。

### M1 完成标准

- 不新增运行时字段或状态。
- 每个结论有实际构造点、调用点和持久化位置依据。
- 明确哪些通用缺口属于 `echo-agent`，哪些产品收敛属于 `echo-agent-cli`。
- M2 实现必须逐条回验本节审计结论，不扩张状态机。

### M1 核心发现

1. 六态 TaskRun 已经足够，问题主要来自外围生命周期并行，而不是状态数量不足。
2. GUI 普通 chat 会发射 synthetic run 并竞争 terminal；前端 chat reducer 还会覆写 TaskRuntime 状态。
3. CLI REPL 绕过 `drive_chat`，尚未达到 GUI/TUI/channel 的主流程对等。
4. `TaskRuntimeStore` 与 `AppState` 各持有一套 run cancel registry，部分路径只改状态不停止真实执行。
5. 启动恢复把 Running 直接改为 Failed，阻断已有的“重读 plan + 跳过 completed task”能力。
6. pipeline background task 仍使用 framework Task 产品路径，与 TaskRuntime 形成双状态源；cron 已经收敛到 TaskRuntime。
7. plan approval 借用 Paused 并依赖进程内 Notify，需要在 M2 改为 interaction/event 事实。

### M2 实施输入

1. 分离 turn 与 TaskRun terminal，删除 GUI synthetic run 和前端跨 store 状态写入。
2. CLI REPL 接入共享 `drive_chat`。
3. 统一 run driver/cancel registry，建立真实可续跑的 crash recovery。
4. 将 background pipeline 迁移到 TaskRuntime，删除 EKO 应用层双生命周期。
5. 校正 Chat/Task/Auto 的真实合同，记录 requested mode 与 observed path，不增加路由状态机。

---

## 十一、第二阶段归档：M2 trigger/run 统一与 terminal 收敛

M2 已于 2026-07-16 完成。完整的改造前证据、实施计划和完成归档见 [2026-07-16-agent-lifecycle-audit.md](../echo-agent-cli/docs/2026-07-16-agent-lifecycle-audit.md)。

### 已完成

- 普通 chat 只有 turn terminal，不再伪造 TaskRun；前端 chat status 不再覆写 TaskRuntime。
- GUI/TUI/CLI/channel 共用 `drive_chat`；Task mode 入口创建 formal run，Auto 记录 `requested_mode + observed_path`。
- `TaskRuntimeStore` 成为唯一 run cancellation registry；pause 停止真实 driver，resume 只做执行恢复。
- interrupted Running run 恢复为 Paused；completed todo 保留，orphaned Running todo 回到 Pending。
- background、cron、research/data/writing pipeline 统一使用 TaskRun；删除 EKO 应用层 framework Task 双生命周期、旧 pipeline graph 和旧 TaskStore 装配。
- 删除 plan approval Notify、展示专用 ExecutionPolicy、旧 route/classifier DTO、无消费者 background HITL bus 和死 create/execute TaskRun IPC。
- 后台 TaskRun 保留并发上限、依赖等待、trigger metadata 与准确 prompt 恢复；用户主动暂停不会被启动恢复自动续跑。

### 验证结果

- `cargo check --workspace` 通过。
- `cargo test --workspace` 通过：app-core 476、runtime e2e 5、CLI lib 41、CLI main 9，零失败。
- GUI target check 与 GUI tests 通过；GUI tests 43，零失败。
- `channels`、`tui+telemetry`、`gui+devtools` feature 组合通过。
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` 通过。
- 前端 `npx tsc -b`、46 个 Vitest、`npm run build`、Prettier check 全部通过。

### 下一步：M4

M4 不再改主状态机，专注工具失败合同：统一参数错误、不可用、超时、取消、瞬态、永久和部分副作用分类；把有限重试、幂等键、postcondition 与简短跨 run 诊断建立在 M3 的 Subagent/tool durable boundary 上。

---

## 十二、第三阶段归档：M3 pause/resume/cancel/crash conformance

M3 已于 2026-07-16 完成。完整实现依据和合同见 [2026-07-16-agent-lifecycle-audit.md](../echo-agent-cli/docs/2026-07-16-agent-lifecycle-audit.md)。

### 已完成

- `echo-agent` 的 permission/HITL 等待可被 invocation cancel 立即打断；Subagent result 显式携带 `cancelled`，不再把取消文本当成功结果。
- Subagent tool start/completed 事件保留稳定 `call_id`，Tauri 执行事件同步透传。
- TaskRuntime 在 Subagent 与主 Agent tool 的开始/终止边界写入持久事件；事件只保存有界摘要，不把原始参数和长结果复制进会话。
- 进程重启后，已完成 Subagent 结果直接进入 review，不再次 dispatch；只读/可重放的未完成工作回到 Pending。
- 写入、执行、网络或敏感工具处于“已开始、无终态”时，Todo 进入 Blocked，run 保持 Paused；background 自动恢复和三端 resume 都会拒绝继续。
- 用户检查工作区后可选择 retry 或 skip；该决策写入 append-only event，再允许 resume。
- GUI 提供暂停、取消、恢复阻塞、retry/skip；TUI 提供 `/task-recovery`、`/task-retry`、`/task-skip`；CLI `/tasks` 提供 pause/resume/recovery/retry/skip。
- GUI task pause/cancel 会清理对应 pending HITL；TUI approval future 被取消时只清理同一 request，避免覆盖更新的审批请求。

### 恢复合同

| 中断事实 | 恢复行为 |
|---|---|
| Subagent 已有 completed terminal | 复用 durable summary，继续 review，不重新 dispatch |
| 只读 Subagent/tool 只有 start | 回到 Pending，允许安全重放 |
| 写入/执行/网络 Subagent/tool 只有 start | Blocked，禁止自动 resume |
| 用户选择 retry | Todo 回 Pending，记录 RecoveryResolved，再由正常 resume 驱动 |
| 用户选择 skip | Todo 进入 Skipped，记录 RecoveryResolved，保留当前工作区 |
| pause/cancel 等待 HITL | 立即拒绝/清理 pending approval，不等待超时 |

### 阶段结论

M3 没有新增 TaskRun 主状态，也没有承诺通用 exactly-once。EKO 对可识别的 completed fact 做 exactly-once reuse；对安全操作提供可重放恢复；对无法证明是否已产生副作用的操作采用人工决策屏障。这比盲目自动重试更符合本地个人助理的真实数据安全边界。

### 验证结果

- `echo-agent ./scripts/verify-all-crates.sh` 通过：8 个 crate 逐项测试、clippy 零警告、独立 feature 矩阵全绿。
- `echo-agent-cli cargo test --workspace` 通过：app-core 483、runtime e2e 5、CLI lib 42、CLI main 9，零失败。
- GUI target check 与 GUI tests 通过；GUI tests 43，零失败。
- `channels`、`tui+telemetry`、`gui+devtools` feature 组合通过。
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` 通过。
- 前端 `npx tsc -b`、49 个 Vitest、`npm run build`、Prettier check 全部通过。

---

## 十三、第四阶段归档：M4 工具错误分类与恢复策略

M4 已于 2026-07-16 完成。专项设计与业界依据见 [2026-07-16-tool-error-recovery.md](../echo-agent-cli/docs/2026-07-16-tool-error-recovery.md)。

阶段提交：`echo-agent@55d7a25`、`echo-agent-cli@c83a04d`。

### 已完成

- `echo-agent` 新增稳定的七类工具失败：`invalid_arguments`、`unavailable`、`timeout`、`cancelled`、`transient`、`permanent`、`partial_side_effect`。
- 恢复合同统一携带 `recovery`、`side_effect`、`retry_after_ms`、`idempotency_key` 和 `postcondition`；未知旧错误保守按 permanent/stop 处理。
- ToolManager 只自动重试显式 `retry` 且副作用可证明安全的失败；使用有界指数 backoff+jitter；流式输出出现后不重试。
- 删除 `web_search` 私有重试循环，失败与重试次数统一进入中央执行路径和 trace，不再隐藏内部失败。
- 修复 unsuccessful `ToolResult` 被 `execute_tool_with_policy` 投影成成功事件的问题；最终 AgentEvent、trace 与 UI 终态保持一致。
- shell 非零退出与 timeout 标记可能副作用和核验条件；timeout 与 cancel 保持独立。
- `read_file` 返回 SHA-256 `content_hash`；`write_file` 支持 `expected_hash` 乐观并发校验，并记录稳定 call-derived idempotency key/postcondition。
- MCP 保留 JSON-RPC code，区分协议/参数错误与 `isError` 工具执行错误；按 MCP read-only/destructive annotation 决定副作用风险。
- Browser 只对 snapshot/diagnostic 等只读调用做重连重试；click/fill/navigation 等连接中断后进入 partial-side-effect + verify，不盲重放。
- TaskRuntime 的 tool terminal boundary 持久化完整失败合同；GUI/TUI/CLI 显示相同 category/recovery，前端保留 postcondition。
- 跨 run 简短诊断复用已有 `TraceAnalyzer.tool_reliability_report` 和 evolution dashboard，没有新增数据库、EvalRunner 或后台指标平台。

### 阶段结论

M4 没有新增主状态机，也没有把 EKO 产品逻辑塞进通用框架。框架只提供通用工具失败原语和执行策略；Browser、TaskRuntime 与 UI 投影留在应用层。恢复语义从“看到错误就重试”收敛为“结构化分类 -> 判断副作用 -> 有界重试或先核验”，降低重复写入、重复提交和假成功的概率。

### 验证结果

- `echo-agent ./scripts/verify-all-crates.sh` 通过：echo_core 264、echo_execution 224、echo_integration 64、echo_tools 92、echo_state 136、echo_orchestration 274、echo_agent 486；逐 crate clippy 和独立 feature 矩阵全绿。
- `echo-agent-cli cargo test --workspace` 通过：app-core 485、runtime e2e 5、CLI lib 42、CLI main 9，零失败。
- GUI target check 与 GUI tests 通过；GUI/channel lib tests 44，零失败。
- `channels`、`tui+telemetry`、`gui+devtools` feature 组合通过。
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` 通过。
- 前端 `npx tsc -b`、50 个 Vitest、`npm run build`、Prettier check 全部通过。

### 下一步：M5

M5 聚焦 `run_code` 真沙箱闭环：盘点所有构造路径，删除生产 bare fallback；统一 local/Docker/K8s 的 timeout、cancel、output-limit 与错误分类；建立 sandbox conformance tests。交互式 terminal 仍是用户主动能力，不受 agent `permission_mode` 门控。

---

## 十四、第五阶段归档：M5 `run_code` Sandbox 闭环

M5 已于 2026-07-16 完成。专项设计、业界依据、框架/应用归属和验收矩阵见 [2026-07-16-run-code-sandbox.md](../echo-agent-cli/docs/2026-07-16-run-code-sandbox.md)。

阶段提交：`echo-agent@2acb75b`、`echo-agent-cli@fa1f814`。

### 已完成

- `RunCodeTool` 删除 `tokio::process` bare fallback；无 sandbox、进程级 executor 或 backend 不可用时统一返回结构化 `unavailable`。
- `SandboxCommand` 新增调用方最低隔离要求；即使 manager 使用 trusted policy，`run_code` 也不能降到 `os-sandbox` 以下。
- 通用 sandbox 合同接入 owning-run cancellation；cancel 与 timeout 保持独立，取消不自动重试，并保留可能副作用的核验条件。
- `ExecutionResult` 增加 cancelled 事实；stdout/stderr 共享一个 UTF-8 安全字节预算，并保留截断前总字节数。
- `SandboxManager` 选择 executor 前检查实际 availability；无满足要求的可用 backend 时 fail closed。
- Docker 普通/limits 执行归一到同一路径，timeout/cancel 后按具体 container 清理；Kubernetes 按具体 Pod 清理；local 取消会关闭流并终止进程组。
- EKO 主 Agent 在启动时探测本机 OS sandbox；不可用时只移除 `run_code`，不阻塞应用启动，也不影响交互式 terminal/MCP。
- Writer Subagent 继承主 Agent 的 `SandboxManager` 和 capability probe；Readonly Subagent 继续使用不含 `run_code` 的只读工具集。
- 为消除当前 Rust 的 future-incompat warning，将框架可选 `rusqlite` 升到 0.32.1、SQLx 锁定到 0.8.6；EKO 仍未启用 `echo-state/sqlite`。

### 阶段结论

M5 把“Agent 生成任意代码”收敛成框架可复用的 fail-closed 执行合同，同时把“本机是否提供该产品能力”的决定留在 EKO 应用层。没有新增权限状态机，也没有用 agent permission mode 限制用户主动终端。缺少沙箱时，产品明确降级为没有 `run_code`，而不是静默在宿主机执行。

### 验证结果

- `echo-agent ./scripts/verify-all-crates.sh` 通过：echo_core 267、echo_execution 227、echo_integration 64、echo_tools 96、echo_state 136、echo_orchestration 274、echo_agent 486；逐 crate clippy 和独立 feature 矩阵全绿。
- `echo-agent-cli cargo test --workspace` 通过：app-core 487、runtime e2e 5、CLI lib 42、CLI main 9，零失败。
- GUI target check 与 GUI tests 通过；GUI/channel lib tests 44，零失败。
- 默认 workspace 与 GUI 两套 clippy 均使用 `-D warnings` 通过。
- 前端 `npx tsc -b`、50 个 Vitest、`npm run build`、Prettier check 全部通过。
- 两个 Rust 仓库已执行 `cargo clean`，释放约 42.8 GiB 编译产物。

### 下一步：M6

M6 聚焦超长工具日志 artifact：完整 stdout/stderr/tool payload 超阈值后写独立文件，对话、事件与会话只保留有界摘要、路径、大小、hash 和保留策略；GUI/TUI/CLI 共享同一 artifact 引用和缺失状态，并补按 run/session 清理。

---

## 十五、第六阶段归档：M6 超长工具日志 Artifact

M6 已于 2026-07-16 完成。专项设计、业界依据、框架/应用归属和验收矩阵见 [2026-07-16-tool-output-artifacts.md](../echo-agent-cli/docs/2026-07-16-tool-output-artifacts.md)。

阶段提交：`echo-agent@b5b2e2e`、`echo-agent-cli@f7708fa`。

### 已完成

- 新增通用流式 artifact writer；shell 不再先截断后 spill，stdout/stderr 超阈值后边执行边完整落盘。
- 非流式超长工具结果继续经统一 truncation stage 写 artifact；模型只接收短预览、路径、大小和 hash。
- ToolResult metadata 与 RunEvent 统一记录 artifact path、bytes、SHA-256、retention 和 handling；conversation 最终投影继续保持 128 KiB/1000 行上限。
- EKO 使用稳定全局 artifact 根，避免 worktree/workspace 删除导致日志失效；conversation 删除级联清理，遗留 scope 30 天兜底。
- GUI 增加完整日志打开入口和缺失提示；TUI 增加 artifact 状态与 `/open-artifact`；CLI terminal 与 `/trace` 输出同一引用。
- artifact 缺失或写入失败不改变原工具终态，避免“日志文件没了”被误判为工具执行失败。
- TaskRuntime `ArtifactProduced` 事件开始持久化 path/metadata，文件权威重建后不再丢 artifact 引用。

### 阶段结论

M6 将完整原始日志与模型/UI 有界视图分离：框架提供通用写入、hash 和清理原语，EKO 只决定目录、retention 和三端交互。没有新增数据库、状态机或平行会话源。10.5MB shell 回归证明完整 artifact 可恢复，而最终 ToolResult 仍限制在 1 MiB，conversation 投影继续有界。

### 验证结果

- 专项回归已通过：10.5MB shell 完整 artifact、非流式 spill/read-back、artifact scope 清理、TaskRuntime path/metadata 重建、TUI 缺失语义和前端有界投影。
- `echo-agent ./scripts/verify-all-crates.sh` 通过：echo_core 269、echo_macros 0、echo_execution 227、echo_integration 64、echo_tools 97、echo_state 136、echo_orchestration 274、echo_agent 486；逐 crate clippy 与独立 feature 矩阵全绿。
- `echo-agent-cli cargo test --workspace` 通过：app-core 488、runtime e2e 5、CLI lib 43、CLI main 9，零失败。
- GUI 44 个测试通过；`channels`、`tui,telemetry`、`gui,devtools` 独立 feature 编译通过；workspace 全 target/all feature clippy 零警告。
- 前端 `npx tsc -b`、52 个 Vitest、`npm run build`、Prettier check 全部通过。
- 两个 Rust 仓库及所有独立子 crate `target` 已执行 `cargo clean`，释放约 65.4 GiB 编译产物。

### 下一步：M7

M7 聚焦 Subagent 结构化结果与真实完成判定：统一 status/summary/artifacts/verification/remaining_work/touched_files，失败和超时可恢复或合理降级，父任务只有在必需产物、验证与 unresolved failures 一致时才能 completed。

---

## 十六、第七阶段归档：M7 Subagent 结构化结果与真实完成判定

M7 已于 2026-07-17 完成。专项设计、业界依据、框架/应用归属和验收矩阵见 [2026-07-17-subagent-results-and-completion.md](../echo-agent-cli/docs/2026-07-17-subagent-results-and-completion.md)。

阶段提交：`echo-agent@954004c`、`echo-agent-cli@a6fc02b`。

### 已完成

- 框架新增有界、UTF-8 安全的 `SubagentOutcome`：统一 `status/summary/artifacts/verification/remaining_work/touched_files`，runtime 覆盖模型自报 status 与 evidence source。
- Sync/Fork/Teammate/Team 统一注入 version 1 result contract；同步 `agent_tool` 返回完整 JSON result，terminal event 携带同一结构。
- `failed/cancelled/timed_out/completed` 终态互斥；retry/delegate 的中间失败不再提前发 terminal，timeout 使用 typed `AgentError::Timeout`。
- observed verification 只来自真实工具事件，同一检查由最新 observed 事实覆盖；模型报告一律是 reported，必需验证使用规范化精确匹配。
- artifact 解析实际文件并补 bytes/SHA-256/producer execution id/availability；result 条目数、路径、details 与 remaining work 全部有界。
- EKO `PlanTask` 新增 `required_artifacts`，`TaskExecutionSummary` 与 `SubagentReleased` 持久化权威 `SubagentTaskResult`；重启恢复复用结果后重新执行同一 completion gate。
- task 只有在 status、summary、remaining work、observed verification、artifact 完整性和 review 全部一致时才进入 Completed；run 还要求无未完成任务与 recovery blocker。
- 删除模型可直接调用的 `task_complete` 绕过路径；unattended 流结束不再无条件完成，writer dispatch 失败也不再回退到可能重复副作用的主 agent。
- GUI/TUI/CLI 统一消费 structured terminal result；三端展示 timed_out、summary、artifact、verification、remaining work 和 touched files，不从自然语言推断完成。

### 阶段结论

M7 将“Subagent 执行结束”和“父任务需求满足”拆成两个事实：框架负责可持久化终态与观察证据，EKO 应用层负责产品级完成门禁。没有新增审批状态机、数据库或权限门控；失败与超时保留结构化剩余工作，已完整结束的 Subagent 可恢复复用但不能绕过验收。

### 验证结果

- `echo-agent ./scripts/verify-all-crates.sh` 全绿：默认 workspace `1553` 个测试、all-targets/all-features `1778` 个测试，独立 feature 矩阵、最小 feature、默认/all-features Clippy 和生产 panic API 门禁全部通过。
- `echo-agent-cli cargo test --workspace` 全绿：app-core `491`、runtime e2e `5`、CLI lib `43`、CLI main `9`；GUI feature `44` 个测试通过。
- `channels`、`tui,telemetry`、GUI target 与 `gui,devtools` 编译通过；workspace 默认/all-features Clippy 均使用 `-D warnings` 通过。
- 前端 `npx tsc -b`、`53` 个 Vitest、`npm run build`、Prettier check 全部通过。
- 两个 Rust workspace 已执行 `cargo clean`，释放约 `65.1 GiB` 构建产物。

### 下一步：M8

M8 聚焦 ownership/dependency 并行调度：明确 task/file ownership，安全并行只读与隔离写任务，收敛 merge、依赖失败、部分成功和多 Subagent 冲突，不再依赖隐式调度顺序保证正确性。

---

## 十七、第八阶段归档：M8 ownership/dependency 并行调度

M8 已于 2026-07-17 完成。专项设计、业界依据、框架/应用归属和验收矩阵见 [2026-07-17-ownership-dependency-scheduling.md](../echo-agent-cli/docs/2026-07-17-ownership-dependency-scheduling.md)。

阶段提交：`echo-agent-cli@420f2ee`；`echo-agent` 无代码改动。

### 已完成

- `PlanTask.files` 收敛为 workspace-relative 精确文件 ownership；绝对路径、父目录穿越、glob、空值和目录式声明统一视为 unknown writer ownership。
- 调度器按依赖就绪任务选择 ownership-safe wave：只读任务可并行，精确且互不重叠的 writer 可并行，重叠或 unknown writer 串行，不再依赖全局 writer semaphore 偶然保证正确性。
- writer 在固定 base commit 的独立 Git worktree 执行；分支名使用稳定安全 hash，避免 task/execution identity 中的空格或标点生成非法 ref。
- Subagent 结构化结果和 review 通过后进入显式 integration 阶段；merge 成功才允许 task completed，merge 失败写入真实 failed terminal 并阻塞依赖任务。
- 集成前使用实际 Git diff 核验 ownership，拒绝越界写入、主工作区同路径脏改动、用户 staged 变更和仓库中已有 merge/rebase/cherry-pick 操作。
- 使用 `git merge-tree` 做无副作用冲突预检，再执行 `--no-ff` merge；只清理自身发起的失败 merge，冲突 worktree 保留并解锁供用户检查。
- merge commit 写入稳定 `EKO-Execution-Id` trailer，重复恢复可识别 already-integrated；无变更与已集成均为幂等成功。
- `merge_started/completed/failed` 进入统一 Subagent run 事件投影；GUI 在 Subagent 完成后仍显示 integration 进行中，并准确展示 merge failure。
- Running task 禁止修改 files、depends_on、kind、tools 等执行合同；计划更新涉及依赖或 ownership 时重新执行图与冲突校验。

### 阶段结论

M8 没有修改通用 `echo-agent` 框架。文件所有权、worktree 生命周期、主工作区脏状态策略和 GUI merge 投影都属于 EKO 编码助理的产品约束，因此保留在 `echo-agent-cli` 应用层。安全并行由显式 ownership 和 Git 隔离证明，完成状态由真实 merge 结果决定，不新增审批状态机或数据库。

### 验证结果

- worktree 14 个真实 Git 回归通过，覆盖并行 disjoint merge、ownership 越界、冲突预检、主工作区脏改动、用户 staged 变更、固定 base 和重复集成幂等。
- `cargo test --workspace` 全绿：app-core `507`、runtime e2e `5`、CLI lib `43`、CLI main `9`；GUI feature `44` 个测试通过。
- 默认 workspace、all-targets/all-features、`channels`、`tui,telemetry`、GUI target 与 `gui,devtools` feature 矩阵通过。
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` 通过。
- 前端 `npx tsc -b`、`54` 个 Vitest、`npm run build`、Prettier check 全部通过。
- `echo-agent-cli` 已执行 `cargo clean`，释放约 `39.7 GiB` 构建产物。

### 下一步：M9

M9 聚焦 usage/cache/protected-context observability：以 provider/API usage 为记账权威，补齐单 run 的 prompt/cache/compression/protected-context 诊断；本地估算只用于预算预警和 usage 缺失观察，不建设后台指标平台。

---

## 十八、第九阶段归档：M9 单 Run Usage/Cache/Protected Context 可观测性

M9 已于 2026-07-17 完成。专项设计、业界依据、框架/应用归属和验收矩阵见 [2026-07-17-run-context-observability.md](../echo-agent-cli/docs/2026-07-17-run-context-observability.md)。

阶段提交：`echo-agent@c9f5487`、`echo-agent-cli@b537610`。

### 已完成

- 每次 Agent invocation 使用唯一 `trace_run_id`；TaskRuntime `run_id` 只作为 parent correlation，工具继续读取产品 run identity，避免并行 Subagent 写同一 trace 记录。
- framework `Run` 持久化 agent/model/provider/turn/execution metadata；`RunEvent::LlmCall` 持久化 provider usage、cache read/write、context limit、角色/来源 breakdown、protected count/tokens 与 canonical SHA-256 cache fingerprint。
- OpenAI-compatible usage 补齐 `cache_write_tokens`；Anthropic/DeepSeek 既有 cache 语义继续归一，provider 未返回 usage 时只展示 estimate，不进入准确 totals。
- auto/manual compression 统一写入 durable `ContextCompression` timeline，记录压缩前后 message/token 与 protected context。
- EKO 以 framework `RunStore` 为唯一诊断事实，按 parent run 聚合 main/Subagent traces；删除 app 内存 `TraceCollector`、TaskRuntime usage ledger/aggregation、重复的 Subagent usage DTO 和 Tauri API。
- GUI 改为 durable run list + 单 run inspector，展示 provider totals、per-call source、cache component 变化、context breakdown、protected warning、compression 和 prompt modules；窄屏改为上下布局。
- CLI 与 TUI 新增同语义 `/trace [run-id]`，共用应用层 DTO 和 formatter；无 id 时选择最近 durable run。
- 删除 `UsageTrendsPanel`、旧 trace timeline/step inspector/token chart/cache panel 及对应生成类型，不建设后台指标平台或 SQLite usage 路径。
- 提交门禁额外清理生产代码中的 panic 路径：锁中毒恢复、HOME/工作区 fallback、spinner 模板、UTF-8 快捷键解析和 Tauri 启动错误均改为显式错误或安全降级。

### 阶段结论

M9 保持“框架记录通用事实、应用生成产品诊断”的边界：`echo-agent` 只提供 provider usage/cache/context/compression 与 trace correlation，EKO 只负责聚合、阈值说明和三端渲染。准确 usage 不与本地估算混算；业务 run 与 trace invocation 不再复用 identity；没有新增数据库、后台评分、自动建议或权限状态机。

### 验证结果

- `echo-agent ./scripts/verify-all-crates.sh` 全绿：默认 workspace `1557` 个测试、all-targets/all-features `1782` 个测试；独立 feature、最小 feature、默认/all-features Clippy 和生产 panic API 门禁全部通过。
- `echo-agent-cli` 默认 workspace 全绿：app-core `501`、runtime e2e `5`、CLI lib `44`、CLI main `9`；all-features CLI lib `80`，GUI feature CLI lib `44`，全部零失败。
- CLI 默认/all-features Clippy 使用 `-D warnings` 通过；production `unwrap/expect/panic`、app-core/非 GUI `unreachable` 门禁通过；`tui`、`channels`、`tui,telemetry`、GUI target 与 `gui,devtools` feature 组合通过。
- 前端 `54` 个 Vitest、`npm run build`、Prettier check 全部通过；桌面与窄屏浏览器检查均无水平溢出、无面板重叠。
- 两个 Rust workspace 已执行 `cargo clean`，分别释放约 `23.8 GiB` 与 `51.4 GiB`，合计约 `75.2 GiB`。

### 下一步：M10

M10 聚焦五入口差异总审计：按 Chat/Task/Auto、run 生命周期、Subagent/HITL、memory/skills/MCP/Browser/附件、tool 错误与 artifact、usage/context 等能力建立矩阵；Tauri/TUI/CLI/channel 消费同一交互事件，cron/background 保留同一持久事实，删除剩余单入口权威状态和无理由功能缺口。

---

## 十九、工程基础归档：echo-agent 根 Package + Workspace

### 决策与依据

- 参考 [Cargo Workspaces 官方文档](https://doc.rust-lang.org/cargo/reference/workspaces.html)：workspace 统一成员命令、`Cargo.lock` 和输出目录；workspace root 可以同时保留 `[package]`，`default-members` 可定义根目录普通命令的默认覆盖范围。
- 保留根 `echo_agent` package，不改成 virtual workspace；新增 7 个显式成员、8 个显式 `default-members`，使用 Edition 2024 对应的 `resolver = "3"`。
- 不把三个独立 git 仓库合成顶层 monorepo，也不为迁移而批量改写 `[workspace.dependencies]`。这次只统一 `echo-agent` 框架内部的构建、测试和依赖解析边界。
- 子 crate 仍是可独立发布、可独立复用的 package；workspace 只解决工程编排，不改变框架与 EKO 应用的职责边界。

### 完成内容

- 根 `Cargo.toml` 变为“根 package + workspace”，8 个成员共享唯一根 lockfile 与根 `target`；清理遗留成员 lockfile，并保留忽略规则防止再次生成后误提交。
- `scripts/verify-all-crates.sh` 改为 workspace 默认/all-features 全矩阵门禁：check、test、Clippy、12 个根 feature、最小 feature 和生产目标 panic API 检查。
- 修复 `.github/workflows/rust-ci.yml` 的失效缩进，CI 与本地脚本使用同一 workspace 语义；用 production-target Clippy 替代会误扫内联测试模块的文本 grep。
- 统一门禁首次覆盖此前未被根命令真正检查的 7 个子 crate，并修复暴露出的 Clippy、feature、测试契约和错误传播问题。没有给 EKO 引入 SQLite；框架自身的可选 SQLite 能力继续保留。

### 验证与提交

- `echo-agent ./scripts/verify-all-crates.sh` 全绿：默认 workspace `1553` 个测试、all-targets/all-features `1769` 个测试，12 个独立 feature、no-default-features、默认/all-features Clippy 和生产 panic API 门禁全部通过。
- `echo-agent-cli cargo check --workspace`、`cargo test --workspace` 通过：app-core 488、runtime e2e 5、CLI lib 43、CLI main 9；GUI target 编译及 44 个 GUI feature 测试通过；全 target/all feature Clippy 零警告。
- 两个 Rust 仓库执行 `cargo clean`，释放约 57.9 GiB 构建产物。
- 阶段提交：`echo-agent@1a60117` (`chore(workspace): unify framework crates`)；`echo-agent-cli` 无代码改动，不产生空提交。

---

## 二十、第十阶段归档：M10 五入口功能对等收尾

M10 已于 2026-07-17 完成。专项设计、业界依据、框架/应用归属和验收合同见 [2026-07-17-surface-parity-closeout.md](../echo-agent-cli/docs/2026-07-17-surface-parity-closeout.md)。

阶段提交：`echo-agent-cli@30c28d7`、`f00e0e0`、`c650f00`、`a176760`、`e443fba`；`echo-agent` 无代码改动。

### 已完成

- `ChatSink` 收敛为单一、穷尽的 `ChatDriverEvent` 入口；`Agent(EventEnvelope)`、`Execution(ExecEvent)`、turn status、requested/observed path 与 interrupt 不再通过默认 no-op 静默丢失。
- TaskRuntime task-local trace 与 framework external trace 都桥接到同一交互事件；Tauri/TUI/CLI/channel 各自只负责渲染，cron/background 继续以 append-only `RuntimeTaskEvent` 为权威。
- Tauri 补齐 budget/guard/memory/safety/parameter/chart 等此前丢弃事件的 notice 投影，前端 reducer 展示 notice 与执行路径差异；未知未来事件降级为信息提示。
- TUI 的 Approval/Input/Selection 全部等待真实用户响应；新增真实 scheduler `/cron list/create/delete/pause/resume/run/reload`，不再把 cron 命令当自然语言交给 Agent。
- CLI `/mode chat|task|auto`、`/plan`、`/remember`、`/forget` 与 `/attach` 使用真实运行时、分层记忆和多模态引用，不再只打印成功或固定 Auto。
- channel 的 streaming/non-streaming 路径统一进入 `drive_chat`；每 sender session 支持 mode、HITL next-message、`/trace`、分层记忆和完整 TaskRuntime 事件文本降级。
- channel 附件先持久化为统一 `AttachmentRef`，主 Agent 与 TaskRuntime Subagent 从同一引用重建多模态消息，不再只有主 Agent 可见。
- cron stream setup/terminal error 会持久化 Failed；`launch_cron_run` 只在真实 Completed 时返回成功，Paused/Cancelled/Failed/非终态均返回错误。
- 测试专用能力矩阵覆盖五入口；共享交互事件与 cron 持久事件 wire contract 验证 lifecycle、error、identity 和 artifact 不丢失。

### 阶段结论

M10 没有给 `echo-agent` 增加 EKO 产品概念。框架既有 `EventEnvelope`、HumanLoop、tool failure 与 trace 原语保持通用；五入口 mode、scheduler、TaskRuntime、渲染和能力证据全部留在 `echo-agent-cli` 应用层。没有新增状态机、数据库或权限门控。

### 验证结果

- `cargo test --workspace --all-features` 全绿：app-core `509`、runtime e2e `5`、CLI lib `79`、CLI main `9`，GUI/Tauri 条件编译测试同步覆盖。
- 默认 workspace、`channels`、GUI target/GUI tests、`tui,telemetry`、`gui,devtools` 与 all-features feature 矩阵通过；channel 专属 `62` 个测试通过。
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`、`cargo fmt --all -- --check`、production 新增 panic/UTF-8 审查通过。
- 前端 `npx tsc -b`、`56` 个 Vitest、`npm run build`、Prettier check 全部通过；Tauri 桌面首屏浏览器检查无 JS 控制台错误、无水平溢出或异常滚动（未启动后端时仅出现预期 API 500 提示）。
- `echo-agent-cli cargo clean` 释放约 `56.6 GiB` 构建产物。

### 下一步：M11

M11 聚焦专业域 Subagent 编排：贯通 DomainProfile、领域默认路由和 complex Run 的正式计划物化，同时保持 plan artifact 与运行状态机分离。

---

## 二十一、第十一阶段归档：M11 专业域 Subagent 编排闭环

M11 已于 2026-07-18 完成。专项设计、业界依据、框架/应用归属和验收合同见 [2026-07-17-domain-subagent-orchestration.md](../echo-agent-cli/docs/2026-07-17-domain-subagent-orchestration.md)。

阶段提交：`echo-agent@1afc37c`、`echo-agent-cli@0435c45`。

### 已完成

- 本次涉及的 `echo-agent` 通用任务原语统一使用 `TaskSubagent`、`TaskSubagentContext`、`max_concurrent_subagents` 和 `subagent_name`，不再暴露第二套执行角色概念。
- `task_create` 从权威 `TaskRun.domain_profile` 写入每个 `PlanTask`，允许显式选择内置或自定义 Subagent；未指定时使用专业域默认路由。
- 内置目录覆盖 explorer、reviewer、planner、summarizer、implementer、general-purpose、data-shaper、analyst；数据分析的 implementation/debugging 默认交给 `analyst`，verification 仍由主 Agent 在当前工作区执行。
- `create_complex_task` 改为独立主 Agent ReAct 驱动：`plan_then_execute` 必须调用 `task_create` 物化非空 DAG 并通过 `task_execute` 执行，prose-only plan 无法越过完成门禁；`direct_execute` 可在轻量场景直接完成。
- 专业域规划方法、执行标准、Subagent 目录和初始分解进入独立 Run prompt；具体任务执行继续注入 DomainProfile execution guidance，形成规划、路由、执行、review 同域闭环。
- TaskRuntime 持久事件与前端合同统一为 `subagent_assigned/subagent_released`，`TaskExecutionSummary.subagent_name` 成为权威字段；本次触及的应用与框架运行时代码已无旧执行角色命名。

### 阶段结论

M11 没有新增 TaskRun 状态，也没有把 EKO 的专业域策略塞进通用框架。框架只保留通用 Subagent 执行合同；DomainProfile、领域 prompt、默认角色路由、complex/direct 产品策略全部留在 `echo-agent-cli`。计划批准仍由 prompt/交互驱动，运行时只验证正式计划是否真实存在与任务是否满足完成门禁。

### 验证结果

- `echo-agent ./scripts/verify-all-crates.sh` 全绿：默认 workspace `1557` 个测试、all-targets/all-features `1782` 个测试；独立 feature、最小 feature、默认/all-features Clippy 和生产 panic API 门禁全部通过。
- `echo-agent-cli` 默认 workspace 全绿：app-core `510`、runtime e2e `5`、CLI lib `44`、CLI main `9`；all-features 为 app-core `510`、runtime e2e `5`、CLI lib `79`、CLI main `9`。
- GUI target 编译与 GUI feature `43` 个测试通过；`channels`、`tui,telemetry`、`gui,devtools` 组合和 workspace all-features Clippy `-D warnings` 全部通过。
- 前端 Prettier、`npx tsc -b`、`npm run build`、`56` 个 Vitest 全部通过。
- 两个 Rust workspace 已执行 `cargo clean`，分别释放约 `26.0 GiB` 与 `57.3 GiB`，合计约 `83.3 GiB`。

### 下一步：M12

M12 聚焦统计推断正确性：保留轻量探索性统计，删除自研正式推断近似；数据 Subagent 通过成熟统计库生成、执行并保存可审阅脚本和可复现 artifact。

---

## 二十二、第十二阶段归档：M12 coding-first 统计推断正确性

M12 已于 2026-07-18 完成。专项设计、业界依据、框架/应用归属和验收合同见 [2026-07-18-statistical-inference-correctness.md](../echo-agent-cli/docs/2026-07-18-statistical-inference-correctness.md)。

阶段提交：`echo-agent@5a2c8a8`、`echo-agent-cli@0894fca`、`echo-website@1d03fbe`。

### 已完成

- 删除框架旧 `hypothesis_test`、`regression` 和 `descriptive_advanced` 正式推断路径，不保留自研近似与成熟库两套实现。
- `statistics` feature 只注册 `exploratory_statistics`，输出有限数值计数、均值、样本标准差、四分位数、矩偏度和超额峰度，并明确 `inference=false`。
- 数据 `analyst` 与 `DomainProfile::DataAnalysis` 合同要求：正式检验、回归和模型先写入任务 working directory 的 `.py`/`.R` 文件，再通过既有沙箱化 `run_code` 执行同一脚本。
- 正式分析 artifact 必须记录输入路径与 SHA-256、包版本、随机种子、分析参数、缺失值处理、诊断、告警和结果路径；禁止回退到手写 p 值近似。
- 框架中英文文档、EKO 专项 spec 和官网工具说明统一为 coding-first 模型；官网补齐 ESLint 9 flat config，使既有 lint 门禁恢复可执行。
- 本机使用 Python 3.12.4、SciPy 1.13.1、statsmodels 0.14.2、pandas 2.2.2 验证多特征 OLS fixture；完整设计矩阵得到 `const=1.333333`、`x1=3.0`、`x2=0.666667`、`R²=0.984016`。

### 阶段结论

EKO 需要加强数据处理与分析，但产品优势不是再造一个黑盒 BI 或统计引擎，而是把 coding 主流程延伸到数据领域：用户在对话中与 AI 共创可读代码，审阅假设和参数，修改后重跑，并保留完整分析 artifact。框架只提供通用探索与沙箱执行原语；领域方法、lineage、Notebook/报告体验留在 EKO 应用层。没有新增运行状态、数据库、Notebook kernel 或统计 DSL。

### 验证结果

- `echo-agent ./scripts/verify-all-crates.sh` 全绿：all-targets/all-features `1785` 个测试；独立 feature、最小 feature、默认/all-features Clippy 和生产 panic API 门禁全部通过。
- `echo-agent-cli` 默认 workspace 全绿：app-core `511`、runtime e2e `5`、CLI lib `44`、CLI main `9`；all-features、GUI feature `43` 个测试、`channels`、`tui,telemetry`、GUI target 与 `gui,devtools` 组合全部通过。
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`、`cargo fmt --all -- --check` 通过。
- EKO 前端 `npx tsc -b`、`npm run build`、`56` 个 Vitest 全部通过；官网 `npm run lint` 与 `npm run build` 通过，仅保留 Rollup 大 chunk 提示。
- 两个 Rust workspace 已执行 `cargo clean`，分别释放约 `25.7 GiB` 与 `47.0 GiB`，合计约 `72.7 GiB`。

### 下一步候选：文件化数据分析工作台

继续使用现有文件、artifact、`run_code`、review 和 Subagent 流程，提供脚本/结果并排审阅、参数修改、重新运行、输入/环境 lineage 与导出；不先自建 Jupyter kernel。只有真实工作流证明逐单元有状态执行不可替代时，才评估接入成熟 Jupyter runtime。

---

## 二十三、记忆与自进化实现评估（2026-07-23 历史快照）

本节归档一次**只读**架构评估的结论与代码入口，供新窗口接续。**尚未动手改代码**。评估同时覆盖 `echo-agent` 框架与 `echo-agent-cli` 应用两侧的记忆（memory）与自进化（evolution）实现。

### 总体判断

架构方向正确、与业界头部实现收敛，问题集中在"接缝"（拼接处的 bug、重复、半接线），而非设计本身。业界对照：

- [Letta / MemGPT sleep-time compute](https://www.letta.com/blog/sleep-time-compute/)：后台 agent 每 N 步（默认 5）"anytime" 重写主 agent 的 memory block，主 agent 随时可见。对应 EKO 的 Dreaming，但 EKO 是"每天 cron + 纯启发式 + 写文件下次启动才读"，退化成了静态配置。
- [Claude Code memory](https://code.claude.com/docs/en/memory.md)：两套系统——CLAUDE.md（指令，入 git）+ auto-memory（`MEMORY.md`，自蒸馏），会话启动加载，"context not enforcement"，建议 <200 行/聚焦。对应 EKO 的文件指令层 + 热层 `MEMORY.md` + Evidence Inbox 审批门。
- [Cursor Dream Mode](https://nexustrade.io/blog/cursor-vs-claude-code-memory-architecture-20260413)：空闲期 4 阶段整理，读 `ENTRYPOINT.md` 索引先了解已有内容，条目一行 150 字、索引 <25KB，**不用向量库**（"把钱花在维护循环，不是存储层"）。

### 三层记忆现状（EKO）

| 层 | 存储 | 何时读 | 何时写 |
|---|---|---|---|
| 指令/热层文件 | `user.md`/`project.md`/`AGENTS.md`/`local.md`/`MEMORY.md` | **仅 boot 时**注入系统提示词 | 用户编辑；Dreaming 促进 `MEMORY.md`；RulePromoter 写 `AGENTS.md` |
| warm 动态 Store | `{root}/.eko/memory/store.json` 或 `~/.eko/store.json` | 每轮 `MemoryRecaller` 复合评分召回 | `remember` 工具、accepted evidence、L3 promotion、TaskRuntime memory_bridge |
| Evidence Inbox（审批门）| `.eko/evolution/evidence-candidates.jsonl` | GUI/TUI 面板 | 自动记忆、BackgroundReviewer、trigger、memory-review 冲突 |

唯一权威写入口：`echo-agent` 的 `MemoryLayerManager::write_memory`。召回复合评分：`S = 0.5·sim + 0.3·decay(age,30d) + 0.2·recall_weight`（`echo-agent/src/evolution/recall.rs`）。

### P0 — 正确性 bug（记忆"写了读不到" / 进化跟错文件）

1. **Boot-time 系统提示词永不刷新**。`memory_context_suffix` 仅在 `echo-agent-cli/echo-agent-app-core/src/runtime.rs:110-114` 设置一次。Dreaming 写进 `MEMORY.md` 后当前会话读不到（要重启）；工作区切换时 `state.rs:893-957` 换了 store/ReviewIntegration/pool 却**没重刷 suffix**，agent 仍带上个项目的指令。业界（Letta）是 anytime 可见——这是记忆闭环里最伤体验的一处。
2. **TUI `/remember` 与 Tauri `add_memory` 写错命名空间、绕过唯一写入口**。TUI 写 `["default","memories"]`（`echo-agent-cli/src/tui/events.rs:2744`），框架召回只查 `["agent","memories"]`，导致手动 remember 永远召回不到。违反 TUI/GUI 功能对等。应统一走 `MemoryLayerManager::write_memory`。
3. **Curator 状态文件路径分裂**。框架 `Curator::default_path` → `~/.echo-agent/curator_state.json`（`echo-agent/src/evolution/curator.rs:214`，注：还是旧名，路径重构漏改框架默认值）；EKO 工作区用 `.eko/evolution/curator-state.json`。而 `execution.rs:135` 的 `touch_skill` 仍用 `default_path`——"最后使用时间"写全局、生命周期老化读工作区，idle 老化算错，技能可能被误判 Stale/Deprecated。

### P1 — 一致性 / 重复

4. **命名空间双轨**：legacy `[agent_name,"memories"]`（`echo-agent/src/agent/react/mod.rs:718`）vs 统一 `["agent","memories"]`。建议删 legacy 工具链（`LegacyStoreRememberTool`/`RecallTool`/…），只留 Layered 一套。
5. **L3 事实抽取两条并行路径**：`StoreMemoryPromoter`（压缩驱逐启发式，key `l3_{hash}`）+ `pre_compaction_flush`（压缩前 LLM，key `flush_{nanos}`）可能对同一事实写两条。二选一或加内容去重。
6. **changelog 文件名不统一**：CLI `change-log.jsonl`（`evolution.rs:868`）vs GUI `changelog.jsonl`（`panels.rs:1468`）——看的是两个文件。
7. **`/memory-review` CLI 新建 `ReviewIntegration` 而非复用共享实例**（`evolution.rs:600`），工作区 rebind 后用错路径。

### P2 — 死代码 / 半接线（可直接删，符合 YAGNI）

- `UnifiedMemory.hot_content` + `refresh_hot()` 全仓库无调用者（`unified_memory.rs:175`）；`MemoryContext.memories` 生产恒为空。
- `MemoryLayer::Cold` / `COLD_NAMESPACE`：stage4 已用 `Archived` 取代，框架侧可作菜单保留，app 侧引用可清。
- 两套 staleness 模型并存：`MemoryMeta::base_staleness` vs `StalenessScorer`；`MemoryReviewer` 产 `StalenessSuggested` 但从不应用（Dreaming 用 recall_count 另一套）。归一。
- `improve::background_review` 是纯 re-export shim；`memory_promoter.rs` 文档写"30天 TTL"但代码不设 `expires_at`（过时注释）。
- Pool agent `memory_context_suffix: None`（`agent_pool.rs:799`），拿不到指令层，与主 agent 不对等。

### P3 — 架构增强（方向性，须先对齐"evolution 已收缩"决策）

> ⚠️ 注意：本文件第一节已明确"**不再建设自进化指标平台**"，"明确不规划"含"自动重写基础 prompt""指标驱动自动修改 memory/rule/skill"。以下 P3 仅在**不违反该边界**时才做，倾向"更好的用户显式触发/空闲触发"，不做后台自动评分。

- Dreaming 触发从"每天 cron"改为"会话结束/空闲触发"，且**结束后热刷新 suffix**（直接解决 P0-1）。
- `MEMORY.md` 加条目级预算护栏（业界 <200 行 / 索引 <25KB / 条目一行）。当前只有 `HOT_TOKEN_BUDGET=2000`，无条目约束。
- 评估向量检索（`EmbeddingStore`/hybrid）对本地个人助理是否过重；可默认关键词 + recency，向量作可选。

### 当时建议处理顺序（非当前任务）

先修 P0（三个正确性 bug，对"记忆能用起来"影响最大）→ 清 P1/P2（一致性 + 死代码）→ P3 作为后续独立里程碑并先对齐 evolution 边界。P0 属于**改聊天主路径 + 工作区切换**的高风险改动，建议在新鲜上下文进行；动手前按第九节规则先写专项 spec、grep 框架与应用确认调用点。

### 关键代码入口（评估用）

- `echo-agent/src/evolution/`：`layer.rs`（MemoryLayerManager 热/warm）、`recall.rs`（复合评分召回）、`dreaming.rs`（DreamingConfig 默认 recall≥5/日）、`curator.rs`（技能生命周期 + default_path bug）、`triggers.rs`、`background_review.rs`、`candidate.rs`、`draft.rs`。
- `echo-agent/echo-core/src/memory/`：`store.rs`（Store trait/StoreItem）、`types.rs`（MemoryMeta/枚举/is_hot_eligible）。
- `echo-agent/echo-state/src/memory/`：`store.rs`（FileStore/InMemoryStore）、`sqlite_store.rs`、`embedding_store.rs`、`typed_store.rs`。
- `echo-agent/src/agent/react/run/context.rs`：`recall_long_term_memories`（每轮召回注入）、`pre_compaction_flush`。
- `echo-agent/src/memory_promoter.rs`：压缩驱逐启发式 L3。
- `echo-agent-cli/echo-agent-app-core/src/`：`unified_memory.rs`、`instruction_provider.rs`、`runtime.rs:105-118`（boot suffix）、`state.rs:893-957`（工作区切换 rebind）、`infra.rs:1131-1176`（spawn_dreaming）、`agent_pool.rs`、`evolution/review_integration.rs`、`evolution/evidence.rs`、`evolution/rule_promoter.rs`、`tasks/task_runtime/memory_bridge.rs`。
- `echo-agent-cli/src/`：`tui/events.rs:2744`（TUI /remember bug）、`tauri/commands/memory.rs`（legacy 原始 Store 管理）、`cli/cmd_impls/evolution.rs`、`cli/cmd_impls/all.rs`。

---

## 二十三.5、Hook / Plugin 收敛历史（P0 正确性 → P1 闭环）

**背景**:2026-08-10 审计发现 Hook/Plugin 系统"声明支持远多于主路径真正可用":33 事件/7 动作的权威定义 vs 文档写 20/5;`HookSource::Plugin` 从未被构造;插件 hook 被错误归到 `Skill` 来源;`echo-agent.yaml` 与两个 `hooks.yaml` 互相 `clear_user_hooks()` 覆盖;插件目录硬编码 `~/.echo-agent` 绕过 `.eko` 覆盖;EKO TaskRuntime 四个 Task Hook + `SubagentCancelled` 未接通;`PluginIntegrator::wire_all` 是零调用死代码,应用层手写装配重复;`reload_plugins` 只重统计不装配。

**业界调研结论**(Claude Code / Codex,2026-08-10):装配主循环放框架层(Codex `discover_handlers`+`Hooks::new` 在框架,app 只传 config + plugin sources;Claude Code `HookSource` 含 Plugin 变体);enable/disable 通过"重新 discovery + 重建 registry"而非活 agent 热插拔。**本项目采纳方向 (A):框架成权威装配器,应用退化为薄 adapter。**

### 分层判定(AGENTS.md 门禁 #1)

| 能力 | 归属 | 依据 |
|---|---|---|
| `HookEvent`/`HookAction`/`HookRegistry`/matcher/执行器/结果合并 | 框架(`echo-core`/`echo-execution`) | 任何复用方都需要的通用机制 |
| `HookSource` enum(含 Plugin 变体) | 框架 | source identity 是通用概念 |
| `PluginRegistry`/manifest/scope/discovery/依赖拓扑 | 框架(`echo-core`) | 通用 |
| `PluginIntegrator::wire_all`(装配主循环) | 框架(facade) | discovery+precedence+装配是通用机制 |
| `plugin_data_base_dir` 可配置 base dir | 框架(`echo-core`,独立 OnceLock,因不能依赖 facade) | 让应用注入品牌目录 |
| `TaskHookBridge`/`SubagentHookBridge` | 框架(facade) | trait→HookRegistry 适配,通用 |
| EKO 品牌 base dir(`.eko`) | 应用 | 产品决策 |
| `HookConfigLoader`(三源合并) | 应用(`echo-agent-app-core`) | 合并的是 EKO 特有的三个文件位置 |
| EKO TaskRuntime 接 bridge 的 4 个 fire 点 | 应用 | EKO 自管 DAG 调度器不走框架 TaskExecutor |
| `PluginRuntimeService`(共享 registry + rebuild) | 应用 | 持 `AgentHandle`,产品级 reload 语义 |

### 进度

#### P0 正确性(进行中)

- ✅ **P0-3 插件路径分裂**:`echo-core/plugin/mod.rs` 新增 `plugin_data_base_dir()` + `set_plugin_data_base_dir_name()`(OnceLock,与 facade `paths` 解耦避免循环依赖);`scope.rs` User、`registry.rs` state/data、`variables.rs` data_dir 全部改走该入口。EKO 在 `main.rs`/`tauri/desktop.rs` 启动时调 `set_plugin_data_base_dir_name(".eko")` 与品牌目录对齐。Project/Local scope 保持 `.echo-agent` 约定(VCS 共享路径须稳定)。已提交 echo-agent `43f2b44`。
- ✅ **P0-2 插件 hook source 身份**:`HookRegistry::register_plugin_hooks(plugin_name, source_dir, def)` 构造 `HookSource::Plugin(name)`;注册前对每个 action 调 `validate()`,无效 action 记日志丢弃(不毒化同插件其它 hook);重注册按 source 替换支持 reload。`PluginIntegrator::wire_all`/`wire_hooks` 改用新方法。3 个单测全绿。已提交 echo-agent `43f2b44`。
- ✅ **P0-5 接通 EKO TaskRuntime 的 4 个 Task Hook + SubagentCancelled**:框架侧 `BridgedTaskHooks::bridge()` 访问器(echo-agent `309a197`)。应用侧:`execute_run`/`execute_task` 加 `hook_bridge`/`subagent_bridge` 参数,4 个生命周期点 fire(before/after/timeout/cancelled);`ExecuteTaskTool` 加注入器;所有执行路径(register/run_driver/scheduler-runner/tasks-service/tui-events/tauri-task_runtime)从对应 agent 的 hook registry 构造 bridge 注入,TUI/GUI 对等。已提交 echo-agent-cli `f88ba84`。

> ⚠️ **P0-5 语义修正(2026-08-10 用户复核发现)**:上述 P0-5 实现按错误语义接通了 Task bridge,有 5 个结构性冲突隐患,**已决定回滚 Task bridge fire 点并重新设计**:
> 1. `TaskCreated` ≠ `before_execute`:PlanTask 在 revision 提交时就创建了,`before_execute` 实际是 `TaskStarted`。动态 plan 更新时这区别关键。
> 2. **task_id 丢失**:`TaskHookBridge::fire_event` 参数名 `_task_id`,用通用 `for_lifecycle()` 构造 Context → Hook 收不到结构化 `task_id`,多同名任务无法关联。
> 3. **Timeout/Cancelled 无承载对象**:`TodoStatus` 只有 Pending/Running/Blocked/Completed/Failed/Skipped(无 Cancelled/TimedOut);取消时未完成 PlanTask 标 `Skipped`、TaskRun 才进 `Cancelled`(`finalize_cancelled_run_state`)。execute_task 的 timeout 是 SubagentRun 级超时,映射成 `TaskTimeout` 是层级错位。
> 4. **Subagent 终态混合模型**:同时有 `SubagentStop` + `SubagentCancelled`,无 `SubagentTimedOut`;`SubagentStop` Context 无 `status`/`subagent_run_id`/`attempt`。取消路径若 Cancelled+Stop 都发会双发。
> 5. **Subagent 双发隐患**:框架 `SubagentExecutor` 能直接发 `SubagentStart/Stop`(经 `unified_hook_executor`,executor.rs:504/580/714),EKO 当前未 set 该选项所以未双发,但应用层 bridge 注入后属结构性脆弱。
>
> **修正决策**(用户确认):① 回滚 executor.rs 的 Task bridge 4 fire 点(保留 P0-1/P0-3 不变);② 现在归一 Subagent 终态模型:`SubagentStop(status=completed|failed|cancelled|timed_out)` 始终只发一次,`SubagentCancelled` 改兼容别名/删;③ HookContext 加 `run_id`/`task_id`/`subagent_run_id`/`attempt`/`plan_revision`/`terminal_status`;④ `TaskCreated` 对应 revision 新增节点,执行边界用 `TaskStarted`;⑤ `TaskTimeout`/`TaskCancelled` 在 PlanTask 状态模型确定前不接线;⑥ 明确 emission owner 边界:`ExecEventScope::{Run,Task,Subagent}` 已正确表达层级,Hook 与其对齐。**这是跨框架公共 API(HookEvent/HookContext)+ 应用层重构,属高风险,建议在新鲜上下文做。**
>
> **修正进度**:
> - ✅ ① 回滚 P0-5 bridge(echo-agent-cli `a30d5e2`,P0-1/P0-3 保留,721 测试全绿)。
> - ✅ ② Subagent 终态归一(echo-agent `472bb25`):新增 `SubagentStopStatus` 枚举(completed/failed/cancelled/timed_out)+ HookContext `subagent_stop_status` 字段 + `for_subagent_stop` 带 status;删除 `HookEvent::SubagentCancelled`(枚举/category/as_str/映射/bridge on_cancelled);`SubagentExecutor` 两个 fire 点传 status,是唯一 emission owner;`SubagentHookBridge::on_after_dispatch` 带 status,删 on_cancelled。业界依据:Claude Code/Codex/OpenAI Agents SDK/AGTP 全部收敛到双事件+status 枚举。
> - ⏳ ③④⑤⑥ Task 事件语义重设计 + HookContext 加 run_id/task_id/attempt:留后续提交(TaskCreated vs TaskStarted、TaskTimeout/Cancelled 承载对象需先定 PlanTask 状态模型)。
>
> **Task 事件重设计完成**(echo-agent `78cac69`):业界调研(Codex item.started/completed + Cursor plan-submit vs build-start + Claude Code TaskCreated/TaskCompleted)确认三段式优于两段式,timeout/cancelled 是终态 status 而非独立事件(Codex CommandExecutionStatus)。框架变更:① HookEvent 新增 `TaskStarted`,删 `TaskTimeout`/`TaskCancelled`(降为 `TaskCompleted` 的 status);② 新增 `TaskTerminalStatus` 枚举(completed/failed/cancelled/timed_out/skipped/blocked);③ `HookContext` 加 `task_terminal_status` + `run_id`/`plan_revision`/`subagent_run_id`/`attempt`(应用层 fire 时填充);④ `TaskHookBridge` 修 task_id 丢失(改用专用工厂)+ 语义重映射(`on_created`/`on_before_execute`→TaskStarted/`on_after_execute(id,subj,result,status)`→TaskCompleted);⑤ `BridgedTaskHooks` trait impl 所有终态收敛到 TaskCompleted(status)。门禁全绿。
>
> **应用层接入待做**(EKO task_runtime):在 `RuntimeEventKind` chokepoint(plan revision commit→TaskCreated、`append_task_status_event`→TaskStarted/Completed(status 映射 TodoStatus)、`record_subagent_released`→SubagentStop(status))接入框架新事件。精确 fire 点见本节调研结论表。
- ✅ **P0-1 唯一 HookConfigLoader**:应用层新建 `hook_config_loader.rs`,三源(echo-agent.yaml 内嵌 + `~/.eko/hooks.yaml` + `.eko/hooks.yaml`)按固定顺序合并成单份再 register;`infra.rs::load_user_hooks` 用 `load_merged`(bootstrap 唯一入口);bootstrap 删除第二次 `load_hooks_files+clear`(消除互覆盖);`/hooks reload` 用 `load_merged_from_disk`(重读内嵌)。7 个单测。已提交 echo-agent-cli `f88ba84`。
- ✅ **P0-2b 应用层薄 adapter**:`runtime.rs::load_plugins` 直接调框架 `PluginIntegrator::wire_all`,删除手写装配循环。已提交 echo-agent-cli `f88ba84`。
- ⏳ **P0-4 PluginRuntimeService**:进程级共享服务,持 `PluginRegistry` + `AgentHandle` 引用,enable/disable/reload 时跑框架 `wire_all` 重建装配(非热插拔)。`AppState` 持有,所有 tauri command 从 state 取而非各自 `build_registry()`。**下一步,建议在新鲜上下文做**(碰 AppState/runtime.rs 整合面,高风险)。

#### P1 闭环(待 P0 稳定后)

- ✅ **P1 插件组件接入**(echo-agent `df8a99b`):Subagent definitions 真装配(register_subagent_definition late-binding + parse_subagent_md);LSP/monitors/themes/output styles discovery(框架无消费者,只 resolve + 报告)。
- ✅ **P0-4 PluginRuntimeService**(echo-agent-cli `6b2e764`):进程级共享服务,enable/disable/reload/install/uninstall 经共享 registry + wire_all rebuild;tauri 8 个 command 从 AppState 取共享服务;reload_plugins 返回完整 ReloadSummary。**已知限制:disable/uninstall 不卸载已注册组件(wire_all additive),需 agent 重启或框架组件移除支持。**
- ✅ **应用层 Task/Subagent 事件接入**(echo-agent-cli `6b2e764`):HookEventDispatcher 订阅 RuntimeEventKind 事件流,翻译成框架 HookEvent;FileTaskShadow event_hook(OnceLock)是单一注入点;register_task_tools_on_agent 注入 dispatcher,所有执行路径自动接入。
- ✅ **P1 semver 版本约束**(echo-agent `0d93085`):PluginDependency::satisfies + resolve_dependencies 执行版本检查。
- ⏳ **P1-reload 真实卸载**:SkillRegistry 当前按 name 存 descriptor,**无来源追踪**(不知道 skill 来自哪个 plugin)。要实现 disable 卸载,需给 SkillDescriptor 加 source 字段或 SkillRegistry 加 by_source 索引(框架重构,影响 SkillDescriptor 序列化 + 所有加载路径)。HookRegistry 已有 unregister(HookSource),但 Skill 缺。**属高风险框架重构,建议新窗口。**
- 🔄 **P1-frontend**:GUI Hooks 面板 + TUI Plugins 命令对等(subagent 进行中)。
- ⏳ 按本地威胁模型审查 HTTP/MCP 限制(保留密钥脱敏/超时/明显错误校验)。
- ⏳ Hook JSON Schema 契约测试(每个非保留事件须有生产触发点)。

**P1-reload + P1-frontend 完成**(echo-agent `0dad670` + echo-agent-cli `commit`):
- ✅ **P1-reload 真实卸载**:框架 SkillDescriptor 加 `source` 字段(serde skip,不 breaking)+ SkillRegistry `by_source` 反向索引 + `unregister_by_source` + `tag_source`。PluginIntegrator::wire_all 装配 skills 后 tag_source("plugin:{id}")。应用层 PluginRuntimeService::disable/uninstall 调 `unregister_by_source` + `HookRegistry::unregister` 从运行中 agent 移除该插件的 skills/hooks,然后 reload 重新装配。MCP 暂不 disconnect(无 per-server API,reload idempotent 让其惰性化)。
- ✅ **P1-frontend**:GUI Hooks 面板(HooksPanel.tsx,列来源+规则数+reload)+ Tauri `list_hooks`/`reload_hooks` command + CLI `/plugins reload` 显示完整 ReloadSummary + TUI/CLI `/hooks` 对等确认。

### 已切换的权威路径

- 应用层 `runtime.rs::load_plugins` 退化为薄 adapter:在 `write_async` 闭包内直接调框架 `PluginIntegrator::new().wire_all(a, &mut plugin_registry).await`,删除手写的 skills/hooks/mcp 三段装配循环。plugin hook 经框架 `register_plugin_hooks` 落到 `HookSource::Plugin`(不再误归 Skill)。
- 应用层 `hook_config_loader::HookConfigLoader` 是 user hook 的唯一加载入口(bootstrap + `/hooks reload` 共用),三源合并后单次 register。旧的 `hooks_config::load_hooks_files` 退化为 re-export shim(仅诊断用,会丢内嵌 hooks)。
- EKO TaskRuntime 的 4 个 Task Hook + SubagentCancelled 经 `TaskHookBridge`/`SubagentHookBridge` fire;bridge 在所有执行路径(ExecuteTaskTool / 直接 execute_run 的 TUI/GUI/cron/background)从对应 agent 的 hook registry 注入。

### 剩余重复 / 待收敛

- `tauri/commands/plugins.rs` 每个 command 仍各自 `build_registry()` 新建独立 registry,与运行中 agent 脱节 —— P0-4 收敛到共享 `PluginRuntimeService`。
- 框架 `PluginIntegrator::wire_skills`/`wire_mcp` 单组件入口仍存在(供 partial wire 场景),非重复,保留。
- `BridgedTaskHooks`(TaskHooks trait impl)与 `TaskHookBridge`(直接方法)并存:前者服务于走框架 TaskExecutor 的复用方,后者服务 EKO 自管 DAG。两者语义不同,保留;EKO 用后者。

---



### `echo-agent-cli`

- `echo-agent-app-core/src/chat_driver.rs`：`drive_chat`、`ChatDriverEvent`、`ChatSink`。
- `echo-agent-app-core/src/hitl/`：TUI/channel/REPL HumanLoop provider。
- `echo-agent-app-core/src/surface_contract.rs`：五入口能力矩阵与 wire contract tests。
- `echo-agent-app-core/src/run_driver.rs`：`drive_run_async`。
- `echo-agent-app-core/src/tasks/task_runtime/`：TaskRun、plan、executor、pause/resume/cancel、cron/background。
- `echo-agent-app-core/src/scheduler/runner.rs`：cron 到统一 run 的入口。
- `echo-agent-app-core/src/infra.rs`：agent/tool/sandbox/runtime store 装配。
- `echo-agent-app-core/src/agent_pool.rs`：pool agent 继承与 prompt/compressor/sandbox 配置。
- `echo-agent-app-core/src/project/prompt.rs`：`PromptAssembler`。
- `echo-agent-app-core/src/observability/`：usage/cache/prompt diagnostics。
- `src/tauri/commands/chat.rs`：GUI sink 与 Tauri transport。
- `src/tui/events.rs`：TUI sink/reducer。
- `src/cli/repl.rs`、`src/cli/channels.rs`：CLI/channel 消费者。
- `web-frontend/src/stores/`：GUI projection；不得成为权威运行状态源。

### `echo-agent`

- `echo-core/src/agent/`：`AgentEvent`、`EventEnvelope`、identity。
- `echo-core/src/tools/`：tool contract、stream、sandbox injection。
- `echo-execution/src/tools.rs`：ToolManager 执行与 sandbox 注入。
- `src/agent/react/run/`：ReAct 主流程、tool batch、checkpoint、terminal。
- `src/agent/snapshot.rs`：runtime checkpoint、transcript projection、tool output guard。
- `echo-tools/src/{shell.rs,code.rs}`：shell 与 `run_code`。
- `echo-tools/src/statistics.rs`：仅探索性描述统计；正式推断由保存的成熟库脚本承担。
- `src/agent/subagent/`：Subagent/Team dispatch、checkpoint、timeout。
- `echo-core/src/llm/cache/`、`echo-integration/src/providers/`：cache layout 与 provider usage。
- `echo-state/src/compression/`：压缩器和 protected context 合并。

---

## 二十四.1、Workspace generation 收敛归档（2026-08-14）

### 分层与权威

- 通用取消、typed terminal、文件 CAS 和有界子进程继续由 `echo-agent` 提供；没有下沉 EKO workspace、UI 或 scheduler 产品策略。
- `AppState::switch_workspace/exit_workspace` 是 EKO generation 变更的唯一权威入口。GUI、TUI、CLI 统一调用它；TUI/CLI 不再自行创建 workspace store 或用注册表首项推断 current。
- 现有 TaskRuntime graph、AgentPool、conversation/memory store 和 `drive_chat` 被直接扩展；没有新增平行 Task/Plan/Todo store、ready frontier 或重试循环。

### 本阶段已切换

- workspace root、目录和全部可预构建 store 先验证；active foreground chat、Running TaskRun、busy pooled Agent 会阻止 generation 变更，防止旧执行写入新 workspace。
- primary/pool 的 working dir、conversation store、memory store、runtime state、tool artifact policy 和 workspace profile 完成重绑后，才发布 `workspace.current` 并通知 config watcher。
- TUI/CLI workspace command、附件、长文本 spill、文件索引和每轮 conversation id 均读取共享 `AppState`；切换后清理旧 surface projection 并使用新 conversation identity。

### 仍未完全收敛

- EKO `TaskRuntimeStore` 仍是进程启动时创建的全局 `Arc`，scheduler/background service 同样持有该实例；当前通过“存在 Running run 时拒绝切换”避免并发污染，但持久根尚未按 workspace generation 重绑。
- plugin/LSP/MCP 等长生命周期服务未加入统一 prepare/commit receipt；`switch_workspace` 后半段若出现罕见失败，仍缺覆盖所有参与者的 rollback 或显式 degraded receipt。
- channel/REPL 当前 turn cancellation、GUI browser event bridge、MCP 配置持久化仍需用同一 capability fixture 做 surface parity 验收。

### 下一阶段删除目标

1. 将 TaskRuntime/scheduler/background service 改为 generation-scoped handle，在无 active run 的 safe point 原子换代；删除启动期全局 root 的隐式假设。
2. 为 plugin/LSP/MCP 增加应用层 prepare/commit/rollback adapter，并让 `AppState` 返回 typed generation receipt；禁止 adapter 自建第二套状态机。
3. 以 GUI/TUI/CLI/channel 同 fixture 关闭剩余 parity 项；真实路径切换后删除所有 surface-local current/workspace 推断。

---

## 二十四.2、五任务重构计划与 Task 5 阶段账本（2026-08-23）

五任务并行重构计划（Task 1 Framework Correctness / Task 2 EKO Control Surface / Task 3
Public Boundary / Task 4 Task-Subagent Kernel / Task 5 Runtime-State Kernel）的 framework wave
已合入 `echo-agent/main@826943b`（本地 main，未 push）。Task 5 应用迁移分支为
`echo-agent-cli:refactor/runtime-state-kernel`，worktree 为
`echo-agent-cli/.worktrees/refactor/runtime-state-kernel`；其相对 Cargo path 通过
`echo-agent-cli/.worktrees/refactor/echo-agent` 指向 framework main，tracked manifest 不写绝对路径。

### Task 5 framework authority（已完成，echo-agent `826943b`）

按计划第一波定义"benchmark、characterization tests、EventJournal/TurnDriver 独立原语"交付：

- **echo-state `journal` 模块**：`EventJournal`/`CheckpointedReducer` 已覆盖连续序列、append
  ambiguous-commit 对账、poison/reopen、torn-tail 修复、固定批次 replay 与 checkpoint compounding。
  这是 EKO `ChatEventLog` 与 TaskRuntime `events.jsonl`+`checkpoint.json` 的迁移目标；应用目录、
  retention、产品事件投影继续留在 EKO adapter。
- **echo-orchestration `runtime` 模块**：`AgentTurnDriver`/`TurnRequest`/`TurnOutcome`/`EventSink`。
  通用单 turn 驱动现以 `&dyn Agent` 借用现有 AgentHandle read guard，owned `EventEnvelope`
  逐项交给 async sink；框架拥有 raw stream、sequence、exactly-one-terminal、provider usage receipt
  与 sink error/close 语义，不需要 EKO 建 relay 或第二套循环。
- **File authorities / MCP / RuntimeTaskService / facade**：FileStore copy-on-write + process lease、
  FileConversationStore canonical shared authority、MCP target reconcile/drain、claim/CAS/retry exhaustion、
  `echo_agent::{runtime,state::journal,tasks,mcp}` 公共入口均已完成并通过最终门禁。
- **验证**：framework workspace all-targets/all-features `2156 passed / 0 failed / 3 ignored`，
  Criterion smoke `12/12`，all-features rustdoc `134 passed / 0 failed / 41 ignored`，no-default 与
  `sqlite/subagent/human-loop/mcp/lsp/a2a/git/database/rag/chart/web/media/channels` 13-feature matrix 全绿。
- 业界依据与分层判定写在两个模块的 rustdoc 里（Codex rollout recorder、LangGraph
  checkpoint persistence、event-sourcing snapshot；通用机制归框架，目录/retention/产品投影归应用）。

### Task 5 CLI Stage 1：共享 chat driver authority（进行中）

实现前全仓库 duplicate/runtime-reachability 搜索结论：GUI (`src/tauri/commands/chat.rs`)、TUI
(`src/tui/events.rs`)、CLI/JSONL (`src/cli/{repl,modes,jsonl}.rs`) 与 channel
(`src/cli/channels.rs`) 的生产模型执行均经 `echo-agent-app-core::chat_driver::{drive_chat,
drive_chat_turn,drive_pooled_chat_turn}` 汇入 `drive_chat_inner`；surface `ChatSink` 只做产品投影，
没有另一条 surface-local Agent stream loop。

分层判定与本阶段唯一 adapter：

- **framework 通用机制**：`AgentTurnDriver` 独占 Agent raw stream、`EventEnvelope` sequence、generic
  terminal/usage receipt 和 async delivery boundary；EKO 在现有 AgentHandle read guard 内以
  `&*guard` 调用，禁止 `as_shared_agent()` Agent-stream relay 或第二套 terminal loop。
- **EKO 产品策略**：`PreparedUserTurn` 合流、InteractionMode 工具可见性、TaskRuntime RunTurn
  admission/continuation/finalization、HITL/resources、workspace working_dir、RunTurn usage/compaction
  幂等投影、webhook、internal continuation transcript suppression 和 surface journal/render 继续留应用层。
- **薄 adapter**：owned framework envelope 先执行 EKO usage/compaction/webhook/suppression，再在
  单个 per-turn blocking projector 中调用现有 journal-before-render `ChatSink`；容量 64 的 bounded
  mpsc 与逐事件 oneshot ack 保证 backpressure、顺序和 journal-before-next-event。未投递返回 typed
  `downstream_disconnect` error，只有已经接收 envelope 后主动关闭才使用 `SinkControl::Closed`。
- **同阶段删除**：删除 `drive_chat_inner` 的 `execute_stream_message_with_invocation_context` setup、
  `envelope_event_stream`、raw drain、generic terminal inference/usage tally，以及应用内重复
  `TurnOutcome` 定义；保留 `ChatTurnOutcome` 作为 EKO receipt projection。

### Task 5 后续阶段仍存在的重复

1. `echo-agent-app-core/src/tasks/task_runtime/executor.rs` 约 3561 行的 task main-agent envelope/drain
   loop，以及约 4223 行的 run-agent drain loop，仍各自拥有 stream/terminal 投影；Stage 1 不迁移，
   下一阶段接 `AgentTurnDriver`/RuntimeTaskService 后删除。
2. `ChatEventLog` 与 TaskRuntime `events.jsonl`/checkpoint 文件算法尚未切换到 framework journal；
   必须在 Stage 1 review 后迁移并同步删除旧扫描、append、checkpoint 实现。
3. TaskRuntime 文件 I/O、boot recovery/safe-point、durable mailbox 与 typed IPC regeneration 仍按后续
   阶段推进；不得因本次 chat driver 切换引入平行 store/validator/executor。

## 二十五、验证规范摘要

完整要求见根目录 `AGENTS.md`。提交前必须全部通过，任何既有失败也要修复。

```bash
# echo-agent：根 package + workspace，统一覆盖全部成员
cd echo-agent
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo clippy --workspace --lib --bins --all-features --locked -- \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D clippy::panic \
  -D clippy::unreachable
cargo test --workspace --all-targets --all-features --locked
cargo check --workspace --lib --no-default-features --locked

# echo-agent-cli：真实 workspace + GUI target
cd echo-agent-cli
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo clippy --workspace --lib --bins --all-features --locked -- \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D clippy::panic \
  -D clippy::unreachable
cargo test --workspace --all-features --locked
cargo check -p echo-agent-app-core --no-default-features --locked
cargo check --no-default-features --features gui --bin echo-agent-tauri
cargo test --no-default-features --features gui

# 前端
cd web-frontend
npx tsc -b
npm run build

# 仅在 AGENTS.md 的磁盘阈值触发时清理对应 workspace
df -h .
du -sh ../echo-agent/target target 2>/dev/null
```

提交使用：

```bash
git -c commit.gpgsign=false commit -m "..."
```

不得提交 worktree 绝对路径；`echo-agent-cli` 不启用 `echo-state/sqlite` feature。
