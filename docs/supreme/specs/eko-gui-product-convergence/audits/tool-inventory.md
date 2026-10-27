# 工具事实清单与 Exa 边界

本报告交付 `tool-inventory-and-exa-boundary` 的跨仓阶段清单。它记录定义、注册、曝光、消费者与读取链，不替代各仓正式 API/ADR，不修改工具、不裁决新的委派政策，也不表示 A2A/LSP 已退役或五仓 main/CI 已交付。

## 2026-10-27 后继核验：结果回退预算

下文§1–§10保留原审计切点。其中§4.3与§9的“spill失败后按四倍字符数回退可能超预算”具名项，已由框架提交 `fcc8a7d1cc8bced22d05d848b79aa664e1b51799` 修复，当前本地 `69296a4dea1047be7fb91b7ab06a839ee57dda7e` 已继承；该方法及相关回归体与引入提交逐字一致。四倍字符数仅作候选上界，原HeuristicTokenizer会对头部、说明和尾部整体计数；零预算、小预算及原8,000-token spill失败回退均有明确处理，省略部分不会被描述成仍可完整读取。

已沿真实消费者核对：[snapshot处理](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/src/agent/snapshot.rs)经[TruncationStage](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/src/agent/react/run/pipeline.rs)写入`ctx.output`和最终`ToolResult.output`，随后由tools phase放入`Message::tool_result`并发出`AgentEvent::ToolResult`。这项核对覆盖最终使用的字符串，不能仅以UTF-8不panic代替预算保证。

本轮复用现成保留测试二进制 `a70d5287aa5ef7a7c3e534acf87b48e50fd44480d6b3a1d309e0b71ec4d052f1`，其1,028个框架输入与当前源码逐SHA相同。配置为`no-default-features; testing,files; lib`、原默认栈；三个原有exact控制各选中1项、各通过1项，均自然退出0且无忽略：

| 原有测试名（共同前缀`agent::react::run::pipeline::tests::`） | 本次实际覆盖 |
| --- | --- |
| `truncation_stage_fallback_honors_token_budget_for_unicode_and_tiny_limits` | ASCII／中文／emoji／混合文本、八种预算及artifact禁用／故障的64组合 |
| `truncation_stage_records_inline_output_metrics` | 正常短ASCII结果与原inline指标 |
| `truncation_stage_falls_back_when_spill_directory_is_unwritable` | 默认预算下真实spill不可写回退 |

因此，这个具名项应按“当前实现已修、同源保留二进制focused 3/3”接续，退出后续待开发队列；原审计原文和历史失败保留。实际回执为框架`.echo-semantic/local/tool-fallback-budget-20261027/final-receipt.json`，SHA `560da19f6e5eb8f0c08c3a181f157c4092dd67d51435ec26f1052edde69ea330`。本轮没有新增生产代码、重新Cargo编译或创建target，`cargo clean`不适用。

这里结算的是原HeuristicTokenizer估计预算的回退／inline范围，未验证provider真实计费token、完整请求、spill成功preview、富媒体、SDK direct、原生或main CI。Web失败分类、其它曝光与消费者去依赖仍按各自证据推进；Exa继续本地保留、不推送、不合main。子仓正式中文streaming文档已包含该实现合同，本轮仅更新顶层阶段审计，不新增公共API、示例或官网行为。

## 2026-10-27 后继核验：Web错误分类

原§4.3/§9的“WebSearch把provider Err一律投影为retryable Transient”也已在`fcc8a7d1…`修复，当前框架69296已继承。生产HTTP helper保留typed HTTP/传输原因：认证类错误为Permanent，408为Timeout，429/5xx为Transient并保存Retry-After；不按错误文本猜HTTP状态。WebSearch通过原`ToolFailure::from_error`保留最终failure/status，ToolManager依据`allows_automatic_retry`、副作用/恢复政策与原等待上限决定是否重试。最终ReAct事件仍保留原`ToolResult`，模型侧可读错误文本不取代typed分类。

本轮首先逐路径核对19项输入和真实消费者，没有把源码读取写成fresh Web测试通过：现有`testing,files`保留二进制不含web，依赖crate的Provider单测也不在root库测试二进制中；旧976项通过仅见提交说明，具名原始回执本轮未定位。后继已在深度修复同一冻结批次实际取得三个原有exact控制各1/1、自然退出0且无忽略：`web::search::tests::direct_search_preserves_complete_remote_failure_and_status`、`web::providers::brave::tests::actual_http_manager_stops_auth_and_waits_for_rate_limit`及`agent::react::run::pipeline::tests::react_tool_pipeline_preserves_web_remote_failure_for_the_caller`。前两项在`echo_tools`的web配置，最后一项在root `echo_agent`的testing,web配置，实际选中和退出分别记录，不借上节预算3/3替代。

因此，该具名项按“现实现已修、WebSearch typed结果／本地HTTP与central retry／ReAct最终结果的focused 3/3”接续，退出待开发队列。源码未因本轮验证改变；当前框架本地检查点为`f3c5af6443fff36dd7088dcb32a665b5fc173489`，其中新增的是独立的深度策略修复。Web原始结果见框架`.echo-semantic/local/subagent-bound-policy-fix-20261027/r3-webtools-controls.json`和`r3-webpipeline-controls.json`，必要制品已保全，准确自有target官方clean0移除2.8GiB；不能扩成真实上游服务、所有Provider、SDK direct或main CI通过。

详细类型、最终重试与结果链、现有准确命令及范围见框架`.echo-semantic/local/web-search-error-classification-20261027/具名风险核对.md`，SHA `cb0ad70a4096f7b7bf0c32a9b7d4f4f3f39703dbe0045cfee79997f19fa8c3ae`。SDK的显式DuckDuckGo factory与direct Host不走同一central retry，且仍固定其独立框架pin，不能由本地框架静态核对取得SDK或真实服务资格。旧原文、Exa hold及其它工具治理义务保留。

