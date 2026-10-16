# EKO 总纲（MASTER-PLAN）

> **跨仓库当前事实源**。本文只记录阶段状态、前向路线、分层边界、验收门和可核验证据；不再保存逐轮实施日志。
> EKO 是本机个人助理：`echo-agent` 是独立可复用框架，`echo-agent-sdk` 是独立 SDK 产品，`echo-agent-cli` 是应用层，`echo-website` 是官网。
> 最后更新：2026-10-17（SDK 全能力治理阶段快照；尚未完成联合验收，历史阶段保留原验收范围）。

## 2026-10-16 SDK 全能力治理（进行中）

治理范围仍以明确接受的跨语言合同为准，完整 Rust public inventory 仅作遥测。
用户已批准 Workflow `protected_claim_v1` 与
`workflow_checkpoint_protect_claim(checkpoint_id, attempt_id)` 反向操作：原 Store 在节点副作用前
保护同一 claim，未知执行不能因租约到期重跑，也不能由 CRUD 解除；缺能力时执行前明确拒绝。
取消结算采用已批准的 B：观察等待有界，原物理执行、claim 和 permit 保留到原执行真实结算，
不得用超时、丢弃 handle 或合成终态替代。正式合同、实现与限制分别由 framework/SDK 维护。

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

SDK 已固定上述通过 CI 的精确 framework revision；
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
编译阶段由其唯一 owner 停止，没有行为结果；两类 Clippy 与完整矩阵正在新切点重跑。
CLI 已按用户指令无损迁入 primary `echo-agent-cli` 的
`fix/Echoyue/cancellation-observation-primary`，C 工作区保留备份；primary framework 的 Exa
任务改动不纳入本次验证。旧 Cargo paths 覆盖产生不支持的依赖图告警，未用于验收。

SDK 的 B fixture 错误地要求普通 turn 必有 `run_id`，需保持可选身份合同并区分 foreign
producer 与原 TaskRun/attempt 回执；Workflow protected-claim 三语言核心30步校验均完成，
但 Java 在关闭阶段报五秒 deadline，整个用例仍失败，具体关闭停点未证。SDK 语义材料的
19处缺标题与后继来源仍需在精确候选验证；22份历史对象的947组来源引用已按上述独立
checkpoint 修正，不能将历史对象批量改为当前来源。当前摘要与 deferred 分组也需最终冻结
核对。以上反例、范围与原件留在各任务的 ignored local，正式三链随子仓阶段提交收束。
本次官方对象解析区分已推 `55bf069` 的63 resolved/4 open，与私有候选和当前工作树的
63/7；后三项 Workflow Finding 尚未公开交付。七项均属已接受行为或必要 Host 生命周期，
不能整体标为 deferred。独立复审新增 Workflow 满队列发送阻挡取消、cleanup join 的风险，
已取得真实产品 before；最小修复后同两项含正常 drain 对照全部通过，增量独审 PASS。
保存配置的四项 dev 行为唯一移到原 config，删除告警屏蔽，未新增 warning 策略；原目录
入口和树外调用差异已写 ADR。新候选的 canonical 严格快照/真实基准变更依据检查及74个
路径必要性审查通过，但新 Host 冷构建和受影响运行验证尚无终态，暂不提交该候选。
direct-run 与 late-payload 的有限红绿及三语后继结果已补入原三链，状态仍 open 等待最终
共同切点。deferred 文档过时 intrinsic 假设已修，自带脚本实际通过，2528项完整互斥分组，
不扩大 accepted 合同。不能将这些局部或结构通过扩为整项生命周期关闭。

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
整个原生 Hook/Stop 通过；已授权工程工作继续，最终现场验收单列。该专项未远端推送或发布。
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
