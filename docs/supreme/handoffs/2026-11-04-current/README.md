# 当前状态证据索引（2026-11-04）

> **历史归档（2026-11-04 至 11-06）**：以下“当前”、批准前后状态、索引和启动入口均按各自原日期解释，不是今天的执行指引。索引保存原字节观察，不重新签作新包或 main 通过；旧制品可以已退役。当前进度见 [协调交接](../eko-current-handoff.md)。

**2026-11-05 精确绑定已获批准并验证：** 当前30项索引已按本次真人裁决重发，CLI锁48ee及policy已绑定，8控制和两条fresh正式风险gate均0（scanner各1、零漏洞、原五warning）。采用前索引副本及原SHA保存在主根local/dependency-exact-binding-20261105-r01/before/，最初proof-index.json不变。WIP新49项基线只承接原48项中的3处获准文档／policy变化并新增原控制脚本，另45项原字节不变；此前两个基线保留，不追改历史。下方旧段按当时时态解释。

## 2026-11-05 重发索引

当前检查脚本使用 `proof-index-2026-11-05.json`（仍30项）。原 `proof-index.json` 保留原字节，
不把旧锁和旧运行改签为当前。新索引将CLI锁明确标为获准生成的769包候选，原五风险policy仍绑定
旧d542；旧App同源回执只签升级前切点。当前Framework已采用calamine0.38，App和SDK精pin未采用。
新WIP基线 `wip-baseline-calamine038-20261105.json` 仅承接已获准改变的Cargo.lock，47项其它成果不变，
原48项基线保留；这项WIP字节核对不是风险接受。以下表格为11月4日历史索引原说明。

生成时间：2026-11-04T05:25:00+00:00。本索引只回答“这条主张由哪份原件支撑、原件此刻是否仍在且未被改动”，**不回答业务结论是否正确**。每条 SHA-256 在写入瞬间由磁盘实算，未从旧回执抄录。

## 仓库读数

| 仓库 | HEAD | 脏路径 |
|---|---|---|
| `echo-agent` | `91797fed1462` | 0 |
| `echo-agent-cli` | `20070d3f7171` | 48 |
| `echo-agent-sdk` | `83ae59e40731` | 0 |
| `echo-website` | `817eaa48ff8c` | 0 |

## 条目

