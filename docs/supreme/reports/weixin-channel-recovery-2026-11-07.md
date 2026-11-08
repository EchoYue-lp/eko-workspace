# 微信渠道恢复：阶段实现与验收记录

## 当前归档交付指针（2026-11-08，Asia/Shanghai）

本地源码交付分别是 Framework IM `1f35f0d2f9274c60f9cb009f29675efc33f80422`（会话关闭/准入观察，channels 局部 171 项含新增 9 项）、CLI 后端/IPC `70ac06a52479ea59b6bd9f67df1173e891b80b24`（15 路径功能组）、Accepted 观察 `bd2a083ee6b24ab27864b8becdb3082f1dff3494`（A2/logger/ADR 三路径）。中央对应交付原件为 `cli-im-backend-commit.json`、`cli-im-accepted-observer-commit.json`；文件位于主根 `.echo-semantic/local/coordination-20261107-r01/`。提交分组不把原冻结工作树验证改称独立 commit-tree 或 main CI 资格。

| 已取得的切点资格 | 原 source manifest SHA256 | 保全程序及结果 |
|---|---|---|
| R08 相关模块 | `7656db4208fbeee224331b9bd5964201e06c9d9e1fd3ee18ee0c66105f6f05dd` | `1bca2d14…`；293 个唯一测试通过，八项重点 exact 已包含 |
| R09 长回复/相邻控制 | `2b63cd1fe60232e0b7e86b2a95ed4847c4d4a09f8d160e00326c63e6317495c5` | `7bbeda7f…`；长回复及四项相邻 exact 各 1P/0F/0I |
| R13 Accepted 观察/相邻控制 | `dba4fa79d0c437a4a7b6ec7c33335ea2a36ca5114390454cf6e1f95e33bb1b31` | `b047bb73…`；观察及四项相邻 exact 各 1P/0F/0I |

这些是各自源码切点的局部质量/运行及独立 Final 资格，不能跨切点累计。CLI 任务证据根为 `echo-agent-cli/.echo-semantic/local/weixin-app-recovery-20261107-r01/`，原件分别见 `native-r08-complete-related-green.json`、`final-review-r08.json`、`native-r09-long-and-adjacent-green.json`、`final-review-r09-long.json`、`receipt-observation-r01/actual-green-r13.json` 及 `receipt-observation-r01/final-review-r13.json`。R06/R07/R12 原失败和 raw/derived 字段层次继续保留。

同源手机材料 `phone-acceptance-preparation-r01/` 已获离线准备 Final C0/I0/M0；`preparation.json` SHA256 为 `84d35cb34be223b7d0cbffab8b2d8562c6420fc95ed83b74c6aa9c9783799fd7`。它不是真实 `app.log`、签名组合包或手机收件资格。

新 GUI 密度候选取已提交 CLI `a1ea657971f3a7190948d292f0eaa7bcc20e0bd9` 与 Framework `2b296b7e3bbc92dc73b416488c6dd9862352c113`，仅叠加明确已接受的 CLI 锁 `48eece730daaa9eff539e416861454441c23c54abf02bacaa2c3bdb375bd6329`。其原件为主根 `.echo-semantic/local/chat-density-20261108-r01/native-a1ea-package-r01/attempt-r01/ready.json`（SHA256 `86f01cac4707d1ea94b8d281d1613842dd166597676fd4b2e1a64b8c8a42f174`），签名 Mach-O 为 `c9d1b2fb3397cf08992ce372014689714b6b9ed34289843e737e74776be8c4cb`。此包仅作 GUI 密度交付，未并入未提交 SRE，不继承本计划最终 source-risk/gates、真实日志或手机验收资格；本次归档不自动启动 GUI。

完整 Plan `plan:ff675784-0933-485c-8d05-03413ce26f4c` 仍为 `ready`。最终组合源码/风险绑定、适用门禁、真实 App 日志和原账号手机验收仍待；本次只归档文档，不改 Todo、断言或历史失败。

## 历史批次概览（保留当时状态）

下文至文末按各批次的原取证时点保留；其中“当前”“本轮”“尚未”“待验证”等均为该批次当时态，不覆盖上方归档交付指针，也不构成重新执行旧批次的指令。

本记录对应原计划 `docs/supreme/plans/2026-11-07T1550-微信渠道恢复/plan_01_恢复原会话收发.md`，
artifact `plan:ff675784-0933-485c-8d05-03413ce26f4c`。目标仍是恢复 Eclip 原微信会话在停止、重连、
历史输入待恢复场景下的可靠收发；计划保持 `ready`，完整 Outcome 尚未通过。

本切点为 2026-11-08 的统一修正批次：Framework 渠道切片已经局部验证并本地提交；应用层已真实编译，
首轮 9 项定向测试为 7 通过、2 失败。两项失败及后继 backtrace 诊断原件全部保留。
准确 identity 的统计修正已在后继程序中转绿；夹具模型配置前提已修正，并实际观察到 routing/answer/Critic 的完整请求。
历史工作空间高层放弃误用传输默认 scope 的 Important 缺口已取得准确运行 RED；后继又定位并修正正常关闭 Pending 被永久记为失败的观察分类问题。
最新 r08 共享程序上，原管理完整收发、B/A 历史断言和最终 clean、冷重开与关闭正负控均已通过；五个相关模块合计 293 个唯一测试、零失败和零忽略，前八项 exact 已包含其中。
R09 长回复发送中 Stop/Reconnect 与四项相邻控制已实际通过，独立 Final pass；不跨快照累计为 298 项。
后端与 IPC 的 15 路径功能组已由中央本地提交 `70ac06a`。手机验收另发现普通回复的 typed Accepted 缺少可关联日志，
其最小观察候选已独立静态通过；R12 已取得原 HTTP/Stop 后的观察计数 RED，R13 同一链路及四项相邻控制已实际 GREEN。
新结果证明普通回复的 typed Accepted 可以生成规定的脱敏事件；真实 App 日志 sink、最终组合签名包及手机收件仍待。
本聊天未构建本计划的最终组合签名 App，未操作 GUI、扫码或发送手机验收消息。其它会话的 R03/R11 包保持其历史切片资格。

## 历史取证来源与所有权

