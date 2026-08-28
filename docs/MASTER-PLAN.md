# EKO 总纲（MASTER-PLAN）

> **跨仓库当前事实源**。本文只记录阶段状态、前向路线、分层边界、验收门和可核验证据；不再保存逐轮实施日志。
> EKO 是本机个人助理：`echo-agent` 是独立可复用框架，`echo-agent-cli` 是应用层，`echo-website` 是官网。
> 最后更新：2026-08-28。

## 当前结论

F0-F5 已完成；F6 底层能力存在但验收闭环未完成。R0 只读边界审计已完成；R1 尚未开始，R2 只有 inventory，R3 与最终发布尚未开始。优化迭代因此不能宣称完成。

| 阶段 | 当前状态 | 结论与证据入口 |
| --- | --- | --- |
| F0 characterization | Complete | 已进入 CLI `main`，作为后续交互回归输入；细节见 [`agent-interaction-convergence-plan`](./2026-08-26-agent-interaction-convergence-plan.md)。 |
| F1 receipt/admission | Complete | Persisted/Accepted/Drained/TurnSettled 已收敛；framework 与 CLI child baseline 已冻结。 |
| F2 Task/Plan/Todo authority | Complete | `TaskStatus` 是唯一执行权威，Todo 仅只读投影；ADR [0015](../echo-agent-cli/docs/adr/0015-task-graph-status-authority.md)。 |
| F3 Agent control tools | Complete | 六个 `agent_*` 工具复用既有 router/subagent/task authority；ADR [0016](../echo-agent-cli/docs/adr/0016-agent-control-tools.md)。底层 full-vector query 仍是 R1/P0 residual。 |
| F4 删除 InteractionMode | Complete | 生产 Rust、DTO、prompt、surface 旧 mode 路径已删除；不得以同义名称恢复。 |
| F5 Agent/Subagent lifecycle | Complete | characterization、生产修复、测试卫生和适用完整门禁已通过；统一使用 Subagent 术语。 |
| F6 cursor/recovery/surface parity | **Partial** | cursor token、multi-target wait、boot reconcile、router restart 测试和静态 surface matrix 已存在，但缺跨重启 cursor、cold/unloaded address、workspace switch/delete、五入口共享 fixture 及无 stranded receipt/handle 的统一验收。 |
| R0 app/framework boundary audit | Complete | 151 个 app-core Rust 文件已分类；审计为只读，不代表迁移已执行。见 [`current-framework-application-boundary-audit`](./2026-08-28-current-framework-application-boundary-audit.md)。 |
| R1 framework-first migration | Not started | R0 的 19 个 `Migrate/converge` 与 8 个 `Conditional` 候选尚未生产切换；当前直接 Delete 为 0。 |
| R2 examples convergence | Inventory only | 64 个 framework examples 已逐项分类，尚未重组或清理；见 [`framework-examples-inventory`](./2026-08-28-framework-examples-inventory.md)。 |
| R3 framework docs/website | Not started | 双语 framework 文档和 website 尚未按新 facade/API 同步；manifest 仍指向旧 child revisions。 |
| G Final Integration/Release | Not started | 10k/100k、10 分钟/1 小时/最终 2 小时 soak、人工 GUI、远端 CI、website 最终同步和 child-first 发布均未完成。 |

## 当前基线

| 仓库 | 本地基线 | 远端/发布状态 |
| --- | --- | --- |
| `echo-agent` | `302453b174086c3795dc026d16eeb668ecc66bed` | `main` 与 `origin/main` 对齐；这是当前 framework producer。 |
| `echo-agent-cli` | `d09f11c7878474d0e01ba2562309d5890e369554` | `main` 与 `origin/main` 对齐；通过相对路径 `../echo-agent` 消费 framework。 |
| `echo-website` | 以其当前 checkout 为准 | `docs-sync-manifest.json` 仍记录 framework `9f8d723`、CLI `e7d9e90`，必须等 R1/R2 稳定后更新。 |
| superproject | `7795843` | 本地 `main` 比远端多 1 个文档提交；远端仍为 `3ec4d96`。未 push/release。 |

F2-F5 的合流与门禁证据集中在 [`plan_03`](./supreme/plans/2026-08-28T0013-项目未完成工作收敛/plan_03_F5收口完整验证主分支合并与资源清理.md) 及两个 child MASTER-PLAN；这些文件记录历史实施证据，不替代本节状态表。

## 分层与不变量

- framework 保持通用 EventEnvelope、turn/checkpoint、Task DAG/revision/validator、Subagent lifecycle、Tool/Store、MCP/LSP/channel、scheduler、HITL 和通用 bounded I/O 原语。框架的合理公开能力（包括 SQLite store）不得因 EKO 不调用而删除。
- EKO 保持 workspace、DomainProfile、产品配置、TaskRun 文件权威、review/worktree、文件/工件/记忆策略、surface 投影、pool、应用生命周期和本地权限策略。EKO CLI 不启用 SQLite。
- 只有一个 turn driver、一个 receipt lifecycle、一个 revisioned Task graph/validator/ready frontier、一个 Conversation Agent router 和一个 attempt-scoped Subagent control owner。不得新增第二 runtime、mailbox、store、DAG loop、status reducer 或 mode 替身。
- GUI、TUI、CLI/JSONL、channel、cron/background 共享同一核心能力；surface 只渲染/适配，不自行推断运行终态。内部产品术语只有 `Subagent`，不得新增 `worker` 命名。
- 取消、失败、部分副作用和恢复必须以持久事实为准；不宣称任意副作用 exactly-once。所有字符串预览使用 UTF-8 安全字符迭代，外部输入不得触发 panic。