| key | 主张 | 字节 | SHA-256（前12） | 相对路径 |
|---|---|---|---|---|
| `cli.lock` | CLI 当前正式锁（768 包） | 200,066 | `d542c37ea3d3` | echo-agent-cli/Cargo.lock |
| `cli.riskPolicy` | CLI 已接受五项依赖风险策略（当前字节） | 1,796 | `2028fdc16ebc` | echo-agent-cli/.cargo/accepted-dependency-risks.json |
| `cli.controls8` | 当前 policy 上 8 项本地控制 8/8 通过 | 1,807 | `c5380f4cad1c` | .echo-semantic/local/validation-stage-cli-controls8-20261104-r01/settlement.json |
| `cli.controls8.raw` | controls8 原始输出 | 1,354 | `65a8e7a4a855` | .echo-semantic/local/validation-stage-cli-controls8-20261104-r01/controls8.raw.log |
| `sdk.lock` | SDK 接收候选锁（c4488065，与 stable/合同门禁同源） | 171,858 | `c4488065aa4a` | echo-agent-sdk/.echo-semantic/local/sdk-native-stagea837-formal83-threeway-candidate-20261101-r01/receiving-main747-source/Cargo.lock |
| `sdk.acceptedExternal` | 已接受外部合同（10126202B，合同门禁校验项） | 10,126,202 | `372c8e018c54` | echo-agent-sdk/.echo-semantic/local/sdk-native-stagea837-formal83-threeway-candidate-20261101-r01/receiving-main747-source/contracts/sdk/accepted-external-contract.json |
| `sdk.facadeCatalog` | 已接受 facade 操作目录（合同门禁校验项） | 611,485 | `7383897bff8d` | echo-agent-sdk/.echo-semantic/local/sdk-native-stagea837-formal83-threeway-candidate-20261101-r01/receiving-main747-source/contracts/sdk/accepted-facade-operation-catalog.json |
| `sdk.parityManifest` | parity 清单（38511556B） | 38,511,556 | `dce04ebc9a93` | echo-agent-sdk/.echo-semantic/local/sdk-native-stagea837-formal83-threeway-candidate-20261101-r01/receiving-main747-source/contracts/sdk/parity-manifest.json |
| `sdk.publicApi` | public API 快照（19205567B，rustdoc 校验项） | 19,205,567 | `b00b0cccd622` | echo-agent-sdk/.echo-semantic/local/sdk-native-stagea837-formal83-threeway-candidate-20261101-r01/receiving-main747-source/contracts/sdk/public-api.txt |
| `sdk.contract.settlement` | 完整已接受合同原入口实跑：自然 0、135 阻断产物 current、145 测试通过 | 5,215 | `732f9b0d084e` | .echo-semantic/local/sdk-native-main747-contract-window-20261104-r01/settlement.json |
| `sdk.contract.raw` | 合同门禁原始输出（SHA d1376470…） | 17,311 | `d1376470c690` | .echo-semantic/local/sdk-native-main747-contract-window-20261104-r01/contract-window.raw.log |
| `sdk.conditional.settlement` | 7 个条件 feature profile 全 0、实跑 200 测试 | 5,523 | `b9585799cd49` | .echo-semantic/local/sdk-native-main747-conditional-features-20261104-r01/settlement.json |
| `sdk.conditional.summary` | 条件矩阵逐步退出码与日志哈希 | 9,208 | `736179ef7b99` | .echo-semantic/local/sdk-native-main747-conditional-features-20261104-r01/summary.json |
| `sdk.stable.01` | stable 窗口 01 fmt 写入 | 820 | `30ff38e90117` | echo-agent-sdk/.echo-semantic/local/sdk-native-stagea837-formal83-threeway-candidate-20261101-r01/receiving-main747-source/.echo-semantic/local/sdk-native-main747-engineering-r01/stable-window-r01/01-format-write.caller.json |
| `sdk.stable.02` | stable 窗口 02 fmt 检查 | 845 | `dac1a024c32e` | echo-agent-sdk/.echo-semantic/local/sdk-native-stagea837-formal83-threeway-candidate-20261101-r01/receiving-main747-source/.echo-semantic/local/sdk-native-main747-engineering-r01/stable-window-r01/02-format-check.caller.json |
| `sdk.stable.03` | stable 窗口 03 无 feature 库编译 | 923 | `6db5919369d0` | echo-agent-sdk/.echo-semantic/local/sdk-native-stagea837-formal83-threeway-candidate-20261101-r01/receiving-main747-source/.echo-semantic/local/sdk-native-main747-engineering-r01/stable-window-r01/03-featureless-library.caller.json |
| `sdk.stable.04` | stable 窗口 04 严格 clippy（-D warnings 等） | 951 | `8ca7e5f1e1f1` | echo-agent-sdk/.echo-semantic/local/sdk-native-stagea837-formal83-threeway-candidate-20261101-r01/receiving-main747-source/.echo-semantic/local/sdk-native-main747-engineering-r01/stable-window-r01/04-strict-clippy.caller.json |
| `sdk.stable.05` | stable 窗口 05 全 workspace 全 targets 全 features | 989 | `2a63a7cb49da` | echo-agent-sdk/.echo-semantic/local/sdk-native-stagea837-formal83-threeway-candidate-20261101-r01/receiving-main747-source/.echo-semantic/local/sdk-native-main747-engineering-r01/stable-window-r01/05-workspace-all-targets-all-features.caller.json |
| `app.entry` | abdd App 构建与保全总回执 | 9,967 | `24066ebc198a` | .echo-semantic/local/repair-App-warm-continuation-20261103-r01/epoch-01/entry.json |
| `app.ready` | 手验入口钉定的 ready 状态（launcher 校验其哈希） | 2,617 | `17f949cbb918` | .echo-semantic/local/repair-App-warm-continuation-20261103-r01/epoch-01/evidence/ready.json |
| `app.binary` | 交付 App 主二进制（abdd3fac…） | 176,890,208 | `abdd3faca008` | .echo-semantic/local/repair-App-warm-continuation-20261103-r01/epoch-01/retained/Eclip.app/Contents/MacOS/echo-agent-tauri |
| `app.launcher` | 用户双击入口（生成但从未执行） | 584 | `37530130ff8d` | .echo-semantic/local/repair-App-warm-continuation-20261103-r01/epoch-01/启动修复版.command |
| `app.sourceIdentity` | 交付包构建源与当前工作树 3946/3946 同源 | 4,031 | `7256cb0b16df` | .echo-semantic/local/validation-stage-app-source-identity-20261104-r01/settlement.json |
| `evidence.integrity` | 交付前证据完整性审计（38143 绝对引用，0 未声明漂移） | 2,666 | `953aab317157` | .echo-semantic/local/validation-stage-evidence-integrity-20261104-r01/settlement.json |
| `evidence.integrity.raw` | 审计逐条结果 | 3,863 | `6d77ffcc153d` | .echo-semantic/local/validation-stage-evidence-integrity-20261104-r01/result-v3.json |
| `cleanup.tierA.r01` | Tier A 两处旧 cargo home 删除回执 | 565 | `ab563318d14b` | .echo-semantic/local/validation-stage-maintenance-20261104-r01/tier-a-cache-removal-r01.json |
| `cleanup.tierA.r02` | Tier A 另两处旧 owned cargo home 删除回执 | 769 | `b704ef548525` | .echo-semantic/local/validation-stage-maintenance-20261104-r01/tier-a-cache-removal-r02.json |
| `disk.report` | 剩余资料组成/可重建性/删除后果的实测报告 | 4,922 | `0883d09ca475` | docs/supreme/reports/remaining-validation-data-2026-11-04.md |
| `acceptance.first4` | 首批四项手验具名用例与真实预期 | 6,415 | `ed276904229e` | docs/supreme/acceptance/eko-first-four-manual-cases-2026-11-04.md |
| `acceptance.79` | 79 项 GUI 手验清单 | 25,797 | `8fbd8f6727f3` | docs/supreme/acceptance/eko-eclip-gui-manual-checklist-2026-11-02.md |