主开发目录为 `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent`，没有另建开发 worktree。
Framework 和 CLI 当前使用 `feature/Echoyue/ssh-session-reuse`；源码包含多会话未提交改动，不能只按 HEAD 重建。
本轮读回 Framework HEAD 为 `2b296b7e3bbc92dc73b416488c6dd9862352c113`，
CLI HEAD 为 `bd2a083ee6b24ab27864b8becdb3082f1dff3494`；这是当前本地提交观察，后继批次仍须重新绑定。
HEAD 只描述提交基线；各批次资格绑定对应 manifest 的完整工作树字节。

IM 仅写所属源码和记录；UI、缓存、执行与保存各有独立 writer。
中央协调会话拥有 Git index、提交和清理；本聊天不操作这些共享交付动作。
共享 App 测试程序由唯一编译负责人保全，消费者直接使用同一路径及 SHA，不重复复制大程序。

## 已确认的源码问题与实现边界

1. 原 `SessionHandler::close` 成功删除会话后缺少结束通知，复用的应用协调器仍可能保留旧 incarnation。
   修复复用 `settle_session_end`，仅在准确会话成功删除后同步通知 `Shutdown`；失败、取消和原物理结算义务仍保留。
2. 旧 `RecoveryRequired` 会继续阻塞同地址新输入，普通 GUI 移除入口又拒绝修改渠道所属项。
   应用增加明确的单条放弃操作，沿原 journal 的版本和 phase 检查追加准确 `Cancelled`，
   校验原账户、凭据代次、工作空间代次和活跃前台 owner，再唤醒原 pump。
   取消队列项不撤销已发生效果，不改原 Failed TaskRun、历史失败事实、`drained=false` 或发送 Unknown。
3. 原 transport 可能忽略 admission 返回错误。修复在原 poll owner 内保留有界、脱敏、按准确未确认 frontier 身份关联的观察，
   其它消息的确认不能清除原错误；失败观察不拥有重试、执行或结算权。
4. 渠道启动失败与完成通知恢复 pending 原先混为同一失败列表。现在分别保留 transport 启动结果和原通知恢复报告。

通用会话与 poll 机制位于 Framework；Eclip 的放弃策略、GUI、配置、原工作空间和 owner 接线留在应用层。
没有增加 inbox、store、SQLite、并行恢复状态机或消息执行器。
设计参考及适用性沿原计划记载的 Tokio graceful shutdown 和 python-telegram-bot stop 合同，
复用原取消、join、结束回调及文件事实，不把轮询 Ready 当作模型执行、发送 Accepted 或手机收件。

## 历史批次：实际验证与失败

### Framework 渠道切片

`echo-agent/.echo-semantic/local/weixin-admission-observation-20261107-r02/` 保留原始 RED、逐轮改动与最终 r04 原件。
最终 `channels-r04.log` 为 **171 通过、0 失败、0 忽略**，其中包含 9 条新增观察回归，不能将 171 与 9 相加。
同批原 all-target channels Clippy 自然 0、零 warning，126 个实际源码输入和 3 个构建输入前后不变。
独立审查通过后，中央本地提交 `1f35f0d2`；原 target 已按 reader=0 与保全回执清理。
这是 channels feature 的局部验证，不代表 default feature、完整仓库门禁、精确 main CI 或 App 验收。

### 应用编译

第一次共享 App NoRun 自然退出 **101**，完整诊断有 12 个错误，见主根：

`.echo-semantic/local/exec-save-reliability-20261107-r02/app-shared-norun.log`。

其中 8 处为测试的模块路径，2 处误将原 owner 的 wake getter 用于 slot；另外 2 处 E0275 共用
控制回复到工作空间初始化的深层 Future 展开路径。
修复使用现有公开类型路径、`cfg(test)` 的原 slot 只读观察，以及 private reply 的一个原位 `BoxFuture` 类型边界；
没有提高 recursion limit、增加 allow、改变公开 API、spawn 或结算行为。

第二轮共享 NoRun r03 实际自然 0、零 warning；保守冻结 1,764 项和完整库存 1,807 项均无漂移，
`formalBindingsChanged=false`。其完整来源回执为：

`.echo-semantic/local/exec-save-reliability-20261107-r02/app-shared-norun-r03-native.json`。

唯一保全程序 SHA256 为 `45454e9d67d3fb4edc3ddcb7ef05434f4fd55aefb9f1c00c4b37b4273494bb70`，
567,617,888 字节，feature 为 `channels`。原 101 失败仍是历史事实，未被后继 0 改写。

### 应用首轮定向运行

原程序的 6 个 filter 共选择 9 项测试；各命令独立设置绝对 `EKO_DATA_DIR`、`TMPDIR`、`TS_RS_EXPORT_DIR`，
loopback 使用 `NO_PROXY/no_proxy`。证据位于 CLI：

`.echo-semantic/local/weixin-app-recovery-20261107-r01/native-r03-first-results.json` 及同名前缀 raw/source 文件。

| 实际选择 | 结果 | 证明范围 |
|---|---|---|
| 原低层准确放弃 | 1 通过 | 版本、原顺序取消、下一项与迟到 uncertainty 合同 |
| 高层放弃 | 4 通过 | 幂等、活跃 owner 拒绝、旧版本/凭据/工作空间代次拒绝、丢弃调用者后原 IO 和 wake 保有 |
| 生产 handler 重开 | 1 通过 | 同协调器、真实结束回调后新 incarnation 的准入 |
| 启动诊断分层 | 1 通过 | 通知恢复 pending 不冒充 transport 启动失败 |
| 原 journal 三 API 竞争 | 0 通过、1 失败，101 | 原断言将整个会话流的 old/next claim 合计为 2，需修正统计目标 |
| 真实管理入口组合 | 0 通过、1 失败，101 | 原子进程自然失败于 `completed_network_input`，根因仍待更细诊断 |

两个 filter 失败不否定其它准确局部观察，也不能被其它通过项覆盖。
这些测试还没有证明整个 IM 链路、最终完整源码快照或手机收件。

### 已证实的测试统计问题和未证实的超时原因

`ChatEventLog::replay` 在提供 conversation 时，stream 身份是 `(workspace, conversation)`，不会按第三个 root/input 参数过滤事件。
原 mixed-data/control 测试也明确核验不同 root 共用同一物理权威。
因此竞争测试旧 `(Cancelled=1, Started=2, Drained=0)` 包含后续输入的第一次 claim，不能据此声称旧输入执行两次。
后继修正按 `ConversationInputFact::identity()` 完整身份计数，仍要求旧 `(1,1,0)`、下一输入 Started 恰一次，
并保留原 raw 前缀和不可复活断言。A2 中同作用域的每输入统计也同步修正。

