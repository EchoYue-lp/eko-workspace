---
schema_version: 3
slug: 项目未完成工作收敛/plan
goal: 将已通过独立门禁的 F2/F3 lane 以单一 authority 合流，并冻结可供 F4 使用的干净基线。
ships: 一条包含 framework F2 bounded task_list、CLI F2 Task graph/Todo authority、CLI
  F3 Agent controls 的干净 integration SHA，以及单次生成的 DTO snapshot 和 F4 baseline
  evidence。
verify: 在独立 integration worktree 中完成 child-first source merge、ADR 0015/0016
  唯一、generated DTO 单次生成、Cargo.lock/path hygiene、framework/CLI/GUI/frontend 适用门禁和
  review 全绿，且主 checkout 既有脏状态哈希不变；不执行 F4 mode 删除或发布动作。
design_ref: null
todos:
  - id: integrate-f2-f3-freeze-f4-baseline
    files:
      - echo-agent/echo-orchestration/src/tasks/task_tools.rs
      - echo-agent/docs/en/09-tasks.md
      - echo-agent/docs/zh/09-tasks.md
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/register.rs
      - echo-agent-cli/echo-agent-app-core/src/agent_control.rs
      - echo-agent-cli/echo-agent-app-core/src/state.rs
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/subagent_control.rs
      - echo-agent-cli/docs/adr/0015-task-graph-status-authority.md
      - echo-agent-cli/docs/adr/0016-agent-control-tools.md
      - echo-agent-cli/docs/README.md
      - echo-agent-cli/docs/architecture.md
      - echo-agent-cli/docs/features.md
      - echo-agent-cli/docs/MASTER-PLAN.md
      - echo-agent-cli/src/tui/events.rs
      - echo-agent-cli/web-frontend/src/generated/index.ts
      - echo-agent-cli/web-frontend/src/generated/AgentTarget.ts
      - echo-agent-cli/web-frontend/src/generated/ConversationTarget.ts
      - echo-agent-cli/web-frontend/src/generated/TaskSubagentTarget.ts
      - echo-agent-cli/Cargo.lock
    summary: 合流 F2/F3 source、生成唯一 DTO snapshot，并冻结未实施 F4 的 integration baseline。
    verify: lane commits 与 review 全绿，integration worktree
      source/generated/Cargo.lock/path clean，framework/CLI/GUI/frontend 适用门禁全部
      exit 0。
artifact_id: plan:8785d4d1-c520-47da-82b3-ce14efe6443b
design_revision: null
---
## Context

- plan_01 已冻结唯一依赖链；F2/F3 现在分别具备本地 commit、clean worktree、专项门禁和独立 review pass，具体验收证据仍需在 coordinator integration worktree 重新核对。
- 当前可消费的 lane 输入为 framework F2 `302453b174086c3795dc026d16eeb668ecc66bed`（包含必需实现 `8c4aca1b27bda649f1f91970f7037f6d97a8365a`）、CLI F2 `b5be9608acd725784da9abb4b215d4820cfa3441`、CLI F3 `fbafb75e728003bbbcae2c2cd73431cccd530954`；F1 CLI 基线为 `20e7584b`，F1 framework 基线为 `9bbca5e`。
- 主 checkout 保留用户既有脏状态，不能作为合流工作树；lane 产生的 generated TS 不是最终事实源，必须在合并后的 Rust 真理源上单次生成。

## Approach

- 先在 framework 专用集成工作树应用 F2 bounded `task_list` producer，再在 CLI coordinator integration worktree 从 `20e7584b` 合入 F2/F3；保持所有 Cargo path 为相对路径，不把 lane-local absolute path 或 generated 噪声带入。
- coordinator 独占 ADR 编号、Cargo.lock 和 generated DTO snapshot：F2 ADR 保留 `0015`，F3 ADR 在集成时重命名为 `0016`；Rust source 合流后只运行一次 ts-rs export，再用 Prettier 规范化并按语义 diff 核对。
- 合流完成后执行 framework、CLI Rust、GUI 条件矩阵和 frontend 的适用门禁；发现任何失败都在 integration worktree 修复并重新 review。只冻结可供 F4 使用的 SHA，不实施 F4 的 `InteractionMode` 删除。

## Global Constraints

