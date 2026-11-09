# 主根收敛交付对账（2026-10-26）

本文件只记录跨仓阶段性工程事实与剩余动作，不替代 Design、Delivery Map、产品文档、TaskRun 权威或用户授权。统计对象是当前 [Delivery Map](../../specs/eko-gui-product-convergence/plans/delivery-map.md) 的 **25 个 Outcome**，不是旧 MASTER 后文的历史 SDK 验收表。本轮仅写本文件，不改业务代码、MASTER、Map、Plan、预检、Git index/refs 或外部 Issue。

对账截点：2026-10-26 07:03–07:12 UTC 的本机只读观察。epoch08 正在编译，GUI 与归档线允许继续写自己的文件，以下结果不为截点后的源码自动续期。子代理调用 `get_goal({})` 实际返回 `goal:null`；协调者确认这是子代理不可见主线程 Goal 的作用域限制，因此依据本线程用户持续确认的完整 Goal 工作，不能据此宣称 Goal 不存在、暂停或完成。

## 1. 当前根、证据边界与已成立事实

唯一继续开发根为 `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent`。外部工作树是冻结迁移来源，不再是新实现或新验证入口。

| Git 根 | 本机实际 HEAD / 分支 | 当前含义 |
| --- | --- | --- |
| superproject | `a91d5deb8372a37388cf4e6106885345d073c14c` / `chore/Echoyue/workspace-maintenance` | 承载跨仓资料；未因子仓切分支而切换自身分支。新输入尚未形成新的顶层交付提交。 |
| `echo-agent` | `9c5e202dcbd20a7de3c017a3ee8adada3e400b86` / `chore/Echoyue/root-workspace-convergence` | 本地已合格组件的集成提交，含工具说明预算修复与默认关闭的微信测试支持；组件同字节采用和各自工程资格不能替代新 EKO consumer 验证。 |
| `echo-agent-cli` | `780ddd0d25c2eac5d1471cb3e5012c9d2f6b3cf9` / `chore/Echoyue/root-workspace-convergence` | HEAD 是已有联合候选；其上迁入 Project/Domain/SSH/微信与任务列表等未提交成果，并有新的根内修改。HEAD 不是当前全部源码身份。 |
| `echo-agent-sdk` | detached `0930eef9f878cf436ed88384030b6bc03d6f6c53` | 保留基线检出；SDK 历史合格 main、消费者退役候选与新 framework pin 候选分别对账，不能由此根指针推导已采用。 |
| `echo-website` | detached `a926ef7cf10bdc03cc0a19b2e406cc6164477485` | 保留基线检出；新产品文档镜像候选及部署均不能由它证明。 |

当前可确认的工作与限制：

1. [根源码承接清单](../../../../.echo-semantic/local/workspace-convergence-20261026/convergence-manifest.json) 保存 Project 与 Task 两组原 patch/未跟踪清单；15 个交叠文件另有 `overlap-reconciliation.json`。这是来源与整合记录，不是当前根全量编译、运行或恢复通过证明。
2. 根 EKO 旧 `root-convergence-rust/run.json` 中 `root-queued-replacement` 实际退出 **101**；新 epoch08 以显式 `channels` feature 重验，不能删掉旧失败或把“已经开始”写成通过。
3. 当前风险重绑定为用户已确认的 consumer digest `b0b83f384877d9ab3b8bdaaa403cd0bd439e6bc9be04106209282ab8bd0fc93f`、lock `3fb8955b56e86eb4bef2454d3328b6ef8eb7f36d7663806387f3b990244e7ed6`、856 packages、policy `03cf671a288612d6d57284fa99505d296a8f31c81323371b40dce084a4b52e40`。epoch08 的 `policy-binding.json`、`reassessment-candidate.json` 与执行脚本可定位此输入；原同六项身份保持是有限合同，**不是 scanner、完整工程或 P1 已通过**。
4. epoch08 计划队列是 `queued_replace` 定向回归及 exact `p1_clean_cold_ready_drives_and_sends_once_then_restart_is_quiet`。截至上述观察，只有编译中的 `queue.log`，尚无 `queue-run.json`、`p1-run.json`、`batch-receipt.json`，因此不填写 selected/pass 数或 Cargo clean 结果。最终以同目录自然退出及准确 target 清理回执为准，不能重用外部 epoch07。
5. 根 GUI `steer-goal-controls-root/node-run.json` 已有实际 2 文件 **37 tests pass**、全前端 noEmit TS 0、局部 lint 0、format write/check 0、docs parity 0（69 pairs / 57 ADRs）；首个 fixture 失败和第一次 TS 2 均保留。它只证明这些根内命令及其执行时源码，不证明原生操作、完整 Node 套件或其后新修改。
6. 外部微信 epoch07 因开发根迁移中断，未进入 P1，专用 target 已清理；中断既不是 P1 pass，也不是该 P1 行为失败。新根验证必须产生自己的结果。

