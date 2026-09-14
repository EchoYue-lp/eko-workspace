---
schema_version: 3
lifecycle: completed
supersedes: null
slug: echo-agent全workspace语义治理/重置SDK治理路由
goal: 将 SDK identity 从全项目完成门禁降级为独立漂移监控，并为全 workspace semantic discovery 建立一致的文档和语义入口
ships: 停止创建逐 identity PR，将 9,682 项 inventory 重定位为 SDK 漂移监控和分类 backlog，并把全
  workspace semantic discovery 设为主路线
verify: "正式 ADR、SDK 文档、历史 Plan 状态和语义材料对新的治理单位保持一致；Plan 07/08 保留各自不可变的历史 design
  revision 且 lifecycle=completed；PR #23 squash 的双前置连续性、strict snapshot 和 change
  evidence 全部通过，semantic-status 的下一入口包含 semantic-discover"
design_ref: null
delivery_ref: docs/supreme/plans/2026-09-13T1448-echo-agent全workspace语义治理/delivery-map.md#sdk-governance-route-reset
todos:
  - id: record-sdk-governance-decision
    files:
      - echo-agent/docs/adr/0031-sdk-identity-governance-scope.md
      - echo-agent/docs/sdk/README.md
      - echo-agent/docs/en/README.md
      - echo-agent/docs/zh/README.md
      - docs/MASTER-PLAN.md
    summary: 以 ADR 和正式文档冻结 SDK identity 的监控定位及全 workspace 语义治理主单位
    verify: 文档明确区分 Host facade parity、SDK identity telemetry 和全 workspace semantic
      closure，且中英文 ADR 索引一致
  - id: close-sdk-plan-state
    files:
      - echo-agent/docs/supreme/specs/2026-09-04-source-first-multilanguage-sdk-runtime/plans/plan_07_实现Facade全Feature适配.md
    summary: 将已被 Plan 08 和当前 main 交付事实证明完成的 Plan 07 从 ready 修正为 completed
    verify: Plan 07 与 Plan 08 均记录 lifecycle=completed 并保留各自历史 design_revision；不存在正式
      plan-09 identity 执行计划
  - id: restore-sdk-semantic-continuity
    files:
      - echo-agent/.echo-semantic
    summary: "补充 PR #23 squash 的双前置 continuity evidence，并刷新受文档变化影响的 SDK 局部语义快照"
    verify: f12563c3 与 37313cd5 到候选结果的 continuity report 无
      conflicted/missing/unknown，strict snapshot 与高风险 change evidence 返回 0
artifact_id: plan:d32d81e6-ac46-4ab0-b4d6-cbd0c97076f5
design_revision: null
---
## Context

- PR #23 已作为最后一个逐 identity SDK 切片 squash merge 到 `echo-agent/main@b21aba01`，当前 canonical 指标为 `5,606 / 9,682`。
- 余下 `4,076` 个未完成 canonical identity 全部是 `surface=intrinsic`；standard、core、family、bridge、invoke 和 value 路由已完成。
- squash continuity 比较保全了 428 项义务，但发现 7 个只存在于目标 main 旧 `evidence.sdk-contracts` 的 source dependency；需要显式退役旧证据引用，不能忽略失败。
- Plan 07 和 Plan 08 分别绑定各自执行时的设计摘要；当前设计已演进，历史 completed Plan 不得改写为当前 design revision 或重新作为可执行计划。
- 现有全 workspace baseline 仍为 open、`coverage=[]`、0 Assets；下一主路线必须是 semantic discovery。

## Approach

- 新增一个 framework ADR，明确 Capability、Behavior、Rule、State Authority、Lifecycle、Finding、Evidence 是项目治理单位，canonical identity 仅用于 SDK inventory/drift。
- 保持 SDK parity manifest 和 CI，不删除 `9,682` 项 inventory；不再把 `all-three-done` 作为全项目完成或逐项 PR 队列。
- Plan 07 的实现已经被 Plan 08 和 main 交付事实覆盖，将其 lifecycle 从 `ready` 修正为 `completed`；Plan 07/08 的历史 design revision 保持不可变，不伪造 plan-09。
- 通过 semantic continuity Evidence 显式退役目标 main 的 7 个旧 source dependency，并保留 PR 分支更新后的 `evidence.sdk-contracts`。
- 本 outcome 只修改 ADR、文档、Plan 状态和 `.echo-semantic`；不改 Rust、三语言 SDK、manifest、contract 或 website。

## Global Constraints

- 全项目语义治理主单位固定为 Capability、Behavior、Rule、State Authority、Lifecycle、Finding、Evidence。
- `5,606 / 9,682` 只能作为 SDK inventory telemetry，不代表全项目完成度。
- 剩余 intrinsic 项保留为分类 backlog；只有真实对外 SDK capability 或 Finding 才能形成后续批量计划。
- 不删除或回滚 PR #20 至 #23 已合并的 SDK 能力。
- 不修改 Rust 业务代码、SDK language source、manifest、contract schema 或 `echo-website`。
- 历史 Plan 的 design revision 不可改写；设计摘要漂移只说明它不能作为当前执行来源，不否定已验证的历史交付。
- 历史审计只能作为 discovery 输入；未重锚当前 revision 前不得恢复为当前结论。
- 使用新任务分支；保留 superproject、`echo-agent-cli`、`echo-website` 和其它未跟踪文件的现有用户改动。

## Files

