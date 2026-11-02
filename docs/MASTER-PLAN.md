# EKO／Eclip 总纲：当前交付状态

更新时间：2026-11-02。用户已明确恢复完整Goal，并由新协调上下文接手。完整目标在 `docs/supreme/goals/eko-eclip-complete-iteration-2026-11-02.md`，现场与准确下一步在 `docs/supreme/handoffs/eko-current-handoff.md`；十执行线及权限见续接目录 `executors.md`。旧三天时间线原字节归入该目录 `history/`，当前表不以历史“live/paused/pending”覆盖最新事实。

唯一Root为 `01a19a4a-2815-7072-88b1-63ff005b2476`，本会话平台Goal已active、完整范围不变、无token预算。十条现有执行线已实发恢复派单：SDK实际main747接收、Framework六冲突合流、CLI及领域文档／源码增量、官网同步和K2准入并行推进；尚未把派单记为实现完成。接管快照及14接受票哈希回读见续接目录 `new-coordinator-ack.json`。K2尚无quietGO，其正式采样前另收全owner工具空闲与App正常退出证明；其它工作仅在真实独占窗口暂停工具。

## 本轮实际已闭合的阶段

- Framework PR201 精main f568，七CI成功，仅Stage0阶段。
- SDK PR14先行退出＋PR15两文档修正精main747be937，九CI全部成功；旧6cd语义14失败保原，外部旧base删除预检接口未被修。
- SDK当前C1303／44冲突解决源码独审通过，五focus八case真实通过、两NoRun W0；最新三程序／FP／.d外置4536件核验后target官方clean0（约3.1GiB）。这是当前业务切片资格，尚非最终真实main接收／完整native SDK交付。
- 本轮旧编译／运行产物／日志删除、五截图精文件删除，以及六历史会话归档已执行。当前三primary target缺席，必要源码/当前App/Auth/Git保全。

## 全部原26项结果

“已有”仅指原scope里程碑，可核输入相同才复用；表中未明确最终闭合的项目仍属于完整Goal，不能因为别项CI通过而完成。

| 项 | 原结果 | 当前状态／资格 | 负责人线 | 剩余验收／动作 |
| --- | --- | --- | --- | --- |
| 1 | 产品边界 | 既定裁决已固定；最终各入口接管未全验 | Framework／EKO／治理 | 按AGENTS保GUI主入口、IM/cron/共享恢复，单独退役具名入口。 |
| 2 | 工具与Exa | 实现／资料已有里程碑；最终真实集成未全验 | 治理／EKO | 保Exa本地分支与未授权范围，不能push/合mainExa。 |
| 3 | 性能基线 | 已有样本；正式可比安静窗口未闭合 | 性能 | 旧70有后台竞争；K2原70待准入。 |
| 4 | GUI视觉 | 已有前端与App里程碑；最终组合人工覆盖不足 | EKO／工作空间 | 浅深主题／响应式／键盘／焦点／可访问性。 |
| 5 | 首页导航 | 品牌／最近任务／工作空间已落地部分验收 | EKO | 最新源码组合实际窗口与导航／历史一致性。 |
| 6 | Steer | CAS／持久输入实现已有；最终真实运行矩阵待验 | EKO／Runtime | 普通Steer由主模型理解，不独立意图模型/强制要求流水线。 |
| 7 | IM／cron管理 | 微信／手动Cron部分人工成功；新自动Cron未通过 | IM／cron／EKO | 当前disabledrev4；启用→整分钟触发→结果/history/通知→禁用，旧Failed保真。 |
| 8 | 独立终端产品入口退役 | 用户已批准；正式最终收敛／接管仍需核 | EKO／Runtime | TUI/REPL/CLI/JSONL退役不连带GUI终端、Agent shell和SDKstdio。 |
| 9 | A2A／LSP消费者去依赖与物理退役 | 先行SDK main747阶段CI全绿；最终原生主线尚待 | SDK／Framework／EKO | 真实main747接收C＋准确当前退役证据，Framework按跨仓依赖顺序收敛。 |
| 10 | 工具收敛 | 六组/曝光/结果与错误预算已有阶段成果 | 治理／EKO | 工具真实性、canonical嵌套恢复、必需工具不可关、无第二权威。 |
| 11 | Hook／ask | 既有实现及有限测试；最终native联动待验 | 治理／Runtime | Stop继续/取消/超时/竞争/过期/并行不补成功。 |
| 12 | HITL | 既有实现；完整GUI/IM跨surface待验 | 治理／IM／EKO | 输入/选择/审批/焦点与过期owner。 |
| 13 | 任务列表／历史Run | IM/cron历史修复人工部分通过 | EKO／IM／性能 | 失败/planless归属、大数据量与性能、正确持久Run来源。 |
| 14 | SSH终端 | 准备与局部实现；两个真实账号验收未齐 | 工作空间 | echo与ubuntu两义务分开，用户GUI认证、断线不回本机。 |
| 15 | 项目工作空间 | 本地/远端/Fork代码已有里程碑 | 工作空间／EKO | HEAD-only worktree、不带脏改动、附件/原文跨源保全、双SSH owner。 |
| 16 | 领域插件 | 实现及有限回归已有；真实全链待验 | 治理／Runtime | 绑定版本/身份，activate/retire/reviewer/foreground/subagent/resume一致。 |
| 17 | 存储实测优化 | r03/两source完成部分实验；H3功能46/10k失败 | 性能／Framework／Runtime | K2真实70及剩36corpora原目标；不把失败partial作合格数据/收益。 |
| 18 | 个人微信及按需新平台 | 用户授权/手机回复与历史有限成功 | IM／cron | 扫码/确认/取消/过期/重连/私聊/HITL/手机收件全范围不齐。 |
| 19 | 旧资源清理 | 本轮具体批次及五文件删除已完成 | Root／治理 | 约47GB可用；保删除清单，后续不无限留旧制品/日志。 |
| 20 | Issue／Finding对账 | 最新增量101issue/203finding有证据；非全新复测 | 官网发行／治理 | 按实际源码/验证/退役/风险接受/未验分类，不擅自closeIssue。 |
| 21 | 发行目标 | 具名平台/版本/包/网站矩阵已准备 | 官网发行／EKO／SDK | 落实每个目标的支持／未验／阻塞，不靠列表代交付。 |
| 22 | 发行制品 | 当前本机arm64App可运行；最终同源分发组合未齐 | EKO／SDK／官网发行 | 真实targets／安装启动恢复／签名公证边界，Windows/Intel不冒通过。 |
| 23 | 包发布 | 未授权执行最终发布 | SDK／Framework／官网发行 | 先做具名合格制品/验证，再取明确外部授权。 |
| 24 | 官网部署 | 未部署 | 官网 | 实现/中文文档/可执行examples/站点来源同步最终快照，部署另授权。 |
| 25 | 主模型Steer理解 | 方向已确认并实现部分 | EKO／Runtime | 沿主模型历史理解，不恢复独立解释A/B/强制验收流水线。 |
| 26 | Goal用户权威与完整原文恢复 | 实现与严格局部测试已有；最终native恢复待验 | EKO／Runtime | 已有Goal仅人类显式CAS改删，current版本cold/import/fork长原文可读，缺context暂停相关新副作用。 |

