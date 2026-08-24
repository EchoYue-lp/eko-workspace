# AGENTS.md

本文件是 AI agent 在本仓库工作时的**最高优先级约束**(优先级高于 agent 默认行为和任何技能)。请严格遵守。

---

## 项目简介(冷启动免重新读)

`eko-workspace` 是 EKO 的顶层 Git superproject,通过 submodule 统一管理三个**互相独立的 git 子仓库**,但不是 monorepo。工作根目录为 `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent`。

技术栈总览:Rust 后端(`echo-agent` 框架 + `echo-agent-cli` 应用)+ React/TypeScript 前端 + Tauri 桌面壳。

---

## 产品定位与安全边界(做安全决策前必读)

**EKO 是本地个人超级智能助理,运行在用户自己的机器上,不部署到线上,不存在多用户 / 公网攻击场景。**

这条定位是所有安全设计的出发点,影响每一个"要不要加权限门控"的决定:

- **威胁模型是本地的**:没有跨用户隔离、没有公网暴露、没有恶意租户。用户能打开这个应用,本身就说明用户信任这台机器。
- **不要套用线上 Web 服务的威胁模型**:诸如"防 XSS→RCE""防 SSRF 内网穿透""多用户权限隔离"这类线上服务的安全闸,**默认不适用于 EKO**。给本地桌面应用硬加这些门控,只会让正常功能(终端、MCP 扩展)在默认配置下不可用,是典型的过度设计。
- **交互式 vs 自动化要区分**:终端、文件选择器这类是**用户主动操作**的开发者工具,不该被"agent 自动执行权限"(`full-auto`/`default` 等 permission_mode)卡住——那是管 agent 自动行为的闸,不是管用户交互的闸。`full-auto` 这类权限模式只应用于 **agent 自动决策路径**(如 agent 自己跑代码、写文件),不要套到用户直接点的功能上。
- **用户自扩展的能力由用户自己负责**:用户配置 MCP server、加载技能、写 hook,是用户在扩展自己的助理——用户在扩展前会自行评估风险,框架不必越权替用户把关到底。保留对**明显错误输入**(命令名拼错、URL 用了明文 http)的轻量校验即可,不要做权限级拦截。
- **何时该加防护**:仅当(1)防止用户无意中的数据丢失(如覆盖未提交改动)、(2)防止框架自身 bug 造成破坏、(3)本地也成立的通用安全(如不把密钥打进日志)时才加。**默认不加权限门控;要加必须在注释里写明"本地场景下为何仍需要"。**

- **数据持久化:echo-agent-cli(EKO)不需要 SQLite。** 本地个人 cowork 场景,对话历史/记忆用文件或内存实现即可。**禁止**给 echo-agent-cli 引入或保留 SQLite 依赖(`SqliteStore` / `SqliteConversationStore` / echo-state 的 `sqlite` feature 在 CLI 侧不启用);也**禁止**把"SQLite schema 变更/迁移/前端契约"当作成本或风险来反对改动——本项目尚处开发阶段,无迁移、无兼容负担(见"代码清理"节,过时代码/旧 schema 可直接删)。echo-state 的 `sqlite` feature 仅供框架其它复用方,**echo-agent-cli 不启用**。

- **多模式功能对等:TUI 与 GUI 是功能完全一样的 Agent 完全体,只是交互方式不同。** 参考标杆:Claude Code 是纯 TUI 产品,广受全球开发者欢迎——证明 TUI 能成为主力 Agent 交互形态。因此 EKO 的 TUI、GUI(以及 CLI/channel)必须**功能对等**:任何一方有的能力(复杂任务/plan/subagent/任务运行时/工具/HITL/记忆/附件…),其它方也应有。**禁止以"某模式不需要"为由拒绝给该模式接入能力。** 代码里若出现"X 模式 doesn't use Y"之类的注释/None 传参,那是**待补的缺口,不是产品定位**——不得把它当成"该模式刻意不要"来解读、更不得当成反对接入的理由。各模式共享同一套核心能力(如阶段 2 的 `drive_chat` 统一驱动),差异只在渲染/事件层。

> 历史教训:agent 看到 `main.rs` 里 `task_runtime_store — TUI doesn't use the task runtime`(传 None)+ TUI 固定 Chat 模式,就推断"TUI 是轻量交互终端、刻意不接 TaskRuntime",并据此建议取消 TUI 的 complex 接入。这是把**当前的缺口**误读成**产品定位**。用户澄清:TUI 目标是与 GUI 功能对等的完全体(对标 Claude Code)。定位已固化于此,不得再把"TUI 不需要 X"当理由。

> 历史教训:agent 多次把"动 SQLite schema/迁移"当作反对 DomainProfile 合并等改动的理由,又在清理 SQLite 死字段时把 `SqliteStore`/`SqliteConversationStore` 当"活代码保留"——都是没记住"echo-agent-cli 不需要 SQLite"这一定位,退回到线上服务的迁移思维。定位已固化于此,不得再犯。