本文件使用的证据层级：**历史交付**只能证明其具名历史范围；**组件 qualified / 源码已迁入**不能改称当前根集成通过；**当前根 focused**不能改称全量门禁；**原生/真实服务/main CI/发布**各需独立原件。未联网刷新，以下不报告任何远端状态为实时事实。

## 2. 全部 Outcome 的当前层级与行动顺序

“未闭合”不表示从零重做：已有代码、合同与限定验证继续保留；本表只指出当前根和完整验收还缺什么。没有任何条目凭本次 Markdown 对账被改为 lifecycle completed。

| # | Outcome | 已完成的资料 / 已有源码与限定结果 | 当前缺口、阻塞与下一动作 |
| --- | --- | --- | --- |
| 1 | `product-boundary-contract` | GUI 主入口、IM/cron 保留、A2A/LSP 与独立 console 退役方向已由用户确认；保留线、ADR 与产品合同有历史来源。 | 根顶层 `AGENTS.md` 仍有旧 TUI/GUI 对等和默认保留 Cargo cache 条款；用户后继方向与逐批准确 clean 指令优先。由协调者同步根与相关子仓规则，不能让旧规则反向取消已确认目标；本文件不改规则。 |
| 2 | `tool-inventory-and-exa-boundary` | [工具清单](../../specs/eko-gui-product-convergence/audits/tool-inventory.md) 已形成正式阶段材料；Exa exact `4343f0b736beb5c8606e31146528a2fe8be80510` 持续本地 hold。 | 清单固定旧 F/E/S 输入，必须用后继差分解释已修项，不把旧风险重复排成待实现；其本地报告完成不代表当前五仓 main/CI 或工具治理全部完成。Exa 不列“下一步必须合 main”。 |
| 3 | `performance-baseline` | 旧来源已有真实 fixture、若干测量、12 个完整输入和原失败/中断记录；1k memory 独立诊断及两次短采样另存，未替代基准。 | 完整原规模/场景仍未闭合；失败 10k memory、tools partial 不能冒完整数据集，1k 诊断不能外推 10k 或写进正常样本分布。保全原件后按正式同源根快照继续，性能测量与重型编译错峰。 |
| 4 | `gui-visual-foundation` | 主题、Modal、基础布局已进入既有 EKO 候选并可复用；根前端有局部检查。 | 常用/最小窗口、浅/深色、键盘/焦点及真实 Tauri 视觉反馈仍需当前根证据；截图/浏览器/旧 Node 不能替代完整原生验收。 |
| 5 | `gui-home-navigation` | 首页、工作空间/普通任务区分、已有导航与身份 fence 的源码和限定原件存在；不从零重做。 | 当前根导航与新 composer controls 的同源组合回归、真实窗口视觉和用户反馈仍待；不能仅凭旧首页分支绿关闭。 |
| 6 | `steer-responsiveness` | S1 fence、S2 工具安全点、S3 durable ack/草稿身份已有后继源码；根新增 queue 编辑/排序与 Goal 控件，37 项根前端检查通过。 | 完整有效范围已扩为 composer 上方的多 Steer 编辑/排序/删除/立即执行及唯一 Goal 修改/删除；见第 3 节漏链。当前根 Rust/CAS/恢复与精确 P1 尚待 epoch08，另需运行中原生验收；Persisted、Drained、实际副作用结算继续分开。 |
| 7 | `im-cron-gui-control` | 原 Channel/Cron 框架组件、EKO runtime/GUI 接线已有候选，根整合承接其有效内容。 | QQ/Feishu/微信及 cron 真实账号/实例/occurrence 控制、隐藏/退出和失败恢复仍要当前根组合证明；单个框架选测不替代 EKO GUI 与实际投递。 |
| 8 | `console-surface-retirement` | R1/R2 接管与 R3 原子删除、R4 feature/依赖/锁/CI/docs 准备均有具名来源。 | 当前根仍保留独立 TUI/CLI/JSONL 产品入口；不可把描述方向或删除候选写成已退役。先完成 GUI/IM/cron/恢复接管和当前门禁，正常删除门禁仍不得绕过；依赖消失须重评实际风险集合。 |
| 9 | `a2a-lsp-consumer-decoupling` | EKO consumer 去 LSP 的既有源码已在后继候选；SDK 独立 `1eac40fb…` 有精确 accepted 差集与原 pin 资料。 | 根 SDK 0930 未由新指针自动采用，双方消费者/剩余能力、Host/三语言与当前主线顺序仍需分别结算。不得仅因 EKO 没用就删除合理框架 API。 |
| 10 | `a2a-lsp-retirement` | 物理退役 dirty 候选和限定工程回执已保留。 | 最终删除语义门禁原自然 1 仍是外部阻塞；用户选择排除外部 echo-semantic 修复没有授予跳过。根 F9c5 也不等于已经采用删除候选；不关闭旧 Issue/Finding。 |
| 11 | `builtin-tool-convergence` | F9c5 已含后继 typed Web failure、spill fallback token 计量和工具说明预算修复；独立组件工程/语义原件保留。 | 当前根实际 consumer chat/task 预算及 Academic/激活后/Subagent/验证/恢复的真实曝光证据仍需具名核对；候选 producer 合格不是当前根 consumer 合格。下一可实施切片见第 4 节。 |
| 12 | `hook-ask-semantics` | 强制交互意图修复与框架接管已有后继；不是待重新设计权限层。 | 当前根 GUI/IM 与既有 HumanLoop 的组合、拒绝/无人可答/超时/取消和精确请求身份仍需所用快照验证；不凭历史 Hook 绿覆盖后继 consumer。 |
| 13 | `gui-hitl-dialogs` | 统一弹窗、队列/来源/焦点和身份修复已有 EKO 候选。 | 当前根审批/输入/选择、多会话与 IM 竞争、过期/取消及真实窗口证据未整体签收。不能由新增 Steer 控件影响 HITL 回答路径。 |
| 14 | `task-list-dialog` | 正式 Plan13 与列表/单历史快照、有界 run/event 页、typed receipt 候选已迁入；原 v1 两个前端 Important 已在 v2 源码闭合，旧票保留。 | 根整合后还需独审与实际 Node/Rust/生成 DTO 检查；合法图 1/10/50/100、run 10/100/1000、独立事件 10k/100k、同源查询/渲染和 Tauri 验收仍待。脱离 memory/tool 整包等待不减这些义务。 |
| 15 | `ssh-remote-terminal` | 系统 OpenSSH/PTY/终端底座与对应源码存在，目标已具名。 | `echo@111.229.245.175` 的旧只读 BatchMode 255 不是 GUI 认证通过；当前根还需用户在 GUI 完成必要认证，并验 resize/Ctrl-C/全屏/多会话/断线/退出。无需再索“目标尚未指定”。 |
| 16 | `project-workspaces` | 三来源项目、通用 FS 接缝、执行/上下文/证据及 SSH/SFTP 候选已在根承接；正常 host/guard、取消未知沿原 owner。 | 根合并后完整工程、三来源创建/重开/切换、同名规则来源隔离及真实远程 FS/exec/Git/isolation/artifact 均需同源验收；独立终端能登录不替代远程项目通过。 |
| 17 | `domain-plugin-runtime` | 原 registry/dispatcher、same-binding admission、lease/resource_guards 和 Project/Domain 联合候选已有源码及历史原件。 | 新根共享源码、前台/Subagent/verification/resume 和 supervised retire 等验收仍待；旧 780/独立 producer 的限定通过不得自动为根 dirty 源续期。保留 General 能力及既有唯一运行时。 |
| 18 | `measured-storage-optimization` | 旧源码调查已定位重复 reconcile/audit/持久化工作；采样证实多个路径都实际执行。 | 未完成正式根同源基准和前后对照，不宣称主导比例、回归或已获得优化收益；不抬 watchdog、缩数据量或默认引入 SQLite。真实无瓶颈项可以交付测量结论。 |
| 19 | `new-im-channel` | 用户选定个人微信，Rust adapter、GUI 扫码/账号 owner、恢复与投递候选已存在；F 默认关闭的测试支持在9c5。 | 当前关键关口是根 epoch08 queue/P1 及其后真实扫码、direct 收发、HITL、重登/断连和 cron 结果投递。HTTP200、token存在、fixture/P1成功都不能单独冒真实微信收到。 |
| 20 | `legacy-resource-cleanup` | 60 view 的源/patch/refs/stash 保存与五 bundle 独立恢复已有回执；45 外部 view 的 ignored 材料已归档读回，见第5节。 | 本次归档切点未执行剩余 view 删除；41 个 native 候选、4 个 host-managed 及根内15 view 分别处理。由清理线 fresh guard、恢复导航/依赖和准确宿主方法结算，不从“备份完”推导“都已删除”。 |
| 21 | `issue-finding-reconciliation` | 原本地审计记录 #188、#189、A2A #32–35、R1/R2/T8、PR18 和旧 SDK Plan06 的不同结论。 | 正式当前根副本/前向链接尚未承接该审计；原文件仍在冻结来源/恢复包。外部状态未刷新，#188本地 resolved 不等于外部 closed；#189来源缺口不是 risk_accepted，删除批准不等于旧缺陷修复。 |
| 22 | `release-target-selection` | “包发布/官网部署按需且独立授权”边界已明确。 | 版本、平台/架构、具体包/注册表、签名与网站部署目标仍未形成具名完整验收范围；不以本机提交代替用户对外部发布的授权。 |
| 23 | `release-artifact-qualification` | 历史保留制品和独立组件二进制可恢复，不等同本次发行制品。 | 依赖本轮所选产品结果与精确源码/锁、本地全量及 main CI、安装/启动/配置、平台/架构签收；当前未开始完成结算。 |
| 24 | `package-publication` | 未执行本轮注册表发布。 | 等具名制品验收及明确外部发布授权；不为完成 Goal 擅自发包。 |
| 25 | `website-deployment` | 官网正式文档镜像存在独立候选，根仍为旧保留检出。 | 先按最终合格子仓源码统一镜像与构建，实际部署/访问和版本核验另需目标及授权；不发布尚未验收能力。 |

