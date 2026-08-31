---
schema_version: 3
slug: 2026-08-30-framework-boundary-and-bilingual-docs/plan
goal: 为 echo-agent-cli 建立 zh 主源、en 审阅翻译的 fail-closed parity
  foundation，并生成可执行的长期文档迁移清单。
ships: docs/README.md 的双语维护契约、docs/doc-parity-manifest.json
  的旧到新路径清单、scripts/check-docs-parity.mjs 的结构与审阅状态检查，以及可供后续内容迁移复用的稳定输出。
verify: parity checker 能在迁移前明确报告缺失的 zh/en 目录和配对文件，在满足 manifest 的配对与 reviewed
  条件时稳定通过；不修改现有 EKO wire、代码和运行数据。
design_ref: docs/supreme/specs/2026-08-30-framework-boundary-and-bilingual-docs/design.md
todos:
  - id: create-doc-parity-foundation
    files:
      - echo-agent-cli/docs/README.md
      - echo-agent-cli/docs/doc-parity-manifest.json
      - echo-agent-cli/scripts/check-docs-parity.mjs
      - echo-agent-cli/README.md
    summary: 建立双语文档契约、迁移清单和 fail-closed parity checker。
    verify: checker 能递归比较 zh/en 相对路径、检查 manifest identity/reviewed
      状态、拒绝缺失目录或未审阅英文，并输出稳定统计。
  - id: record-next-doc-migration
    files:
      - docs/supreme/specs/2026-08-30-framework-boundary-and-bilingual-docs/plans/plan_04_迁移EKO长期文档内容.md
    summary: 为完整文档移动、翻译和 website 接入建立下一阶段可执行计划。
    verify: 下一计划明确旧路径映射、翻译批次、删除目标、website sourcePath/hash 和最终 gates，不把运行数据清理混入。
artifact_id: plan:3cbb617c-f905-4815-8b1f-26d84ca98ebb
design_revision: sha256:7758c064650d16549ca5ca46899cb88e5ebbc0f630a089990c8f29d67ab6bb63
---
## Context

完整文档重组包含约 35 个长期页面和 25 个 ADR，必须先有机械 gate 和迁移清单，才能逐批完成 zh 主源与 en 审阅翻译。本计划先交付 parity 基础，不移动现有文档、不复制未审阅内容。

## Approach

- 用 doc-parity-manifest.json 固定旧路径、新相对路径、文档类型和翻译状态。
- checker 只读取文档目录与 manifest，递归比较路径、ADR identity、语言提示和 reviewed 状态；缺目录、缺配对或状态不一致时 fail closed。
- root docs/README.md 只描述语言入口与维护规则；现有根文档继续作为迁移前事实源，直到后续计划逐批切换。

## Global Constraints

- zh 是编辑主源，en 只有在语义等价且 reviewed=true 时才可发布。
- 不将当前混合中文/英文页面复制到 en 冒充翻译；未迁移文档必须在清单中标记 migration_pending。
- checker 不写文件、不访问运行数据、不改 wire/serde/TS/API，不引入数据库。
- 不删除 docs 根文档、.txt、.eko、空目录、缓存或任何运行证据；删除属于后续计划。

## Files

- Modify: `echo-agent-cli/docs/README.md` — 双语维护规则和迁移入口。
- Create: `echo-agent-cli/docs/doc-parity-manifest.json` — 旧/新路径、类型、翻译状态和 ADR identity 清单。
- Create: `echo-agent-cli/scripts/check-docs-parity.mjs` — fail-closed parity checker。
- Modify: `echo-agent-cli/README.md` — 增加 parity gate 入口和文档迁移状态说明。
- Create: `docs/supreme/specs/2026-08-30-framework-boundary-and-bilingual-docs/plans/plan_04_迁移EKO长期文档内容.md` — 下一阶段完整迁移计划。

## Reuse

- `echo-agent-cli/docs/README.md:3` 现有长期文档边界说明。
- `echo-agent-cli/docs/README.md:10` 现有导航作为 manifest 初始 inventory。
- `echo-agent-cli/docs/adr/` 现有 ADR 编号和标题作为 identity 输入。
- `echo-website/scripts/sync-docs.mjs:184` clean-source revision/hash 检查作为后续发布接入点。

## Todos

### create-doc-parity-foundation

requirements:
- § 双语文档信息架构
- § 翻译与 parity gate
- § 合并与迁移规则
- § 运行数据保护

interfaces:
- consumes: 当前 echo-agent-cli/docs 根文档、architecture 专题、ADR inventory。
- produces: doc-parity-manifest.json、check-docs-parity.mjs、root README maintenance contract。

steps:

1. 生成旧路径到目标相对路径的 manifest，标记 tutorial/reference/architecture/operations/adr/project-status 类型和 migration_pending/reviewed 状态。
   verify: manifest 覆盖当前 35 个长期 Markdown 页面及 25 个 ADR，路径和 ADR 编号无重复。
   expected: 后续翻译批次有单一、可审阅的迁移清单。
2. 实现 checker，递归比较 docs/zh 与 docs/en 的 Markdown 集合，检查 manifest identity、ADR 编号/状态/superseded、reviewed marker 和语言目录提示；缺失目标目录时输出 actionable failure。
   verify: 缺失配对、未审阅英文、ADR identity 漂移和语言错置均返回非零；当前迁移前 checkout 输出稳定的阻断统计。
   expected: parity gate 在内容迁移前就能阻断不完整发布。
3. 更新 root docs/README.md 与 CLI README，说明 zh/en 维护契约、checker 入口和当前迁移 pending 状态。
   verify: README 不再把根散落文档描述为最终双语发布结构，且没有引入旧路径兼容副本。
   expected: 贡献者能从根入口找到清单和 gate。

### record-next-doc-migration

requirements:
- § 目标行为
- § 合并与迁移规则
- § 候选交付结果与依赖
- § 验收标准

interfaces:
- consumes: 当前 Plan 03 manifest、docs inventory 和 website sync contract。
- produces: plan_04 具体迁移/翻译批次及删除清单。

steps:

1. 编写下一计划，按 architecture、reference/operations、ADR 和 project-status 批次列出 sourcePath、目标路径、翻译审阅和旧文件删除边界。
   verify: 每批可独立验证和停止，未审阅英文、运行数据和 website 生成资产有明确门禁。
   expected: 内容迁移无需重新盘点或猜测 owner。

## Diagram

```mermaid
flowchart LR
  I[Current docs inventory] --> M[Parity manifest]
  M --> G[Fail-closed checker]
  G --> B[Plan 04 translation batches]
  B --> W[Website reviewed projection]
```

## Decisions

- 本计划只建立 gate 和 inventory，不移动或复制未审阅文档。
- 完整 zh/en 内容迁移由下一计划按批次执行；.txt/.eko/空目录清理另行处理。