- 本计划只交付 F2/F3 合流与 F4 冻结基线；不删除或改名 `InteractionMode`，不执行 F5/F6/R0/R1/R2/R3，不运行长时 soak、10k/100k、远端 CI、push、merge、publish、gitlink 更新或 worktree cleanup。
- framework producer 必须先于 CLI consumer；CLI `Cargo.toml` 的 framework path 保持相对路径 `../echo-agent`。framework 与 CLI 的 Cargo.lock 由 coordinator 在 integration worktree 单独核对，未发生 manifest 变化时保持 F1 基线内容。
- 单一 authority：framework Task graph/revision/validator、EKO Todo projection、Conversation AgentRouter、SubagentControlService、receipt lifecycle 和 cursor owner 不得新增第二实现。
- 只使用 `Subagent` 术语；CLI 不启用 SQLite；所有字符串预览 UTF-8 安全，生产与测试不引入无证明的 unwrap/expect/直接索引/panic。
- generated TS 只从合流后的 Rust `#[ts(export)]` 真理源生成一次；lane-local generated 修改全部视为投影噪声，除非语义 diff 明确属于本次合流。
- 保留 F2 review 的 paused-row UI affordance Minor、F3 schema/truncated/interrupt metadata Minor，以及 ADR 已记录的底层 full-vector query R1/P0 residual；这些不在本计划扩展范围，但不得被遗漏或伪装成已解决。
- 主 checkout 的既有脏状态必须在合流前后哈希不变；integration worktree 必须独立且最终 clean。所有本地 commit 显式使用 `git -c commit.gpgsign=false`。

## Files

- Modify: `echo-agent/echo-orchestration/src/tasks/task_tools.rs` — 采用已验证的 bounded `task_list` producer。
- Modify: `echo-agent/docs/en/09-tasks.md` — 记录 limit、opaque cursor、detail level 和 page metadata。
- Modify: `echo-agent/docs/zh/09-tasks.md` — 同步中文任务工具合同。
- Modify: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/types.rs` — 合入 F2 canonical TaskStatus/PlanRevision 投影。
- Modify: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs` — 合入 Task graph/reorder projection 与 parity tests。
- Modify: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/register.rs` — 合入 F3 tools registration wiring。
- Create: `echo-agent-cli/echo-agent-app-core/src/agent_control.rs` — 合入 F3 Agent control adapter。
- Modify: `echo-agent-cli/echo-agent-app-core/src/state.rs` — 合入 shared router/registry/delivery authority wiring。
- Modify: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/subagent_control.rs` — 合入 typed duplicate/replay helpers。
- Create: `echo-agent-cli/docs/adr/0015-task-graph-status-authority.md` — 保留 F2 ADR 编号。
- Create: `echo-agent-cli/docs/adr/0016-agent-control-tools.md` — 集成时将 F3 ADR 从 lane-local 0015 重编号为 0016。
- Modify: `echo-agent-cli/docs/README.md` — 合并 F2/F3 文档索引。
- Modify: `echo-agent-cli/docs/architecture.md` — 合并 authority/adapter 说明。
- Modify: `echo-agent-cli/docs/features.md` — 合并 Agent control feature 说明。
- Modify: `echo-agent-cli/docs/MASTER-PLAN.md` — 记录已合入的 F2/F3 与 F4 冻结断点。
- Modify: `echo-agent-cli/src/tui/events.rs` — 保留 canonical Todo projection 与顺序 parity。
- Modify: `echo-agent-cli/web-frontend/src/generated/index.ts` — 接收单次 Rust 真理源生成的 DTO 导出。
- Modify: `echo-agent-cli/web-frontend/src/generated/AgentTarget.ts` — 接收 F3 target DTO projection。
- Modify: `echo-agent-cli/web-frontend/src/generated/ConversationTarget.ts` — 接收 Conversation target DTO projection。
- Modify: `echo-agent-cli/web-frontend/src/generated/TaskSubagentTarget.ts` — 接收 workspace-qualified Subagent target DTO。
- Modify: `echo-agent-cli/Cargo.lock` — 仅在合流确实改变依赖解析时由 coordinator 核对和更新。

## Reuse

- `docs/supreme/plans/2026-08-28T0013-项目未完成工作收敛/plan_01_冻结统一执行路线.md:158-210` — F2/F3 当前基线、P0 退出门和 coordinator ownership。
- `docs/2026-08-26-agent-interaction-convergence-plan.md:298-328` — Iteration 2 Task/Plan/Todo authority 与 bounded task_list 合同。
- `docs/2026-08-26-agent-interaction-convergence-plan.md:330-371` — Iteration 3 Agent control、typed target、exact-once 和 generation 合同。
- `echo-agent/echo-orchestration/src/tasks/task_tools.rs:144-220` — 已验证 bounded TaskListTool 与现有 TaskRevisionService。
- `echo-agent-cli/echo-agent-app-core/src/agent_router.rs` — durable Conversation mailbox、target generation 和 delivery owner。
- `echo-agent-cli/echo-agent-app-core/src/state.rs:3313-3325` — AppState owned message enqueue + delivery wake authority。
- `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/subagent_control.rs` — exact attempt、typed receipt replay 和 mailbox settlement owner。
- `echo-agent/echo-agent-learning/tests/documentation_contract.rs` — 后续 docs/examples contract 的既有门禁，不在本计划新增 verifier。

## Todos

### integrate-f2-f3-freeze-f4-baseline