## 2026-10-27 后继核验：primary verification实际请求

该消费者的缺省、`read_file`收窄及非空无交集三个控制，已由epoch11的原factory/execute_run到模型API实际3/3通过，0 failed/0 ignored；list/run自然0，不再列为待开发。原`followup-tools-green.log` SHA为`492924bbcfce121366ee06106b9e47154f16b9403ddf537124475208213ad0bc`，前后1358项源清单逐字相同；只记录工具名/定义字节和原估算token门槛，不保存模型正文，不编造原log未给出的实测数值。

当前F f3c5/CLI 612的主要曝光路径与原件连续，原三测试主体未改；F think的后继required校验只借用最终请求但可能阻止到达模型，故当前整合/失败结算资格仍须各自取证，不能把整树说成epoch11或fresh全矩阵。精确逐路径核对见CLI`.echo-semantic/local/remaining-tool-consumer-audit-20261027/剩余工具曝光消费核对.md`。本轮只读，不重跑保留binary或新建Cargo target。

后继`a366d984d1397532ee16da05b0c7efc23deac53b`已把`tool_search`激活到第二次实际请求的单一新增测试合入本地：正/负两run首请求均只有read_file/tool_search，正向实际搜索回执后第二请求新增list_dir，排除它的负向仍原两工具；目标目录工具不执行。原三请求测试主体、单请求和预算保留并同批重跑，4个工具消费者case全0/0ignored，另既有depth/hard-bound1项单列0。7次模型请求最大14833bytes/3708估算tokens；1365构建输入前后相同、同批GUI编译零warning，fmt/check和精确一源artifact/task/final均0，globalClosure not_checked。必要制品/原日志保全后准确clean0移除5.1GiB，目标不存在；回执在CLI`tool-search-request-consumer-20261027/actual-implementation/`。本次仍不证明list_dir执行、provider真实token、所有插件/Subagent/恢复/SDK曝光，完整工具治理继续开放。

原§9泛化的verification差距须扣除上述具名控制；原AcademicResearch初始四搜索是旧切点，当前General/plugin化后不能照旧名造新测试。不以这个局部结果关闭整体工具治理。

## 1. 输入、源码身份与证据等级

执行来源为 [Plan02](../plans/plan_02_tool-inventory-and-exa-boundary.md)，artifact_id `plan:92c06a40-2352-4be1-893b-c30d70d83dee`；设计与依赖见 [Design](../design.md)、[Delivery Map](../plans/delivery-map.md)。用户明确将已独审的五仓合同内容作为实施基线；主线及 CI 收口仍归协调者。

| 输入                   | 精确身份                                                                  |
| ---------------------- | ------------------------------------------------------------------------- |
| 顶层工作树起点         | `44401e2062429ed675e2aa48464fe1a72b73cba8`                                |
| Plan02 SHA-256         | `b90b4250afd31baaa8165674f9c57f699e77d193d256d0254b6e582ee7f2f6b2`        |
| Design SHA-256         | `1add234fe9bd5bbf6865cbddcad63f59b7c7264eba2e8fa245d13fd4daf12031`        |
| Delivery Map SHA-256   | `6dae79f97ba0666a7024a0a09a58ed302a17d090f42311c5cebd17386191b644`        |
| F：framework 本次读取  | `c3b56b4ee8a73565e6a2a621b9a7ddfb644ed3aa`                                |
| E：EKO 本次读取        | `0de6960a8610002b471e99034ca39c3dad903ec7`                                |
| S：SDK 本次读取        | `961da0e0d17b815653d80cb4bf4973248f543770`                                |
| SDK 独立 framework pin | `f7bac62eee04a963f92a3e4308b6ea2336e91cc0`；不能用 F 代替 Host 的构建来源 |

所有源码通过独立仓库 `git show <SHA>:<path>` 读取，未初始化的 superproject 子模块不作为源码。增量核对确认：F 相对 `51726454985fd6f064b21a7e9c539e22d494c987` 只改 AGENTS 与 ADR0090；E 相对 `c262cfed8e77abed38a27c1fff93547997ec9486` 只改规则、双语产品文档/ADR0050及 parity manifest；S 相对 `0930eef9f878cf436ed88384030b6bc03d6f6c53` 只改规则、ADR0042与退役边界文档。下列 GitHub 源码链接固定在这三个已发布生产基线，所引用代码与本次 F/E/S 字节相同；本地合同提交不是已发布链接。

本地合同依据另以 Git 对象定位：`F:docs/adr/0090-a2a-lsp-retirement-contract.md`、`E:docs/en/adr/0050-gui-product-convergence.md`、`S:docs/adr/0042-sdk-a2a-lsp-retirement-boundary.md`、`S:docs/sdk/governance/a2a-lsp-retirement-boundary.md`。前缀严格指上表 SHA。

证据分三层：**源码事实**表示可定位静态路径和条件；**已有回归入口**表示当前提交包含该测试，不表示本次执行；**本次直接验证**只覆盖输入哈希、静态合同重算、文档/链接、差异与任务语义检查。未运行 Rust、模型调用、HTTP mock 或真实服务，未采集耗时/token用量/内存基准。此前准备聊天与历史通过数仅是导航，不作为本次运行回执。

## 2. 从定义到实际消费的共同链

框架的 `Tool`/`ToolContext`/`ToolResult` 是通用协议；`ToolManager` 是注册、schema/参数校验、执行与预算遥测入口。`ReactAgent` 构造先建工具管理器，装 feature 工具与 memory，再挂 `tool_search`、可选委派和消息工具；模型请求经过 invocation 的 disabled/visible、skill、plan/readonly 等条件筛选。[F 构造][f-react]、[F 注册与校验][f-manager]、[F 模型筛选][f-visible]。