## 3. 有效要求链：当前仍缺的五类闭合

当前明确有效的用户要求不能在恢复时只剩“做了哪些代码”的摘要。Delivery Map 负责结果与依赖、Plan 负责实施切片；两者都不等于完整用户原文账本。以下缺口是工程追踪义务，不创建新的运行时状态机，也不要求用户重复批准已经确认的产品方向。

### 3.1 完整原文与唯一定位

尚缺一份能逐条定位**原用户消息、原文完整内容、时间/身份以及适用范围**的有效要求清单。当前地图已覆盖25个结果，但仅有工作摘要和现有文档链接，不能证明后续每条修正、保留条件、拒绝项均已逐条承接。未取得的原文不得按记忆补写成引号内的用户原话；本文件也不声称已审查全部聊天历史。

应保留的明确条目至少包括：GUI主入口与退役范围、IM/cron、Exa本地hold、唯一开发根、逐批准确clean、GUI/任务/Workspace关系、远程项目与认证、HITL、Steer的持久事实和副作用边界、多Steer编辑排序删除/立即执行、唯一Goal修改与结束运行、完整原生/高量验收、风险精确绑定和外部工具修复排除、源码/历史/忽略材料可恢复清理，以及按需发布的独立授权。原文取得后再逐项补身份，不由这些主题名替代原文。

