# 本会话回执索引（第 18–40 轮）

本文件只做导航：每条一句话结论 + 原件路径。回执原件全部在 `.echo-semantic/local/<目录>/settlement.json`，本文件不改写其中任何字节。

**读法**：先看结论，需要细节再打开原件。带 ⚠ 的是推翻了先前结论的更正，带 ✎ 的是修正验收文档预期。

---

## 一、磁盘与清理

| 目录 | 结论 |
|---|---|
| `validation-stage-disk-composition-refresh-20261104-r01` | 65 GiB 中 **61 GiB（94%）是四仓 `.echo-semantic` 证据树**，构建缓存仅约 1.0 GiB。 |
| 同上（第 23 轮补章） | `.worktrees/` 内**没有一个是 git worktree**（无 `.git`），实为历史证据目录；`ZCode` 是第三方 `zai-org/ZCode` 克隆。另有 2 个**真正的外部 worktree**（`a8c2d1ae`／`40fd804`），提交均已在任务分支内、无独有工作。 |
| 同上（第 22 轮重复分析） | ≥50 MiB 文件 156 个名义 39.50 GiB，但按 `st_blocks` 实际分配，同名同尺寸重复组仅可回收 **1.88 GiB**（7 份 `large.bin` 名义 14 GiB、实际≈0，稀疏）。**不动证据的回收上限约 2.9 GiB**。 |
| `validation-stage-maintenance-20261104-r01` | Tier A 删除 4 处旧 owned cargo home 共 4.157 GiB，两处 target 官方 clean 共 4.9 GiB；工作区 69.555→65.47 GiB。 |

## 二、SDK 同源与门禁

| 目录 | 结论 |
|---|---|
| `validation-stage-sdk-lineage-audit-20261104-r01` | 分支 10 提交（3 个 `WIP:`）不在任何远端；origin 仅有 `6cd2e1b` 退役 + `747be93` 文档校准；81/96 分支新增文件不在 origin。 |
| `validation-stage-sdk-merge-plan-20261104-r01` | 12 节方案；第 13 节为后续更正。 |
| `validation-stage-sdk-prior-merge-20261104-r01` | 发现 2026-11-01 已存在三方合并（`candidate-source`，44 冲突已解），"合流未开始"的说法不成立。 |
| `validation-stage-sdk-pin-direction-20261104-r01` | `6cd2e1b` 正文明确"九项原生消息后继保持独立待三方合流"→ origin 有意排除原生消息。 |
| `validation-stage-sdk-contract-baseline-delta-20261104-r01` | 基线 origin 5352 条/433 路由、分支 5344/432；12 条共享条目签名不同；**全部是 pin-008→340 漂移而非退役**，基线须重生成不能二选一。 |
| `validation-stage-sdk-current-plan-20261104-r01` | `state/mod.rs:106` 的 `current_plan` 在 pin `3409932` 被整体移除；`wire.rs` 的 `structure!` 映射会因缺列编译失败——**不是"安全可加"字段**。 |
| `validation-stage-sdk-residual-attribution-20261104-r01` | 残差归因（147 差异）；**"9 份 plan 文档缺失"此条已被第 24 轮推翻**。 |
| ⚠ `validation-stage-sdk-receiving-candidate-recheck-20261104-r01` | 撤回"缺 9 份文档"：`git ls-tree` 默认 `quotePath` 输出八进制转义串被当路径判断存在性。实测 **相同 1075／不同 147／缺失 7**，缺失 7 个正是退役删除项，与回执 `D=7` 吻合。 |
| `validation-stage-sdk-gate-integrity-20261104-r01` | 合同门禁回执自洽：日志 sha256 `d1376470…` 重算一致、`naturalExit` 两处均 0、834.8s、`noUpdateFlag` 无重生成；候选树 2930 文件最新 mtime `2026-11-02T06:26Z`，**门禁后未改动**。 |
| `validation-stage-sdk-delta-coverage-20261104-r01` | 147 处改动覆盖分级：**A 级** Rust 32 项（589 测试 0 失败 + 135 合同产物）、**B 级** 三语言 85 项（预构建产物、`noCargoAllowed`、自述不构成 SourceFinal/canonical/mainCI）。129/147 有执行型证据。**不得把 B 级当 A 级用**。 |
| `validation-stage-main747-r01` | 含网络前置实测：GitHub 443 超时（rc=28），`index.crates.io`/`api.github.com` 可用，`git@github.com` SSH 可认证。 |

## 三、CLI 与分支拓扑

