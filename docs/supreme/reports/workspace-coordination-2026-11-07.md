# 工作区协调整理（2026-11-07）

用户授权本会话协调 IM channel、缓存、执行终端等现有会话，按功能整理并提交已完成改动，及时清理旧产物。所有操作限定唯一主目录；本轮不推送、合并或发布。

## 写入责任与验证安排

| 工作 | 会话 | 当前责任与交付状态 |
| --- | --- | --- |
| IM channel | 排查 IM channel 故障（01a1b4ec） | 微信 channel、应用 channel_runtime/management/conversation_input、Tauri channels/desktop；恢复和准入观测仍有待修边界，WIP不提交 |
| 缓存 | 优化缓存前缀稳定性（01a1b57d-7911） | core compression/cache、state compression、provider adapters、react context、应用 turn_context/unified_memory/compact_context；已有真实RED，最终验证未完成 |
| 执行与保存 | 修复执行与保存可靠性（01a1b57d-ad25） | cell/tools/shell、command_cell、snapshot保存段、FileStore、应用command_cells/limits；已有真实RED，最终验证未完成 |
| UI及运行观察 | 排查缓存命中率和运行失败（01a1b53e） | 前端、chat journal/projection、react think计时等；完整writer回执已登记，仍未交付 |
| 协调整理 | 本会话（01a1b583） | Git index/commit、旧产物清理、按功能分组与交付核对；不代写各活动会话源码 |

共享文件按单一writer交接，验证期间冻结被读源码。UI已完成echo_core LlmUsage字段代的静态同步并冻结core；可靠性和IM既有reader结束后，缓存先完成wire摘要及Anthropic工具结果合并后的断点修复，再发布新冻结SHA。root think计时仍由UI写，root/app-core当前不能作为全包冻结验收。可靠性后续app-core同源测试程序可供缓存复用，避免重复重编译。完整矩阵串行，focused Rust遵守并发和jobs上限。

## 起始未提交改动分组

- 活动IM、缓存、执行/保存、UI：分别保留，未有最终交付回执前不stage。
- Framework媒体依赖/vendor与两仓精确风险绑定：独立历史功能组，不能借协调授权扩大风险接受或把旧gate视为当前通过。
- SDK退役边界：四份新增回归测试与对应中文说明可独立复核；SDK语义图、baseline及历史Evidence另组保留，避免混成一次提交。
- 顶层验收索引、旧交付文档、SSH/GUI报告及spec：属于跨仓阶段材料；保留其历史状态，本轮只提交独立完成且实际验证的材料。
- 官网当前工作树干净；不为清理或测试专门修改官网。

## 初始实测与恢复

主目录初始du约52.90GiB，df可用约35GiB（并行构建中，非原子测量）。源码HEAD/分支、空index、完整tracked binary diff已保存到主根ignored `.echo-semantic/local/coordination-20261107-r01/`。四子仓为独立Git仓库；不自动更新未推送子提交的顶层gitlink。

缓存清理仅选无reader且确认可重建的准确Cargo target；旧测试程序另外按精确退役清单处理。当前App、活动target、用户配置/认证/历史、历史恢复包、源码备份与原始失败日志保留。不会使用git clean/reset或批量删除未跟踪文件；嵌套目录只清其中已确认为缓存的cargo-target，保留其余未知来源文件。

## 本轮执行结果

