# 缓存前缀稳定性实施与验收记录

中央归档说明（2026-11-08）：以下各批次的提交、程序和结果按其记录时点理解。
缓存实现及受控验证已本地提交，缓存子会话等待最终来源与门禁材料，原 Plan 保持 ready，
中央协调 Goal 继续 active。官网后继提交已到 `c089e61b…`，应用静态产品投影绑定
`3c13b2ff…`；本报告内较早的 `20070/pending` 是当时状态，不是当前官网状态。
后继界面密度提交 `a1ea6579…` 及未完成的执行候选不继承旧验证资格。
最终 consumer 复评、精确风险接受和原 actual/canonical 门禁仍未闭合，真实厂商缓存收益未验收。

原 r05 状态（历史）：本地实现与focused动态工程验证已通过，代码审查无新增阻断；完整原Plan验收仍缺Outcome4最终两仓AGENTS精确门禁、最终风险复评及正式官网同步回执，Plan保持ready/Goal进行中。真实命中收益和发布继续独立未验收。

最新复审状态：Framework 通用机制/API/adapter 子集已正式独立 pass，0 阻断。
信封边界以发布 v1 marker-only Boundary 与新 v2 明确 placement/空行终止闭合；
Core 489P、State 387P、严格 Clippy 0、原 adapter 5P，原四条真实 RED 均已转绿。
多模态 marker 保护回归另有真实 0P/1F → 1P 证据。审查后本聊天再执行8组14case全0。
新 v2 Root compact 10P 和公开示例均已实际执行，source pre/post无漂移。
App r03 实际消费者已执行：compact_context 27P/1F（原增长夹具缺 Running 前置）、turn_context 7P/1I、unified_memory 4P；失败原件保留。2026-11-08 已在统一无 reader 窗口补齐该测试前置，并落盘三协议真实 store/projector→ContextManager→public adapter→本机 HTTP 的旧/新各三请求对照、正文抵达断言，以及 Root Fresh/Fork 同 marker/version 隔离负控。后继 r05 App 新程序已实际验证完整 compact 31P（含新 HTTP 3 项）、turn 7P/1I、memory 4P，原增长精确用例另 1P（属于重复，不累计）。Root 新 Tail/Fresh/Fork 实际1P也已取得，独立本地工程审查PASS；审后再次实际HTTP3P/Root1P、1764输入与native无漂移、格式与diff全0。这些证据不代表原Outcome4最终AGENTS适用门禁通过，正式Plan生命周期仍ready。

共同 r04 Clippy 自然 101，保守输入及完整库存前后均无漂移，bindings 未改；阻断为其他负责人文件内五处 lint，缓存源没有诊断。这不等于新测试通过，公开示例编译与 NoRun 尚未到达。后继修正只由对应 writer 进行，缓存正式源 `compact_context=8dd4ebd4…`、`subagent/context=461b8974…` 保持冻结。

截至 r05：共同 App-core lib/tests Clippy 已自然 0、零 warning，冻结输入与完整库存前后无漂移，bindings 不变；本聊天已核原始 before/after/log，索引为 `app-r05-clippy-readback.json`。公开应用示例仅编译检查已自然 0；App 新 NoRun 也自然 0/W0（8m26s），冻结1764/完整1807均无漂移、bindings不变。唯一只读 native `649ebd30…`（568,258,896 bytes）由 owner 保全，本聊天直接复用执行原增长及三 filter，未再复制大程序。Root 新 Tail 使用 `default+subagent` 新程序 `fa937630…` 实际1P；原 default 编译0但不含case，不冒0test通过。原 r03/r04 失败保留。

## 执行来源与工作区

执行来源为 `2026-11-07T1729-缓存前缀稳定性/design.md` 及其唯一 `plan_01_动态事实追加与缓存收益验证.md`。用户明确要求立即实施；另明确停止外部 DeepSeek Harness 后将相关未提交增量交给本聊天续接。Plan 的设计绑定校验通过，生命周期仍为 `ready`。

