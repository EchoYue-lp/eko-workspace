---
schema_version: 3
supersedes: null
slug: echo-agent-Framework-concept-convergence/修复READMEfeature表
goal: 让双语 README 的公开 feature 表与 root Cargo manifest 的真实 feature 集合保持一致
ships: 双语 README 的 feature 表只列 Cargo manifest 真实 feature，Task API 的 core
  可用性不再被伪装成不存在的 tasks feature
verify: 旧 README 在 Cargo-derived feature contract 中稳定失败并只报告多余 tasks；修复后双语表覆盖
  manifest 除 default 外的全部 feature，documentation contract、formatter、相关
  Clippy/check、semantic strict/change-evidence、92/92 Issue 对账和独立复审全部通过
design_ref: docs/supreme/specs/2026-09-14T0500-echo-agent-Framework-concept-convergence/design.md
delivery_ref: docs/supreme/specs/2026-09-14T0500-echo-agent-Framework-concept-convergence/plans/delivery-map.md#repair-readme-feature-table
todos:
  - id: derive-feature-table-contract
    files:
      - echo-agent/echo-agent-learning/tests/documentation_contract.rs
    summary: 扩展现有文档合同，从 Cargo metadata 校验双语 README feature 表的精确集合
    verify: 合同以 root echo_agent package 的 features map 为权威，排除 default 后对缺失、多余和重复表项
      fail closed
  - id: correct-bilingual-feature-table
    files:
      - echo-agent/README.md
      - echo-agent/README.zh.md
    summary: 移除不存在的 tasks feature，并在表旁明确 Task API 属于 framework core
    verify: 两份 README feature 集合与 Cargo metadata 一致，Task API 无需 feature
      的语义对等，workspace topology 与 demo34 命令不变
  - id: close-feature-table-finding
    files:
      - echo-agent/.echo-semantic
    summary: "写入 repair、verification 与独立复审证据并只关闭 Finding #79"
    verify: "finding.public-feature-table-drift 具备三类关闭 refs；其它 73 个 open Finding
      状态不变；Issue #79 保持 OPEN 等待远端 main"
artifact_id: plan:b461207e-1fcd-478e-88b9-b61ca51f9832
lifecycle: completed
design_revision: sha256:769b79d4599ee82922893785238c59a3131bab90b5bc250a6ae8f7aea5d0c875
---
## Approach

- 在新的 `fix/Echoyue/readme-feature-table` 分支从当前 `53accbac` 基线执行；修改前重新记录 semantic-preflight，首个有效差异后执行 semantic-diff。
- 复用上一 outcome 已建立的 `cargo metadata` helper，在 `WorkspacePackage` 中读取结构化 `features` map；只选择 workspace root `echo_agent` package，排除 `default` 后形成公开 feature expected set。
- 从两份 README 各自唯一的 feature table section 提取反引号包裹的第一列 feature 名；对 section 缺失、表项重复、manifest feature 缺失和 README 多余 feature fail closed。
- 先在旧 README 上取得只报告多余 `tasks` 的有效 red，再删除两表的 `tasks` 行，并在表旁用中英文明确 Task API 属于 framework core、没有独立 feature。
- 独立 review 后写入 rereview Audit并只把 `finding.public-feature-table-drift` 标为 resolved；GitHub Issue #79 在本地提交进入远端 main 前保持 OPEN。

## Global Constraints

- Root `Cargo.toml`/Cargo metadata 是 feature topology 唯一权威；README 和测试只能消费，不得维护第二份 expected feature 清单。
- 本 Plan 只修复 #79；不得修改 #80 的 demo34 命令、其它示例命令、workspace package topology 或其它 open Finding。
- `default` 是 Cargo 内部空 feature key，不作为可显式启用能力行；`full` 和其余真实 feature 必须各出现一次。
- Task API 属于 framework core，没有 `tasks` feature；不得为兼容错误文档而新增 Cargo feature alias。
- 英文与中文 README 的 feature 名集合和 Task core 说明必须语义对等；描述列的既有非本问题内容不批量重写。
- 不新增依赖、public Rust API、runtime 状态、协议或 SDK identity；`echo-agent-cli`、`echo-website`、examples 与 Cargo manifests 不修改。
- Issue #79 已存在且 marker 唯一；发现新独立语义问题时先建立唯一 Issue/Finding，再决定是否进入当前范围。

## Files

- Modify: `echo-agent/echo-agent-learning/tests/documentation_contract.rs` — 增加 Cargo-derived feature table 集合合同。
- Modify: `echo-agent/README.md` — 删除不存在的 `tasks` 行并就近说明 Task API 属于 core。
- Modify: `echo-agent/README.zh.md` — 与英文 README 对等修正 feature 表和 Task core 说明。
- Modify: `echo-agent/.echo-semantic` — 刷新 workspace map/docs asset/source snapshot，增加 repair/verification/rereview 并关闭 Finding #79。

