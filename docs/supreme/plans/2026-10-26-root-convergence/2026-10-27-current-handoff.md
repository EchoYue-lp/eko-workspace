# 2026-10-27 GUI 收敛阶段交接

## 当前结论与边界

本轮有可用成果，也有正式门禁和真实产品缺口，**Goal 及 26 项 Outcome 尚未整体验收**。唯一开发根为 `/Users/ls/MyWork/code/ylp_agent_learn/lp-agent`，四子仓独立维护。本报告仅承载跨仓阶段状态；目标与依赖仍看既有 Design／Delivery Map，公共 API 与产品说明仍由所属子仓维护。

本报告切点：Root `6263ed9357b1cf6b533dfaa12f2fae66297cad9a`；Framework `f3c5af6443fff36dd7088dcb32a665b5fc173489`；EKO `1a3551a0dac37463b8dd2848fb73b82d4d9f3c07`；SDK `525453fc9b9448358cc8a2f46bbcf401b10943ec`；官网 `83ff57fd24c7beaebd82939138f756edf5d5e32b`。HEAD 不代表各仓当前全部未提交源码资格。应用品牌 32 路径已本地提交 `1a3551a0`，官网品牌 30 路径已本地提交 `83ff57fd`；Framework、SDK 候选仍未提交，各自资格不合并成一张成功收据。

## 已取得的有限结果

| 结果 | 最新证据与限制 |
| --- | --- |
| 基础配置／独立 IM | EKO `7045853c` 已本地提交 6 路径：基础配置常显，QQ／飞书移除重复配置表单并进入独立 IM 渠道页面。复用原渠道服务、密钥与连接逻辑，独审 C0/I0/M0；原 source／staged／final 均 0，globalClosure 为 not_checked。尚未在新版原生验收。 |
| 品牌前端组合 | 主流程实际完整前端 75 文件、608 项通过；格式、lint、类型与 Vite 构建全 0，465 输入清单无漂移。应用 32 路径另含桌面诊断两处字符串；品牌已由主流程单独本地提交 `1a3551a0`，新 Tauri 尚未重建。608 项只覆盖该前端组合。 |
| 官网品牌 | 品牌会话已交接官网 49 项、浏览器 15 项及构建通过；主流程已按本仓 30 路径完成正式提交门禁并本地保存为 `83ff57fd`。官网尚未部署，Framework 正式新源同步仍待来源闭合。 |
| 原生基本交互 | epoch16 的 HELLO／同对话 HITL 已获用户确认，真实 journal 核到接纳、消费、原 HITL 结果与运行结算；不等于有效要求 Applied 或崩溃恢复验收。 |
| Subagent 默认层数 | 默认总层数 1 的实际 reviewer、父子 lineage、结算及下一层零调用已核；新 Run 的 limit=3 也已保存，旧 Run 仍绑定 1。深度 3 执行尚失败，不能把“设置保存成功”当执行通过。 |
| SSH 部分人工验收 | 用户确认 `ubuntu@111.229.245.175:22` 的测试连接、`/home/ubuntu/ssh-test` 创建／进入及刷新侧栏显示成功。完整终端和远程项目文件／执行／Git／隔离／产物未因此通过。 |

## 两项当前阻塞

**原生深度 3 属于已定位的产品缺陷。** EKO attempt helper 丢失已有 Run 身份，导致同 conversation 的新 Run、同 task／attempt 碰撞旧 settled 记录；真实 readonly reviewer 工厂又会把通用 `agent_tool` 过滤掉。下一独立修复复用既有 exact identity、原 guidance 表及派发路径，只读限制与工具正上界必须一并保留。不得随机 task ID、清 registry、把通用工具全局标只读或换 writer 夹具伪造通过。新 Design／Plan 正独立准备，本报告不冻结其 API；不要求用户在旧原生上反复重试。

**zstd 工程验证通过，正式工具校验仍失败。** Framework 456 路径含 445 个官方 vendor 文件；28 项实际 Rust、16 项来源控制、14 项政策控制、demo46、格式、实际锁及五源完整 audit、deny 四类别和正式精确风险 caller 均 0。原 sourceTask 仍自然退出 1／402 项：两项启发信号和 400 个 vendor 逐路径依据缺口。独立检查确认原工具没有合法目录来源展开入口；外部 echo-semantic 修复不纳 Goal，不扩大 exclusion、不造形式 Evidence。候选未 stage／commit，没有新正式 Framework revision；SDK 不抢先改 pin。

