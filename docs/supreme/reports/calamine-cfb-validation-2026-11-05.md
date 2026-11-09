# Calamine CFB 真实失败、局部修复与版本升级待决

## 最新结果：精确绑定已获批准并通过正式门禁

用户直接批准：“同意本次 Framework 8fb62e 与 CLI 48eece 新锁的精确绑定，不自动接受后续新风险。”授权原件在主根 `.echo-semantic/local/dependency-exact-binding-20261105-r01/authorization.json`，SHA256 `a2e324628e7ad1c11dc737405a48cb46a36e8333f109e15f66b4836810815a62`。以下“待决”“原policy未改”均保留其先前时态，本段为最新结果。

Framework正式baseline现绑定8fb全锁和49组108个完整身份，仅新增quick-xml0.42官方registry身份，原107个身份完整保留。原deny策略、工具、命令、严格parser和历史验证正文不变；用同一份完整官方scanner0原raw／native receipt执行正式rebound checker，实际自然0。这是新绑定对既有原始扫描的复核，不另称一次fresh scanner。原14项控制自然0。

CLI正式policy仅lock字段变为48ee，原42a1 consumer与原五项完整风险身份不变；8项控制仅机械更新769包精确期望及相应负控，全部自然0。默认fresh actual-lock和canonical-origin两正式gate／caller实际各0；官方scanner各1，零漏洞、恰原五项warning、ignore=[]、noFetch=false，原stderr各0字节，DB965bf9ad／1403条。两个独立命令分别产生同内容audit JSON，SHA256均为 `7fde83a1cfad12eefb4034668bae61da3a2fb91f61762da821f301f33fb59b13`。原件、完整命令和出口均在本次授权同目录，不借候选结果签正式通过。

根及两仓AGENTS与CLI风险决定同步此次裁决；原始proof-index、原48项WIP基线和所有失败原件保留。公共API、examples和官网产品行为没有新增变化，无需同步业务内容。本轮未编译Rust或创建target，Cargo clean不适用。SDK pin及现App未更新，完整工程、GUI79、原26项和main CI仍独立开放，不把这次绑定称为整体交付完成。

日期：2026-11-05。此记录属于当前验收与同源收敛Goal，保留原26项义务；不包含发布、
推送、合并、GUI或用户配置操作，也不把局部回归计作完整发行资格。

## 当前后继：用户已批准采用0.38，精确绑定待决

用户直接选择“采用0.38.0并生成新锁候选；逐项复评，不自动接受新风险”。以下旧局部阶段
文字保留其原时态；当前正式Framework已应用已审49项升级补丁，vendor为官方0.38.0，
旧0.36目录已移除，root manifest／registry-origin／唯一生成器已同步。SDK精pin仍340，
SDK锁仍c448，App未重建。第三方0.38新增样式／属性API，不宣整个第三方Rust API相同；
第一方工具入口和schema不变。

### 实际新锁与完整差量

| 仓库 | 新锁SHA256 | 包数量 | 相对原冻结锁的变化 |
| --- | --- | --- | --- |
| Framework | `8fb62e249fc8fa3114a51b85277f201a98bb190572a80293a91555d8a519e928` | 676→677 | calamine0.36.1→0.38.0、增加quick-xml0.42；tempfile原节点的getrandom边0.3.4→0.4.3 |
| CLI | `48eece730daaa9eff539e416861454441c23c54abf02bacaa2c3bdb375bd6329` | 768→769 | 同一calamine升级及quick-xml新增；iana-time-zone原节点的windows-core边0.62.2→0.61.2，旧quick-xml引用增加版本消歧 |
| SDK | 原c448不变 | 原图不变 | 现pin尚未采用Framework本轮修复，不编造新的Git revision或锁 |

两条额外边均符合其原官方依赖约束；不是calamine修复强制要求，也不作为Windows或运行等价证明。
CLI比较基准是旧d542的原构建快照，未用HEAD混入旧async-executor WIP差量。
原48项WIP保留47项逐字节不变，唯一获准改变为Cargo.lock；新WIP字节基线与旧基线并存，
不代表依赖风险已接受。

### 新源实际验证与审查