管理入口失败另做了一次只读 backtrace 诊断，使用同程序、原超时和全部断言，
仅增加 `RUST_BACKTRACE=full`、`RUST_LIB_BACKTRACE=1`，子进程实际输出 backtrace，证明环境继承生效。
该次仍自然 101，child 自然 101、`timed_out=false`、无 kill error。
栈指向 `completed_network_input`，尚不能确定是哪条输入或哪个等待条件没有成立。
父进程的无关 HTTP fixture 为 0，不能拿它冒充内层真实管理 fixture 的计数。

另用同一旧 r03 保全程序，对原有冷重开用例做诊断对照，exact 1 通过、0 失败、0 忽略、自然 0。
原恢复 child 实际路由/回答/验证各一次，模型合计 3 次、loopback 原 HTTP 接受业务回复 1 次；
已完成后再重开 child 的模型与业务回复均为 0，所有 child 自然 0、协议错误为 0。
证据为 CLI `native-r03-cold-reference.log` 和 `cold-r03-reference-observation.json`。
该对照说明旧程序的基础冷恢复路径可工作，不能推出管理入口等待的具体失败原因，也不是后继新源码资格。

后继两份测试只增加错误路径的有界阶段、真实模型请求类别/数量、receipt/TaskRun 状态和内层 HTTP 观察，
不输出 prompt/token、不改变四次业务回复、12 次模型调用、claim 恰一次或预算要求。
独立静态审查通过，格式/diff 检查自然 0，当前 7 源冻结见 CLI：

`.echo-semantic/local/weixin-app-recovery-20261107-r01/app-freeze-oracle-r05.json`。

该后继尚待共同 Clippy、NoRun 和实际运行；静态审查不构成 A2 超时修复证明。

### 后继共同质量门禁

共同源码 r04 的原 App Clippy 已自然退出 101，所有来源前后无漂移，原绑定未变。
完整诊断分别落在执行限额测试 getter 的 lib dead-code、UI 的引用比较与测试辅助参数、
IM 关闭回调测试的两处复杂参数类型；后继示例和 NoRun 因该门禁失败尚未到达。
原件为主根 `exec-save-reliability-20261107-r02/app-shared-clippy-r04.log` 及同批 after 回执。

IM 在明确 reader=0 的限定窗口内，仅为原 `channel_runtime.rs` 测试模块增加一个等价私有类型别名，
两处辅助函数参数改用该别名，注册、返回、索引、poison 和原断言均保持，生产前缀逐字不变。
独立窄审查、格式及 diff 检查通过，原七个来源无漂移；后继共八源清单为 CLI
`.echo-semantic/local/weixin-app-recovery-20261107-r01/app-freeze-lint-r06.json`。
这个后继仍须通过同一原质量门禁及真实程序重测，不能据别名等价性宣称整批 Clippy 已通过。

该别名及其它 owner 的限定修正共同封为 source r05 后，原
`cargo clippy -p echo-agent-app-core --lib --tests --features channels --locked -- -D warnings`
已实际自然 0、零 warning。`app-shared-clippy-r05.log`、原 `run.json` 命令回执及
`app-shared-clippy-r05-after.json` 显示 1,764 项保守冻结输入无漂移、完整库存无漂移、
`formalBindingsChanged=false`。原 r04 的 101 保留；此新结果只证明 r05 的该质量门禁通过，
随后原公开 App-core 示例检查也已自然 0，
`app-public-example-r05.log`／`app-public-example-r05-after.json` 保留编译成功、两类来源无漂移及生成文件未改的证据；
没有运行该示例。新的原 NoRun 已自然 0、零 warning，完整来源及生成绑定前后未变，
唯一保全程序 SHA256 为 `649ebd30dc65fd5f20557aebe9c6d2523f1c0ece964464f5a8173f4eaa8211f7`，568,258,896 字节。
原件见 `app-shared-norun-r05-native.json`；消费者没有重复复制该程序。

### 后继运行与配置前提定位

使用这个新程序优先复验原两个失败：竞争测试 exact 1 通过、自然 0；管理组合测试 exact 0 通过、1 失败、自然 101。
原件为 CLI `native-r05-journal-race.log`、`native-r05-managed-control.log`、各自实际选名、源码和环境回执。
管理 child 自然 101、`timed_out=false`；错误路径诊断如下：

- 等待阶段为 `second-input`，两条输入均为 attempt 1、`drained=true`、TurnSettled Completed，对应 TaskRun Completed 且不活跃。
- 实际模型请求只有 routing、answer、routing、answer 共 4 次，没有 Critic；第二次 answer 消费了预置脚本中的路由 JSON。
- 内层真实 HTTP 接受 2 次发送，其中业务正文只有 1 次，第二条发送分类为 routing JSON；等待第二条业务回复因此到期。

沿原生产路径核对的前提差异是：`application()` 的 seed pool 由 `AgentPool::new_for_test` 带测试模型创建，
但 `AppState::from_shared` 得到 `EkoConfig::default()`，其 configured models/providers 为空。
原冷恢复对照不调用渠道配置保存，pool 保留原测试模型；管理用例调用真实 `update_channels_config_owned`，
它按原完整配置发布路径把空配置同步给 pool。
随后 factory 的 `runtime_model/prepared_llm` 为 None，不安装原条件 Critic；
原 Mock override 只替换已经存在的 owned Critic，并不为未配置模型创建 Critic。
因此这里的三响应脚本与实际两请求路径发生错位，不是已证实的消息重发或 driver 丢失。

候选只向 AppState 传入与既有 `new_for_test` 逐字段相同的测试模型配置，
保持原 pool 最大 4 个 agent、Mock transport、factory 条件 Critic、12 次调用、4 次业务回复和所有 claim/历史/预算断言。
没有修改产品逻辑、手动增加 Critic 或降低调用数。
单文件候选 `model-fixture-r07.patch` 已获独立窄静态通过，随后在 reader=0 的明确单文件窗口内落地，
A2 源码为 `4aa1411d8d5ab755f758a4506cd2456f1cd0eb7ea87bc9cc6352ae351121d675`，其它七个来源逐字未变。
本次单文件格式及 diff 检查自然 0，新八源清单为 CLI `app-freeze-model-r07.json`。
后继仍必须用统一新程序验证管理用例和原冷恢复对照，才能把候选记为修复通过。