> 历史教训:曾有一批安全 commit 给 `create_terminal` / `connect_mcp_server` 加了 `require_full_auto` 门控,导致默认 `default` 权限下终端打不开、MCP 连不上——这是把"线上服务防 XSS"的思维错误套到本地桌面应用上。这类门控已移除,新增功能不要再重蹈覆辙。

### 统一术语:只有 Subagent,没有 Worker(强制)

**`echo-agent` 和 `echo-agent-cli` 在产品模型、领域模型、运行时模型和代码术语中都只有 `Subagent`,没有 `Worker` 概念。** 标准关系是 `TaskRun → PlanTask → SubagentRun`;执行 PlanTask 的角色、实例、并发槽位和事件都统一归入 subagent 语义。

- **禁止新增 worker 术语**:类型、trait、结构体字段、函数、模块、变量、工具参数、事件、日志、注释、文档和 UI 文案均使用 `Subagent` / `subagent`,不得新建 `Worker` / `worker` 命名。
- **内部实现也不例外**:不得以"worker 只是线程池/调度器内部叫法"为由保留两套术语。执行槽、并发许可、运行句柄等若服务于 subagent,也使用 `subagent` 命名。
- **随手清理是强制要求**:后续优化、迭代、修复过程中,只要在所触及的 `echo-agent` / `echo-agent-cli` 代码或文档中遇到遗留 `worker` 命名,必须在同一改动中顺便迁移为 `subagent`,并同步修改调用点、序列化字段、事件、测试和文档;不得继续扩散旧术语。
- **外部固定协议例外**:仅当第三方 API / 协议的固定名称无法修改时,可在最小适配边界保留其原始 wire name,但内部必须立即转换为 subagent 术语,并写明这是外部兼容字段,不是本项目概念。

> 历史教训:项目曾把产品层称为 Subagent,同时在执行器、池、事件和注释里继续使用 Worker,造成两套心智模型并存,也让后续方案误以为存在独立的 worker 层。现在统一删除该区分:它们都是 Subagent。

---

## 关键决策:先调研业界优秀实现,不要闭门造车(强制)

**背景**:曾发生过 agent 凭直觉设计架构(如把"plan 是否被批准"塞进 run 运行时状态机,做出 13 状态的复杂状态机),结果与 Claude Code / Codex / Cursor / Devin 等成熟系统的共识相悖(它们都把 plan 当 artifact、用 prompt/权限驱动批准、不靠运行时状态机锁),引入了本可避免的系统性 bug。这类问题不是"代码写错",而是"没看过别人怎么做就自己拍"。

**规则:遇到关键决策(架构选型、状态机设计、API 形状、Agent 编排模式、数据流模型等),动手前必须先搜索调研业界成熟实现,再给出方案。**

- **必查的参考实现**(按相关度选):
  - **Claude Code**:agent 编排、plan mode、subagent、TodoWrite、工具/权限模型、skills 机制。注意它的 plan mode 是 prompt 注入而非运行时强制——很多"看起来需要状态机"的事,成熟做法是 prompt/行为驱动。
  - **Codex (OpenAI)**:非交互事件流(`codex exec --json` 的 `item.in_progress/completed/failed`)、任务生命周期、sandbox 权限模型。
  - **Cursor / Devin**:plan-then-execute 模式、plan 作为可编辑 artifact、background agent、approval gate 设计。
  - 其它按问题相关度补充(Windsurf、Copilot Workspace、LangChain plan-and-execute、SWE-bench 上的高rank方案等)。
- **怎么做**:用 WebSearch / web reader 查官方文档、设计博客、逆向分析文章、GitHub issue。提炼**跨系统的共性模式**(多个独立实现收敛到同一种做法 = 强信号),再结合 EKO 本地个人助理的定位(见上节)取舍。
- **输出要求**:在给用户的方案/spec 里,明确写出"参考了哪些实现、它们怎么做、本项目为何这样取舍"。不允许只给结论不列依据。
- **不是每次都查**:只有"关键决策"要查。修个明显 bug、改个文案、加个测试不需要。判断标准——**这个决策错了会影响多处、且业界已有成熟做法**,就要查。

> 反面教训:那次 13 状态机的 bug,如果动手前先花 10 分钟查"Claude Code / Codex 怎么做 run 状态机",就会发现它们根本没有"Planning/AwaitingApproval/Ready"这种 plan-审批-状态,会直接避开整个 bug。调研成本远低于返工成本。

---

## 关键决策:这个功能该放在框架还是应用?(强制)

**背景**:本仓库 `echo-agent`(通用 Agent 框架)和 `echo-agent-cli`(EKO 应用)职责分明——框架是可被其它项目复用的通用能力,应用是 EKO 的产品逻辑。但曾出现把 **EKO 专属概念**(本地桌面助手才需要的多资源信号量、approval gate、特定 UI 投影表)硬塞进通用框架的提案,代价是框架被产品逻辑污染、复用性下降,且与"通用框架应保持轻量"的业界共识相悖。

