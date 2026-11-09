---
title: 完整回复末尾的工作空间分叉设计
artifact: design
carrier: markdown
---

# 完整回复末尾的工作空间分叉设计

## 问题与目标

本文件细化[GUI 产品收敛总设计](../eko-gui-product-convergence/design.md)中已批准的回复分叉目标，属于跨仓阶段工程材料。正式产品说明和应用 ADR 仍由 `echo-agent-cli` 维护；当前协调状态仍以[唯一交接入口](../../handoffs/eko-current-handoff.md)为准。

用户要求把分叉入口放在每轮完整回复末尾，点击后选择“当前工作空间”或“新 Worktree”；顶部“新建支线对话”入口已经退出，既有 Side Conversation 历史和共享能力继续保留。2026-10-27 用户进一步明确：“不要，从上一次 Git 提交开始。”因此新 Worktree 使用源项目当前已提交的 `HEAD`，不复制未提交、未跟踪或被忽略的文件；这些文件留在原目录。

两种形式都复制同一段已提交对话历史，创建等待用户继续的新普通会话。区别是执行文件位置：当前工作空间共用原项目目录，新 Worktree 使用独立 Git 目录和对应的新工作空间。源码修改、文档校验或组件截图不等于新版桌面已加载或真实 SSH 已通过。

## 目标行为与范围

一次可分叉的回合从一个可见用户输入开始，结束于下一可见用户输入之前的最终完整助手回复。轮内工具调用、中间助手输出和工具结果都是该回合的历史，不各自拥有分叉入口。内部协作消息和独立工具历史展示不成为普通回复入口。界面每轮只在最后一条可分叉回复之后显示一个按钮；流式、待结算和没有最终正文的回合不能绕过后端校验。

点击按钮打开同一选择菜单。菜单说明当前工作空间共用文件，新 Worktree 从最近一次 Git 提交开始。非 Git、没有可用提交或无法确认项目环境时，仅新 Worktree 不可用并说明原因，普通分叉仍遵守原对话准入。detached HEAD 有可用提交时不被错误判为无 Git；实际以解析出的 commit OID 为基点，而非在后继创建时重新解释可能已变化的 `HEAD`。

成功后新会话出现在准确目标工作空间中并保持空闲，不自动发出下一条用户输入，不启动旧任务或继承旧执行许可。原会话、原历史、原目录与运行资源不被修改或切走。当前窗口仍处于本次请求的来源和导航代际时才自动打开结果；用户已经切换工作空间、会话或页面时，迟到结果只保留其真实资源事实，不抢夺新焦点。

本结果不新增 Side Conversation 协作协议、任务 CRUD、store 或执行器，不恢复顶部入口，不克隆旧 `TaskRun/PlanTask/SubagentRun`、Goal、控制 owner 或源会话专属模型覆盖。新会话按其目标工作空间的既有工厂、配置和权限规则装配；复制历史中的目标文字不等于复制一个仍可执行的 Goal。普通编辑/重新生成的“回合之前分叉并重发”行为不改变。

## 分层与复用依据

分叉目的地、项目位置和 GUI 导航属于 EKO 应用策略。Framework 继续拥有 canonical ConversationStore、消息和 managed import 所需通用合同；本结果不要求新增 Framework 工作空间概念或 SDK API。

| 已有路径 | 本结果复用及最小补齐 |
| --- | --- |
| `ChatPanel.handleFork`、`MessageBubble`、`conversationStore.forkCurrent` | 沿现可见用户回合定位和迟到 fence 增加每轮唯一入口与两种目的地，不造第二套消息树。 |
| Tauri `branch_prefix/branch_conversation` | 保留 canonical 完整回合、内部消息过滤、source replacement admission 与新 conversation 的 managed import；补目标工作空间的薄协调接缝。 |
| Tauri `ScopedWorktreeControl/create_worktree`、应用 `tasks/task_runtime/worktree.rs` | 复用原项目 executor、FileSystem、Git 锁、创建和查询；必要时把现有创建逻辑提取为应用内可组合入口，并让原 GUI 入口使用它，删除被取代的重复逻辑。不得复制第三套 `git worktree add` 算法。 |
| `AppState.create_project_workspace_owned`、WorkspaceRegistry、WorkspaceRuntimeHost | 原工作空间登记与位置/代际权威创建目标，原工厂取得该目标的文件、命令和终端环境；不把整个源工作空间改关联来伪造单个分叉隔离。 |
| `managed_conversation.replace_and_resume` 与原 import/checkpoint CAS | 在准确目标 runtime 导入同一 canonical 前缀，保持恢复与新会话空闲；不转移原运行控制身份。 |
| `attachments`、`prepared_turn` 的原物化、文件引用和 artifact reader | 迁移目标继续读取所必需的产品自有附件/原文/输出引用，不放宽目标 reader 的根范围，不复制源运行账本。 |