- 已对4处旧Cargo target真实执行准确`cargo clean`，均自然0，删除后路径不存在。清理前目录分配合计20.214GiB；Cargo自行报告的逻辑删除量另保原日志，不把两种口径相加。唯一long-type诊断已复制并逐SHA核验。
- 第二批实际退役18个旧SSH测试程序，逻辑文件字节合计9.379GiB，先保留每个程序的SHA/模式/大小。旧批次最后g05程序及新提交批次程序、当前App均保留，g05前后SHA一致；全部源码及原日志不删。被删历史程序不再承诺直接重跑。
- 第二批后阶段df可用约64.49GiB；并行构建与APFS共享存在，不能把df变化或上述不同计量直接相加，当成独占物理回收量。
- SDK已独立提交`0025d84e1a5a08634c9596092c46ca37eed0304c`，仅四回归测试及中文说明，5文件168新增行；Rust定向1P、Python/TypeScript/Java各2P，TS build及fmt write/check均0，独立源码审查0 findings。其余SDK语义材料保留未提交；当前剩余完整diff与初始排除这5文件后的diff逐字节一致。
- 本次SDK批次结束后先保全8MB测试程序及SHA/模式，reader0后准确clean本任务sdk-target，出口0。该新建批次清理不计入上述旧缓存回收量。
- 原验收文档一致性13项、证据索引30条当前检查通过；不因此升级产品行为验收。
- Framework、CLI活动修复尚未回交冻结，保持未暂存；顶层gitlink不更新，SDK提交未推送。main CI、原生App、真实手机IM验收均未由本轮执行。

原始回执位于主根ignored `.echo-semantic/local/coordination-20261107-r01/`：`baseline.json`、`ownership.json`、`functional-inventory.json`、`cleanup-preflight.json`、`cleanup-results.json`、`cleanup-settlement.json`、`native-retirement.json`、`sdk-validation.json`和`sdk-commit.json`。这些是本轮协调与清理证据，不是产品完成声明。

## 第二阶段进展（2026-11-07）

- Framework 已独立提交媒体安全升级组 `eec388d1`：37 个 Git 变更路径，官方 Calamine 0.38.0、获准精确锁及规范、CFB 负控；本轮 CFB／XLSX／DOCX 共4P，定向 Clippy、16项 origin 控制、14项 baseline 控制与 fmt write/check 均0。新鲜官方 advisory 全扫描未在本切片执行；定向验证绑定完整工作区内容，不冒单独 commit-only、完整工程、SDK／App／main资格。
- Framework 已独立提交路径别名夹具修正 `2bd3f13d`，仅两处 expected 路径 canonicalize 与一条注释。同一旧 native 在主根 symlink TMP 中实际 RED101、真实 TMP 控制0；新 native 在两环境原单测各1P。产品 authority 与原 no-rekey／文件字节／历史恢复断言不变。
- 缓存、执行与保存、UI 三组已批准 design／Plan 原字节分别提交为 `a9df55a`、`806b8e1`、`e069e43`。schema／设计绑定实际通过，不修改 lifecycle，也不代表各实现完成。
- 约9.5MB真实检查点隔离 profile 已自然0，绑定旧 `a27f31…` native 与1479输入零漂移切点：Store cold691ms／warm561ms，完整coordinator cold3412ms／warm1080ms，每段原10s预算。cold指新authority／目录，没有清OS pagecache；raw／approved字段往返断言通过，嵌套耗时不相加，不冒后继新源码重新编译的结果。
- UI 完整 `npm test-r03` 自然0：lint0、1001P、4个原skip；此前997P／2F原件保留。浏览器只证明受控真实组件的两宽布局及交互，不替代native App、真实手机IM或main验收。
- 媒体批次的两份最新程序与dep-info／SHA已保全，reader0后准确清理自有media-target，natural0，Cargo报告4145文件／833.4MiB；不计作上一阶段旧缓存回收量。

剩余运行时工作继续由各原writer负责：失败／取消已报告usage的三adapter纯查询与真实retry attempt观测；manager产品lease向原SSH Unknown物理resource guard的传递；IM静态全准入路径已闭合候选的9／171及Clippy实际验证；随后一次共享app-core程序供UI、缓存、执行和工作区回归使用。已知旧RED、Clippy101及尚未验证的候选全部保留。旧目录／SSH组按调用闭包整理，模型展示与最近任务折叠分开，等待两条原Registry fresh回归后再提交。

