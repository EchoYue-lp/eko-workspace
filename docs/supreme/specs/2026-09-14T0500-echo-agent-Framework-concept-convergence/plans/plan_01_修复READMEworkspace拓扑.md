---
schema_version: 3
supersedes: null
slug: echo-agent-Framework-concept-convergence/修复READMEworkspace拓扑
goal: 让双语 README 的 workspace 拓扑与 Cargo metadata 的 11-package 事实保持一致
ships: 双语 README 以 Cargo metadata 为权威展示完整 11-package workspace，并明确 SDK
  protocol、SDK Host 与 learning consumer 的边界
verify: 旧 README 在 Cargo-derived topology contract 中稳定失败；修复后双语
  README、documentation contract、formatter、相关 Clippy/check、semantic
  strict/change-evidence、92/92 Issue 对账和独立复审全部通过
design_ref: docs/supreme/specs/2026-09-14T0500-echo-agent-Framework-concept-convergence/design.md
delivery_ref: docs/supreme/specs/2026-09-14T0500-echo-agent-Framework-concept-convergence/plans/delivery-map.md#repair-readme-workspace-topology
todos:
  - id: derive-workspace-topology-contract
    files:
      - echo-agent/echo-agent-learning/tests/documentation_contract.rs
    summary: 让现有文档合同从 Cargo metadata 校验双语 README 的完整 workspace package 集合
    verify: 合同以 root package 加十个 workspace members 为权威，并在 README 缺少任一 package 或
      package 分类错误时 fail closed
  - id: correct-bilingual-workspace-topology
    files:
      - echo-agent/README.md
      - echo-agent/README.zh.md
    summary: 补齐 SDK protocol 与 SDK Host 并修正 11-package 分类和计数
    verify: 两份 README 同构展示 8 个 framework/runtime package、2 个 SDK package 与 1 个
      learning package，且职责与实际依赖方向一致
  - id: close-workspace-topology-finding
    files:
      - echo-agent/.echo-semantic
    summary: "写入 repair、verification 与独立复审证据并只关闭 Finding #114"
    verify: "finding.workspace-topology-doc-drift 具备三类关闭 refs；其它 74 个 open Finding
      状态不变；Issue #114 保持 OPEN 等待远端 main"
artifact_id: plan:c32efa4b-b70e-40f1-854c-69728e1dd682
lifecycle: completed
design_revision: sha256:769b79d4599ee82922893785238c59a3131bab90b5bc250a6ae8f7aea5d0c875
---
## Approach

- 在新的 `fix/Echoyue/readme-workspace-topology` 分支从当前 `9d1f3f2b` 基线执行；修改前先完成 semantic-preflight，首个有效差异后执行 semantic-diff。
- 复用 `echo-agent-learning/tests/documentation_contract.rs` 的 root 路径、Markdown 读取和 `serde_json` 能力，通过 `cargo metadata --no-deps --format-version 1 --locked` 获取结构化 package/manifest 事实；不手写第二份 workspace manifest，也不引入 TOML parser 依赖。
- 先加入能在旧 README 上命中 SDK package 缺失与错误 `8+1` 计数的合同，保存有效 red；再同时修正中英文 workspace tree 和 highlights。
- README 将 11 个 package 明确分为：root `echo_agent` 加七个 split framework crate，共 8 个 framework/runtime package；`echo-sdk-protocol` 与 `echo-sdk-host` 共 2 个 SDK package；`echo-agent-learning` 为 1 个不发布 consumer。
- 独立 review 后写入 rereview Audit 并只把 `finding.workspace-topology-doc-drift` 标为 resolved；GitHub Issue #114 在本地提交进入远端 main 前保持 OPEN。

## Global Constraints

- Cargo manifests 与 `cargo metadata` 是 package topology 唯一权威；README 和测试只能消费，不得另建手写 package 清单作为第二权威。
- 本 Plan 只修复 workspace topology；不得顺手修复 #79 feature table、#80 example target 或其它 open Finding。
- Root facade、SDK protocol、SDK Host 与 learning consumer 的角色必须分开；不得把 SDK crates 画成 root crate 内部模块。
- 只使用 Subagent 术语，不引入第二套执行角色术语；不新增 public Rust API、feature、依赖、runtime 状态或 SDK identity。
- 英文与中文 README 的 package 集合、分组、数量和依赖方向必须语义对等。
- `echo-agent-cli` 与 `echo-website` 不修改；本切片只是 framework README 与其合同，未改变外部 SDK contract。
- Issue #114 已存在且 marker 唯一；发现新独立语义问题时先建立唯一 Issue/Finding，再决定是否进入当前范围。

## Files

- Modify: `echo-agent/echo-agent-learning/tests/documentation_contract.rs` — 增加 Cargo-derived workspace topology 文档合同。
- Modify: `echo-agent/README.md` — 补齐 SDK packages 并修正 workspace 分组/计数。
- Modify: `echo-agent/README.zh.md` — 与英文 README 对等修正 workspace 分组/计数。
- Modify: `echo-agent/.echo-semantic` — 刷新 workspace map/behavior/asset/source snapshot，增加 repair/verification/rereview 并关闭 Finding #114。