**规则:动手写任何"能放在框架、也能放在应用"的功能前,必须先回答这个问题——它属于框架层还是应用层?想清楚再动手。**

判断标准:

- **放框架的条件**(全部满足):这是**任何**用 echo-agent 的 agent 都可能需要的通用能力(不依赖 EKO 的产品决策)、与具体业务/产品形态无关、放在框架后能被多个项目复用。
- **放应用的条件**(满足任一):只服务于 EKO 的产品形态(本地桌面助手)、依赖 EKO 特有的产品决策(本地优先、交互式审批、特定 UI 字段)、换一个产品就不成立。
- **拿不准时的默认**:放在**应用层**。应用层下沉到框架容易,框架污染后清理难。先在应用层跑通、确认是通用需求,再下沉——这符合"YAGNI"。
- **把应用概念塞进框架的代价**:框架体积膨胀、被特定产品的状态机/并发模型/数据模型绑架,新项目复用时要先剥掉一堆无关代码。这和"过度加权限门控"一样,是典型的过度设计。

### 删除框架代码的判定:echo-agent 不服务于 echo-agent-cli(强制)

**echo-agent 是独立的通用框架,不是 echo-agent-cli 的私有库。** 框架 API 的存在不由 echo-agent-cli 是否调用决定。

**规则:删 echo-agent(框架)任何代码前,判定"死"的标准必须是"框架内部 + 所有合理复用方都不需要",而不是"echo-agent-cli 没调用"。**

具体:

- **❌ 错误判定**:"echo-agent-cli 没用 SqliteStore/没调 HybridCompressor → 框架该删它"。这是把应用当框架的唯一消费者。通用框架提供多个 `Store` 实现(FileStore/SqliteStore/InMemoryStore)、多个压缩器(SlidingWindow/Summary/Hybrid)是**正常的框架设计**——给不同复用方按需选。echo-agent-cli 选 FileStore 是 CLI 的事,不构成删 SqliteStore 的理由。
- **✅ 正确判定**(删框架代码须满足其一):
  1. **框架内部也无人构造/调用**(`#[allow(dead_code)]`、只在 `#[cfg(test)]`、被永远 false 的 guard 挡住),且**不是合理的对外 API**(无 pub、无 doc、无 doctest、非 trait 的多实现之一)。
  2. **被新实现明确取代且新实现覆盖旧的全部能力**(如旧 HumanGate 被新审批路径取代)——此时删的是"框架内部的旧机制",不是"框架对外 API 的一个选项"。
  3. 框架自身的 bug/冗余(如重复的内部 helper)。
- **拿不准时的默认:保留**。框架 API 删了,复用方(现在或将来)的代码会断;留着至多多占一点编译时间。YAGNI 在框架层是**反向**的——对内私有死代码删,对外 pub API 谨慎删。
- **判断动作**:删框架 pub 类型/fn/feature 前,先 grep **整个 echo-agent 仓库**(不只当前 crate)确认无构造点/调用点;再判断它是不是"trait 的多实现之一 / 合理对外选项"——是的话,**即使当前无调用也保留**(它是框架的能力菜单,不是死代码)。

> 历史教训:agent 多次因"echo-agent-cli 没调用"就判定 echo-agent 的 pub API(如 HybridCompressor、SqliteStore)可删,或把"CLI 不用 SQLite"当成"框架该清理 SQLite 实现"。都是把应用当框架唯一消费者。echo-agent 服务的是**所有**用它的项目,CLI 只是其一。

> 历史教训:曾有一份合并方案要把 EKO 的"四信号量并发模型(write/shell/llm/subagent)+ 12 张关系型投影表 + approval gate"全部收归框架 `TaskExecutor`/`TaskStore`,理由是"统一一套调度器"。但这些是编码 agent 才需要的并发约束和 UI 投影,通用框架根本没有——硬塞等于把产品逻辑写进框架。调研后发现这些在应用层做得更好,框架应只保留通用原语。

---

## 关键决策:动手前先查"是不是已经有了"(强制,防重复造轮子)

**背景**:本仓库代码量大、子 crate 多,框架和应用各自有任务/存储/调度/工具的实现,且历史迭代留下过**死代码**(编译进来、注册进来,但运行时永不触达)。曾有一份重构计划要把 `run_id / title / kind / agent_role / files / summary / Skipped / Paused` 等"新增"到框架 `Task` 结构——但这些字段**当时就已经存在**(`task.rs` 里早已写好)。计划是基于过时认知写的,差点做无用功。

**规则:动手新增任何东西(字段、函数、类型、模块、工具、表)之前,必须先在仓库里查清楚:这个东西是不是已经存在?在框架还是应用?是活的还是死的?确认后再决定是复用、扩展还是新建。**

具体怎么做:

