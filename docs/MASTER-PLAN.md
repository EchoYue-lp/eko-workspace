# EKO 总纲（MASTER-PLAN）

> **跨仓库当前事实源**。本文只记录阶段状态、前向路线、分层边界、验收门和可核验证据；不再保存逐轮实施日志。
> EKO 是本机个人助理：`echo-agent` 是独立可复用框架，`echo-agent-cli` 是应用层，`echo-website` 是官网。
> 最后更新：2026-09-04。

## 当前结论

F0-F6、R1、R2 和 R3 文档/website 收敛已完成；G 的当前静态门禁已完成，但完整测试与 Final Integration/Release 仍为 conditional，因此整个优化迭代不能宣称发布完成。

| 阶段 | 当前状态 | 结论与证据入口 |
| --- | --- | --- |
| F0 characterization | Complete | 已进入 CLI `main`，作为后续交互回归输入；细节见 [`agent-interaction-convergence-plan`](./2026-08-26-agent-interaction-convergence-plan.md)。 |
| F1 receipt/admission | Complete | Persisted/Accepted/Drained/TurnSettled 已收敛；framework 与 CLI child baseline 已冻结。 |
| F2 Task/Plan/Todo authority | Complete | `TaskStatus` 是唯一执行权威，Todo 仅只读投影；ADR [0015](../echo-agent-cli/docs/zh/adr/0015-task-graph-status-authority.md)。 |
| F3 Agent control tools | Complete | 六个 `agent_*` 工具复用既有 router/subagent/task authority；bounded query 已下推。ADR [0016](../echo-agent-cli/docs/zh/adr/0016-agent-control-tools.md)。 |
| F4 删除 InteractionMode | Complete | 生产 Rust、DTO、prompt、surface 旧 mode 路径已删除；不得以同义名称恢复。 |
| F5 Agent/Subagent lifecycle | Complete | characterization、生产修复、测试卫生和适用完整门禁已通过；统一使用 Subagent 术语。 |
| F6 cursor/recovery/surface parity | Complete | restart cursor、cold address、workspace switch/delete、boot reconcile、terminal exactly-once 和五入口 canonical fixture 已形成 executable 闭环。 |
| R0 app/framework boundary audit | Complete | 151 个 app-core Rust 文件已分类，现已升级为 R1 final closure ledger。见 [`boundary closure`](./2026-08-28-current-framework-application-boundary-audit.md)。 |
| R1 framework-first migration | Complete | 19 个 M 与 8 个 C 均有最终 disposition；turn、TaskRuntime、artifact、bootstrap、diff、plugin、memory、tool-control、background 和旧路径已收敛，无第二 authority。 |
| R2 examples convergence | Complete | 43 个教学/组合 demo 与 21 个 executable contracts 已统一迁入 `echo-agent/echo-agent-learning`；feature、panic、UTF-8、facade 与链接合同通过。见 [`framework-examples-inventory`](./2026-08-28-framework-examples-inventory.md)。 |
| R3 framework docs/website | Complete / source sync current | framework 与 EKO 正式双语文档、示例路径、discovery 与 source-aware sync 已完成；当前 website manifest 绑定 CLI `c29ca20` 与 framework `4ad095b`。 |
| R4 app-core global modularization | Complete | app-core authority modules are physically split behind `echo_agent_app_core::api`; CLI/TUI/Tauri/channel/examples/tests use the facade. Wire and persistence contracts remain unchanged. R4 validation remains pinned to code `0e762ab` and framework `125ea5f`; current child baselines are listed below. See `echo-agent-cli/docs/zh/adr/0025-app-core-global-modularization.md`. |
| Framework capability placement correction | Complete | 通用 primitive 不再以第二消费者作为准入门槛；AgentPool/AgentRouter/ChatEventLog/Plugin/Extension 的逐符号 disposition 见 [`framework capability placement audit`](./2026-08-30-framework-capability-placement-audit.md)。 |
| AgentPool keyed admission kernel | Complete | `echo-agent` 的 `KeyedExecutionAdmission` 负责 opaque-key lease、per-key process permit、retirement、close/wait；EKO `AgentPool` 保留 Agent cache、capacity class、workspace、plugin/model/tool policy。 |
| AgentRouter delivery ledger contract | Typed API complete / development schema reset | framework typed `DeliveryLedger<Route, Payload>` 与统一 `DeliveryTransition` 已成为唯一 lifecycle/projection/retry authority；EKO 直接使用 `AgentAddress`/`AgentMessage`，旧 `ClaimSettlement`、`AgentInboxProjection`、`FoldedDelivery`、legacy event wire 与 checkpoint codec 均已删除。 |
| Framework-native domain values | Complete for current migration lane | EKO 权限状态直接保存 framework `PermissionRule`；matcher、behavior、source 由 framework `FromStr` 校验；TaskRuntime 直接保存 framework `TaskExecution`；Subagent 与 conversation-input outcome 直接使用 framework 类型；framework 配置通过标准 `From<FrameworkConfig> for AgentConfig` 转换；delivery/subagent 对外入口统一为 `echo_agent::delivery`、`echo_agent::subagent`，两类执行结果直接返回 `ExecutionUsage`，不再保留 split-crate workspace facade、转换 trait、framework-shaped DTO 或 `to_framework_*`/`from_framework_*` helper。ADR framework `0021`、EKO `0027`。 |
| Typed LLM timeout contract | Complete for framework/API integration | framework `LlmTimeouts` 同时拥有 non-stream request 与 stream first-chunk/idle/overall 边界；`LlmConfig` 提供 client 默认，`ChatRequest` 使用同类型覆盖单次调用。Chat Completions、Responses、Anthropic 共用唯一 SSE transport；旧 `ECHO_AGENT_STREAM_*`、Anthropic 私有 byte loop、固定 reqwest 120s stream 总超时和 `llm::{core,integration,config,providers}` 迁移 facade 已删除。ADR framework `0022`。 |
| Dead AgentRunner contract cleanup | Complete | `src/runner.rs` 从未进入 module tree，且 deprecated facade 文档引用不存在的 context/eval builder API；现已删除。Agent 构建、trace 与 eval 分别直接使用 `ReactAgentBuilder`、`RunStore`、`EvalRunner`，Eval/Tracing 双语文档和 website 不再声称存在自动 recorder 或第二 runner。 |
| Current EKO product schema authority | Complete | `.eko/learned-rules.md`、`.eko/workspace.json`、原样 cron prompt 与 `TaskRuntimeStore::new()` 是唯一当前输入。`.eko/AGENTS.md`、root `.workspace.json` reader、`[plan]` marker strip、`TaskRuntimeStore::open()` 和生产 run-id-only wrapper 已删除；retired workspace marker 只用于防覆盖，journal rebuild 与保守 worktree cleanup 继续保护当前数据。EKO ADR `0028`。 |
| Framework-native Agent/model config | Complete | EKO 直接保存 framework `AgentSettings`，并以产品默认结构化合并部分 YAML；顶层 `model` 只含 `default_model_id` 且拒绝未知 mirror 字段；`ConfiguredModel`/`ModelProviderConfig` 分别唯一拥有模型与连接事实；`resolve_runtime_model -> Result` 是所有 surface 共用的唯一 resolver，不再合成 synthetic model。EKO ADR `0029`。 |
| Current Skill format authority | Complete / official format enforced | `SkillDocument::parse/parse_at` is the sole parser/writer authority. File frontmatter accepts only agentskills.io fields, rejects private extensions, invalid types/null, name mismatch, and official length violations; Skill files carry no private Hook sidecar, and host/plugin Hook configuration stays separate. `validate_skill_markdown/validate_skill_dir`, SkillsHub, install, Plugin validation, SkillMerger, and examples use the same contract. |
| Enabled Skill 运行时权威 + catalog 收缩 | Complete / locally merged; remote delivery pending | `enabled-skills.json` 只保留 `{category, enabled, baseline}` flat policy 与原子写，损坏配置 fail-open；generation CAS、operation identity 与 repair debt 已删除。catalog 41 → 39 → 24，默认启用 8 → 5，methodology baseline 4 → 1。TUI/GUI 可激活当前会话 Skill，Tauri bundle 使用官方 resource dir，Agent Plugins 1.0 的 skills 面按包原子安装。framework `ac00815`、EKO `f194c43`、website `35490e8` 已在各自本地 `main`；尚未 push，顶层 gitlink 因此尚未提交。EKO ADR `0032`、`0033`、`0036`。 |
| Unified Subagent prompt architecture | Complete / static gates passed | framework `SubagentPromptCompiler` 统一拥有 stable system prompt 与 typed invocation messages；真实 registered capability、`ToolVisibilityPolicy`、typed `access_mode`、effective workspace、allowlist override、结构化历史与附件均在单一编译链收敛。direct/planned/fork/teammate/team member/plugin/primary TaskRuntime 共用该合同；角色 Markdown 只保留角色、方法和领域知识。framework ADR `0024`、EKO ADR `0030`。 |
| Deterministic CommandCell watch | Complete / static gates passed | framework `CommandCellWatcher` 持有 observation lease、按 byte cursor drain 并返回 typed terminal；EKO 只保留 exact workspace/conversation/root、generation、Ready/delivery/ack、恢复与 surface policy。模型驱动的 `awaiter` Subagent、provider summary/status、Subagent attempt identity 与旧 active spec 已删除。framework ADR `0025`、EKO ADR `0031`。 |
| Bilingual docs parity foundation | Complete | CLI 已提交 40 对 zh/en 镜像文档（含 30 个 ADR）和 fail-closed checker；website sync 强制执行 parity 并记录 sourcePath/hash。 |
| Repository hygiene | Complete / runtime scope protected | 已清理 20 个空缓存/源码占位目录，并在全部验证后按磁盘规则清理两个 Cargo target 缓存（共释放 48.5 GiB）；tracked audit `.txt`、website `.txt`、115 个 `.eko` runtime trace、8 个 soak roots 和验收 worktree 均保留，未按扩展名删除运行数据。 |
| G Final Integration/Release | Conditional / full gates pending | 当前 framework/app all-target compile、严格 Clippy、双语 parity 与 website static checks 已通过；测试、frontend build、GUI、soak、人工 GUI、远端 CI 和 release 尚未重新验证。 |
| 长程任务优化(统一 turn-run 绑定) | Closure complete on local `main` / `echo-agent-cli@be273d3` + `echo-website@ae91c39`,未推送 | `feature/unified-turn-run@9524b47` 的 9 项 review 修复已作为基线,closure 补齐 typed provenance memory gate、quiet-check 与 RunTurn claim 原子复核、五入口长 steer 原文保真、`ForegroundTurnSnapshot.run_id` TS 绑定和 `PlanRevisionCommitted` 任务 UI 激活。第三轮独立复审 Critical/Important/Minor 均为 0。CLI 门禁已通过 fmt、Clippy 双档、workspace all-features(app-core 1534 passed/9 ignored、CLI lib 276、main 11、JSONL 6)、no-default、GUI check/tests(207 + main 1 + JSONL 6)、前端 Prettier/ESLint/Vitest 244/build、双语 parity；website source-aware docs check 与完整 verify 通过(39 单测、174 static/discovery routes、Playwright 15 passed/1 skipped)。后续项(独立 ADR):task_execute §10.1 同步 await 合同的后台化。 |