官网精确文档镜像继续遵守clean-source与manifest门禁，待相关源组提交后同步；不绕过当前dirty检查，不推送、合并或发布，不更新未推送子提交的顶层gitlink。阶段回执仍在本报告的ignored目录，新增 `round2-local-delivery.json`、`state-alias-debug.json`、`state-alias-reproduction.json`、`state-alias-verification.json`、`media-r03-validation.json`、`media-clean-settlement.json` 与 `provider-usage-handoff/handoff.json`。

## 第三阶段进展（2026-11-07 晚）

- Framework 微信准入观测与会话退出组已按原作者精确四文件回交提交 `1f35f0d2`。channels 实际171P（其中包含新增9项，不能相加），定向 Clippy 与源码独审通过；App、手机收发和 main CI 不在此结论内。
- CLI 最近任务折叠已独立提交 `ad0a22b8`，仅三个文件的对应部分。基于干净 CLI HEAD 加折叠候选的30P、ESLint／Prettier均0；正式工作树中的 SSH 与其它 UI 未提交字节保持原样。
- 官网已提交 `70884ba9`，从干净 Framework `1f35f0d2` 源视图同步正式镜像，20个路径；source check、24项文档同步测试、格式／lint、342条静态路由构建均0。首轮 sitemap 漂移失败保留，正式 discovery 同步后原检查通过。应用文档投影仍保持原 pending revision，不以脏应用源码更新镜像；没有部署。
- 根规范已提交 `6d51fa96`，仅同步已有两仓新锁的精确人类裁决。原授权文件SHA核对一致，不新增风险接受，不变更锁或子仓指针。
- 缓存与执行／保存的 Framework 候选已取得 v2 投影信封及原 `Parts` 场景回归、Core489P、State387P、orchestration559P、Shell30P、Execution355P、Root定向回归与定向 Clippy 的自然0，各结果仍分别绑定其回执源码切点。原旧格式／Parts／资源 guard 的RED、测试格式及 Clippy 失败均保留。两个公开示例实际执行0，IM示例仅编译0，不声明外部provider或手机资格。
- Framework 剩余实现已在私有 Git 索引中合成为产品scope、缓存前缀、执行与保存、模型用量观测四组；独立边界审查0 findings。合成36文件与当前字节一致，但未分别执行各中间tree测试；正式索引未被该审查修改，尚未提交这四组。
- 最新前端完整切点审后格式／lint／构建均0，1010P、4个原skip；真实原CSP下本地图表九场景及两宽布局通过，npm审计0。正式 web-mode 资产与私有浏览器资产分别记录。App／Tauri／真实Rust DTO到React及原生窗口尚未验收，UI整体仍为WIP。
- 共享 App 首次 `cargo test -p echo-agent-app-core --lib --features channels --no-run --locked` 已于22:30:11自然101结束，未产生测试程序，后继消费者均 `not_reached`。完整12项错误为8处测试引用、2处 wake 观测方法误用、2处异步类型递归。cargo／rustc reader归零且正式bindings未改后，已给IM限定修正窗口；不以加递归上限、放宽断言或新任务替代修正。
- 首编保守清单中一份独立媒体集成测试XLS发生4153→4608字节漂移，来源未确认，原始失败／漂移和当前文件均保全，不恢复或提交。原8份dep-info与源码关系证明它不属于 App 库输入；下一清单将区分保守冻结清单与实际依赖，不回写旧回执。
- 计划覆盖审查还发现两项待补：执行模块的退出检查与producer注册竞态，需要真实原路径barrier RED→修复→GREEN；缓存需补真实projector到三adapter的增长链与旧producer事后隔离对照。已按不重叠文件分别交给原writer，下一唯一 App 编译等三方重新冻结，不并开Cargo。

本阶段已结束的缓存、IM与媒体独立target均在关键程序、dep-info和原始日志保全、reader归零后准确清理；SDK/media的新建批次清理与前述旧缓存计量分开。临时官网干净源视图和已完成折叠候选目录也已核对后删除。共享执行／App target继续保留以复用编译。当前df可用约60GiB；这是并行构建中的阶段观察，不能据此推算独占物理回收量。当前App、配置、认证、用户历史和必要源码恢复包继续保留。

