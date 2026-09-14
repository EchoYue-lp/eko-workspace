---
schema_version: 3
supersedes: null
slug: echo-agent全workspace语义治理/合并最后一个SDK身份切片
goal: 结束逐 identity SDK 切片主线，并为全 workspace 语义治理建立可恢复的干净 main 切换点
ships: "将 CI 全绿的 PR #23 作为最后一个逐 identity SDK 切片 squash merge 到 echo-agent
  main，并更新 superproject 的 echo-agent 指针"
verify: "PR #23 为 MERGED，echo-agent origin/main 与 superproject 的 echo-agent
  gitlink 指向该 squash 结果，canonical SDK 指标为 5,606 / 9,682，且没有继续创建逐 identity 的 PR
  #24"
design_ref: null
delivery_ref: docs/supreme/plans/2026-09-13T1448-echo-agent全workspace语义治理/delivery-map.md#sdk-final-slice-merge
todos:
  - id: squash-merge-memory-policy-slice
    files:
      - echo-agent
    summary: "复核并 squash merge PR #23，更新本地 main 和 superproject gitlink，同时保留其它工作树改动"
    verify: "PR #23 的最终 head 与全绿检查被复核后变为 MERGED；main manifest 可重算得到 5,606 /
      9,682，superproject 只更新 echo-agent gitlink"
artifact_id: plan:41fddfae-171c-4e4e-9192-078e440d8fbd
lifecycle: completed
design_revision: null
---
## Context

- `echo-agent` 的 PR #20、#21、#22 已进入 main；PR #23 是最后一个仍打开的逐 identity Memory Policy 切片。
- PR #23 当前代码可合并且 10 个 CI job 全部成功，`BLOCKED` 来自 main ruleset 的提交签名要求，不是冲突或测试失败。
- 当前分支的 `5,606 / 9,682` 只表示 canonical SDK identity 在 TypeScript、Python、Java 三端均为 done，不表示全 workspace 语义治理进度。

## Approach

- 保留 PR #23 的三提交历史作为审阅证据，通过 GitHub squash merge 生成 main 的单一任务提交。
- 仅在合并前重新核对 head、base、mergeability、全部 required checks 和 ruleset blocker；事实变化时停止，不猜测。
- 合并后将本地 `echo-agent/main` 更新到远端结果，再只更新 superproject 的 `echo-agent` gitlink。
- 本 Plan 完成后停止创建逐 identity PR；SDK 路由重置和全 workspace discovery 由 delivery map 的后续独立 outcome 承担。

## Global Constraints

- 不重写、rebase 或修改 PR #23 已通过 CI 的业务代码和 SDK 合同。
- 不创建 PR #24 或新的逐 identity 切片。
- SDK identity 计数只能作为 SDK inventory 指标，不得作为全项目语义治理完成度。
- 子仓库先合并，superproject 后更新 gitlink；不得让 gitlink 指向未推送提交。
- 只更新 `echo-agent` gitlink，保留 `echo-agent-cli`、`echo-website`、`docs/supreme/specs/eko-unified-task-subagent-surface/` 和 `rust_out` 的现有用户改动。
- 本 outcome 不修改 `echo-website`，也不开始业务代码修复或全仓 discovery。

## Files

- Modify: `echo-agent` — 更新 submodule checkout 与 superproject gitlink，使其指向 PR #23 的 main squash 结果。

## Reuse

- `echo-agent/contracts/sdk/parity-manifest.json` — 现有 canonical identity 与三语言状态事实源，继续用于可重算指标，不另建清单。
- `echo-agent/.echo-semantic/maps/map.sdk-facade-parity.md` — 保留现有 SDK 局部语义边界，不把它扩成全项目治理主单位。
- `echo-agent/docs/supreme/specs/2026-09-04-source-first-multilanguage-sdk-runtime/plans/plan_08_完成FacadePublicAPIParity.md` — 复用已完成的 Host facade 路由结论，不 reopen。
- `AGENTS.md` — 复用 child-first、任务级 squash merge、门禁全绿和保留用户工作树改动的仓库规则。

## Todos

### squash-merge-memory-policy-slice

requirements:
- 用户确认：PR #23 已全绿且无需回滚，应作为最后一个逐 identity SDK 切片收口。
- 用户确认：后续停止按 SDK identity 数量推进，转向全 workspace 语义发现。
- 仓库约束：子仓库任务通过门禁后 squash merge，并在之后更新 superproject gitlink。

interfaces:
- consumes: GitHub PR #23 head `37313cd5303ca21b4a232335f342b2b59da554df`、base `echo-agent/main`、required check 结果、`contracts/sdk/parity-manifest.json`。
- produces: PR #23 的 main squash commit、更新后的本地 `echo-agent/main`、指向该提交的 superproject `echo-agent` gitlink。

steps:

1. 重新读取 PR #23 的 head、base、mergeability、review/ruleset 状态和全部 required checks，并确认本地 feature 分支与远端一致。
   verify: PR 仍以 `37313cd5303ca21b4a232335f342b2b59da554df` 为 head，代码 `MERGEABLE`，全部检查成功，唯一 blocker 仍是 required signatures。
   expected: 合并输入与本计划调查证据一致；若 head、base、检查或 blocker 改变，则停止并重新评估。

2. 使用 GitHub squash merge 将 PR #23 合入 main；只有 required-signatures ruleset 需要时才使用当前账户已有的 admin bypass。
   verify: PR 状态为 `MERGED`，远端 main 新增一个可追溯到 PR #23 的 squash commit，未产生第二套 SDK 实现或额外 identity PR。
   expected: Memory Policy slice 以一个任务级提交进入 main，原三个 unsigned feature commits 不被重写。

3. 更新本地 `echo-agent/main` 到远端 squash 结果，重算 canonical 三语言 done 指标，并在 superproject 只暂存 `echo-agent` gitlink。
   verify: main 上可重算得到 canonical total `9,682`、all-three-done `5,606`；superproject diff 只包含预期的 `echo-agent` 指针变化，所有其它已有改动保持原样。
   expected: 获得可用于后续 route reset 和 semantic discovery 的干净 framework main 基准，且没有覆盖任何用户工作。

## Decisions

- PR #23 是最后一个逐 identity 切片；其后不再用单个类型、字段、方法或枚举变体拆 PR。
- required-signatures blocker 通过 GitHub squash merge 处理，不在本地重写已经全绿的提交。
- SDK inventory 保留，但从全项目完成门禁降级为独立 SDK 漂移监控与分类 backlog。