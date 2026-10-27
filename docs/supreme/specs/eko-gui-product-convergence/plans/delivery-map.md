---
schema_version: 1
artifact: delivery-map
design_ref: docs/supreme/specs/eko-gui-product-convergence/design.md
outcomes:
  product-boundary-contract:
    ships: 在各自正式规则与 ADR 中固化 EKO 的 GUI 主入口、IM/定时任务保留边界及 A2A/LSP/TUI/CLI
      退役合同，提供精确可恢复的跨仓能力处置清单。
    depends_on: []
  tool-inventory-and-exa-boundary:
    ships: 形成覆盖定义、注册、模型曝光、消费者和结果产物的工具事实清单，并核对 Exa 独立提交及其暂不推送、不合 main 的边界。
    depends_on:
      - product-boundary-contract
  performance-baseline:
    ships: 提供会话/任务查询、文件冷恢复、记忆写入、GUI 列表和工具 schema/结果开销的可复现基准及同源比较方法。
    depends_on:
      - product-boundary-contract
  gui-visual-foundation:
    ships: 交付统一主题、排版、布局、导航、Modal 和状态呈现的 GUI 基础，并完成支持窗口与主题下的真实桌面视觉审阅。
    depends_on:
      - product-boundary-contract
  gui-home-navigation:
    ships: GUI
      首页明确区分工作空间与任务，提供新建任务、最近任务和工作空间导航，输入区以左下工作空间/权限、右下模型的布局复用真实操作，并通过目标隔离和真实窗口视觉验收。
    depends_on:
      - gui-visual-foundation
  steer-responsiveness:
    ships: 运行中追加输入复用原持久化入口与 tracked
      回执，及时确认且不重复提交、不串会话，并在未启动工具的安全边界优先考虑纠正，保留已启动操作结算；同时把 TaskRun 的 Steer/Goal
      控制收敛到 composer 上方：Steer 支持多条持久化输入的编辑、拖拽排序、删除和安全点立即执行，Goal 在同一 TaskRun
      内保持唯一，修改按 revision/CAS 立即刷新权威投影并使旧计划/证据失效，删除以结束当前目标运行实现。
    depends_on:
      - product-boundary-contract
  effective-requirements:
    ships: 在同一 TaskRuntime
      中持久保存完整原始要求及来源、范围、版本和替换关系，幂等连接输入接纳、上下文消费与目标/计划/证据更新，压缩和冷恢复保留仍有效约束，关键上下文缺失时暂停相关副作用，并以独立原始要求分母检查工作项、交付物及验证证据的覆盖。
    depends_on:
      - product-boundary-contract
      - steer-responsiveness
  im-cron-gui-control:
    ships: 在 GUI 中管理已有 IM 渠道的配置与连接，并完成定时任务启停、执行、结果与恢复的共享运行时接线。
    depends_on:
      - gui-visual-foundation
  console-surface-retirement:
    ships: EKO 独立 TUI、REPL/CLI/JSONL 和专用启动入口退出主线，GUI、IM、cron、内置终端及共享任务恢复保持可用。
    depends_on:
      - im-cron-gui-control
  a2a-lsp-consumer-decoupling:
    ships: 在仍保留 A2A/LSP API 的框架上完成 EKO/SDK 消费者去依赖，精确退役相应工具、feature
      请求与已接受合同，保留其余能力可构建和可用。
    depends_on:
      - tool-inventory-and-exa-boundary
  a2a-lsp-retirement:
    ships: 在框架、SDK、EKO 和官网完成 A2A/LSP 实现、feature、注册、配置、插件组件、accepted 合同、示例和文档的精确退役。
    depends_on:
      - a2a-lsp-consumer-decoupling
  builtin-tool-convergence:
    ships: 基于实际盘点收敛重复与过时工具，统一工具曝光、类型化结果、大输出读取、错误及执行边界；复用现有 delegation policy 实现
      Subagent 默认不再派发和 GUI 可配置最多三层，并对 Exa 给出保持独立提交的兼容性对账。
    depends_on:
      - tool-inventory-and-exa-boundary
      - a2a-lsp-retirement
      - performance-baseline
  hook-ask-semantics:
    ships: 框架保留 hook Ask/RequireApproval 的交互要求，并在允许规则、缓存、拒绝、无 provider、超时和取消场景下给出一致结果。
    depends_on:
      - product-boundary-contract
  gui-hitl-dialogs:
    ships: GUI 将审批、输入和选择统一为身份准确、队列可发现、可键盘操作的弹窗，保留与 IM 回答竞争及过期/取消语义。
    depends_on:
      - gui-visual-foundation
      - hook-ask-semantics
  task-list-dialog:
    ships: 任务列表与弹窗复用后端任务事实、进度和执行回执，统一任务层级、等待原因、详情及操作，并在高数据量下保持有界呈现。
    depends_on:
      - gui-visual-foundation
  ssh-remote-terminal:
    ships: 用户可从 GUI 经现有终端服务和系统 OpenSSH 连接并操作远端，真实验证认证、交互、resize、中断、多会话及断线/关闭。
    depends_on:
      - gui-visual-foundation
      - tool-inventory-and-exa-boundary
  project-workspaces:
    ships: 用户可从现有本地目录、新建本地目录或远程目录创建项目，项目上下文、文件、Agent 执行、Git、隔离、产物校验与终端使用同一真实工作环境。
    depends_on:
      - gui-visual-foundation
      - tool-inventory-and-exa-boundary
  domain-plugin-runtime:
    ships: 专业方法按任务激活并绑定插件身份/版本，工具、证据、完成和恢复由既有运行时统一执行，EKO 常驻宿主与提示词相应收敛。
    depends_on:
      - builtin-tool-convergence
      - console-surface-retirement
  measured-storage-optimization:
    ships: 对基准确认的性能瓶颈在既有文件权威内完成针对性优化和同源回归；无瓶颈项保留测量结论，不引入 SQLite 或虚构改进。
    depends_on:
      - performance-baseline
      - builtin-tool-convergence
      - task-list-dialog
  new-im-channel:
    ships: 通过腾讯官方个人微信协议接入原生 Rust 渠道 adapter，GUI 扫码和 direct
      单聊复用原消息/HITL/生命周期，并以明确账号和收件人验证回复及定时结果投递。
    depends_on:
      - im-cron-gui-control
  legacy-resource-cleanup:
    ships: 依据已完成旧补丁处置、当前占用和路径依赖，保存恢复依据并清理明确获授权的旧工作区与引用，保护剩余源码、stash 和证据。
    depends_on: []
  issue-finding-reconciliation:
    ships: 将历史修复、A2A/LSP 和旧入口退役对应的 Issue/Finding 按各自真实证据对账，具名区分修复、退役和风险接受并保留历史事实。
    depends_on:
      - a2a-lsp-retirement
      - console-surface-retirement
  release-target-selection:
    ships: 明确当前目标版本的产品范围、平台/架构、SDK 包、注册表、网站环境和制品要求，并给出后续实际发布授权的具名边界。
    depends_on:
      - product-boundary-contract
  release-artifact-qualification:
    ships: 对明确选定目标版本构建、检查并验收可审阅的桌面/框架/SDK/网站制品，绑定必要产品结果、精确源码与依赖锁，形成恢复与安装说明。
    depends_on:
      - release-target-selection
      - console-surface-retirement
      - a2a-lsp-retirement
      - builtin-tool-convergence
      - gui-visual-foundation
      - im-cron-gui-control
      - gui-hitl-dialogs
      - task-list-dialog
      - ssh-remote-terminal
      - project-workspaces
      - gui-home-navigation
      - steer-responsiveness
      - new-im-channel
      - effective-requirements
      - performance-baseline
      - domain-plugin-runtime
      - measured-storage-optimization
  package-publication:
    ships: 在具名制品及目标获明确发布授权后发布所选桌面/框架/SDK 包，核对真实注册表或分发目标中的身份与可安装性。
    depends_on:
      - release-artifact-qualification
  website-deployment:
    ships: 在网站制品、正式源镜像和部署目标获明确授权后部署官网，核对实际访问结果及其对已交付能力和历史版本的描述。
    depends_on:
      - release-artifact-qualification
