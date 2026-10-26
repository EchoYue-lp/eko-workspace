# 工作区收敛与恢复清单

## 2026-10-26 两处默认 target 接管核对

本轮只读核查 CLI 与 framework 默认 target，未执行 clean、压缩、去重或提交。准确 cwd/manifest/target 参数见[clean 二元清单](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/default-target-handoff-20261026/clean-pair.json)，完整依据见[接管说明](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/default-target-handoff-20261026/接管说明.md)。协调者可依据当前缓存内容、消费者与制品保全证据接管官方 clean，执行前仍须在串行窗口再检查无新使用者及漂移；本线不执行，无需重新征求人类许可。

| 精确默认 target | 原分配空间 | 接管依据与限制 |
| --- | ---: | --- |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/target` | 631,898,112 bytes，约 0.589 GiB | 3,937 文件；原生辅助文件 165 个均为 build-script/proc-macro，产品/测试执行原件 0；无当前消费者及两次 metadata 漂移 |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/target` | 180,080,640 bytes，约 0.168 GiB | 1,883 文件；原生辅助文件 90 个均为 build-script/proc-macro，产品/测试执行原件 0；无当前消费者及两次 metadata 漂移 |

两项共 **811,978,752 bytes，约 0.756 GiB**，未计为已回收。逐文件哈希清单、进程观察及 target 外保全在独立证据目录。13 份 GUI/core 必要二进制均在 target 外，重新回读 SHA 与保存回执匹配；51 个保留可执行文件的 `otool -L` 全部退出 0，动态依赖没有指向两默认 target。PID43779、53333 和 core epoch10 专用 target 继续保护。

来源仍有明确限制：Cargo 缓存与文件时间指向 2026-10-24 17 时和 2026-10-26 14 时，尚未绑定精确 producer 命令及原批次出口；**不声明原批次已通过**，不借 GUI/core 专用 target 的 clean0 充当默认 target 的历史回执。接管依据是当前缓存均可再生、没有唯一产品/测试原件、无消费者且必要制品已保全；历史归因缺口原样保留，实际 clean 由主线程后继取证。

## 2026-10-26 17:59 根内保留制品与 target 只读盘点

本轮仅盘点，**未删除、压缩、去重或执行 Cargo/clean**。累计 51 个 worktree 回收沿用原证据，没有重删、重做恢复包或反复归档 host；源码、新交付文档、refs/stash/Exa/tag 均未由本线变更。完整清单在[制品盘点报告](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/retained-artifacts-audit-20261026/盘点报告.md)，机器证据在[最终观察切点清单](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/retained-artifacts-audit-20261026/inventory-final.json)。

- 发现 **127 个文件、7 个 target**：其中 117 项为本维护范围的具名制品/恢复包/fixture，另 10 项是 ZCode 随附搜索依赖包，仅记录为范围外保护，不纳入维护候选。PID43779 仍使用 GUI epoch10 原件，SHA `b330bf25ce33a0eab716673ee498a23acbc66bc64173ed20142b9b43cfcd7ae2`；PID53333 的 fixture 入口 SHA 为 `b81099a4be23b6e9c0c6ec7123ac366ea9189d40bce4540c9c8445d80589c465`，当前服务的前端路径包含 `settings-layout-20261026/artifacts/frontend-dist`。这些活动入口及其前端/配置继续保护。
- core epoch10 专用 target 切点约 **7.042 GiB**；初检由 cargo58421/rustc58435 使用，后检即使暂未见这些进程，也不能认定 owner 已确认批次结束。扫描期间同名测试二进制被重链接，后继 target 与新 `retained-native-control-box-v1` 的精 SHA 均为 `cb9c295b640ebd443df3a0b0cbcaa0c5f5dca09c0ec3c74c423e558bb9e39cad`。旧 `a73c5e…` 只属于此前 target 观察及仍保留的 renderer 版本，不冒充整棵活动目录的固定身份。
- 另外发现 CLI 默认 target **631,898,112 bytes**、framework 默认 target **180,080,640 bytes**，均未观察到活动使用者，但没有本次 owner 完成/停用回执，标为归属待确认。三处 target 为零字节 regular-file 占用，另有旧 Java target 36,864 bytes；无未知活动 target 的实证，不据未观察到进程自动清理。
- 共有 **9 组相同 SHA/大小/mode**：1 组涉及进行中 core target，保护；8 组为静态重复候选，逻辑冗余上界 **992,786,764 bytes（0.925 GiB）**。其中两组是既有源码恢复包，继续保留原 source 保护边界；本次只报告身份重复。这不是已回收量，APFS 物理收益未知；只有父线程确认运行及回执恢复依赖后，才可保持所有路径、原字节与 mode 做精确无损去重。
- GUI epoch09/10 两个二进制 SHA 不同；core epoch09 六份及 epoch10 三份 target 外保留制品也各有不同 SHA。未观察到使用者的历史版本可列无损压缩候选，不能按同名或较早 epoch 当作过期垃圾。完整 SHA、大小、引用回执及恢复条件逐项列于报告。
- 本次检索 28,265 个小型 JSON/JSONL/Markdown 的引用位置；再读上轮 58 份成员索引的 289,892 行，没有命中本次制品 SHA。未解压、未重复冷验证；不能因为上轮已归档就认为这些 root-local 原件已有恢复覆盖。旧恢复包 SHA 明确引用既有通过回执，本次未重算其内容；其它列明制品为本次流式 SHA。

初版引用正则扫描开销过大，仅中断本线自有只读扫描进程，原 exit130/log 保留；后继线性解析复用已完成 SHA 后成功，不重复读取大制品。当前切点仍允许 GUI/core 作者继续工作，任何后续动作前须重新核验身份与使用者；本报告不授予删除或压缩权限。

下一维护动作：父线程与对应产物 owner 核对八组静态重复及停止使用版本的回执/恢复依赖，再对精确对象确认动作。本线没有 Cargo 产物，不执行形式 clean；本次不复核原生验收，后文 SSH 未通过说明保留其当时切点，当前原生状态以 GUI 所属最新回执为准。