Framework 候选清单 SHA-256 为 `dcb8c4b35b46f7426bacec73fe634481bcd7e63b017bd7f0b9e065ad063908b7`，完整 diff 为 `f6c0092753ae3e36395bf6c124acb9326c16c8240ac6f78e674170f1d8149605`。Root 保全包 `framework-456-frozen-sources.tar.gz` SHA-256 为 `b71fb65a079ce8704c54ef27a054463ecdc8eeca21762a2f41a0eae4e773b6e0`，逐 456 文件 bytes／mode／hash 回读通过。必要制品保存后准确 clean 0，移除 6672 文件／3.6GiB，target 不存在；不借该数字宣称 APFS 净收益。

SDK 仍保护原 94 路径候选、5352 保留身份／431 routes／39 项已退役操作合同与独立 `f7bac62` pin。其原 fresh 扫描 4 漏洞／10 unsound 的失败未被上述 Framework 候选替代；后继须取得正式可恢复来源，再验证 pin／根 patch／五源来源、真实 Host Parquet、保留合同及三语言，不复制 vendor、不扩大风险接受。

## 26 项剩余结果对账

下表是既有 Delivery Map 的工程状态摘要，不改依赖、生命周期或验收标准，不表示每项从零开始。

| Outcome | 当前仍需完成 |
| --- | --- |
| product-boundary-contract | 已同步规则和固定恢复分支／tag；实际保留／退役行为及各仓交付证据继续对账。 |
| tool-inventory-and-exa-boundary | 保持既有工具事实清单与 Exa 独立边界；不按旧清单重复开发已修项。 |
| performance-baseline | 重绑当前源码／数据，补原规模与失败场景；ready24／blocked8 是历史准备分类，正式测量排静默窗口。 |
| gui-visual-foundation | 品牌与统一界面源码收尾后，仍验真实 Tauri 的窗口／主题／键盘／焦点。 |
| gui-home-navigation | 布局方向和 SSH 侧栏已有用户确认；新界面组合及三来源、切换和隔离仍验。 |
| steer-responsiveness | 现有控件／CAS／持久接纳结果继续保留；补完整原生、多输入与失败／迟到／恢复组合。 |
| effective-requirements | A 独立无工具理解／B 并入主模型的路线仍待用户选择；有效全集、作用域／替换、Applied、目标／计划／证据及独立原始要求覆盖未完成。 |
| im-cron-gui-control | 独立 IM 入口已提交；现有渠道与 cron 的连接／occurrence／结果／退出和恢复仍逐项验收。 |
| console-surface-retirement | GUI／IM／cron／共享恢复先接管并验证，随后退役 TUI／独立 REPL／CLI／JSONL；内置终端等保留。 |
| a2a-lsp-consumer-decoupling | EKO／SDK 候选保留，SDK 正式来源、安全与保留合同及适用 main／CI 尚待。 |
| a2a-lsp-retirement | 消费者解除依赖后才物理删除并验证保留能力；原外部工具合同阻塞保留。 |
| builtin-tool-convergence | 工具曝光与结果／错误已有局部实证；当前优先修真实只读工厂深度缺口，完整消费者组合仍待。 |
| hook-ask-semantics | 保留已验证切片；完整交互、取消／超时、当前合流及 main／CI 不由旧测试自动续期。 |
| gui-hitl-dialogs | 本地 HITL 人工通过；完整审批／输入／选择、竞争／过期／取消／焦点及 IM 联动仍验。 |
| task-list-dialog | 列表／历史运行／Goal 来源修复已保留；完整原生、高数据量与性能范围仍待。 |
| ssh-remote-terminal | 补中文、resize、全屏、中断、exit、多会话、断线／关闭的完整真实终端验收。 |
| project-workspaces | 补本地两来源和远程来源的重开／切换、文件／命令／Git／隔离／产物及终端同一环境验收。 |
| domain-plugin-runtime | 按任务绑定、激活／retire、foreground／Subagent／verification／resume 的真实一致消费继续验证。 |
| measured-storage-optimization | 由同源测量证明瓶颈后再优化；无瓶颈则交付测量结论，CLI 不引入 SQLite。 |
| new-im-channel | 个人微信自动化消费者资格按原切点保留；真实扫码、手机单聊／HITL、重连及定时完成＋收件双事实仍待。新增其它平台按需。 |
| legacy-resource-cleanup | 51 个旧 worktree 已可恢复回收；剩余 host、refs／stash／tag／恢复材料及新品牌 outputs 继续按归属维护。 |
| issue-finding-reconciliation | 对齐修复／退役／风险接受的具名证据，保留原失败与未验边界；本轮不关闭外部 Issue。 |
| release-target-selection | 具名版本、平台／架构、包／注册表、网站目标及制品要求仍待明确。 |
| release-artifact-qualification | 新 Tauri 重建、安装／启动／恢复与对应正式来源、完整门禁及精确 main／CI 仍待。 |
| package-publication | 未执行，待具名合格制品和明确发布授权。 |
| website-deployment | 未执行，待网站合格制品、正式来源与明确部署授权。 |