## Reuse

- `echo-agent/Cargo.toml` — workspace members 与 root package — package topology 唯一事实源。
- `echo-agent/echo-agent-learning/tests/documentation_contract.rs` — `collect_markdown_files`、root path、`serde_json` — 扩展现有文档合同而不新增测试框架。
- `echo-agent/.echo-semantic/maps/map.workspace-architecture.md` — `crate-dag-and-layering` — 已定位 #114、源码与关闭目标。
- `echo-agent/.echo-semantic/evidence/evidence.workspace-structure.md` — 11-package DAG 与 consumer 证据 — 复用已有 discovery 结论。
- `echo-agent/.echo-semantic/audits/audit.workspace-architecture.contract-evidence.md` — workspace topology 反例 — 作为修复前审计基线。

## Todos

### derive-workspace-topology-contract

requirements:
- § 独立 Finding 前置
- § 文档事实源
- § 异常与边界场景
- § 文档校验扩展现有 contract

interfaces:
- consumes: root `Cargo.toml`、Cargo metadata JSON、双语 README workspace 段落、现有 documentation contract helpers
- produces: Cargo-derived workspace package set 与双语 README topology fail-closed contract

steps:

1. 在现有 documentation contract 中读取 workspace root，并通过 Cargo metadata 取得 root package 与全部 workspace members 的 manifest/package 身份。
   verify: 派生集合精确包含 11 个 package，且 root、两个 SDK package和 learning consumer 可按 manifest path 区分。
   expected: 测试不依赖 README 自己声明的计数来建立 expected topology。

2. 校验英文和中文 README 的 workspace tree 覆盖每个 Cargo package，并校验 `8 framework/runtime + 2 SDK + 1 learning = 11` 的分组陈述。
   verify: 在旧 README 上执行目标 contract，缺少 `echo-sdk-protocol`/`echo-sdk-host` 或保留 `8+1` 时稳定失败。
   expected: red 明确指出 package 缺失或计数不一致，而不是编译错误或不相关失败。

### correct-bilingual-workspace-topology

requirements:
- § 架构分层
- § 独立 Finding 前置
- § 验收标准

interfaces:
- consumes: Cargo-derived package set、README 现有 workspace tree/highlights、上一 todo 的 failing contract
- produces: 语义对等且覆盖 11 package 的英文/中文 workspace topology

steps:

1. 在两份 README 的 workspace tree 中补充 `echo-sdk-protocol` 与 `echo-sdk-host`，说明前者拥有协议/生成合同、后者是使用 root facade 的运行时 Host。
   verify: package 名称、职责和树中位置与 Cargo dependency DAG 一致，root `src/` 仍明确为 facade/composition。
   expected: 读者可以区分 split framework crates、SDK boundary 和 learning consumer。

2. 将 `8 production + 1 teaching` 更正为 `8 framework/runtime + 2 SDK + 1 learning` 的 11-package 陈述，并保持中英文结构一致。
   verify: 上一 todo 的 topology contract 从 red 变 green，且 `git diff` 不包含 feature table、example command 或 runtime 改动。
   expected: #114 的直接反例消失，#79/#80 仍保持 open 且未被顺带修改。

### close-workspace-topology-finding

requirements:
- § 文档发布流
- § 独立 Finding 前置
- § 复用与实现约束
- § 验收标准

interfaces:
- consumes: red/green 日志、Cargo metadata、双语 README diff、Issue #114、最终工程验证与独立 review
- produces: workspace topology repair/verification Evidence、rereview Audit、resolved Finding #114 与更新后的治理计数

steps:

1. 刷新 workspace architecture map/behavior/framework docs asset 与 source snapshot，并记录 repair/verification Evidence。
   verify: Evidence 同时绑定 Cargo manifest、双语 README、documentation contract 和有效 red/green，且不宣称 #79/#80 已修复。
   expected: package topology 的事实源、消费者与验证链可以从 Finding 追溯。

2. 在独立 review 无 blocker 后写 rereview Audit，将 `finding.workspace-topology-doc-drift` 标为 resolved，并执行最终语义与 Issue 对账。
   verify: semantic strict/change-evidence、92/92 Finding-Issue 映射和 diff check 通过；92 个 Finding 中 73 open、19 resolved；Issue #114 仍为 OPEN。
   expected: 本地语义关闭与远端 Issue 交付状态严格分离。

## Decisions

- 当前 outcome 只修复 #114，因为它是后续 Architecture 文档的 package DAG 前置；#79 和 #80 保持各自独立 outcome。
- 测试消费 `cargo metadata` 的结构化输出，不使用字符串正则解析 Cargo.toml，也不新增依赖。
- 11-package 分类采用 `8 framework/runtime + 2 SDK + 1 learning`，其中 root facade 计入 framework/runtime。
- README topology 修复不触发 SDK inventory 生成；最终只做零漂移检查。
