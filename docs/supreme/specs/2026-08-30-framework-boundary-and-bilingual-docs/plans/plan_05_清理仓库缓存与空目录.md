---
schema_version: 3
slug: 2026-08-30-framework-boundary-and-bilingual-docs/plan
goal: 清理已确认可重建的工具缓存与无引用空源码占位目录，同时保留所有正式资产、审计证据和 EKO 运行数据。
ships: 一份按所有权分类的 hygiene 记录、精确删除的空缓存/占位目录，以及可复核的 tracked/runtime `.txt` 保留结论。
verify: tracked `.txt` 仍全部有正式用途或证据引用；`.eko` 与 soak scope 未被删除；列出的缓存和空源码目录在无活动
  owner 下消失；三仓和 superproject 工作树只包含本计划文档变化。
design_ref: docs/supreme/specs/2026-08-30-framework-boundary-and-bilingual-docs/design.md
todos:
  - id: classify-hygiene-scope
    files:
      - docs/2026-08-30-repository-hygiene.md
    summary: 记录 txt、运行数据、缓存和空目录的所有权分类与删除边界。
    verify: 记录 tracked txt、保留的 runtime/website assets、精确可清理目录和明确不触碰的 `.eko`/soak scope。
  - id: remove-safe-empty-directories
    files:
      - echo-agent/.playwright-mcp/
      - echo-agent/src/handoff/
      - echo-agent/src/notebook/
      - echo-agent-cli/.claude/worktrees/
      - echo-agent-cli/chrome-extension/
      - echo-agent-cli/echo-agent-app-core/src/tasks/pipelines/
      - echo-agent-cli/echo-agent-app-core/target/test-tmp/
      - echo-agent-cli/echo-agent-app-core/web-frontend/src/generated/
      - echo-agent-cli/evals/
      - echo-agent-cli/src/bin/
      - echo-agent-cli/web-frontend/src/components/changes/
      - echo-agent-cli/web-frontend/src/components/chat/tools/
      - echo-agent-cli/web-frontend/src/components/notebook/
      - echo-agent-cli/web-frontend/src/components/permissions/
      - echo-agent-cli/web-frontend/src/components/runtime/
      - echo-agent-cli/web-frontend/node_modules/.vite-temp/
      - echo-website/.worktrees/
    summary: 删除无活动 owner、无文件、可重建且无生成器要求的缓存和源码占位目录。
    verify: 删除目标在操作前为空且不属于 `.git`、`.eko`、soak root、正式 website public/dist 或
      node_modules 实体依赖；操作后路径不存在。
  - id: close-hygiene-status
    files:
      - docs/MASTER-PLAN.md
    summary: 在跨仓事实源记录 hygiene 完成状态和保留的运行数据边界。
    verify: MASTER-PLAN 明确 tracked/runtime `.txt` 保留、空目录清理完成、未授权 runtime scope
      清理仍待人工确认。
artifact_id: plan:c75b9b65-5f3a-40bb-b9b1-965d7097fcc1
design_revision: sha256:7758c064650d16549ca5ca46899cb88e5ebbc0f630a089990c8f29d67ab6bb63
---
## Context

文档迁移已完成，下一步只处理仓库卫生。`.txt` 同时承载 website 正式资产、审计证据和 EKO runtime trace；空目录同时包含缓存和源码占位，不能统一删除。本计划只清理明确安全的空目录并留下分类记录。

## Approach

- 先保存精确清单和所有权分类，再删除零文件、可重建、无活动 owner 的缓存/占位目录。
- tracked audit txt、website public/dist txt、`.eko` runtime/soak 数据全部保留。
- 删除空目录不作为代码行为变更；未来功能随首个真实文件重新创建。

## Global Constraints

- 不删除任何 `.eko`、soak/release evidence、用户 artifact、journal、trace 或 tracked audit txt。
- 不操作 `.git`、workspace 根、未知 owner 的目录或非空 node_modules package。
- 运行时 scope 无法确认归属时默认保留。
- 只提交 hygiene 记录和顶层状态更新；空目录删除本身不产生 Git diff。

## Files