- **先用 Grep / Glob 搜整个仓库**(不只当前 crate):按名称、按概念、按相邻命名风格搜。框架和应用常常各自有同名概念的不同实现(如框架的 `create_task` 工具 vs 应用的 `task_create` 工具),要分清哪个是活的、哪个是死的。
- **区分"定义存在"和"运行时可达"**:一个类型/工具可能编译进来甚至注册了,却永远不被调用(典型信号:`#[allow(dead_code)]`、只在 `#[cfg(test)]` 里用、调用点被某个永远为 false 的 guard 挡住)。动手前要确认你要改/删的东西是不是真在跑。
- **能复用就不新建,能扩展就不另起**:发现已有实现能覆盖需求时,优先用/扩展现有的,而不是并行造一套。这和本文件"代码清理:无需兼容,过时代码可直接删"是一条线的两端——发现重复就归一,发现过时就删。
- **框架和应用都要查**:别只看应用就以为"框架没有、需要新增"。应用有某个概念,不代表框架也需要;框架已经有的能力,应用不要重复实现。

> 反面教训:那次"新增 `run_id`/`title`/`kind`/`Skipped`/`Paused` 字段"的计划,如果在动手前对 `Task` 结构 grep 一次,5 分钟就能发现这些字段全已存在,整段 Phase 1 工作量(~2 小时)直接省掉。查的成本永远远低于造轮子的成本。

### 实现前门禁:先定分层,再证明没有重复(强制)

上面两条不是写方案时才考虑的原则,而是**每次实现前必须完成的代码门禁**。任何同时涉及 `echo-agent` 与 `echo-agent-cli` 的能力,动手前必须留下可核验结论:

**必须先想清楚框架层与应用层的定位;严禁重复造轮子。** 分层结论和全仓库重复性搜索缺一不可,不得以工期、迁移复杂度或“先留着以后删”为由跳过。

1. **先写分层判定**:明确列出“通用机制 / EKO 产品策略 / 适配边界”三部分。依赖 DAG、重试、取消、revision safe point 等跨产品成立的机制放框架;DomainProfile、reviewer 策略、worktree、文件权威、UI/TUI/CLI 投影等留应用。拿不准时先留应用,不得把 EKO 字段包装成“通用配置”后塞进框架。
2. **先搜完整仓库再新增**:按类型名、trait、字段、行为和调用路径同时搜索 `echo-agent` 与 `echo-agent-cli`;必须区分“已经定义”“已经注册”和“主路径真实可达”。没有完成这一步,不得新增类型、状态、validator、store 或 executor。
3. **严禁平行实现同一语义**:同一种动态 PlanTask 调度、DAG 校验、状态迁移或 revision 语义只能有一个权威实现。迁移阶段可以短暂有 adapter,但每个阶段都必须切换至少一条真实主路径并删除被替代逻辑;禁止只新增抽象、保留旧主循环,把删除工作无限推后。
4. **适配器必须保持薄且转换无损**:应用 adapter 只做类型转换、metadata 注入、产品 policy/hook 和调用框架服务;语义不同的字段(如可执行检查与语义验收)不得为了迁就旧结构而压平成一个字段,转换必须有 round-trip/字段级测试。adapter 不得重新拥有 ready frontier、DAG 主循环、通用重试/取消、死锁判断或第二套 plan validator。若 adapter 开始出现这些逻辑,说明分层已经失败,必须停下来重新设计。
5. **未完全收敛必须显式归档**:如果一次提交只能完成分阶段迁移,必须在 `docs/MASTER-PLAN.md` 写清已切换的权威路径、仍存在的重复、下一阶段删除目标。复杂度只能决定迁移顺序,不能成为长期保留双实现的理由。
6. **任务关系只有一个权威 API**:框架默认提供 `task_create/task_update/task_list`,EKO 在此基础上增加 `task_execute`;单 Task、Todo 列表和依赖 DAG 都属于同一个 revisioned TaskRun graph。`TaskPlan` 只能是可编辑/可审阅的版本化 artifact,`TodoItem` 只能是 UI 投影,不得各自拥有 store、状态机或执行器。旧的进程全局 `todo_write` 已由框架直接删除,不得重新引入;也不得重新引入 `plan_create/plan_patch/plan_execute` 或其它平行任务 CRUD。

---

## 三个项目的定位

| 项目 | 类型 | 定位 |
|---|---|---|
| **`echo-agent/`** | Rust crate (`echo_agent` v0.2.0) | **核心 Agent 框架**。生产级 Rust AI Agent 框架:ReAct 引擎、多 agent、记忆、流式、MCP、IM 渠道、工作流。是其他项目的基础依赖。 |
| **`echo-agent-cli/`** | Rust workspace + Tauri + 前端 | **应用层 = EKO**。基于 `echo-agent` 的 CLI 与桌面应用(品牌名 EKO)。包含子模块:`echo-agent-app-core`(应用核心逻辑、任务/会话/状态)、`web-frontend/`(React+TS+Tailwind v4+Zustand 前端)、`src-tauri/`(Tauri 桌面壳)。 |
| **`echo-website/`** | Vite 站点 | **官网 / 展示站点**。独立的 Vite 驱动网站,与 agent 运行时无依赖关系。 |

