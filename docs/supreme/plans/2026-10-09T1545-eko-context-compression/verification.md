# EKO 上下文压缩验收记录

本地计划实施完成，已按框架 → 应用 → 官网顺序 squash 合入三仓远端 main，最终 PR 与 main push CI 全部成功。顶层同步这三个已验收指针、计划与交付记录。

## 最终交付快照

| 项目 | PR | 远端 main | 候选 / tree 对账 |
| --- | --- | --- | --- |
| 框架压缩 | [178](https://github.com/EchoYue-lp/echo-agent/pull/178) | e5372b8ca3dc308ce8dde4a0092ea58b8d7dd21d | 13a8b285；tree dfc3ca4c11e544d2e6a467e3bec04152453fbdb4 |
| 框架清理前置修复 | [179](https://github.com/EchoYue-lp/echo-agent/pull/179) | 1486d2f4a26adbefab393679c58da2804e5f4a75 | a7340a0a；tree 650f6a55107054740b6cc41a54852a7876a2d459 |
| EKO 应用 | [10](https://github.com/EchoYue-lp/echo-agent-cli/pull/10) | f98f50985c9dbc8aad8c1c7e873a8997fe5024f0 | 467ce40；tree 8d3e50b76e66768b3241f5c7a426d265c23fa45e |
| 官网 | [4](https://github.com/EchoYue-lp/echo-website/pull/4) | b1609b126014bf0cb120e68a2a665a9c64b8fa54 | 70936e0；tree 6e4a615584e532d0bd2c62e0c7b0e1e7b18ffc16 |

所有 squash 提交的 GitHub 签名 verified，Git tree 与已审、已验证候选逐字节相同。官网 manifest 绑定最终框架 1486d2f4 和应用 f98f509；完整 EKO 产品文档仍以应用仓库为权威。顶层候选仅更新三个 gitlink、MASTER-PLAN 与本计划/验收材料。

## 结果与权威边界

Summary、IncrementalSummary 与 SlidingWindow 共用按 token/turn 选择近期原文的机制。较旧 turn 整轮保留；过大当前 turn 保留完整请求与近期原子工具组，单个请求超过硬预算明确报错。普通 system、canonical 与 protected projection 保持现有权威，生成摘要不会积累为 canonical policy。取消、失败与候选摘要不会提前发布上下文或增量观察缓存。

EKO 默认 compress_window=0，使用有效 Agent 窗口 25%、至多20K token 的近期预算；显式正数使用旧消息条数上限。CLI、TUI、GUI、channel 共用安装策略、focus、provider cancellation 与 app-core journal safe point。TaskRuntimeStore 保留 goal/recovery/steer 权威，最近四条已记录 steer 按时间顺序保留完整有界 excerpt。ConversationStore 保留完整 transcript。没有新增 SQLite、goal store 或平行压缩权威。

## 验证矩阵

| 验收 | 证据 |
| --- | --- |
| 独立 Review | 压缩实现、canonical fixture、清理前置修复、CI 拆分及官网全部 pass，剩余 findings 0 |
| 框架完整门禁 | 原压缩3396 tests；最终清理修复3398 passed、0 failed、3 marked doctests ignored；fmt、两档clippy、all-target/all-feature、examples、无默认feature通过 |
| 框架独立 feature | 原17项与最终Unix条件分支对应的17项矩阵全部通过；后者在合并后补齐，日志cleanup-feature-matrix-1791606710018.log exit0 |
| 应用完整门禁 | 原workspace1922 tests、GUI218 tests通过；最终dependency组合完整Rust/GUI命令exit0，源码fmt、两档clippy、workspace、no-default与GUI均通过 |
| 前端 | 55个文件287 tests、零警告lint、Prettier与build通过；源码无后续增量，最终PR frontend再次通过；Vite保留大于700KB bundle提示 |
| 语义与文档 | 两仓strict source snapshot、分阶段change evidence、59-pair/49-ADR parity通过；最终squash tree再校验通过 |
| 官网本地 | 完整verify、39unit/15E2E、source-aware hash通过；1项mobile-only case按既有条件在desktop跳过，mobile已执行 |
| 官网独立对账 | 270个framework副本、118个应用源文档、8个产品投影全部byte/hash一致；最终仅重绑应用squash revision，正文不变 |

## 远端 CI

| 快照 | PR CI | main push CI |
| --- | --- | --- |
| 框架压缩 e5372b8c | [38017100137](https://github.com/EchoYue-lp/echo-agent/actions/runs/38017100137)：7/7 success | [38018256965](https://github.com/EchoYue-lp/echo-agent/actions/runs/38018256965)：7/7 success |
| 框架最终1486d2f4 | [38022028655](https://github.com/EchoYue-lp/echo-agent/actions/runs/38022028655)：7/7 success | [38022865621](https://github.com/EchoYue-lp/echo-agent/actions/runs/38022865621)：7/7 success |
| 应用f98f509 | [38022938018](https://github.com/EchoYue-lp/echo-agent-cli/actions/runs/38022938018)：4/4 success | [38024090595](https://github.com/EchoYue-lp/echo-agent-cli/actions/runs/38024090595)：4/4 success |
| 官网b1609b1 | [38024164122](https://github.com/EchoYue-lp/echo-website/actions/runs/38024164122)与[branch push](https://github.com/EchoYue-lp/echo-website/actions/runs/38024159746)：success | [38024315426](https://github.com/EchoYue-lp/echo-website/actions/runs/38024315426)：success |

GitHub连接曾使监听命令退出1；重新查询同一run确认状态后继续等待，没有把监听超时当测试失败或另起替代任务。

## 集成前置修复

首轮应用run38018370296的combined Linux job被取消，下载日志返回BlobNotFound，不能据此断言具体失败用例或取消因果。已复用另一线程保存的Ubuntu procps-ng4.0.4安全syscall-injection probe：kill -KILL -2443被解析为kill(-2,SIGKILL)，加--才是kill(-2443,SIGKILL)；两次调用均注入EPERM，未发送信号。

框架PR179修正CommandCell与direct-shell两处argv，保留原取消、超时、进程组创建和reap owner。无信号argv回归修复前失败，修复后两crate436+203 tests通过；完整门禁及独立review随后通过。未新增API、依赖或权限门控。最终Unix条件分支的独立feature矩阵在合并后补齐，明确保留验证时机记录。

应用CI复用已审本地候选6e72a0ad的预算拆分：quality与完整默认feature app-core suite分别使用30分钟，所有原命令、flags、framework main、TS_RS隔离与frontend保持。linux-rust聚合只接受两项success；16种成功/失败/取消/跳过组合全部检验。没有采用草稿PR9的临时诊断步骤，也未修改该线程checkout/PR。

## 本地命令收据

原压缩完整收据：framework-outcome-final-fixed-1791561703908.log；eko-frontend-outcome-final-1791559251408.log；eko-gui-final-resourced-1791562837600.log。

最终前置修复：group-kill-before-1791603131588.log（预期失败）；group-kill-after-1791603521608.log（两crate通过）；group-kill-final-gate-1791604187994.log（完整gate exit0）。

最终应用dependency组合：eko-prerequisite-consumer-final-1791604760842.log（所有Rust/GUI命令exit0）。最终官网：website-final-sources-1791605680622.log（完整verify与source check exit0）。本地日志位于各主仓库.git/worktrees/任务工作树/supreme/logs，回收任务工作树前归档。

三仓日志已逐字节核验后复制到本机持久归档：/Users/ls/.codex/visualizations/2026/09/16/01a0a981-ed75-7691-9a92-ae0df22ac4cb/remote-integration-kciHZY/（framework-logs、application-logs、website-logs）。回收工作树不会删除该归档。远端审核以本记录的GitHub run链接、源码tree与可重跑命令为证据。

## 工作区与边界

三套子仓库任务工作树使用原有相对Cargo path依赖，未改manifest路径。superproject交付使用从5459c20隔离的managed worktree；原主checkout、其它线程分支、未提交/未跟踪材料和SDK全部保留。两份Plan已由artifact CLI标记completed并通过schema/delivery binding校验。

此前磁盘耗尽的链接尝试和canonical fixture失败均保留真实日志；已清理可重建debug缓存并用单任务、关闭调试符号/incremental及一致的macOS11.0目标完成验证。canonical fixture隔离非目标checkout rules，原suffix断言保留并增加真实驱逐/预算断言；没有削弱产品约束或跳过失败用例。

原生EKO窗口、真实外部模型、官网部署不属于本次Git远端集成验收。框架其它未闭合semantic findings及全局release/soak结论不由本次结果改写。