## 2026-10-26 原生验收状态补充（历史切点）

据总调度本轮同步，用户已确认原生界面布局方向符合参考；该接受范围仅覆盖布局方向。**SSH 真实交互验收仍未通过**：同一 `ubuntu@111.229.245.175` 账号使用密码可在系统终端成功连接，但 EKO 仍停在 `password` 提示，目录验证也未通过。GUI 线已进入调试修复；修复与重新验收结果尚待该线提供，不将布局已接受扩大为 SSH 登录、目录访问或整体交互通过。

本次仅补记验收状态，不重新验证本线已完成的工作区回收，不启动 Cargo，也不改写原始日志或历史证据。后续新增或修改的文档说明统一使用中文；固定代码、命令、API/协议标识和原始日志保持原文。

## 2026-10-26 累计结果：51 个副本已回收

两批累计通过原生 `git worktree remove` 删除 **51/51 个已核验本地旧副本**；51 次实际命令均退出 0。逐个互不嵌套目标原分配空间合计 **23,881,756,672 bytes（22.24 GiB）**：外部 41 项 8.52 GiB，根内旧 `.worktrees` 10 项 13.73 GiB。内 10 项新增恢复档案 **6,280,105,589 bytes（5.85 GiB）**；归档成本与删除路径原占用分别报告，`df` 另受根内两条并行 focused 构建影响，不据此推算独占净回收空间。

| 当前注册工作树 | 数量 | 处理 |
| --- | ---: | --- |
| primary：主根、framework、CLI、SDK、website | 5 | 保留；合法未提交成果继续由各线维护 |
| Codex host 管理路径 | 4 | 保留恢复/身份；旧设计附件 queued 未确认，fbc4 保护 |
| 本地外部旧副本 | 41 | 已删除 |
| 根内旧 `.worktrees` 副本 | 10 | 已删除 |

累计后检剩余五仓注册数为 **3 / 2 / 2 / 1 / 1**，共 **9 = primary 5 + host 4**。315 个当前 named refs（原 314 加总调度精确授权新增 SDK active 分支）、全部 **65 个 branch、18 条 stash、123 个 tag** 保持；Exa、新 root 分支与 retirement 锚点未删改。primary 的采集切点 **33 个未跟踪文件** 全部属于有效源码、测试、ADR 或工程材料，未删除、未 stage。分类不是“垃圾判定”，主根后续新增成果也继续保留。

- [累计后检：51 删除、9 保留、refs/stash/tag、档案与进程](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/internal-reclaim/postcheck.json)，实际退出 0、failures 为空。
- [内 10 项原始逐对象回执](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/internal-reclaim/remove-receipts.jsonl)、[汇总](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/internal-reclaim/removal-summary.json)、[精确计划](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/internal-reclaim/execution-plan.json)。
- [65 branches / 18 stashes / 123 tags / 33 primary 未跟踪文件逐项分类](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/internal-reclaim/root-object-classification.json)。所有对象保持，没有 drop/pop stash 或删除 refs；34 条历史已包含分支与 10 条有唯一历史分支的进一步功能价值对账继续保留，不以 bundle 可恢复代替删除依据。
- [内 10 项统一恢复入口](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/internal-reclaim/RESTORE.md)。复用原 source checkpoint 与五仓 bundle，**没有新建或重复冷解包 bundle**。根内十份主档案加 vendor 源码补档共 **189,166 成员、32,792,247,609 逻辑 bytes**，内容与 mode 均经流式 tar 读回；最终只对这些新档案作 SHA 后检，没有重复解包旧原件。

根内 source 与原 checkpoint 对账零漂移，224 份 Cargo manifest、14 份根 Cargo config、源 symlink 和 lsof/ps 检查无保留消费者或活动使用者；三个内部配对按 CLI consumer 先、framework supplier 后删除。每对象再次核验 source、ignored hash、registration/common-dir/lock 与进程。target、node_modules、venv 等可再生缓存未当必要证据全量保存；23 个历史 vendor 源码目录另行补档，11,521 成员逐根覆盖，避免仅按目录名误删源码。

九个稀疏测试文件约 **18 GiB 逻辑大小、823,296 bytes 原分配空间**，原长度、分配块、mode、稀疏属性及 SHA 均保留。压缩和验证全程流式，未 `extractall` 或实体展开原大文件。[稀疏恢复工具](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/internal-reclaim/restore_sparse.py) 按零块 seek/非零块 write/精确 truncate/校验 SHA 恢复；64 MiB 有界合成夹具已验证保持稀疏，恢复分配约 1.00 MiB。原 8 MiB 夹具在本机本身不稀疏的失败与尺寸对照诊断保留，不冒充原大文件恢复或 native 验收。

本线没有运行 Rust/前端构建，没有自有 target 需 `cargo clean`；完整矩阵、性能、native GUI/SSH/微信与 main CI 均不在本维护证据内。仅维护恢复材料与本清单，未修改产品代码或正式产品文档、examples、官网，未 stage/commit/push/merge/关闭 Issue/发布/部署。

下一项维护动作仍为总调度核实旧设计附件 queued 归档结果并识别 fbc4 所属宿主附件。外部上级空目录/旧 sibling symlink 与根内空 `.worktrees` 容器未递归删除；它们不是原生 Git 回收目标，不能把 51 个副本已退出表述为所有父目录已清空。

---

## 第一批：本地外部 41 项（15:18 后检切点）

本线已用原生 `git worktree remove` 回收 **41/41 个本地非 host 外部 worktree**；41 次命令实际退出 0，其中 23 个已归档 dirty 目标使用精确 `--force`。逐路径 `du` 分配空间合计 **9,144,037,376 bytes（8.52 GiB）**；同批磁盘 free 观察增加 8.23 GiB，受并行根构建影响，两者不混同。

| 仓库 | 删除 worktree | 删除路径原分配空间 |
| --- | ---: | ---: |
| echo-agent | 17 | 1,667,846,144 bytes |
| echo-agent-cli | 19 | 6,069,428,224 bytes |
| echo-agent-sdk | 2 | 1,104,166,912 bytes |
| echo-website | 1 | 244,613,120 bytes |
| eko-workspace | 2 | 57,982,976 bytes |