requirements:
- 用户明确要求：等待 F2/F3 各自完成门禁和 review 后，只创建本主题下一序号计划，范围限定为合流与 F4 冻结基线。
- plan_01 的 P0 F2/F3 合流退出门：两 lane clean、双 review pass、集成分支完整适用门禁通过、干净 SHA 冻结。
- AGENTS.md 的 framework-first、单一 authority、generated snapshot、文档同步、Cargo path、GPG 和主 checkout 保护约束。

interfaces:
- consumes: framework F2 commit `302453b174086c3795dc026d16eeb668ecc66bed`、CLI F2 commit `b5be9608acd725784da9abb4b215d4820cfa3441`、CLI F3 commit `fbafb75e728003bbbcae2c2cd73431cccd530954`、F1 framework/CLI 基线和两份独立 review evidence。
- produces: framework-first integration source、F2 ADR 0015、F3 ADR 0016、单次 generated DTO snapshot、Cargo.lock/path 复核记录、适用门禁证据和供 F4 使用的干净本地 integration SHA。

steps:

1. 在不触碰主 checkout 的前提下核对三份 lane commit 的 parent、clean status、review pass 和未跟踪 generated/临时文件；记录主 checkout 既有 dirty path 的哈希快照，并从 CLI `20e7584b` 创建独立 coordinator integration worktree。
   verify: framework/CLI lane 的 `git status --porcelain` 为空；lane SHA 与本计划输入一致；integration worktree HEAD 等于 `20e7584b`；主 checkout dirty path 哈希前后一致；Cargo path 不含 worktree absolute path。
   expected: 只有独立 integration worktree 接收合流写入，未知用户改动和 lane-local generated 噪声不被覆盖或带入。

2. 先在 framework integration worktree 应用 `302453b`（保留其 bounded `task_list` 及中英文文档），再在 CLI integration worktree 按 owner 合入 F2 `b5be960` 与 F3 `fbafb75`；解决 `docs/README.md`、`docs/architecture.md` 和 ADR 编号冲突，F2 保留 `0015`、F3 重命名 `0016`，不合入 lane-local generated 文件或 `.supreme` 日志。
   verify: source diff 只包含两条 lane 的预期文件；ADR 编号唯一；framework commit 位于 CLI consumer 之前；`git diff --check` 通过；`git diff -- Cargo.lock` 为空或有明确 manifest 变化理由。
   expected: framework、CLI、ADR 和文档各只有一个 authority，没有第二 task/status/router/store/reducer，Cargo.lock 不被无关噪声改写。

3. 从合流后的 Rust 真理源执行一次 ts-rs export（使用 app-core 已有 export tests/standard workspace test 命令），再运行 frontend Prettier；随后执行仓库规定的 framework 与 CLI 提交门禁及 GUI/frontend 条件矩阵，必要时在 integration branch 修复并重新 review。
   verify: generated DTO 只包含预期 F2/F3 语义（PlanTask/TaskPlan 删除、Todo/status/retry 更新、Agent target DTO 新增）；framework 与 CLI 的 `cargo fmt --all -- --check`、all-target/all-feature clippy、strict panic/unwrap/expect clippy、workspace tests、CLI app-core no-default、GUI check/test、frontend Prettier/tests/build 全部 exit 0。
   expected: 合流后的所有 Rust、generated TS、GUI/TUI/CLI consumers 和 docs 在同一 source snapshot 上通过，无 full-gate 失败被静默跳过。

4. 在 integration worktree 中执行独立实现 review，核对 F2/F3 变更与 lane review 的 traceability，并冻结只供 F4 消费的干净本地 SHA；记录 F2 paused-row Minor、F3 schema/truncated/interrupt metadata Minor 和 R1/P0 full-vector residual，不在本阶段修改这些后续范围。
   verify: reviewer 返回 pass；integration worktree `git status --porcelain` 为空；F4 前置 characterization 显示 `InteractionMode` 仍存在且未被改名/删除；普通 chat/task tool admission 的基线测试通过；无 push、merge、gitlink、长时 soak 或发布副作用。
   expected: 获得唯一、可复现、未混入 F4 实现的 integration SHA，plan_03/F4 可以直接以该 SHA 为输入。

## Decisions

- framework F2 的必需 producer 使用 `8c4aca1b27bda649f1f91970f7037f6d97a8365a`；若 coordinator 需要 bounded cursor 额外证据，可包含其子孙 `302453b174086c3795dc026d16eeb668ecc66bed`，但不得把可选测试误当作新的 authority。
- F2 CLI 以 `b5be9608acd725784da9abb4b215d4820cfa3441` 为唯一 lane tip，F3 CLI 以 `fbafb75e728003bbbcae2c2cd73431cccd530954` 为唯一 lane tip；后续合流任何源码变化都必须重新 review。
- 跨 workspace 的 Agent control 在本计划中沿用 F3 已明确的 scoped fail-closed 语义；真正全量 bounded query API 留在 R1/P0，不在 F4 冻结前扩张。