EKO factory 提供产品预算、工作空间、shell policy、command cells 与角色配置；应用 bootstrap/AgentPool 再装任务、协作、研究及动态能力。`chat_driver` 创建 invocation 后由统一 Agent 链执行。`record_schema_budget` 的初始集合只是一次选择，最终模型请求仍在 think 阶段重新计量。[E factory][e-factory]、[E chat][e-chat]、[F 实际请求统计][f-think]。

SDK facade 具有独立、按 Cargo feature 编译的封闭操作集合。其直接工具操作从 `family_tools` 绑定已有 Tool 的 schema，调用共享 `validate_tool_input` 后 `execute_with_context`，再转换 typed wire result；它不是 EKO 模型的另一个 tool_search registry，也没有自动经过 ReAct 的输出截断阶段。[S facade factory][s-tools]、[S direct][s-direct]。

## 3. 分组清单：定义、注册、条件、owner 与副作用

以下按语义组列出生产入口。表中类型集合来自静态源，不是一次运行的注册数量；feature 编译、构造成功、配置注入、角色、动态连接和 invocation 都能改变实际集合。每个静态定义均不能仅因存在就视为已消费。

### 3.1 框架 StandardToolPack

共同注册权威是 [echo-tools/src/registry.rs][f-pack]；`ReactAgent::register_feature_gated_tools` 仅在 `enable_tool` 时安装，readonly 是此 pack 的能力投影。各定义在对应 `echo-tools/src/` 模块，EKO通过 factory 开启选定根 feature，SDK通过 `framework-*` facade feature 选择独立操作。

| 定义组／类型或名称                                                                                                                                 | 编译与注册条件                                       | 曝光／实际消费者与读取                                                                                                                 | 状态及副作用 owner                                                                           |
| -------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- |
| artifact：`ReadArtifactTool`／`read_artifact`                                                                                                      | `artifact`；`files` 会启用它                         | EKO首轮可见；模型分页读；GUI另有相同 reader 的适配。SDK accepted source operation 见§5                                                 | 框架 artifact reader；仅按既有引用读取，不执行原工具                                         |
| 文件：ReadFile/ListDir/Grep/Glob/ApplyPatch/Diff/RepoMap/CodeSearch                                                                                | `files`                                              | 标准 pack 8类；EKO首轮包含 read_file/list_dir/grep/glob/apply_patch/diff，repo_map/code_search 按需激活；SDK files factory另有保留工具 | 文件内容权威在工作空间；Path/ToolContext限定目标；apply_patch写文件，其余读取                |
| 进程：ShellTool、RunCodeTool                                                                                                                       | `shell`；可注入 command-cell registry                | shell首轮；run_code在EKO TaskRun初始集合，普通chat可按需激活；SDK shell直接调用另算                                                    | 框架Tool/Sandbox原执行owner与cell registry；EKO替换同名shell注入产品policy，不新增执行器     |
| Git：status/diff/log/blame/branch/commit，enter/exit/list worktree                                                                                 | `git`                                                | EKO通常deferred；SDK git family使用同一类型                                                                                            | Git仓库与worktree原语；branch/commit/enter/exit有写副作用，不能以只读描述绕过能力筛选        |
| RAG：RagChunkDocumentTool                                                                                                                          | `rag`                                                | 标准pack只自动装chunk；index/search需显式依赖注入，SDK有session绑定的RAG资源                                                           | 框架RagStore/embedder；是否联网由实际embedder决定，不把chunk等同已建索引                     |
| 图表：GenerateChartTool                                                                                                                            | `chart`                                              | EKO按需；SDK chart family                                                                                                              | 工具生成图表spec/HTML文本，不直接落文件；需要时由通用输出层spill，artifact不代表业务验收完成 |
| 数据库：SqlQuery/ListTables/DescribeTable                                                                                                          | `database`                                           | EKO本次app-core未请求该feature；仍是合理框架/SDK能力                                                                                   | 数据库执行器；SQL实际读写分类及连接配置决定副作用，不能因EKO没用删除pub实现                  |
| Web：WebFetch/WebExtract/WebSearch                                                                                                                 | `web`                                                | EKO初始chat有search/fetch，TaskRun初始组有search；其他可激活。标准pack与SDK直接factory当前均显式DuckDuckGo                             | search查询provider；fetch读取URL；extract处理内容。HTTP客户端与取消/预算边界归各工具及执行层 |
| 文档/媒体：ViewImage、ImageFetch、PDF2类、Excel5类、Word3类、Text4类                                                                               | `media`；ImageFetch构造成功才装；ExcelLoad另需`data` | EKO延迟schema；`view_image`保留多模态能力；SDK media family按构造可用性报告                                                            | 文件读取、网络抓图、Excel/text导出分别归原工具；不能将整个media组称为无副作用                |
| 数据：Read/Filter/Aggregate/Stats/Transform/Export/Profile/TopN/Contribution/Bin/Ratio/MultiRead/Join/Correlate/Pivot；Missing/Outlier/Consistency | `data`                                               | 本次EKO script-first组合不请求data；SDK data family仍保留这些工具                                                                      | 框架数据处理与导出；是合理能力菜单，不以应用未调用认定死代码                                 |
| 统计：ExploratoryStatisticsTool                                                                                                                    | `statistics`，间接启用`data`                         | EKO未请求；SDK statistics family                                                                                                       | 框架统计处理；不得只为缩小EKO注册数删框架选项                                                |
| 学术：Arxiv/SemanticScholar/PubMed/ClinicalTrials搜索、PdfFetch、BibtexGenerate                                                                    | `research`                                           | AcademicResearch初始曝光四个已注册搜索；其余deferred；EKO AutoIngest wrapper和research_library为应用适配                               | provider抓取、PDF文件、引用产物归原工具；EKO摄取写入工作空间研究资料                         |