唯一工作根为 `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent`，未创建外部工作树。两子仓分支均为 `feature/Echoyue/ssh-session-reuse`。启动 Framework/CLI HEAD 分别为 `f2e44dc9` / `5f5fd491`，叠加启动时已有增量；相关源码前像已保全。协调者后续分别提交媒体、alias、IM，以及图表/工作区切片；截至 r04 冻结 Framework/CLI HEAD 为 `1f35f0d2` / `665a1800`，提交过程未改变相应冻结源码字节。验证以各次真实输入清单为准，不能用旧 HEAD 标签替代当前字节。

Git index、提交和清理由协调聊天 `01a1b583-b8c0-7ea2-96f7-5cbd77bc3d25` 统一负责。本缓存聊天未 stage、commit、push、merge、运行 App 或调用真实模型。

## 最终行为

- 框架复用原 `ContextProjection/ContextManager`：固定规则保留边界替换，动态事实按最新语义版本追加；重复 prepare 去重，A→B→A 重新追加纠正，清除为一次显式失效。恢复去重从原消息信封重建，不增加 store。
- 仅最新动态版本受保护，旧版经原自动/手动压缩回收；prepared transform 保留版本状态并随原提交/回滚处理。Visibility Horizon 复用真实用户请求判定，动态 User 快照不推进用户回合计数。
- Eclip 将 Goal 固定指令与运行状态、plan revision、预算分开。有界预算只按剩余量 5% 台阶刷新，正文不再夹带每步精确剩余值；无界预算无内容。记忆、turn contract、进度与后台 watch 在事实消失时显式失效。
- 原文引用版本只排除 `observed_through_sequence`，其余完整来源及嵌套 locator 字段使用规范 JSON 计算版本；真正定位或来源变化刷新，序列化错误仍沿 required projector 失败路径传播。新增定位反例属于来源键合同测试，不冒充完整目录迁移的 runtime 验收。
- 缓存分段使用框架版本信封，旧动态版本作为稳定历史；不按 Eclip marker 或裸正文前缀猜测。MEMORY.md 无法可靠自动区分规则与事实，继续保留固定指令权限并接受规则更新后的正常冷 miss。
- 三协议在实际请求构造器后诊断完整 prompt。Anthropic 的 HistoryIndex 经 Core→wire 映射处理多工具结果合并，覆盖普通、原厂 signed reasoning 与工具调用 assistant 分支。
- 语义诊断按 Anthropic 官方 SDK 合同统一字符串简写与单 Text block；保留块额外字段、顺序、工具 JSON 和签名差异。实际 body 的 `wire_representation_hash` 单列，不能把语义稳定叫作原 HTTP JSON 字节稳定。日志只输出摘要、数量和路径；字节度量不叫 token。

正式机制与策略分别见 Framework [ADR 0108](../../../echo-agent/docs/adr/0108-versioned-context-projections.md)、Eclip [ADR 0065](../../../echo-agent-cli/docs/zh/adr/0065-cache-prefix-facts.md)。

## 原始基线与失败保留

原 run SHA256 `877e196c2d7f18abe901276ce9227b91a4d60ef4cfdfe09b96c1be807c3c76df` 和检查点 `e96ab169a28802560bf5ec9168f37174bea9e0779afe11fead82720abac1c309` 已从原件重新匹配。54 次请求累计输入 4,739,565，缓存命中 1,540,352，比率为 32.49986%；其中 52 次固定命中 29,440。统计与查找回执分别为 `original-run-statistics.json`、`original-checkpoint.json`。