## 行动顺序与维护

1. **并行接续**：应用／官网品牌已各自完成本地里程碑，官网来源元数据后继独立对账；原生深度在独立正式 Design／Plan 内准备，不与品牌或 zstd 混写。基础配置／IM 的已提交成果不重复开发。
2. **核心优先**：原失败回归→exact Run／guidance 修复→受限只读委派→真实 reviewer 工厂和层数／越权负控制。共享文件单一 writer，验证期间冻结输入。
3. **统一桌面批次**：相关源码稳定后重建、保全指纹／制品并准确 clean，再针对当时具体制品与草稿状态安排维护切换。当前 epoch16 binary `f3fedd879…`／148 资产 `70c64653…`，记录 PID 56017／fixture 55840，不含上述新修复。
4. **分开解除依赖**：zstd 正式工具阻塞原样保留；SDK 等正式来源后下一 Plan。有效要求路线待答、个人微信与 SSH 人工验收各自推进，不拖住独立 UI／文档工作。
5. **继续完整收敛**：按 Map 依赖验收保留能力和退役，再进行完整矩阵／精确 main CI、性能与发行制品准备；包发布、官网部署不自动执行。

最近维护回执仍为五仓 66 分支、18 stash、123 tag；51 个非 host 旧 worktree 已回收，余 9 注册路径由 5 主检出与 4 host 路径组成。queued host 归档不当作已完成。Exa `4343f0b736beb5c8606e31146528a2fe8be80510` 保持已提交本机、不 push、不合 main；固定分支／tag、恢复包、README 品牌差异和 `outputs/branding/` 保留，本次无新删除。

最多两条 focused Cargo、jobs≤2／debug=0；完整矩阵与性能测量错峰，可用空间低于 5GiB 不开新构建。每批保存原始结果和必要制品、确认进程／读者结束后立即准确 clean。本报告是纯文档更新，无 Cargo，不清其它 owner target。所有提交仅本地，主线程唯一协调 index／commit；不 push／merge、关闭 Issue、发布／部署或晋升未推送子提交 gitlink。

## 本机证据与恢复入口

以下原件均在主目录内，位置与摘要只定位身份，不替代执行或扩大验收范围：

- EKO `.echo-semantic/local/gui-basic-settings-entry-20261027/root-commit-receipt.json` 和 `root-frontend-integrated-brand-epoch02/qualification.json`：6 路径提交与品牌后 608 项前端组合。
- Framework `.echo-semantic/local/zstd-safe-chain-implementation-20261027/final-actor-handoff.json`：28／16／14、正式 dependency caller 0、最终 sourceTask 1／402 与 target clean。
- Root `.echo-semantic/local/zstd-framework-blocked-preservation-20261027/preservation-receipt.json`：456 文件恢复包回读与正式工具阻塞。
- EKO `.echo-semantic/local/gui-native-epoch16-preparation-20261027/`：维护切换、HELLO／HITL、depth1 以及 depth3 失败 journal 回读；Root `.echo-semantic/local/native-depth3-attempt-collision-20261027/`：根因调查及最小修复 API／验证补充。
- Root `.echo-semantic/local/root-maintenance-delta-20261027/snapshot-20261027T0327.json`：最近维护切点。它不是本报告重新扫描远端或新删除回执。

本轮仅更新 [MASTER-PLAN](../../../MASTER-PLAN.md)、[下一动作](next-actions.md)及本报告，保留历史正文。不改正式产品设计／计划 lifecycle、示例或网站源码；这些路径与本次跨仓状态摘要无新的行为变化，品牌／官网代码由各自独立任务维护。未取得全局闭合、远端 main／CI 或发布资格。
