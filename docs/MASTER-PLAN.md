# EKO 总纲（MASTER-PLAN）

> **跨仓库当前事实源**。本文只记录阶段状态、前向路线、分层边界、验收门和可核验证据；不再保存逐轮实施日志。
> EKO 是本机个人助理：`echo-agent` 是独立可复用框架，`echo-agent-sdk` 是独立 SDK 产品，`echo-agent-cli` 是应用层，`echo-website` 是官网。
> 最后更新：2026-10-20（SDK 全能力治理阶段快照；尚未完成联合验收，历史阶段保留原验收范围）。

## 2026-10-16 SDK 全能力治理（进行中）

治理范围仍以明确接受的跨语言合同为准，完整 Rust public inventory 仅作遥测。
用户已批准 Workflow `protected_claim_v1` 与
`workflow_checkpoint_protect_claim(checkpoint_id, attempt_id)` 反向操作：原 Store 在节点副作用前
保护同一 claim，未知执行不能因租约到期重跑，也不能由 CRUD 解除；缺能力时执行前明确拒绝。
取消结算采用已批准的 B：观察等待有界，原物理执行、claim 和 permit 保留到原执行真实结算，
不得用超时、丢弃 handle 或合成终态替代。正式合同、实现与限制分别由 framework/SDK 维护。

本 goal 当前验收与开发状态如下；后文各切点保留原历史范围，不替代本表的待办。

| 仓库 | 本 goal 已验收的 main 快照 | 最新开发 checkpoint 与未完成事项 |
| --- | --- | --- |
| framework | `3560812c`：PR193 squash main，精确CI七项全绿，累计strict及远端冷恢复通过 | 本goal后继framework治理目标已接受为完整revision `3560812caede84998a1f6f26d7bf3fa81ea0486c`，tree与已审 `8138d81e` 精同。预算/模型发布、配置化client构造、压缩保留ordinal/CAS、canonical/effective根System及Intent回归均保有限独立复审；六完整门禁与17独立feature全0，完整测试3653通过、0失败、3个原有外部环境用例未执行、不计覆盖。累计38个必要文件经独审，无日志或临时副本；1764工程输入与实际门禁快照逐字一致。仅远端main及必要证据refs冷恢复后，38当前文件与1039历史源码对象一致，同一累计basef3fa strict退出0；原缺对象失败保留。原生产治理目标6563及Graph补证f3fa保其历史资格，广域Behavior不升级。SDK继续独立精pin356；CLI全量暴露post-build裸client注入清空配置后，在既有配置化构造边界补Builder原client/config成对入口，该局部修复已以`ffe4c19acf6d33cd419f2237de3209b4e3015e80`正常提交并推送，精11既有文件经源码、运行收据和必要性独审；六项focused均0，7构造、11 Builder及7发布用例分组计数，不去重相加，原demo26仅编译检查。真实同步最新origin/main356后，原完整fmt两项及两档workspace Clippy均0，full workspace/all-target/all-feature测试正在运行，no-default及17独立feature尚未执行；无编译profile弱化。不得借这些局部结果授本任务main资格，也不改变已接受356的历史资格。CLI新消费者、最终官网及联合gitlink仍待，不代替全goal验收。 |
| SDK | `09f670fb`：PR6 metadata squash main，新精确CI进行中 | `86fe347f` 媒体闭合及`736131b5`历史反例补证已正常推送；最终682路径（569A/113M）全文件必要性独审通过，PR5合并前相对9ef仅14份既有材料，668路径及已验证工程快照不变，无日志、二进制或缓存入库。PR5 squash main完整revision `1e1c4f03f670561d6ca804a56d35967eeea191ce`，tree与已审7361精同，GitHub签名有效。46个有限Behavior verified，extension-bridge及family-bridge、资源流与语言客户端整体共4项仍needs_review；97 Audit保历史revision/freshness和原challenges，只补具名结论，原119 Evidence及maps保留。当前74resolved/1open：04eb两处media alias修复经本次完整verification及原ToolsBridge Audit精准rereview正式闭合；managed旧Linux被动退出原因未知，当前回归通过不能替代历史根因修复。accepted5621 identities/454 routes不扩，F356精pin、合同及deferred/Rust-Host边界一致。冻结9ef工程源码的原14完整Rust命令均自然0并获独审，workspace主集合521通过/零失败/零忽略，其他配置不重复相加；1222受控输入零漂移，最终full Host为a17。唯一原官方完整语言入口自然0：TS358通过、Python442通过加18子测试另计、Java379通过，均零失败/跳过；三个quickstart及#120各语12场景已独立确认，Host SHA/mtime保持。最终150roots/2418文件归档全SHA零差异；后处理copy2曾使free低于5GiB，已等字节恢复稀疏存储，原测试入口自身无资源bound，两阶段分别保留真实收据。工程资格不借旧208a/0bbc成绩。代码main1e1的原始push run39394908918实际九项全成功并获独审；Windows项仅编译，Linux语言job直接观察TS358、PY442加18 subtests，Java用Temurin17完整quiet入口成功、无上传逐case XML，不借本地379计数。当前Host bridge29通过、managed具名case通过，仅作当前回归、不归因历史超时；各feature配置分别保真实计数。独立空仓仅从GitHub main及35个必要annotated refs恢复53源码epoch，11799对象无缺失，固定canonical c0bc strict/base30d/change-evidence自然0并获独审。依赖job49个duplicate版本组均在原接受基线，零其它类别，未扩大例外。main派生文档分支已以fe28a264正常提交精13材料（5原Audit、7份当前说明及baseline仅摘要），两项独立复审及canonical/artifact均0；其余1209跟踪路径包括工程代码、测试、锁、合同、CI、示例逐字不变。PR6正常squash成当前完整main `09f670fbbde98bc44a7c710c6419a6ea0d7d469c`，唯一parent1e1、tree与已审fe28精同且签名有效。新原始push run39405806697及该main独立GitHub-only冷恢复正在验收，不能借1e1成绩冒当前main全绿；无本机工程矩阵重跑。原生Hook/Stop及managed历史未知、4项整体hold仍待，Plan READY，不关闭外部Issue或宣称全goal完成。 |
| CLI | `f57b90a2` 历史 main 验收 | `0d1bbd39` 原最终文稿保其资格；新CLI/F356兄弟隔离worktree的两处context_only literal及普通导入修复已独审，framework保canonical根System唯一权威，CLI删除裸base手工注入并按非System实际消息计数。原6用例before真实3pass/3fail，after同6真实全0且测试体未弱化；双语ADR0045同步。原glib0.18.5/proc-macro-error1.0.4/bincode1.3.3精版本冻结，有限消费指纹重绑及原6风险单元全0。两次官方在线扫描均scanner1/riskgate1、零漏洞/六项warning：新增GUI通知链zbus_names4.3.4、zvariant5.13.1、zvariant_derive5.14.0未被接受；最新兼容上游仍依赖zbus5，不能声称普通升级已修复或把原三项接受扩成六项。原始失败与上游来源保留，此七路径切片尚未提交。新消费者原八命令的fmt两项及两档Clippy均自然0；workspace测试自然101，首个core集合1630通过、17失败、9个原有忽略，后续target及06–08未执行。17项同源：factory在配置化build后调用裸set_llm_client清空配置；真实失败二进制及完整raw已外置保真，保留原断言。修复先补framework既有Builder成对client/config入口，再薄适配三条factory，不新增应用配置权威、不回退裸client合同。完整新框架消费者/GUI门禁、插件两处已有差异人工接管、原生Stop、精确main及官网最终同步仍待。primary及旧冻结候选保持不动；原CLI聊天网络失败后由协调线程委派子智能体接续。 |
| website | `ccdbfb4b` 精确main；`335c26cc`后继任务分支checkpoint | framework356-only同步已正常推送完整revision `335c26cc7e4467c1e953eeb3fe40aa9df8d68891`，精10文件为8篇正式镜像、来源manifest和官方生成llms-full，独立必要性复审通过；290映射及来源逐字一致，五项官方focused同步/发现检查全0。CLI仍精绑定原main f57，118个来源文件和8篇EKO投影未改；六页双语EKO文稿草稿仍待最终CLI来源资格。此checkpoint未合官网main、未运行完整site/build/browser门禁，无部署；新framework/CLI精确main接受后再做最终联合来源同步、全量官网门禁及gitlink。 |