| 记录 | 自然出口与结果 | 处理 |
| --- | --- | --- |
| state-red.log | 101，A→B→A 恢复后漏追加 | 只与最新版本比较，后继回归通过。 |
| focused-green.log | 101；core 481P，state 379P/2F | 完整快照说明使 64-token 夹具不足，改为可合法容纳且仍强制压缩；macOS 路径别名旧断言由协调者单独修正，旧失败保留。 |
| adapter-contracts.log | 101，4P/1F | 识别到官方等价文本表示差异，规范化并单列表示摘要；原 5 条断言未改。 |
| focused-clippy.log | 101，3 项测试 lint | 仅移除多余 clone / unit 绑定，保留调用和断言。 |
| raw-source-key-before.rs/.json | 源码反例；后继 App r03 来源键单测已实际通过 | 旧版本键忽略 locator；新增负控后修完整来源键，不称已取得完整目录迁移 runtime RED。 |
| app-r03-cache-compact-context / growth-exact | 27P/1F 与精确 0P/1F，自然 101 | seed helper 未 transition Running；claim 的 NotSubmitted(RunNotRunning) 被忽略。只补原生命周期前置和 Started 断言，原预算/前缀断言保持；后继 r05 同精确用例1P、完整compact31P均已 GREEN。 |
| app-shared-clippy-r04 | 自然 101；完整输入/绑定无漂移 | limits 测试访问方法 dead_code、dispatch 引用比较、usage 测试 helper 参数过多、IM 两个复杂测试类型；均交原 owner 最小修正，不加 allow。 |

原始日志、前像、二进制及输入清单位于主根 ignored [证据目录](../../../.echo-semantic/local/cache-prefix-20261107-r01/)，没有将旧失败覆盖为后继成功。

## 验证范围

| 验证 | 已取得结果 | 证据与限制 |
| --- | --- | --- |
| core/state 完整 lib | core 484P、state 381P，自然 0 | 101 个真实依赖输入前后无漂移；使用主根 canonical TMPDIR。该来源代不等于后继新增 API 的全仓资格。 |
| 新语义诊断后的 core 完整 lib | 485P，自然 0 | `core-diagnostics-final.log`。 |
| 三协议真实请求构造器 | 原 5 条全部通过，自然 0 | `adapter-contracts-final.log`，160 输入无漂移；包括三次连续增长、description 负控、合并工具结果与显式断点，不是实际 HTTP/live。 |
| 测试 lint 修正与 alias 新夹具 | 三原单测各 1P，自然 0 | `alias-fixture-green`、`clear-version-green`、`core-fingerprint-green` 各自真实 receipt；101 输入无漂移。旧/新 state native 独立保全。 |
| 后继 provider producer r04 代 | 原 default 5 条再次 5P；完整 focused Clippy 自然 0，0 warning/error；core 默认查询原单测 1P | 160 输入前后无漂移，`adapter-contracts-r04`、`focused-clippy-final`、`core-reported-default-r04` 均有真实 receipt。新增 usage 行为归 UI；本任务的请求构造区段和原测试经前像比较逐字节不变，不将这项资格称作完整 usage 行为验收。 |
| 原模型压缩入口 | 10P/0F/0I，自然 0 | 自核并复制最终 root native `edb2ade1…`，自行运行 compact 原 filter；新 v2 dd04a1c2 程序已另行复验10P，1479 输入前后无漂移。含 required failure、取消/未结算分类、horizon 与 transcript 持久化，不称全 root 1009 项已执行。 |
| Eclip 真实生产者 r03 | compact 27P/1F；turn 7P/1I；memory 4P，自然结果分别 101/0/0 | 唯一保全 native `45454e9d…`，本聊天直接复用不再复制 568 MB 程序。四批保守源码与 native SHA 前后无漂移；来源键、长 Goal、原文读取/冷恢复用例已执行。turn 原忽略项依赖独立 r03 binding 与默认 60 秒 ShellTool，不称本轮运行。 |
| r05 App 生产者与增长修正 | compact31P/0F/0I、turn7P/0F/1I、memory4P/0F/0I，全组自然0；独立42P/1I | exact growth另1P是同用例重复；HTTP3已包含于31，均不叠加。两批1764输入/native SHA前后0漂移，显式本机NO_PROXY与隔离EKO/TMP/TS票齐全。 |
| r05 新应用→协议本机HTTP对照 | 三协议各旧/新3请求，共18次；3用例全部通过 | 所有12个相邻请求diff均观察旧路径早部改变、新路径append-only；每次当前Some投影全文抵达实际请求。app-r05-cache-http-summary.json绑定原log SHA；旧producer为事后隔离复现。 |
| Root 新 Tail/Fresh/Fork scope 负控 | 新 `default+subagent` 程序实际1P/0F/0I，自然0；审后再1P | native `fa937630…`，源码`461b8974…`；1764输入/native前后无漂移，原 default缺feature事实保留。 |
| usage 归一化及缺失/零值 | 原缓存字段/率边界/write/Responses 共 11P；三协议缺/null/零原合同另 4P，均自然 0 | 复用已验证 SHA 的 v2 native，真实日志分别为 usage-contract-results.json、provider-usage-absence-zero.receipt.json；是本地合同，非 live 收益。 |
| 原三个独立本机 HTTP 兼容用例 | 3 项各 1P、自然 0 | http-loopback-results.json；实际 localhost/fake key。它们覆盖旧强合同，不能替代上述应用动态 producer 连续请求的新对照。 |
| 公开示例 | 原 Cargo 命令自然 0；本聊天核 SHA 后直接 native 再运行自然 0 | `context_projection_cache`，旧程序 `5d5c6774…` 原cut保留；新 v2 `fc1076db…` 已自核执行0、1479 输入无漂移，不调用模型。 |
| 本地实现独立 Review / 格式 / diff | 本地实现/focused工程PASS；审后实际4case及格式/diff均0 | final-local-engineering-pass.json；完整Plan Outcome4最终AGENTS精确门禁仍缺，不能以此row宣称整个Plan完成。 |