### 历史工作空间的高层放弃缺口

独立边界复核另确认 1 项 Important：恢复入口会按同账户/凭据代次扫描历史输入，
saved DATA 和原 pump/claim 均按输入保存的工作空间 B/generation 处理，
GUI 的明确放弃动作也透传 B 的完整 identity/revision。
但当前 `abandon_recovery_input` 先把 wake 的 transport 默认工作空间 A/generation 与 B 比较，
因默认绑定不同而拒绝合法的 B 项。原计划没有要求用户必须手动回 B 重连才可处置历史项。
该缺口不是“历史 B 永久不可处置”，但会使原保存绑定与高层动作的权威不一致，仍须修复。

两阶段候选已分别准备；test-only 回归随后在明确窗口内落地，正式管理实现尚未修改：

1. test-only 回归沿原公开管理器、同 peer/pump 及已真实执行过的 B scope，
   构造原 B Failed(false)/RecoveryUncertain，并验证活跃 B owner、旧代次及缺失 scope 仍拒绝；
   合法 B 的高层放弃应成功，保持 B/A 原 raw 前缀、Failed TaskRun、默认 A、四次业务回复及 12 次模型调用。
   只有真实执行到合法 B 调用并因原默认 A 比较失败，才算该缺口的运行 RED；静态准备或其它前置失败不算。
2. 取得该 RED 后，最小适配复用原 `channel_control_for_generation(B原地址, B原generation)` 和 B 的 managed admission；
   publication、原 control 及 operation 沿同一个 owned IO 保有到取消和原 pump 唤醒结束。
   wake 仍证明原 transport、账户和 slot，不改变 GUI 焦点，不增加状态权威或 claim。
   仅移除已无消费者的 wake 默认 generation 副本，fresh ingress 的 handler generation 字段完整保留；ADR 同步澄清原 scope 合同。

精确四文件/阶段及哈希见 CLI `historical-workspace-r08-scope.json`，test-only 与 production 补丁分别保留。
两部分独立静态审查无新增阻断，`git apply --check` 与最终格式检查均自然 0。
正式 A2 测试现为 `8537429f7ff45240661e672992157e14918ec2bc721dde04986a6d5afdd65042`，
其它七个来源未变，新八源冻结清单为 CLI `app-freeze-historical-red-r08.json`；
production management/runtime/ADR 分别仍是 cfd/260/75f 前像，生产补丁没有采用。
随后 r06-red 共享 Clippy 与 NoRun 均实际自然 0、零 warning，来源及生成绑定无漂移。
唯一保全程序 `8033bbdab2b389a03b6c0c0571982f97b14de22460b8273e755b174cc2dd330d`、568,359,584 字节，
绑定原 `88275cd5…` source manifest；遗留的旧 shutdown selector 字段没有用于选测，当前范围由独立 clarification 和明确 GO 固定。
实际 `--list` 确认正确 exact 名称后，先跑冷恢复控制，得到 1 通过、自然 0；再跑管理回归，得到 0 通过、1 失败、父/child 自然 101、`timed_out=false`。

该次准确 RED 的实际阶段是 `historical-workspace-b-abandon`，错误是
`original historical workspace-B abandonment: 该输入属于另一工作空间；未执行放弃`。
此前 first、second、B 的原管理/pump 路径实际完成，内层 HTTP 接受 3 条业务回复、协议错误为 0；
模型请求 routing/answer/Critic 各 3 次，共 9 次，证明原模型配置前提已经正常。
B 原历史仍是 attempt 1、`drained=false`、Failed/RecoveryRequired，A 原 Failed/Unknown 保留。
这证明了目标 guard 错误，不能把失败测试称为产品成功。原件为 CLI `native-r06-*`、解码 child stderr 和
`historical-workspace-r06-actual-red.json`，source 八项后核无漂移，所有原 reader 已自然退出。

中央核实上述 RED 和 reader=0 后开放三文件 GREEN 窗，精确采用已审薄适配、私有 wake 元数据清理及 ADR 说明：
management 为 `76f275ad…`，runtime 为 `efa2930c…`，ADR 为 `f64a042f…`，测试 `8537429f…` 及另外四项来源完全未改。
限定格式/diff 检查自然 0，八源新清单是 CLI `app-freeze-historical-green-r09.json`。
生产代码已经修改，仍待新共同程序验证四条业务回复、12 次模型调用、B/A 原记录和全部取消负控；
Important 缺口尚未因静态结论或代码采用而关闭。

### 管理链路推进与最终清理失败

共同 r07-im-green 的原 Clippy 和 NoRun 均自然 0、零 warning，1,764 项冻结来源和 1,807 项完整库存无漂移，生成绑定未变。
实际源清单 `app-shared-source-ready-r07-im-green.json` 的 SHA256 为 `c24ffb12475910979fb4ef8d2645fac3c7c9732baedbbe8feb5ae2616df98251`；
唯一保全程序 SHA256 为 `f19945ddd3e781cb68686825d600ba18b30e44c84d2ccb956e76594fa1d8505a`，568,336,912 字节。
其资格原件为主根 `exec-save-reliability-20261107-r02/app-shared-norun-r07-im-green-native.json`。

同程序冷重开控制 exact 1 通过、自然 0；管理测试仍为 0 通过、1 失败，父与 child 自然 101，`timed_out=false`。
管理测试的原业务 body 返回成功后，最终 `wait_final().is_clean()` 失败：回执保留了
`Original channel cleanup is still pending` 和 `original execution ownership is not physically settled; resources retained`。
所以可确认它已越过 B 原 scope 放弃、四条业务回复、12 次模型调用及原失败历史断言，但整项仍失败，不能记作完整 GREEN。
父层 fixture 的零计数也不能替代内层业务观察。原件为 CLI `native-r07-*`、`managed-r07-child-stderr.txt` 和
`historical-workspace-r07-green-attempt.json`；全部原 reader/child 已退出，八项源后核无漂移。

### 正常 Pending 与真实失败的最小分类修复

沿真实 producer/consumer 查明：原 cleanup watch 开放且还没有回执时，`observe_shutdown` 返回错误；
根把该暂态及通用“仍保留资源”说明追加到永久失败列表。随后同一原 producer 真实完成并发布物理终局，
`wait_final` 为保留历史真实错误而合并先前失败，使正常退出也永远不 clean。