> 顶层 `eko-workspace` 仓库只管理跨项目文档、协作规则和 submodule 提交指针。三个目录仍各自是独立 git 仓库(`git -C echo-agent` / `git -C echo-agent-cli` / `git -C echo-website`),子仓库代码必须在所属子仓库内提交。

---

## 代码、文档与示例同步(强制)

1. **任何代码修改都必须同步检查并修改对应的文档、`examples` 代码示例以及 `echo-website` 相关内容**,确保实现、示例和对外说明始终一致。
2. **凡是涉及项目架构、关键组件或重点代码的新增与修改，都必须补充或更新对应文档和代码注释**。注释应重点说明设计意图、关键约束和非显而易见的逻辑，不写只是复述代码表面行为的无效注释。
3. **提交前必须明确检查文档、`examples` 和 `echo-website` 是否需要同步更新**。某项与本次改动无关或确实不适用时，必须在提交说明、PR 描述或任务交付说明中写明原因，不得静默跳过。
4. **架构变更必须使用 ADR(Architecture Decision Record)记录**。ADR 至少应包含背景、候选方案、最终决策、取舍理由和影响范围；优先遵循所属子仓库已有的 ADR 约定，没有现有约定时放入该子仓库的 `docs/adr/` 目录。
5. **`examples` 必须纳入可执行的编译或测试链路**，并作为提交前验证的一部分，防止示例与真实 API 长期漂移。新增或修改示例时，必须同步补齐对应的编译或测试入口。

---

## Rust 编码硬性约束(最高优先级)

### 1. 字符串处理:UTF-8 安全,禁止字节级截断

**背景**:Rust `str::len()` 返回**字节数**而非字符数;用字节索引切片(`&s[..n]`、`&s[n..]`)在遇到中文、emoji 等 UTF-8 多字节字符时会切到字符中间 → **panic**。

**规则:处理任意可能含中文/emoji 的字符串时,必须用字符迭代器,全部使用 `take`,禁止字节截断。**

正确写法(本项目既有 pattern,大量在用):
```rust
// 截断到 N 个字符(Unicode 标量值),永不会 panic
let preview: String = s.chars().take(N).collect::<String>();
// 判断长度用字符数
if s.chars().count() > N { ... }
// 需要"前 N 字符 + 后缀"
format!("{}...", s.chars().take(100).collect::<String>())
```

错误写法(**禁止**,会在中文/emoji 上 panic):
```rust
&s[..100]      // ❌ 字节切片,中文 3 字节会切到中间
&s[100..]      // ❌ 同上
s.len() > 100  // ❌ 这是字节数,不是字符数(判断可改用 chars().count())
```

参考既有正确实现:`echo-agent/src/agent/snapshot.rs` 的 UTF-8 安全截断、`trace/mod.rs` 的 `chars().take(80)`、`react/run/execution.rs` 的 `chars().take()`。

### 2. 禁止任何会导致系统 panic 的 API(最严,全部禁止)

**规则:绝不使用任何在异常输入下会 panic 的 API。** 处理外部/不可信输入(用户文本、文件内容、网络数据、工具结果、配置)时尤其严格;内部逻辑同样遵守。

禁止清单 → 必须用安全替代:

| 禁止 | 安全替代 |
|---|---|
| `.unwrap()` | `.ok_or(...)?` / `unwrap_or(default)` / `unwrap_or_else(\|\| ...)` |
| `.expect("msg")` | 同上,带明确错误处理 |
| `arr[i]` / `v[i]`(可能越界) | `arr.get(i)` 返回 `Option`,再处理 |
| `&s[..n]` 字节切片 | `.chars().take(n).collect()`(见上) |
| `"123".parse::<i32>()` 不处理错误 | `.parse().map_err(...)?` 或 `parse::<i32>().unwrap_or(0)` |
| 整数运算可能溢溢出 | `checked_add` / `checked_mul` / `saturating_*` / `wrapping_*` |
| `panic!` / `unreachable!` / `todo!` | 返回 `Result` 或处理该分支 |

**审查尺度**:对每一处 `unwrap`/`expect`/直接索引都质疑。只有"经过前置校验、逻辑上 100% 不可能失败"的极少数内部场景才可酌情保留,且须有注释说明为何安全。

---

## 提交与推送(GPG 签名 + 验证)

### 提交/推送:跳过 GPG 签名

**背景**:本机全局 `commit.gpgsign = true`,但 GPG 签名在当前环境会反复失败。每次提交不要让 git 反复尝试签名再失败。

**规则:所有 `git commit` 必须显式关闭 GPG 签名。**

```bash
# 顶层 superproject 只提交跨项目文档、规则和 submodule 指针:
git add AGENTS.md README.md docs/ .gitmodules echo-agent echo-agent-cli echo-website
git -c commit.gpgsign=false commit -m "..."

# 子仓库代码在对应子仓库目录内单独提交:
cd echo-agent        # 或 echo-agent-cli / echo-website
git add ...
git -c commit.gpgsign=false commit -m "..."
# 推送同理(推送本身不签名,正常 push 即可):
git push
```