标准库、现有序列化和平台 Git/OpenSSH 足以承接协调与文件复制。已安装 FileSystem、project resources、registry、managed import 和原子物化能力继续承担实际操作。最小自定义部分是目的地协调、跨工作空间可读引用转换、准确结果回执及原 UI 选择流程，不新增依赖、通用调度器或全局补偿状态机。

只读事实以 CLI `0ebd8984be26affc32a130df7a26b128494d6c65` 的相关路径为基点，后继实际字节与 SHA 见根 `.echo-semantic/local/reply-fork-workspaces-design-20261027/source-facts.json`；它不是业务通过收据。主要事实仍为：原 branch 只能同工作空间，原 create_worktree 只返回 Git 目录 descriptor，尚未组成完整的新工作空间分叉。

## 同一来源快照与新会话

后端沿现 source managed replacement admission 取得可复制的 canonical 已提交前缀，仍排除内部消息，并检查该回合最后一条是非空助手正文且没有未完成 tool call。不能仅凭前端 ordinal、`executionCompleted` 或按钮禁用状态证明回合完整。稳定请求身份同时绑定来源 workspace/generation、conversation、实际前缀边界与目的地形式，防重复提交或旧来源回执被错认。

两种目的地使用同一前缀选择规则。当前工作空间形式沿原 runtime 创建新 conversation；新 Worktree 形式先固定准确目标环境和基点，再沿原项目 Git、工作空间登记和目标 managed import 完成组合。源 admission 的持有与释放沿原生命周期；不缩短结算等待、不直接丢弃活跃 future，也不因要复制历史而暂停、取消或迁走原 Subagent。

复制保留 canonical 消息顺序、正文、原 tool call/result 和必要附件。目标 conversation 的消息身份重新建立；原 `message_id/execution_steps/execution_rounds` 等来源 UI 执行引用不被当作目标活动执行事实。历史工具输出仍是历史；没有准确状态证据时沿既有恢复呈现明确未知，不借用源运行的 live owner 或把消息复制解释成一次新执行成功。

源 transcript、目标 managed import 和目标 checkpoint 各由原 owner/CAS 维护。导入提交但回复丢失、checkpoint 尚未完成等情况，先查询和恢复已提交事实，不能重复创建 conversation、重放用户输入或绕过原 epoch/version 核验。

## 项目、SSH 与引用绑定

新 Worktree 在来源实际项目环境中创建，使用操作已固定的源 commit OID。源码、Git、命令、终端和产物必须随后从目标工作空间的同一 project location/execution location 解析。已有目录重用与新的工作空间登记只接受本次真实资源身份；同名目录、分支或 workspace 已被其它请求占用时明确冲突，不覆盖、不清空。

本地创建沿原本地项目 owner。SSH 的 Worktree 创建继续使用来源已认证连接及其实际远端 Git/FileSystem/executor，来源 workspace 的连接映射和物理 owner 不转移。目标工作空间沿现有 `create_project_ssh_auth` 和 GUI 认证流程建立独立 draft `ProjectSshConnection` 与认证 PTY；系统 OpenSSH/agent 可以自动完成认证时仍沿原能力，否则仅在 GUI 中请求用户输入。目标 draft 的实际主机、账号、端口须与来源有效目标吻合，目标目录则是本次创建的远端 Worktree。