## 当前基线

| 仓库 | 本地基线 | 远端/发布状态 |
| --- | --- | --- |
| `echo-agent` | `ac00815` | 本轮未修改 framework；作为统一 turn-run closure 的干净依赖基线。 |
| `echo-agent-cli` | `be273d3` (`main`) | 统一 turn-run 绑定的 review remediation 与 closure 已合入本地 `main`；第三轮独立复审 0 findings，Rust workspace/no-default/GUI 与 frontend 全部门禁通过，尚未推送。 |
| `echo-website` | `ae91c39` (`main`) | EKO storage authority、CLI source manifest 与 `llms-full.txt` 已同步并合入本地 `main`；source-aware docs check 和完整 website verify 通过，尚未推送。 |
| superproject | local `main` | 已记录上述三个 child 指针与 closure 证据；远端 CI、push/release 与最终发布仍未闭合。 |

F2-F5 的合流与门禁证据集中在 [`plan_03`](./supreme/plans/2026-08-28T0013-项目未完成工作收敛/plan_03_F5收口完整验证主分支合并与资源清理.md) 及两个 child MASTER-PLAN；这些文件记录历史实施证据，不替代本节状态表。

## 分层与不变量

- framework 保持通用 EventEnvelope、turn/checkpoint、Task DAG/revision/validator、Subagent lifecycle、Tool/Store、MCP/LSP/channel、scheduler、HITL 和通用 bounded I/O 原语。框架的合理公开能力（包括 SQLite store）不得因 EKO 不调用而删除。
- EKO 保持 workspace、DomainProfile、产品配置、TaskRun 文件权威、review/worktree、文件/工件/记忆策略、surface 投影、pool、应用生命周期和本地权限策略。EKO CLI 不启用 SQLite。
- R4 的 app-core 物理模块化不改变上述 owner：`state/`、`tasks/task_runtime/store/`、`tasks/task_runtime/executor/`、`agent_router/`、`chat_event_log/`、`agent_pool/`、`extension_control/`、`plugin_runtime/` 和 `infra/` 通过单一 `api/` facade 对外；没有第二 runtime/store/DAG/status/publication authority。
- 只有一个 turn driver、一个 receipt lifecycle、一个 revisioned Task graph/validator/ready frontier、一个 Conversation Agent router 和一个 attempt-scoped Subagent control owner。不得新增第二 runtime、mailbox、store、DAG loop、status reducer 或 mode 替身。
- GUI、TUI、CLI/JSONL、channel、cron/background 共享同一核心能力；surface 只渲染/适配，不自行推断运行终态。内部执行角色统一使用 `Subagent` 术语。
- 取消、失败、部分副作用和恢复必须以持久事实为准；不宣称任意副作用 exactly-once。所有字符串预览使用 UTF-8 安全字符迭代，外部输入不得触发 panic。