design_revision: sha256:97233f9e54882b4462ccbe061ebf9db212c863cde184eddf9376a065da2bdec3
---
# 可并行执行安排

本图把已确认 Design 的完整目标拆成独立结果；依赖由 YAML 的 depends_on 唯一表达，下面的批次和工作组是调度建议，不记录完成状态或百分比。具体执行结果由用户或协调者给出的精确 Plan 路径选择，不按序号推断当前项。已有视觉基础与工作空间接口可供独立首页结果复用；未选中的结果只保留在图中，不创建占位 Plan。

## 启动和并行方式

实施从五仓已固定的 `pre-retirement-2026-10-24` 恢复基线开始，对照届时远端 main 的实际增量，不从 primary 的旧检出或旧 dirty worktree 回灌。保留分支/tag 和旧补丁采纳修复均已完成，不重新列为开发任务。

采用一个集成负责人加四个按文件分工的 Subagent 工作组：框架/SDK、应用运行时、GUI、工具与测量。只读盘点、文档、设计核对和小范围前端工作可同时推进；独立工作组不表示可以同时修改共享文件。工程收尾和发行目标澄清由集成负责人穿插处理。

每个正在写代码的独立结果使用一个明确归属的隔离工作区；已完成的工作区先结算证据再回收，不为所有未来结果预建 worktree。各组读取同一版 Design、边界 ADR 和接缝约定，知道其他组同时工作，不回退他人修改。