### 3.2 替换关系与仍然有效的约束

已有典型替换必须显式连起来：

- 旧“TUI/GUI长期对等”被后继“GUI主入口、保留IM/cron、退役独立console”取代，合理Framework/SDK能力仍保留。
- “侧栏任务=项目”的早期表述被“Workspace=项目；普通任务=Conversation；正式执行=TaskRun”纠正，SSH终端底座也不再冒远程项目完成。
- 旧多WT开发被本次唯一主根要求取代；外部源的旧工程结果保留原范围，不自动迁成主根通过。
- 旧输入摘要与后继 composer 上方的多Steer/唯一Goal交互必须合并到同一有效要求链；Steer不是新建Goal/Plan，手动Goal编辑则复用原revision/CAS，二者不能混淆。

目前主根 map 的 `steer-responsiveness.ships` 已包含新交互，但 **Plan12仍是旧S2/S3三个Todo**：没有queue编辑input-revision与queue-revision双CAS、Goal唯一/修改/结束的明确Todo与验收，`out_of_scope`又仍写“不新增…Goal/Plan revision”。这不能被误读为取消用户后继批准的手动Goal操作，也不能把新控件的测试计为旧Plan验收已经完整。需在同一Outcome正常刷新Plan12或记录精确已确认的增量实施合同，明确“复用已有revision”和“不由Steer新建revision”的边界。Plan13仍是列表/历史读取结果，不应悄悄吸收新输入或Goal执行权威。