> 注意:每个子项目是独立仓库,提交前务必确认当前仓库。跨仓库改动必须先提交并推送子仓库,再提交顶层 superproject 中的 submodule 指针;不得只提交指向未推送子提交的指针。

### 验证分层:迭代快检 + 提交门禁 + 条件矩阵

**规则:提交前必须完成与改动范围匹配的验证,所有已执行命令必须全部通过(零失败、零报错)才能提交。** 不再要求每次任务重复执行默认 feature、all-features 和逐 feature 三套高度重叠的全量矩阵。

**关键:遇到任何失败 —— 包括"预先存在的失败""与我改动无关的失败""测试本身写错的失败" —— 都必须修复,不得跳过、不得绕过、不得用 `--no-verify` 或只跑子集蒙混。** 否则测试套件会逐渐失效,项目无法维护。具体:

- 测试失败 → 修测试或修被测代码,直到对应测试全绿。
- 编译警告/错误 → 全部消除(`cargo clippy` / `cargo test` / `tsc` 零错误)。
- **格式化(强制,CI 依赖)** → `cargo fmt --all` 跑过**且** `cargo fmt --all -- --check` 退出码 0(零 diff)。CI 会跑 fmt 检查,**fmt 不干净的提交会让 CI 红**——本地必须先 fmt 再提交,不能依赖 CI 兜底。前端同理(`prettier --check` / 项目脚本)。
- 只有"全部通过"这一个状态可以提交;不存在"大部分通过就先提交"。

#### 1. 迭代快检(开发过程中)

开发过程中优先跑最小相关集合,不要每改一处就重跑全 workspace:

```bash
# Rust:按改动 crate / 测试名缩小范围
cargo test -p <crate> <test_name>
cargo check -p <crate>

# echo-agent 的默认 workspace 快检
cd echo-agent
cargo test --workspace --locked

# 前端:按文件或测试名缩小范围
cd echo-agent-cli/web-frontend
npx vitest run <test-file>
```

快检只用于迭代,不能替代下面的提交门禁。

#### 2. 提交前门禁(强制)

`echo-agent` 与 `echo-agent-cli` 都是真正的 Cargo workspace。`cargo ... --workspace` 会覆盖 workspace 全成员,**不再逐 crate 重复执行命令**,也不再额外跑一遍默认 feature 的 `cargo check/test/clippy`。

`echo-agent` 提交前执行:

```bash
cd echo-agent
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo clippy --workspace --lib --bins --all-features --locked -- \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D clippy::panic \
  -D clippy::unreachable
cargo test --workspace --all-targets --all-features --locked
cargo check --workspace --lib --no-default-features --locked
```

`echo-agent-cli` 提交前执行:

```bash
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
```

> `cargo test` 和 `cargo clippy` 都会完成编译,因此不再机械追加一个相同 feature 组合的 `cargo check --workspace`。

#### 3. 条件矩阵(仅相关改动强制)

以下验证只在改动触及对应风险面时执行:

- 修改 `Cargo.toml`、feature 定义、`#[cfg(...)]` 分支、可选依赖或跨 crate 公共 API → `echo-agent` 额外逐个编译根 crate 的独立 feature:

```bash
cd echo-agent
for feature in sqlite subagent human-loop mcp lsp a2a git database rag chart web media; do
  cargo check -p echo_agent --no-default-features --features "$feature" --locked || exit 1
done
```
- 修改 `echo-agent-cli/src-tauri/`、`src/tauri/`、GUI feature 或相关依赖 → 额外执行:

```bash
cd echo-agent-cli
cargo check --no-default-features --features gui --bin echo-agent-tauri
cargo test --no-default-features --features gui
```

- 修改 `echo-agent-cli/web-frontend/` → 额外执行:

```bash
cd echo-agent-cli/web-frontend
npx prettier --check "src/**/*.{ts,tsx}"
npm test
npm run build
```

如果某个条件矩阵在当前环境无法运行(例如缺系统依赖),必须在提交说明中明确列出未验证项和原因,不得静默跳过。

#### 为什么不再默认跑逐 feature 矩阵

**这是历史血泪教训,违反必导致测试套件失效。** echo-agent 根 `Cargo.toml` 现在同时是根 package `echo_agent` 和 workspace root,显式包含 7 个子 crate(`echo_core`/`echo_macros`/`echo_execution`/`echo_integration`/`echo_tools`/`echo_state`/`echo_orchestration`)。迁移前根 manifest 只是普通 package,`cargo test --workspace` 实际只覆盖根 crate,曾让子 crate 测试编译错误隐藏整个提交周期。

现在 workspace 已经原生解决全成员覆盖问题。逐 feature 矩阵仍能发现 feature 隔离错误,但与 all-features 门禁高度重叠且代价很高,所以改为只在 feature 拓扑、条件编译或公共 API 变化时运行,并继续由 CI/专项审计兜底。

**只有所有适用的提交前门禁和条件矩阵全部通过,才执行 `git -c commit.gpgsign=false commit`。**

