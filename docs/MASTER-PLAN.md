# EKO 总纲（MASTER-PLAN）

> **跨仓库当前事实源**。本文只记录阶段状态、前向路线、分层边界、验收门和可核验证据；不再保存逐轮实施日志。
> EKO 是本机个人助理：`echo-agent` 是独立可复用框架，`echo-agent-cli` 是应用层，`echo-website` 是官网。
> 最后更新：2026-08-29。

## 当前结论

F0-F6、R1 和 R2 已完成；R3 正在同步最终 framework/EKO 文档与 website。Final Integration/Release 仍未完成，因此整个优化迭代不能宣称发布完成。

| 阶段 | 当前状态 | 结论与证据入口 |
| --- | --- | --- |
| F0 characterization | Complete | 已进入 CLI `main`，作为后续交互回归输入；细节见 [`agent-interaction-convergence-plan`](./2026-08-26-agent-interaction-convergence-plan.md)。 |
| F1 receipt/admission | Complete | Persisted/Accepted/Drained/TurnSettled 已收敛；framework 与 CLI child baseline 已冻结。 |
| F2 Task/Plan/Todo authority | Complete | `TaskStatus` 是唯一执行权威，Todo 仅只读投影；ADR [0015](../echo-agent-cli/docs/adr/0015-task-graph-status-authority.md)。 |
| F3 Agent control tools | Complete | 六个 `agent_*` 工具复用既有 router/subagent/task authority；bounded query 已下推。ADR [0016](../echo-agent-cli/docs/adr/0016-agent-control-tools.md)。 |
| F4 删除 InteractionMode | Complete | 生产 Rust、DTO、prompt、surface 旧 mode 路径已删除；不得以同义名称恢复。 |
| F5 Agent/Subagent lifecycle | Complete | characterization、生产修复、测试卫生和适用完整门禁已通过；统一使用 Subagent 术语。 |
| F6 cursor/recovery/surface parity | Complete | restart cursor、cold address、workspace switch/delete、boot reconcile、terminal exactly-once 和五入口 canonical fixture 已形成 executable 闭环。 |
| R0 app/framework boundary audit | Complete | 151 个 app-core Rust 文件已分类，现已升级为 R1 final closure ledger。见 [`boundary closure`](./2026-08-28-current-framework-application-boundary-audit.md)。 |
| R1 framework-first migration | Complete | 19 个 M 与 8 个 C 均有最终 disposition；turn、TaskRuntime、artifact、bootstrap、diff、plugin、memory、tool-control、background 和旧路径已收敛，无第二 authority。 |
| R2 examples convergence | Complete | 43 个 root examples + 21 个 executable contracts 覆盖 64 个唯一场景；feature、panic、UTF-8、facade 与链接合同通过。见 [`framework-examples-inventory`](./2026-08-28-framework-examples-inventory.md)。 |
| R3 framework docs/website | In progress | child 正式文档按最终 API/examples 收敛中；website source sync、manifest 和最终双语核对尚未完成。 |
| G Final Integration/Release | Not started | 10k/100k、10 分钟/1 小时/最终 2 小时 soak、人工 GUI、远端 CI、website 最终同步和 child-first 发布均未完成。 |

## 当前基线

| 仓库 | 本地基线 | 远端/发布状态 |
| --- | --- | --- |
| `echo-agent` | `1446cae` | R1/R2 framework producer 完成；尚未执行最终 release/push 流程。 |
| `echo-agent-cli` | `0417443` | F6/R1/R2 应用与正式状态文档完成；通过相对路径 `../echo-agent` 消费 framework。 |
| `echo-website` | `c25c86d` + R3 working tree | manifest 已准备指向 framework `1446cae`、CLI `df88546`；source sync/build/E2E、提交与发布仍进行中。 |
| superproject | 当前 checkout | child gitlink 尚待 R3/G 按 child-first 顺序统一更新和发布。 |

F2-F5 的合流与门禁证据集中在 [`plan_03`](./supreme/plans/2026-08-28T0013-项目未完成工作收敛/plan_03_F5收口完整验证主分支合并与资源清理.md) 及两个 child MASTER-PLAN；这些文件记录历史实施证据，不替代本节状态表。

## 分层与不变量