框架 [PR #187](https://github.com/EchoYue-lp/echo-agent/pull/187) 已 squash 合入
`875cc22f2e99c7ccd83c07e02c03fe221c5e3f6c`，签名有效、tree 与已审候选一致。
精确 main CI `38849170013` 的六项运行时/平台 job 成功，依赖策略 job 因
`zip 2.4.2`、`zip 8.6.0` 已撤回而失败，因此该提交尚未通过交付验收，SDK 不升级到此快照。
后继修复 [PR #190](https://github.com/EchoYue-lp/echo-agent/pull/190) 已合入
`9886502c83d6b1f6fa9212eb221db0e5874deaee`：保留三个媒体父包的 API/feature，适配非撤回的
zip 9，并补齐 path/Git 来源的 registry-origin 审计；正式决策见框架 ADR 0089。
六项完整本地门禁及 17 项独立 feature 检查全通过，主测试计数 3,622 成功、0 失败、
3 项既有 opt-in 忽略；不重复计入两个子进程结果。独立复审、必要路径检查通过，签名有效、
tree 与已审候选一致；精确 main CI `38862267005` 七项全部成功，远端冷克隆的严格语义
检查通过。按用户“旧提交无法找到则按最新代码审计、处理”的明确裁决，相关能力重新绑定
当前源码与已执行用例；不宣称恢复七个缺失历史 tree，也不把旧红绿记录作为新执行。
本轮框架验证后官方 `cargo clean` 移除 7.2 GiB 缓存，源码及证据保留。

SDK 后继 B 场景在上述固定 `9886502` 来源上又确认一个 ACP 集成缺口：原 driver 返回
`ExecutionNotSettled` 并保留 lease 后，其错误直接进入官方 `connection.spawn(task)`，按
ACP actor 合同关闭整连接。原 Host 自行退出2，不是 `run_wait` 的正常有界等待，也不能
把 foreign physical exit 当作原 Run 收据。唯一框架子智能体已接手原干净隔离工作区的
`fix/Echoyue/acp-run-task-error-isolation`，先以官方连接复现，标准 adapter 与 SDK 通用入口同审；
RunRegistry、lease、receipt 权威保留，SDK 不吞错或复制状态。该反例首先按988复现，
必要修复的来源资格与后继消费者升级分别验收，旧 CI 通过不覆盖这条新反例。
后继开发 checkpoint `30b9bf0539efbec838f6229e7736e5889b483639` 已提交推送，精13个
文件及 tree 与审定候选一致。原官方 ACP 连接两项真实 red 在最小修复后同例通过，
29项 ACP 集成、43项 ACP 单元和 focused Clippy 全通过，独立实现及必要性复审 PASS。
新 `wait_result` 只暴露原 Run 的非终态观察，真实 receipt 优先，原 lease 不释放；未新增
三语言合同。原 canonical 的精 Git 候选严格/真实988基准变更检查通过；当前0.5运行态
校验的历史引用、闭包与范围限制单列，不称原生 Stop 通过。[PR #191](https://github.com/EchoYue-lp/echo-agent/pull/191)
的最终文稿 checkpoint `b7319a81e8c24b1b595e0daee74ea52828512547` 已提交推送；累计仍仅
13个必要文件，无原始产物入库。后继同源码的六项完整门禁实际全0、零告警及输入漂移；主87个 harness 共
3627成功、0失败、3项既有 opt-in 忽略，两个各1成功的嵌套 echo-state 子进程单列，不将
原始3629合计当主计数。17项独立 feature 矩阵也已全部退出0；文档合同15项通过，最终
独立实现及必要性复审均 PASS。PR 已正常 squash 合入
`6563d26570cc4a8d93c86b7e37443b0559476807`，签名有效、唯一父为988，tree 与最终已审
候选完全一致；精确 main 的 canonical 严格快照及真实988基准变更检查实际通过。
精确 main CI `38958501642` 七项全部成功，协调者独立核对远端身份及各 job，正式接受
`6563d26570cc4a8d93c86b7e37443b0559476807` 为后继治理目标。SDK 与 CLI 先完成当前
988切点的有限里程碑，再在各自冻结边界升级并重验；不将新框架通过冒充消费者已验证。
仅官方清理本任务独占 target 后回收7.97 GiB，源码及证据校验不变，不动 SDK/CLI 缓存。

SDK 下述历史 checkpoint 固定已通过 CI 的988来源；后继6563切换及新 Host 资格见本节后段，
各切点结果不互相替代。
[PR #5](https://github.com/EchoYue-lp/echo-agent-sdk/pull/5) 的开发 checkpoint
`637a308e0d1c7f13339d279eb1f91aedfb403685` 已推送，修复 canonical generator 对 workspace
membership 的选择，独立复审和对应 9 项测试通过。后继已推送 checkpoint
`4ca17639265af41b5a39586331346f165d1a610f` 保存当前锁的依赖复验及必要元数据修正；
两项 audit 在既有配置下退出0，deny 保留49组原已接受 duplicate、没有增量，不把它称为
零告警。28 profiles 的唯一生成与合同检查已经完成，仍不代替后继 runtime 的验收。
后继 `55bf069cf75832416f5d61ba247cd6b7df5cd25c` 已提交并推送，仅将22份历史对象的
23行版本字段绑定到可恢复的 `019e700279677a2eb77e265b5038d9d581bc22da`；对应完整源码
摘要 `2bbd6457d7b6ea031b8999d1466d85f5a5c1d2d994218d45f008ed6d5f1fa276` 和947组
来源引用已用原校验器核对，独立复审 PASS。正文、状态和原失败范围未重写；不代表其余
新材料、baseline、完整 runtime 或 main 验收通过。

Issue #120 的具名场景在不可变 Host `45fc7b267941864f4528f55d8d7373d048f5afc824b2fb389c7c8c8fbc680abc`
切点取得 TypeScript 12、Python 12、Java 9+3 项真实通过，零失败、零跳过，独立复审 PASS。
182项生产/合同输入和语言来源已核对，包含 snapshot ACK 后严格更大水位的 replay/live
continuation、迟到 ACK、terminal EOF 与 journal 换代；Java 两次运行互不重叠。
它不证明后继 callback 改动后的最终整套、SDK main 或外部 Issue 已关闭。

callback observer unwind 提前释放，以及 invalid ACK 被投影为普通终止错误，均已取得真实
before；后继同源7项 focused 通过、局部独立复审 PASS，保留原 guard/lease/配额与 foreign
身份边界，宽 Finding 继续 open。保存的 dev opt1 配置保留 assertions、overflow checks 和
unwind，删除旧链接告警屏蔽；无 opt-level/assertions/overflow/panic 环境覆盖的 Host
构建实际退出0、零告警，产物
`8bf4a1d6deb647e608153c807ca07785dd31f7d3223a88cc3daf9b835b8bff5c` 的 EH 为15,489,244字节，
低于16 MiB，保存配置下的 unwind focused 也通过。A/B 累计 Rust/protocol/pin 必须按真实
依赖边界验证和提交，不能仅 stage 局部代码而借用完整 dirty 树的通过结果。

CLI [PR #12](https://github.com/EchoYue-lp/echo-agent-cli/pull/12) 的开发 checkpoint
`c874a79aa25f8bef16ba4df2e809ceb447bb1604` 已推送，compose 早绑定、Running/Paused
恢复和既有 run 身份/错配拒绝三项定向验证各通过 1 项。后继具名 cron 场景实际1项通过并
经独立复审；channel 丢失观察身份、GUI launch 把 pending owner 写成 Failed 两条独立
before 均已复现，两个同例 after 与普通错误/正常/continuation 等控制共6项通过、独立
复审及15路径必要性检查 PASS。验证使用与实际 CLI 字节/模式一致的冻结消费者和干净
framework `9886502`；不称原生 Tauri UI 或 OS 物理执行验收。all-target/all-feature Clippy
后继实际失败过4项测试 lint；最小修正后原命令 v2 已退出0、零告警，源码与干净 framework
均无漂移。两条局部修复、6项测试与该 Clippy 分别验收，不称其余完整门禁已通过。
这些行为修复、对应测试与必要文稿已在
`dfd5750ce1b00ac1e36a2e778d469f2c3e82b62c` 独立提交并推送；24个必要路径不含原始
日志、临时产物或会话预检。普通、strict 与变更证据检查通过；绑定任务/会话的仓库收据
仍因将5个历史 Git 引用当成当前文件读取而失败，已交插件专项调查，不修改历史锚来消错。

用户已明确保留 `glib 0.18.5`、`proc-macro-error 1.0.4`、`bincode 1.3.3`，记录风险并延期升级；
停止相应升级、替换与回补研究。三项真实审计告警及 exit1 保留；CLI 上述提交已保存精确
Decision 与 `risk_accepted` Finding，绑定版本、registry 来源、checksum 和消费者闭包。
当时门禁处置尚待验证，不称告警消失或缺陷已修复，也不扩大其它告警/编译/测试豁免。
后继窄 checkpoint `32c6719d4f15c4c27b59258a7ff22579357dfe11` 已提交并推送，实现
精确风险门禁及6个反例测试方法；11个文件的独立必要性审查 PASS。官方 actual-lock 与
干净 framework 来源重建的 registry-origin 在线扫描均保留 scanner exit1、三项告警、
零漏洞，单独风险核对 gate0；未关闭 fetch。该次只对扫描进程指定 HTTP/1.1，默认 fetch
失败原件仍保留，不推定更底层原因或默认 Linux CI 已通过。四类 panic Clippy 实际通过。
新全范围复审另发现 shared chat preparation 将 `RecoveryBlocked` 字符串化并错误释放
foreground 的缺口。后继 `629de1ea95627ce8284dae1fcf56221b7d0be476` 已修复并提交推送：
原持久化 pending-intent 重开后的共享入口实际 before 失败，普通错误对照通过；同两项
after 全通过、零告警及输入漂移。四种 surface 参数仍只算一个顶层用例，不称原生界面
E2E。10个必要文件经独立审查 PASS。风险门禁先拒绝新源码，再经独立确认消费者边界未
扩大后重评指纹；锁和三项身份不变，后继双在线结果仍为 scanner1/gate0。旧完整测试在
编译阶段由其唯一 owner 停止，没有行为结果。新切点两类 Clippy 均实际退出0、零告警和
源码漂移；完整 workspace 的首次运行在行为用例前被9 GiB target 守卫中断，原件保留为
资源中断，不算产品 red 或通过。仅官方清理本任务独占缓存后冷跑原完整命令，预算及
覆盖不变；该轮进入行为后出现多项真实失败，另两条忽略 cancel 的永久 pending fixture
无法产生 EOF，已保全未完成结果并仅停止自有进程。代表用例确认 foreground 的关闭分支
可能抢先吞掉已发布 Cancelled 收据；产品读序和资源 scope/真实 EOF/故障阶段等 fixture
问题分开修复，保留强断言，不将全部失败归为 fixture。完整命令、no-default 及 GUI 门禁
均仍须通过，前述局部结果不能替代。
后继精确46例重跑为40成功、6失败、0忽略，零编译告警及输入漂移；parked 与
aborted-reporter 两例确认原终态写入债务的资源被提前释放，分别保有真实 red 和独立
before 产物。现有债务结构携带同一物理 owner、严格确认后才释放的最小修复正在验证。
其余失败涉及真实进程退出模拟、恢复/续跑及故障阶段预期，逐桶保强断言调查；不把
caller future abort 当作进程死亡，也不把目录缺失/不可读直接推为无未结算意图。
后继冻结组合48项实际全通过，零失败、零忽略及来源漂移；原 parked/abort、错误owner、
来源IO、重复ACK、目录恢复及真正子进程退出后重开等控制分别保留。另一个明确声明
输入差异的原产物反例实际失败：取消已真实完成后遇读取错误，原兜底把债务目标写成
Failed，预期仍为 Cancelled。后继复用原取消目标修复该分支，原失败、同owner journal、
单次receipt及精确产物均保留。最终 focused49 实际49成功、0失败、0忽略、零告警及输入
漂移，来源仍为988；不借此覆盖6563后的完整门禁。必要修复及证据已在
`e0f09e80743c7a784ff43bf590f083e7de43fec3` 提交推送；协调侧独立累计检查 main f57 至该
提交的83个路径，全部有保留依据，无 raw/log/PF、临时产物、凭据或新增发布路径。
风险仅重绑消费者指纹，版本、锁和三项接受身份未扩大。后继
`9819a51318df04996103b8b56b2413e59822cb20` 仅补两份历史来源持久化文稿，固定增量独审
通过；累计84路径均有必要性，无新增运行代码或告警豁免。两条精确 evidence refs 的远端
对象、冷仓32条唯一来源及9份对象已独立核对，不借此改写历史观察或声称原生 Stop 通过。
独立 CF 已 clean 切到6563；945项新来源已冻结，完整门禁及精确 main 仍待。
该切点的原完整 workspace 命令后继实际退出101：app-core 单测1647成功、9项既有忽略，
嵌套子进程单列；CLI 库285成功、3失败，后续目标未完成。运行无源码/CF漂移或资源中断。
Channel 原 driver 的实际提前失败与 GUI 均确认缺少测试前置的应用 lifecycle owner；
TUI 原静态字符串提取越过 resume 分支，不能据此认定真实输入进入 ingress。后继复用
公开 owner 绑定并保持原强断言，以真实 TaskRun/Goal 与输入事实验证 TUI；全量仍须重验。
CLI 已按用户指令无损迁入 primary `echo-agent-cli` 的
`fix/Echoyue/cancellation-observation-primary`，C 工作区保留备份；primary framework 的 Exa
任务改动不纳入本次验证。旧 Cargo paths 覆盖产生不支持的依赖图告警，未用于验收。

SDK 后继 `335193791c70a6cb077033c3f8f0cb1223e5266e` 已正常提交并推送，74个文件的
提交树与最终独立审阅树一致，无原始产物入库。A/B 累计生产、协议、三语、生成物和必要
文稿按真实依赖共同纳入该切片，验证限于下述实际用例；另9条独立 Task/protected-claim
fixture 与平台诊断路径仍未纳入。
保存配置四项 dev 行为唯一归原 config；无行为及 Rust warning flags 环境覆盖，保留原
debug/strip/inc/jobs 资源设置的 Host 构建实际退出0、零告警，新产物
`14a0cb58c44bdb84990262f1324e1a2513513a8b40fc0640a9d36f91f95e27e9` 的 EH 为15,488,572
字节，unwind 信息保留。当前共同切点的 drain 两例、unwind 和 direct-run 各一例全通过，
三语言 late-payload 各一例通过，611项语言输入无漂移；旧7控、纯回归及#120另保原切点。
原 canonical 的严格快照、真实基准变更依据及必要性审查通过；19处缺标题及后继来源已
修正，不等于本地原生 Stop 或完整运行验收。deferred 原脚本已修，2528项完整互斥分组，
accepted 仍5621，均不以库存数量充当完成率。

官方对象解析在该已推提交为65 resolved/5 open：三项 Workflow Finding 纳入交付，direct-run
与 late Completed 两个具名缺陷在当前三链、真实验证及独审后有限 resolved；历史原件保留。
其余宽 cleanup、Sandbox、Task resume、Graph claim、managed exit 仍 open，均为已接受行为
或必要 Host 生命周期，不能整体 deferred。B fixture 仍需修复普通 turn 可缺 `run_id` 的
假设，严格区分 foreign producer 与原 TaskRun/attempt 回执。protected-claim 的旧三语言
30步核心校验的 Java 关闭失败已取得分阶段事实：原 Host 在 native writes 完成后仍活着，
五秒时由既有 SDK watchdog 回收为137，并非 stderr/dispose 卡死。后继 A 有限 checkpoint
`404d447f1a69fb4af2843272005b894063888ed9` 已提交推送14个必要文件，精树与已审候选
一致；三语 A 各一例通过，Java 另有合法 local-close 控制一例通过。测试允许本机 transport
清理成功，但无原回执的 A 场景要求实际 Host 非零退出；错误分支精核 typed timeout、
原取消地址、同 owner retry 及清理后 backend 不变，不把137或 transport retry 叫作自然
退出、业务 receipt 或 claim 结算。5份旧来源元数据仅无损改绑可恢复 Git335，canonical
严格/真实基准检查、局部独审及 artifact 检查通过；B四路径和 Linux诊断一条仍未提交。
原 target 下358份诊断已按 bytes/hash 固化到 ignored local，未提交 raw。宽 Graph Finding
仍 open，外部进程退出不能替原 callback 回执；这些局部闭合不关闭外部 Issue #120、SDK
main、持久恢复义务或整体治理。
五项 open 已只读盘点：Sandbox 原 connection-scoped 实现及 cleanup 先于 Cancelled 的
强断言已有；Graph 的 Memory/File、renew/expiry/reopen/CRUD 上游证明也已有，不能泛称
整组未验。后继按精确来源复用、补必要当前用例及正式三链和独审；原历史切点的 Graph
foreign/lateUnknown 仅源码审记录保留，后来补证单列于下段。宽 cleanup 的未证入口与
独立 Linux 十秒诊断仍具名待办。
Graph 后继测试 checkpoint `c1547d6b92201fd3c702782fe0972e1a99e13318` 已提交推送：补齐
foreign 观察不被本地观察覆盖、terminal 后 Unknown 不借先前收据两项负例，五项控制及
默认 workflow 子集93项实际全通过。生产前缀、Node、Store 不变，未出现产品 red；7个
必要文件、最终严格/真实6563基准检查及独立复审通过，无原始产物入库。后继同一精确
提交及1023项冻结输入的六条本地 main 门禁已全部实际退出0、零告警和来源漂移；主87个
harness 为3629成功、0失败、3项既有 live opt-in 忽略，两个各1成功的嵌套子进程单列。
两条新增负例及原 matching 正控在全套实际通过。最终7个必要路径及薄文稿经独立复审，
[PR #192](https://github.com/EchoYue-lp/echo-agent/pull/192) 正常 squash 合入
`f3fa0705f62fe813a5a86bc41cd673f82f915a61`，唯一父为6563、签名有效、tree 与已审候选
一致；精确 main 严格语义/真实6563基准变更检查退出0，精确 main CI `38972460664`
七项全部成功，协调者已独立核对远端身份和各 job。该补证切片完成交付；不改变 SDK
已接受6563生产 pin，也不把 Memory 同实例反例扩称跨 provider 或重启持久化验证。
新 target 的90份引用可执行产物已压缩并逐成员核 SHA，137份原记录前后 hash 不变；
仅官方清理本任务独占 target 后释放6.38 GiB，未触及 SDK/CLI 缓存。
正式 Task 后继 checkpoint `fa62f3b3d0df3ed5b14c6d9786dfffce5db57da6` 已提交推送，精树与
审定候选一致；13个必要文件经独立复审，foreign/Linux 原工作差异保留，无原始产物入库。
仅供后续 foreign 的 UUID 缺省分支已从该候选剔除，要求原 Task 双身份；输入变化后全新
TypeScript/Python/Java 各5项完整 Task 套件实际通过，零失败、零跳过，611项输入各无漂移，
原四类经典场景仍覆盖。同 claim 的 A 只由原 matching attempt 完成一次结算，B 即时
提交 canonical 结果，C/D 消费原依赖结果，B execution 不变；结果限于988/14a切点。
原 ADR0060 和 Tasks 文档明确 next wave 等待本 wave 全部结算；unknown A 仍持 claim 与
已结算 Paused 不同。新用例原“释放 A 前 C 必须完成”的失败保留为超出该合同的 oracle，
不降低原 Paused 场景的独立继续断言，也不据当前实现新增调度承诺。
后继6563 pin 的依赖解析已退出0，656个包的版本、checksum和依赖边不变，仅11个 Git
来源改精确 revision。唯一生成入口实际退出0，28 profiles、117项产物；5621个 accepted
身份及454条路由不变，5份阻断/共享合同逐字节不变，仅4份非阻断遥测变化。
新增 `RunEntry::wait_result` 归 process-local Host/Rust intrinsic，不进入三语言合同。
官方合同检查实际退出0。新保存配置 Host 构建实际退出0、零告警，产物
`ac4332105734bbedda144b5c033fedc6610de4b5b9d9a5a0c11dd4953a28a87f` 的 EH 为15,491,256
字节，保留 unwind；原14a产物与新产物均保全到 target 外。不借旧 Host 验收新 pin。
同 dropped-next 原 waiter 的 quota 单测实际1项通过；Sandbox 原强断言用例补齐原 RPC
真实 Cancelled 回应后同例1项通过，原失败保留为失效 oracle，不称新增生产缺陷。
SDK 历史335/404/637的精确 annotated evidence refs 已按既有来源策略推送；独立空仓只从
远端 main 与三条显式 refs 取回，8份对象、11组观察及42条引用逐字节匹配，无 task branch、
PR 或本地对象借用。它只关闭这三条来源持久化缺口，最终 squash main 冷检仍待。
后继官方语言入口实际退出1：完整 telemetry collector 编译退出0、原 Host ac433 复用，
TypeScript 354项中351成功、3失败、0跳过，Python/Java 尚未启动。三例分别涉及迟到的
factory construction、Workflow stream owner cancel/close 与 cancelled tool callback；
独立源码复核发现旧测试未释放原物理 owner。第一轮修订后最窄运行中 Workflow 三语和
TS Tool 通过；Factory 新增的回执检查却误读只记录 Host→SDK 的单向 trace，Java 另将
invocation timeout 错当成必须取消独立 cleanup owner。此前源码复审漏核这两个边界，
已由实际失败纠正，原审查和失败保留。后继只给既有 proxy 补方向明确的真实回传记录，
并验证原超时之后 cleanup 仍由原 owner 持有；不改生产、Host、pin 或超时，不制造收据。
后继实际复验已完成：TS3、Python2、Java3全部通过；再补强原 RPC 唯一回复断言后，
Factory TS1、Python1、Java2实际全通过，零失败、error、skip；独立源码及原始帧/日志复审
通过。两切点按未变输入衔接，不重复计数或替代完整官方入口。
有限 checkpoint `410e0f1d857bbf8bcfd805376638794697524de9` 已正常提交推送：29个必要
路径经独立复审，tree 与最终候选一致，canonical 严格快照/真实 main 基准检查、artifact、
fmt 和对应语言格式检查通过。仅纳入已资格6563 pin、必要官方遥测、原测试修正及薄证据；
候选四份 Task 文件采用 fa62 原 blob，foreign 后继改动保留为工作树差异，未纳入本提交。必要历史来源
fa62 已有精确 annotated evidence ref，远端 main 加该 tag 的空仓冷取回、七个来源 blob
等价均独立核对通过；旧 Task 的988/14a正文和状态未重写。
协调者已独立核对公开 SHA、tree、父提交及 PR #5 head；PR 仍为草稿。原官方全语言
第二轮后继实际退出1：TS354项中353成功、1失败、0跳过，Python/Java 未启动；前述三类
oracle 本轮均通过。新失败已定位到 collector 启动清单的 create/write 窗口：文件存在时
内容尚为空，真实 Host 尚未启动，业务 flush 未执行。后继修正原进程的完整清单就绪条件，
保原期限、退出与清理边界，先验证已声明的瞬态读取反例，不伪造 ready 或业务成功。完整语言与
Rust、四份后继 Task 改动、五项 open Finding、平台、精确 SDK main 和最终冷检仍待，
不以该 checkpoint 宣称整体完成。

官网独立 checkpoint `28f38b523319f7470a2715f64135395b0f30e3f6` 已提交并推送：290个
framework 镜像逐字节匹配干净 `9886502`，CLI 的118个来源文件和8个投影仍绑定原
`f57b90a2`；未复制 SDK 合同或新增 SDK pin。source-aware 检查及完整官网验证通过，
包括39项单测、306条静态路由、15项浏览器测试，另有1项既有条件跳过；52个文件的独立
必要性审查及 artifact 检查通过，无原始产物入库。CI 改为仅 push main，原 jobs 和来源
检查保留。该阶段 [PR #5](https://github.com/EchoYue-lp/echo-website/pull/5) 已 squash 合入
`faa9987d8da642d59e4a5c8e302d725cf18e36d2`，tree 与上述已审候选完全相同；精确 main
CI `38938673709` 的唯一 verify job 及全部步骤成功，实际检出的来源为 `9886502`/
`f57b90a2`。协调者已独立复核远端 SHA、tree、CI 和实际 checkout 日志。平台 action/
runner 的非阻断弃用提示保留，不宣称 CI 零告警；无部署。最终 EKO 投影与联合验收仍待
CLI main 后收束，不能将本阶段通过代替后继来源更新的验收。
后继框架6563同步已以6个必要文件单独交付：[PR #6](https://github.com/EchoYue-lp/echo-website/pull/6)
合入 `ccdbfb4b35b7b867cf39b059bbca56c3b01fbb9c`，tree 与已审候选一致；精确 main CI
`38960733265` 全部步骤成功。协调者独立核对 CI 及实际 checkout 日志，来源确为6563/
f57，CLI 118个来源和8个投影不变；完整官网验证及独立复审通过。无部署、无原始产物
入库；后继 CLI 投影与联合验收仍须独立完成，平台弃用提示保留。

SDK/CLI 的剩余完整 Rust/GUI 与语言门禁、官网最终来源同步、四仓 main 验收和最终
gitlink 更新仍待完成。Echo Semantic 支撑修复已由其
[PR #4](https://github.com/EchoYue-lp/echo-semantic/pull/4) 交付到
`c0bc306bd36d2b1b76def2b6c7308a07dda4f54b`，用于无损读取 SDK 原有二进制 diff。

顶层协调规则 checkpoint `d36f4c1d` 已推送任务分支；四个 gitlink 尚保持下方历史验收快照。
本轮协调线程为 `01a143e5-5a92-7293-8655-f880c3f698ae`；SDK 任务
`01a14385-3980-70e2-af93-07f29fb43ed2` 独占原 SDK worktree，CLI 任务
`01a143e8-d621-7f81-b968-b08783991777` 独占上述 primary CLI 任务分支，协调者负责官网、
顶层集成和联合验收。framework Issue #120 在 GitHub 仍为 OPEN；SDK 本地 watermark
Finding 与当前 Host 有限验证不能代替精确 SDK main 的最终结果或外部 Issue 结算。
Hook 专项 `01a1442d-160b-7523-a946-32184bc23e72` 已经独审及完整检查并正规本机安装；
P 的真实 pre-edit 根/宿主证据成立，S 的新 pre-edit 记录仍缺失，宿主四项 Hook 显示
`trustStatus=modified`，已请求用户在 `/hooks` 审阅信任。不能据安装成功或编辑未拒绝冒称
整个原生 Hook/Stop 通过；已授权工程工作继续，最终现场验收单列。后继正常提交推进又
触发 Stop 绑定失效，原现场已保全并复现；与历史来源处理一起形成的隔离候选 Node134、
Python95 项实际通过，但正式应用仍待两处受保护既有改动的明确接纳，不据自动续跑授权
覆盖这些文件，也不手改安装缓存或绑定记录。该专项未远端推送或发布。
每个可独立验收里程碑经 focused 验证、复审和必要路径检查后及时提交；正式 squash 前再次
逐项检查完整 PR 文件及引用，raw/log/PF/临时探针不进入 main，仅保留必要源码、合同、
测试、fixture、正式文档、CI 和最小长期证据结论。最终只更新已推送且精确 main CI 验收
通过的子仓 gitlink，不用任务分支 checkpoint 冒充 main 交付。
原主 checkout 的无关差异继续保留；CLI 目录迁移是用户后继明确授权，不推广到其它子仓。
本轮不发布 npm/PyPI/Maven/crates.io 或插件版本，不部署官网；本节不声明全项目已完成。

## 2026-10-11 SDK 独立仓接入

完整 SDK 产品与过滤 Git 历史归属独立 `echo-agent-sdk`，顶层接入第四个 submodule。
Protocol 不依赖 framework；Host 固定已推送的 framework extraction revision `27c7701e`，
不随顶层 framework gitlink 隐式升级。Framework 已删除可编辑 SDK 副本与 SDK CI；
EKO 与官网使用独立 SDK 链接。

兼容门禁只承诺明确接受的跨语言合同。三语言直接读取 accepted artifact，完整 Rust
inventory 数量与 intrinsic membership 是非阻断遥测。SDK 全能力语义治理与发布制品
属于后续独立工作，不把 inventory 数量计入 framework 完成率。验收范围、子仓交付和
用户工作区保留记录见 [SDK 迁移验收](./sdk-migration-final-acceptance.md)。

## 2026-10-11 Background Review 入口结算

按用户 B 选择，#38 的应用层修复纳入本轮交付。CLI
[PR #11](https://github.com/EchoYue-lp/echo-agent-cli/pull/11) 候选
`b34df102e8aca5d787c23d851f9c85bc748521ff` 在 4 项 PR CI 全绿后 squash 合入签名有效的
远端 main `f57b90a2b0962fa5561f8839a3df70be1f261ffa`，Git tree 与候选一致，actual main
strict semantic 通过。原顶层 checkout 的既有差异
完整保留，应用、framework 证据及顶层指针分别使用专用 worktree。

修复 Tauri 正常退出绕过服务结算、workspace bootstrap 恢复身份和 ProductData 回滚顺序；
复用单一 Review owner/Store/generation，保留全部弱投影目标、显式 P/Q 项目指令及同项目
link 幂等，Memory 归因排除 recall telemetry 并对旧指纹保持 unknown。框架公共 API、
SDK、前端契约和 website 无改动。应用双语 ADR 0043 是正式行为说明。

联合定向 94/94、桌面 adapter 7/7、两份独立只读复审 pass；完整本地门禁已通过两档
Clippy、fmt/check、workspace all-feature（app-core 1,611 passed / 9 ignored、CLI 285，
集成/doctest 零失败）、app-core no-default、GUI check/tests（216 + Tauri 1 + JSONL 6
passed，3 ignored doctests）、59 对文档/49 ADR 和 semantic strict/change-evidence。
原生窗口、真实 provider、Tauri restart/setup panic、较早 post-fork 初始化错误的完整
清理尚未验证；发布检查失败的显式清理和 EffectsApplied 恢复是本轮较窄的验证结论。

外部 #38 因真实入口反例重新打开，旧 PR #5 的 resolved 不能代替本次验收。Framework
[PR #180](https://github.com/EchoYue-lp/echo-agent/pull/180) 的7项 CI 全绿后 squash 合入
签名有效的 main `95ec66c6083a8feb17a1c28de0d01e1a24ca8a28`，tree 与候选 `a0c5b95d`
一致，actual main strict semantic 通过；仅改四份语义材料，框架代码/API 无变化。
独立复审 pass，完整 `scripts/verify.sh` 退出码 0。CLI 精确 main run 38106297341 的
4项 CI 全绿；framework 精确 main run 38107875170 的7项 CI 全绿。
#38 已在 repair、verification、independent rereview 全部进入主线并验收后关闭，
[关闭记录](https://github.com/EchoYue-lp/echo-agent/issues/38#issuecomment-6105020331) 绑定实际快照和收据。
SDK #120 与 A2A #32–35 继续排除，既有 release/soak 阶段不在本轮重开。

## 2026-10-10 上下文压缩远端集成

框架 [PR #178](https://github.com/EchoYue-lp/echo-agent/pull/178) 已在 7 项 CI 全绿后 squash
合入 `e5372b8ca3dc308ce8dde4a0092ea58b8d7dd21d`，签名 verified，Git tree 与完整验证的
`13a8b285` 候选一致。Summary、IncrementalSummary、SlidingWindow 共用 token/turn
尾部选择，当前请求和工具组保真；取消、fallback 和增量观察缓存复用 ContextManager 权威。

随后修复了远端集成发现的 process-group cleanup 前置缺陷：[框架 PR #179](https://github.com/EchoYue-lp/echo-agent/pull/179)
在本地完整门禁（3,398 tests）与7项 PR CI 全绿后合入 `1486d2f4a26adbefab393679c58da2804e5f4a75`，
签名 verified、tree 与已审候选一致。负数 PGID 通过 `kill -KILL --` 明确分隔；两条原清理
路径继续拥有取消、超时和回收权威。Ubuntu 的无信号 syscall probe 证明原解析缺陷，但不
宣称已证明历史 cancelled CI 的具体因果。

应用 [PR #10](https://github.com/EchoYue-lp/echo-agent-cli/pull/10) 在4项 CI 和完整本地
Rust/GUI 门禁通过后合入 `f98f50985c9dbc8aad8c1c7e873a8997fe5024f0`；官网
[PR #4](https://github.com/EchoYue-lp/echo-website/pull/4) 在2项 verify 通过后合入
`b1609b126014bf0cb120e68a2a665a9c64b8fa54`。两者签名 verified、tree 与验收候选一致。
CI 复用已审预算拆分、保留全部原检查与失败聚合；官网 manifest 绑定上述实际 main。
EKO 的 default policy、goal/recovery projection、focus 与 journal 仍属于
应用层。独立复审全部 pass，剩余 findings 0；本地框架 3,396 tests/17 feature、应用
1,922 tests、GUI 218 tests、前端 287 tests 和官网 39 unit/15 E2E 已通过。
完整源码、文档与验证材料见[压缩验收记录](./supreme/plans/2026-10-09T1545-eko-context-compression/verification.md)。
三仓最终 main push CI 全绿，精确run与snapshot见验收记录。三个 submodule 指针
在本次顶层交付中同步，保留原工作区与其它线程成果；
不将本次本地/CI 证据扩展为真实模型或原生 EKO 窗口验收。

## 2026-10-08 GUI Fork 增量交付

EKO 应用候选 `8bd2a6f` 已提交、推送至 [CLI PR #8](https://github.com/EchoYue-lp/echo-agent-cli/pull/8)。完成回复下方的独立 Fork 与普通会话列表已接通，复用原有 branch IPC、managed import 与会话权威。正式行为说明和双语 ADR 0045 均在应用子仓库维护；framework、普通 TUI `/fork` 与 Side Conversation 的分层边界保持既有约定。

候选本地完整门禁通过：两档 Rust Clippy、fmt、workspace all-features（app-core 1594 passed / 9 ignored、CLI 281、main 11、JSONL 6）、app-core no-default、GUI check/tests（212、Tauri main 1、JSONL 6）、前端 Prettier/ESLint/290 tests/build、57 对双语文档与 strict semantic snapshot/change-evidence。已修复完整测试暴露的 live-steer 固定睡眠竞争，用流式入口和持久 mailbox acceptance 同步测试，未放宽断言。浏览器使用真实组件与本地 API fixture 验证分叉选中及源/分支独立列表条目；不宣称原生 Tauri 窗口验收。

该候选已包含在 CLI PR #8 的最终 squash main `40fd804` 中，Fork 与自动协作边界按下节共同交付。Framework examples 与 echo-website 不消费这个 EKO 回复操作入口，无需修改。既有阶段的全局 release/soak 结论不由本次 GUI 增量改写。

## 当前结论

### 2026-10-09 合并后 Docker 测试门禁收尾

framework `d8a73701` 的 main CI 曾在 invalid-create cleanup fixture 间歇失败，同一提交重跑通过。`echo-agent` [PR #177](https://github.com/EchoYue-lp/echo-agent/pull/177) 已合入 `a8c2d1ae3fce675633ea20d4e3d49258c327f634`，7 项 CI 全绿，Git tree 与本地验证候选 `707f5279` 一致。普通注入 CLI 恢复到原生产 control deadline，仅 dedicated hung info/create/rm 测试保留 100ms。有限延迟 200ms 后返回 invalid ID 的确定性回归在修复前失败、修复后 StartFailed 与 info/create/rm 结算通过。

本地 Docker 28/28、受影响 echo_execution 331/331、完整 workspace/all-target/all-feature tests、两档严格 Clippy、fmt、workspace no-default 与 strict semantic snapshot/change-evidence 均通过。生产 Docker 配置/执行、GUI、Sidechat/Fork 与 AI 自动协作行为没有变化；CLI、SDK、website 不消费 test-only override，examples 已进入 all-target 链路。此阶段仅同步 framework 修复指针与验收材料，不重开其它旧阶段。

顶层同步 framework squash main 指针。EKO 开发版持续运行，旧用户 journal 的格式警告不由本次测试修复解决。CLI `40fd804` 的 PR 检查均成功；main push run `37802448671` 的 frontend 与 Linux lint/no-default 均通过，Linux app-core test job `113397896291` 被取消且平台未保留可下载日志，不能据此认定代码失败，也不能声称该 main run 全绿。已请求精确重跑该 job，其结果单独结算。

### 2026-10-08 Agent 自动协作边界

按用户明确决定，EKO 不提供人工发协作消息、维护 Agent 组或管理投递收据的页面。CLI `f014323` 删除旧弹窗、组编辑器、聊天入口与无消费者的 GUI adapter，并完整撤回未提交的动态管理台/查询。模型工具、AgentRouter/AppState、TaskRuntime 与 Sidechat/Fork 保留；正式产品决定在 CLI 双语 ADR 0047 和 top-level/CLI AGENTS 同步维护。

本地 Rust 1.99 完整门禁全绿：app-core 1594 passed/9 ignored、CLI 280、main 11、JSONL 6、GUI 211/main 1/JSONL 6，前端 287 tests/build、两档 Clippy、no-default、58 对文档/48 ADR 与 strict semantic deletion evidence。人工 UI 有意退役，保留核心与 Side/Fork 的验证不冒充 UI 等价。

Framework [PR #176](https://github.com/EchoYue-lp/echo-agent/pull/176) 已合入签名有效的 `d8a73701768a3433c8b2afd5de08da573754673c`，7 项 CI 全绿，Git tree 与验证候选 `c13937b5` 一致。修复 Rust 1.99 atomic API 与 generated must-use lint，保留 Rust 1.95 minimum 和原 lazy dispatch；完整 workspace/all-target/all-feature、no-default、18-feature 与 semantic 门禁通过。CLI [PR #8](https://github.com/EchoYue-lp/echo-agent-cli/pull/8) 已合入 `40fd80447cdf3d7627399af0b2069a8b4da5327a`，Linux 与 frontend CI 全绿，Git tree 与验证候选 `f014323` 一致。本顶层 [PR #16](https://github.com/EchoYue-lp/eko-workspace/pull/16) 同步两者 main 指针；旧用户 journal 的恢复警告未通过清空数据或迁移绕过。开发版已重新启动，运行新源码且人工 Agent 协作入口已移除。

F0-F6、R1、R2 和 R3 文档/website 收敛已完成；G 的当前静态门禁已完成，但完整测试与 Final Integration/Release 仍为 conditional，因此整个优化迭代不能宣称发布完成。

| 阶段 | 当前状态 | 结论与证据入口 |
| --- | --- | --- |
| F0 characterization | Complete | 已进入 CLI `main`，作为后续交互回归输入；细节见 [`agent-interaction-convergence-plan`](./2026-08-26-agent-interaction-convergence-plan.md)。 |
| F1 receipt/admission | Complete | Persisted/Accepted/Drained/TurnSettled 已收敛；framework 与 CLI child baseline 已冻结。 |
| F2 Task/Plan/Todo authority | Complete | `TaskStatus` 是唯一执行权威，Todo 仅只读投影；ADR [0015](../echo-agent-cli/docs/zh/adr/0015-task-graph-status-authority.md)。 |
| F3 Agent control tools | Complete | 六个 `agent_*` 工具复用既有 router/subagent/task authority；bounded query 已下推。ADR [0016](../echo-agent-cli/docs/zh/adr/0016-agent-control-tools.md)。 |
| F4 删除 InteractionMode | Complete | 生产 Rust、DTO、prompt、surface 旧 mode 路径已删除；不得以同义名称恢复。 |
| F5 Agent/Subagent lifecycle | Complete | characterization、生产修复、测试卫生和适用完整门禁已通过；统一使用 Subagent 术语。 |
| F6 cursor/recovery/surface parity | Complete | restart cursor、cold address、workspace switch/delete、boot reconcile、terminal exactly-once 和五入口 canonical fixture 已形成 executable 闭环。 |
| R0 app/framework boundary audit | Complete | 151 个 app-core Rust 文件已分类，现已升级为 R1 final closure ledger。见 [`boundary closure`](./2026-08-28-current-framework-application-boundary-audit.md)。 |
| R1 framework-first migration | Complete | 19 个 M 与 8 个 C 均有最终 disposition；turn、TaskRuntime、artifact、bootstrap、diff、plugin、memory、tool-control、background 和旧路径已收敛，无第二 authority。 |
| R2 examples convergence | Complete | 43 个教学/组合 demo 与 21 个 executable contracts 已统一迁入 `echo-agent/echo-agent-learning`；feature、panic、UTF-8、facade 与链接合同通过。见 [`framework-examples-inventory`](./2026-08-28-framework-examples-inventory.md)。 |
| R3 framework docs/website | Complete / 2026-10-10 验收快照 | framework 与 EKO 正式双语文档、示例路径、discovery 与 source-aware sync 在该次交付完成；website manifest 绑定 CLI `f98f509` 与 framework `1486d2f4`，对应 2026-10-10 的压缩交付快照。本轮私有生命周期修复与证据交付不更新 website，不把历史站点快照当作本轮 main 验收。 |
| R4 app-core global modularization | Complete | app-core authority modules are physically split behind `echo_agent_app_core::api`; CLI/TUI/Tauri/channel/examples/tests use the facade. Wire and persistence contracts remain unchanged. R4 validation remains pinned to code `0e762ab` and framework `125ea5f`; current child baselines are listed below. See `echo-agent-cli/docs/zh/adr/0025-app-core-global-modularization.md`. |
| Framework capability placement correction | Complete | 通用 primitive 不再以第二消费者作为准入门槛；AgentPool/AgentRouter/ChatEventLog/Plugin/Extension 的逐符号 disposition 见 [`framework capability placement audit`](./2026-08-30-framework-capability-placement-audit.md)。 |
| AgentPool keyed admission kernel | Complete | `echo-agent` 的 `KeyedExecutionAdmission` 负责 opaque-key lease、per-key process permit、retirement、close/wait；EKO `AgentPool` 保留 Agent cache、capacity class、workspace、plugin/model/tool policy。 |
| AgentRouter delivery ledger contract | Typed API complete / development schema reset | framework typed `DeliveryLedger<Route, Payload>` 与统一 `DeliveryTransition` 已成为唯一 lifecycle/projection/retry authority；EKO 直接使用 `AgentAddress`/`AgentMessage`，旧 `ClaimSettlement`、`AgentInboxProjection`、`FoldedDelivery`、legacy event wire 与 checkpoint codec 均已删除。 |
| Framework-native domain values | Complete for current migration lane | EKO 权限状态直接保存 framework `PermissionRule`；matcher、behavior、source 由 framework `FromStr` 校验；TaskRuntime 直接保存 framework `TaskExecution`；Subagent 与 conversation-input outcome 直接使用 framework 类型；framework 配置通过标准 `From<FrameworkConfig> for AgentConfig` 转换；delivery/subagent 对外入口统一为 `echo_agent::delivery`、`echo_agent::subagent`，两类执行结果直接返回 `ExecutionUsage`，不再保留 split-crate workspace facade、转换 trait、framework-shaped DTO 或 `to_framework_*`/`from_framework_*` helper。ADR framework `0021`、EKO `0027`。 |
| Typed LLM timeout contract | Complete for framework/API integration | framework `LlmTimeouts` 同时拥有 non-stream request 与 stream first-chunk/idle/overall 边界；`LlmConfig` 提供 client 默认，`ChatRequest` 使用同类型覆盖单次调用。Chat Completions、Responses、Anthropic 共用唯一 SSE transport；旧 `ECHO_AGENT_STREAM_*`、Anthropic 私有 byte loop、固定 reqwest 120s stream 总超时和 `llm::{core,integration,config,providers}` 迁移 facade 已删除。ADR framework `0022`。 |
| Dead AgentRunner contract cleanup | Complete | `src/runner.rs` 从未进入 module tree，且 deprecated facade 文档引用不存在的 context/eval builder API；现已删除。Agent 构建、trace 与 eval 分别直接使用 `ReactAgentBuilder`、`RunStore`、`EvalRunner`，Eval/Tracing 双语文档和 website 不再声称存在自动 recorder 或第二 runner。 |
| Current EKO product schema authority | Complete | `.eko/learned-rules.md`、`.eko/workspace.json`、原样 cron prompt 与 `TaskRuntimeStore::new()` 是唯一当前输入。`.eko/AGENTS.md`、root `.workspace.json` reader、`[plan]` marker strip、`TaskRuntimeStore::open()` 和生产 run-id-only wrapper 已删除；retired workspace marker 只用于防覆盖，journal rebuild 与保守 worktree cleanup 继续保护当前数据。EKO ADR `0028`。 |
| Framework-native Agent/model config | Complete | EKO 直接保存 framework `AgentSettings`，并以产品默认结构化合并部分 YAML；顶层 `model` 只含 `default_model_id` 且拒绝未知 mirror 字段；`ConfiguredModel`/`ModelProviderConfig` 分别唯一拥有模型与连接事实；`resolve_runtime_model -> Result` 是所有 surface 共用的唯一 resolver，不再合成 synthetic model。EKO ADR `0029`。 |
| Current Skill format authority | Complete / official format enforced | `SkillDocument::parse/parse_at` is the sole parser/writer authority. File frontmatter accepts only agentskills.io fields, rejects private extensions, invalid types/null, name mismatch, and official length violations; Skill files carry no private Hook sidecar, and host/plugin Hook configuration stays separate. `validate_skill_markdown/validate_skill_dir`, SkillsHub, install, Plugin validation, SkillMerger, and examples use the same contract. |
| Enabled Skill 运行时权威 + catalog 收缩 | Complete / locally merged; remote delivery pending | `enabled-skills.json` 只保留 `{category, enabled, baseline}` flat policy 与原子写，损坏配置 fail-open；generation CAS、operation identity 与 repair debt 已删除。catalog 41 → 39 → 24，默认启用 8 → 5，methodology baseline 4 → 1。TUI/GUI 可激活当前会话 Skill，Tauri bundle 使用官方 resource dir，Agent Plugins 1.0 的 skills 面按包原子安装。framework `ac00815`、EKO `f194c43`、website `35490e8` 已在各自本地 `main`；尚未 push，顶层 gitlink 因此尚未提交。EKO ADR `0032`、`0033`、`0036`。 |
| Unified Subagent prompt architecture | Complete / static gates passed | framework `SubagentPromptCompiler` 统一拥有 stable system prompt 与 typed invocation messages；真实 registered capability、`ToolVisibilityPolicy`、typed `access_mode`、effective workspace、allowlist override、结构化历史与附件均在单一编译链收敛。direct/planned/fork/teammate/team member/plugin/primary TaskRuntime 共用该合同；角色 Markdown 只保留角色、方法和领域知识。framework ADR `0024`、EKO ADR `0030`。 |
| Deterministic CommandCell watch | Complete / static gates passed | framework `CommandCellWatcher` 持有 observation lease、按 byte cursor drain 并返回 typed terminal；EKO 只保留 exact workspace/conversation/root、generation、Ready/delivery/ack、恢复与 surface policy。模型驱动的 `awaiter` Subagent、provider summary/status、Subagent attempt identity 与旧 active spec 已删除。framework ADR `0025`、EKO ADR `0031`。 |
| Bilingual docs parity foundation | Complete | CLI 已提交 40 对 zh/en 镜像文档（含 30 个 ADR）和 fail-closed checker；website sync 强制执行 parity 并记录 sourcePath/hash。 |
| Repository hygiene | Complete / runtime scope protected | 已清理 20 个空缓存/源码占位目录，并在全部验证后按磁盘规则清理两个 Cargo target 缓存（共释放 48.5 GiB）；tracked audit `.txt`、website `.txt`、115 个 `.eko` runtime trace、8 个 soak roots 和验收 worktree 均保留，未按扩展名删除运行数据。 |
| G Final Integration/Release | Conditional / full gates pending | 当前 framework/app all-target compile、严格 Clippy、双语 parity 与 website static checks 已通过；测试、frontend build、GUI、soak、人工 GUI、远端 CI 和 release 尚未重新验证。 |
| 长程任务优化(统一 turn-run 绑定) | Closure complete on local `main` / `echo-agent-cli@be273d3` + `echo-website@ae91c39`,未推送 | `feature/unified-turn-run@9524b47` 的 9 项 review 修复已作为基线,closure 补齐 typed provenance memory gate、quiet-check 与 RunTurn claim 原子复核、五入口长 steer 原文保真、`ForegroundTurnSnapshot.run_id` TS 绑定和 `PlanRevisionCommitted` 任务 UI 激活。第三轮独立复审 Critical/Important/Minor 均为 0。CLI 门禁已通过 fmt、Clippy 双档、workspace all-features(app-core 1534 passed/9 ignored、CLI lib 276、main 11、JSONL 6)、no-default、GUI check/tests(207 + main 1 + JSONL 6)、前端 Prettier/ESLint/Vitest 244/build、双语 parity；website source-aware docs check 与完整 verify 通过(39 单测、174 static/discovery routes、Playwright 15 passed/1 skipped)。后续项(独立 ADR):task_execute §10.1 同步 await 合同的后台化。 |
| echo-agent 全 workspace 语义治理 | Baseline complete / 当前修复范围已闭合 | 最新 framework 清单为 96 个 Finding：91 resolved、5 open；5 个 open 是本轮排除的 SDK #120 与 A2A #32–35。#38 的 CLI 修复、framework 三类证据及两仓精确 main CI 已验收，外部 Issue 已关闭；不以旧 resolved 代替本次验收。治理单位为 Capability / Behavior / Rule / State Authority / Lifecycle / Finding / Evidence；SDK identity inventory 不计项目完成率。历史路线保留于 [`delivery map`](./supreme/plans/2026-09-13T1448-echo-agent全workspace语义治理/delivery-map.md) 与 [`优化清单`](./2026-09-14-echo-agent-semantic-governance-findings.md)。 |

## 2026-10-11 已验收基线（历史快照）

| 仓库 | 本轮验收快照 | 远端/发布状态 |
| --- | --- | --- |
| `echo-agent` | `95ec66c6`（精确远端 main） | PR180 已合入签名有效的远端 main，tree 与已审候选一致；完整本地、PR与精确main各7项CI、actual main strict semantic通过，#38已关闭。 |
| `echo-agent-cli` | `f57b90a2`（精确远端 main） | PR11 已合入签名有效的远端 main；tree 与已审候选一致，完整本地、PR 与精确 main 的4项CI、strict semantic 通过。 |
| `echo-website` | `0dc2dc5`（SDK projection main） | PR2 已合入远端 main；source-aware manifest、完整 verify、PR 与精确 main CI 通过，正式来源只取已提交 revision。 |
| `echo-agent-sdk` | `30d7e21`（SDK independent main） | PR4 的 8 项 CI 与精确 main 的 8 项 CI 全绿，tree 与已审候选一致；完整本地及独立检出验证通过；只承诺 accepted contract，源码交付证据见 SDK 迁移验收。 |
| superproject | PR20 的四仓快照 | 从最新 origin/main 隔离，新增 SDK 第四 gitlink、同步 website 已交付指针及跨仓规则；远端四仓新检出验证通过，原 checkout 的未提交材料和旧 task worktree 保留。 |

F2-F5 的合流与门禁证据集中在 [`plan_03`](./supreme/plans/2026-08-28T0013-项目未完成工作收敛/plan_03_F5收口完整验证主分支合并与资源清理.md) 及两个 child MASTER-PLAN；这些文件记录历史实施证据，不替代本节状态表。

## 分层与不变量

- framework 保持通用 EventEnvelope、turn/checkpoint、Task DAG/revision/validator、Subagent lifecycle、Tool/Store、MCP/LSP/channel、scheduler、HITL 和通用 bounded I/O 原语。框架的合理公开能力（包括 SQLite store）不得因 EKO 不调用而删除。
- EKO 保持 workspace、DomainProfile、产品配置、TaskRun 文件权威、review/worktree、文件/工件/记忆策略、surface 投影、pool、应用生命周期和本地权限策略。EKO CLI 不启用 SQLite。
- R4 的 app-core 物理模块化不改变上述 owner：`state/`、`tasks/task_runtime/store/`、`tasks/task_runtime/executor/`、`agent_router/`、`chat_event_log/`、`agent_pool/`、`extension_control/`、`plugin_runtime/` 和 `infra/` 通过单一 `api/` facade 对外；没有第二 runtime/store/DAG/status/publication authority。
- 只有一个 turn driver、一个 receipt lifecycle、一个 revisioned Task graph/validator/ready frontier、一个 Conversation Agent router 和一个 attempt-scoped Subagent control owner。不得新增第二 runtime、mailbox、store、DAG loop、status reducer 或 mode 替身。
- GUI、TUI、CLI/JSONL、channel、cron/background 共享同一核心能力；surface 只渲染/适配，不自行推断运行终态。内部执行角色统一使用 `Subagent` 术语。
- 取消、失败、部分副作用和恢复必须以持久事实为准；不宣称任意副作用 exactly-once。所有字符串预览使用 UTF-8 安全字符迭代，外部输入不得触发 panic。
- 语义修复严格执行一 Finding 一 GitHub Issue：Issue 正文包含唯一 `echo-semantic-finding` marker，Finding 回写唯一 URL；新问题先建 Issue 再进入修复，只有修复进入远端主线且 repair、verification、independent rereview 三类证据闭合后才关闭 Issue。

## 已完成收敛

- F6：Conversation/TaskSubagent cursor 跨重启恢复、cold/unloaded address、workspace generation/delete、boot reconcile、terminal exactly-once 和 GUI/TUI/CLI/JSONL/channel 同 fixture 已通过。
- R1：framework producer first、EKO product integration second、旧路径同阶段删除。最终逐项证据和 SHA 见 [`boundary closure`](./2026-08-28-current-framework-application-boundary-audit.md)。
- R2：`echo-agent-learning` 统一承载 43 个编号 demo、13 个 Rust 学习章节和 21 个 executable contracts；正式 framework 文档只引用 learning package 的示例路径。
- 全方向通信矩阵（2026-09-03）：框架 `feature/agent-communication`（SubagentLineage/UplinkSink/共享控制面/subagent_message+list，ADR 0027，demo50）与 CLI `feature/agent-communication`（EKO uplink sink、escalation→NeedsInput、同 run 兄弟投递、8 角色全开 can_delegate、agent_spawn/resume/handoff/group 四工具，ADR 0034）在各自 worktree 分支完成并通过 fmt/clippy 双档/feature 矩阵/gui bin/前端三件套；CLI 全量测试 76 项失败全部为运行中 EKO 实例持有 `~/.eko/tasks` 文件锁的环境冲突（零断言回归），待实例退出后复跑补验再合并。

## 下一阶段路线

### echo-agent 全 workspace 语义治理（当前修复范围已闭合，2026-10-11）

全 workspace discovery、状态权威/生命周期审计及 Finding 修复已持续进入框架 main。
当前实际清单为 96 个 Finding：91 resolved、5 open；5 个 open 均是本轮排除的 SDK #120
与 A2A #32–35。GitHub 功能请求 #10 不是语义 Finding，不计入该清单。#38 的新入口
验收见本文件 2026-10-11 阶段记录；当前纳入目标的已确认 Finding 均已交付并关闭。

本轮只完成当前范围的终态、恢复、持久化和副作用结算，不重开 SDK/website 或产品发布。
后续发现须以最新 main 的真实入口、唯一状态权威、取消/恢复与持久事实反例为依据，按依赖
关系形成可独立验收的 Finding 切片；完整门禁只在稳定的 PR/main 交付点执行。
SDK inventory 继续作为漂移监控和独立 backlog，不按 identity 数量衡量项目完成度。

统一Tool Surface / ToolRouter草案的首次复审结论为block：surface必须按model request/iteration generation冻结，必须复用现有`ToolInvocation`/`ToolRuntime`/`ToolExecutionContext`，execution traits不得取代动态Permission与现有ToolManager/network生命周期。草案在修订和复审通过前不是正式ADR，不得进入实现。

### 全方向通信矩阵收尾（Complete，2026-09-03）

76 项锁冲突测试在 EKO 实例退出后复验全绿（CLI 全量 1844 通过 0 失败）；squash merge 落地：echo-agent main `53473e3`、echo-agent-cli main `8a7fd85`，均已推送；worktree/分支/symlink 已清理；superproject gitlink `d8388dd` 已推送。合并前两仓主 checkout 的遗留 WIP 已 stash 保留：echo-agent `archive/pre-agent-communication-merge-wip-critic-schema-20260903`、echo-agent-cli `archive/pre-agent-communication-merge-wip-pool-primary-frontend-20260903`（未自动 pop，由所有者决定恢复时机）。

### R3：framework docs/website（Complete）

framework 中英文 README、公共 API/feature/examples 文档和 EKO 正式产品文档已按 framework `d9cb003` 与 CLI `c29ca20` 完成复核；typed Delivery Ledger、immutable `SkillDocument`、official Skill format/validator、typed Task/Subagent/Tool/LLM API、统一 Subagent prompt compiler、registered capability/visibility/access authority 与 deterministic CommandCell watch 均已同步到 website，source/discovery/site checks 已通过；本阶段未运行完整测试、frontend build、GUI、soak、release 或浏览器人工门禁。

### G：Final Integration/Release

在 R3 完成后一次性执行三仓适用完整门禁、fault matrix、10k/100k release 性能、10 分钟/1 小时/最终 2 小时 soak、人工 GUI、远端 CI 修复与复验、child-first push 和 superproject gitlink 更新。所有命令必须真实 exit 0；被阻塞或中断必须保留 exit code 和原因，不得称为通过。

## 证据与文档索引

- 交互/任务路线与依赖：[`2026-08-26-agent-interaction-convergence-plan.md`](./2026-08-26-agent-interaction-convergence-plan.md)。
- 扩展与最终集成边界：[`2026-08-26-extension-control-final-integration-unified-plan.md`](./2026-08-26-extension-control-final-integration-unified-plan.md)。
- R1 boundary closure：[`2026-08-28-current-framework-application-boundary-audit.md`](./2026-08-28-current-framework-application-boundary-audit.md)。
- R2 examples inventory：[`2026-08-28-framework-examples-inventory.md`](./2026-08-28-framework-examples-inventory.md)。
- Tools/app-core boundary assessment：[`2026-08-29-tools-app-core-boundary-assessment.md`](./2026-08-29-tools-app-core-boundary-assessment.md)。
- Framework capability placement audit：[`2026-08-30-framework-capability-placement-audit.md`](./2026-08-30-framework-capability-placement-audit.md)。
- AgentRouter delivery ledger phase 1：[`plan_06_冻结AgentRouter通用ledger契约`](./supreme/specs/2026-08-30-framework-boundary-and-bilingual-docs/plans/plan_06_冻结AgentRouter通用ledger契约.md)。
- AgentRouter delivery ledger phase 2：[`plan_07_AgentRouter生产ledger切换`](./supreme/specs/2026-08-30-framework-boundary-and-bilingual-docs/plans/plan_07_AgentRouter生产ledger切换.md)。
- AgentRouter delivery ledger phase 3：[`plan_08_AgentRouter删除旧projection`](./supreme/specs/2026-08-30-framework-boundary-and-bilingual-docs/plans/plan_08_AgentRouter删除旧projection.md)。

以上三份计划均为 `status: superseded` 的历史工程记录。它们原先规划的 adapter、legacy
wire/checkpoint codec 和旧 projection 保留策略没有延续到当前实现；typed framework
`DeliveryLedger` 与 EKO 直接 typed 集成的 ADR 0019/0026 才是现行事实源。

- Repository hygiene：[`2026-08-30-repository-hygiene.md`](./2026-08-30-repository-hygiene.md)。
- Skill 系统重构（Phase 0-5 已实现、复审、验证并本地合并，待 remote delivery 与顶层 gitlink）：[`2026-09-03-skill-system-refactor.md`](./2026-09-03-skill-system-refactor.md)。
- EKO website projection review：[`2026-08-29-eko-website-projection-review.md`](./2026-08-29-eko-website-projection-review.md)。
- EKO 应用事实源：[`echo-agent-cli/docs/zh/project-status.md`](../echo-agent-cli/docs/zh/project-status.md)、[`architecture overview`](../echo-agent-cli/docs/zh/architecture/overview.md)、[`features`](../echo-agent-cli/docs/zh/features.md)。
- Agent control：[`ADR 0016`](../echo-agent-cli/docs/zh/adr/0016-agent-control-tools.md)；bounded query 已由生产 store API 下推。

历史 `docs/superpowers/`、`docs/supreme/` 计划和旧审计只作为可追溯证据，不能直接授权新代码修改；新阶段需先创建同主题专项 plan/ADR（若涉及架构），并注明 owner、删除目标和退出门。

## 提交与验证约束

跨仓修改顺序固定为 framework → CLI → website → superproject gitlink。提交显式关闭 GPG 签名：

```text
git -c commit.gpgsign=false commit -m "..."
```

提交前按 `AGENTS.md` 执行与改动匹配的 fmt、clippy、workspace tests、no-default/feature matrix、GUI/frontend 和文档链接检查；磁盘不足时才按规则清理对应 `target/`。本文件不把未执行的性能、soak、人工 GUI、远端 CI 或 release 命令写成通过证据。