## 已完成收敛

- F6：Conversation/TaskSubagent cursor 跨重启恢复、cold/unloaded address、workspace generation/delete、boot reconcile、terminal exactly-once 和 GUI/TUI/CLI/JSONL/channel 同 fixture 已通过。
- R1：framework producer first、EKO product integration second、旧路径同阶段删除。最终逐项证据和 SHA 见 [`boundary closure`](./2026-08-28-current-framework-application-boundary-audit.md)。
- R2：`echo-agent-learning` 统一承载 43 个编号 demo、13 个 Rust 学习章节和 21 个 executable contracts；正式 framework 文档只引用 learning package 的示例路径。
- 全方向通信矩阵（2026-09-03）：框架 `feature/agent-communication`（SubagentLineage/UplinkSink/共享控制面/subagent_message+list，ADR 0027，demo50）与 CLI `feature/agent-communication`（EKO uplink sink、escalation→NeedsInput、同 run 兄弟投递、8 角色全开 can_delegate、agent_spawn/resume/handoff/group 四工具，ADR 0034）在各自 worktree 分支完成并通过 fmt/clippy 双档/feature 矩阵/gui bin/前端三件套；CLI 全量测试 76 项失败全部为运行中 EKO 实例持有 `~/.eko/tasks` 文件锁的环境冲突（零断言回归），待实例退出后复跑补验再合并。

