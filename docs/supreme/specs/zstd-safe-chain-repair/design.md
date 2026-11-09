---
title: Framework 与 SDK 的 zstd 安全依赖链修复
artifact: design
carrier: markdown
---

# Framework 与 SDK 的 zstd 安全依赖链修复

## 决策状态与权威

用户于 2026-10-27 明确采用方案 A：由 Framework 唯一维护两份官方 Polars 发布包的最小补丁，补全五项审计来源合同；SDK 通过同一个正式、可恢复的 Framework revision 消费，不复制 vendor，不增加 ignore 或扩大风险接受。

这是本次跨仓工程设计入口。长期 Framework 选择由所属 [ADR 0100](../../../../echo-agent/docs/adr/0100-zstd-safe-chain-vendoring.md) 解释；SDK 的既有 accepted/source 合同、Host adapter 与生成器继续拥有各自事实，不由顶层文档代替。本阶段只新增正式设计与 ADR；下述目标状态、标准依赖门禁和消费者验收尚待实际实现与验证。

## 问题与目标

已发布的 `polars-arrow`、`polars-parquet` 0.55.2 约束 zstd 0.13，使 SDK 完整 facade 的数据能力仍解析到 `zstd 0.13.3 → zstd-safe 7.2.4 → zstd-sys 2.0.16+zstd.1.5.7`。SDK 本轮原锁在既有审计政策下仍报告四项漏洞和十项 unsound 告警。只改锁无法越过父包约束。

目标是在保留 Framework data/statistics、SDK 5,352 项接受身份、431 条路由及真实 Parquet/Host 能力的前提下，形成并验证真实安全链 `zstd 0.14.3 → zstd-safe 8.2.0 → zstd-sys 2.1.2+zstd.1.5.7`，同时保持包归属、锁依赖边、独立来源及失败证据可信。

这条依赖修复不作为 EKO GUI 工作的先决条件。当前 GUI 消费图没有启用 data/statistics，CLI 当前锁没有 Polars/zstd；不能为 SDK 完整数据 facade 的依赖门禁停止独立 GUI 收尾，也不能用 GUI 构建通过证明 SDK 数据链通过。

## 目标行为与范围

Framework 从当前工作分支的真实源码/锁扩展，而不是覆盖为旧探针快照。根 workspace 用两条 `[patch.crates-io]` path 选择其自有 vendor，并明确 exclude 两个第三方包。Polars 版本和工具/API 功能保持，只有两个 normalized manifest 的 zstd version 要求由 0.13 改为 0.14.3；其余声明、生产 Rust 与原发布成员保持原字节。

独立 SDK root 显式以 canonical Framework Git URL、同一个精确 revision 配置 Host 依赖和两条根 patch，全部匹配父边只选择这一份 Arrow/Parquet。Framework 依赖自带的 root patch 不会自动传播到 SDK。既有三个媒体 vendor 继续通过 Framework 原相对 path 机制传递，不混称为五包都自动继承。

本设计不删除 data/statistics、存储 API 或其它合理 Framework public API，不改变 SDK 协议载荷、产品权限、任务运行时或 UI，也不处理外部 echo-semantic 合同修复。不发布 crate、SDK 包或网站，不 push/merge，不把本地候选当 main/CI 或远端交付。

## 系统边界与已有实现

| 边界           | 唯一责任与复用对象                                                                       | 本次变化                                                           |
| -------------- | ---------------------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| 通用 Framework | `echo-tools/src/data.rs` / `data_quality.rs` / `statistics.rs`、原 Polars 与 zstd 机制   | 维护发布包的两份最小源码补丁；保留真实数据能力                     |
| 构建与审计     | `vendor/registry-origins.toml`、`scripts/generate-vendored-media-audit-lock.py` 及原回归 | 从三个准确来源扩为五个；保留唯一投影算法、原 source/owner 拒绝行为 |
| SDK 适配       | Host 精确 Git pin、SDK 根 Cargo patch、原来源断言、唯一生成器与 Host 路由                | 同步真实来源/锁并验证保留合同；不复制 vendor/审计算法/运行权威     |
| EKO 应用       | 既有 GUI、IM、cron 与文件持久化                                                          | 当前不加入 data/zstd 依赖，不新增应用 workaround 或运行门控        |

已按定义、注册和真实调用检查：Framework 原 `load_dataframe` 返回 `DataFrame`，ParquetReader/Writer 仍由原路径使用；StandardToolPack 注册 15 个 data 和三个质量工具。SDK 原 facade 的 18 个 data 操作经 Host `run_bound_tool` 执行原工具，再转换 `ToolResultWire`。现有 Host CSV 测试和 JSON 导出夹具不等于实际 Parquet/Zstd 往返验收。