只有该独立 draft 可以交给原 `create_project_workspace_owned` 的连接转移流程，再登记目标 workspace。现 `reassign_project_ssh` 会移除 previous owner，创建失败还会关闭目标连接，因此不得把来源已绑定的 `connection_id` 传入，也不得把同一个连接 Arc 双重登记来伪造独立归属。目标创建失败、目标关闭或来源关闭分别只结算各自连接/操作；双方同时存在时原文件、命令和终端仍独立可用。这里补的是原应用连接 owner 的正确组合，不新增 SSH 协议、凭据 store 或自动执行权限闸。

断线、连接代际变化、独立目标认证未完成或远端检查失败时不回退本机。密码、私钥只由用户在 GUI 输入，分叉请求、日志、模型历史和工作空间持久元数据不保存它们。不把界面主动分叉套上 Agent 自动执行权限模式门控。

代码目录与产品数据根分开。新 Worktree 的未提交文件取舍不意味着可以丢弃聊天附件或长输入原文：它们是复制对话所需的产品数据，不是额外同步用户当前代码。跨工作空间时，源 canonical 前缀中可证明归属的产品自有附件、原始输入和工具输出引用必须在目标产品数据根中继续可读。沿原物化/复制路径校验完整 bytes、大小和 SHA，按准确目标 conversation 发布其引用；内联内容和多模态附件也保持内容与媒体类型。

仅转换已有合同可识别、归属可核验的产品资源描述符。不得对用户正文中的任意路径做字符串替换，不把源项目目录、整份 `.eko`、任务 journal、密钥、运行控制记录或所有未跟踪文件复制过去；也不得为了读取旧引用扩大目标 `read_artifact` 根范围。来源资源缺失、无法验证或无法无损转换时，报告具体资源并保留真实部分结果，不能只复制空预览后声称完成。目标成功导入后，来源会话/工作空间关闭或受支持的独立清理不得使目标已有副本失去原文；验证范围必须覆盖这一点。

## 重复、失败与恢复

界面一次点击沿原提交保护固定请求身份，重复 Enter/双击和对同一结果的重试复用它；用户明确再次发起独立分叉时才产生另一个请求。后端在各原 owner 中核对来源与已创建资源，重试只继续已证明属于该请求的目标，不依据名称猜测身份，也不因为观察者取消就释放仍在执行的 Git/import owner。

回执只包含各步骤已经发生的可验证事实：是否已创建 Worktree、准确 descriptor/commit、是否已登记 workspace、是否导入 conversation，以及是否被当前窗口打开；不把这些事实压成单一 success toast。复用既有创建/切换回执、registry、Git worktree 查询和 managed-import receipt 承接，不建立平行持久 store 或一套新的任务状态机。

若创建 Git 目录后查询 descriptor 失败，错误不代表未发生创建。若后继登记、资源迁移、导入或切换失败，保留已创建的目录/分支、工作空间和原始错误；当前请求可显示“已创建，尚未打开”并沿既有打开/导入入口继续。重新启动后，已完成登记的资源仍在原列表中；Git 创建早于登记的间隙也须能从原 Worktree 列表发现，而不是靠本轮内存变量宣称一切已回滚。无法确认原请求执行状态时先重读实际资源，不自动重发创建。

不能无条件 `remove_dir_all`、强制移除 dirty Worktree 或删除用户随后写入的文件。普通同工作空间分叉沿原对本次不可用新 conversation 的清理方式；跨资源分叉不擅自补偿删除新目录。只有原 owner 可以证明本次独占、未发表且无后继使用的临时物化数据，才沿其已有准确清理入口处理。所有原始记录、部分成功事实与恢复定位继续保留。

## 验收标准