## 下一阶段路线

### 全方向通信矩阵收尾（Complete，2026-09-03）

76 项锁冲突测试在 EKO 实例退出后复验全绿（CLI 全量 1844 通过 0 失败）；squash merge 落地：echo-agent main `53473e3`、echo-agent-cli main `8a7fd85`，均已推送；worktree/分支/symlink 已清理；superproject gitlink `d8388dd` 已推送。合并前两仓主 checkout 的遗留 WIP 已 stash 保留：echo-agent `archive/pre-agent-communication-merge-wip-critic-schema-20260903`、echo-agent-cli `archive/pre-agent-communication-merge-wip-pool-primary-frontend-20260903`（未自动 pop，由所有者决定恢复时机）。

### R3：framework docs/website（Complete）

framework 中英文 README、公共 API/feature/examples 文档和 EKO 正式产品文档已按 framework `d9cb003` 与 CLI `c29ca20` 完成复核；typed Delivery Ledger、immutable `SkillDocument`、official Skill format/validator、typed Task/Subagent/Tool/LLM API、统一 Subagent prompt compiler、registered capability/visibility/access authority 与 deterministic CommandCell watch 均已同步到 website，source/discovery/site checks 已通过；本阶段未运行完整测试、frontend build、GUI、soak、release 或浏览器人工门禁。