- framework 保持通用 EventEnvelope、turn/checkpoint、Task DAG/revision/validator、Subagent lifecycle、Tool/Store、MCP/LSP/channel、scheduler、HITL 和通用 bounded I/O 原语。框架的合理公开能力（包括 SQLite store）不得因 EKO 不调用而删除。
- EKO 保持 workspace、DomainProfile、产品配置、TaskRun 文件权威、review/worktree、文件/工件/记忆策略、surface 投影、pool、应用生命周期和本地权限策略。EKO CLI 不启用 SQLite。
- 只有一个 turn driver、一个 receipt lifecycle、一个 revisioned Task graph/validator/ready frontier、一个 Conversation Agent router 和一个 attempt-scoped Subagent control owner。不得新增第二 runtime、mailbox、store、DAG loop、status reducer 或 mode 替身。
- GUI、TUI、CLI/JSONL、channel、cron/background 共享同一核心能力；surface 只渲染/适配，不自行推断运行终态。内部产品术语只有 `Subagent`，不得新增 `worker` 命名。
- 取消、失败、部分副作用和恢复必须以持久事实为准；不宣称任意副作用 exactly-once。所有字符串预览使用 UTF-8 安全字符迭代，外部输入不得触发 panic。

## 已完成收敛

- F6：Conversation/TaskSubagent cursor 跨重启恢复、cold/unloaded address、workspace generation/delete、boot reconcile、terminal exactly-once 和 GUI/TUI/CLI/JSONL/channel 同 fixture 已通过。
- R1：framework producer first、EKO adapter second、旧路径同阶段删除。最终逐项证据和 SHA 见 [`boundary closure`](./2026-08-28-current-framework-application-boundary-audit.md)。
- R2：29 个 root composition/teaching examples、21 个 executable contracts、14 个 conditional scenarios 共同覆盖 64 个唯一场景；正式 framework 链接已切到真实路径。

## 下一阶段路线

### R3：framework docs/website

同步 framework 中英文 README、公共 API/feature/examples 文档、EKO 正式产品文档和 website manifest。正式 framework 文档归 `echo-agent`，EKO 产品文档归 `echo-agent-cli`；顶层只保留跨仓协作和阶段证据。website 更新必须以最终 child SHA 和 source-discovery/build/E2E 结果为准。

### G：Final Integration/Release

完成 R3 后一次性执行三仓适用完整门禁、fault matrix、10k/100k release 性能、10 分钟/1 小时/最终 2 小时 soak、人工 GUI、远端 CI 修复与复验、website 同步、child-first push 和 superproject gitlink 更新。所有命令必须真实 exit 0；被阻塞或中断必须保留 exit code 和原因，不得称为通过。

## 证据与文档索引

- 交互/任务路线与依赖：[`2026-08-26-agent-interaction-convergence-plan.md`](./2026-08-26-agent-interaction-convergence-plan.md)。
- 扩展与最终集成边界：[`2026-08-26-extension-control-final-integration-unified-plan.md`](./2026-08-26-extension-control-final-integration-unified-plan.md)。
- R1 boundary closure：[`2026-08-28-current-framework-application-boundary-audit.md`](./2026-08-28-current-framework-application-boundary-audit.md)。
- R2 examples inventory：[`2026-08-28-framework-examples-inventory.md`](./2026-08-28-framework-examples-inventory.md)。
- EKO 应用事实源：[`echo-agent-cli/docs/MASTER-PLAN.md`](../echo-agent-cli/docs/MASTER-PLAN.md)、[`architecture.md`](../echo-agent-cli/docs/architecture.md)、[`features.md`](../echo-agent-cli/docs/features.md)。
- Agent control：[`ADR 0016`](../echo-agent-cli/docs/adr/0016-agent-control-tools.md)；bounded query 已由生产 store API 下推。

历史 `docs/superpowers/`、`docs/supreme/` 计划和旧审计只作为可追溯证据，不能直接授权新代码修改；新阶段需先创建同主题专项 plan/ADR（若涉及架构），并注明 owner、删除目标和退出门。

## 提交与验证约束

跨仓修改顺序固定为 framework → CLI → website → superproject gitlink。提交显式关闭 GPG 签名：

```text
git -c commit.gpgsign=false commit -m "..."
```

提交前按 `AGENTS.md` 执行与改动匹配的 fmt、clippy、workspace tests、no-default/feature matrix、GUI/frontend 和文档链接检查；磁盘不足时才按规则清理对应 `target/`。本文件不把未执行的性能、soak、人工 GUI、远端 CI 或 release 命令写成通过证据。