## 本索引**不能**据此宣称的事

- **GUI 79 项**：全部由用户手点，目前 0 项有真人结果；本索引里的 App 条目只证明包还在、没被换掉。
- **SDK fresh 风险双门禁**：`github.com:443` 与 `raw.githubusercontent.com` 当前超时，RustSec HTTPS clone 不可达，扫描未取得有效 audit JSON。
- **SDK normal Source/Final 与 canonical**：外部 echo-semantic 工具侧的 `task_digest` 绑定冲突与 delete-preflight 缺口，按约定不在本仓修复。
- **SDK nightly 测试矩阵、public 83 公开来源、两外部 Source 合同**：均未运行或仍开放。
- **CLI publication RED→GREEN**：需要单独资源窗口与 Root 派工，未启动。
- **回执业务结论的正确性**：本索引只核对字节与存在性，不重新判定任何一次验证的结论是否成立。

## 维护约定

源码或锁发生变化后，本索引中对应的 `sha256` 必然失效。交付前应重算并重发，而不是沿用旧条目——这与 2026-11-02 索引的情形相同（那份 233 条今日复核仍全部完好，但描述的是当时的快照）。


## 如何复核本索引

```bash
python3 -B docs/supreme/handoffs/2026-11-04-current/verify_proof_index.py
```

退出码 0 表示 30 条全部与写入时一致；非 0 会逐条打印 `CHANGED`/`MISSING`。**差异不是去改索引迁就现状，而是“索引该重发了”的信号。**

## 相关文件

- `remaining-26-ledger.md`：原完整迭代 26 项的可执行剩余台账（每项写清还缺什么、由谁闭合、本会话有无新增证据）。
- `sdk-merge-plan-draft.md`：SDK 两线合并方案草稿（59 个文件级冲突、解决顺序、合并后门禁），仅分析未执行。