清理前 source 295 项与 ignored 110 项 SHA 全部匹配；47 份 tar 的 100,726 个成员内容及 mode 重新独立读取通过；45 个冻结外部源无漂移，FIFO 仅核对元数据、未打开。721 份 Cargo manifest、52 份 Cargo config 与源 symlink 检查未发现保留工作区指向删除目标；17 组跨 worktree 依赖按消费者先、框架后执行。每对象删除前再次核验 source/ignored 哈希、Git common-dir/registration/lock、lsof/ps 和 snapshot。

- [逐对象删除原始回执](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/reclaim-execution/remove-receipts.jsonl)、[汇总](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/reclaim-execution/removal-summary.json)、[精确目标与执行顺序](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/reclaim-execution/execution-plan.json)。
- [最终后检](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/reclaim-execution/postcheck.json) 于 15:18:27 实际退出 0：剩余 **19 个注册 worktree = 根内 15 + host 4**。原 314 条 named refs、全部 18 条 stash 和五仓 common-dir 保持。15 个根内对象及 4 个 host 路径均保留；冻结保留对象的 HEAD/index/patch/非忽略源文件不变，主根合法并行编辑单独标明。
- SDK 唯一新增 active 分支 `chore/Echoyue/root-workspace-convergence@0930eef9…` 由总调度在 15:16:15 创建、用户直接确认，HEAD 未变。本线未新增/删除 refs。[精确授权增量](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/reclaim-execution/authorized-primary-ref-delta.json) 不扩大到其它 refs。
- 两个诊断失败均原样保留：首次独立对象命令误把 object directory 当作 git-dir，实际 exit128；纠正 `GIT_OBJECT_DIRECTORY` 后五仓独立读取通过。首次后检因 SDK 新增分支实际 exit1，保存在 `postcheck-before-sdk-adjudication.*`；核验用户授权、只加入该精确身份后重跑实际 exit0。文档生成的一次 stdin 编码错误未产生写入，后改用文件补丁完成。

### Host 保护对象

以下四个注册路径未手动移除。旧设计附件的宿主归档请求已提交，但工具仅返回 queued，磁盘与注册仍存在；**结果未确认，不能宣称已归档**。`fbc4` 不属于总调度当前附件，继续保护。Git 管理目录及 common-dir 逐项保存于最终后检。

| ID | 精确路径 | Git worktree 管理项 | 状态 |
| --- | --- | --- | --- |
| eko-workspace-02 | `/Users/ls/.codex/worktrees/eko-gui-product-convergence/lp-agent` | 根 `.git/worktrees/lp-agent1` | 归档请求已提交，结果未确认 |
| eko-workspace-03 | `/Users/ls/.codex/worktrees/fbc4/lp-agent` | 根 `.git/worktrees/lp-agent` | fbc4：host 归属保护 |
| echo-agent-02 | `/Users/ls/.codex/worktrees/fbc4/lp-agent/echo-agent` | framework `.git/worktrees/echo-agent1` | fbc4：host 归属保护 |
| echo-agent-cli-02 | `/Users/ls/.codex/worktrees/fbc4/lp-agent/echo-agent-cli` | CLI `.git/worktrees/echo-agent-cli` | fbc4：host 归属保护 |

### 分支分类与保留

初检 64 条 local branch；总调度后续新增 1 条 active SDK 分支，最终 **65 条均保留**。没有 fetch、branch delete、checkout、stage、commit、push、merge、Issue 关闭、发布或部署。分类基于本地 main/active-root 历史和当前注册，不能据此宣称远端 main 最新；unique history 也不等于功能未被 squash 交付。

| 分类 | 数量 | 本次动作 |
| --- | ---: | --- |
| main / active / Exa / full-capabilities / retirement 保护 | 18 | 保留 |
| 根内保留 worktree 当前持有 | 3 | 保留 |
| 历史已经包含于本地 main 或当前 root | 34 | 依最新“不改 refs”指令保留 |
| 有唯一历史，功能价值需逐项对照 | 10 | 保留，不能仅凭 bundle 可恢复而删除 |

[65 条逐分支名称、SHA、用途摘要、包含关系和保留理由](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/reclaim-execution/branch-classification-final.json)。全部 tags、Exa `4343f0b7…` 与 pre-retirement refs 保留。

### 恢复与后续边界

统一恢复入口：[RESTORE.md](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/reclaim-execution/RESTORE.md)。源码和 Git 历史使用原 `archive-inventory/`；ignored 证据与二进制使用 `ignored-evidence/` 的成员索引。两个旧 checkpoint、清单与 SHA 均未改写。已删除路径仅用于恢复定位，不能将归档里的旧通过结果当作当前根已通过。

外部上级目录未递归删除：仍有 33 个空 task parent 和 8 条失效的 sibling `echo-agent` symlink；它们不是本次 41 个已注册目标，原位保留并记录在[残留精确清单](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/reclaim-execution/external-parent-remainders.json)。本线完成的是本地外部注册 worktree 回收，不声称整个外部父目录已清空。

下一项维护动作：总调度核实旧设计附件 queued 归档结果，并识别 fbc4 的所属宿主附件；不以手工删除绕过宿主生命周期。随后可按精确清单安排空 parent/失效 symlink 的独立清理，以及有唯一历史分支的功能价值对账。

验证范围是 focused 工程维护；本线未运行 Cargo/前端构建，没有自有 target 需要 cargo clean。完整矩阵、性能、native GUI/SSH/微信和 main CI 均未由本线验证；根内并行运行结果由相应交付线负责。运行时代码、正式文档、examples、官网没有由本线修改，因此无需同步其产品说明。

---

## 以下为 14:42 备份切点的历史清单

**下文“保留/未删除/ignored 未复制”等描述仅代表源备份完成时的历史状态。当前删除、保留与补齐 ignored 归档后的状态以上文实际回收结果及逐项回执为准。**

备份切点：2026-10-26T14:42:33+0800。完成：2026-10-26T14:43:43+0800。最终只读保全复核：2026-10-26T14:46:45+0800。