## 消费者与交付限制

SDK Host 的生产 `ExtensionAgentComponentProxy` 仍构造旧二字段投影，实际精确依赖 framework `340993201cc937fc8e4783b66be399ef9f14e3d5`。本次不升级该 pin 或既有跨语言合同；未来升级时需要同步构造与合同决定。完整 public inventory 不能代替其独立消费验收。

官网已检查，当前没有依赖这份构造 API 的缓存说明；本次不新增量化宣传。新增公开 Rust 示例进入 Cargo examples 编译链。没有为本优化修改统计公式、隐藏低命中值，或将无 usage 解释成零命中。

真实 provider 前后收益、App、完整合并门禁及精确 main CI 均未安排或未执行；不声称达到 95%、发布或线上验收。源码/合同通过与真实收益分别验收。


## 2026-11-08 对照与证据边界

新旧 HTTP 对照中的旧 Goal/Recovery producer 精确取自 CLI `ad0a22b87bc115a6fd157c7ee49e63cf3e58017f`、blob `67bdd20cfdfb51bc8767361dea7c1abeb4701a49`，完整文件 SHA256 为 `c51507608695153cb58db7e840530cb6cf442c1960f160b2b9185aafd36c85c3`。两个函数仅做测试可见性和等价格式化适配，六个共用 helper 与旧 blob 逐字一致。旧三个边界替换 marker 使用当前 `replacing` 构造与信封，不能称历史二进制回放；这是一项明确标注的事后隔离复现。

新夹具接线为：两个 arm 均使用真实 store/延续配置/Running 与 claim Started 路径，通过解析合成 HTTP usage 记账，并记录真实 Steer 推进来源与 recovery。每协议旧/新各三请求，共 18 次有界 localhost 调用；无真实 vendor 凭据。每次检查捕获请求内当前投影全文、原始 Goal 保持，以及最终协议语义前缀。合成 usage 只驱动状态，不用于计费、延迟或命中率收益证明。r05新程序已实际执行该对照，三协议全部通过；每协议两个旧路径diff均非append-only、两个新路径diff均append-only且无早部差异。例：Chat 第2次比较中，旧路径只保留1个完整相同消息，而新路径保留6个；common_component_bytes分别为146/5258。该度量是规范语义组件字节数，不能称原HTTP字节LCP、token或命中率。完整三协议逐步结果见 `app-r05-cache-http-summary.json` 与原始log，未将本地合成HTTP当live收益。