### 按磁盘压力 cargo clean,保留增量编译缓存

**背景**:Rust 编译产物（`target/` 目录）极其占空间。本项目两个 Rust 子仓库多次编译及 feature 矩阵验证，累计可能占用大几十 GB。本机磁盘空间有限（约 460GB），不清理会很快耗尽。

**规则:不再每次任务结束强制 `cargo clean`。** 默认保留 `target/` 增量缓存,避免下一任务从零编译。提交前先按需检查:

```bash
df -h .
du -sh echo-agent/target echo-agent-cli/target 2>/dev/null
```

满足任一条件时再清理:

- 可用磁盘空间低于约 50 GiB。
- 两个 `target/` 合计超过约 60 GiB。
- 构建缓存损坏、工具链大版本切换,或用户明确要求清理。

清理时只清理造成压力的 workspace;只有两个仓库都明显膨胀时才分别执行 `cargo clean`。`cargo clean` 必须放在全部验证之后,清理后不再重复编译验证。

---

## 代码清理:无需兼容,过时代码可直接删

**规则:本项目不需要保留向后兼容性。** 任何你认为过时、被替代、无人调用的代码(函数、模块、类型、feature gate、注释里提到但已不存在的路径),都可以**直接删除**,不需要为了"可能有人依赖"而保留。

具体:

- 删除优先于保留:如果一个函数被新实现取代、一个 enum 变体不再构造、一个 feature gate 下的代码不再需要,**删掉它**,不要留着加 `#[deprecated]` 或 `#[allow(dead_code)]`。
- 不要为"可能将来还会用"的代码留死路径。YAGNI —— 删了,将来真需要再写(那时上下文更清楚)。
- 重构时,如果发现两套做同一件事的机制(例如"旧的 HumanGate 路径"和"新的审批路径"),确认新的能覆盖需求后,**删除旧的**,不要双系统并存。
- 删除时连带清理:调用点、测试、文档、导入。删完必须按上节执行提交前门禁;若涉及 feature/`#[cfg]`/公共 API,再执行条件 feature 矩阵,确保没有遗漏的引用。
- 删除是有价值的贡献:死代码和双系统是最大的维护负担。看到就删,不要"留着以防万一"。

> 例外:如果一段代码你**不确定**是否还有人用(例如被反射式调用、被外部插件引用),先 grep 确认无调用点再删;确认不了的,在 PR 描述里标注"疑删,请 review",而不是默默保留。

---

## Worktree 并行开发与合并规范

**背景**：本项目使用 git worktree 在 `echo-agent/.worktrees/` 和 `echo-agent-cli/.worktrees/` 下创建隔离工作区，实现多 feature 并行开发。worktree 模式高效但也有**多个容易踩的坑**——以下规则是实战中沉淀的，必须遵守。

### 1. Cargo.toml 路径：合并前必须改回相对路径

**坑**：在 worktree 里开发时，为了让 `echo-agent-cli` 编译到 worktree 里未发布的 `echo-agent` 改动，会临时把 `Cargo.toml` 的 `path` 指向 worktree 的绝对路径（如 `path = "/Users/.../.worktrees/feature/xxx"`）。这个路径**只在作者本机存在**，合并后任何其他人 / CI / fresh clone 都无法构建。

**规则**：
- worktree 开发期间可以用绝对路径或 symlink 临时让编译通过。
- **合并到 main 之前**，必须把所有 `Cargo.toml` 中的 `path` 改回**正确的相对路径**：
  - `echo-agent-cli/Cargo.toml` → `path = "../echo-agent"`
  - `echo-agent-cli/echo-agent-app-core/Cargo.toml` → `path = "../../echo-agent"`
- 检查方式：`grep -rn "worktrees\|/Users/" */Cargo.toml`，确保零命中。

### 2. .gitignore 中的 .worktrees/ 不能删

**坑**：`.worktrees/` 目录如果在 `.gitignore` 里被意外删除，后续 `git status` 会把整个 worktree 目录当作未跟踪内容，`git add .` 会把它提交进去。

**规则**：`.gitignore` 必须包含 `.worktrees/`。合并前检查 diff，如果 `.gitignore` 有删除该行的改动，必须还原。

### 3. 合并前必须先 merge main

**坑**：worktree 分支从 main 创建后，main 可能已经有新提交（如另一个 feature 先合并了）。直接 squash merge 会**丢失 main 上的新功能**——diff 看起来像"删除了 main 新加的代码"。

**规则**：
- 合并前，在 worktree 分支上先执行 `git merge main`（比 rebase 简单）。
- merge 时 `commit.gpgsign=true` 的仓库会卡住——用 `git commit --no-gpg-sign --no-edit`。
- merge 后验证 main 的改动还在：grep 关键函数确认未丢失。
- **只有 merge main 成功后**，才能执行 squash merge 到 main。

### 4. squash merge 后的分支删除用 -D