| 目录 | 结论 |
|---|---|
| `validation-stage-branch-topology-20261104-r01` | Framework/CLI/SDK 的 `main` 即 origin；CLI `main` 自身领先 3／落后 2；website `main` = origin。三个任务分支同名 `chore/Echoyue/root-workspace-convergence`。 |
| `validation-stage-cli-baseline-content-20261104-r01` | 5 个分叉提交的内容分支上都有（逐项列出符号与测试）；tokio 为 **1.53.3**；本地 main 的 3 个提交与上游 #13/#14 重复。 |
| `validation-stage-cli-controls8-20261104-r01` | CLI 当前 cut 的 controls 8/8 自然 0。 |

## 四、交付包身份与构建链

| 目录 | 结论 |
|---|---|
| `validation-stage-app-source-identity-20261104-r01` | 清单 `6abe867d…`、3946 条、两仓 HEAD 匹配、CLI 脏状态 2353 字节逐字节相同。 |
| `validation-stage-app-identity-recheck2-20261104-r01` | 二次复核：3946/3946 字节相同。 |
| `validation-stage-app-bundle-identity-20261104-r01` | 三个 `Eclip.app` 主二进制各异：`abdd3fac…`(176,890,208 B)=当前交付；**`latest-…` 目录里是旧 `c5488208…`(176,894,784 B)，缺 SSH 重连入口**。 |
| `validation-stage-launch-chain-check-20261104-r01` | 启动脚本可执行、钉的 `ready.json` 哈希 `17f949cb…` 重算一致、`codesign --deep --strict` 退出 0（adhoc，仅本机）。 |
| `validation-stage-bundle-fix-content-20261104-r01` | 构建日志第 3/6 行证明跑了 `npm run build:tauri`，第 159 行产出 `index-DskFRksZ.js`（534.45 kB），该文件 grep 命中三处修复文案。**工作树 `dist` 不是交付所用产物**。 |
| `validation-stage-three-fixes-static-20261104-r01` | 三项修复均在源码：温度全路径 `None`、C02 默认位置、C04 重连提示。 |

## 五、验收文档核对与修正

| 目录 | 结论 |
|---|---|
| `validation-stage-checklist-label-check-20261104-r01` | 79 项/13 组；28 条引用字符串 22 条直接命中，6 条零命中且均有解释；无指向已消失控件的条目。 |
| `validation-stage-first4-doc-claims-20261104-r01` | 四项用例 10 条代码断言**全部成立**。 |
| `validation-stage-c02-subitems-20261104-r01` | C02 三个小项均有代码支撑；观察：重复目录错误串为英文，中英混排，未改。 |
| ✎ `validation-stage-a06-fields-correction-20261104-r01` | A06 原写"推理深度/输出上限/上下文三项"有误：实际只有 `Max tokens`、`Context window` **两项**（英文标签），**无推理深度字段**。 |
| ⚠ `validation-stage-c04-prerequisite-20261104-r01` | 先误判"该 SSH 工作空间不存在"，后续更正：启动脚本经 `EKO_DATA_DIR` 指向 `isolated-runtime/data`，那里**有且仅有** `ubuntu-ssh-test-20261102`。用脚本启动即可照原文做。 |
| `validation-stage-ssh-credential-boundary-20261104-r01` | C04 凭据断言成立且更强：`TerminalTarget::Ssh{host,user,port}` **结构无密码字段**，认证委托系统 ssh，`project_ssh.rs` 1902 行**零**日志调用。 |
| `validation-stage-gui-prereq-inventory-20261104-r01` | 隔离数据根：会话 9、run 17、任务 7；`channels` 仅 weixin；`plugins` 仅 preferences.json（无插件）。A/C 本地/E/F/L 可做；I/J 的 QQ 飞书、M 不可做。 |
| `validation-stage-steer-static-20261104-r01` | Steer 沿主模型历史理解：`stream_channel.rs:986 drain_steer_into_context` 把 steer 推入同一 `ContextManager`；无独立解释器/A-B/分类器。真实矩阵仍缺。 |

## 六、其它

| 目录 | 结论 |
|---|---|
| `validation-stage-evidence-integrity-20261104-r01` | 38,143 条引用、38,131 一致、**0 条未回执漂移**；12 条悬挂全是被官方 clean 删除的 target 产物。 |
| `validation-stage-static-rule-scan-20261104-r01` | 静态硬约束扫描结果。 |

---

## 本会话两次自我更正的共同点

- 第 24 轮：把 `git ls-tree` 的转义输出当文件系统路径 → 误报"缺 9 份文档"。
- 第 30／33 轮：`codesign … | head` 后取 `$?`（拿到 `head` 的）、只查默认数据根就断定工作空间不存在。

三者同属**把中间输出或默认路径当成事实**。回执里都写了根因，不只是改数字。

## 这些回执没有被任何东西自动校验

`proof-index.json` 只覆盖 30 个交付与文档条目，**不覆盖这 32 个 validation-stage 回执**。若要复用，靠本索引导航，不要假定它们仍与当前源码一致——第 24 轮已经证明过"回执里的旧结论会被后来的实测推翻"。