### G：Final Integration/Release

在 R3 完成后一次性执行三仓适用完整门禁、fault matrix、10k/100k release 性能、10 分钟/1 小时/最终 2 小时 soak、人工 GUI、远端 CI 修复与复验、child-first push 和 superproject gitlink 更新。所有命令必须真实 exit 0；被阻塞或中断必须保留 exit code 和原因，不得称为通过。

## 证据与文档索引

- 交互/任务路线与依赖：[`2026-08-26-agent-interaction-convergence-plan.md`](./2026-08-26-agent-interaction-convergence-plan.md)。
- 扩展与最终集成边界：[`2026-08-26-extension-control-final-integration-unified-plan.md`](./2026-08-26-extension-control-final-integration-unified-plan.md)。
- R1 boundary closure：[`2026-08-28-current-framework-application-boundary-audit.md`](./2026-08-28-current-framework-application-boundary-audit.md)。
- R2 examples inventory：[`2026-08-28-framework-examples-inventory.md`](./2026-08-28-framework-examples-inventory.md)。
- Tools/app-core boundary assessment：[`2026-08-29-tools-app-core-boundary-assessment.md`](./2026-08-29-tools-app-core-boundary-assessment.md)。
- Framework capability placement audit：[`2026-08-30-framework-capability-placement-audit.md`](./2026-08-30-framework-capability-placement-audit.md)。
- AgentRouter delivery ledger phase 1：[`plan_06_冻结AgentRouter通用ledger契约`](./supreme/specs/2026-08-30-framework-boundary-and-bilingual-docs/plans/plan_06_冻结AgentRouter通用ledger契约.md)。
- AgentRouter delivery ledger phase 2：[`plan_07_AgentRouter生产ledger切换`](./supreme/specs/2026-08-30-framework-boundary-and-bilingual-docs/plans/plan_07_AgentRouter生产ledger切换.md)。
- AgentRouter delivery ledger phase 3：[`plan_08_AgentRouter删除旧projection`](./supreme/specs/2026-08-30-framework-boundary-and-bilingual-docs/plans/plan_08_AgentRouter删除旧projection.md)。

以上三份计划均为 `status: superseded` 的历史工程记录。它们原先规划的 adapter、legacy
wire/checkpoint codec 和旧 projection 保留策略没有延续到当前实现；typed framework
`DeliveryLedger` 与 EKO 直接 typed 集成的 ADR 0019/0026 才是现行事实源。

- Repository hygiene：[`2026-08-30-repository-hygiene.md`](./2026-08-30-repository-hygiene.md)。
- Skill 系统重构（Phase 0-5 已实现、复审、验证并本地合并，待 remote delivery 与顶层 gitlink）：[`2026-09-03-skill-system-refactor.md`](./2026-09-03-skill-system-refactor.md)。
- EKO website projection review：[`2026-08-29-eko-website-projection-review.md`](./2026-08-29-eko-website-projection-review.md)。
- EKO 应用事实源：[`echo-agent-cli/docs/zh/project-status.md`](../echo-agent-cli/docs/zh/project-status.md)、[`architecture overview`](../echo-agent-cli/docs/zh/architecture/overview.md)、[`features`](../echo-agent-cli/docs/zh/features.md)。
- Agent control：[`ADR 0016`](../echo-agent-cli/docs/zh/adr/0016-agent-control-tools.md)；bounded query 已由生产 store API 下推。

历史 `docs/superpowers/`、`docs/supreme/` 计划和旧审计只作为可追溯证据，不能直接授权新代码修改；新阶段需先创建同主题专项 plan/ADR（若涉及架构），并注明 owner、删除目标和退出门。

## 提交与验证约束

跨仓修改顺序固定为 framework → CLI → website → superproject gitlink。提交显式关闭 GPG 签名：

```text
git -c commit.gpgsign=false commit -m "..."
```

提交前按 `AGENTS.md` 执行与改动匹配的 fmt、clippy、workspace tests、no-default/feature matrix、GUI/frontend 和文档链接检查；磁盘不足时才按规则清理对应 `target/`。本文件不把未执行的性能、soak、人工 GUI、远端 CI 或 release 命令写成通过证据。