squash merge 不创建 merge commit，`git branch -d` 会报 "not fully merged"。用 `git branch -D feature/xxx` 强制删除——改动已通过 squash 在 main 上了。

### 5. worktree 清理顺序

```bash
git worktree remove .worktrees/feature/xxx --force
git worktree prune
git branch -D feature/xxx
rm -f .worktrees/feature/echo-agent  # 删除临时 symlink
```

### 6. 跨仓库依赖的合并顺序

`echo-agent-cli` 依赖 `echo-agent`。如果框架加了新字段而 CLI 用了它，**必须先合并 echo-agent 到 main，再合并 echo-agent-cli**——否则 CLI 合并后编译失败。

---

## 上下文管理:阶段归档 + 动态窗口(防上下文污染/丢失)

**背景**:长会话里 Agent 会出现注意力涣散、指令遵从下降(上下文污染);但频繁换窗口又丢前因后果(上下文丢失)。本仓库是大工程(多里程碑、跨子仓库、长重构),必须主动管理上下文,不能任其无限增长。

**核心判断:按"耦合度"决定是否同窗口连续,不是按"任务大小"。**

### 规则 1:强耦合任务 → 同窗口连续推进
- 后一个任务**高度依赖**前一个任务的执行结果/调试日志/失败方案 → 留同窗口。
- 典型:写核心逻辑 → 跑测试报错 → 根据报错修 bug → 优化。这些"隐性知识"(报错栈、试过的失败方案)换窗口后同步成本极高。

### 规则 2:弱耦合/原子化/确定性任务 → 换新窗口(或重读文档恢复)
- 任务目标明确、不需要前一个任务的"中间推理过程" → 果断换窗口。
- 典型:主线写完补单测、抽硬编码到 config、删死代码。只需静态代码上下文,不需历史对话。
- **换窗口时必须靠文档恢复全局,不靠记忆**——重读 `docs/MASTER-PLAN.md` + 对应 spec/plan。

### 规则 3:阶段归档(Checkpointing)—— 既是上下文管理,也是状态保全
- **每个阶段完成 + 提交 git 后,立刻更新 `docs/MASTER-PLAN.md`**:状态总表、决策记录、当前在哪、下一步做什么。
- MASTER-PLAN 是**跨上下文的单一事实源**——无论这个会话继续还是新开会话,读它就能恢复全局。
- 旧窗口的冗长报错/废弃思路不进 MASTER-PLAN(那是干扰),只进"高密度知识压缩"(决策 + 状态 + 依据 file:line)。

### 规则 4:小步快跑 + 主动断代
- 同窗口内完成一个"能跑通的特性"或"修一个特定 bug"(约 5-10 轮)。
- 编译通过 + 测试绿 + **git 提交** = 阶段目标 100% 达成 → 这是断代点。
- 断代后:要么更新 MASTER-PLAN 换窗口,要么重读 MASTER-PLAN 在同窗口"刷新"上下文再继续。

### 规则 5:高风险步骤特别审慎
- 动**读写权威/核心数据流/聊天主路径**等高风险改动时,优先在新鲜上下文做(重读文档恢复,不靠长会话记忆)。
- 这类步骤出错代价高(聊天 run 坏、数据丢失),值得用清醒上下文换较低出错率。

### 规则 6:主动提醒换窗口(用户感知)
- Agent **判断该换窗口时(会话已长 + 即将做高风险/弱耦合任务),必须主动明确告诉用户**,不能默默继续或默默停。
- 提醒要说清三件事:(1) 建议换窗口的理由(会话长/步骤高风险/任务弱耦合);(2) 当前状态已归档到 MASTER-PLAN(换窗口不丢上下文);(3) 下一步是什么(新窗口读 MASTER-PLAN 接续)。
- **不替用户决定**——给出"继续本窗口(重读文档刷新)"和"换新窗口"两个选项,让用户拍。
- 典型话术:"当前会话已较长,接下来是写权威切换(高风险)。建议换新窗口——MASTER-PLAN 已记好状态,新窗口读它就能接续。你要继续还是换窗口?"

> 实操:本仓库的 `docs/MASTER-PLAN.md` 已按此规则维护——每个里程碑/阶段推进都更新它。它是"动态窗口策略"的落地:同窗口靠它刷新,换窗口靠它接续,无需人工搬运对话历史。Agent 换窗口提醒是这条策略的"用户接口"——不提醒,用户就无法在正确时机开新窗口。

---

## 工作环境注意事项

- **Bash 工具每次是新 shell**,工作目录不持久。涉及路径的命令必须用绝对路径或在命令开头 `cd` 到目标子仓库目录(否则会出现 "could not find Cargo.toml" / "No such file" 类错误)。
- Rust crate 名用下划线:`echo_agent`(不是 `echo-agent`),`cargo check -p echo_agent`。
- 代码风格:与周围代码一致(命名、注释密度、惯用法)。注释和 commit message 可用中文。
- 用户指令优先级 > 本文件 > 技能 > 默认行为。当用户在对话中给出与本文件冲突的具体指令时,遵循用户当次指令。