唯一开发根为 `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent`；其它 worktree 已冻结，只读采集。本文和指定根内 `archive-inventory/` 是本批次唯一写入位置。未删除/归档 worktree，未切换 HEAD、修改原仓 index/refs、stash 或清理证据。主代理在根内继续开发，本批次不回退或覆盖其修改。

本批次覆盖顶层 superproject 和四个独立子仓库的全部 **60 个已注册 worktree**。保存 120 份 worktree/staged binary patch、60 个非忽略源文件归档和逐项回执；138 个非忽略源文件已逐文件 SHA-256 回读一致。仅排除 5 个 `__pycache__/*.pyc` 缓存文件，详细路径在清单中。

5 份 Git bundle 保存全部分支/tag/其它 refs、全部 18 条 stash reflog 条目以及 37 个唯一 linked/detached HEAD。`git bundle verify` 全部退出 0；再从 bundle 解包到备份目录内独立对象目录，未借用原仓 objects，314 个 refs 对象、所有 stash 和 worktree HEAD 读取全部通过。

**ignored 证据只测量和列出，未复制。全部原 worktree 与 ignored 证据继续保留，本批备份不能作为删除它们的批准或完整证据归档。** `target`、`node_modules`、`vendor`、缓存及依赖目录未复制。

最终复核冻结 worktree 的 HEAD/status/patch/源文件：外部漂移为零；5 仓 refs/stash 均未变。主根 CLI 在备份后由主代理继续开发，检测到正常 worktree patch 漂移，因此本归档代表备份切点，不声称覆盖后续修改。

## 入口与空间估算

- [完整 worktree/状态/非忽略源文件/ignored 目录初始清单](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/inventory-estimate.json)。
- [344 条 ignored 紧凑条目的空间与保留清单](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/ignored-space-inventory.json)：`du` 零失败。
- [备份回执](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/backup-receipt.json)、[独立 bundle 恢复对象核验](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/independent-bundle-verification.json)、[最终源与 refs 保全复核](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/final-source-preservation-check.json)。
- [SHA-256 总清单](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/SHA256SUMS)、[恢复说明](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/RESTORE.md)。

备份前可用 43.03 GiB；源文件估算 2.00 MiB，Git 对象 bundle 保守上界 328.25 MiB。实际 bundle 合计 67.12 MiB；patch 合计 4.99 MiB，源文件压缩归档合计 565.53 KiB。

空间测量是逐路径的 allocated size；顶层、嵌套 worktree 和 ignored 路径可能重叠，禁止将下表空间直接相加。独立对象目录只用于验证 bundle 自足性，写入也全部位于根内指定归档目录。

## 每仓汇总

| 仓库 | Worktree | dirty | tracked 改动 | untracked | stash 条目 | bundle 大小 | verify |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| eko-workspace | 5 | 3 | 3 | 18 | 0 | 7.65 MiB | 0 / 独立对象 PASS |
| echo-agent | 24 | 5 | 83 | 21 | 2 | 19.03 MiB | 0 / 独立对象 PASS |
| echo-agent-cli | 24 | 19 | 494 | 100 | 15 | 9.79 MiB | 0 / 独立对象 PASS |
| echo-agent-sdk | 4 | 1 | 0 | 4 | 1 | 28.30 MiB | 0 / 独立对象 PASS |
| echo-website | 3 | 0 | 0 | 0 | 0 | 2.35 MiB | 0 / 独立对象 PASS |

## 每个 worktree 的冻结快照

下表 dirty/未跟踪计数是备份切点的真实状态；完整状态路径、index SHA、patch SHA、未跟踪文件 SHA 和排除项均可从对应备份回执恢复。ignored 列给出条目数与证据目录大小，逐条完整大小见 ignored-space-inventory.json。

### eko-workspace