- 正式Framework0.38源的CFB2项与原真实XLSX／DOCX2项全部通过，Clippy及fmt write／check各0。
- CLI `cargo check -p echo-agent-app-core --no-default-features --locked --offline` 自然0，编译当前Framework0.38消费图。
- 原来源投影控制16项全部通过。
- 两份新锁的首轮fresh官方audit JSON均零漏洞、原五完整warning不变，0436不再出现；scanner仍各1，不能称零warning审计。原输出含数据库锁等待提示，完整保留。
- 后继为排除数据库等待提示而串行重验时，actual和canonical均在原URL fetch阶段退出1／无有效audit，原件保留；不把前一次有效JSON改成这次网络成功，不继续盲重试。
- Framework官方deny自然0，原严格parser完整比较为49组108身份；仅原quick-xml组增加0.42.0，原107身份无退缺。格式行顺序不作为身份变化。
- 独立源码／锁复评pass、零findings。当前结果不签完整工程、平台、SDK新pin、App或mainCI。

### 可审阅的精确绑定提案

Framework正式baseline仍1442／49组107身份；CLI正式policy仍2028／旧d542／原42a1消费者。
提案仅把Framework的锁绑定更新为上表8fb并接受新增quick-xml0.42完整registry身份，
以及把CLI policy的锁字段更新为48ee、原五风险和consumer42a1不变。没有新增漏洞／ignore／allow／过滤。
CLI原8项风险控制当前实际1（3失败、7子错误），首因为新锁未获原policy绑定；不能声称通过。
同名8项控制的769包精确期望修正已备补丁，断言和负控仍保留，未在正式树采用。

所有提案都在本仓local的 `calamine-advisory-validation-20261104-r01/`：

- `framework-duplicate-baseline-proposal.json`：PENDING标识，完整49／108；未改正式baseline。
- `cli-risk-policy-proposal.json`：仅新锁字段；未改正式policy。
- `cli-exact-lock-binding-proposal.patch`：CLI policy和8控制的精确包数量后继；尚未应用。
- `framework038-lock-delta.json`、`cli038-lock-delta.json`、`upgrade038-official-risk-comparison.json`：实际全差量与原输出入口。

原AGENTS禁止将旧精确版本集合自动扩大；因此源码采用授权已执行，精确新集合／锁的接受仍须
用户看到上述实际结果后单独裁决。裁决不预授未来版本或风险，fresh正式gate仍按真实结果结算。

本升级批次的2个实际媒体程序及原FP／dep-info／必要check产物已同卷保全4776文件／约78.58MiB，
reader0后准确 `target-upgrade` 官方clean自然0，移除1208文件／1.1GiB，target已不存在。
最终原件为 `upgrade-settlement.json`（SHA256 `ccb7bbd7939c6057b1510c91c0908d94a12dd8c3537d0d4fa1dfef818983a515`）。
30项索引和48项WIP基线已按本次获准锁变化重发，旧原件保留；当前检查仍逐项核字节，
不把索引／WIP通过当成精确依赖policy通过。

## 新的实际阻塞

原官方RustSec仓库恢复可达后，沿CLI原脚本、不改URL／fetch／ignore／timeout执行两项fresh门禁。
输入仍是CLI consumer `42a1dd0dd843ee8629ff12dc20d238dc9356c7b35d29e4bd3b353c0b72d53f42`、
policy `2028fdc16ebc07c0d29ae6aa9c5a6902eed61b03360760d4a14a980aa0a751be`、
锁 `d542c37ea3d32ca9a8064f0ef473fa3e2fd7175c0576c8e97d4564db09ed39e1`／768包。
两次数据库均为 `965bf9ad083b58e6340d65829d85217b18be9290`／1403条。

| 原入口 | 官方scanner | 独立精确gate／caller | 实际结果 |
| --- | --- | --- | --- |
| actual-lock | 1 | 0 | 零漏洞、原五项完整warning；noFetch=false、ignore=[] |
| canonical registry-origin | 1 | 1 | 原五项warning及新增calamine0.36.1漏洞；不能接受该新漏洞 |