标准 pack 已只使用 `apply_patch` 作为文件变更入口。`CreateFile/DeleteFile/WriteFile/AppendFile/UpdateFile/MoveFile/EditFile` 等未自动进入该pack，不代表公共定义已删除；SDK files factory明确仍消费其中六类。[F pack回归][f-pack-tests]、[S files factory][s-tools]。`diff` 与 `git_diff` 输入和目标不同，未证明可以无损合并。

### 3.2 框架额外注册与动态扩展

| 组                   | 定义→真实注册→曝光条件                                                                                                    | owner／结果与限制                                                                                                                                              |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 最终回答/发现        | `FinalAnswerTool`在ReactAgent构造注册；`ToolSearchTool`在共享manager建立后注册                                            | final_answer结束业务回答但不证明原物理producer清理；tool_search只激活eligible名字，返回名字/描述，下一次请求发送schema                                         |
| HumanLoop            | `human_in_loop`在`human-loop`且`enable_human_in_loop`时注册                                                               | 框架请求/回答身份和Permission/HumanLoop；GUI/IM仅呈现/答复。工具审批与主动输入不同，不因已注册就证明hook Ask必达GUI                                            |
| 后台cell             | 注入`command_cells`才注册`wait`、`stop_cell`、`list_cells`                                                                | 与shell(background=true)共享registry；等待/停止是原命令观察与控制，不是第二套TaskRun CRUD                                                                      |
| Store/layered memory | store启用时装recall/search_memory；绑定layer manager后同名替换并装layered remember等                                      | MemoryLayerManager/Store拥有记忆与审计；EKO共享workspace store及热投影。不是每个Agent都必有全部memory工具                                                      |
| Subagent             | `subagent`且`register_agent_dispatch_tool`注册`agent_tool`；独立开关装`subagent_message/subagent_list`                    | 原SubagentExecutor/registry/uplink；消息工具依赖ToolContext身份，primary上不能当作有效子agent消息路径                                                          |
| 任务关系             | 框架`task_create/task_update/task_list`由`TaskRevisionService`支持；`register_task_tools`替换同名工具并同步Agent accessor | revisioned TaskRun graph是唯一任务关系权威；TaskPlan是artifact、Todo是投影；不能新增todo_write或平行plan CRUD                                                  |
| Skills/plugins       | `capabilities.rs`注册/替换activate_skill、read_skill_resource、run_skill_script；plugin生命周期复用相同registry           | 描述符、加载policy、技能脚本和原publication/cleanup owner。激活可能扩大当前eligible集合，必须在真实调用处检查，不以提示词代替限制                              |
| MCP及浏览器sidecar   | 实际连接后将server工具映射入相同manager；MCP resources有专用工具集合，EKO初始集合不直接曝光                               | MCP client/manager/preparation/transport拥有连接与清理；浏览器sidecar/session是EKO产品owner。配置存在、server可列或schema注册均不等于服务可用/物理副作用已结算 |

注册锚点：[ReactAgent额外工具][f-react]、[memory替换][f-memory]、[任务替换权威][f-tasks]、[Skills/MCP能力入口][f-capabilities]。`ThinkTool`等定义或示例不自动算入本次默认注册；动态工具集合也不能由静态表反推数量。

### 3.3 EKO 应用工具及适配

| 组                 | 真实注册与消费者                                                                                                                                     | 曝光／authority／副作用                                                                                                                                                             |
| ------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| TaskRun产品执行    | `tasks/task_runtime/register.rs`装框架关系工具、`task_execute`及`create_complex_task/check_run_status/cancel_run`；独立run driver也装ExecuteTaskTool | task_*四工具初始可见；三个complex/run助手不在初始白名单，可由tool_search激活。它们依赖chat_resources/TaskRuntime原store与执行owner；是待核对的近邻入口，不能只凭名字判死或另建store |
| Agent协作          | `agent_control.rs`装agent_list/inspect/message/followup/wait/interrupt/spawn/resume/handoff/group；AppState绑定版本替换bootstrap fallback            | 默认deferred；应用AgentControlService、workspace/conversation地址、投递/唤醒与Subagent控制各保留原owner。GUI不新增人工协作控制台                                                    |
| 研究               | runtime/AgentPool装research_library；research_connectors用AutoIngestResearchTool包装已注册学术工具                                                   | wrapper复用原Tool，摄取另写应用研究库；两处Agent构造不表示同一registry重复装两份；是否真实摄取须看当前scope及执行回执                                                               |
| command-cell watch | `register_command_cell_watch_tools`装watch_cell、interrupt_command_cell_watch                                                                        | 初始可见；TaskRuntime CommandCellService控制持续观察，不取代框架cell执行owner                                                                                                       |
| shell产品policy    | factory用`replace_tool`安装EkoCommandPolicy、sandbox与cell launcher                                                                                  | 原shell同名替换；用户交互式TerminalService与Agent自动shell不同，终端不受Agent full-auto门控                                                                                         |
| LSP（待去依赖）    | runtime::register_lsp_tools启动manager并装diagnostics/definition/references/hover/status五工具；plugin runtime/IPC/GUI还有真实消费者                 | 当前生产仍在；初始白名单未包含不能证明不可达，连接、plugin、tool_search/显式调用仍须审计。只按具名退休合同去依赖                                                                    |

锚点：[E任务注册][e-register]、[E独立run driver][e-run-driver]、[E协作注册][e-control]、[E研究适配][e-research]、[E cells][e-cells]、[E LSP注册][e-lsp]。Scheduler/TaskService构造的Agent仍消费共享factory及任务链，不因未来console退役取消其工具、HITL或恢复能力。