## 下一阶段路线

### F6：cursor、recovery、surface parity

F6 是当前最近的功能闭环，owner 在 `echo-agent-cli` app-core；framework 只提供通用事件/cursor/checkpoint 原语。

1. 用同一 deterministic fixture 覆盖 Conversation 与 TaskSubagent cursor 的 append、wait、消费、进程重启和 resume；确认已确认 terminal 不重复投递。
2. 覆盖 cold/unloaded address、workspace switch/delete、generation 变化和 router restart；boot reconcile 必须清理 orphan、stranded receipt、lease 和 handle。
3. 让 GUI/TUI/CLI/JSONL/channel 使用同一 fixture 与事件合同，核对 identity、error、artifact、HITL、cancel 和 terminal 投影完全一致。
4. 删除仍存在的 surface-local 地址/状态推断；保留 app adapter，但 generic terminal/retry/settlement 只能来自 framework authority。

F6 退出门：无重复 terminal、无 stranded receipt/handle、跨重启 cursor 可恢复、workspace 删除不残留可达资源、五入口同 fixture 全绿。未满足前只能标记 Partial。

### R1：framework-first 迁移

输入是 R0 审计，不重新发明架构。优先迁移并在同一阶段删除被替代路径：

- turn/event、tool/artifact 的通用事实与 app adapter 的重复投影；
- Task runtime adapter、应用 bootstrap、plugin/memory generation 中已证明通用的机制；
- Tauri command 薄化及 shared service composition，保持 workspace、UI、review/worktree 等 EKO 策略在应用层。

每个切片都必须先核对 framework 内部及合理复用方的定义/注册/可达性，再由 framework producer 先改、CLI adapter 无损切换、旧主路径同阶段删除。退出门是 framework 独立门禁、CLI round-trip/contract tests、旧定义注册可达性为零且无第二 authority。R0 的 `Keep` 项不因 EKO 未调用而删除。

### R2：examples 收敛

以稳定 public facade 为前提处理 inventory 的 64 个 root examples：

- `keep-root` 保留为 framework public API 示例；`move-consumer` 迁入 `echo-agent-examples`，只依赖 `echo_agent`；`move-test` 转为 feature-isolated deterministic contract；`conditional` 明确凭证/服务/运行时前置条件并 fail-fast。
- 修复 manifest drift（至少 demo19、31、43、47、70），清零 14 个文件的 52 处 `unwrap/expect`、byte slice/byte-count 和 JSON direct-index 风险。
- `echo-rust-learning` 的 13 个教学 examples 留在教学 crate，不计入 64 个 framework examples。

R2 退出门：64 个文件均有 disposition，保留项可编译，facade-only consumer probe 全绿，禁用 panic/UTF-8 扫描为零；inventory 当前明确不是通过证据。

### R3：framework docs/website

R1/R2 稳定后，同步 framework 中英文 README、公共 API/feature/examples 文档、doctest 和 website manifest。正式 framework 文档归 `echo-agent`，EKO 产品文档归 `echo-agent-cli`；顶层只保留跨仓协作和阶段证据。更新 website 前必须以 child commit SHA 和 source-discovery/build/E2E 结果为准，不提前改旧 manifest。

### G：Final Integration/Release

完成 F6、R1、R2、R3 后一次性执行三仓适用完整门禁、fault matrix、10k/100k release 性能、10 分钟/1 小时/最终 2 小时 soak、人工 GUI、远端 CI 修复与复验、website 同步、child-first push 和 superproject gitlink 更新。所有命令必须真实 exit 0；被阻塞或中断必须保留 exit code 和原因，不得称为通过。

## 证据与文档索引

- 交互/任务路线与依赖：[`2026-08-26-agent-interaction-convergence-plan.md`](./2026-08-26-agent-interaction-convergence-plan.md)。
- 扩展与最终集成边界：[`2026-08-26-extension-control-final-integration-unified-plan.md`](./2026-08-26-extension-control-final-integration-unified-plan.md)。
- R0 边界 inventory：[`2026-08-28-current-framework-application-boundary-audit.md`](./2026-08-28-current-framework-application-boundary-audit.md)。
- R2 examples inventory：[`2026-08-28-framework-examples-inventory.md`](./2026-08-28-framework-examples-inventory.md)。
- EKO 应用事实源：[`echo-agent-cli/docs/MASTER-PLAN.md`](../echo-agent-cli/docs/MASTER-PLAN.md)、[`architecture.md`](../echo-agent-cli/docs/architecture.md)、[`features.md`](../echo-agent-cli/docs/features.md)。
- Agent control residual：[`ADR 0016`](../echo-agent-cli/docs/adr/0016-agent-control-tools.md)；底层 `list_events`/`list_subagent_runs` 仍需真正 bounded query，下推前不得宣称解决。

历史 `docs/superpowers/`、`docs/supreme/` 计划和旧审计只作为可追溯证据，不能直接授权新代码修改；新阶段需先创建同主题专项 plan/ADR（若涉及架构），并注明 owner、删除目标和退出门。

## 提交与验证约束

跨仓修改顺序固定为 framework → CLI → website → superproject gitlink。提交显式关闭 GPG 签名：

```text
git -c commit.gpgsign=false commit -m "..."
```

提交前按 `AGENTS.md` 执行与改动匹配的 fmt、clippy、workspace tests、no-default/feature matrix、GUI/frontend 和文档链接检查；磁盘不足时才按规则清理对应 `target/`。本文件不把未执行的性能、soak、人工 GUI、远端 CI 或 release 命令写成通过证据。