标准库和平台能力不能替代 Polars 的 zstd 版本约束；已安装依赖已提供实际压解函数、`BasicDecompressor` 和私有 context 缓存，不需要新 wrapper、缓存、store 或第二解析器。最小自定义内容就是两个显式 manifest 补丁与既有审计声明/回归的有界扩展。

## 核心结构与数据流

### 原包归属与构建图

| 包             | 版本   | 原官方 `.crate` SHA-256                                            | 原发布成员数 | 许可证             |
| -------------- | ------ | ------------------------------------------------------------------ | ------------ | ------------------ |
| polars-arrow   | 0.55.2 | `c5ab55460ca2553d1a2bbaf2046998cfbbb685bc18d1e168b4367b63301eb271` | 287          | MIT AND Apache-2.0 |
| polars-parquet | 0.55.2 | `d33a17979d2afa0309437478662cf996361a6e1768bf467db82bd4f4a9443064` | 158          | MIT AND Apache-2.0 |

原 VCS 均为 `d7488c71ecfbc77790292ff5b365b991c08380ce`，路径分别为 `crates/polars-arrow` 和 `crates/polars-parquet`。保留两个包的完整许可、归属文件、`Cargo.toml.orig` 与 `.cargo_vcs_info.json`。445 个成员逐个校验，除两个 normalized manifest 的指定补丁外不变；不删开发成员或为了格式遍历另改包内容。

原 archive checksum 只证明发布来源。修改后的树由正式 Framework commit/tree、文件清单及实际差异证明，不宣称仍等于原 archive。当前锁由实际 manifest 解析产生，保存所有真实依赖边；不能手改版本/checksum，不能复制旧探针整锁而降级当前 Tokio 或抹去其它已交付修复。

### 五来源审计合同

继续使用 schema 1：顶层只有 `schema_version` 和 `package`；每项只有 `name`、`version`、`checksum`、`manifest`。准确列出 `rust_xlsxwriter 0.82.0`、`calamine 0.36.1`、`docx-rs 0.4.22` 及两个 Polars 0.55.2。原三项身份不变，新增两项使用上表原 archive checksum 和 Framework vendor manifest 路径。

审计数据流是：真实消费锁与同 revision Framework 的完整五源声明 → 原唯一生成器校验声明并从消费图确定应投影集合 → 临时 registry-origin 视图 → 原官方 scanner。构建仍只读取真实消费锁；审计视图不回写 Cargo.lock、不作为 registry 可解析构建图或另一份 source authority。

完整声明与实际投影数量分别约束。原三个媒体父包继续必需；对于每个 Polars 父包，只要消费锁存在该包记录，或任一包的实际 dependency 引用它，就必须纳入应投影集合，并拒绝被引用却缺失的记录。只有该项包记录和全部依赖引用均不存在、确实未进入该消费图时才不投影。按每项真实包身份与依赖引用判断，不采用 CLI 特例、手工 scope flag、静默跳过缺包或单纯数量判断；未来合法 Arrow-only 消费图也遵守同一规则。

本次直接核对的真实锁：Framework 679 包及 SDK 653 包中，两父均存在，Arrow/Parquet 分别有 15/4 条父引用，修复后两个完整数据场景应投影五项。CLI 856 包中两父和 zstd 链均无 package 记录、无任何 dependency 引用，原三个媒体父包各由 `echo_tools` 引用，所以在相同完整五项声明下应继续得到原三项投影。扩大归属声明不能令这个无数据能力的真实消费者原门禁失败。

必须满足：

- 五来源名称/版本唯一且完整，原 checksum 合法，manifest 属于该 Framework vendor 且与声明身份一致；拒绝链接逃逸和未知字段。
- 锁中只有一个 path/Git `echo_tools` owner，应投影的每个父包恰有一条、版本准确、与 owner 的实际 SourceId 相同；无关 fork、混合 revision、应有却缺失的包、重复包、错版本、registry 回落或意外 checksum 均拒绝。依赖引用必须与实际包身份相符，不把未解析引用当成未选入图。
- 只有应投影包的 `source` / `checksum` 变为原 crates.io 身份；由补丁 manifest 产生的实际 dependency edges、版本、其它包和字段全部保持。不能恢复成旧依赖边来迁就 scanner；未入图的 Polars 声明不产生虚构锁记录。
- 输入/硬链接别名不能被输出覆盖，已有输出不能被悄悄覆盖，错误保留自然非零出口。原 strict3 返回 0 只能证明原三项，不能证明新增两项归属覆盖。