## 4. 曝光、预算、结果与取消边界

### 4.1 曝光与schema统计

[tool_exposure.rs][e-exposure]已经启用deferred_schemas/cursor_pagination/bounded_results/content_free_telemetry。普通初始policy包含控制、文件、目录、shell/cells、task_*、web_search/web_fetch、diff、recall/search_memory；TaskRun改用含run_code的集合；AcademicResearch额外加入四个实际已注册的学术搜索。集合始终与registered名字取交集。

`ToolVisibilityState`区分available、eligible、visible；tool_search只激活eligible工具。模型侧还过滤disabled、skills、plan/readonly；注册和可发现性不授予执行权限。[F可见性][f-visible]。`ToolManager::schema_stats_for`排序并序列化定义，统计UTF-8 bytes与HeuristicTokenizer估算；实际request也再计量。现有测试使用注册catalog≤64、首轮schema≤16,000 bytes／4,000 estimated tokens等门槛；这些是**测试断言**，本次没有运行得到实际注册数或schema字节，不调整门槛，也不把estimated tokens当provider真实计费token。

### 4.2 结果与完整读取

1. `ToolResult`保留success、output/data、typed failure、metadata及artifact等事实。ReAct `TruncationStage`把模型用文本交给统一`process_tool_output_for_call`；GUI/SDK其它载荷不因此自动满足相同预算。[F输出阶段][f-truncate]。
2. EKO factory默认模型结果预算4,000，正值配置可以覆盖；artifact默认threshold为32KiB，超token预算也可触发spill。框架其它消费者可选自己的预算。[E预算配置][e-factory]。
3. spill成功保留`ToolOutputArtifactRef{path,artifact_bytes,payload_bytes,sha256,retention}`，模型得到明确preview和完整读取指引。`read_artifact`按cursor读取、校验身份/SHA，内容页默认/上限3,500 tokens；首末页hash及UTF-8完整性有现成回归。spill失败仅为显式截断，不可声称丢失部分可恢复。[F spill][f-output]、[F reader][f-reader]。
4. GUI `ToolExecutionService::read_output`先读已记录stream，再读typed artifact页，返回next_cursor/complete；已有完整读取接口，不需再建产物store。它的页限和模型token限是不同合同，前端展开/事件payload仍需独立测量。[E完整输出][e-output]。
5. SDK direct facade捕获Session Agent的artifact policy和workspace/conversation身份，但不经过ReAct截断阶段；typed wire转换保留failure、side_effect、artifact等字段。不能向SDK直接套用EKO的4K默认。accepted `read_artifact_page`是source route，不能从SDK files family名字列表缺少read_artifact推断没有读取入口。[S direct][s-direct]、[S typed wire][s-wire]。

集合分页已由`PageRequest/PageInfo`实现：limit、query/limit/结果快照指纹、cursor错配拒绝及模型可见continuation。[F分页][f-page]。它先收集整个Vec再指纹和切页；不是扫描/内存有界的证明。web_search每页重新调用provider，结果快照变化会拒绝旧cursor；artifact reader则围绕已保存完整结果读取。两者不能混为同一种恢复保证。

### 4.3 错误、取消与副作用

ToolManager拥有schema校验、timeout/retry、read-only结果cache与并发permit；ToolContext承接scope、cancel、trace及执行身份。ReAct pipeline保留typed失败与原执行owner，逻辑terminal、observer取消或路径存在都不能作为物理执行已停止的证据。shell/RunCode、MCP、Subagent、任务journal、GUI投影各有原owner，本报告不建立共同“已完成”布尔量。

SDK facade direct路径在本次所读`ToolContext`构造中仅显式注入workspace/conversation/call/artifact，其余取默认；取消、deadline和handle teardown需在Host外层及具体工具的组合上验证，不能把共享输入校验等同整个ReAct执行政策。T8 MCP preparation cancellation及Host组合的未证范围保持未证；框架已有Finding的resolved状态不覆盖其明确排除的SDK Host/LSP adapter。[S direct][s-direct]、[F Finding边界][f-t8]。

WebSearch当前把provider Err统一投影为retryable Transient；Exa对401/402/429的文字区分不能据此推断最终typed retry政策正确。另一个需补回归的源码风险是：spill失败的fallback按每token四字符裁剪，而HeuristicTokenizer对非ASCII权重更高；UTF-8不panic不等于最终token数在预算内。这两项均未在本报告修复或关闭。[F web错误][f-search]、[F fallback][f-output]、[F tokenizer][f-tokenizer]。

## 5. SDK：三个分母与真实消费者

本次只读解析S的正式JSON，重算结果为：

| 分母                               |             静态结果 | 能证明／不能证明                                                                            |
| ---------------------------------- | -------------------: | ------------------------------------------------------------------------------------------- |
| accepted canonical `(path,kind)`   | 5,621，去重后亦5,621 | 已接受的跨语言身份；不是模型工具数，不自动扩大到全部Rust pub API                            |
| accepted facade catalog route rows |                  454 | 路由行；family route可包含多个operation，不等于454次Tool注册                                |
| catalog family rows                |                   41 | 分类表含intrinsic/source_operation/value等；不是41种模型工具                                |
| closed family operation names      |     按family分别列出 | web=3、files=14、shell=5等只表示协议operation列表；实际factory可复用同一Tool或走专门handler |

权威输入为`contracts/sdk/accepted-external-contract.json`和`accepted-facade-operation-catalog.json`，`source-contract.json`绑定兼容digest；本次不运行generator、不改117项生成物、不升级pin。完整Rust inventory和Host-only表仍属其既有非阻断遥测边界。