## 批次与可观察产物

任务列表的独立实现复用已冻结、通过工程检查的 GUI 主题、Modal 与 TaskRuntime 接口基线，不等待其它记忆和工具基准。列表自身的同源查询、渲染、高数据量及原生验收仍须完成；GUI 基础自身尚待的视觉反馈也不因此标为已完成。完整 performance-baseline 继续作为独立结果，全部原规模与场景仍是整个交付的必要条件，正式测量与重型编译继续错峰。

批次不是整批等待屏障：一项前置完成即可启动其下游，未完成的独立结果不阻塞其它路径。阅读表格时结合 YAML 中的精确依赖。

| 批次 | 工作组与结果                                                                               | 具体执行内容和主要文件                                                                                                                        | 本批产物                                                                                         |
| ---- | ------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ |
| 0    | 集成牵头：product-boundary-contract（固化产品与退役边界）                                  | 按当前 Plan 将框架、EKO、SDK 规则/ADR 分给独占文档路径；顶层规则与官网同步由集成负责人合流                                                    | 同步的产品合同；精确退役/保留清单；后续接缝和恢复依据                                            |
| 1    | 工具与测量：tool-inventory-and-exa-boundary（工具事实与 Exa 对账）                         | 查框架 Tool/StandardToolPack、`echo-agent-cli/echo-agent-app-core/src/tool_exposure.rs`、实际注册与 SDK accepted 合同；保持 Exa 本机提交独立  | 活跃/重复/过时/可复用工具清单；消费者与产物/错误/取消边界；已有 schema/结果预算事实              |
| 1    | 测量：performance-baseline（性能基线）                                                     | 复用 `echo-state/src/memory/` 与 EKO `tasks/task_runtime/file_store.rs` 等真实入口；在隔离测试/benchmark 范围构造数据                         | 会话/任务查询、冷恢复、记忆写入、列表渲染与工具开销的冷/热基准及源码/数据身份                    |
| 1    | GUI：gui-visual-foundation（统一视觉基础）                                                 | `web-frontend/src/index.css`、`components/common/`、`components/layout/`；统一主题、字号、间距、Modal 和导航                                  | 可复用的视觉基础与真实 Tauri 窗口审阅；不顺带改任务/HITL 状态                                    |
| 1    | 框架：hook-ask-semantics（保留强制交互意图）                                               | `src/agent/react/run/pipeline.rs`、`src/agent/snapshot.rs`、`echo-orchestration/src/human_loop/` 的真实决策链                                 | Ask/RequireApproval 不被自动允许或缓存吞掉；拒绝、无 provider、超时、取消的回归                  |
| 1    | 集成：release-target-selection（澄清发行目标）与 legacy-resource-cleanup（授权旧资源清理） | 版本/平台/注册表/网站环境逐项澄清；旧目录先只读核对差异、进程、路径依赖和恢复引用                                                             | 具名发行范围；授权范围内的恢复与删除回执。两者分开结算，不因一个完成宣称另一个完成               |
| 1起 | GUI：首页与导航（gui-home-navigation） | 复用现有 Conversation 新建、Workspace 切换/三来源表单、权限/模型 API；独占 App/Sidebar/ChatInput/Home 的布局与目标 fence，不改运行时 | 清晰工作空间/任务关系、真实 composer/menu 操作、浏览器与真实 Tauri 视觉及可访问性证据 |
| 1起 | 输入响应：steer-responsiveness（追加输入响应与身份） | 沿原 durable ingress/mailbox/回执修复确认、草稿身份和 late-response fence；工具启动前/波次间增强由原执行循环消费 | 反例证明不重复/不串目标，持久接收及时可见，已启动操作和完整工具配对保留；不冒称 Pi 已有每工具间保证 |
| 2    | 框架/SDK与应用：a2a-lsp-consumer-decoupling（消费者解除依赖）                              | EKO 移除 LSP 工具/插件/设置消费和 feature 请求；SDK 对精确 accepted 身份、路由、语言包装及 Host feature 同步退役；框架保留旧 API 到本结果验收 | 当前框架仍保留两能力时，新的 EKO/SDK 消费者已不再依赖、宣告或调用它们，未退役合同继续通过        |
| 2    | 应用+GUI：im-cron-gui-control（GUI 接管 IM/定时任务）                                      | 复用 `src/cli/channels*` 的行为；应用 owner 承接后台服务；`src/tauri/commands/config.rs`、`scheduler.rs` 与 GUI 面板薄适配                    | IM 配置/连接/重连/结果管理；cron 创建/修改/启停/手动运行与历史结果，隐藏窗口及真正退出行为可验证 |
| 2    | GUI：gui-hitl-dialogs（统一 HITL 弹窗）                                                    | `components/chat/ChatPanel.tsx`、审批/输入/选择呈现、`stores/chatStore.ts`、既有 dispatcher 与 IPC                                            | 来源、会话/任务、队列、一次有效回答、取消/过期与焦点行为；真实桌面验证                           |
| 2    | GUI：task-list-dialog（任务列表弹窗）                                                      | `components/tasks/TasksPanel.tsx`、`components/task/TaskRuntimePanel.tsx`、任务投影 store；入口接线交集成负责人                               | 同一任务事实的列表/详情，可信进度与操作回执，大数据量下有界显示                                  |
| 2    | 应用终端：ssh-remote-terminal（SSH 远程终端）                                              | `echo-agent-app-core/src/terminal.rs`、`src/tauri/terminal.rs`、`components/terminal/`，复用 PTY/xterm 和系统 SSH                             | GUI 可连接真实远端，认证/resize/中文/全屏程序/Ctrl-C/断线/多会话结算可核                         |
| 2起  | 项目环境：project-workspaces（三来源项目与真实环境）                                       | 复用 Workspace 创建/切换、项目上下文与原文件/执行/终端 owner；通用 I/O 适配与 EKO 位置/认证策略分层，公共接缝由唯一写入者合流                  | 本地已有、本地新建、远程项目的上下文、文件、命令、隔离、证据和项目终端使用同一位置             |
| 3    | 框架/SDK：a2a-lsp-retirement（框架物理退役及最终合同）                                     | 在消费者先解除依赖后，删除 `src/a2a/`、`echo-core/src/lsp/`、`echo-integration/src/lsp/`、`src/lsp.rs`、LSP 工具及 feature；消费者与官网对账  | 主线无两能力的活跃实现/广告/空壳；剩余能力通过更新后的适用验证                                   |
| 3    | 应用：console-surface-retirement（退役独立终端产品入口）                                   | GUI/IM/cron 已验证后，归位 `src/tauri/desktop.rs` 参数和 `commands/panels.rs` 的 CLI 依赖，删除独立 TUI/REPL/CLI/JSONL 与专属 provider/依赖   | 桌面启动、IM、cron、共享任务恢复和 GUI 内置终端均可用；旧产品入口退出                            |
| 3    | 工具：builtin-tool-convergence（内置工具收敛）                                             | 消费工具清单、退役结果与基准；复用已有曝光、分页、4K 模型结果预算和类型化 artifact 机制，先证缺口再修                                         | 真实重复/过时项退出，保留能力的输入、错误、完整结果读取及所有执行路径一致                        |
| 3起  | IM扩展：new-im-channel（个人微信扫码）                                                     | 已具名的 Tencent/openclaw-weixin 协议 Rust adapter、原 ChannelManager/SessionHandler 和 EKO QR GUI；cron 完成出口新增明确 account/recipient 投递，不新建会话/任务/调度权威                                                                  | 真实扫码/数字验证/过期取消、direct 单聊/HITL、账号/peer/context token 和重连；分别证明任务完成与微信收到，不虚构服务端额度                          |
| 4    | 应用：domain-plugin-runtime（领域插件）                                                    | 在工具收敛和入口收敛后，改 `tasks/task_runtime/profiles.rs`、既有 `plugin_components.rs`/`plugin_runtime/` 与执行消费点                       | 插件按任务激活和绑定版本；直接执行/Subagent/验证/恢复消费同一工具、证据与完成规则                |
| 4    | 测量：measured-storage-optimization（实测性能优化）                                        | 根据同源基准改命中的查询、投影、索引或写入路径；不为完成条目而换存储                                                                          | 可复现的前后比较和保真回归；无瓶颈项交付测量结论，不伪造优化收益                                 |
| 4起  | 集成：issue-finding-reconciliation（问题账本结算）                                         | 消费精确 main、修复/退役证据，核对 #188/#189、A2A #32–#35 及关联正式对象                                                                      | 满足各自条件的具名修复/退役/风险记录；Issue 关闭按授权执行                                       |
| 5    | 集成：release-artifact-qualification（发行制品验收）                                       | 已选发行目标与对应产品依赖全部交付后，在匹配源码/锁/平台上构建安装包、SDK 包/Host及网站制品                                                   | 可审阅的制品、安装/启动/配置验证、来源和恢复说明；不因源码合并就宣称已发布                       |
| 5    | 发布：package-publication（包发布）与 website-deployment（官网部署）                       | 各自制品/目标获明确授权后独立执行；官网只描述已交付能力                                                                                       | 注册表或分发目标核验、网站实际访问与版本核验，两项分别结算                                       |