此次只改既有私有观察合同：`Ok(None)` 表示原开放 watch 尚在途，根继续保留 owner/依赖及原 continuation；
`Ok(Some(...))` 表示原物理回执及其真实失败；`Err` 保留真实 stop 错误、无回执关闭及未确认结果。
合法 retry 即使刚开始，首次观察仍返回原真实 stop 错误；没有按最终错误文本清空失败。
读取 watch 的值与 closed 状态使用同一同步读 guard，避免把旧 None 与后来的关闭状态混合。
公共 `is_clean`、`wait_final`、receipt 形状、owner/waker、join 顺序、预算和 B scope 放弃函数均未改变。

新增正常 Pending 最终 clean 正控、retry 保留真实错误控制，并加强既有 stop error 终局断言；
原 primary error、unwind、缺失/关闭终局观察等负控继续保留。
参考依据是当前锁定 Tokio 1.53.3 和 Rust 1.99.1 本机官方源码的 Pending/Ready 及 watch 合同，
原路径与哈希见 `lifecycle-pending-r10-reference.json`；网页工具错误未记成成功调研。
两个源码文件和既有 `docs/zh/architecture/runtime.md` 在明确写窗内采用，独立审查 Critical/Important/Minor 均为 0，
正式 rustfmt check 和 diff check 自然 0。新冻结为 `app-freeze-lifecycle-green-r10.json`，
SHA256 `9a1c50a3b304480969dfcccb347a8f641c74fab8b4b376ad382170ca2870cc3d`，机械合计 8 Rust 加 2 文档。
当前源码已停止写入并交回唯一共同编译批次；尚未取得这次分类修复的动态 GREEN。

### r08 同一程序的完整相关回归

新共同切点 `app-shared-source-ready-r08-mixed.json` 的 SHA256 为
`7656db4208fbeee224331b9bd5964201e06c9d9e1fd3ee18ee0c66105f6f05dd`，准确绑定本任务 r10 十项来源。
原 Clippy 和 NoRun 均自然 0、零 warning，1,764 项保守冻结输入及完整库存无漂移，生成绑定未变。
唯一保全程序 SHA256 为 `1bca2d14e3081b72b422e630adb772027fc1c357f9c567770c6e315bc4246321`，
568,598,496 字节；原回执为主根 `exec-save-reliability-20261107-r02/app-shared-norun-r08-mixed-native.json`。
消费者直接使用同一只读路径，没有重复复制或另起 Cargo。

先运行的八项 exact 全部通过、自然 0：原冷恢复、完整管理组合、正常 Pending 最终 clean、retry 保留真实错误，
以及 primary error、真实 stop error、unwind 和缺失/关闭终局证据四项负控。
管理 child 实际自然 0、`timed_out=false`，内层回执是 12 次模型调用、4 次原 HTTP 业务回复和零协议错误；
这次已越过最终 `is_clean`，不再仅是业务 body 成功。
冷恢复实际路由/回答/验证各一次，已完成后再重开模型和业务发送均为 0。
原 r07 的 101 保持原件与原结论，没有改写成通过。

随后以同一程序按真实 `--list` 运行五个完整计划模块：

| 模块 | 当前实际结果 |
|---|---|
| `channel_runtime::` | 180 通过、0 失败、0 忽略 |
| `conversation_input::tests::` | 26 通过、0 失败、0 忽略 |
| `chat_event_log::tests::` | 46 通过、0 失败、0 忽略 |
| `tasks::task_runtime::weixin_delivery::tests::` | 9 通过、0 失败、0 忽略 |
| `runtime::tests::` 原主模块 | 32 通过、0 失败、0 忽略 |

实际名称集合去重为 293，八项 exact 均包含在内，不能累计成 301。
libtest 的 substring 会把 `runtime::tests::` 同时匹配到其它命名模块；只排除实际列表中其它模块的完整名称，
主模块 32 项完整执行，没有禁用模块内测试或新增 ignore。
每个命令均自然 0、无 signal/error/truncation，本任务十项来源逐组后核无漂移；组内 13 份 A2 child 回执均为 0、无 timeout。
本聊天所有原 parent/child 已结束并向中央及唯一编译负责人明确交回 reader=0。
原日志、选择清单、环境与来源在 CLI 本任务 `native-r08-*`，汇总为 `native-r08-complete-related-green.json`；
内层管理/冷恢复解码观察为 `historical-workspace-r08-actual-green.json`。
这些结果证明当前 App-core 相关回归通过，长回复新候选、Tauri/签名 App 和手机层仍分别待验。
本轮独立 Final Review 已读回原始命令/日志、真实选名、child 输出和程序 hash/fingerprint/来源绑定，
对“原 B scope 适配与 Pending 观察分类”切片给出 pass，Critical/Important/Minor 均为 0。
该 Final 不关闭完整 Plan 的长回复覆盖 Important，也不代替根 GUI、签名或手机验收。

### 完整 Outcome 预审的覆盖保留

独立预审核对当前十项来源全部匹配，其余主链静态未见新实现断裂，但完整 Outcome 仍为阻断：
Critical 0、Important 1、Minor 0。Important 是原计划明确要求的“长回复发送中停止及重连”组合覆盖不足。
当前管理用例是短回复，Stop 均发生在对应 TaskRun 完成后；Framework 长输出、App 取消及 fence 原语分别通过，
也不能替代同一 public manager 加真实 App deferred reply 的进行中发送场景。