SDK files factory仍有6个单文件写/删/移动工具，标准pack已改为apply_patch；这是**真实消费者差异**，不是证明它们死代码。SDK web factory显式DuckDuckGo，未自动追随StandardToolPack选provider。SDK RAG index/search有session资源绑定。保留这些入口的输入、错误、实际文件副作用和typed结果，需通过SDK自己的Host/三语言回归；EKO曝光测试不能代替它们。

## 6. direct／Subagent／verification／recovery接缝

| 路径                            | 实际消费点                                                                                                                               | 本次确认与未证边界                                                                                                                                                                               |
| ------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 普通直接chat／绑定run的chat     | `chat_driver.rs:1648`选择普通或TaskRun初始工具；AgentInvocationContext→snapshot→think→ToolManager/pipeline                               | scope和初始集合真实接入；实际模型选择成功率、注册schema总量与服务调用未测                                                                                                                        |
| owned TaskRun turn              | `executor/events.rs:137`选择DomainProfile集合；原TaskRuntime operation/trace/resource guards继续拥有任务事实                             | AcademicResearch只额外曝光已注册搜索；不是任意领域工具都常驻                                                                                                                                     |
| PlanTask Subagent               | `executor/unattended.rs:1183`调用delegate_to_agent_attempt并传allowed_tools/cancel/lineage；framework executor构造invocation             | 非空allowed_tools产生实际disabled集合；`visible_tools: None`意味着并未采用EKO主调用的渐进曝光，仍经角色/skill/readonly等资格过滤，不能宣称预算对等。[F子执行][f-subagent]                        |
| primary verification Agent turn | `executor/unattended.rs:1497`为其建立TaskRun初始可见集合并传入invocation                                                                 | 只确认这条Agent验证路径；显式命令验证不等于模型工具请求，不因共用factory就宣称全部verification/SDK路径预算一致                                                                                   |
| boot/recovery                   | `task_runtime/boot_reconciler.rs`、原journal/claim/barrier→run driver；重新进入Agent时依旧消费factory与invocation                        | 恢复事实与重新执行分开；未知物理工作不重放，artifact按原引用读取。没有本次跨重启工具消费回归，不能把恢复类型存在当作完整验收                                                                     |
| SDK direct facade               | family Tool：session authority→factory→共享输入校验→execute_with_context→typed wire；source operation由专用handler解析参数并直接调用原语 | family Tool与source operation也分开；read_artifact_page由专用handler调用reader并snapshot_value，不走Tool输入校验；Host取消/错误/资源释放范围需自己的组合证据。[S source reader][s-source-reader] |
| GUI展开结果                     | ToolExecutionService→stream/artifact page→next_cursor/complete                                                                           | 复用现有reader；GUI首次渲染、长列表和大结果事件体积仍交性能基准                                                                                                                                  |

EKO ADR0034的嵌套委派/协作约定与ADR0039的单层Subagent、移除子层执行工具约定存在政策差异；当前factory仍依据`can_delegate`注册agent_tool，TaskRun还传NestedDelegationPolicy。本报告只呈现这组文档与实现事实，交后续builtin/domain合同对账，不把任一旧ADR自动当作本轮新政策，也不因提示词说明替代实际限制。

## 7. A2A/LSP消费者先去依赖的精确清单

当前代码和feature仍存在。已批准的是具名退役方向，不是“EKO没用所以框架死代码”。ACP、MCP、channels、Subagent、shell和SDK Host stdio继续保留。

| owner／文件                                                                                                                        | 后续去依赖或退役对象                                                                    | 必须保留的接缝                                                                  |
| ---------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- |
| EKO `Cargo.toml`、`echo-agent-app-core/Cargo.toml`                                                                                 | LSP feature请求；以真实manifest核对，A2A不凭名称追加删除                                | GUI/IM/cron所需feature与framework依赖关系                                       |
| EKO `runtime.rs:2031`                                                                                                              | register_lsp_tools及5个工具注册、服务器初始化                                           | AgentPool、MCP、任务、插件、恢复和shutdown其它步骤                              |
| EKO `plugin_components.rs`、`plugin_runtime/{types,runtime,publication}.rs`、`extension_commands.rs`                               | lsp_configs、PluginLspRuntime/binding、prepare/reload/rebind/cleanup、旧插件请求        | 原generation publication、receipt、rollback及其它合法组件；旧请求要明确退役诊断 |
| EKO `src/tauri/commands/lsp.rs`、`src/tauri/mod.rs`、LspPanel、SettingsDialog、endpoints、uiStore、类型投影                        | IPC/UI/配置读取与宣告                                                                   | 生成类型由唯一入口更新；console产品入口退役另交付，不能连带删除IM adapter       |
| SDK `echo-sdk-host/Cargo.toml`、`src/features.rs`、`core_profile/facade/integrations.rs`、Protocol facade与三语言包装              | framework-a2a/lsp、full facade成员、资源/stream/handler/advertisement及精确accepted身份 | 仍使用原pin即可先去依赖；不能删整个bridge/integrations或强迫升级framework       |
| framework `Cargo.toml`、`src/a2a/`、`src/lsp.rs`、`echo-core/src/lsp/`、`echo-integration/src/lsp/`、`src/tools/lsp.rs`及re-export | 在EKO/SDK消费者验收后物理退役、同步矩阵/示例/正式文档                                   | 合理其它pub工具/Store/压缩器不按EKO使用量删除；官网只在真实交付后对账           |

S正式accepted selector重算：A2A为160 identities／9 route rows／9 closed operation names；LSP为106／14／30。A2A的160含47个intrinsic-route身份，不能只筛`route.family == a2a`。这些是后续生成对账的候选分母，**本次删除数为0**。逐身份清单复用S的退役边界文档，不复制成新的accepted权威。

## 8. Exa本机独立来源与漂移