### 公共类型与接受合同

`polars_parquet::parquet::compression::DecompressionContext::Zstd` 的 payload 类型来自 `zstd_safe::DCtx`。safe7 → safe8 改变 crate 身份，两父也从 registry 包变成 Framework path/Git 包；因此“Rust 源未改”不等于全部 Rust 类型身份等价。直接把旧 safe7 的 `DCtx` 赋给新变体不获兼容承诺。

保留原 `Unset`、`BasicDecompressor`、私有 context 复用，以及 `load_dataframe -> DataFrame` / `is_numeric(&DataType)` 的真实使用路径，不以删除公开 helper 或收缩 features 避开验收。无需新增 SDK wire 类型，原 `ToolResultWire` 不携带 DataFrame/DCtx。

SDK 保留集以原 accepted contract 为准，完整 public inventory 仍是非阻断遥测。只由原 export/schema/catalog 入口在新正式 pin 下产生真实差异；不得手改生成 digest，不把新发现 Rust 类型自动纳入三语言。若 accepted 输入字节不变，source-contract digest 应自然不变；若有保留项变化，则具体解释和审查，不能删除原 retained-entry 检查或机械重置基线。

### 来源切换的资格关系

Framework 补丁及适用门禁成立、精确 Git 对象/tree 可恢复，是 SDK 改 pin 的前置。SDK 的 Host pin、根 patch、锁、`check-sdk-contracts.sh` 的 expected source、`check-a2a-lsp-retirement.py` 的 pin 断言及自身 baseline 来源绑定必须一致；原 269 项退出、5,352 项保留、431 路由与保留行漂移检查继续有效。新 pin 同时包含原 f7bac 至今的其它 Framework 改动，不能仅凭两个 manifest 差异证明整个 pin 的兼容性。

当前不允许 push。可先由协调者形成经门禁的 Framework 本地 Git commit，保存精确对象/tree、恢复材料与回读结果，再开展明确标识的本地 SDK 候选验证。若沿已有方式使用进程级 Git 传输或自有 Cargo cache，必须记录实际对象从何取得、精确 SHA 与原命令，不改全局设置；canonical URL/revision 与物理传输来源分别如实记录。不能用临时 `file://` manifest、path-only 编译、Git alternates 或探针提交 `7448e551...` 冒充正式 canonical 来源。

本地候选、远端已可取、无本地补充的冷恢复、精确 main/CI 是不同资格。现有工具若不能在不改来源权威的情况下验证本地候选，则保留来源阻断；不删除门禁、不伪造远端成功，也不为通过擅自 push。后续公开来源与冷恢复须等相应授权，设计采用本身不授权发布。

## 异常、并发与恢复

| 场景                                                                             | 必须观察到的行为                                                                           |
| -------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| 五源声明少项，或应投影包缺失、错版本、错 owner/source、非法 checksum 或 manifest | 明确拒绝；不继续扫描并宣告完整；真正未进入图且无任何依赖引用的 Polars 与被引用缺包分别处理 |
| SDK 忘记根 patch 或与 Host revision 不同                                         | 真实 metadata/来源断言失败；不容忍 registry 副本混用                                       |
| index/yanked 查询错误，即使 scanner 自然 0                                       | 保留 stderr，完整依赖资格未取得；五个原包的补证不冒整个锁图检查                            |
| 新 duplicate 组/来源/版本、其它 warning 或漏洞                                   | 原独立风险合同继续阻断，不新增 ignore、不按计数相同放行                                    |
| 新 pin 导致保留 accepted 项或真实消费者失败                                      | 保留原失败与具体差异，修复或报告新取舍，不扩大接受范围                                     |
| 验证期间其它协作者修改输入                                                       | 失去该快照资格，冻结后重验受影响批次；不把不同源码结果拼成一套通过                         |
| 部分来源/锁/pin 切换失败                                                         | 仅恢复本任务配对文件到已记录切点，保留新候选和原日志；不单独回退 manifest 留下失配锁/断言  |

所有源码、构建、日志与恢复材料留在唯一开发根。维护 SDK 既有 94 项候选、固定分支/tag、Exa 提交和运行中的原生 GUI，不因本依赖修复丢弃或重写。每批 Cargo 结束先保存原始日志、自然出口和必要制品，确认准确自有 target 无读取者后 clean；最多两个 focused 批次、jobs≤2，低于 5 GiB 停止新构建。纯文档阶段没有 Cargo target，清理不适用。

## 参考实现与关键取舍