## Reuse

- `echo-agent/Cargo.toml` — root `[features]` — feature topology 唯一事实源。
- `echo-agent/echo-agent-learning/tests/documentation_contract.rs` — `workspace_packages`、`WorkspacePackage`、`markdown_section` 与 `regex` — 扩展既有结构化 metadata 和 Markdown 检查。
- `echo-agent/.echo-semantic/maps/map.workspace-architecture.md` — `feature-topology` — 已定位 #79 与关闭目标。
- `echo-agent/.echo-semantic/evidence/evidence.workspace-structure.md` — manifest/README/contract 边界 — 复用现有 discovery 证据。
- `echo-agent/.echo-semantic/audits/audit.workspace-architecture.contract-evidence.md` — 不存在 `tasks` feature 的直接反例 — 修复前审计基线。

## Todos

### derive-feature-table-contract

requirements:
- § 独立 Finding 前置
- § 文档事实源
- § 异常与边界场景
- § 文档校验扩展现有 contract

interfaces:
- consumes: Cargo metadata root package features map、双语 README feature table、现有 workspace metadata/Markdown helpers
- produces: manifest-derived public feature set 与双语 README exact-set contract

steps:

1. 扩展 metadata package 投影以读取 features map，并从 root `echo_agent` package 派生除 `default` 外的公开 feature set。
   verify: 当前 manifest 派生 27 个可列 feature，包含 `full`/`subagent`/`testing` 且不包含 `tasks`。
   expected: Expected set 来自 Cargo JSON，不在测试中重复硬编码全部 feature 名。

2. 从英文 `### Feature Flags` 与中文 `## Feature Flags` 的表格第一列提取 feature 名并与 expected set 比较。
   verify: section、header、重复、缺失与多余项分别形成明确失败；旧 README 运行时只报告 extra `tasks`。
   expected: Red 证明真实公开表漂移，不把后续 Feature Matrix 的自然语言词误识别为表项。

### correct-bilingual-feature-table

requirements:
- § 独立 Finding 前置
- § 文档事实源
- § 验收标准

interfaces:
- consumes: manifest-derived feature set、旧 README 上的有效 red、现有双语 feature 表
- produces: 与 Cargo metadata 精确对等的双语 feature 表和 Task core 说明

steps:

1. 只删除两份 README feature 表中的 `tasks` 行，保留所有真实 feature 的现有次序与描述。
   verify: 双语表分别包含 27 个唯一真实 feature，缺失/多余集合为空。
   expected: 用户不再复制 `features=["tasks"]` 并触发 Cargo resolution 错误。

2. 在每个 feature 表旁明确 Task APIs 是 framework core 且无需独立 feature。
   verify: 中英文含义对等，且 git diff 不改变 #80 demo34 命令、Cargo.toml、workspace topology、examples 或 SDK 路径。
   expected: feature 表和后文 Task API 说明不再自相矛盾。

### close-feature-table-finding

requirements:
- § 文档发布流
- § 独立 Finding 前置
- § 复用与实现约束
- § 验收标准

interfaces:
- consumes: red/green 日志、Cargo feature metadata、双语 README diff、Issue #79、最终工程验证与独立 review
- produces: feature table repair/verification Evidence、rereview Audit、resolved Finding #79 与更新后的治理计数

steps:

1. 刷新 workspace architecture map/behavior/framework docs asset 与 source snapshot，并记录 repair/verification Evidence。
   verify: Evidence 同时绑定 Cargo manifest、双语 README、documentation contract 和有效 red/green，且不宣称 #80 已修复。
   expected: feature topology 的事实源、消费者和验证链可从 Finding 追溯。

2. 在独立 review 无 blocker 后写 rereview Audit，将 `finding.public-feature-table-drift` 标为 resolved，并执行最终语义与 Issue 对账。
   verify: semantic strict/change-evidence、92/92 Finding-Issue 映射和 diff check 通过；92 个 Finding 中 72 open、20 resolved；Issue #79 仍为 OPEN。
   expected: 本地语义关闭与远端 Issue 交付状态严格分离。

## Decisions

- 当前 outcome 只修复 #79；#80 继续作为下一独立 outcome。
- Feature expected set 直接来自 Cargo metadata root package `features` keys，测试不解析 TOML 文本。
- `default` 不进入公开可启用 feature 表，`full` 保留；Task API 用表旁 prose 表达 core 可用性。
- 本切片不触发 SDK inventory 生成；最终只做 SDK 路径零漂移检查。