| 路径与备份 | HEAD / branch | tracked / untracked | ignored 证据与空间 | 总占用（含嵌套） |
| --- | --- | ---: | --- | ---: |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent`<br>[eko-workspace-01](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/eko-workspace-01/receipt.json) | `a91d5deb8372a37388cf4e6106885345d073c14c`<br>`chore/Echoyue/workspace-maintenance` | 2 / 17 | 28 条；`.echo-semantic/local` 16.92 MiB（保留）; `.supreme` 0.00 B（保留） | 23.34 GiB |
| `/Users/ls/.codex/worktrees/eko-gui-product-convergence/lp-agent`<br>[eko-workspace-02](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/eko-workspace-02/receipt.json) | `6688c6251d00cb068a7045513562e92b469a33ca`<br>`doc/Echoyue/eko-gui-product-convergence` | 1 / 0 | 4 条；`.echo-semantic/local` 140.01 MiB（保留） | 165.29 MiB |
| `/Users/ls/.codex/worktrees/fbc4/lp-agent`<br>[eko-workspace-03](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/eko-workspace-03/receipt.json) | `5459c20da6045ff967d9779d59550417cb599400`<br>`(detached)` | 0 / 0 | 4 条；未发现具名证据目录 | 331.86 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/personal-wechat-channel-docs/lp-agent`<br>[eko-workspace-04](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/eko-workspace-04/receipt.json) | `d1848d729d8ef084d6d9bf1e6897c7dec98a21d7`<br>`doc/Echoyue/personal-wechat-channel` | 0 / 0 | 3 条；`.echo-semantic/local` 4.79 MiB（保留） | 30.04 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/steer-responsiveness-docs/lp-agent`<br>[eko-workspace-05](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/eko-workspace-05/receipt.json) | `7d47e3f6c3ecc763e229fe2a2d77f57ede75d9bb`<br>`doc/Echoyue/steer-responsiveness` | 0 / 1 | 4 条；`.echo-semantic/local` 72.00 KiB（保留） | 25.26 MiB |

### echo-agent

| 路径与备份 | HEAD / branch | tracked / untracked | ignored 证据与空间 | 总占用（含嵌套） |
| --- | --- | ---: | --- | ---: |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent`<br>[echo-agent-01](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-01/receipt.json) | `9c5e202dcbd20a7de3c017a3ee8adada3e400b86`<br>`chore/Echoyue/root-workspace-convergence` | 0 / 0 | 47 条；`.supreme` 9.34 MiB（保留） | 407.21 MiB |
| `/Users/ls/.codex/worktrees/fbc4/lp-agent/echo-agent`<br>[echo-agent-02](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-02/receipt.json) | `a8c2d1ae3fce675633ea20d4e3d49258c327f634`<br>`(detached)` | 0 / 0 | 1 条；未发现具名证据目录 | 18.44 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/a2a-lsp-retirement/echo-agent`<br>[echo-agent-03](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-03/receipt.json) | `0a18157140b1212c06db38ae5edbe4326381c891`<br>`refactor/Echoyue/a2a-lsp-retirement` | 58 / 2 | 3 条；`.echo-semantic/local` 66.52 MiB（保留） | 109.14 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/domain-plugin-runtime/echo-agent`<br>[echo-agent-04](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-04/receipt.json) | `8cd7148745d014733e02e3c54f56069c52437027`<br>`feature/Echoyue/framework-tool-domain` | 0 / 0 | 3 条；`.echo-semantic/local` 17.97 MiB（保留） | 61.88 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/gui-runtime-framework-integration/echo-agent`<br>[echo-agent-05](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-05/receipt.json) | `0a18157140b1212c06db38ae5edbe4326381c891`<br>`feature/Echoyue/gui-runtime-framework-integration` | 0 / 0 | 3 条；`.echo-semantic/local` 25.68 MiB（保留） | 69.43 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/gui-subagent-depth-policy/echo-agent`<br>[echo-agent-06](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-06/receipt.json) | `e4e50396876c5e7e85b52cdc3dbb510bab9c4f4d`<br>`(detached)` | 0 / 0 | 0 条；未发现具名证据目录 | 42.51 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/gui-visual-integration/echo-agent`<br>[echo-agent-07](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-07/receipt.json) | `c3b56b4ee8a73565e6a2a621b9a7ddfb644ed3aa`<br>`(detached)` | 0 / 0 | 0 条；未发现具名证据目录 | 42.48 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/im-channel-control/echo-agent`<br>[echo-agent-08](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-08/receipt.json) | `edbd7b0a5304f8410e7059900468db5de0c7ae4c`<br>`feature/Echoyue/channel-control-observations` | 0 / 0 | 4 条；`.echo-semantic/local` 13.44 MiB（保留） | 56.95 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/im-cron-framework-integration/echo-agent`<br>[echo-agent-09](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-09/receipt.json) | `634a85487e2e0af15e1bc7c9b5e55b1498922fce`<br>`feature/Echoyue/im-cron-gui-control` | 0 / 0 | 3 条；`.echo-semantic/local` 19.06 MiB（保留） | 62.65 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/im-cron-gui-control/echo-agent`<br>[echo-agent-10](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-10/receipt.json) | `634a85487e2e0af15e1bc7c9b5e55b1498922fce`<br>`(detached)` | 0 / 0 | 0 条；未发现具名证据目录 | 42.67 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/lsp-frozen-diagnostics/echo-agent`<br>[echo-agent-11](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-11/receipt.json) | `e4e50396876c5e7e85b52cdc3dbb510bab9c4f4d`<br>`fix/Echoyue/lsp-frozen-diagnostics` | 0 / 0 | 3 条；`.echo-semantic/local` 41.71 MiB（保留） | 85.10 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/performance-baseline-native-a/echo-agent`<br>[echo-agent-12](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-12/receipt.json) | `c3b56b4ee8a73565e6a2a621b9a7ddfb644ed3aa`<br>`test/Echoyue/performance-baseline-native-a` | 1 / 5 | 3 条；`.echo-semantic/local` 0.00 B（保留） | 42.70 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/performance-baseline/echo-agent`<br>[echo-agent-13](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-13/receipt.json) | `c3b56b4ee8a73565e6a2a621b9a7ddfb644ed3aa`<br>`test/Echoyue/performance-baseline` | 1 / 5 | 3 条；`.echo-semantic/local` 52.84 MiB（保留） | 95.54 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/personal-wechat-channel-v2/echo-agent`<br>[echo-agent-14](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-14/receipt.json) | `83b1bedd28ec70f745d8cdb13cea62aa2a802167`<br>`test/Echoyue/weixin-cross-crate-fixture` | 0 / 0 | 3 条；`.echo-semantic/local` 26.43 MiB（保留） | 70.79 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/personal-wechat-channel/echo-agent`<br>[echo-agent-15](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-15/receipt.json) | `8cd7148745d014733e02e3c54f56069c52437027`<br>`feature/Echoyue/personal-wechat` | 7 / 8 | 3 条；`.echo-semantic/local` 172.00 KiB（保留） | 43.43 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/project-workspaces/echo-agent`<br>[echo-agent-16](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-16/receipt.json) | `9c5e202dcbd20a7de3c017a3ee8adada3e400b86`<br>`feature/Echoyue/project-filesystem-adapter` | 0 / 0 | 3 条；`.echo-semantic/local` 28.93 MiB（保留） | 73.28 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/ssh-remote-terminal/echo-agent`<br>[echo-agent-17](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-17/receipt.json) | `e4e50396876c5e7e85b52cdc3dbb510bab9c4f4d`<br>`(detached)` | 0 / 0 | 0 条；未发现具名证据目录 | 42.51 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/steer-safe-points/echo-agent`<br>[echo-agent-18](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-18/receipt.json) | `a1a30f1264eac7a04ea198714801055c3a4ad806`<br>`fix/Echoyue/tool-schema-budget` | 0 / 0 | 3 条；`.echo-semantic/local` 560.37 MiB（保留） | 604.71 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/tool-output-budget/echo-agent`<br>[echo-agent-19](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-19/receipt.json) | `0a18157140b1212c06db38ae5edbe4326381c891`<br>`fix/Echoyue/tool-output-budget` | 16 / 1 | 3 条；`.echo-semantic/local` 1.78 MiB（保留） | 44.80 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.worktrees/echo-agent-cli-worktrees/echo-agent`<br>[echo-agent-20](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-20/receipt.json) | `6563d26570cc4a8d93c86b7e37443b0559476807`<br>`(detached)` | 0 / 0 | 0 条；未发现具名证据目录 | 42.05 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.worktrees/echo-agent-cli-worktrees/framework-main-consumer/echo-agent`<br>[echo-agent-21](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-21/receipt.json) | `f7bac62eee04a963f92a3e4308b6ea2336e91cc0`<br>`(detached)` | 0 / 0 | 0 条；未发现具名证据目录 | 42.43 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.worktrees/echo-agent-worktrees/configured-builder-client-pair`<br>[echo-agent-22](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-22/receipt.json) | `f7bac62eee04a963f92a3e4308b6ea2336e91cc0`<br>`(detached)` | 0 / 0 | 2 条；`.echo-semantic/local` 2.82 GiB（保留） | 2.86 GiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.worktrees/echo-agent-worktrees/task-runtime-store-control`<br>[echo-agent-23](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-23/receipt.json) | `3560812caede84998a1f6f26d7bf3fa81ea0486c`<br>`(detached)` | 0 / 0 | 2 条；`.echo-semantic/local` 3.26 GiB（保留） | 3.30 GiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/.worktrees/echo-agent`<br>[echo-agent-24](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-24/receipt.json) | `f7bac62eee04a963f92a3e4308b6ea2336e91cc0`<br>`(detached)` | 0 / 0 | 0 条；未发现具名证据目录 | 42.43 MiB |