本次 map 新增 ships 后，Design 的Steer段与 Plan12 digest/旧正文也未同时承接新增交互。当前文件身份见第6节；这条正式链待对应owner统一更新并正常验证，本文件不代改或伪报已一致。用户明确授权下的独立根内源码工作可以并行，不因补文档而重新索取同一产品批准。

### 3.3 要求到证据的逐项对应

每条有效要求仍需绑定 `Design/Map/Plan/Todo → 真实调用路径 → 原反例/正常控制 → 当前根source/lock/命令/自然退出 → 原生或真实服务证据`。没有直接证据的格子应保持未验；一个测试通过不能同时结算相邻功能，任何版本变化只复用可证明不变的范围。

近期重点反例是：编辑/排序/删除与派发竞争、同input_id重试、input_revision/queue_revision冲突、channel-bound输入不能被GUI篡改或夺取；立即执行不能抢占已启动工具或重放未知副作用；唯一Goal与新revision使旧计划/证据失效；结束Goal应走原取消/结算而不是抹除事实。原生还需体现实际composer位置、正常与最小窗口、拖拽与键盘、草稿/焦点/错误和用户可理解的回执。

### 3.4 压缩与恢复

MASTER当前章能恢复总方向，但当前缺少“完整有效要求+替换/保留关系+未验原件+下一动作”的统一恢复入口。未来上下文压缩或接管应先重读当前主根MASTER、唯一Design/Map及所选Plan，再读取对应source/回执；不得仅凭agent summary、旧完成计数、旧source digest或`ready`状态恢复。

归档索引解决字节恢复，不自动解决“哪份要求已被谁替换”的语义恢复。恢复包必须保留原消息定位、未裁决项、失败/中断原件和代码采用关系；不能把旧external source目录当当前Git查询根，也不能把backup当Git仓。

### 3.5 独立漏项检查

本轮对照 map 的25项已逐行覆盖，但**尚未完成独立的全部有效用户要求漏项检查**。本文件作者自查、此前Plan/source独审、部分测试绿都不能替代它。后续独立reader应以可定位原文和替换关系反向检查25项及新增交互，区分“未实现”“未测”“已被替代”“真实外部输入待提供”；只报告具名漏项，不根据旧Plan ready重开已交付R1/R2、SDK拆分或718旧补丁审查。

## 4. 下一条工具治理切片：当前根实际曝光验证

依据 [tool-inventory §4、§6、§9](../../specs/eko-gui-product-convergence/audits/tool-inventory.md)，下一可直接实施的最小切片应是：**先在当前根冻结consumer上复验真实factory的chat/task可见schema及按需激活边界，再扩一条具名Subagent/verification实际模型请求控制**。这是验证与证据补齐；只有出现真实失败后才定位最小代码修复，不先发明新工具、registry或validator。

已有接缝和验证入口：