按原计划范围，仅在 CLI 本任务 ignored 目录 `long-reply-r11/` 准备单一原 A2 文件的测试候选：
真实长回复沿原 UTF-8 chunker 到本机 HTTP，收到首个请求但尚未回复 ret0 时调用 public Stop，
观察原 token 取消和原发送仍未物理结算；随后开放同一 HTTP 回执并等待原 Stop、callback 和 pump 完成，
再以同协调器重连、新输入回执，以及真实旧 instance 模板经新 accepting sender 的 typed StaleDelivery 拒绝验证隔离。
已发起的原请求按真实回执保留，不把迟到结果伪装成撤回；候选不增加生产 owner、协议字段或预算。
候选最终独立审查 Critical/Important/Minor 均为 0，格式/dry-apply 检查自然 0；
收紧旧前缀 Accepted 恰 1、总 HTTP/Accepted 恰 2 后，测试定义已准备好，仍待正式采用和真实运行。
候选与补丁 SHA256 分别为 `2fc776266d8f0a00213e953e35ba7a3364cc52b4307d13e1ef7b936af00bf0f4`、
`942c83ea43306aef18240fe04d24c6a2e78b8dbe6cdaf502deb56b56a0357b2f`。
Root 条件程序及其读者结束后，中央通过 `coordination-20261107-r01/r09-ui-im-source-window-go.json`
开放唯一 A2 文件窗口；本聊天实核前像 `8537429f…` 后精确采用已审 `2fc77626…`，其余九项来源逐字保持。
scoped rustfmt write/check 和 diff check 均自然 0，格式写后仍是已审字节；没有修改生产代码、旧 case 或预算。
采用回执为 CLI 本任务 `long-reply-r11/formal-adoption-r09.json`，新冻结为 `app-freeze-long-reply-r11.json`，
SHA256 `c9cc32d5a525587e08eddaef0259bd6b95c64246ddc59a5de0adf0bd3aed66e2`。
原有 A2 路径被替换，清单仍是 10 项（8 Rust、2 文档），不是新增第 11 路径。
源码已停写并交回中央 R09 共同批次；新测试仍为 `not_reached`，旧 R08 的 293 项只保留其原切点资格。

### R09 长回复实际结果与独立 Final

共同源码清单 `app-shared-source-ready-r09-green-candidate.json` 的 SHA256 为
`2b63cd1fe60232e0b7e86b2a95ed4847c4d4a09f8d160e00326c63e6317495c5`，绑定上述 c9cc 十源。
新保全程序 `7bbeda7f6279cb439f93307c85caeb697dd29594c8f32c045020a2f040138097`、569,110,560 字节，
原 Clippy/NoRun 自然 0、来源及生成绑定无漂移。
按授权只执行长回复、管理完整收发、冷恢复、正常 Pending 及真实 stop error 五项 exact，每项 1 通过、零失败、零忽略、自然 0。
五个原 child 全部自然 0、无超时或 kill，十源后核无漂移，所有本聊天 reader 已退出并回交。

长回复实际计划 4 个原 UTF-8 chunk、6 次模型调用，原 HTTP gate 开放前 Stop 保持等待；
同一原发送回执返回后，经原 Stop 物理 join 和结束 callback，再由同协调器建立新 incarnation。
HTTP 总数恰 2（旧首段和新回复），真实旧模板交给新 accepting sender 得到 typed `StaleDelivery`，HTTP 未增长。
管理回归同时保持 12 次模型调用、4 条业务回复和最终 clean。
原件、选名、源码/环境及 child 解码回执在 CLI 本任务 `native-r09-*`、`native-r09-long-and-adjacent-green.json`。
独立 Final 读回源码、原日志、哈希及来源绑定后给出 pass，Critical/Important/Minor 为 0，
`final-review-r09-long.json` 明确此前长回复覆盖 Important 在 App-core loopback 层闭合。
R08 的 293 项没有重跑或跨快照相加；最终签名 App 和手机层继续分别待验。

### 后端提交边界与 R12 Accepted 观察测试

中央只读审查并私有重建本任务 15 路径后端/IPC 子集后，本地提交
`70ac06a52479ea59b6bd9f67df1173e891b80b24`，tree `7360355af682ffcbe613158823e418aee1476dc9`。
该提交包含必要的 journal 新取消方法与 projection 的 Cancelled 终态保护；后者及 Tauri 注册仅取 IM hunk，
usage/presentation 剩余改动留给 UI。四份共享 TS 消费者另外交付，不把 backend 注册成功当成按钮交付。
源码 2,483 项与真实 index 的前后检查见中央 `cli-im-backend-commit.json`；这是本地功能分组，未推送/合并，也没有 subset-only 编译资格。

为满足原计划按消息逐层记录的手机验收义务，另查明普通回复的 `Accepted` arm 为空，Framework 仅记录发送失败，
现有 completion/cron ledger 属于另一条链路。此为观察证据缺口，不是已证明发送失败。
三路径候选仅在原 typed Accepted 返回后向既有 App 日志记录回执 ID 的 SHA256、ordinal 和固定 accepted 结果，
显式 `parent: None` 防止 caller span 字段附带私有数据；不增加 store、调度、结算、重试、依赖或权限。
原 A2 HTTP gate 测试按 0→1→2 验证回执前后计数，并校验字段白名单、摘要、ordinal、root event 及捕获上限。
候选、测试与 ADR 的原件在 CLI 本任务 `receipt-observation-r01/`，独立静态 C0/I0/M0；未据此填运行成功。

中央 R12 精确窗口只允许 A2 test-only：`2fc77626…`→`8ef1e646eb4ee9ded522c5cafb673549ef409b40b7a196eb8700607d7108761a`，
test patch SHA256 `5225d94f3b93f314c86ad5788a7d7dc5c2cc1636bb6fa0ef96df0de1033fdffc`。
正式采用后 scoped fmt write/check 与 diff check 自然 0，另外九项、生产 logger、ADR、HEAD 及 index 均未变。
重新读取最新 SRE r02 及 base 的 25 路径，无 IM 三路径重叠；写入只限本次 A2，未执行 Cargo/GUI/手机。
新十源 `app-freeze-accepted-test-only-r12.json` 的 SHA256 为
`dc12e8acdc25708ea365644c0db9e6b2d22842da34436afbce46a34f06328b16`。
源码已停止写入，待中央原共享 warm target 的唯一 App 程序运行长回复 exact。
只有真实越过原 HTTP ACK 和物理 Stop 后，在 `accepted-after-original-http` 因观察 0、预期 1 失败，才是目标 RED；当前未到达运行终局。

R12 后继原 App Clippy 和 NoRun 均自然 0、零 warning，来源/库存及外层输入无漂移、生成绑定未变。
新程序 SHA256 `2c97843bba9b76668a64dcd2e6ee501412333ca5db8adb5b6eba3a50b695a5ea`，569,220,048 字节，
源清单 `app-shared-source-ready-r12-accepted-red.json` SHA256 `d860fbe6e6d07ae3487d070370635f3ab945b184d03001e73ba6aa7de4b2591a`。
按中央唯一 exact GO，先实际 `--list` 选 1，再运行一次原长回复用例；测试 0 通过、1 失败、自然 101，
原 child 自然 101、`timed_out=false`、无 kill，所有原 parent/child 已退出并回交 reader=0，十源后核无漂移。