### echo-agent-cli

| 路径与备份 | HEAD / branch | tracked / untracked | ignored 证据与空间 | 总占用（含嵌套） |
| --- | --- | ---: | --- | ---: |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli`<br>[echo-agent-cli-01](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-cli-01/receipt.json) | `780ddd0d25c2eac5d1471cb3e5012c9d2f6b3cf9`<br>`chore/Echoyue/root-workspace-convergence` | 79 / 12 | 28 条；`.echo-semantic/local` 5.78 GiB（保留）; `.supreme` 0.00 B（保留） | 6.76 GiB |
| `/Users/ls/.codex/worktrees/fbc4/lp-agent/echo-agent-cli`<br>[echo-agent-cli-02](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-cli-02/receipt.json) | `40fd80447cdf3d7627399af0b2069a8b4da5327a`<br>`(detached)` | 0 / 0 | 6 条；未发现具名证据目录 | 289.07 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/console-gui-decoupling/echo-agent-cli`<br>[echo-agent-cli-03](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-cli-03/receipt.json) | `28bb9459bde9872b8d36cdd4a6eb98929feb0927`<br>`refactor/Echoyue/console-gui-decoupling` | 8 / 0 | 3 条；`.echo-semantic/local` 200.00 KiB（保留） | 16.34 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/domain-plugin-runtime/echo-agent-cli`<br>[echo-agent-cli-04](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-cli-04/receipt.json) | `01137a1e9186fb6c9d9b677e72b3fe4cc0b99a3b`<br>`feature/Echoyue/domain-plugin-runtime` | 80 / 10 | 3 条；`.echo-semantic/local` 630.17 MiB（保留） | 647.21 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/framework-context-defaults/echo-agent-cli`<br>[echo-agent-cli-05](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-cli-05/receipt.json) | `4ab0de9fcc071bc9b390732737f9cfc09176c4f9`<br>`fix/Echoyue/framework-context-defaults` | 3 / 0 | 3 条；`.echo-semantic/local` 668.00 KiB（保留） | 16.66 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/gui-runtime-app-integration/echo-agent-cli`<br>[echo-agent-cli-06](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-cli-06/receipt.json) | `01137a1e9186fb6c9d9b677e72b3fe4cc0b99a3b`<br>`refactor/Echoyue/console-surface-retirement` | 0 / 2 | 8 条；`.echo-semantic/local` 1.10 GiB（保留） | 1.12 GiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/gui-subagent-depth-policy/echo-agent-cli`<br>[echo-agent-cli-07](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-cli-07/receipt.json) | `2b52251f1bb43d5844c1749028bd188cf6899eca`<br>`feature/Echoyue/gui-subagent-depth-policy` | 41 / 4 | 5 条；`.echo-semantic/local` 1.65 MiB（保留） | 21.02 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/gui-visual-integration/echo-agent-cli`<br>[echo-agent-cli-08](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-cli-08/receipt.json) | `db5010ed888cf50f2400fbba31075ab63c5626e9`<br>`feature/Echoyue/gui-visual-integration` | 0 / 0 | 6 条；`.echo-semantic/local` 13.84 MiB（保留） | 302.96 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/hitl-request-identity-validation/echo-agent-cli`<br>[echo-agent-cli-09](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-cli-09/receipt.json) | `d7fc64fd85bd23a9c9958b41be90ce24c511d4ee`<br>`test/Echoyue/hitl-request-identity-validation` | 26 / 3 | 3 条；`.echo-semantic/local` 5.23 MiB（保留） | 22.43 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/im-cron-gui-control/echo-agent-cli`<br>[echo-agent-cli-10](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-cli-10/receipt.json) | `d7fc64fd85bd23a9c9958b41be90ce24c511d4ee`<br>`feature/Echoyue/im-cron-gui-control` | 25 / 3 | 4 条；`.echo-semantic/local` 2.34 MiB（保留） | 18.57 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/performance-baseline-native-a/echo-agent-cli`<br>[echo-agent-cli-11](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-cli-11/receipt.json) | `db5010ed888cf50f2400fbba31075ab63c5626e9`<br>`test/Echoyue/performance-baseline-native-a` | 4 / 11 | 6 条；`.echo-semantic/local` 164.51 MiB（保留） | 466.00 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/performance-baseline/echo-agent-cli`<br>[echo-agent-cli-12](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-cli-12/receipt.json) | `db5010ed888cf50f2400fbba31075ab63c5626e9`<br>`test/Echoyue/performance-baseline` | 4 / 11 | 2 条；`.echo-semantic/local` 1014.44 MiB（保留） | 1.01 GiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/personal-wechat-channel-v2/echo-agent-cli`<br>[echo-agent-cli-13](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-cli-13/receipt.json) | `01137a1e9186fb6c9d9b677e72b3fe4cc0b99a3b`<br>`feature/Echoyue/personal-wechat-v2` | 44 / 11 | 2 条；`.echo-semantic/local` 748.00 KiB（保留） | 17.82 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/personal-wechat-channel/echo-agent-cli`<br>[echo-agent-cli-14](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-cli-14/receipt.json) | `01137a1e9186fb6c9d9b677e72b3fe4cc0b99a3b`<br>`feature/Echoyue/personal-wechat` | 38 / 10 | 3 条；`.echo-semantic/local` 420.00 KiB（保留） | 17.36 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/project-execution-environment/echo-agent-cli`<br>[echo-agent-cli-15](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-cli-15/receipt.json) | `01137a1e9186fb6c9d9b677e72b3fe4cc0b99a3b`<br>`feature/Echoyue/project-execution-environment` | 22 / 1 | 3 条；`.echo-semantic/local` 6.24 MiB（保留） | 22.99 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/project-gui-files/echo-agent-cli`<br>[echo-agent-cli-16](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-cli-16/receipt.json) | `01137a1e9186fb6c9d9b677e72b3fe4cc0b99a3b`<br>`feature/Echoyue/project-gui-files` | 5 / 0 | 3 条；`.echo-semantic/local` 804.00 KiB（保留） | 17.48 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/project-ssh-transport/echo-agent-cli`<br>[echo-agent-cli-17](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-cli-17/receipt.json) | `01137a1e9186fb6c9d9b677e72b3fe4cc0b99a3b`<br>`feature/Echoyue/project-ssh-transport` | 4 / 9 | 3 条；`.echo-semantic/local` 54.94 MiB（保留） | 71.81 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/project-workspaces/echo-agent-cli`<br>[echo-agent-cli-18](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-cli-18/receipt.json) | `780ddd0d25c2eac5d1471cb3e5012c9d2f6b3cf9`<br>`feature/Echoyue/project-workspaces` | 54 / 13 | 5 条；`.echo-semantic/local` 1.26 GiB（保留） | 1.28 GiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/ssh-remote-terminal/echo-agent-cli`<br>[echo-agent-cli-19](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-cli-19/receipt.json) | `28bb9459bde9872b8d36cdd4a6eb98929feb0927`<br>`feature/Echoyue/ssh-remote-terminal` | 0 / 0 | 6 条；`.echo-semantic/local` 21.52 MiB（保留） | 294.29 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/steer-input-scope/echo-agent-cli`<br>[echo-agent-cli-20](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-cli-20/receipt.json) | `0607e1b244c0ec8c34f70511f9e859efa3de22d5`<br>`fix/Echoyue/steer-input-scope` | 4 / 0 | 4 条；`.echo-semantic/local` 2.71 MiB（保留） | 19.52 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/task-list-dialog/echo-agent-cli`<br>[echo-agent-cli-21](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-cli-21/receipt.json) | `780ddd0d25c2eac5d1471cb3e5012c9d2f6b3cf9`<br>`feature/Echoyue/task-list-dialog` | 40 / 0 | 5 条；`.echo-semantic/local` 27.56 MiB（保留） | 326.06 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.worktrees/echo-agent-cli-worktrees/framework-main-consumer/echo-agent-cli`<br>[echo-agent-cli-22](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-cli-22/receipt.json) | `0616c6cd25e2496c6940b74c983f4ffa08003518`<br>`(detached)` | 0 / 0 | 8 条；`.echo-semantic/local` 575.00 MiB（保留） | 591.52 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.worktrees/echo-agent-cli-worktrees/main-only-ci`<br>[echo-agent-cli-23](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-cli-23/receipt.json) | `c874a79aa25f8bef16ba4df2e809ceb447bb1604`<br>`fix/Echoyue/cancellation-observation-consumer` | 13 / 0 | 7 条；`.echo-semantic/local` 96.75 MiB（保留） | 113.04 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/.worktrees/plugin-component-preparation`<br>[echo-agent-cli-24](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-cli-24/receipt.json) | `aeededcacf2796e112731a5b453211023e1266cb`<br>`fix/Echoyue/plugin-component-preparation` | 0 / 0 | 2 条；未发现具名证据目录 | 14.09 MiB |

### echo-agent-sdk

| 路径与备份 | HEAD / branch | tracked / untracked | ignored 证据与空间 | 总占用（含嵌套） |
| --- | --- | ---: | --- | ---: |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-sdk`<br>[echo-agent-sdk-01](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-sdk-01/receipt.json) | `0930eef9f878cf436ed88384030b6bc03d6f6c53`<br>`(detached)` | 0 / 0 | 3 条；未发现具名证据目录 | 141.67 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/sdk-a2a-lsp-consumer-decoupling/echo-agent-sdk`<br>[echo-agent-sdk-02](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-sdk-02/receipt.json) | `1eac40fbf224e510ab29aea128ce59c20f8b4a52`<br>`refactor/Echoyue/sdk-a2a-lsp-consumer-decoupling` | 0 / 0 | 20 条；`.echo-semantic/local` 362.50 MiB（保留） | 601.67 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/sdk-framework-tool-domain-candidate/echo-agent-sdk`<br>[echo-agent-sdk-03](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-sdk-03/receipt.json) | `31848d2fa7dc5e1a0c9c5c91c0d9fe3ec55dd0f4`<br>`test/Echoyue/framework-tool-domain-consumer` | 0 / 0 | 3 条；`.echo-semantic/local` 365.85 MiB（保留） | 451.34 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.worktrees/echo-agent-sdk-worktrees/sdk-semantic-governance`<br>[echo-agent-sdk-04](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-agent-sdk-04/receipt.json) | `7138de2ccc87f8967129c8bd7859dee1f84ec736`<br>`(detached)` | 0 / 4 | 36 条；`.echo-semantic/local` 6.12 GiB（保留） | 6.29 GiB |

### echo-website

| 路径与备份 | HEAD / branch | tracked / untracked | ignored 证据与空间 | 总占用（含嵌套） |
| --- | --- | ---: | --- | ---: |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-website`<br>[echo-website-01](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-website-01/receipt.json) | `a926ef7cf10bdc03cc0a19b2e406cc6164477485`<br>`(detached)` | 0 / 0 | 8 条；`.supreme` 104.00 KiB（保留） | 285.81 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent-worktrees/gui-product-contract/echo-website`<br>[echo-website-02](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-website-02/receipt.json) | `dc4dc197050591411ff251cda2553ca37e0d4592`<br>`doc/Echoyue/gui-visual-docs-sync` | 0 / 0 | 7 条；`.echo-semantic/local` 1.23 MiB（保留） | 233.28 MiB |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.worktrees/echo-website-worktrees/main-only-ci`<br>[echo-website-03](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/workspace-convergence-20261026/archive-inventory/worktrees/echo-website-03/receipt.json) | `24e3d79c3c5cfd34c250a3433b477b0f0a918e04`<br>`chore/Echoyue/main-only-ci` | 0 / 0 | 4 条；`.echo-semantic/local` 239.00 MiB（保留） | 461.16 MiB |

