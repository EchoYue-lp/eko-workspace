# 工作区历史资料对账与维护交接

日期：2026-10-24。本文是跨仓阶段性资料索引，记录原件、交付依据和后续归位条件；各子仓正式源码、合同及ADR继续拥有产品事实。本文不修改旧Plan lifecycle、不选择委派深度，也不替代本轮GUI收敛设计或工具清单。

## 固定对照与本次范围

| 仓库 | 读取的origin/main固定提交 |
| --- | --- |
| eko-workspace | `9a96ff9675c340c9d6ef18d63dd2a2b16663962e` |
| echo-agent | `51726454985fd6f064b21a7e9c539e22d494c987` |
| echo-agent-cli | `c262cfed8e77abed38a27c1fff93547997ec9486` |
| echo-agent-sdk | `0930eef9f878cf436ed88384030b6bc03d6f6c53` |

使用上述Git对象读取源码与已提交交付材料，没有重新运行历史CI、Host或产品测试。下文“已交付”表示固定材料及相应实现支持的历史范围，不表示本次新运行。

首阶段维护提交`00e0afc48bccc1571a38020ab9a4042fc9e50fb1`从固定workspace提交建立，只修改原压缩验收记录并新增本文。该切点primary仍在原分支，14份未跟踪docs、已修改压缩验收、ZCode与三个gitlink漂移均未被归位或删除。后继维护只将这14份原字节纳入下述历史档案并保留来源导航；实际本机归位以独立操作回执为准。Exa的暂不推送/不合main边界不变。当前GUI主题、工具清单、业务代码、SDK/CLI正式文档由原owner继续维护。

## 已保存的PR18尾节

[原压缩验收记录](../2026-10-09T1545-eko-context-compression/verification.md)只追加primary用户已有的“顶层集成结果”尾节；此前main正文保持逐字节不变。原primary文件SHA-256为`176b1abef199e55d564b3171706a00af102cdb39826d58e8b54e904642e3ed55`，固定main旧正文为`c81576b0290b64c0e7c5a481f6479061fbab8661ee474065dd2bd62a4a718031`。

本次离线核对：`eadd2aa86f6674c67124b3218f0d1a01bf254be6`与候选`10d7093`的Git tree均为`132f51ffd3aaaaf7957cc121a2e70dc85fe591b8`；前者已在固定main历史中。尾节中的GitHub签名、PR和当时最终对账属于原交付记录，本次未重新查询或执行。原生窗口、真实模型、部署与全局release/soak仍不由此取得验收。

## 十四份未跟踪文档逐项索引

下列路径指primary原址；SHA-256对应完整原始字节，不是裁剪摘要。状态原样记录，本文没有把ready/abandoned改写为completed。