精确本机来源为`4343f0b736beb5c8606e31146528a2fe8be80510`，保留分支`feature/Echoyue/exa-web-search`。本次核对local ref与Git对象，不push、不merge、不cherry-pick；没有远端可恢复性或真实服务通过的声明。该来源按本地`git show <SHA>:echo-tools/src/web/providers/exa.rs`读取，不提供虚构的公开blob链接。

| 项目         | 当前F／SDK事实                                                                                     | Exa独立提交                                                                                                                         |
| ------------ | -------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| 接口         | SearchProvider→`Vec<SearchResult{title,url,snippet}>`；共享WebSearchTool分页                       | 沿用该接口；highlights归一化为snippet                                                                                               |
| provider选择 | StandardToolPack显式DuckDuckGo；auto另支持Tavily→Brave→DuckDuckGo；SDK web factory也显式DuckDuckGo | auto优先Exa，再Tavily/Brave/DDG；标准pack改调用auto；构造时选择，失败不静默换provider                                               |
| 搜索与抓取   | web_search负责检索，web_fetch负责读取URL                                                           | 不改web_fetch，不把搜索摘要当完整网页                                                                                               |
| 漂移         | 对Exa涉及17路径，将F与Exa parent比较                                                               | 14个原有文件字节一致，3个新增文件在F仍不存在；这证明所涉路径无后继文本漂移，不证明可编译/无合同影响                                 |
| 错误与结果   | WebSearchTool最终Err投影和分页仍为共享代码                                                         | provider限制结果数、非空key、15秒client timeout、bounded HTTP body，错误文字不包含上游响应体；最终typed重试分类仍须§4对账           |
| 验证         | 本次只读确认测试定义                                                                               | provider单测及`echo-tools/tests/exa_search.rs`含mock、UTF-8、畸形响应、错误与child-process优先级用例；本次未执行，不当作真实Exa验收 |

Exa提交不属于当前保留主线工具治理必须合入的条件。即使未来单独获准采用，SDK的独立pin和显式provider factory仍须单独评估，不能随框架default变化宣称SDK已支持。

## 9. 可行动的后续切片与验证入口

| 后续Outcome／最小切片                | owner与精确接缝                                                                   | 已有入口／仍需证据                                                                                                          |
| ------------------------------------ | --------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| a2a-lsp-consumer-decoupling          | EKO runtime/plugin/manifest；SDK feature/accepted/handlers，由各唯一owner串行合流 | 消费者仍在旧框架时可构建；剩余family、真实Host、三语言与MCP等保真，不整包删E2E                                              |
| builtin-tool-convergence：结果回退   | F snapshot.rs与pipeline测试                                                       | 扩展truncation_stage_is_utf8_safe／spill不可写用例，断言最终token上限；本报告未修复                                         |
| builtin-tool-convergence：曝光一致性 | E tool_exposure/factory、F subagent/executor、验证invocation                      | 在invocation_schema_budget_does_not_regress基础上补Academic、激活后、Subagent、验证/恢复实际request；先对齐ADR0034/0039政策 |
| builtin-tool-convergence：近邻入口   | 标准文件pack vs SDK六类写工具、EKO complex/run助手、diff/git_diff                 | 先按输入/副作用/消费者证明可替代，再按所属合同收敛；本次不直接判死或新增平行CRUD                                            |
| builtin-tool-convergence：Web失败    | F web/search.rs与provider mocks；Exa继续独立                                      | 认证/额度/限流/网络错误在最终ToolResult和重试路径的分类；不用provider文字断言代替端到端结果                                 |
| performance-baseline                 | 单独测量会话；只使用现有权威与fixture                                             | 冷/热会话与任务查询、journal完整open、memory完整写入、GUI渲染、schema与大结果；记录同源规模、分布/RSS/I/O，不发明阈值或收益 |
| T8/Host组合                          | framework MCP owner与SDK Host owner                                               | 复用prepare/cleanup receipt回归和Host teardown入口；缺少跨层取消/实际消费证据的范围继续未证，不用某个Finding状态整体关闭    |

现成focused入口还包括：`tool_search_activates_only_eligible_matches`、pagination的query/limit/snapshot mismatch回归、`q_flt_v10_paged_spilled_result_is_fully_recoverable`、pipeline的spill/trace/guard回归、SDK `tool_behavior_governance_e2e`与`facade_feature_adapters_e2e`。它们本轮只读确认，Rust编译租约归GUI会话；本任务无Cargo产物，`cargo clean`不适用。

## 10. 本次交付验证边界

报告通过独立事实复审后，仅提交本文件。直接验证重新核对三仓代码增量、正式SDK分母/退役selector、Exa对象/ref/17路径差异、固定源码链接的路径与行号、Plan/Design/map哈希、Markdown与`git diff --check`，并运行专用工作树task范围语义验证。真实退出、初始诊断及修正重跑保存在本工作树ignored `.echo-semantic/local/tool-inventory-and-exa-boundary/`；这些日志不是正式语义对象或新的生产权威。

文档同步判断：新增的是顶层跨仓阶段审计材料，生产实现、公开API、配置、示例和官网宣传均不改变，故不修改子仓正式文档、examples或官网；后续实际退役/工具变更仍必须按各仓规则同步。本地报告提交不代表Plan lifecycle完成、五仓main/CI完成或发布；Plan/map及协调者生命周期保持只读。