## refs、Exa、retirement 与 Codex 保护边界

每仓 `bundles/<repo>.refs.txt`、`<repo>.stashes.txt` 保存精确 refs 与 stash reflog 映射；bundle 的 refs 与原始清单逐项一致。Exa 分支 `feature/Echoyue/exa-web-search` → `4343f0b736beb5c8606e31146528a2fe8be80510` 与 pre-retirement tags 均受保护并包含于 bundle。framework `pre-retirement-2026-10-24` → commit `51726454985fd6f064b21a7e9c539e22d494c987`，CLI 同名 tag → commit `c262cfed8e77abed38a27c1fff93547997ec9486`；其它 retirement/lsp 与 evidence/checkpoint refs 一并保存，未删改。

CLI 相对 Cargo 依赖与框架 sibling 目录继续原位保留。不得因为 clean、detached、behind、同 HEAD 或 upstream gone 直接删除；尤其 `main-only-ci`、plugin preparation、fbc4 及 framework consumer 依赖必须作为配对边界处理。

此前 Codex artifact 复核显示 active `eko-gui-product-convergence/lp-agent` 有 delivery-map dirty；已归档 `eko-workspace-maintenance/lp-agent` 的实际工作目录已移除，保留恢复元数据。聊天归档状态与 Git worktree 清理不是同一动作，本批未操作聊天或 artifacts。