| primary原始绝对路径 | 原字节SHA-256 | 价值及已交付依据 | 当前归属与处置 |
| --- | --- | --- | --- |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/supreme/plans/2026-10-22T2047-parallel-remediation/contract-freeze.md` | `931dbc4166922d895d70103b22cfed76ad4fa60c6e935bab2029e6e5da2c1ebf` | 历史交付后继待对账。第13节的A1–A7由CLI fb98ffc后继修复；原授权失误历史保留，当前精确复评以所属仓正式材料为准。 | issue-finding-reconciliation；整组来源确认后可归档primary副本 |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/supreme/plans/2026-10-22T2047-parallel-remediation/r1-boot-interruption.md` | `ee883b197b3493e95c8bb83c85ffe9adb40eedb0f605c84dd4cab2644cbf927a` | 已修复问题的修复前证据。CLI ADR0049及fb98ffc内生产epoch/恢复实现已接管；原报告未执行测试的事实不改写。 | issue-finding-reconciliation；与contract导航成组处理 |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/supreme/plans/2026-10-22T2047-parallel-remediation/r2-sdk-child-handles.md` | `5d948978d0017064a213c23f04506f4074fd2e5904dc8992ca088edbef170b41` | 本地关闭修复已交付，原边界保留。SDK 4acd5b8/ADR0041修复本地子scope；未扩Host/wire，不宣称原报告全部子问题resolved。 | issue-finding-reconciliation；与contract导航成组处理 |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/supreme/specs/2026-09-15-echo-agent-sdk-independent-repository/design.md` | `3c69e37c3f49bdb68950bd93eddaf58d5a4279467f936bca6c3d3a6bc3049812` | 已实现的跨仓迁移设计。独立SDK、单向Host依赖、纯Protocol及accepted/telemetry边界已有正式实现和ADR。 | issue-finding-reconciliation / legacy-resource-cleanup；先保全SDK来源定位 |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/supreme/specs/2026-09-15-echo-agent-sdk-independent-repository/plans/delivery-map.md` | `a543a13d170772c7c2be183f17246b3cc0f0256d99ccc73f4d3389d7d565f420` | 旧迁移路线图待状态收口。源码导入、连续历史、框架extraction、clean pin、消费者投影及第四submodule均有后继交付。 | issue-finding-reconciliation / legacy-resource-cleanup；先保全SDK来源定位 |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/supreme/specs/2026-09-15-echo-agent-sdk-independent-repository/plans/plan_01_独立交付完整SDK产品.md` | `71d12c04ea66403439c0006879783a884210ef3ff22286d13ecbe7de04f0dff4` | abandoned历史计划。原整体方案被后继切片替代；不改为completed，不重跑原方案。 | issue-finding-reconciliation / legacy-resource-cleanup；先保全SDK来源定位 |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/supreme/specs/2026-09-15-echo-agent-sdk-independent-repository/plans/plan_02_建立Framework迁移基线.md` | `d1129336c36dce19ec9b97e96afcf7355872e49488bfce60f1444f318d7303fc` | abandoned历史计划。旧迁移基线路线已替代；保留原状态与原验收文字。 | issue-finding-reconciliation / legacy-resource-cleanup；先保全SDK来源定位 |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/supreme/specs/2026-09-15-echo-agent-sdk-independent-repository/plans/plan_03_先迁移SDK源码.md` | `bc9a31bedccf7af210fd19fcf7c47445cf622f7a5e290471ac475bef64b0e8bc` | completed历史计划。完整源码导入与迁移来源保存在SDK MIGRATION-SOURCE；无需重复导入。 | issue-finding-reconciliation / legacy-resource-cleanup；先保全SDK来源定位 |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/supreme/specs/2026-09-15-echo-agent-sdk-independent-repository/plans/plan_04_Framework移除SDK资产.md` | `57eb7de84b7d0def3f465d7517b72dfe51bc526094fbefc0fbec1ab95ab5d922` | completed历史计划。框架27c7701e执行SDK extraction，当前framework workspace无SDK成员。 | issue-finding-reconciliation / legacy-resource-cleanup；先保全SDK来源定位 |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/supreme/specs/2026-09-15-echo-agent-sdk-independent-repository/plans/plan_05_SDK源码连续性.md` | `f95f2378f1e129cca54ff04b55fa02e474f8ba3eae641f62a7005d087709adc3` | completed历史计划。SDK b80cf06保留连续历史，且为固定SDK0930eef9的祖先。 | issue-finding-reconciliation / legacy-resource-cleanup；先保全SDK来源定位 |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/supreme/specs/2026-09-15-echo-agent-sdk-independent-repository/plans/plan_06_SDK框架Pin与合同收敛.md` | `426eb673ed9274888655c7b5569f7292cb93c792b5f5df1085ece12aaf8991a9` | ready旧计划，主体已有后继交付。纯Protocol、Host精确f7bac62e、accepted阻断/完整Rust遥测分离已存在；旧175487目标不能当当前pin。 | issue-finding-reconciliation / legacy-resource-cleanup；先保全SDK来源定位 |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/supreme/specs/eko-unified-task-subagent-surface/design.md` | `441628718b9c36567801cab9b84352476d2bfff1334d37edfa4d87a950f5ae51` | 部分目标已实施，边界仍需对账。单图和shared admission已实现；direct工具曝光/归属和委派条款适用路径未整体收口。 | builtin-tool-convergence / domain-plugin-runtime；保留旧状态 |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/supreme/specs/eko-unified-task-subagent-surface/plans/delivery-map.md` | `dad9ea11dbb09e2666b300b6e092c9886aa547e3df539dde602b8fd9ef162cc1` | 后继结果部分已有代码。kernel已实现；EKO容量5、去重复subagent semaphore已接线；不把整图判为全未做或全完成。 | builtin-tool-convergence / domain-plugin-runtime；保留旧状态 |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/docs/supreme/specs/eko-unified-task-subagent-surface/plans/plan_01_下沉共享Subagent准入内核.md` | `1844f55488c1913147c80d9b48d3553124cebc2fd047b8f8fe9a077e4167a92a` | completed历史计划。框架ExecutionAdmission及RuntimeTaskService/SubagentExecutor接入真实存在；不重复造kernel。 | builtin-tool-convergence / domain-plugin-runtime；保留旧状态 |

## 并行整改记录的后继