2026-11-08 对原 run 和检查点原件再次实际哈希复核，仍分别为 `877e196c…` / `e96ab169…`；回执 `original-evidence-readback-20261108.json`。原 32.49986% 与 52 次固定命中事实不因新合同或事后复现改写。本轮新增均为验收测试，原公开示例、SDK pin 与官网量化说明不因测试增量扩张或改写。


## 当前交付与保留项

Framework 精确缓存切片21路径82hunk（原patch `37a7c1e4…`，原base `f24e8d59…`）已由中央本地提交 `3c4951535aaf3711bad618ea3970c2ac8e94157d`，提交前后冻结工作区字节不变。CLI三个源码和ADR0065、持久化仅缓存hunk的精确5路径25hunk切片（`d19dbeb0…`，原base `665a1800…`）已核 reverse check 0，并由中央提交 `0d9b7f2c5291b1ad93ec1d172100678dd739cc5a`；排除持久化末段后台回执/关机等其它任务内容。本聊天未操作index、commit、push或顶层gitlink。

原turn忽略项依赖独立 r03 fixture binding 和默认60秒ShellTool，本任务没有运行，不计作通过。真实provider收益、GUI/App操作、SDK pin升级、完整合并门禁/main CI与发布分别未验收或未授权；这些保留项不由本地合同通过替代。本地实现Review及审后focused验证已完成。中央亲读原Outcome4后确认：CLI此前cut consumer `0150f0…` 与人类接受的 `42a…` 不同，后继A2/UI源切点尚会变化，最终exact gate 0缺失。必须等最终两仓sourcecut冻结后做精确风险Source复评、取得原AGENTS全部适用门禁实际0，以及官网正式同步回执，再按精确原范围处理Plan生命周期。不能把这项当前接受缺口改写为仅未来merge要求；不因本地代码已提交而自动接受新consumer或新风险。四ADR docs parity原失败与处理原件同样保留。该文档/风险处理不自动升级CLI产品projection或任何native/live/main资格。


## 文档实际收口与最终门禁缺口

官网正式同步已本地提交 `6df68a5c2924c22971e591b74e10225c7ae6854e`，基于Framework干净提交视图 `120e71a6…`；sourcecheck、docs24项、format/lint/site/build348路由均有原成功回执。332个镜像/221唯一原文hash按源切点核对，本聊天另从该Git提交取ADR0108 blob，SHA `4c77990c…` 与正式框架原文一致。CLI新ADR0065已在canonical文档登记中明确not_required，原full docs parity check为0，旧失败不改写。源回执索引见docs-website-delivery-readback.json。官网没有发布；原应用20070 pending产品projection保持，未宣称95%或native验收升级。

因此本地文档/官网待办已有实交付；当前完整Plan的具名未完成项仍是Outcome4：最终两仓sourcecut及其AGENTS适用门禁、当前CLI consumer与人类绑定不符后的精确风险Source复评、actual/canonical原gate真实0。先前独立Review的focused工程PASS不能覆盖这项缺口，审查方已明确纠正范围；Plan仍ready、Goal仍进行中。


最终风险输入已有后继：中央恢复了原42a全部274份正文，12份旧Rust源码按原19WIP SHA从保留恢复包精确找回。IMmodel-r07新consumer为`005d6e4c…`/282，8新增、62修改、0删除、212相同；0150属于夹具前旧cut，原事实保留。完整diff为818,352 bytes，来源与恢复票在中央coordination目录。尚有后继UI源码变化，policy、授权、扫描与最终gate0均未因此自动改变。本聊天仅交自有21Framework/5CLI差量的cache-dependency-exposure-facts.md，不覆盖全部62个修改或接受任何新绑定。