本报告及根规范是协作材料，不改变公共API或示例，故不另增示例或官网产品页。已交付源码组的文档／示例／官网检查分别见各精确回执。所有本地提交均未推送，尚未合并、发布或更新未推送子提交的顶层gitlink；本目标继续执行。

## 第四阶段进展（2026-11-07 夜）

- 共享 App 第二轮原 NoRun 已自然0、零warning，原12项编译错误在新切点消除；首编101原件保持。保守冻结1764项、完整库存1807项前后零漂移，正式bindings未改。该清单明确为保守超集，不再误称精确编译DAG。唯一保全程序SHA为 `45454e9d67d3fb4edc3ddcb7ef05434f4fd55aefb9f1c00c4b37b4273494bb70`，567617888字节、555模式；四会话直接复用同一文件，各自仅保日志与小型回执。
- 图表可见闭包独立提交 `aa357644`，精确8文件，含消息消费入口、本地官方渲染、必要依赖修复和ADR0067。干净CLI HEAD加该8文件候选的审后格式／lint／963P（4原skip）／构建／audit0及原CSP九场景均通过。MessageBubble其余UI工作树字节完整保留，只物化两个原候选新文件；1764项冻结输入前后不变。没有原生App或main CI声明。
- 工作区目录与SSH重连组独立提交 `665a1800`，精确10文件。共享App两条Registry回归均实际list1、run1P；新Chart提交加原10文件候选的四组前端75P、源码零漂移，先前源码独审保持。正式10文件均未改写字节，最近任务折叠与其它UI工作保留。Rust资格绑定完整共享WIP源，不冒该commit单独编译；ADR0055同步，框架／SDK API与示例调用不变，官网应用投影仍待独立正式同步。
- 两份已完成工作区候选源码副本在确认10个blob均已进入665a、lsof无reader后删除；保留补丁、索引、源清单和全部原日志，正式node_modules未改。清理前目录分配分别4820／21152KiB，只作本批计量。候选重建中两次私有工作树 `git apply --index` 失败保留，改用私有索引回放并导出后逐文件核对通过，未改正式源码或索引。
- 执行模块取得真实退出竞态RED：原准入检查后、producer登记前暂停，真实shutdown提前返回，精确负控0P1F／101落在最终断言；原owner先完整清理。另九个具名消费者各1P（包含上述Registry两项，不重复累计）。最小修正只复用既有launch锁包住原cancel／close，独审通过，尚未采用；等所有reader退出统一处理。
- 缓存App实测为compact_context27P1F／101、turn_context7P及1个原ignore、unified_memory4P。唯一compact失败的夹具未把真实TaskRun置为Running，却忽略NotSubmitted，属于前置缺失；修正保留原usage／预算／前缀断言并加入Started检查，仍待新程序GREEN。真实projector到三adapter及旧producer的事后隔离对照、Subagent Tail继承补测已在ignored候选通过静态审查，正式源暂未写入。
- IM首批九项实际7P2F，含四项高层owner／IO保有控制通过。race失败已确认测试把同一会话流中原输入与后续输入的Started混计，修正将按完整identity保留各自恰一次断言与原raw前缀。manager跨scope组合的原child另有真实失败，正在按原日志定位，不能称组合通过或手机验收。UI用量七项已通过，4100条裁剪／冷恢复及其余原组仍在原进程执行；不提前解冻源码。

CLI两次提交只推进Git元数据，冻结源码字节前后均不变；共享App仍按原manifest、环境和完整WIP切点验收。下一批安排为所有reader自然退出后统一限定写窗，先App scoped Clippy，再原NoRun及真实RED对应的GREEN；不通过跳过已知失败取得全组声明。共享target、当前App、用户数据及必要恢复材料继续保留，所有提交仍仅本地，本目标继续执行。
