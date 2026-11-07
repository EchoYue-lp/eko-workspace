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

## 未提交改动分组

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