固定main中的[四仓旧补丁交付记录](../2026-10-23-legacy-patch-closure/delivery.md)已覆盖14组旧补丁及后续修复。primary contract第13节仍比固定main同路径多出复审、授权失误及R1–R3进行中记录；其完整原字节继续保存在原位与下述归档，不能删除历史错误或把第12节早期“已完成”拿来覆盖第13节。

- A1：bootstrap/pool失败的typed cleanup receipt已有进程保管和重新接管；A2：restore失败保留原withdrawal与claims；A3：retry在原ledger中逐项await，确认结算后才移除。依据CLI `fb98ffc`中的`plugin_runtime/receipt_ledger.rs`、`runtime.rs:960-994,2668-2698`和对应回归。
- A4/A5：MCP后台连接按精确Agent/名称/operation保存，超时观察不再无界等待同一Agent锁。固定CLI源码`mcp_config_runtime.rs:671-684`及具名交错回归已存在。
- A6/A7：迟到summary/review与最终执行receipt跟随durable结算，已由同一CLI交付修复；原取消/Superseded/abandonment范围见既有交付记录，不扩大为任意终态组合已验证。
- 原“待签认d2e7/9c834”的授权缺口保留为当时事实。当前CLI正式`dependency-risk-decision.md`已有2026-10-23有界复评及后继锁修补记录；最终锁`cca90ea328dbdbb5ebf819b5afb8845714c202e5f5fd1b9c032811c35f0e1307`、consumer `47bb8f6a1d67d4ab808903c04750d6eaf8d3ba3944d64e60eccc311f913cc22b`只绑定原六项风险。它不是零warning或风险修复，不自动授权新任务consumer刷新。