参考 [Cargo 官方依赖覆盖](https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html) 和 [Git 包发现](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-dependencies-from-git-repositories)，并复用本仓 [ADR 0089](../../../../echo-agent/docs/adr/0089-non-yanked-media-dependencies.md) 的原归属、拒绝行为及审计投影。它们共同支持“一个源码所有者、独立消费根明确选择、实际锁与审计归属分开”的做法。本轮已用本机 Rust 1.95 附带官方 Cargo 手册核对规则；在线读取失败不作为当前在线内容已校验的证据。

| 候选                                              | 结论                                                                         |
| ------------------------------------------------- | ---------------------------------------------------------------------------- |
| 等待上游新 Polars 或只更新锁                      | 前者无法立即闭合已知问题，后者不满足父约束；将来上游满足时可移除补丁         |
| 删除数据能力、风险 ignore、改名/版本 shim         | 破坏范围或隐藏身份，不采用                                                   |
| SDK 自存 vendor、复制审计算法或整批反向依赖       | 增加第二权威与维护面；已有两根包发现证明无需如此                             |
| Framework 两包最小补丁，SDK 显式同 revision patch | 用户已采用；承担本地父包维护和独立 root 配置成本，保留真实类型影响和独立验收 |

## 验收标准

| 要求                 | 可证明该要求的实际证据                                                                                                                                                                                                      |
| -------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 原包与最小补丁       | 官方 archive/checksum、完整许可/VCS、445 成员逐字节清单，唯一两 manifest delta                                                                                                                                              |
| 五源声明与消费图投影 | 完整五项声明与 manifest 校验；正式 Framework/SDK 完整数据锁投影五项、真实 CLI 锁投影三项；合法单父消费图沿同一规则；仍被 dependency 引用的 Arrow/Parquet 缺包、重复/错版/错 source 均失败；除 source/checksum 外字段/边不变 |
| 真实解析图           | 正式 Framework 与 SDK 完整 locked metadata；同源唯一两父与安全链，旧链/registry 混用为零；真实锁差异审查                                                                                                                    |
| 标准依赖资格         | 两仓各自 fresh 标准 registry/RustSec、正常 audit/deny 加五源视图、完整 yanked/license/source；原精确 duplicate 集及自然出口完整，不继承另一仓豁免                                                                           |
| Framework 实际能力   | 真实 Arrow IPC 与 Parquet 压解、context 跨尺寸复用及坏帧恢复；原 data/statistics 和示例编译；原 read/export 工具进行含中文、null、数值的 Parquet/Zstd 往返，错误路径明确；相关 media+data 组合涵盖 Excel→Parquet            |
| SDK 保留消费者       | 同正式 pin 原 contract/catalog 全检查和唯一生成器；5,352 身份/431 路由不扩，18 data 操作与 Host 的实际 Parquet 往返、原 Rust/三语言/quickstart 及适用矩阵分别取证                                                           |
| 恢复与交付           | 精确本地 Git 对象/树及回读、来源传输记录、配对恢复材料、准确 clean 回执；远端冷恢复与 main/CI 未验证时明确保持未完成                                                                                                        |

正式实现必须检查并同步 Framework vendor/依赖说明、SDK pin/治理文档、既有 examples 与官网来源说明。没有新用户功能或 schema 时无需新增 API 宣传；不得宣称 registry 发布可用或 root patch 自动继承。文档同步检查及不适用理由仍须交付记录。

## 已有证据与剩余边界

已审隔离实验足以证明所选路线可行：F/SDK 两完整图分别 679/653 节点可解析、两父真实压解 3/3、同一 1,378 条数据库下 SDK 的十四项具名告警消失。它们属于旧 f7bac 探针和临时 Git 来源，没有正式 full Host、三语言、fresh 标准依赖或全部 yanked 资格；其中五项 index 查询失败及手工实验 cache 观察原样保留。live Framework 已修 Tokio，不能用旧探针锁降级或把旧 Tokio 告警说成当前新增。

原始证据位于 Framework 的 `.echo-semantic/local/zstd-patch-probe-20261027/`：`隔离实验结论.md` SHA `5662dd304b62bc3623e87fa57cfc5fe51015342cde41dde84d22fdc6fe17540b`；公共边界报告 `.echo-semantic/local/zstd-public-boundary-20261027/公共合同影响与采用边界.md` SHA `40b8058b9ab957fe6ff9130784471d61e8be6608af6fc34014d8aeeb6181e438`。这些是运行原件和调查限制，不取代上述正式代码权威或新快照验收。

后续执行以本设计及所属 ADR 为依据，由协调者指定精确 Plan/Build 切片并保持来源切换前置条件；本设计不生成第二套实施计划，不把任何尚缺结果标为完成。