同一行多个产品层的修改共同形成一个行为结果，不按“后端做完了”局部宣布交付。上表不是在所有前置条件尚未完成时同时启动全部实现。

## 项目工作环境与终端底座的关系

用户最新明确 Workspace 本身就是工作空间/项目/工程；任务和会话归属于它。左侧“新建任务”“最近任务”复用 Conversation，“工作空间”负责选择和创建；已有本地目录、新建本地目录、远程 SSH 设备与目录三来源属于创建 Workspace。[项目工作环境细化设计及恢复依据](../../../plans/2026-10-26-root-convergence/next-actions.md#项目工作环境细化设计恢复)细化该单一行为结果。子主题直接绑定自己的设计入口，不复制父图，也不新建冗余交付图。

project-workspaces 复用 ssh-remote-terminal 的连接、PTY、输出和关闭机制，但终端登录不能代替项目交付。两者不设置互相等待的依赖：项目环境可在既有终端底座上开发，并在同一次真实项目场景分别取得各自验收证据。远程项目新增的文件/上下文/执行/产物义务不能转嫁给旧终端结果来宣称完成，也不能因旧终端真实登录尚未验收而冻结独立项目源码工作。

2026-10-26 用户已明确修订两者的创建接缝：SSH工作空间表单提供临时掩码密码栏；“测试连接”只验SSH认证，不出现黑色终端，也不校验项目目录；创建时再校验目录，相对路径按该远端账号实际home解析。普通完整终端能力及原输出/关闭验收继续保留，不新增连接状态机或权限门控。密码不进入持久项目配置、日志或模型。最新实测目标为 `ubuntu@111.229.245.175`，home=`/home/ubuntu`，相对 `ssh-test` 对应 `/home/ubuntu/ssh-test`。后继原生epoch10已有用户明确确认登录测试与创建成功，**仅结算这两项**；完整终端与远程项目剩余验收不由此关闭。原 `echo` 探测及修复前失败继续保留历史身份。两项原Outcome及依赖不变，Plan06承接认证/终端边界，完整项目文件/执行等仍归project-workspaces。

通用文件与执行适配可与项目 GUI/本地两来源工作并行；双方先固定薄适配接缝，EKO Workspace、factory、AgentPool、共享 IPC 和 manifest/lock 的冲突写入仍由唯一负责人串行集成。Subagent、后台命令和工作树使用实际目标项目与 checkout，复用原 TaskRun、物理资源和证据 owner。正式重编译与真实远端验收按统一资源安排，不与其它完整矩阵争用机器。

## 首页、追加输入与个人微信的并行边界

首页导航只依赖已交付视觉基础，复用已固定 Workspace API。它不等待 project-workspaces 的全部远程 I/O/SSH unknown 收尾，也不以个人微信或 Steer 增强完成为实施前提。跨工作空间最近任务只读聚合现有 Conversation 列表，保留身份和 generation fence；新建任务不创建目录/Workspace，也不新增 TaskStore。

steer-responsiveness 与首页可独立推进，但 ChatInput、会话投影等真实交集仍由唯一写入者串行合流；纯布局不顺带修改输入持久化/回执或 Framework 执行语义。原模型前和工具安全点、GUI 异步 fence、固定草稿输入身份分别取证，既有后台/HITL/IM/cron 入口不能因界面调整消失。

new-im-channel 已由用户指定个人微信，沿官方协议与原 ChannelManager/SessionHandler/应用生命周期接入；企业微信、群聊、完整微信账号接管或未验证的媒体不自动纳入承诺。二维码/数字验证/过期/取消属于原账号管理流程；定时结果需补真实 account + recipient 目的地和回执，现有 Webhook 不能充当 IM 投递证据。这里只维护该结果与依赖，待该结果被明确选中时再创建单 Outcome Plan。

## 为什么退役要先处理消费者

当前 CLI CI 会检出 framework main，GUI 和 app-core 又直接使用 LSP。若先合框架删除、再等待应用适配，会让当前 CLI main 无法编译。SDK 还具有独立 framework pin 和已接受合同，不能靠顶层 gitlink 推断它已迁移。

因此本图在原退役目标内拆出消费者解除依赖这一可独立验收结果：在旧框架仍可编译时，先让 EKO/SDK 主线不再请求退役能力，再删除框架实现。新增框架 API 仍遵循框架先交付、消费者后采用；撤销 API 使用相反的去依赖顺序，在产品边界 ADR 和规则中写明适用条件。

SDK 可以先在原 pin 上精确退役两类合同，最后再按实际需要评估是否升级 pin；不为形式统一强迫 SDK 追随 CLI 框架版本。每个可合入主线的提交都必须保持其当时受支持的消费者可构建。

## 文件归属与合流接缝

| 文件或边界                                                                                  | 并行时的唯一写入归属                             | 其他组如何接入                                                                         |
| ------------------------------------------------------------------------------------------- | ------------------------------------------------ | -------------------------------------------------------------------------------------- |
| 各仓 Cargo manifest/lock、feature、CI matrix、SDK 生成合同                                  | 当前结果的集成负责人；在该结果内集中更新一次     | 其它组提交精确依赖/API需求，由负责人核对；不各自全量刷新锁或117项产物                  |
| EKO `runtime.rs`、`state/app_state.rs`、`src/tauri/desktop.rs`、`src/lib.rs`                | 当前合流结果的应用集成负责人                     | 消费者解除依赖、IM、入口退役、插件各给出真实接缝变更，串行应用并复验                   |
| `plugin_runtime/` 的 LSP 退役与领域插件演进                                                 | 消费者解除依赖先独占；其后才允许领域插件结果写入 | 不同 worktree 也不能并行改相同生命周期 owner 并各自宣称通过                            |
| `index.css`、`components/common/Modal.tsx` 等公共组件                                       | GUI 视觉基础组                                   | HITL/任务/SSH 复用已落地的接口，需求先反馈，不复制 Modal 或自行再改全局 token          |
| `SettingsDialog.tsx`、主导航、`src/tauri/mod.rs` 的 invoke 注册、共享 endpoints/生成 barrel | GUI/应用集成负责人                               | IM/SSH/HITL/任务/LSP 菜单各提供自身组件和命令接缝；负责人批次内合并注册后统一验证      |
| `ChatPanel.tsx`、HITL 请求/响应桥与 `chatStore.ts`                                          | HITL 组                                          | 任务弹窗不从聊天文本挖任务、不重写 HITL 队列；工具组如需事件变更，先锁定生产类型再接线 |
| 任务详情/列表组件与 taskRuntimeStore                                                        | 任务列表组                                       | GUI 基础只改公共布局，IM/cron 只消费任务身份；避免多组同时改状态解释                   |
| `terminal.rs`、Tauri terminal adapter 与 `components/terminal/`                             | SSH 组                                           | 视觉基础提供公共样式后不进入终端行为；工具组不得另建远程终端 owner                     |
| 工具生产注册/曝光与类型化结果链                                                             | 工具组                                           | 基准组测现有接口；需要性能修复时在独立结果接管命中路径，不能测试时顺手改实现           |
| 官网源镜像/manifest/discovery、顶层 gitlink、统一验证记录                                   | 集成负责人                                       | 收到各子仓已推送且验收的精确提交后生成；不能并发运行镜像同步互相覆盖                   |

所有组都不是独占整个代码库；只拥有已派发路径。跨出范围、改变返回类型、生命周期、完成语义或公共合同必须先回到对应 Plan 和预检对齐。接缝变更必须实际进入集成树并验收，不能只交补丁片段即报告生产可用。

## 合流与验收节奏

1. 选中结果先核对依赖的真实交付证据、工作区归属和允许路径；保留原 primary、Exa 分支、固定 tag 及仍需恢复的原件。
2. 开发期间按所触及行为执行 focused 测试、类型/格式和相关 crate 检查。bug 修复使用有意义的反例与正常控制；视觉、真实 IM/SSH 和平台行为分别取证。
3. 一个结果源码冻结后先独立只读复审，再完成所属仓规定的适用完整门禁；不按每个小提交重复全矩阵，也不以 focused 绿代替合并门禁。
4. 消费者去依赖与框架删除按上节顺序；其它新增通用机制按框架先、消费者后。框架 main 推进会影响 CLI CI，相关跨仓结果逐个合流并等待精确提交 CI，不同时推多个 main 让 CI 相互取消。
5. 子仓提交/推送/合并均按独立仓库和会话授权处理；确认后才更新顶层 gitlink。记录 source/framework/lock、退出码、跳过范围、真实服务/模拟和原生/非原生平台。
6. 完成的工作区在确认成果、原件和路径依赖有去处后回收。历史账本和发布仍按各自 Outcome 结算，不能借其它组已通过而关闭未验收结果。

## 历史成果与遗留合同的承接

SDK 独立拆仓、Protocol 无框架依赖、Host 精确 pin、accepted 合同与 Rust 遥测分离已经交付；旧拆仓 Plan06 仍为 ready 且引用旧 pin，不代表应重做实现。`issue-finding-reconciliation` 按正式 SDK 迁移/接受记录对账其旧验收和状态，`legacy-resource-cleanup` 保存原计划及 `MIGRATION-SOURCE.md` 的历史来源定位后再归档。SDK 当前独立 pin 继续遵循所属仓合同。

旧 unified-task 材料中的唯一任务图、共享 `ExecutionAdmission` 与默认容量5已有真实接线，不再列为待新建机制。`builtin-tool-convergence` 与 `domain-plugin-runtime` 必须对账 EKO ADR0034/0039 对同一批内置子 Agent 的 `agent_tool` 曝光与委派边界条款：区分 direct fallback、TaskRuntime 显式 policy 和 PlanTask `allowed_tools`，不能由框架默认深度推断所有产品路径；本次最新完整Goal已明确默认不再派发和可配置最多三层，按下文已批准要求统一实际限制，不再重复要求选择方向。旧材料中“全部 direct dispatch 归 TaskRun”的适用范围一并核对；旧 map 的完成状态不能代替实际路径证明。

R1 启动中断恢复、R2 本地 SDK 子句柄与压缩 PR18 已有后继提交和正式 ADR/验收依据。`issue-finding-reconciliation` 按具体子问题记录后继结论，保留 contract 第13节、修复前报告及未证实边界；不把整份旧报告一律标成 resolved。EKO 后台持有任务的修复不能替代框架 MCP preparation 取消合同，R2 的 Host 故障注入/资源保留边界也不能据局部修复宣称通过。这些先做事实对账，需要实现时由既有工具治理结果承接，不据旧 ready 状态重复开发。

## 本机并发预算

当前机器为 16 GiB 内存、8 个逻辑 CPU。前次同仓并发构建曾发生 target 混用与磁盘耗尽，本计划采用保守预算：最多两条 focused Rust 构建同时运行，各自 jobs 至多2、dev/test debuginfo 关闭；完整 all-feature/GUI/语言矩阵同一时间只运行一套。前端、浏览器与真实 Host 也纳入实际内存观察，紧张时暂停新的重任务。

当前唯一开发/写入/构建根为 `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent` 及四个子仓，不再新建或继续使用外部 worktree 开发。每个 Rust 批次在根内使用自有独立 CARGO_TARGET_DIR；ts-rs 导出也在本批根内独占目录，不覆盖正式类型。用户于2026-10-24要求每个子任务、子智能体在编译或测试批次结束后及时执行 cargo clean：先把日志、真实退出结果、必要 Host/制品与验收原件保存到 target 外，再确认没有活动 cargo/rustc/测试子进程/Host 使用，针对本任务的准确 manifest/target 清理并记录磁盘前后。没有 Cargo 产物的文档和只读任务注明不适用，不机械执行无关清理。每次重构建前检查磁盘；可用空间低于5 GiB停止新构建并先处置明确可重建的本任务缓存。不因清理而重跑已通过的同源验证。

旧资源维护已获用户单独授权，可与产品合同同步独立推进，不受产品合同交付顺序阻塞。范围覆盖五仓的所有未跟踪/未提交文件、分支与 worktree 的逐项分类和处置；优先回收已完成旧补丁来源。删除前保存并读回验证 patch、未跟踪及 ignored 原件和必要提交；保护活动任务、primary 用户内容、stash、Exa、完整能力基线/tag及其它项目资料。历史 Host 二进制先逐文件校验压缩归档，再移除原件并保留恢复索引，不能把历史原件冒充可重建缓存。未知价值的独有提交、嵌套独立仓和宿主管理工作树先核对来源与回收权限，不按目录年龄批量删除。

这限制的是本机重构建并发，不限制独立源码阅读、文档撰写和方案审查；不能把四个工作组误解为四个 cargo workspace 同时全量编译。

## 发布与剩余选择

首批必验产品结果已经在 release-artifact-qualification 的依赖中绑定：入口与协议退役、工具收敛、统一 GUI、IM/cron、HITL、任务列表、SSH 和完整项目工作环境。首页导航、Steer 响应性和个人微信也已绑定为发行制品的必要依赖。选择发行平台、包、注册表与网站环境后，再核对明确纳入该版本的其它领域插件与性能结果；具体制品 Plan 创建前核对实际依赖均已交付。

个人微信已经具名，不再等待平台选择；真实扫码及 SSH 验收仍由用户在 GUI 完成必要授权和受控目标选择，发布另需平台/凭据/目标范围。这些真实服务/发行输入不阻塞已明确的独立前端与源码工作。Exa 的暂不推送/不合 main 指令持续有效；未经新授权，不通过发版或工具治理顺带合并它。

包发布与官网部署在各自制品核验、范围确认和外部动作授权后可以独立推进。设计中的全部目标仍在本图跟踪，未纳入某次版本的项目不标为已完成。需要新的架构/公共合同选择时返回设计澄清，避免执行计划悄悄新增产品承诺。

## 2026-10-26 原计划接续与已批准要求

当前 Plan12 在原 steer-responsiveness 结果内承接 composer 多 Steer 和唯一 Goal 控制；Plan13 保留正式历史运行、有限 run/event 查询与准确回执的结果，并消费 canonical Goal/证据新鲜度，不能因 controls 新增测试通过就关闭列表/原生/高数据量验收。完整有效要求已由用户明确选中，新增 effective-requirements 结果及 Plan14，复用原 TaskRuntime；原文修复、有效投影、关键读取失败、目标/证据连接和独立覆盖均须成立，不能将第一切片等同全链完成。

builtin-tool-convergence 的默认委派行为以最新完整 Goal 为准：Subagent 默认不能再次派发、GUI 可配置最多三层、默认嵌套零。旧 ADR0034/0039 的差异按此裁决统一实际工具曝光和恢复策略；如 UI 总层数含首层，应验证默认一层与嵌套零同义，不再以旧冲突要求用户重复选方向。框架可复用上限仍与 EKO 产品默认分开。

四条当前线路的实际文件 ownership、证据切点、剩余12项微信命令及完整 Goal 的执行顺序见[根内下一动作](../../../plans/2026-10-26-root-convergence/next-actions.md)。本图依赖是最终交付依赖；不阻塞独立前端、文档、归档或源码切片准备。有效要求的原文/恢复切片可在 Steer 主链真实源码和回执上并行准备，最终仍须与该控制链集成验收。

外部 echo-semantic 工具合同修复排除；已有最终退役 semantic 自然1及来源/合同阻塞继续保留，排除不等于豁免。用户最新允许且要求及时本地里程碑commit，Git暂存/提交由主线程唯一协调；本交付对账线不自行stage或commit。push/merge、关闭Issue、发布或部署仍禁止。全部26项及Plan14未完成范围继续对账，局部提交不等于完整验收；Exa本机提交与保留tag继续保护。

## 2026-10-26 原生epoch10后继证据切点

原生构建退出0、无编译警告，二进制 `b330bf25…` 绑定consumer `d205acd2…`，保全后准确clean6.8GiB；总调度已实际启动PID43779/窗口163930并独立核对前台与真实Tauri读取。第二版受控模型30项离线及JSON/SSE冒烟通过。SSH用户确认登录测试和创建成功，当前权威目标为 `ubuntu@111.229.245.175:22 /home/ubuntu/ssh-test`；普通完整终端矩阵仍未完成。

ubuntu真实成功只结算该账号与路径，Goal要求的 `echo@111.229.245.175` 支持仍待具名验证，不能以ubuntu或侧栏刷新成功替代。旧echo非交互认证失败继续保留；没有因本地提交授权扩张真实连接、凭据收集或外部动作范围。

侧栏仅更新current未同步list的修复已有4项失败转绿及独立复审0，用户已明确“刷新后已显示ssh-test”。原生PID43779/b330及SSH连接保留；最后前端148文件、manifest `25d8584a…` 已加载，设置移除overview/默认模型/5组分类及普通聊天信息气泡修复均已载入。settings-layout交接 `71b16bf4…` 的154测试与构建均0，保留两项既有Vite静态/动态import同chunk提示，不称零warning。聊天原journal保留，真实HITL请求/同目标release/工具success仅结算其具名往返；完整设置视觉/HITL/终端矩阵仍开放。

微信d205旧P1退出101及原10秒超时保留。后继核心epoch10已有按generation退休被动launcher的产品修复，consumer `2f7d326d…`；原P1实际选中1、通过1/失败0、退出0，P2/N1/N2及5项邻近各选中1、分别退出0。首个durable_reflect默认栈SIGABRT原signal保留，完整13未过，不以新增邻近回归替代原13分母；旧清单已恢复到根recovered-original-13，但恢复不是新执行。原文/有效要求两项原失败继续开放，完整Goal/Design不缩小。核心新源码未进入保留原生b330制品，不据此宣称当前原生采用新Rust。

工程回收已累计51=外部41+内部10，原路径合计22.24GiB；internal-reclaim/postcheck确认仅剩5主检出+4host路径，无failure。本轮不新增删除，refs/stash/tag/Exa/恢复材料继续保留。精确SHA、回执路径及下一动作见[下一动作表的当前切点](../../../plans/2026-10-26-root-convergence/next-actions.md#原生epoch10与后继源码增量的当前切点)。只同步局部证据，不重新GET、运行构建、修改计划生命周期或外部Issue。

工具治理的primary verification原三控制与后继搜索激活消费者已由本地a366d984同批实际4/4通过，新增case含允许/排除list_dir两run的首请求延迟、原搜索回执及第二次实际ModelRequest.tools；共7请求，最大14833bytes/3708估算tokens，原16000/4000预算和三项单请求断言保留。另原depth/hard-bound1项独立通过，GUI同冻结build0/零warning，准确clean0移除5.1GiB；目标工具不执行，原provider真实开销与其它消费者未证。该切片不再列未应用候选，不把一consumer扩大到全部插件、Subagent、恢复或SDK，完整builtin-tool-convergence依赖与验收不变；原切片和后继资格见[下一动作表](../../../plans/2026-10-26-root-convergence/next-actions.md#工具与性能剩余证据)。
