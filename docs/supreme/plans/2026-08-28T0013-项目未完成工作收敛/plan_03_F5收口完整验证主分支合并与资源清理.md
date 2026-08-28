---
schema_version: 3
slug: 项目未完成工作收敛/plan
goal: 完成 F5 测试证据与测试卫生收口，经过完整验证后将 framework 与 CLI 合并主分支并安全清理废弃资源。
ships: F5 单一 squash 结果、260 项测试 lint 清零、framework/CLI 完整验证证据、主分支合并和废弃
  worktree/branch 清单。
verify: F5/H1/H2/H3 全部有可追溯提交，framework producer 与 CLI consumer 的完整适用门禁全部 exit
  0，独立 review pass，主分支合并无用户改动丢失，清理后仅保留必要主分支和明确保护项。
design_ref: null
todos:
  - id: complete-f5-and-hygiene
    files:
      - echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs
      - echo-agent-cli/echo-agent-app-core/src/workspace/registry.rs
      - echo-agent-cli/echo-agent-app-core/src/subagent_loader.rs
    summary: 完成 F5 characterization 合流和 H1/H2/H3 测试卫生清理。
    verify: F5 单一 squash commit、H1/H2/H3 独占提交、260 命中全部有 disposition 且目标扫描为零。
  - id: run-complete-validation
    files:
      - echo-agent/.github/workflows/rust-ci.yml
      - echo-agent-cli/.github/workflows/rust-ci.yml
      - echo-agent-cli/web-frontend/src
    summary: 执行 framework/CLI/GUI/frontend 完整提交验证和最终 review。
    verify: 所有适用门禁、测试、文档/示例合同和静态 hygiene 扫描真实 exit 0，review pass。
  - id: merge-main
    files:
      - echo-agent/Cargo.toml
      - echo-agent-cli/Cargo.toml
      - echo-agent-cli/docs/MASTER-PLAN.md
    summary: 按 framework producer first 将已验证结果合入两个主分支。
    verify: 两仓 main 包含已验证 SHA，Cargo path 相对，post-merge smoke 通过，用户 dirty patch 未丢失。
  - id: cleanup-merged-resources
    files:
      - docs/MASTER-PLAN.md
    summary: 逐项删除无 owner、clean 且已合并的废弃 worktree、branch 和 target。
    verify: 两仓 worktree/branch 只剩必要主分支与明确保留项，源 checkout/其它项目 worktree/保护 patch 不受影响。
artifact_id: plan:54804e00-2402-4407-80b0-31c72dff270d
design_revision: null
---
## Context

- 当前 P0 integration 已包含 F4 基线 `4462b8aee9a4409fead54d7607d7df34990c0aad`、F5 squash `bb977e37225aa04b30f28e3ec45bbd789fdcc1fa`、runtime_state hygiene `843a13aabc35afa8cf6c6106656e377f89337695`；H2 `8b03ec8`、H3 `15cc4d9` 已提交，H1 尚未完成。
- 用户已授权清理指定 integration target、重跑 combined gate、将 F5 四个小任务压缩为一次提交、完成完整验证后合并主分支，并清理两个子仓库的废弃资源。
- 本里程碑只收口 F5、测试卫生、完整验证、主分支合并和安全清理；F6、R1、examples 实际重组、website 生产同步和长时 soak 后置。

## Approach

- H1/H2/H3 在独立 worktree 只修改 test-only panic API；F5 characterization 保持单一 squash，hygiene 保持后续提交。
- framework producer 先于 CLI consumer；所有 Cargo 验证串行执行，任何已执行失败都必须修复并复跑。
- 主分支合并前保存用户 dirty patch/hash；已被最终历史覆盖的 dirty hunk 只留证据后清理，其余保留保护分支或 patch。
- 只删除 clean、无 active owner、无 unique unmerged commit 的 worktree/branch；禁止整体删除 `/Users/ls/.codex/worktrees`。

## Global Constraints