本批是有限的源代码与 Git 历史恢复备份；不扫描 `.echo-semantic/local` 内嵌的历史 normal-git-view、冷克隆或任意未注册目录为新的维护对象，不复制 ignored 原始证据。下一步如需删除，必须先单独保存所需 ignored evidence 并再核对依赖、归属和当时快照。

## 2026-10-26 19:40 两处默认缓存实际 clean 与交接冻结

本节为前文“两处默认 target 接管核对”的执行后继；前面的未执行说明保留其历史切点。主线程已明确把这两个准确缓存的维护执行权交给本线，本线按原二元数组完成串行官方 clean，未清理其它路径，未操作 index 或 commit。

| 准确 target | 清理前 `du` 分配空间 | 官方命令自然出口与原始输出 | 清理后 |
| --- | ---: | --- | --- |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent-cli/target` | 631,898,112 bytes | 19:40:14，退出 0；`Removed 3937 files, 589.9MiB total` | 准确目录已移除 |
| `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/echo-agent/target` | 180,080,640 bytes | 19:40:15，退出 0；`Removed 1883 files, 165.5MiB total` | 准确目录已移除 |

逐项目命令均显式指定准确 `--manifest-path` 和 `--target-dir`，在所属仓库 cwd 执行；先等待 CLI 官方命令自然结束，再执行 framework。清理前重新枚举全部文件与目录，CLI 3,937 文件/1,037 目录、framework 1,883 文件/580 目录，文件 metadata 与原清单零漂移；最终重复快照无新目录或文件变化。两项目即时 ps/lsof 均无打开文件、cwd 或命令参数消费者。

两准确目录清理前原分配空间合计 **811,978,752 bytes，约 0.756 GiB**。这与官方工具显示的逻辑删除量分别记录，**不把主机 `df` 净变化当成原占用总和或独占释放量**；前后原始 df/du 均保留。清理后对不存在目录执行 `du` 的自然出口为 1、原文为 `No such file or directory`，这是预期的目录缺失观察，不伪写为 du 退出 0。

执行前复核 13 份 target 外必要制品 SHA 回读及 51 项动态依赖边界全部通过；清理后再次回读同 13 份制品，SHA 与 mode 均不变。PID43779 原生 GUI、PID53333 fixture 在清理后仍可见。本线未操作 core epoch10 专用 target、任何 target 外原件、源码/新文档、refs/stash/tag/Exa、host-managed 目录或历史重复候选。producer 精确历史命令及原批次出口依旧未知，**本次 clean0 只表示本次缓存清理成功，不补写原批次测试或构建成功**。

- [执行前保全、完整 metadata 和使用者检查](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/default-target-handoff-20261026/execution/preflight.json)。
- [两次准确命令、自然出口、时间与 df/du 原始回执](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/default-target-handoff-20261026/execution/clean-results.json)；[CLI 原始 clean 日志](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/default-target-handoff-20261026/execution/echo-agent-cli-clean.log)、[framework 原始 clean 日志](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/default-target-handoff-20261026/execution/echo-agent-clean.log)。
- [两个 target 已去除、13 份保全原件及受保护进程后检](/Users/ls/MyWork/code/ylp_agent_learn/lp-agent/.echo-semantic/local/default-target-handoff-20261026/execution/postcheck.json)。

本线在交付最终文件 SHA、Git blob 与末段增量 patch 后冻结 `workspace-inventory.md` 写入，交由主线程独占 index/本地里程碑 commit；未自行 stage 或 commit。后续若重新生成默认 target，应作为新批次记录，不覆盖本节清理切点。