R1的进程丢失问题已有[CLI ADR0049](https://github.com/EchoYue-lp/echo-agent-cli/blob/c262cfed8e77abed38a27c1fff93547997ec9486/docs/en/adr/0049-taskruntime-process-owner-recovery.md)：唯一intent写边界盖入process epoch，持root OS排他租约后原子退休旧进程admission债务并分类Running/Paused副作用；当前进程、畸形epoch和unsafe副作用继续受原约束，不伪造物理完成或重放工具。原R1报告自身“未执行测试”仍是原事实；后继回归及精确main资格由既有交付记录分别提供。

R2的本地scope问题已有SDK `4acd5b81cf3f64b17a7aed546d4aaa2e145e52cb`及[ADR0041](https://github.com/EchoYue-lp/echo-agent-sdk/blob/0930eef9f878cf436ed88384030b6bc03d6f6c53/docs/adr/0041-session-close-local-child-scope.md)：成功父关闭后收束现有、首次和late子订阅，保留缓冲事实，失败关闭可重试，取消观察者不取消原清理。它未新增Host终态通知、run/close或关闭后replay合同，EOF不代表业务成功；不能把原R2全部子问题批量resolved。

旧T8要分开记账：默认`retaining_source_stream`吞首个尾错已由框架`a39a4cde`及当前`echo-core/src/agent/mod.rs:1520-1583`覆盖；通用MCP preparation的取消合同没有因EKO保持原后台任务运行而自动全部闭合。当前scope仍有building等待，manager仍保存preparing ticket。是否存在其他真实复用路径的取消反例、是否需框架改动，先在既有工具生命周期清单定向核对，不直接增加“补Drop”实现承诺。R2中未复现的Host等待/持久化失败、资源保留等边界同样按具体证据保存，不并入另一个已接受的历史managed-exit UNKNOWN风险身份。

## SDK拆仓与旧ready Plan06

[SDK ADR0001](https://github.com/EchoYue-lp/echo-agent-sdk/blob/0930eef9f878cf436ed88384030b6bc03d6f6c53/docs/adr/0001-sdk-repository-boundary.md)、正式README及合同产物已拥有迁移后的长期事实：Protocol无framework依赖，Host单向消费精确framework，accepted external contract是阻断范围，完整Rust inventory是非阻断遥测。SDK `b80cf06`保留过滤后连续历史，framework `27c7701e`执行extraction，SDK `863cd5b`交付clean pin及合同分离；workspace `c98a810`纳入第四submodule，`853eee9`继续集成accepted SDK。

旧Plan06虽为ready，其主体不是尚待从零开发。原Plan硬写`1754877996778afac4e4db77ce37c330496760ea`，正式历史extraction pin为`27c7701e1eb116db1076da7f84bb68898544a44c`，当前Host manifest精确pin `f7bac62eee04a963f92a3e4308b6ea2336e91cc0`。当前SDK接受及后继完整门禁见固定交付记录；accepted 5621 identities/454 routes与4项非阻断遥测保留各自范围，不重跑旧pin、不自动刷117产物、不由本文更改Plan lifecycle。未来A2A/LSP合同退役属于新产品收敛结果。

SDK固定main的`MIGRATION-SOURCE.md:198-199`仍文字定位到superproject的`docs/supreme/specs/2026-09-15-echo-agent-sdk-independent-repository/`。首阶段快照中，该topic的8份原文件确实存在于primary，尚未在固定workspace main被跟踪。后继维护将原字节保存为`.original`历史数据，并在这个原topic目录提供[来源索引](../../specs/2026-09-15-echo-agent-sdk-independent-repository/README.md)，让SDK原有目录定位仍可用；不是把原文件写成从未存在或没有价值。本维护不改SDK原文件、不把8份旧计划复制为新的产品权威。

## Unified Task/Subagent后继范围

共享kernel已有真实实现：framework `ExecutionAdmission`被RuntimeTaskService与SubagentExecutor共同消费；CLI `infra/factory.rs:978-983`、`tasks/task_runtime/register.rs:45-52`和`executor/review.rs:11-58`注入同一个进程admission；`executor/limits.rs:50-74,124-131`的产品默认容量为5，旧ProcessExecutionGovernor只剩write/shell/LLM。Task graph、revision及终态仍由既有framework服务和EKO journal拥有。涉及范围内命名只观察到Tokio固定`worker_threads`参数，不据此新增全仓重命名任务。

旧设计“所有direct派发附着TaskRun”和当前`register.rs:109-110`、factory提示明确的run-less ephemeral `agent_tool`还需按路径对账。委派层数也不能简单由framework默认值推出：

- Accepted ADR0034对8个EKO内置角色声明`can_delegate:true`；factory统一装配这些角色，按该字段注册/同步子Agent的dispatch工具。
- Proposed ADR0038及Accepted ADR0039写明Child Agent不接收agent_tool/task_execute、一层限制；未查到二者对同一direct工具曝光路径的显式supersedes或GUI/headless例外。
- TaskRuntime显式传入delegation policy，再受PlanTask allowed_tools过滤；其config与无显式policy的direct fallback不同。源码不足以支持“每条EKO路径都是三层”的结论，本次未运行实际嵌套反例。后继ADR0040处理事件投影；ADR0042的Side Conversation一层约束又明确保留普通Task/Subagent能力，不能替代普通派发政策。

因此旧map只按部分已实施记账。剩余同路径合同/曝光核对进入现有`builtin-tool-convergence`及`domain-plugin-runtime`，不新增产品选择或Outcome、不自行选择深度。GUI视觉/任务弹窗消费同一后端权威，不借UI改版复制另一套调度器。framework ADR0029头部仍Proposed但正文已有Implemented段，是材料状态对账，不否定现存kernel。

## 归档候选与导航条件

1. parallel-remediation三份：本报告保存第13节后继说明与原字节索引后，可由协调者在精确源提交/恢复依据完成时成组归档primary副本。r1/r2仍被contract相对引用，必须保持同组或在目标checkout已有对应正式文件；本次不提前移除。
2. SDK拆仓八份：源码迁移与旧Plan原状态已纳入历史原件档案，原目录来源索引保留MIGRATION定位及Plan06后继映射；可在原件校验与归位候选独审通过后回收primary未跟踪副本，不用info/exclude隐藏。
3. unified三份：kernel已交付的旧Plan可历史归档；设计/map中尚需核对的执行边界须已被现有工具/领域插件治理接管，原始SHA及恢复位置继续可查。不能因新GUI设计存在就废弃Task/SDK基础能力。
4. compression修改：有效尾节已进入本维护快照；是否把primary切回、恢复或与main归位由协调者后续执行，本次没有修改原件。

此前已归档CodeIntelligence三份与完成的cli-sdk-docs-governance三份。本维护候选重新检查root HEAD/固定origin-main的README、AGENTS、MASTER，15保留文档及当前GUI topic，共26源码快照、58条本地inline/reference/HTML链接，未发现活链接依赖这6份原件；输入路径、固定revision/内容hash及逐链接结果保存在本维护树`.echo-semantic/local/workspace-maintenance-reconciliation/navigation-audit.json`。root固定main所有tracked docs的精确topic/Plan名称检索亦无匹配。该结论只覆盖这6项导航，不等于全仓链接或外站验收；未改历史正文“消灭”链接。r1/r2曾归档后已按原字节/权限/mtime恢复以保导航，这一过程保存在原维护receipt历史阶段文件。

## 原件归档与恢复索引

本机备份根：`/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktree-backups/goal-maintenance-2026-10-24/primary-files/`。

- `primary-files-original.tar.gz`：SHA-256 `e4893de22bee9b437ae960ad51f4fb30af397288cad650033d5818c504ed149c`，1697338 bytes；保存302个原始普通文件，包括本文14份docs及compression修改原件。
- `initial-inventory.json`：原path/status、SHA-256、size、mode、mtimeNs；`maintenance-receipt.json`：逐项归档/原位保留理由，固定SHA-256 `ec02c9e0315f88e54c79112b477e970b85d734a3ff1ec9599c5b4db9b4509bb6`。
- 首次文件维护结束时的处置数为302=287移除+15普通文件原位保留；另4个独立仓目录状态行保持。当时root从301未跟踪降到15：14份docs加ZCode；4项未暂存为compression与3个gitlink漂移。289/13是恢复r1/r2前的更早阶段。后继归位另记自己的真实处置数，不回写这份历史收据冒充其当时已完成。
- 302原件完整读回与完整隔离恢复演练已核hash/mode/mtimeNs；当前287移除项又单独完整恢复演练。两份报告恢复见`navigation-restoration.json`，旧阶段receipt另存，不改归档原字节。
- `restore.py --destination <明确恢复目录> --selection all`可恢复全部302原件；`--selection removed`只恢复当前287移除项，**不包含原位保留的14份docs**。恢复器遇到不同内容的既有文件拒绝覆盖；后续原件归档后须使用本文path索引或在隔离目录选all恢复，不假定removed清单自动更新。

上述路径是本机原件定位，未随Git提交分发；原件不可获得时不能把本报告当作恢复文件本身。另一个legacy_cleanup的`legacy-integration-history`由其owner独占，本任务不写入或更改其回执。

## 首阶段维护提交的验收边界（00e0afc）

首阶段直接检查覆盖：两文件允许范围、main旧正文及原用户尾节逐字节一致、15源文件未改变、14文档原始hash逐项存在于归档索引、PR18 tree一致、相对文档导航和固定来源提交可解析。该阶段已通过独立审查及task范围语义验证后本地提交；无push、merge、PR、Issue关闭、包发布或部署。

没有Cargo编译/测试或自有target产物；cargo clean不适用。没有业务实现、examples或官网行为变化；正式产品文档和官网仍由所属owner按后续结果同步，不为本历史资料报告复制产品说明。文档验收不冒充原生GUI、SSH、真实IM、新Host或全项目验证。


## 后继归位候选与历史档案

[历史原件档案](archive/README.md)保存上表14份原始文档，逐文件字节SHA与原primary一致；`.original`保留原内容并避免旧ready Plan成为新执行入口。SDK原topic的[目录索引](../../specs/2026-09-15-echo-agent-sdk-independent-repository/README.md)把既有MIGRATION定位引向8份原件。parallel三份在档案内成组保存；目标checkout还保留固定main同路径的contract/r1/r2，活跃相对导航仍可解析。unified未闭合边界继续交给当前工具/领域插件治理，没有改旧Plan状态或委派策略。

前置恢复已读取`legacy-integration-history/final-audit.json`、`recovery-index.json`与`retirement-receipt.json`：W4及其四个私有submodule、两bare证据库与私有cold linked checkout已由原owner回收；9库strict import/full fsck均0、146归档payload回收后复核通过。primary CLI原`3dad583c9aad7d9df8e7a5db00631bd2db6557bb`、`c5bf87033872f15b37e105e8e22a12c9320e177c`、`b85ec037c6ec33e46bf998ba59407aab1566ef5b`均完整保全；本任务只读这些回执，未写其目录。

下面是须先独审的可逆本机操作候选，不是已执行记录：再次备份并逐项验证primary这14原件、压缩修改、原HEAD/index及stash；检查精确路径无占用或并发改变；确认本地维护提交及恢复包完整，提交父列表严格只有已审首阶段提交；回收本任务已完成managed维护worktree以释放分支；把15份原件同盘移入恢复位置，以独占创建写回原HEAD的compression基线（原路径若有新文件则拒绝覆盖），然后用不递归submodule的普通Git switch进入同一维护分支。任一步失败即保留真实回执和恢复位置，不做hard reset、不清stash、不push/merge main，也不覆盖其它任务文件。

四个子仓refs/config/index不在此操作范围。切入维护树的目标gitlink仍为固定已接受主线（framework51726454、CLIc262cfed、SDK0930eef9、websitea926ef7c）；原primary子仓HEAD若不变，仍会显式显示其checkout差异。ZCode继续原位保留、未跟踪可见；不通过ignore隐藏或提交私有子仓指针来制造零差异。子仓对齐由协调者另行独占处理，不能把该候选声称为已完成四仓构建或全工作区干净。