- Create: `echo-agent/docs/adr/0031-sdk-identity-governance-scope.md` — 记录项目语义治理单位、SDK identity 监控定位和 squash 证据处置。
- Modify: `echo-agent/docs/sdk/README.md` — 增加当前 identity 指标、分类和完成度边界。
- Modify: `echo-agent/docs/en/README.md` — 将 ADR 0026 至 0031 补入英文索引。
- Modify: `echo-agent/docs/zh/README.md` — 将 ADR 0026 至 0031 补入中文索引。
- Modify: `docs/MASTER-PLAN.md` — 将 SDK identity 主线改为已收口，并指向全 workspace discovery。
- Modify: `echo-agent/docs/supreme/specs/2026-09-04-source-first-multilanguage-sdk-runtime/plans/plan_07_实现Facade全Feature适配.md` — 仅更新 artifact lifecycle。
- Modify: `echo-agent/.echo-semantic` — 新增 continuity Evidence，更新 SDK 局部对象快照和治理入口说明。

## Reuse

- `echo-agent/contracts/sdk/parity-manifest.json` — 保留为唯一 SDK identity inventory 和 drift 数据源。
- `echo-agent/.echo-semantic/evidence/evidence.sdk-contracts.md` — 保留 PR 分支更新后的 source refs 作为当前 SDK 证据。
- `echo-agent/docs/adr/0028-source-first-multilanguage-sdk-runtime.md` — 保留 ACP/Host/三语言 SDK 架构，不改写其运行时权威。
- `echo-agent/docs/adr/0014-framework-capability-placement.md` — 复用 framework/application 分层判定。
- `echo-agent/docs/adr/0008-canonical-runtime-task-authority.md` — 复用单一任务权威和 plan artifact 原则。
- `echo-agent/.echo-semantic/baseline.md` — 扩展现有局部基线，不建立第二套治理目录。

## Todos

### record-sdk-governance-decision

requirements:
- 用户确认：停止按 identity 数量推进，改为按语义边界和 Finding 推进。
- 用户确认：`5,606 / 9,682` 只代表 SDK facade 映射进度。
- 仓库约束：关键架构和契约决策必须记录 ADR，正式 framework 文档归 `echo-agent`。

interfaces:
- consumes: SDK parity manifest、现有 ADR 0008/0014/0028、PR #23 merge 结果、全 workspace semantic status。
- produces: ADR 0031、SDK 状态说明、中英文 ADR 索引、顶层阶段状态。

steps:

1. 新增 ADR 0031，记录候选方案、最终决策、完成度边界、兼容影响、回滚方式以及 PR #23 双前置连续性处置。
   verify: ADR 具有 Accepted 状态、Context、Options、Decision、Consequences，并明确列出 7 个 continuity obligation 和两个 predecessor revision。
   expected: 后续参与者不能再把 SDK all-three-done 比例当成全项目语义完成度或逐项任务队列。

2. 同步 SDK README、中英文 ADR 索引和顶层 MASTER-PLAN。
   verify: 所有状态说明一致地区分 Host facade parity、SDK identity telemetry、intrinsic backlog 和全 workspace semantic closure。
   expected: 正式文档入口与阶段性计划使用相同进度口径，website 保持未修改。

### close-sdk-plan-state

requirements:
- 当前 main 和 completed Plan 08 已证明 Plan 07 的 adapter 交付，不应继续显示为 ready。
- 用户确认：不再恢复 plan-09 逐 identity 执行路线。

interfaces:
- consumes: Plan 07、Plan 08、SDK delivery map 和当前 main 的 Host route 证据。
- produces: lifecycle=completed 的 Plan 07 artifact，且 Plan 07/08 保留各自历史 design revision。

steps:

1. 使用 Plan artifact CLI 将 Plan 07 lifecycle 原子更新为 completed，并读取 Plan 07/08 的 lifecycle 与 design revision。
   verify: 两份 artifact 均记录 lifecycle=completed 且 design revision 未被重写；当前主题下不存在正式 plan-09 文件。
   expected: 历史计划状态不再把已交付 Host adapter 误投影为当前待执行工作，也不把历史计划伪装成绑定当前设计的新计划。

### restore-sdk-semantic-continuity

requirements:
- Echo Semantic 要求 squash 合并保全两个前置版本的语义义务。
- 当前 continuity report 有 428 preserved 和 7 conflicted source dependency，失败不得跳过。

interfaces:
- consumes: merge base `f12563c33de96b89baf9312807182f9500baa159`、predecessors `f12563c33de96b89baf9312807182f9500baa159` 与 `37313cd5303ca21b4a232335f342b2b59da554df`、ADR 0031、现有 SDK semantic objects。
- produces: `evidence.sdk-pr23-squash-continuity`、刷新后的 source snapshot、通过的 continuity/change-evidence/strict-snapshot 报告。

steps:

1. 为 7 个目标 main 旧 source dependency 写入显式 retired resolution，绑定两个 predecessor fingerprint、ADR 0031 摘要、兼容影响和回滚方式。
   verify: continuity Evidence 结构合法，且每个原 conflicted obligation 都有唯一 resolution。
   expected: 旧证据引用被明确退役，PR 分支的新证据引用继续作为当前事实，不删除行为或公共契约。

2. 刷新受正式文档和 Plan 状态变化影响的 SDK 局部语义对象 source snapshot，并运行语义三层验证。
   verify: 双前置 continuity report 只含 preserved/replaced/retired；strict snapshot、高风险 change evidence 和 semantic-status 均返回可解释结果。
   expected: SDK 局部基线保持 open 但结构和快照一致，下一入口明确为全 workspace semantic-discover。

## Decisions

- SDK inventory 保留完整，不再作为项目级完成门禁。
- 7 个冲突是旧 evidence source dependency 的退役，不是 SDK 行为、公共 API 或运行时权威的删除。
- 历史 completed Plan 保留原 design revision；不通过改写摘要让它重新变成当前执行来源。
- 全 workspace baseline 完成前不发布全项目完成百分比。
- `echo-website` 等真正对外 contract 稳定后再同步。