- Create: `docs/2026-08-30-repository-hygiene.md` — 分类、保留和删除证据。
- Modify: `docs/MASTER-PLAN.md` — hygiene 当前状态和 residual。
- Delete: `echo-agent/.playwright-mcp/` — 已确认为空、可重建或无实现占位。
- Delete: `echo-agent/src/handoff/` — 已确认为空、可重建或无实现占位。
- Delete: `echo-agent/src/notebook/` — 已确认为空、可重建或无实现占位。
- Delete: `echo-agent-cli/.claude/worktrees/` — 已确认为空、可重建或无实现占位。
- Delete: `echo-agent-cli/chrome-extension/` — 已确认为空、可重建或无实现占位。
- Delete: `echo-agent-cli/echo-agent-app-core/src/tasks/pipelines/` — 已确认为空、可重建或无实现占位。
- Delete: `echo-agent-cli/echo-agent-app-core/target/test-tmp/` — 已确认为空、可重建或无实现占位。
- Delete: `echo-agent-cli/echo-agent-app-core/web-frontend/src/generated/` — 已确认为空、可重建或无实现占位。
- Delete: `echo-agent-cli/evals/` — 已确认为空、可重建或无实现占位。
- Delete: `echo-agent-cli/src/bin/` — 已确认为空、可重建或无实现占位。
- Delete: `echo-agent-cli/web-frontend/src/components/changes/` — 已确认为空、可重建或无实现占位。
- Delete: `echo-agent-cli/web-frontend/src/components/chat/tools/` — 已确认为空、可重建或无实现占位。
- Delete: `echo-agent-cli/web-frontend/src/components/notebook/` — 已确认为空、可重建或无实现占位。
- Delete: `echo-agent-cli/web-frontend/src/components/permissions/` — 已确认为空、可重建或无实现占位。
- Delete: `echo-agent-cli/web-frontend/src/components/runtime/` — 已确认为空、可重建或无实现占位。
- Delete: `echo-agent-cli/web-frontend/node_modules/.vite-temp/` — 已确认为空、可重建或无实现占位。
- Delete: `echo-website/.worktrees/` — 已确认为空、可重建或无实现占位。

## Reuse

- `docs/supreme/specs/2026-08-30-framework-boundary-and-bilingual-docs/design.md § .txt 与空目录清理设计` — 所有权和生命周期规则。
- 三个子仓库 `.gitignore` — 缓存与生成目录忽略边界。

## Todos

### classify-hygiene-scope

requirements:
- § .txt 与空目录清理设计
- § 运行数据保护
- § 空目录

interfaces:
- consumes: tracked/untracked txt、空目录、ignore 规则和运行根。
- produces: docs/2026-08-30-repository-hygiene.md。

steps:

1. 记录 tracked txt、website generated txt、`.eko` runtime/soak txt、缓存和空源码目录的数量、路径模式与所有权。
   verify: 每类都有保留/清理结论，未知 scope 明确标记为保留。
   expected: 删除动作有可复核的精确输入。

### remove-safe-empty-directories

requirements:
- § .txt 与空目录清理设计
- § 空目录
- § 清理时仍有活动 owner

interfaces:
- consumes: hygiene scope record 和当前活动进程/owner 检查。
- produces: 精确目标目录的本地清理结果。

steps:

1. 确认目标为空、非 `.git`/`.eko`/soak、无活动 owner 且无生成器要求。
   verify: 每个目标 `find` 文件数为 0，且无真实消费者。
   expected: 删除范围不包含用户数据或正式资产。
2. 删除精确缓存/占位目录，保留所有非空目录。
   verify: 目标路径消失，tracked 文件、runtime scope 和 website assets 数量不变。
   expected: 空目录清理完成且可由工具/首个文件重建。

### close-hygiene-status

requirements:
- § 验收标准
- § 运行数据保护

interfaces:
- consumes: 分类记录和清理结果。
- produces: 顶层 hygiene 状态。

steps:

1. 更新 MASTER-PLAN，区分已清理缓存、保留运行数据和待人工裁决的 soak scope。
   verify: 文档不声称删除未授权数据，tracked txt 仍有用途或证据引用。
   expected: 后续清理可从事实源继续。

## Decisions

- 正式 website `.txt`、tracked audit txt、`.eko` runtime/soak scope 均保留。
- 空目录删除采用精确路径，不使用宽泛递归删除。
- 未确认 owner 的运行 scope 不在本计划内。