- EKO `echo-agent-app-core/src/tool_exposure.rs` 的 `initial_visible_tools`、`initial_visible_tools_for_task_run`、`invocation_schema_budget_does_not_regress`；已有测试通过真实Agent构造/任务工具注册取得definitions与registered，分别核chat/task原16,000 bytes / 4,000估算token约束，合理deferred工具仍可发现。
- Framework `ToolDefinition::from_tool → ToolManager::schema_stats_for` 及原实际invocation/model request路径；read-only/role/allowed-tools硬上界和 `tool_search` 的eligible/visible含义继续由原机制决定，不能把registered集合当初始schema或用提示词取代执行限制。
- 新增一条实际模型请求控制可选择现有TaskRun primary verification，或原Subagent invocation，但应只选明确owner与既有fixture能支撑的一条；记录默认、激活后、角色零交集、迟到注册及恢复的真实范围，不把单测覆盖推广到全部路径。GUI depth默认1/可选1..3已确认，不能为预算重新选择委派策略或把Framework max4控制改为EKO常量。

需要保留的后继差分：清单固定旧c3b/0de/961，当前F9c5 `web/search.rs` 已保留 `ToolFailure::from_error`/HTTP status，`snapshot.rs` spill fallback 也已有原tokenizer计数约束，因此不能再把旧“所有provider错误统一Transient”和“每token四字符fallback”列为从零待修。当前工具说明压缩也已是F组件，原394focused等只是producer范围；静态预算17B余量不等于当前根factory或所有模型路径已绿。

本切片不合并Exa、不删合理Framework/SDK公开工具、不调整预算阈值、不接受新依赖风险，不展开SDK/P1C/Tool17旧审计或被排除的外部治理工具修复。SDK真实Host/HTTP401/429、T8 preparation取消等未证边界分别保持，不由本切片关闭。当前epoch08尚占Cargo，源码/测试定义可以由其owner按不重叠路径推进，实际编译/测量等协调者安排资源后运行。

## 5. 工程清理、Issue/Finding与外部动作

新一轮根内备份已经保存60个registered view的patch、非忽略源码、37个唯一linked/detached HEAD、314个refs对象及18条stash reflog；五个bundle自然verify与独立对象恢复均通过。该数字是备份切点，不能冒当前registry总数。

外部ignored归档进一步覆盖45个view、47个gzip/tar、100,726成员、6,283,074,656 logical bytes（压缩1,484,242,312B），包括12个历史二进制和失败原件，逐成员解码SHA读回及冻结源复核已记录。`ignored-evidence/README.md`、`archive-index.json`、`not-saved-classification.json`、`symlink-recovery-index.json`及`reclaim-plan.md`解释恢复与排除范围；本轮不重读大包或重跑其核验。

这些资料仍不等于所有worktree已回收。41个native候选需逐项fresh来源/读者/真实入边/宿主与恢复条件；4个host-managed必须走宿主生命周期；根内15个view不在该外部ignored清理片。源/ref、宿主任务归档、branch删除、stash与独立ZCode是不同对象，不批量混删。本报告写完后也必须先由清理owner核最新回执再更新状态。

Issue/Finding只使用本地材料：冻结来源中的 `docs/supreme/audits/eko-issue-finding-reconciliation-2026-10-25.md` 及其后继段、根内 `docs/supreme/plans/2026-10-24-workspace-maintenance/reconciliation.md`。前者当前未在主根同路径存在，后续应将有效内容与来源导航接入根内，不因外部view已备份就假报本根正式审计已承接。不得编辑旧正文来消灭历史链接或错误。

- #188 ZIP修复和本地Finding resolved有具名历史依据，外部Issue最近观测open仍只是旧API观察；不能称今日closed。
- #189七旧树不可得保留unknown/open；用户现码审查授权不等于risk_accepted，也不豁免新快照门禁。
- A2A #32–35退役与旧缺陷修复分开；最终semantic1和排除外部工具修复的决定保留，不能由代码删除、root编译或本报告关闭。
- R1、R2本地child、T8默认尾错、PR18与旧SDK Plan06主体已有历史交付；Host/preparation/资源保留等未证后继按原范围保持，既有718旧补丁审查不重做。
- 所有外部Issue close/comment、main合并/推送、注册表发布与网站部署继续按独立授权和具体证据处理；本轮无任何此类动作。

## 6. 本次采用的材料身份与恢复导航