- 不执行 10k/100k、10 分钟、1 小时或 2 小时 soak；这些仍属于 Final Gate。
- `echo-agent`、`echo-agent-cli` 不引入 SQLite、第二 runtime/receipt/mailbox/status authority、旧角色术语或 mode 替身。
- 任何已执行失败都不得跳过；临时 symlink 不得进入提交。
- F5 squash 历史不重写；H1/H2/H3 与 `843a13a` 作为独立 hygiene 提交。
- 主分支合并顺序为 framework → CLI；CLI 必须使用已提交 framework main 的相对 Cargo path。

## Files

- Modify: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/store.rs` — H1 test-only cleanup。
- Modify: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/file_store.rs` — H1 test-only cleanup。
- Modify: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/event_rebuild.rs` — H1 test-only cleanup。
- Modify: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/ledger.rs` — H1 test-only cleanup。
- Modify: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/review.rs` — H1 test-only cleanup。
- Modify: `echo-agent-cli/echo-agent-app-core/src/tasks/task_runtime/memory_bridge.rs` — H1 test-only cleanup。
- Modify: `echo-agent-cli/echo-agent-app-core/src/workspace/registry.rs` — H2 test-only cleanup。
- Modify: `echo-agent-cli/echo-agent-app-core/src/workspace/migration.rs` — H2 test-only cleanup。
- Modify: `echo-agent-cli/echo-agent-app-core/src/workspace/layout.rs` — H2 test-only cleanup。
- Modify: `echo-agent-cli/echo-agent-app-core/src/workspace/mod.rs` — H2 test-only cleanup。
- Modify: `echo-agent-cli/echo-agent-app-core/src/utils.rs` — H2 test-only cleanup。
- Modify: `echo-agent-cli/echo-agent-app-core/src/diff.rs` — H2 test-only cleanup。
- Modify: `echo-agent-cli/echo-agent-app-core/src/context_window.rs` — H2 test-only cleanup。
- Modify: `echo-agent-cli/echo-agent-app-core/src/subagent_loader.rs` — H3 test-only cleanup。
- Modify: `echo-agent-cli/echo-agent-app-core/src/infra.rs` — H3 test-only cleanup。
- Modify: `echo-agent-cli/echo-agent-app-core/src/state/reliability_contracts.rs` — H3 test-only cleanup。
- Modify: `echo-agent-cli/echo-agent-app-core/tests/runtime_state_e2e.rs` — existing hygiene `843a13a`。
- Modify: `docs/MASTER-PLAN.md` — milestone and cleanup record。
- Modify: `echo-agent-cli/docs/MASTER-PLAN.md` — final CLI merge record。
- Modify: `echo-agent/Cargo.toml` — producer/main verification。
- Modify: `echo-agent-cli/Cargo.toml` — relative path verification。
- Modify: `echo-agent-cli/web-frontend/src` — post-merge frontend smoke。
- Modify: `echo-agent/.github/workflows/rust-ci.yml` — reproducible CI fix only if found。
- Modify: `echo-agent-cli/.github/workflows/rust-ci.yml` — reproducible CI fix only if found。

## Reuse

- `docs/supreme/plans/2026-08-28T0013-项目未完成工作收敛/plan_01_冻结统一执行路线.md` — overall dependency order。
- `docs/supreme/plans/2026-08-28T0013-项目未完成工作收敛/plan_02_F2F3合流与F4基线冻结.md` — coordinator ownership。
- P0 integration F5/hygiene commits and existing framework/CLI/website gates。

## Todos

### complete-f5-and-hygiene

requirements:
- 用户要求 F5 四个小任务完成并合并为一次 F5 提交。
- 任何已执行测试 lint 失败必须修复，测试区域不得保留 panic API。

interfaces:
- consumes: F5 `bb977e3`、`843a13a`、H2 `8b03ec8`、H3 `15cc4d9`、H1 pending。
- produces: F5 单一 squash、独立 hygiene commits、clean integration tree。

steps:

1. 完成 H1，核对 H1/H2/H3 文件集合互不重叠，解释原始 260 命中。
   verify: 三 lane clean、commit 可解析、只含 test-only 修改、目标扫描为零。
   expected: 无跨 lane 覆盖或静默跳过。
2. 按 F5 squash、843a13a、H1、H2、H3 顺序合入 integration，移除临时 symlink。
   verify: F5 仍为单一提交，hygiene 可回溯，`git diff --check` 通过。
   expected: integration 历史和职责边界清晰。

### run-complete-validation

requirements:
- 用户要求合并前完全验证所有适用内容。
- framework/CLI/GUI/frontend 提交门禁必须真实 exit 0。

interfaces:
- consumes: clean integration history、verified framework producer、CLI consumer、generated DTO。
- produces: complete validation evidence and independent review pass。

steps:

1. 按 framework → CLI 执行 fmt、两套 clippy、严格 panic lint、workspace tests、no-default、GUI、frontend、docs/examples contracts。
   verify: 所有适用命令 exit 0；失败有修复和复跑记录。
   expected: 不以 focused gate 或旧证据替代完整验证。
2. 执行 authority、SQLite、旧角色术语、绝对 worktree path、InteractionMode 和 generated DTO 扫描。
   verify: 无第二 runtime/store/reducer，F4/F5 合同保持，CLI 不启用 SQLite。
   expected: 架构卫生与编译门禁一致。
3. 对 integration 全部新增和 hygiene diff 做最终只读 review。
   verify: reviewer pass，工作树 clean，命令/版本/资源证据齐全。
   expected: 只有已审内容进入主分支。

### merge-main

requirements:
- 用户要求完整验证结束后合并主分支。
- framework-first、相对 Cargo path、用户 dirty patch 保护。

interfaces:
- consumes: framework verified SHA、CLI verified SHA、dirty-path snapshot。
- produces: 已验证的 `echo-agent/main` 和 `echo-agent-cli/main`。

steps:

1. 保存主 checkout dirty patch/hash，确认无 active build 和目标 worktree dirty。
   verify: patch 可恢复，5 个 dirty 文件逐项有处理结论。
   expected: 不覆盖用户改动。
2. 合入 framework main，再合入 CLI main；不在源 checkout 解冲突。
   verify: parent/child SHA、Cargo path、branch tracking、status 可复核。
   expected: CLI 指向已验证 framework producer。
3. 合并后运行最小 post-merge smoke 和 status 检查。
   verify: smoke exit 0，主分支无未预期改动。
   expected: main 与 integration 结果一致。

### cleanup-merged-resources

requirements:
- 用户要求清理两个子仓库的废弃 worktree、branch 和构建废物。
- 删除仅限 clean、无 active owner、无 unique unmerged commit 的对象。

interfaces:
- consumes: merged mains、worktree/branch list、active thread snapshot、dirty protection record。
- produces: clean worktree/branch list and retained-item report。

steps:

1. 枚举 `/Users/ls/.codex/worktrees` 中属于两个子仓库的对象，按 owner、dirty、unique commit、project 分类。
   verify: 每个待删对象有证据，其它项目对象排除。
   expected: 不误删当前任务或其它项目。
2. 删除已合并/废弃的 F0/F1/F2/F3/F4/F5/H1/H2/H3/integration worktree、local branch 和无 owner target。
   verify: 两仓 worktree/branch 只剩必要主分支和明确保留项，无 dangling registration/process。
   expected: 其它项目 worktree 不变。
3. 最终核对两仓、superproject status、gitlink、保护 patch，并记录删除/保留清单。
   verify: 主分支和可清理 worktree clean，gitlink 与 child SHA 一致。
   expected: 下一里程碑从单一主分支开始。

## Decisions

- 本里程碑闭环边界为 F5/test hygiene/complete validation/main merge/scoped cleanup。
- F6、R1、examples 实际重组、website 生产同步和长时 Final Gate 后置。
- `/Users/ls/.codex/worktrees` 禁止整体删除；dirty、active 或 unique unmerged 对象只保留并报告。