原 child 的内层实际诊断为 `accepted-after-original-http`、provider `accepted=1`／`send_requests=1`、
协议错误 0、held request 已见、模型调用 3 和计划 chunk 4；错误恰为 `actual accepted observation count mismatch`。
到达此检查前，真实代码已依次 await 原 Stop、核验结束 callback 的 wake 消失、等待原 pump idle，
并通过原 management `shutdown_settled`，故这次不是编译、subscriber 初始化、夹具或外层超时失败。
内层实际 HTTP 接纳 1 与外层无关 provider 的 0 分别保留；不能拿父层零计数替代原发送观察。

原错误未打印 capture 的具体 len：证据保留 rawPrintedActualCount 为 null。
由隔离 collector 初始为空、唯一 target 在冻结生产代码没有 emitter、初始零检查通过，以及该 expected 1 检查失败，
可来源绑定推导 capture 为 0；该推导未伪装成原始诊断数字。
完整 raw、命令/选名/源身份、解码 child 输出及分类见 CLI 本任务 `native-r12-*`、
`accepted-r12-child-{stdout,stderr}.txt` 和 `receipt-observation-r01/actual-red-r12.json`。
这是发送已接纳后的观察缺口 RED，测试仍是失败状态；尚未证明生产 logger 已运行或进入真实 app.log。

R12 target RED 随后由独立 reviewer 读回原程序/大小、冻结来源、child/raw 和实际前置顺序，
给出成立、Critical/Important/Minor 为 0，记录在 `receipt-observation-r01/review-actual-red-r12.json`。
审查继续区分 expected 1、源码推导 0 与 raw 未打印数字，不把这次结果解释成发送失败。

中央 R13 只开放 logger 和 ADR 两路径；采用此前独审的生产补丁
`c323dff92fdbf381bbbca0ecb2598d1ec8d947f2852c0513bbc4c87bbd4afea8`，不重复贴含 A2 的三路径完整补丁。
`channel_runtime.rs` 精确变为 `1be91c6fbc9cbe53a17ecd37457411faad529b7a1325baf500e6ac1a22b4c167`，
ADR 0056 精确变为 `c400621679032a3d19c934256238d571e669002c958fa45e94ac9c5430bcad98`。
A2 保持 `8ef1e646…`，其余八项、HEAD 和 index 均未变；最新 SRE r02/base 25 路径重新比对无重叠。
scoped fmt write/check 和 diff check 自然 0，生产事件只有原实际 Accepted 返回后的摘要、ordinal、固定结果及显式 root parent。
新冻结 `app-freeze-accepted-production-r13.json` SHA256 为
`064aac6c33d69acc107f38d7176c22da8a346c01d68521d0c6107898817c38a3`，采用回执为 `receipt-observation-r01/formal-production-adoption-r13.json`。
本聊天已停止写入并交回唯一共享编译批次；当前不能据源码采用或原 R09 GREEN 声称新观察已运行通过。

R13 共同源清单 `app-shared-source-ready-r13-accepted-green.json` 的 SHA256 为
`dba4fa79d0c437a4a7b6ec7c33335ea2a36ca5114390454cf6e1f95e33bb1b31`，准确绑定上述 064aac 十源。
原 Clippy 自然 0、零 warning，NoRun 自然 0，来源/库存/外层保护检查无漂移、生成绑定未变。
唯一程序 SHA256 为 `b047bb7306c9a639afc3ea94719004835aaeba512459ba3f0ca1f74f1c4ac3c8`，569,240,864 字节。
同一新程序按原 GO 依次运行 long、managed、cold、pending、stoperror 五项 exact，各 1 通过、0 失败、0 忽略、自然 0。
五个原 child 全部自然 0、无超时或 kill；全部 parent/child 已退出并回交 reader=0，十源后核无漂移。

long 原 child 回执直接记录 Accepted 观察数 2、字段脱敏检查 true，以及计划 chunk 4、模型调用 6、HTTP 2、
新业务回复 1、协议错误 0、原 Stop 物理 join true、旧 fence 未到 HTTP。
原源码强断言依次核验回执前 0、Stop 和原 ACK 后 1、新回复后 2，逐条与真实 HTTP client ID 的摘要/ordinal 对照，
并检查固定四字段、固定消息、`Event.is_root`；单条哈希和原始 ID 未额外 dump 到回执，不能说它们是 raw 直接字段。
managed 原 inner 回执继续保持 12 次模型调用、4 条业务回复及最终 clean；其余冷恢复、正常 Pending 和真实 stop error 均通过。
R12 的自然 101、来源及 derived 0/raw 未打印数字区别完整保留，未将历史失败改成成功。
新 raw/命令/选名/来源保存在 CLI 本任务 `native-r13-*`，完整观察为 `receipt-observation-r01/actual-green-r13.json`。
本轮不重复或跨快照累加原 293 项。上述是 App-core 真回环链路的事件捕获证明，尚非真实产品 `app.log` 落盘或手机收件证明。
独立 Final 已核验原始记录、来源绑定、五个 child 及原强断言，给出 pass，Critical/Important/Minor 为 0；
`receipt-observation-r01/final-review-r13.json` 同时确认不将此结果扩展成真实 App 日志或手机证明。
三路径交接材料在 `receipt-observation-r01/commit-handoff-r13/`，补丁 SHA256 为
`5b47cc21b20364170c19918a1739392cb4b512e9be763cd46c832a17d5699230`，以 CLI `3c13b2ff…` 为基线，
三个完整当前差量无 UI/SRE 混入或新依赖，private HEAD patch check 为 0，正式 HEAD/index/来源保持。
该交接也获独立 Final pass；没有另做 HEAD＋subset-only 编译，不冒充独立提交树资格，中央再按原授权处理本地提交。

中央随后精确本地提交 `bd2a083ee6b24ab27864b8becdb3082f1dff3494`，仅 A2/logger/ADR 三 whole paths，
来源 2,487 项与十源工作字节不变，真实 index 为空；回执为中央 `cli-im-accepted-observer-commit.json`。
提交仍限定 R12 RED/R13 五 exact 及独立 Final 的源码切片资格，不签真实 App 日志、手机、SRE、风险门禁或 main CI。

### 最终同源手机验收准备