下列SHA用于定位本次对账输入，不是为后续改动背书；根内新增文件和GUI进度在并行变化，最终采用时协调者需读回。

| 材料 | 本次 SHA-256 / 记录 |
| --- | --- |
| 主根Design | `d8e2fdf13bad582f8044f027c9fa5ad6e6dcf6c128de1346fbadf4406fdc759f` |
| 主根Delivery Map（已含新Steer/Goal ships） | `8439dd65e4349c30fd37785fa46a158c7014d0df4fb655fcb80d30527a8b45c7` |
| 主根Plan12 | `580a01370d4ca688ce9d8a08380e9df9b3269b6fb2f65499e71d85f756c7a014`；`plan:a7405e25-3e08-436b-bc70-a64d0dcb0db3` |
| 主根Plan13 | `4a4fbd735cc02badfeb584023b37c70972012956e43d384e4316521ea91bfca5`；`plan:b814efe9-63c9-4946-9cb5-ee75e7d8af68` |
| 工具事实清单 | `919adb65f193d5741a1fcbae0c398eb0a88950d54e81ebe41ec0fde011a4ad52` |
| 主根MASTER当前读本 | `329bec5ced4619dfc44f302d6188802f8521df61b5c03a102bf7280d639b251b`；本轮未改 |
| 旧Issue/Finding审计冻结来源 | SHA `96cbf37d46d2ef0404744506d108f23dff32b6bc9013cfb4bee7d8d67559c0a7`；原绝对位置 `/Users/ls/.codex/worktrees/eko-gui-product-convergence/lp-agent/docs/supreme/audits/eko-issue-finding-reconciliation-2026-10-25.md`，现仅恢复来源，非当前开发根 |
| 既有资料维护对账 | `9bd0e4753d4946fcefcc6c4d0a67e08e610d29e1a0cfe8d9e79b7feec5c36706`；根内2026-10-24-workspace-maintenance/reconciliation.md |

当前根原件入口：

- `.echo-semantic/local/workspace-convergence-20261026/`：迁移来源、交叠整合、源备份及ignored恢复索引。
- `echo-agent-cli/.echo-semantic/local/root-convergence-rust/`：旧根queued replacement真实101，保原件。
- `echo-agent-cli/.echo-semantic/local/root-consumer-reassessment/epoch-08/`：当前风险输入、唯一queue/P1执行和最终准确clean回执；本次截点尚未产生最终结果。
- `echo-agent-cli/.echo-semantic/local/steer-goal-controls-root/`：根GUI独占4路径的真实Node/TS/lint/format/docs parity记录；不是整个GUI的签收。
- `workspace-inventory.md` 与 `.echo-semantic/local/workspace-maintenance-20261026/worktree-inventory.md`：清理线维护入口，实际删除与归档按其最新回执读回。

## 7. 接续顺序

1. **当前根验证线持续执行**：等待epoch08自然退出；原selected/pass/fail、错误、source/lock、targetreader与准确clean分别记账，失败沿真实根因修复后同入口复验，不为本报告重跑已过同源检查。
2. **轻量线继续并行**：GUI完成已有controls独审与同根验收，清理线完成有界恢复/归档与fresh回收；本对账把有效要求原文、替换关系和Plan12增量接缝交协调者正式承接。共享代码和文档仍各有唯一writer，不抢占正在编译的Rust源码。
3. **工具治理选择一个具名小切片**：先当前根factory预算/曝光真实验证，后续只对证据显示的缺口实现；不回到全仓盲删工具或外部治理工具修复。
4. **产品真实验收与完整交付**：在具名稳定源上完成列表/HITL/首页、IM/cron/微信及SSH项目的原生/真实服务要求；再依接管事实处理console/A2A/LSP剩余退役门，按所属仓规则完成适用完整矩阵、精确main CI和文档/官网同步。发布与部署最后按具名制品及授权分开执行。

本次仅新增跨仓阶段对账，不改变产品行为/API/示例或官网内容；正式子仓文档、examples、官网镜像不适用于本文件的实现同步。未编译、测试、运行scanner或连接服务，没有Cargo产物，`cargo clean`不适用。未stage、commit、checkout、更新gitlink或其它refs；也没有消费/关闭主线程Goal。