[f-react]: https://github.com/EchoYue-lp/echo-agent/blob/51726454985fd6f064b21a7e9c539e22d494c987/src/agent/react/mod.rs#L664
[f-manager]: https://github.com/EchoYue-lp/echo-agent/blob/51726454985fd6f064b21a7e9c539e22d494c987/echo-execution/src/tools.rs#L294
[f-visible]: https://github.com/EchoYue-lp/echo-agent/blob/51726454985fd6f064b21a7e9c539e22d494c987/src/agent/snapshot.rs#L558
[f-think]: https://github.com/EchoYue-lp/echo-agent/blob/51726454985fd6f064b21a7e9c539e22d494c987/src/agent/react/run/phases/think.rs#L741
[f-pack]: https://github.com/EchoYue-lp/echo-agent/blob/51726454985fd6f064b21a7e9c539e22d494c987/echo-tools/src/registry.rs#L28
[f-pack-tests]: https://github.com/EchoYue-lp/echo-agent/blob/51726454985fd6f064b21a7e9c539e22d494c987/echo-tools/src/registry.rs#L274
[f-memory]: https://github.com/EchoYue-lp/echo-agent/blob/51726454985fd6f064b21a7e9c539e22d494c987/src/agent/react/mod.rs#L1449
[f-tasks]: https://github.com/EchoYue-lp/echo-agent/blob/51726454985fd6f064b21a7e9c539e22d494c987/src/tasks.rs#L15
[f-capabilities]: https://github.com/EchoYue-lp/echo-agent/blob/51726454985fd6f064b21a7e9c539e22d494c987/src/agent/react/capabilities.rs#L960
[f-truncate]: https://github.com/EchoYue-lp/echo-agent/blob/51726454985fd6f064b21a7e9c539e22d494c987/src/agent/react/run/pipeline.rs#L1362
[f-output]: https://github.com/EchoYue-lp/echo-agent/blob/51726454985fd6f064b21a7e9c539e22d494c987/src/agent/snapshot.rs#L2787
[f-reader]: https://github.com/EchoYue-lp/echo-agent/blob/51726454985fd6f064b21a7e9c539e22d494c987/echo-tools/src/files/artifact.rs#L683
[f-page]: https://github.com/EchoYue-lp/echo-agent/blob/51726454985fd6f064b21a7e9c539e22d494c987/echo-core/src/tools/pagination.rs#L120
[f-search]: https://github.com/EchoYue-lp/echo-agent/blob/51726454985fd6f064b21a7e9c539e22d494c987/echo-tools/src/web/search.rs#L127
[f-tokenizer]: https://github.com/EchoYue-lp/echo-agent/blob/51726454985fd6f064b21a7e9c539e22d494c987/echo-core/src/tokenizer.rs#L50
[f-subagent]: https://github.com/EchoYue-lp/echo-agent/blob/51726454985fd6f064b21a7e9c539e22d494c987/src/agent/subagent/executor.rs#L3333
[f-t8]: https://github.com/EchoYue-lp/echo-agent/blob/51726454985fd6f064b21a7e9c539e22d494c987/.echo-semantic/findings/finding.extension-cleanup-settlement.md#L41
[e-factory]: https://github.com/EchoYue-lp/echo-agent-cli/blob/c262cfed8e77abed38a27c1fff93547997ec9486/echo-agent-app-core/src/infra/factory.rs#L61
[e-chat]: https://github.com/EchoYue-lp/echo-agent-cli/blob/c262cfed8e77abed38a27c1fff93547997ec9486/echo-agent-app-core/src/chat_driver.rs#L1648
[e-exposure]: https://github.com/EchoYue-lp/echo-agent-cli/blob/c262cfed8e77abed38a27c1fff93547997ec9486/echo-agent-app-core/src/tool_exposure.rs#L17
[e-register]: https://github.com/EchoYue-lp/echo-agent-cli/blob/c262cfed8e77abed38a27c1fff93547997ec9486/echo-agent-app-core/src/tasks/task_runtime/register.rs#L40
[e-run-driver]: https://github.com/EchoYue-lp/echo-agent-cli/blob/c262cfed8e77abed38a27c1fff93547997ec9486/echo-agent-app-core/src/run_driver.rs#L145
[e-control]: https://github.com/EchoYue-lp/echo-agent-cli/blob/c262cfed8e77abed38a27c1fff93547997ec9486/echo-agent-app-core/src/agent_control.rs#L2299
[e-research]: https://github.com/EchoYue-lp/echo-agent-cli/blob/c262cfed8e77abed38a27c1fff93547997ec9486/echo-agent-app-core/src/research_connectors.rs#L855
[e-cells]: https://github.com/EchoYue-lp/echo-agent-cli/blob/c262cfed8e77abed38a27c1fff93547997ec9486/echo-agent-app-core/src/tasks/task_runtime/command_cells.rs#L1781
[e-lsp]: https://github.com/EchoYue-lp/echo-agent-cli/blob/c262cfed8e77abed38a27c1fff93547997ec9486/echo-agent-app-core/src/runtime.rs#L2031
[e-output]: https://github.com/EchoYue-lp/echo-agent-cli/blob/c262cfed8e77abed38a27c1fff93547997ec9486/echo-agent-app-core/src/tool_execution.rs#L617
[s-tools]: https://github.com/EchoYue-lp/echo-agent-sdk/blob/0930eef9f878cf436ed88384030b6bc03d6f6c53/echo-sdk-host/src/core_profile/facade/tools.rs#L206
[s-direct]: https://github.com/EchoYue-lp/echo-agent-sdk/blob/0930eef9f878cf436ed88384030b6bc03d6f6c53/echo-sdk-host/src/core_profile/facade/tools.rs#L438
[s-wire]: https://github.com/EchoYue-lp/echo-agent-sdk/blob/0930eef9f878cf436ed88384030b6bc03d6f6c53/echo-sdk-host/src/core_profile/wire.rs#L382
[s-source-reader]: https://github.com/EchoYue-lp/echo-agent-sdk/blob/0930eef9f878cf436ed88384030b6bc03d6f6c53/echo-sdk-host/src/core_profile/facade/source_operations.rs#L3871