1. 一轮含多条中间 assistant、tool call/result、内部消息和最终回复时，只有最后完整回复有一个入口；流式、未结算、半轮、内部及独立 tool 展示没有错误入口。菜单、键盘、焦点、浅深主题与宽窄窗口可用，既有 Side Conversation 历史和普通编辑/重生成不退化。
2. 两种形式复制完全相同的 canonical 前缀；源后续消息不混入，源会话保持原样。新会话空闲，不继承旧 Goal/TaskRun/Subagent/control owner 或源 conversation model override；目标装配取其原工作空间规则。
3. 本地真实 Git 夹具同时包含 tracked 未提交改动、untracked 和 ignored 文件。新 Worktree 的 HEAD 等于固定源 commit，代码内容等于该提交且不带这些改动；源三个类别原 bytes/mode 均不变。detached HEAD 正控制、非 Git/unborn 负控制以及已连接 SSH 同目标场景分别验。
4. 目标文件、Agent 命令、Git、内置终端和产物回读确属同一目标。SSH 来源与目标具有独立连接 owner/PTY，创建失败不迁走/关闭来源；双方并存及任一关闭后另一方继续使用均有正控制，同Arc双登记、来源connection_id误作draft、错账号/旧代际和断线均为负控制。认证只走原GUI，不收集聊天中的密码；本地仿真或 namespaced provider 不替代具名真实 SSH 验收。
5. 内联和文件附件、长原文、artifact-backed 工具记录在目标实际 reader 中逐内容/大小/SHA 回读，跨页/中文/emoji/长文本末尾完整。来源独立关闭/清理后目标副本仍可读；未知历史执行状态不伪成功，不把原实时执行引用接给目标。
6. 并发双击、回复丢失、观察者取消、source/target 切换、来源 append、Git 已创建但 list 失败、登记/导入/CAS/切换失败、进程重开与迟到回执均有正反控制；已创建资源可找回，不重复创建或无条件删除，焦点只属于仍有效的用户选择。
7. 实现同步所属 CLI ADR、中文说明、生成合同和适用示例；官网按可恢复的精确源同步。focused、前端组合、GUI Rust、真实新版 Tauri、本地/SSH、完整门禁和 main/CI 分别记录。开发验证与产生的 Worktree 夹具均位于唯一主目录内；最多五个 focused Cargo、jobs≤5/debug0，重型矩阵同窗一套，批后保全并确认准确 target 无 reader 后再独立执行 clean。

## 官方参考与取舍

[Git Worktree 官方文档](https://git-scm.com/docs/git-worktree)将工作树连接到明确提交，并保护有改动的删除场景；[Claude Code Worktrees](https://code.claude.com/docs/en/worktrees.md)把代码隔离与额外文件复制分开，默认只检出版本化文件；[Claude Code Sessions](https://code.claude.com/docs/en/sessions.md#branch-a-session)说明复制 transcript 与原会话保留，同时明确进程内分支会承接该进程的某些权限和后台活动。这些共同支持“对话前缀与代码目录分开处理、资源身份可恢复”的做法。

Eclip 采用用户指定的当前本地 `HEAD`，保留未推送但已提交的代码；不照搬 Claude 的默认远端分支、额外文件包含配置或进程内后台接管。现有 EKO fork 已以新 conversation/import 保持空闲，因此沿用它的模型/权限装配和运行准入，并把新目录接入同一 Workspace 权威；不复制旧活动 owner，也不新增第三套 Worktree 管理。

Git 官方同日原件 SHA 为 `cf843ea6e3edda3fdda65877063f6c9b8a8e91b54041bc39d0bc89246a6bc72d`；本次直接取得 Claude Worktrees HTTP200 原件 SHA 为 `9c07a62bca75ec0604e4fd9413d38c51533594f55a33eabff9ace2c20cb0e6cc`；同日已保全 Claude Sessions HTTP200 原件 SHA 为 `182a4a7be94a7be87e7a5687d284182be2d655b26d9c2ab788b3c21fd3793447`。位置与回执分别在根 `.echo-semantic/local/fork-two-forms-20261027/references/`、`.echo-semantic/local/reply-fork-workspaces-design-20261027/references/` 与 CLI `.echo-semantic/local/steer-goal-validation-20261027/raw-history-references/` 的 ignored 证据主题。Web reader 本次 POST `/v1/alpha/search` 返回404；Codex 官方页面此前403，均没有取得可引用内容，不据截图或访问失败推断其继承合同。