新增项为 [RUSTSEC-2026-0436](https://rustsec.org/advisories/RUSTSEC-2026-0436.html)：
CFB/OLE2目录尾部短块可导致固定偏移读取panic，循环DIFAT链可造成无界分配。
Framework `echo-tools/src/excel.rs` 的XLS／XLSB及VBA消费可达该vendored实现；
本地用户读取损坏附件也成立，修复属于框架通用解析机制，不增加EKO权限门控。
官方scanner跳过path／Git包，因此actual0不否定canonical发现。

## 已验证的局部修复

参考上游 [PR762修复提交](https://github.com/tafia/calamine/commit/b8ec4f09cba6aea7d8359ababc240c9ed064751b)，
Framework唯一维护现有vendor的DIFAT遍历上界、空链typed错误和短目录块过滤；
内部计数按本仓规则使用饱和加法。未改正式版本、manifest、registry-origin、三个锁或精确风险基线。

两份上游样本按Git blob和SHA256核验：短目录4153字节，循环DIFAT512字节。
它们进入现有 `echo_tools`／`media` 集成测试入口；没有新增解析器或平行生产权威。

| 验证 | 自然结果 | 资格 |
| --- | --- | --- |
| 旧0.36.1短目录样本 | 101，0P／1F | 实际panic：目录块57字节，读取64字节；旧OOM样本未运行 |
| 局部修复两样本 | 0，2P | 修复后实际通过；加强版OOM断言要求CFB扇区边界错误 |
| 正式Framework媒体入口 | 0，CFB2P＋XLSX／DOCX2P | 真实工具保留Unicode、数值、布尔及文档内容 |
| 定向Clippy `-D warnings` | 0 | 原媒体两测试目标，零编译warning |
| Framework fmt write／check | 各0 | 未改变当前冻结源码 |
| 官方0.38.0及本地依赖适配候选 | 各0，两样本2P | 有重叠，不相加；最新加强断言后0.38候选2P |
| 独立审查 | pass，零findings | 仅PR762局部修复与提案一致性；不签任意损坏CFB输入全面安全 |

本次不重跑SDK589，因为SDK精pin未采用Framework WIP，SDK已验证输入未变。
现有App同样未包含此修复，旧同源回执仅保其当时源码切点，不继续宣称现Framework工作树与App全相同。
examples／官网调用方式、公共API和工具schema未变，无需新增同步内容；Framework vendor说明与样本说明已更新。

## 官方0.38.0升级的可审候选

官方归档SHA256为 `f431b67cba6122a1f7d26b7adafc71bbdf07026095fe811f5efc025d8748e1d8`。
候选保留MIT许可及完整runtime成员，保留既有非yanked的Zip9.0.3，移除依赖未用开发target，
增加独立workspace声明，DIFAT计数使用饱和加法。49项补丁已生成，`git apply --check`实际0，尚未应用。
补丁SHA256为 `49d554bf0fa38c05780893bc1702815d9913289a3ca3c8f6eec2d9eb515c2789`。

0.38将quick-xml从0.41升级到0.42.0，新checksum为
`41b1177fdf999d2321d3fb46ff47159d9c1fb9ad66a4879f8c50a0b504615e9b`。
Framework／SDK现有duplicate精确集合包含quick-xml0.39.4、0.41.0；新版本身份不在原接受范围。
CLI精确绑定还要求新锁重新裁决。既有AGENTS及机器gate明确禁止自动扩张新组／版本／来源或锁。

推荐由用户批准采用官方0.38.0并生成实际锁候选，再逐项复评真实图与原风险身份，
在同源媒体／完整工程／风险门禁通过后准备后续交付。该批准不等于提前接受任意新风险，
更不授权改ignore、虚构checksum、重标0.36.1为0.38、自动升级SDK pin或远端发布。
另一选择是暂保局部回补，继续将canonical风险门禁标阻塞；不能把局部2P代作风险修复完成。

## 保全与资源收口

无reader后，同卷移动保全本轮4个当前程序及原FP／dep-info／build输出共3490文件，约19.45MiB；
旧RED程序另存，原始失败／成功日志均在target外。准确自有target官方clean自然0，
移除965文件／892.4MiB，target已不存在；未清旧目录或用户资料。

原件入口：

- Framework `.echo-semantic/local/calamine-advisory-validation-20261104-r01/settlement.json`。
- 同目录 `inputs.json`、`before-preserved.json`、`preservation-manifest.json` 和 `calamine038-upgrade-proposal.patch`。
- CLI `.echo-semantic/local/current-validation-risk-20261104-r01/settlement.json`，含两个fresh原audit／gate／caller。

SDK本轮10份资料另已保持旧正文／状态／修复链并完成当前来源收敛，installed0.5独立strict实际31错→0；
旧c0bc原argv仍1／4错，两种资格分列，不修改外部工具求绿。