## 新增完整原生消息／统一终态／长任务进度（与原26同批交付）

Framework、所有Provider、EKO、SDK、示例和官网要一起收敛：具名退役ThinkTool／FinalAnswerTool；reasoning／assistant commentary/partial/final／tool／runtime状态无损分开；真实ID/顺序/stream和nonstream/签名加密history完整。存在工具先结清，截断／空响应／异常断流不能成功；候选统一经结构化Guard/Critic/StopHook/持久化/结算，Stop继续保持同执行，最终发布/成功幂等一次。消费者不能看到FinalAnswer就停读，必须真实settled＋源EOF。旧版本兼容不保，当前版本自己的cold/reload/Fork/import/取消恢复和不重放必须验。

应用默认60秒无可见自然语言，在下一模型请求安全点提醒一次；长工具未返回显示真实操作/耗时/最后事件，不额外模型循环、不打断工具、不拿状态冒新发现。纯文、多消息、多并行工具、混合、未知phase、断流/截断/校验失败/取消/恢复/Subagent/GUI历史/IMcron/三语/长等待全场景需真实资格，现仍有native缺口。

## 立即实施顺序（可独立工作必须并行）

1. SDK真实base747接收已测C与main两doc历史，准确当前native退役证据＋正式Source/Final/适用门禁；现有八case输入不变不重跑。
2. Framework按实际消费者去依赖状态收敛main，EKO／IM／cron跟进最终组合；其它领域按本表交真实增量。
3. K2安静70样本窗口可fresh安排：Rover现gone，正常退出9970/原用户数据保护、全owner toolidle、原N与预算，测后同包同U重开；仅窗口内局部停止其它工具。
4. 最新稳定组合App构建→真实GUI/终端/SSH/手机IM/自动Cron/HITL/Subagent/进度/恢复→36corpora／性能→IssueFinding／文档／examples／站点→具名发行制品，外部push/merge/publish/deploy最后问精scope。

## 执行纪律

同一主根、不新外部worktree；Root唯一stage/commit、各repo独立、所有commit显式GPGfalse；不晋升未push子repo指针。完整heavy/正式采样独占；最多5focused各独立T/jobs≤5/debug0、fresh<5GiB不启重构建；合理成批→必要target外保全→reader0→官方精clean，失败不造TAG或抹退出码。保持当前UserRoot/Auth，已删旧程序/归档不得假称可重放。十现有会话继续复用，新协调会话接手后旧Root不双写。