按中央后继安排，本聊天在 CLI 自有 ignored 目录 `phone-acceptance-preparation-r01/` 准备了取证层次与用户步骤，
独立准备 Final pass，Critical/Important/Minor 均为 0；`preparation.json` 与 `final-review.json` 保存范围和原件。
材料明确先绑定最终组合 source/适用风险门禁/真实构建、fresh 资产、签名和 entry/ready，再由用户原数据、原账号、原工作空间
明确放弃准确阻塞项，手机 A/B 两条新消息、停止重连后的 C、App 重开后的 D 分别取证。
包、authenticated Ready、原 DATA/claim/TaskRun、Accepted app.log 和手机收件各自记录，Unknown 不自动重发或改写。

最小离线 `evidence.py` 只计算准确 input ID/ordinal 的原 client 摘要，可选读取显式冻结导出的日志窗口，
输出匹配计数与最多 64 条偏移/行摘要，不输出原 ID、正文或凭据，不读取 journal/配置，不调用 IPC/网络。
两处准备工具 Minor（正文引用伪日志头、超长无换行尾部标志）修正后，六次最终合成检查自然 0；
原 formatter quoted/unquoted、完整伪头、消息前缀、超长 partial、70 总计/64 位置及隐私负控均过。
这仅是工具离线准备，不证明真实 App 日志或手机收件。

已审构包配方的 11 份文件哈希保持，未运行 prepare/build/finalize，未生成本计划 inputs/ready/entry/execution/build-qualification；
十源、HEAD bd2a083、空 index 均未变，没有操作 GUI/手机、读取真实 App 日志/业务历史/凭据或执行 Cargo。
UI 隔离 R03/R11 包保持旧 source 资格，不能继承本次 logger。实际构包资源窗口仍由中央在最终组合源和门禁准备好后给出。

## 历史验收快照与仍保留的整体义务

当前前端快照另已读回原完整命令日志及 `frontend-code-assets-freeze-r08.json`（SHA256 `c11804f370afab46bdb689f62dd8efcdb20301eee5be0a6a95414038b2668899`）：
`npm test` 的 lint 自然 0、单测 1,025 通过和 4 项既有跳过，格式及默认 Web 模式构建自然 0。
514 项前端来源、152 项资产在本次只读后核均无漂移；微信 Ready/error 测试保持精确 `e8b165bf…` 字节。
132 项定向测试和渠道页 15 项包含在上述完整结果内，不另相加。
本聊天未重复运行前端；读回回执在 CLI 本任务 `frontend-r08-readback.json`。
该结果覆盖当前前端消费者，尚未覆盖 Tauri 根编译、真实 Rust producer、签名 App 或手机收件。
后继正式前端 R11 的原完整命令已另读回：1,027 通过、4 项既有跳过，lint、格式及 Web 构建自然 0。
`frontend-code-assets-freeze-r11.json`（SHA256 `2c97b8bcfab207fe1dd8d54b53287e20326eecf8118191326a7c7e9976e70310`）
的 518 项源码、152 项资产无漂移，读回记录为 `frontend-r11-readback.json`；仍只说明该 Web 模式快照。
根 GUI 的 r08 Clippy 后继实际自然 101，只有 `src/tauri/commands/config.rs:189` 的 `collapsible_if`，
来源后核无漂移；根 GUI NoRun 和业务测试尚未到达。
原件为主根 `exec-save-reliability-20261107-r02/root-gui-clippy-r08.log` 及 after 回执，
该质量失败不计作图表目标 RED，也不改本轮 293 项 IM App-core 的原结果。
后继 `root-gui-source-ready-r08b-lint.json`（SHA256 `380a2a9192e4c68cc7ca923a751efd135a69b672ca957383bfe5bfa306bcc2b7`）
的原 Clippy 与 NoRun 已自然 0、来源与生成绑定无漂移；这仍是根条件程序的资格，尚不代表签名 App 或手机验收。

| 原 Todo | 当前事实与剩余义务 |
|---|---|
| 关闭通知 | Framework 局部、应用真实 handler 重开、完整管理终局及关闭正负控通过；R09 长回复发送中 Stop/Reconnect 及物理 join 已实际通过 |
| 准确放弃 | r08 完整相关组通过：高层幂等/活跃/旧代次拒绝、owned IO、原 journal 竞争、历史 B 原 scope 和后续消息恰一次；原失败及 Unknown 保留 |
| 准入诊断 | Framework 局部、启动报告分层和当前完整前端检查通过；Ready/error 共存及刷新覆盖已纳入，最终原 Rust/GUI 全链路仍待 |
| 生产控制闭环 | R08 五模块 293 唯一回归、R09 长回复/相邻五项、R13 Accepted 事件/相邻五项分别通过；原 R12 RED 保留，真实 App 日志和手机仍待 |
| 签名 App 与手机 | 本计划配方保持准备态；其它历史 UI 包不追认当前 IM 观察源码，最终组合包、门禁和手机收件仍待 |

签名候选配方位于主根 `.echo-semantic/local/weixin-recovery-App-20261107-r01/`，
复用本机已保留的构建、签名、资格、保全和手动启动原语。
已消除旧 SSH design 元数据继承，并将外层观察脚本纳入配方身份绑定；
尚未运行 prepare/build/finalize，没有伪造 inputs、ready 或签名结果。
最终还须绑定完整两仓源码与新前端资产、读真实构建出口、签名 strict verify、检查专用入口，
再由用户在原数据、原账号和原工作空间执行明确放弃、两条新消息、停止重连及 App 重开后的手机收发。

## 文档、示例与官网

Framework guide 15 和 ADR 0099 已随框架切片维护；应用放弃和通知分层合同在 CLI 中文 ADR 0056，仍是唯一正式产品来源。
既有 `demo38_im_channels` 是 QQ/飞书示例，已说明微信需要 durable ingress opt-in；本次没有新增媒体、平台或示例能力。
已读回中央原 `cargo check -p echo-agent-learning --example demo38_im_channels --no-default-features --features channels --locked --offline`
自然 0，原日志 `exec-save-reliability-20261107-r02/v2-im-example-check.log`、命令回执 `run.json` 和
`v2-im-example-check-after.json` 的 1,479 个来源前后无漂移。该证据只证明对应源码切片可编译，没有执行示例或真实登录。
官网由中央从 Framework 精确提交的干净视图走原同步入口，本地提交 `70884ba`；未推送或部署。
CLI ADR 的官网投影仍按站点现有合同处理，不另造第二份正式产品文档。

当前没有合并、精确 main CI、发布或线上验收声明。
